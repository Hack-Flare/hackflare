use crate::dns::DnsConfig;
use rand::seq::SliceRandom;
use std::net::UdpSocket;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::cache::{self, ROOT_CACHE_TTL_SECS};
use super::hints;
use super::message::{build_query, clamp_tld_ttl, extract_ns_and_glue, parse_rrs, tld_from_name, DnsHeader};
use super::transport::{send_recv, tcp_send_recv};

const MAX_UPSTREAM_SERVERS_PER_ROUND: usize = 8;
const MAX_CONCURRENT_RESOLVES: usize = 128;

static ACTIVE_RESOLVES: std::sync::LazyLock<AtomicUsize> =
    std::sync::LazyLock::new(|| AtomicUsize::new(0));

static ROOT_HINTS: std::sync::LazyLock<Vec<String>> =
    std::sync::LazyLock::new(hints::load_root_hint_servers);

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

pub fn resolve(name: &str, qtype: u16, config: &DnsConfig) -> Option<Vec<u8>> {
    resolve_internal(name, qtype, config.recursion_rounds, config)
}

#[allow(
    clippy::too_many_lines,
    clippy::items_after_statements,
    clippy::used_underscore_binding,
    clippy::similar_names
)]
fn resolve_internal(
    name: &str,
    qtype: u16,
    max_depth: usize,
    config: &DnsConfig,
) -> Option<Vec<u8>> {
    if max_depth == 0 {
        return None;
    }
    let _resolve_guard = acquire_resolve_slot()?;
    let sock = UdpSocket::bind(("0.0.0.0", 0)).ok()?;
    let _ = sock.set_read_timeout(Some(config.udp_timeout));

    cache::CACHE.seed_root_cache(&ROOT_HINTS, ROOT_CACHE_TTL_SECS);

    if let Some(data) = cache::CACHE.get_query(name, qtype) {
        return Some(data);
    }

    let requested_tld = tld_from_name(name);

    let mut servers: Vec<String> = requested_tld
        .as_ref()
        .and_then(|tld| cache::CACHE.get_delegation(tld))
        .or_else(|| cache::CACHE.get_root_glue())
        .unwrap_or_else(|| ROOT_HINTS.clone());

    if servers.len() > MAX_UPSTREAM_SERVERS_PER_ROUND {
        servers.truncate(MAX_UPSTREAM_SERVERS_PER_ROUND);
    }
    let mut qname = name.to_string();
    let mut tried_root_fallback = false;
    for _round in 0..max_depth {
        let qid = rand::random::<u16>();
        let req = build_query(qid, &qname, qtype);
        let mut next_servers: Vec<String> = Vec::new();
        let mut round_servers = servers.clone();
        round_servers.shuffle(&mut rand::rng());
        if round_servers.len() > MAX_UPSTREAM_SERVERS_PER_ROUND {
            round_servers.truncate(MAX_UPSTREAM_SERVERS_PER_ROUND);
        }
        for srv in &round_servers {
            let mut resp_opt = send_recv(&sock, srv, &req, qid, &qname, qtype, config);
            if resp_opt.is_none() {
                resp_opt = tcp_send_recv(srv, &req);
            }
            if let Some(mut resp) = resp_opt {
                let truncated = resp.len() >= 12
                    && DnsHeader::from_wire(&resp).is_some_and(|h| h.is_truncated());
                if truncated
                    && let Some(tcp_resp) = tcp_send_recv(srv, &req)
                {
                    resp = tcp_resp;
                }
                if resp.len() < 12 {
                    debug_log(
                        &format!("short response from {srv} while resolving {qname}"),
                        config,
                    );
                    continue;
                }
                let Some(header) = DnsHeader::from_wire(&resp) else {
                    continue;
                };
                let ancount = header.ancount as usize;
                let nscount = header.nscount as usize;
                let arcount = header.arcount as usize;
                let mut pos = 12usize;
                let (_qn, p2) = crate::dns::wire::parse_qname(&resp, pos)?;
                pos = p2 + 4;
                if ancount > 0
                    && let Some(ans_rrs) = parse_rrs(&resp, pos, ancount)
                {
                    let mut min_ttl: Option<u32> = None;
                    for rr in &ans_rrs {
                        if rr.rtype == qtype {
                            cache::CACHE.put_query(name, qtype, resp.clone(), rr.ttl);
                            debug_log(&format!("resolved {name} type {qtype} via {srv}"), config);
                            return Some(resp.clone());
                        }
                        if let Some(mt) = min_ttl {
                            if rr.ttl < mt {
                                min_ttl = Some(rr.ttl);
                            }
                        } else {
                            min_ttl = Some(rr.ttl);
                        }
                        if rr.rtype == 5
                            && let Some((cname, _)) = crate::dns::wire::parse_qname(&resp, rr.pos)
                        {
                            qname = cname;
                            next_servers.clear();
                            break;
                        }
                    }
                }
                let auth_pos = if ancount > 0
                    && let Some(list) = parse_rrs(&resp, pos, ancount)
                {
                    list.last().map_or(pos, |rr| rr.pos + rr.rdlen)
                } else {
                    pos
                };
                let authority_rrs = parse_rrs(&resp, auth_pos, nscount).unwrap_or_default();
                let referral_ttl_secs = authority_rrs
                    .iter()
                    .map(|rr| u64::from(rr.ttl))
                    .min()
                    .unwrap_or(ROOT_CACHE_TTL_SECS);
                let after_auth = authority_rrs
                    .last()
                    .map_or(auth_pos, |last| last.pos + last.rdlen);
                let additional_rrs = parse_rrs(&resp, after_auth, arcount).unwrap_or_default();
                let (ns_names, glue_ips) =
                    extract_ns_and_glue(&resp, &authority_rrs, &additional_rrs);

                if _round == 0 && !ns_names.is_empty() {
                    cache::CACHE.update_root_cache(&ns_names, &glue_ips, ROOT_CACHE_TTL_SECS);
                }

                if glue_ips.is_empty() {
                    for nsname in ns_names {
                        if let Some(ip_resp) =
                            resolve_internal(&nsname, 1, max_depth - 1, config)
                            && ip_resp.len() >= 12
                        {
                            let an = u16::from_be_bytes([ip_resp[6], ip_resp[7]]) as usize;
                            if an > 0 {
                                let mut p = 12usize;
                                let Some((_q, p2)) =
                                    crate::dns::wire::parse_qname(&ip_resp, p)
                                else {
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
                                            next_servers.push(ip);
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    for ip in glue_ips {
                        next_servers.push(ip);
                    }
                }
                if !next_servers.is_empty() {
                    next_servers.sort();
                    next_servers.dedup();
                    if next_servers.len() > MAX_UPSTREAM_SERVERS_PER_ROUND {
                        next_servers.truncate(MAX_UPSTREAM_SERVERS_PER_ROUND);
                    }
                    if _round == 0
                        && let Some(tld) = requested_tld.as_ref()
                    {
                        let ttl = clamp_tld_ttl(referral_ttl_secs);
                        cache::CACHE.put_delegation(tld, &next_servers, ttl);
                    }
                    servers.clone_from(&next_servers);
                    break;
                }
            } else {
                debug_log(
                    &format!("no response from {srv} while resolving {qname}"),
                    config,
                );
            }
        }

        if next_servers.is_empty()
            && !tried_root_fallback
            && !servers.is_empty()
            && servers != *ROOT_HINTS
        {
            tried_root_fallback = true;
            servers.clone_from(&ROOT_HINTS);
            continue;
        }

        if next_servers.is_empty() && servers == *ROOT_HINTS {
            tried_root_fallback = true;
        }
    }
    debug_log(
        &format!("resolution failed for {name} type {qtype}"),
        config,
    );
    None
}
