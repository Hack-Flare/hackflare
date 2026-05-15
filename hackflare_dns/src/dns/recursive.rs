use crate::dns::DnsConfig;
use crate::dns::engine::{encode_name_labels_vec, parse_qname};

use postgres::NoTls;
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
    "198.41.0.4",
    "170.247.170.2",
    "192.33.4.12",
    "199.7.91.13",
    "192.203.230.10",
    "192.5.5.241",
    "192.112.36.4",
    "198.97.190.53",
    "192.36.148.17",
    "192.58.128.30",
    "193.0.14.129",
    "199.7.83.42",
    "202.12.27.33",
];

const ROOT_CACHE_TTL_SECS: u64 = 86400;
const TLD_DELEGATION_MIN_TTL_SECS: u64 = 3600;
const TLD_DELEGATION_MAX_TTL_SECS: u64 = 86400;
const MAX_QUERY_CACHE_ENTRIES: usize = 10_000;
const MAX_ROOT_CACHE_ENTRIES: usize = 256;
const MAX_DELEGATION_CACHE_ENTRIES: usize = 1024;
const MAX_UPSTREAM_SERVERS_PER_ROUND: usize = 8;
const MAX_CONCURRENT_RESOLVES: usize = 128;

static ACTIVE_RESOLVES: std::sync::LazyLock<AtomicUsize> =
    std::sync::LazyLock::new(|| AtomicUsize::new(0));

fn udp_attempts_per_server(config: &DnsConfig) -> usize {
    config.udp_attempts.max(1)
}

const fn udp_attempt_timeout(config: &DnsConfig) -> Duration {
    config.udp_timeout
}

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
        format!("[{addr}]:53")
    } else {
        format!("{addr}:53")
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
    load_root_hint_servers_internal(None)
}

fn load_root_hint_servers_internal(custom_path: Option<&std::path::PathBuf>) -> Vec<String> {
    // Try loading from database first (will use DATABASE_URL from config if available)
    if let Some(db_hints) = load_root_hints_from_db()
        && !db_hints.is_empty()
    {
        return db_hints;
    }

    // Try custom path if provided
    if let Some(path) = custom_path
        && let Ok(content) = fs::read_to_string(path)
    {
        let parsed = parse_root_hints(&content);
        if !parsed.is_empty() {
            return parsed;
        }
    }

    // Try standard file locations
    let paths = root_hint_candidate_paths();

    for path in &paths {
        if let Ok(content) = fs::read_to_string(path) {
            let parsed = parse_root_hints(&content);
            if !parsed.is_empty() {
                return parsed;
            }
        }
    }

    // Try creating root hints file if needed
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

    // Fall back to hardcoded root servers
    ROOT_SERVERS.iter().map(|&s| s.to_string()).collect()
}

fn load_root_hints_from_db() -> Option<Vec<String>> {
    let db_url = env::var("DATABASE_URL").ok()?;
    let mut client = postgres::Client::connect(&db_url, NoTls).ok()?;

    // Try to query root hints; if table doesn't exist, return None and fall back
    let result = client.query(
        "SELECT ip_address FROM dns_root_hints ORDER BY ip_address",
        &[],
    );

    let rows = result.ok()?;
    let mut hints: Vec<String> = rows
        .iter()
        .filter_map(|row| {
            let ip: String = row.get(0);
            if ip.is_empty() { None } else { Some(ip) }
        })
        .collect();

    hints.sort();
    hints.dedup();

    if hints.is_empty() { None } else { Some(hints) }
}

// Initialize the dns_root_hints table if it doesn't exist and populate it with the default root servers.
// This is a utility function that can be called at startup to ensure root hints are available in the database.
//
// # Errors
//
// Returns an error if the database connection fails or any query fails.
#[allow(clippy::missing_errors_doc)]
pub fn ensure_root_hints_in_db(db_url: &str) -> Result<(), String> {
    let mut client = postgres::Client::connect(db_url, NoTls)
        .map_err(|e| format!("Failed to connect to database: {e}"))?;

    // Create table if it doesn't exist
    client
        .execute(
            "CREATE TABLE IF NOT EXISTS dns_root_hints (
                id SERIAL PRIMARY KEY,
                ip_address VARCHAR(45) NOT NULL UNIQUE,
                created_at TIMESTAMP DEFAULT NOW(),
                updated_at TIMESTAMP DEFAULT NOW()
            )",
            &[],
        )
        .map_err(|e| format!("Failed to create dns_root_hints table: {e}"))?;

    // Check if table is empty
    let count_result = client
        .query_one("SELECT COUNT(*) FROM dns_root_hints", &[])
        .map_err(|e| format!("Failed to count root hints: {e}"))?;

    let count: i64 = count_result.get(0);

    if count == 0 {
        // Populate with default root servers
        for ip in &ROOT_SERVERS {
            client
                .execute(
                    "INSERT INTO dns_root_hints (ip_address) VALUES ($1) ON CONFLICT (ip_address) DO NOTHING",
                    &[ip],
                )
                .map_err(|e| format!("Failed to insert root hint: {e}"))?;
        }
    }

    Ok(())
}

fn tld_from_name(name: &str) -> Option<String> {
    name.split('.')
        .rev()
        .find(|label| !label.is_empty())
        .map(str::to_ascii_lowercase)
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
    let qdcount = u16::from_be_bytes([resp[4], resp[5]]);
    if qdcount != 1 {
        return false;
    }
    let mut pos = 12usize;
    let Some((qname, p2)) = parse_qname(resp, pos) else {
        return false;
    };
    pos = p2;
    if qname.trim_end_matches('.') != expected_qname.trim_end_matches('.') {
        return false;
    }
    if pos + 4 > resp.len() {
        return false;
    }
    let qtype = u16::from_be_bytes([resp[pos], resp[pos + 1]]);
    qtype == expected_qtype
}

