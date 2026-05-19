use crate::dns::DnsConfig;
use rand::seq::SliceRandom;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::cache::{self, ROOT_CACHE_TTL_SECS};
use super::error::ResolveError;
use super::hints;
use super::message::{
    DnsHeader, RecordInfo, build_query, clamp_tld_ttl, extract_ns_and_glue, parse_rrs,
    tld_from_name,
};
use super::transport::{UdpTransport, tcp_send_recv};

const MAX_UPSTREAM_SERVERS_PER_ROUND: usize = 8;
const MAX_CONCURRENT_RESOLVES: usize = 128;

static ACTIVE_RESOLVES: std::sync::LazyLock<AtomicUsize> =
    std::sync::LazyLock::new(|| AtomicUsize::new(0));

static ROOT_HINTS: std::sync::LazyLock<hints::RootHints> =
    std::sync::LazyLock::new(hints::RootHints::load);

fn debug_log(msg: &str, config: &DnsConfig) {
    if config.recursion_debug {
        eprintln!("[hackflare:dns:recursive] {msg}");
    }
}

struct ResolveGuard;

impl Drop for ResolveGuard {
    fn drop(&mut self) {
        ACTIVE_RESOLVES.fetch_sub(1, Ordering::AcqRel);
    }
}

fn acquire_resolve_slot() -> Option<ResolveGuard> {
    let mut current = ACTIVE_RESOLVES.load(Ordering::Acquire);
    loop {
        if current >= MAX_CONCURRENT_RESOLVES {
            return None;
        }
        match ACTIVE_RESOLVES.compare_exchange(
            current,
            current + 1,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) => return Some(ResolveGuard),
            Err(next) => current = next,
        }
    }
}

pub fn resolve(name: &str, qtype: u16, config: &DnsConfig) -> Result<Vec<u8>, ResolveError> {
    resolve_internal(name, qtype, config.recursion_rounds, config)
}

fn select_upstream_servers(requested_tld: &Option<String>) -> Vec<String> {
    requested_tld
        .as_ref()
        .and_then(|tld| cache::CACHE.get_delegation(tld))
        .or_else(|| cache::CACHE.get_root_glue())
        .unwrap_or_else(|| ROOT_HINTS.servers().to_vec())
}

fn try_query(
    transport: &UdpTransport,
    srv: &str,
    req: &[u8],
    qid: u16,
    qname: &str,
    qtype: u16,
    config: &DnsConfig,
) -> Option<Vec<u8>> {
    let mut resp = transport.send_recv(srv, req, qid, qname, qtype, config);
    if resp.is_none() {
        resp = tcp_send_recv(srv, req);
    }
    if let Some(ref r) = resp
        && r.len() >= 12
        && DnsHeader::from_wire(r).is_some_and(|h| h.is_truncated())
    {
        resp = tcp_send_recv(srv, req);
    }
    resp
}

struct ParsedResponse {
    _header: DnsHeader,
    answers: Vec<RecordInfo>,
    authorities: Vec<RecordInfo>,
    additionals: Vec<RecordInfo>,
}

fn parse_sections(resp: &[u8]) -> Option<ParsedResponse> {
    let header = DnsHeader::from_wire(resp)?;
    let ancount = header.ancount as usize;
    let nscount = header.nscount as usize;
    let arcount = header.arcount as usize;
    let mut pos = 12usize;
    let (_qn, p2) = crate::dns::wire::parse_qname(resp, pos)?;
    pos = p2 + 4;

    let answers = if ancount > 0 {
        parse_rrs(resp, pos, ancount).unwrap_or_default()
    } else {
        Vec::new()
    };

    let auth_pos = answers.last().map_or(pos, |rr| rr.pos + rr.rdlen);

    let authorities = parse_rrs(resp, auth_pos, nscount).unwrap_or_default();
    let after_auth = authorities
        .last()
        .map_or(auth_pos, |last| last.pos + last.rdlen);
    let additionals = parse_rrs(resp, after_auth, arcount).unwrap_or_default();

    Some(ParsedResponse {
        _header: header,
        answers,
        authorities,
        additionals,
    })
}

fn check_direct_answer(answers: &[RecordInfo], qtype: u16) -> Option<u32> {
    for rr in answers {
        if rr.rtype == qtype {
            return Some(rr.ttl);
        }
    }
    None
}

fn find_cname(answers: &[RecordInfo], resp: &[u8]) -> Option<String> {
    for rr in answers {
        if rr.rtype == 5
            && let Some((cname, _)) = crate::dns::wire::parse_qname(resp, rr.pos)
        {
            return Some(cname);
        }
    }
    None
}

