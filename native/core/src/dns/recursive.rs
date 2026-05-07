use crate::dns::engine::{encode_name_labels_vec, parse_qname};
use once_cell::sync::Lazy;
use rand::seq::SliceRandom;
use std::collections::HashMap;
type CacheKey = (String, u16);
type CacheValue = (Vec<u8>, Instant);
use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, TcpStream, UdpSocket};
use std::path::Path;
use std::str;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

const ROOT_SERVERS: [&str; 13] = [
    "198.41.0.4",      // A
    "170.247.170.2",   // B (Updated)
    "192.33.4.12",     // C
    "199.7.91.13",     // D
    "192.203.230.10",  // E
    "192.5.5.241",     // F
    "192.112.36.4",    // G
    "198.97.190.53",   // H
    "192.36.148.17",   // I
    "192.58.128.30",   // J
    "193.0.14.129",    // K
    "199.7.83.42",     // L
    "202.12.27.33",    // M
];


const DEFAULT_UDP_ATTEMPTS_PER_SERVER: usize = 4;
const DEFAULT_UDP_ATTEMPT_TIMEOUT_MS: u64 = 2500;
const ROOT_CACHE_TTL_SECS: u64 = 86400;
const TLD_DELEGATION_MIN_TTL_SECS: u64 = 3600;
const TLD_DELEGATION_MAX_TTL_SECS: u64 = 86400;
const DEFAULT_RECURSION_ROUNDS: usize = 8;
const MAX_QUERY_CACHE_ENTRIES: usize = 10_000;
const MAX_ROOT_CACHE_ENTRIES: usize = 256;
const MAX_DELEGATION_CACHE_ENTRIES: usize = 1024;
const MAX_UPSTREAM_SERVERS_PER_ROUND: usize = 8;
const MAX_CONCURRENT_RESOLVES: usize = 128;

static ACTIVE_RESOLVES: Lazy<AtomicUsize> = Lazy::new(|| AtomicUsize::new(0));

fn env_u64(name: &str, default: u64) -> u64 {
    env::var(name)
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(default)
}

fn env_usize(name: &str, default: usize) -> usize {
    env::var(name)
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(default)
}

fn udp_attempts_per_server() -> usize {
    env_usize("HACKFLARE_DNS_UDP_ATTEMPTS", DEFAULT_UDP_ATTEMPTS_PER_SERVER).max(1)
}

fn udp_attempt_timeout() -> Duration {
    Duration::from_millis(env_u64(
        "HACKFLARE_DNS_UDP_TIMEOUT_MS",
        DEFAULT_UDP_ATTEMPT_TIMEOUT_MS,
    ))
}

fn recursion_round_limit() -> usize {
    env_usize("HACKFLARE_DNS_RECURSION_ROUNDS", DEFAULT_RECURSION_ROUNDS).max(1)
}

fn recursion_debug_enabled() -> bool {
    env::var("HACKFLARE_DNS_RECURSION_DEBUG")
        .ok()
        .map(|v| {
            let val = v.trim().to_ascii_lowercase();
            val == "1" || val == "true" || val == "yes" || val == "on"
        })
        .unwrap_or(false)
}

fn debug_log(msg: &str) {
    if recursion_debug_enabled() {
        eprintln!("[hackflare:dns:recursive] {}", msg);
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

fn prune_query_cache(cache: &mut HashMap<CacheKey, CacheValue>) {
    let now = Instant::now();
    cache.retain(|_, (_, exp)| now < *exp);
    while cache.len() > MAX_QUERY_CACHE_ENTRIES {
        if let Some(key) = cache.keys().next().cloned() {
            cache.remove(&key);
        } else {
            break;
        }
    }
}

fn prune_root_cache(cache: &mut HashMap<String, RootCacheValue>) {
    let now = Instant::now();
    cache.retain(|_, (_, _, exp)| now < *exp);
    while cache.len() > MAX_ROOT_CACHE_ENTRIES {
        if let Some(key) = cache.keys().next().cloned() {
            cache.remove(&key);
        } else {
            break;
        }
    }
}

fn prune_delegation_cache(cache: &mut HashMap<String, DelegationCacheValue>) {
    let now = Instant::now();
    cache.retain(|_, (_, exp)| now < *exp);
    while cache.len() > MAX_DELEGATION_CACHE_ENTRIES {
        if let Some(key) = cache.keys().next().cloned() {
            cache.remove(&key);
        } else {
            break;
        }
    }
}

fn socket_target(addr: &str) -> String {
    if addr.contains(':') && !addr.starts_with('[') {
        format!("[{}]:53", addr)
    } else {
        format!("{}:53", addr)
    }
}

fn build_query(id: u16, qname: &str, qtype: u16) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&id.to_be_bytes());

    out.extend_from_slice(&0x0100u16.to_be_bytes());
    out.extend_from_slice(&1u16.to_be_bytes());
    out.extend_from_slice(&0u16.to_be_bytes());
    out.extend_from_slice(&0u16.to_be_bytes());
    out.extend_from_slice(&0u16.to_be_bytes());
    out.extend_from_slice(&encode_name_labels_vec(qname));
    out.extend_from_slice(&qtype.to_be_bytes());
    out.extend_from_slice(&1u16.to_be_bytes());
    out
}

