use crate::dns::engine::{encode_name_labels_vec, parse_qname};
use once_cell::sync::Lazy;
use std::collections::HashMap;
type CacheKey = (String, u16);
type CacheValue = (Vec<u8>, Instant);
use std::io::{Read, Write};
use std::net::{TcpStream, UdpSocket};
use std::str;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const ROOT_SERVERS: [&str; 13] = [
    "198.41.0.4",
    "199.9.14.201",
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

fn send_recv(sock: &UdpSocket, addr: &str, msg: &[u8]) -> Option<Vec<u8>> {
    let target = format!("{}:53", addr);
    let _ = sock.send_to(msg, &target).ok()?;
    let mut buf = [0u8; 4096];
    match sock.recv_from(&mut buf) {
        Ok((amt, _)) => Some(buf[..amt].to_vec()),
        Err(_) => None,
    }
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
    let target = format!("{}:53", addr);
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

pub fn resolve(name: &str, qtype: u16, max_depth: usize) -> Option<Vec<u8>> {
    if max_depth == 0 {
        return None;
    }
    let sock = UdpSocket::bind(("0.0.0.0", 0)).ok()?;
    let _ = sock.set_read_timeout(Some(Duration::from_secs(2)));

    static CACHE: Lazy<Mutex<HashMap<CacheKey, CacheValue>>> =
        Lazy::new(|| Mutex::new(HashMap::new()));

    if let Ok(c) = CACHE.lock()
        && let Some((data, exp)) = c.get(&(name.to_string(), qtype))
        && Instant::now() < *exp
    {
        return Some(data.clone());
    }

    let mut servers: Vec<String> = ROOT_SERVERS.iter().map(|s| s.to_string()).collect();
    let mut qname = name.to_string();
    for _round in 0..6 {
        let qid = rand::random::<u16>();
        let req = build_query(qid, &qname, qtype);
        let mut next_servers: Vec<String> = Vec::new();
        for srv in &servers {
            let mut resp_opt = send_recv(&sock, srv, &req);
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
                                let exp = Instant::now() + Duration::from_secs((*ttl).into());
                                c.insert((name.to_string(), qtype), (resp.clone(), exp));
                            }
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
                let mut after_auth = auth_pos;
                if let Some(last) = authority_rrs.last() {
                    after_auth = last.1 + last.2;
                }
                let additional_rrs = parse_rrs(&resp, after_auth, arcount).unwrap_or_default();
                let (ns_names, glue_ips) =
                    extract_ns_and_glue(&resp, &authority_rrs, &additional_rrs);
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
                    servers = next_servers.clone();
                    break;
                }
            }
        }
    }
    None
}