fn resolve_ns_ips(ns_names: &[String], max_depth: usize, config: &DnsConfig) -> Vec<String> {
    let mut ips = Vec::new();
    for nsname in ns_names {
        let Ok(ip_resp) = resolve_internal(nsname, 1, max_depth - 1, config) else {
            continue;
        };
        if ip_resp.len() >= 12 {
            let an = u16::from_be_bytes([ip_resp[6], ip_resp[7]]) as usize;
            if an == 0 {
                continue;
            }
            let mut p = 12usize;
            let Some((_q, p2)) = crate::dns::wire::parse_qname(&ip_resp, p) else {
                continue;
            };
            p = p2 + 4;
            if let Some(a_rrs) = parse_rrs(&ip_resp, p, an) {
                for rr in a_rrs {
                    if rr.rtype == 1 && rr.rdlen == 4 {
                        let ip = format!(
                            "{}.{}.{}.{}",
                            ip_resp[rr.pos],
                            ip_resp[rr.pos + 1],
                            ip_resp[rr.pos + 2],
                            ip_resp[rr.pos + 3]
                        );
                        ips.push(ip);
                    }
                }
            }
        }
    }
    ips
}

fn resolve_internal(
    name: &str,
    qtype: u16,
    max_depth: usize,
    config: &DnsConfig,
) -> Result<Vec<u8>, ResolveError> {
    if max_depth == 0 {
        return Err(ResolveError::ResolutionFailed);
    }
    let _resolve_guard = acquire_resolve_slot().ok_or(ResolveError::TooManyConcurrentResolves)?;
    let transport = UdpTransport::bind(config.udp_timeout)
        .ok_or_else(|| ResolveError::BindFailed("udp socket bind".to_string()))?;

    cache::CACHE.seed_root_cache(ROOT_HINTS.servers(), ROOT_CACHE_TTL_SECS);

    if let Some(data) = cache::CACHE.get_query(name, qtype) {
        return Ok(data);
    }

    let requested_tld = tld_from_name(name);
    let mut servers = select_upstream_servers(&requested_tld);
    servers.truncate(MAX_UPSTREAM_SERVERS_PER_ROUND);

    let mut qname = name.to_string();
    let mut tried_root_fallback = false;

    for round in 0..max_depth {
        let qid = rand::random::<u16>();
        let req = build_query(qid, &qname, qtype);
        let mut found_servers = false;

        let mut round_servers = servers.clone();
        round_servers.shuffle(&mut rand::rng());
        round_servers.truncate(MAX_UPSTREAM_SERVERS_PER_ROUND);

        for srv in &round_servers {
            let resp = try_query(&transport, srv, &req, qid, &qname, qtype, config);
            let Some(resp) = resp else {
                debug_log(
                    &format!("no response from {srv} while resolving {qname}"),
                    config,
                );
                continue;
            };
            if resp.len() < 12 {
                debug_log(
                    &format!("short response from {srv} while resolving {qname}"),
                    config,
                );
                continue;
            }
            let Some(parsed) = parse_sections(&resp) else {
                continue;
            };

            if let Some(ttl) = check_direct_answer(&parsed.answers, qtype) {
                cache::CACHE.put_query(name, qtype, resp.clone(), ttl);
                debug_log(&format!("resolved {name} type {qtype} via {srv}"), config);
                return Ok(resp);
            }

            if let Some(cname) = find_cname(&parsed.answers, &resp) {
                qname = cname;
                break;
            }

            let (ns_names, glue_ips) =
                extract_ns_and_glue(&resp, &parsed.authorities, &parsed.additionals);

            if round == 0 && !ns_names.is_empty() {
                cache::CACHE.update_root_cache(&ns_names, &glue_ips, ROOT_CACHE_TTL_SECS);
            }

            let mut next_servers: Vec<String> = if !glue_ips.is_empty() {
                glue_ips
            } else if !ns_names.is_empty() {
                resolve_ns_ips(&ns_names, max_depth, config)
            } else {
                continue;
            };

            next_servers.sort();
            next_servers.dedup();
            next_servers.truncate(MAX_UPSTREAM_SERVERS_PER_ROUND);

            if round == 0
                && let Some(tld) = requested_tld.as_ref()
            {
                let referral_ttl = parsed
                    .authorities
                    .iter()
                    .map(|rr| u64::from(rr.ttl))
                    .min()
                    .unwrap_or(ROOT_CACHE_TTL_SECS);
                let ttl = clamp_tld_ttl(referral_ttl);
                cache::CACHE.put_delegation(tld, &next_servers, ttl);
            }
            servers = next_servers;
            found_servers = true;
            break;
        }

        if found_servers {
            continue;
        }

        if !tried_root_fallback && !servers.is_empty() && servers.as_slice() != ROOT_HINTS.servers()
        {
            tried_root_fallback = true;
            servers = ROOT_HINTS.servers().to_vec();
            continue;
        }

        if servers.as_slice() == ROOT_HINTS.servers() {
            tried_root_fallback = true;
        }
    }

    debug_log(
        &format!("resolution failed for {name} type {qtype}"),
        config,
    );
    Err(ResolveError::ResolutionFailed)
}