fn parse_root_hints(content: &str) -> Vec<String> {
    let mut ips = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with(';') || trimmed.starts_with('#') {
            continue;
        }
        for token in trimmed.split_whitespace() {
            if let Ok(ip) = token.parse::<Ipv4Addr>() {
                ips.push(ip.to_string());
            }
        }
    }
    ips.sort();
    ips.dedup();
    ips
}

fn root_hints_content() -> String {
    let mut out = String::from("; Auto-generated root hints by Hackflare\n");
    for ip in ROOT_SERVERS {
        out.push_str(ip);
        out.push('\n');
    }
    out
}

fn root_hint_candidate_paths() -> Vec<String> {
    let mut paths: Vec<String> = Vec::new();
    if let Ok(path) = env::var("HACKFLARE_ROOT_HINTS_FILE")
        && !path.trim().is_empty()
    {
        paths.push(path);
    }
    paths.push("/etc/hackflare/root.hints".to_string());
    paths.push("/etc/bind/db.root".to_string());
    paths.push("/etc/named.root".to_string());
    paths.push("./root.hints".to_string());
    paths.push("/tmp/hackflare/root.hints".to_string());
    paths
}

fn try_create_root_hints_file(path: &str) -> bool {
    let p = Path::new(path);
    if p.exists() {
        return false;
    }
    if let Some(parent) = p.parent()
        && fs::create_dir_all(parent).is_err()
    {
        return false;
    }
    fs::write(p, root_hints_content()).is_ok()
}

fn load_root_hint_servers() -> Vec<String> {
    let paths = root_hint_candidate_paths();

    for path in &paths {
        if let Ok(content) = fs::read_to_string(path) {
            let parsed = parse_root_hints(&content);
            if !parsed.is_empty() {
                return parsed;
            }
        }
    }

    for path in &paths {
        if try_create_root_hints_file(path)
            && let Ok(content) = fs::read_to_string(path)
        {
            let parsed = parse_root_hints(&content);
            if !parsed.is_empty() {
                return parsed;
            }
        }
    }

    ROOT_SERVERS.iter().map(|s| s.to_string()).collect()
}

fn tld_from_name(name: &str) -> Option<String> {
    name.split('.')
        .rev()
        .find(|label| !label.is_empty())
        .map(|s| s.to_ascii_lowercase())
}

fn clamp_tld_ttl(ttl_secs: u64) -> u64 {
    ttl_secs.clamp(TLD_DELEGATION_MIN_TTL_SECS, TLD_DELEGATION_MAX_TTL_SECS)
}

fn response_matches_expected(
    resp: &[u8],
    expected_id: u16,
    expected_qname: &str,
    expected_qtype: u16,
) -> bool {
    if resp.len() < 12 {
        return false;
    }
    let id = u16::from_be_bytes([resp[0], resp[1]]);
    if id != expected_id {
        return false;
    }
    let flags = u16::from_be_bytes([resp[2], resp[3]]);
    if flags & 0x8000 == 0 {
        return false;
    }
    let qdcount = u16::from_be_bytes([resp[4], resp[5]]);
    if qdcount == 0 {
        return false;
    }
    let (qname, pos) = match parse_qname(resp, 12) {
        Some(v) => v,
        None => return false,
    };
    if !qname.eq_ignore_ascii_case(expected_qname) {
        return false;
    }
    if pos + 4 > resp.len() {
        return false;
    }
    let qtype = u16::from_be_bytes([resp[pos], resp[pos + 1]]);
    qtype == expected_qtype
}