fn parse_rrs(buf: &[u8], mut pos: usize, count: usize) -> Option<Vec<(u16, usize, usize, u32)>> {
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        let (_, p2) = parse_qname(buf, pos)?;
        pos = p2;
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
    let len = u16::try_from(msg.len()).unwrap_or(0).to_be_bytes();
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

fn send_recv(
    sock: &UdpSocket,
    addr: &str,
    msg: &[u8],
    qid: u16,
    qname: &str,
    qtype: u16,
    config: &DnsConfig,
) -> Option<Vec<u8>> {
    let target = socket_target(addr);
    let expected_ip: IpAddr = addr.parse().ok()?;
    let mut buf = [0u8; 4096];
    let attempts = udp_attempts_per_server(config);
    let timeout = udp_attempt_timeout(config);

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
    let _ = sock.set_read_timeout(Some(udp_attempt_timeout(config)));

    static CACHE: std::sync::LazyLock<Mutex<HashMap<CacheKey, CacheValue>>> =
        std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));
    static ROOT_CACHE: std::sync::LazyLock<Mutex<HashMap<String, RootCacheValue>>> =
        std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));
    static DELEGATION_CACHE: std::sync::LazyLock<Mutex<HashMap<String, DelegationCacheValue>>> =
        std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));
    static ROOT_HINTS: std::sync::LazyLock<Vec<String>> =
        std::sync::LazyLock::new(load_root_hint_servers);

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
                    debug_log(
                        &format!("short response from {srv} while resolving {qname}"),
                        config,
                    );
                    continue;
                }
                let ancount = u16::from_be_bytes([resp[6], resp[7]]) as usize;
                let nscount = u16::from_be_bytes([resp[8], resp[9]]) as usize;
                #[allow(clippy::similar_names)]
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
                            debug_log(&format!("resolved {name} type {qtype} via {srv}"), config);
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
                let auth_pos = if ancount > 0
                    && let Some(list) = parse_rrs(&resp, pos, ancount)
                {
                    list.last().map_or(pos, |(_, p, rd, _)| p + rd)
                } else {
                    pos
                };
                let authority_rrs = parse_rrs(&resp, auth_pos, nscount).unwrap_or_default();
                let referral_ttl_secs = authority_rrs
                    .iter()
                    .map(|(_, _, _, ttl)| u64::from(*ttl))
                    .min()
                    .unwrap_or(ROOT_CACHE_TTL_SECS);
                let after_auth = authority_rrs
                    .last()
                    .map_or(auth_pos, |last| last.1 + last.2);
                let additional_rrs = parse_rrs(&resp, after_auth, arcount).unwrap_or_default();
                let (ns_names, glue_ips) =
                    extract_ns_and_glue(&resp, &authority_rrs, &additional_rrs);

                if _round == 0
                    && !ns_names.is_empty()
                    && let Ok(mut roots) = ROOT_CACHE.lock()
                {
                    let exp = Instant::now() + Duration::from_secs(ROOT_CACHE_TTL_SECS);
                    roots.insert(
                        "__root__".to_string(),
                        (ns_names.clone(), glue_ips.clone(), exp),
                    );
                }

                if glue_ips.is_empty() {
                    for nsname in ns_names {
                        if let Some(ip_resp) = resolve_internal(&nsname, 1, max_depth - 1, config)
                            && ip_resp.len() >= 12
                        {
                            let an = u16::from_be_bytes([ip_resp[6], ip_resp[7]]) as usize;
                            if an > 0 {
                                let mut p = 12usize;
                                let (_q, p2) =
                                    parse_qname(&ip_resp, p).unwrap_or_else(|| (String::new(), p));
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
                    #[allow(clippy::used_underscore_binding)]
                    if _round == 0
                        && let Some(tld) = requested_tld.as_ref()
                        && let Ok(mut delegations) = DELEGATION_CACHE.lock()
                    {
                        prune_delegation_cache(&mut delegations);
                        let ttl = clamp_tld_ttl(referral_ttl_secs);
                        let exp = Instant::now() + Duration::from_secs(ttl);
                        delegations.insert(tld.clone(), (next_servers.clone(), exp));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helper_functions_cover_basic_recursion_utility_logic() {
        assert_eq!(tld_from_name("WWW.Example.COM."), Some("com".to_string()));
        assert_eq!(clamp_tld_ttl(10), TLD_DELEGATION_MIN_TTL_SECS);
        assert_eq!(clamp_tld_ttl(999_999), TLD_DELEGATION_MAX_TTL_SECS);
        assert_eq!(socket_target("192.0.2.1"), "192.0.2.1:53");
        assert_eq!(socket_target("2001:db8::1"), "[2001:db8::1]:53");

        let mut response = build_query(0x1234, "example.com", 1);
        response[2] |= 0x80;
        assert!(response_matches_expected(
            &response,
            0x1234,
            "example.com",
            1
        ));
        assert!(!response_matches_expected(
            &response,
            0x9999,
            "example.com",
            1
        ));
    }

    #[test]
    fn root_hint_parser_deduplicates_ips() {
        let hints = parse_root_hints("; comment\n198.41.0.4 198.41.0.4\n170.247.170.2\n");
        assert_eq!(
            hints,
            vec!["170.247.170.2".to_string(), "198.41.0.4".to_string()]
        );
    }
}