fn send_recv(
    sock: &UdpSocket,
    addr: &str,
    msg: &[u8],
    qid: u16,
    qname: &str,
    qtype: u16,
) -> Option<Vec<u8>> {
    let target = socket_target(addr);
    let expected_ip: IpAddr = addr.parse().ok()?;
    let mut buf = [0u8; 4096];
    let attempts = udp_attempts_per_server();
    let timeout = udp_attempt_timeout();

    for _ in 0..attempts {
        let _ = sock.send_to(msg, &target).ok()?;
        let deadline = Instant::now() + timeout;

        loop {
            if Instant::now() >= deadline {
                break;
            }
            match sock.recv_from(&mut buf) {
                Ok((amt, src)) => {
                    if src.port() != 53 || src.ip() != expected_ip {
                        continue;
                    }
                    let candidate = &buf[..amt];
                    if response_matches_expected(candidate, qid, qname, qtype) {
                        return Some(candidate.to_vec());
                    }
                }
                Err(ref e)
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut =>
                {
                    break;
                }
                Err(_) => break,
            }
        }
    }

    None
}

fn parse_rrs(buf: &[u8], mut pos: usize, count: usize) -> Option<Vec<(u16, usize, usize, u32)>> {
    let mut out = Vec::new();
    for _ in 0..count {
        let (_name, new_pos) = parse_qname(buf, pos)?;
        pos = new_pos;
        if pos + 10 > buf.len() {
            return None;
        }
        let rtype = u16::from_be_bytes([buf[pos], buf[pos + 1]]);
        let _class = u16::from_be_bytes([buf[pos + 2], buf[pos + 3]]);
        let ttl = u32::from_be_bytes([buf[pos + 4], buf[pos + 5], buf[pos + 6], buf[pos + 7]]);
        let rdlen = u16::from_be_bytes([buf[pos + 8], buf[pos + 9]]) as usize;
        pos += 10;
        if pos + rdlen > buf.len() {
            return None;
        }
        out.push((rtype, pos, rdlen, ttl));
        pos += rdlen;
    }
    Some(out)
}

fn tcp_send_recv(addr: &str, msg: &[u8]) -> Option<Vec<u8>> {
    let target = socket_target(addr);
    let sockaddr = target.parse().ok()?;
    let mut stream = TcpStream::connect_timeout(&sockaddr, Duration::from_secs(3)).ok()?;
    stream.set_read_timeout(Some(Duration::from_secs(4))).ok()?;
    stream
        .set_write_timeout(Some(Duration::from_secs(4)))
        .ok()?;
    let len = (msg.len() as u16).to_be_bytes();
    if stream.write_all(&len).is_err() {
        return None;
    }
    if stream.write_all(msg).is_err() {
        return None;
    }
    let mut lenbuf = [0u8; 2];
    if stream.read_exact(&mut lenbuf).is_err() {
        return None;
    }
    let rlen = u16::from_be_bytes(lenbuf) as usize;
    let mut buf = vec![0u8; rlen];
    if stream.read_exact(&mut buf).is_err() {
        return None;
    }
    Some(buf)
}

fn extract_ns_and_glue(
    buf: &[u8],
    authority_rrs: &[(u16, usize, usize, u32)],
    additional_rrs: &[(u16, usize, usize, u32)],
) -> (Vec<String>, Vec<String>) {
    let mut ns_names: Vec<String> = Vec::new();
    let mut glue_ips: Vec<String> = Vec::new();
    for (rtype, rpos, _rdlen, _ttl) in authority_rrs {
        if *rtype == 2
            && let Some((name, _)) = parse_qname(buf, *rpos)
        {
            ns_names.push(name);
        }
    }
    for (rtype, rpos, rdlen, _ttl) in additional_rrs {
        if *rtype == 1 && *rdlen == 4 {
            let ip = format!(
                "{}.{}.{}.{}",
                buf[*rpos],
                buf[*rpos + 1],
                buf[*rpos + 2],
                buf[*rpos + 3]
            );
            glue_ips.push(ip);
        } else if *rtype == 28
            && *rdlen == 16
            && let Ok(ipv6) = <[u8; 16]>::try_from(&buf[*rpos..*rpos + 16])
        {
            glue_ips.push(std::net::Ipv6Addr::from(ipv6).to_string());
        }
    }
    (ns_names, glue_ips)
}

type RootCacheValue = (Vec<String>, Vec<String>, Instant);
type DelegationCacheValue = (Vec<String>, Instant);

pub fn resolve(name: &str, qtype: u16, max_depth: usize) -> Option<Vec<u8>> {
    if max_depth == 0 {
        return None;
    }
    let _resolve_guard = acquire_resolve_slot()?;
    let sock = UdpSocket::bind(("0.0.0.0", 0)).ok()?;
    let _ = sock.set_read_timeout(Some(udp_attempt_timeout()));

    static CACHE: Lazy<Mutex<HashMap<CacheKey, CacheValue>>> =
        Lazy::new(|| Mutex::new(HashMap::new()));
    static ROOT_CACHE: Lazy<Mutex<HashMap<String, RootCacheValue>>> =
        Lazy::new(|| Mutex::new(HashMap::new()));
    static DELEGATION_CACHE: Lazy<Mutex<HashMap<String, DelegationCacheValue>>> =
        Lazy::new(|| Mutex::new(HashMap::new()));
    static ROOT_HINTS: Lazy<Vec<String>> = Lazy::new(load_root_hint_servers);

    if let Ok(mut roots) = ROOT_CACHE.lock()
        && {
            prune_root_cache(&mut roots);
            !roots.contains_key("__root__")
        }
        && !ROOT_HINTS.is_empty()
    {
        let exp = Instant::now() + Duration::from_secs(ROOT_CACHE_TTL_SECS);
        roots.insert(
            "__root__".to_string(),
            (Vec::new(), ROOT_HINTS.clone(), exp),
        );
    }

    if let Ok(mut c) = CACHE.lock()
        && {
            prune_query_cache(&mut c);
            true
        }
        && let Some((data, exp)) = c.get(&(name.to_string(), qtype))
        && Instant::now() < *exp
    {
        return Some(data.clone());
    }

    let requested_tld = tld_from_name(name);

    let mut servers: Vec<String> = if let Some(tld) = requested_tld.as_ref()
        && let Ok(delegations) = DELEGATION_CACHE.lock()
        && {
            drop(delegations);
            true
        }
        && let Ok(mut delegations) = DELEGATION_CACHE.lock()
        && {
            prune_delegation_cache(&mut delegations);
            true
        }
        && let Some((cached, exp)) = delegations.get(tld)
        && Instant::now() < *exp
        && !cached.is_empty()
    {
        cached.clone()
    } else if let Ok(roots) = ROOT_CACHE.lock()
        && let Some((_ns_names, glue_ips, exp)) = roots.get("__root__")
        && Instant::now() < *exp
        && !glue_ips.is_empty()
    {
        glue_ips.clone()
    } else {
        ROOT_HINTS.clone()
    };
    if servers.len() > MAX_UPSTREAM_SERVERS_PER_ROUND {
        servers.truncate(MAX_UPSTREAM_SERVERS_PER_ROUND);
    }
    let mut qname = name.to_string();
    let mut tried_root_fallback = false;
    for _round in 0..recursion_round_limit() {
        let qid = rand::random::<u16>();
        let req = build_query(qid, &qname, qtype);
        let mut next_servers: Vec<String> = Vec::new();
        let mut round_servers = servers.clone();
        round_servers.shuffle(&mut rand::rng());
        if round_servers.len() > MAX_UPSTREAM_SERVERS_PER_ROUND {
            round_servers.truncate(MAX_UPSTREAM_SERVERS_PER_ROUND);
        }
        for srv in &round_servers {
            let mut resp_opt = send_recv(&sock, srv, &req, qid, &qname, qtype);
            if resp_opt.is_none() {
                resp_opt = tcp_send_recv(srv, &req);
            }
            if let Some(mut resp) = resp_opt {
                let flags = if resp.len() >= 4 {
                    u16::from_be_bytes([resp[2], resp[3]])
                } else {
                    0
                };
                if flags & 0x0200 != 0
                    && let Some(tcp_resp) = tcp_send_recv(srv, &req)
                {
                    resp = tcp_resp;
                }
                if resp.len() < 12 {
                    debug_log(&format!("short response from {} while resolving {}", srv, qname));
                    continue;
                }
                let ancount = u16::from_be_bytes([resp[6], resp[7]]) as usize;
                let nscount = u16::from_be_bytes([resp[8], resp[9]]) as usize;
                let arcount = u16::from_be_bytes([resp[10], resp[11]]) as usize;
                let mut pos = 12usize;
                let (_qn, p2) = parse_qname(&resp, pos)?;
                pos = p2 + 4;
                if ancount > 0
                    && let Some(ans_rrs) = parse_rrs(&resp, pos, ancount)
                {
                    let mut min_ttl: Option<u32> = None;
                    for (rtype, rpos, _rdlen, ttl) in &ans_rrs {
                        if *rtype == qtype {
                            if let Ok(mut c) = CACHE.lock() {
                                prune_query_cache(&mut c);
                                let exp = Instant::now() + Duration::from_secs((*ttl).into());
                                c.insert((name.to_string(), qtype), (resp.clone(), exp));
                            }
                            debug_log(&format!("resolved {} type {} via {}", name, qtype, srv));
                            return Some(resp.clone());
                        }
                        if let Some(mt) = min_ttl {
                            if *ttl < mt {
                                min_ttl = Some(*ttl);
                            }
                        } else {
                            min_ttl = Some(*ttl);
                        }
                        if *rtype == 5
                            && let Some((cname, _)) = parse_qname(&resp, *rpos)
                        {
                            qname = cname;
                            next_servers.clear();
                            break;
                        }
                    }
                }
                let mut after_pos = pos;
                if ancount > 0
                    && let Some(list) = parse_rrs(&resp, pos, ancount)
                {
                    after_pos = list.last().map(|(_, p, rd, _)| p + rd).unwrap_or(pos);
                }
                let auth_pos = after_pos;
                let authority_rrs = parse_rrs(&resp, auth_pos, nscount).unwrap_or_default();
                let referral_ttl_secs = authority_rrs
                    .iter()
                    .map(|(_, _, _, ttl)| *ttl as u64)
                    .min()
                    .unwrap_or(ROOT_CACHE_TTL_SECS);
                let mut after_auth = auth_pos;
                if let Some(last) = authority_rrs.last() {
                    after_auth = last.1 + last.2;
                }
                let additional_rrs = parse_rrs(&resp, after_auth, arcount).unwrap_or_default();
                let (ns_names, glue_ips) =
                    extract_ns_and_glue(&resp, &authority_rrs, &additional_rrs);

                if _round == 0 && !ns_names.is_empty()
                    && let Ok(mut roots) = ROOT_CACHE.lock() {
                        let exp = Instant::now() + Duration::from_secs(ROOT_CACHE_TTL_SECS);
                        roots.insert(
                            "__root__".to_string(),
                            (ns_names.clone(), glue_ips.clone(), exp),
                        );
                    }

                if !glue_ips.is_empty() {
                    for ip in glue_ips {
                        next_servers.push(ip);
                    }
                } else {
                    for nsname in ns_names {
                        if let Some(ip_resp) = resolve(&nsname, 1, max_depth - 1)
                            && ip_resp.len() >= 12
                        {
                            let an = u16::from_be_bytes([ip_resp[6], ip_resp[7]]) as usize;
                            if an > 0 {
                                let mut p = 12usize;
                                let (_q, p2) =
                                    parse_qname(&ip_resp, p).unwrap_or(("".to_string(), p));
                                p = p2 + 4;
                                if let Some(a_rrs) = parse_rrs(&ip_resp, p, an) {
                                    for (rt, rpos, rdlen, _) in a_rrs {
                                        if rt == 1 && rdlen == 4 {
                                            let ip = format!(
                                                "{}.{}.{}.{}",
                                                ip_resp[rpos],
                                                ip_resp[rpos + 1],
                                                ip_resp[rpos + 2],
                                                ip_resp[rpos + 3]
                                            );
                                            next_servers.push(ip);
                                        }
                                    }
                                }
                            }
                        }
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
                        && let Ok(mut delegations) = DELEGATION_CACHE.lock()
                    {
                        prune_delegation_cache(&mut delegations);
                        let ttl = clamp_tld_ttl(referral_ttl_secs);
                        let exp = Instant::now() + Duration::from_secs(ttl);
                        delegations.insert(tld.clone(), (next_servers.clone(), exp));
                    }
                    servers = next_servers.clone();
                    break;
                }
            } else {
                debug_log(&format!("no response from {} while resolving {}", srv, qname));
            }
        }

        if next_servers.is_empty() && !tried_root_fallback && !servers.is_empty() && servers != *ROOT_HINTS {
            tried_root_fallback = true;
            servers = ROOT_HINTS.clone();
            continue;
        }

        if next_servers.is_empty() && servers == *ROOT_HINTS {
            tried_root_fallback = true;
        }
    }
    debug_log(&format!("resolution failed for {} type {}", name, qtype));
    None
}
