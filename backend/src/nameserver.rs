use std::env;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::Arc;

use tokio::net::UdpSocket;
use tracing::{error, info, warn};

use crate::domain::dns::{DnsService, RecordType};

pub async fn run_dns_server(bind_addr: SocketAddr, dns: Arc<DnsService>) -> std::io::Result<()> {
    let socket = UdpSocket::bind(bind_addr).await?;
    info!(%bind_addr, "dns nameserver listening");

    let mut buffer = [0_u8; 4096];

    loop {
        let (size, peer) = socket.recv_from(&mut buffer).await?;
        let payload = &buffer[..size];

        match build_response(payload, &dns).await {
            Some(resp) => {
                if let Err(err) = socket.send_to(&resp, peer).await {
                    error!(%peer, error = %err, "failed sending dns response");
                }
            }
            None => {
                warn!(%peer, "no response generated for packet");
            }
        }
    }
}

fn recursion_enabled_from_env() -> bool {
    env::var("HACKFLARE_DNS_RECURSION_ENABLED")
        .ok()
        .map(|v| {
            let v = v.trim().to_ascii_lowercase();
            v == "1" || v == "true" || v == "yes" || v == "on"
        })
        .unwrap_or(false)
}

fn recursion_max_depth_from_env() -> usize {
    env::var("HACKFLARE_DNS_RECURSION_MAX_DEPTH")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .map(|v| v.clamp(1, 16))
        .unwrap_or(6)
}

async fn build_response(req: &[u8], dns: &DnsService) -> Option<Vec<u8>> {
    // follow same defensive parsing strategy as native/core: return SERVFAIL when malformed
    if req.len() < 12 {
        let id = if req.len() >= 2 {
            u16::from_be_bytes([req[0], req[1]])
        } else {
            0
        };
        let req_flags = if req.len() >= 4 {
            u16::from_be_bytes([req[2], req[3]])
        } else {
            0
        };
        return Some(build_servfail(
            id,
            req_flags,
            recursion_enabled_from_env(),
            &[],
        ));
    }

    let id = u16::from_be_bytes([req[0], req[1]]);
    let qdcount = u16::from_be_bytes([req[4], req[5]]);
    let ancount = u16::from_be_bytes([req[6], req[7]]);
    let nscount = u16::from_be_bytes([req[8], req[9]]);
    let arcount = u16::from_be_bytes([req[10], req[11]]);
    if qdcount == 0 {
        let req_flags = u16::from_be_bytes([req[2], req[3]]);
        return Some(build_servfail(
            id,
            req_flags,
            recursion_enabled_from_env(),
            &req[12..],
        ));
    }

    let mut pos = 12usize;
    let (qname, new_pos) = parse_qname(req, pos)?;
    pos = new_pos;
    if pos + 4 > req.len() {
        let req_flags = u16::from_be_bytes([req[2], req[3]]);
        return Some(build_servfail(
            id,
            req_flags,
            recursion_enabled_from_env(),
            &req[12..pos],
        ));
    }
    let qtype = u16::from_be_bytes([req[pos], req[pos + 1]]);
    let _qclass = u16::from_be_bytes([req[pos + 2], req[pos + 3]]);

    // parse additional for EDNS OPT
    let mut client_edns_size: usize = 0;
    let mut client_do: bool = false;
    let pos_after_question = pos + 4;
    let mut rr_pos = pos_after_question;
    let skip_rrs = ancount as usize + nscount as usize;
    for _ in 0..skip_rrs {
        if rr_pos >= req.len() {
            break;
        }
        if let Some((_name, newp)) = parse_qname(req, rr_pos) {
            rr_pos = newp;
        } else {
            break;
        }
        if rr_pos + 10 > req.len() {
            break;
        }
        let rdlen = u16::from_be_bytes([req[rr_pos + 8], req[rr_pos + 9]]) as usize;
        rr_pos += 10 + rdlen;
    }
    for _ in 0..(arcount as usize) {
        if rr_pos >= req.len() {
            break;
        }
        if let Some((_name, newp)) = parse_qname(req, rr_pos) {
            rr_pos = newp;
        } else {
            break;
        }
        if rr_pos + 10 > req.len() {
            break;
        }
        let typ = u16::from_be_bytes([req[rr_pos], req[rr_pos + 1]]);
        let class = u16::from_be_bytes([req[rr_pos + 2], req[rr_pos + 3]]);
        let ttl = u32::from_be_bytes([
            req[rr_pos + 4],
            req[rr_pos + 5],
            req[rr_pos + 6],
            req[rr_pos + 7],
        ]);
        let rdlen = u16::from_be_bytes([req[rr_pos + 8], req[rr_pos + 9]]) as usize;
        rr_pos += 10;
        if typ == 41 {
            client_edns_size = class as usize;
            client_do = (ttl & 0x8000) != 0;
        }
        rr_pos += rdlen;
    }

    let qtype_str = map_qtype(qtype);

    let req_flags = u16::from_be_bytes([req[2], req[3]]);

    let mut reverse_name: Option<String> = None;
    let mut is_ip_literal = false;
    if let Ok(v4) = qname.parse::<Ipv4Addr>() {
        let o = v4.octets();
        reverse_name = Some(format!("{}.{}.{}.{}.in-addr.arpa", o[3], o[2], o[1], o[0]));
        is_ip_literal = true;
    } else if let Ok(v6) = qname.parse::<Ipv6Addr>() {
        let octs = v6.octets();
        let mut nibbles: Vec<String> = Vec::new();
        for b in octs.iter().rev() {
            nibbles.push(format!("{:x}", b & 0x0f));
            nibbles.push(format!("{:x}", (b >> 4) & 0x0f));
        }
        let rev = nibbles.join(".");
        reverse_name = Some(format!("{}.ip6.arpa", rev));
        is_ip_literal = true;
    }

    let mut recs = if qtype == 255 {
        // ANY
        dns.find_records(&qname, None)
    } else if qtype_str.is_empty() {
        Vec::new()
    } else {
        dns.find_records(
            &qname,
            Some(match qtype_str {
                "A" => RecordType::A,
                "AAAA" => RecordType::Aaaa,
                "CNAME" => RecordType::Cname,
                "TXT" => RecordType::Txt,
                "NS" => RecordType::Ns,
                "PTR" => RecordType::Ptr,
                "MX" => RecordType::Mx,
                _ => {
                    return Some(build_servfail(
                        id,
                        req_flags,
                        recursion_enabled_from_env(),
                        &req[12..pos + 4],
                    ));
                }
            }),
        )
    };

    if is_ip_literal
        && recs.is_empty()
        && let Some(rn) = reverse_name.as_ref()
    {
        let ptrs = dns.find_records(rn, Some(RecordType::Ptr));
        if !ptrs.is_empty() {
            recs = ptrs;
        } else {
            let mut r = build_nxdomain_with_soa(
                id,
                req_flags,
                recursion_enabled_from_env(),
                &req[12..pos + 4],
            );
            if client_edns_size > 0 {
                append_opt(&mut r, client_edns_size, client_do);
            }
            return Some(r);
        }
    }

    let label_count = qname.split('.').filter(|label| !label.is_empty()).count();
    if recs.is_empty() && reverse_name.is_none() && label_count < 2 {
        let mut r = build_nxdomain_with_soa(
            id,
            req_flags,
            recursion_enabled_from_env(),
            &req[12..pos + 4],
        );
        if client_edns_size > 0 {
            append_opt(&mut r, client_edns_size, client_do);
        }
        return Some(r);
    }

    if recs.is_empty() {
        // If the DNS service has a zone for this name, return NoError (empty answer)
        if dns.has_zone_for_name(&qname) {
            let mut resp: Vec<u8> = Vec::new();
            resp.extend_from_slice(&id.to_be_bytes());
            let mut flags: u16 = 0x8000;
            flags |= req_flags & 0x0100;
            if recursion_enabled_from_env() {
                flags |= 0x0080;
            }
            resp.extend_from_slice(&flags.to_be_bytes());
            resp.extend_from_slice(&1u16.to_be_bytes());
            resp.extend_from_slice(&0u16.to_be_bytes());
            resp.extend_from_slice(&0u16.to_be_bytes());
            resp.extend_from_slice(&0u16.to_be_bytes());
            resp.extend_from_slice(&req[12..pos + 4]);
            if client_edns_size > 0 {
                append_opt(&mut resp, client_edns_size, client_do);
            }
            return Some(resp);
        }

        if !recursion_enabled_from_env() {
            let mut r = build_nxdomain_with_soa(
                id,
                req_flags,
                recursion_enabled_from_env(),
                &req[12..pos + 4],
            );
            if client_edns_size > 0 {
                append_opt(&mut r, client_edns_size, client_do);
            }
            return Some(r);
        }

        if let Some(mut recursive_resp) =
            crate::recursive::resolve(&qname, qtype, recursion_max_depth_from_env()).await
        {
            if client_edns_size > 0 {
                append_opt(&mut recursive_resp, client_edns_size, client_do);
            }
            return Some(recursive_resp);
        }

        let mut r = build_servfail(
            id,
            req_flags,
            recursion_enabled_from_env(),
            &req[12..pos + 4],
        );
        if client_edns_size > 0 {
            append_opt(&mut r, client_edns_size, client_do);
        }
        return Some(r);
    }

    let mut resp: Vec<u8> = Vec::new();
    resp.extend_from_slice(&id.to_be_bytes());

    let mut flags: u16 = 0x8000;
    if !recs.is_empty() {
        flags |= 0x0400;
    }
    flags |= req_flags & 0x0100;
    if recursion_enabled_from_env() {
        flags |= 0x0080;
    }
    let ar_out = if client_edns_size > 0 { 1u16 } else { 0u16 };
    resp.extend_from_slice(&flags.to_be_bytes());
    resp.extend_from_slice(&1u16.to_be_bytes());
    resp.extend_from_slice(&(recs.len() as u16).to_be_bytes());
    resp.extend_from_slice(&0u16.to_be_bytes());
    resp.extend_from_slice(&ar_out.to_be_bytes());

    // copy question
    resp.extend_from_slice(&req[12..pos + 4]);

    for ans in recs {
        resp.extend_from_slice(&0xC00C_u16.to_be_bytes());
        let rtype = map_qtype_to_num(&ans.record_type);
        resp.extend_from_slice(&rtype.to_be_bytes());
        resp.extend_from_slice(&1u16.to_be_bytes());
        resp.extend_from_slice(&ans.ttl.to_be_bytes());

        if let Some(rdata) = encode_rdata(&ans.record_type, &ans.value) {
            resp.extend_from_slice(&((rdata.len() as u16).to_be_bytes()));
            resp.extend_from_slice(&rdata);
        } else {
            resp.extend_from_slice(&0u16.to_be_bytes());
        }
    }

    if client_edns_size > 0 {
        append_opt(&mut resp, client_edns_size, client_do);
    }

    Some(resp)
}

fn map_qtype(q: u16) -> &'static str {
    match q {
        1 => "A",
        2 => "NS",
        5 => "CNAME",
        6 => "SOA",
        12 => "PTR",
        13 => "HINFO",
        15 => "MX",
        16 => "TXT",
        28 => "AAAA",
        29 => "LOC",
        33 => "SRV",
        43 => "DS",
        44 => "SSHFP",
        46 => "RRSIG",
        47 => "NSEC",
        48 => "DNSKEY",
        50 => "NSEC3",
        52 => "TLSA",
        64 => "SVCB",
        65 => "HTTPS",
        255 => "ANY",
        257 => "CAA",
        _ => "",
    }
}

fn map_qtype_to_num(rt: &RecordType) -> u16 {
    match rt {
        RecordType::A => 1,
        RecordType::Ns => 2,
        RecordType::Aaaa => 28,
        RecordType::Cname => 5,
        RecordType::Ptr => 12,
        RecordType::Mx => 15,
        RecordType::Txt => 16,
    }
}

fn parse_qname(buf: &[u8], mut pos: usize) -> Option<(String, usize)> {
    let mut labels: Vec<String> = Vec::new();
    let mut jumped = false;
    let mut orig_pos = pos;
    let mut seen = 0usize;
    loop {
        if pos >= buf.len() {
            return None;
        }
        if seen > buf.len() {
            return None;
        }
        let len = buf[pos];
        if len & 0xC0 == 0xC0 {
            if pos + 1 >= buf.len() {
                return None;
            }
            let b2 = buf[pos + 1];
            let offset = ((len as u16 & 0x3F) << 8) | b2 as u16;
            let offset = offset as usize;
            if offset >= buf.len() {
                return None;
            }
            if !jumped {
                orig_pos = pos + 2;
            }
            pos = offset;
            jumped = true;
            seen += 1;
            continue;
        }
        let l = len as usize;
        pos += 1;
        if l == 0 {
            break;
        }
        if pos + l > buf.len() {
            return None;
        }
        match std::str::from_utf8(&buf[pos..pos + l]) {
            Ok(s) => labels.push(s.to_string()),
            Err(_) => return None,
        }
        pos += l;
        seen += 1;
    }
    let name = labels.join(".");
    if jumped {
        Some((name, orig_pos))
    } else {
        Some((name, pos))
    }
}

fn encode_name_labels_vec(name: &str) -> Vec<u8> {
    let mut out = Vec::new();
    for label in name.split('.') {
        let l = label.len();
        if l == 0 {
            continue;
        }
        out.push(l as u8);
        out.extend_from_slice(label.as_bytes());
    }
    out.push(0);
    out
}

fn append_opt(resp: &mut Vec<u8>, client_size: usize, client_do: bool) {
    let server_size: u16 = std::env::var("HACKFLARE_DNS_UDP_SIZE")
        .ok()
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(4096u16);
    let size = std::cmp::min(server_size, client_size as u16);
    let flags: u16 = if client_do { 0x8000 } else { 0 };
    resp.extend_from_slice(&[0u8]);
    resp.extend_from_slice(&41u16.to_be_bytes());
    resp.extend_from_slice(&size.to_be_bytes());
    let ttl: u32 = flags as u32;
    resp.extend_from_slice(&ttl.to_be_bytes());
    resp.extend_from_slice(&0u16.to_be_bytes());
}

fn load_soa_config() -> (String, String, u32, u32, u32, u32, u32, u32) {
    let mname =
        env::var("HACKFLARE_DNS_SOA_MNAME").unwrap_or_else(|_| "a.root-servers.net.".to_string());
    let rname = env::var("HACKFLARE_DNS_SOA_RNAME")
        .unwrap_or_else(|_| "nstld.verisign-grs.com.".to_string());
    let serial = env::var("HACKFLARE_DNS_SOA_SERIAL")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(2026042000);
    let refresh = env::var("HACKFLARE_DNS_SOA_REFRESH")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(1800);
    let retry = env::var("HACKFLARE_DNS_SOA_RETRY")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(900);
    let expire = env::var("HACKFLARE_DNS_SOA_EXPIRE")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(604_800);
    let minimum = env::var("HACKFLARE_DNS_SOA_MINIMUM")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(86_400);
    let ttl = env::var("HACKFLARE_DNS_SOA_TTL")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(86_400);
    (mname, rname, serial, refresh, retry, expire, minimum, ttl)
}

fn build_nxdomain_with_soa(
    id: u16,
    req_flags: u16,
    recursion_enabled: bool,
    question_section: &[u8],
) -> Vec<u8> {
    let (mname, rname, serial, refresh, retry, expire, minimum, ttl) = load_soa_config();
    let mut resp: Vec<u8> = Vec::new();
    resp.extend_from_slice(&id.to_be_bytes());
    let mut flags: u16 = 0x8000;
    flags |= req_flags & 0x0100;
    if recursion_enabled {
        flags |= 0x0080;
    }
    flags |= 3;
    resp.extend_from_slice(&flags.to_be_bytes());
    resp.extend_from_slice(&1u16.to_be_bytes());
    resp.extend_from_slice(&0u16.to_be_bytes());
    resp.extend_from_slice(&1u16.to_be_bytes());
    resp.extend_from_slice(&0u16.to_be_bytes());
    resp.extend_from_slice(question_section);

    resp.extend_from_slice(&0x00u8.to_be_bytes());
    resp.extend_from_slice(&6u16.to_be_bytes());
    resp.extend_from_slice(&1u16.to_be_bytes());
    resp.extend_from_slice(&ttl.to_be_bytes());

    let rdata = encode_soa_rdata(&mname, &rname, serial, refresh, retry, expire, minimum);
    resp.extend_from_slice(&(rdata.len() as u16).to_be_bytes());
    resp.extend_from_slice(&rdata);

    resp
}

fn build_servfail(
    id: u16,
    req_flags: u16,
    recursion_enabled: bool,
    question_section: &[u8],
) -> Vec<u8> {
    let mut resp: Vec<u8> = Vec::new();
    resp.extend_from_slice(&id.to_be_bytes());
    let mut flags: u16 = 0x8000;
    flags |= req_flags & 0x0100;
    if recursion_enabled {
        flags |= 0x0080;
    }
    flags |= 2;
    resp.extend_from_slice(&flags.to_be_bytes());
    resp.extend_from_slice(&1u16.to_be_bytes());
    resp.extend_from_slice(&0u16.to_be_bytes());
    resp.extend_from_slice(&0u16.to_be_bytes());
    resp.extend_from_slice(&0u16.to_be_bytes());
    resp.extend_from_slice(question_section);
    resp
}

fn encode_soa_rdata(
    mname: &str,
    rname: &str,
    serial: u32,
    refresh: u32,
    retry: u32,
    expire: u32,
    minimum: u32,
) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&encode_name_labels_vec(mname));
    out.extend_from_slice(&encode_name_labels_vec(rname));
    out.extend_from_slice(&serial.to_be_bytes());
    out.extend_from_slice(&refresh.to_be_bytes());
    out.extend_from_slice(&retry.to_be_bytes());
    out.extend_from_slice(&expire.to_be_bytes());
    out.extend_from_slice(&minimum.to_be_bytes());
    out
}

fn encode_rdata(record_type: &RecordType, value: &str) -> Option<Vec<u8>> {
    match record_type {
        RecordType::A => value
            .parse::<Ipv4Addr>()
            .ok()
            .map(|ip| ip.octets().to_vec()),
        RecordType::Aaaa => value
            .parse::<Ipv6Addr>()
            .ok()
            .map(|ip| ip.octets().to_vec()),
        RecordType::Cname => Some(encode_name_labels_vec(value)),
        RecordType::Ns => Some(encode_name_labels_vec(value)),
        RecordType::Ptr => Some(encode_name_labels_vec(value)),
        RecordType::Mx => {
            let mut parts = value.split_whitespace();
            let pref = parts.next()?.parse::<u16>().ok()?;
            let host = parts.next()?;
            if parts.next().is_some() {
                return None;
            }

            let mut out = Vec::new();
            out.extend_from_slice(&pref.to_be_bytes());
            out.extend_from_slice(&encode_name_labels_vec(host));
            Some(out)
        }
        RecordType::Txt => {
            let mut out = Vec::new();
            let bytes = value.as_bytes();
            if bytes.len() > 255 {
                return None;
            }
            out.push(bytes.len() as u8);
            out.extend_from_slice(bytes);
            Some(out)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::dns::{DnsService, NewRecordInput, RecordType};

    use super::{build_response, encode_name_labels_vec, run_dns_server};

    fn build_query(name: &str, qtype: u16) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&0x1234u16.to_be_bytes());
        out.extend_from_slice(&0x0100u16.to_be_bytes());
        out.extend_from_slice(&1u16.to_be_bytes());
        out.extend_from_slice(&0u16.to_be_bytes());
        out.extend_from_slice(&0u16.to_be_bytes());
        out.extend_from_slice(&0u16.to_be_bytes());
        out.extend_from_slice(&encode_name_labels_vec(name));
        out.extend_from_slice(&qtype.to_be_bytes());
        out.extend_from_slice(&1u16.to_be_bytes());
        out
    }

    fn response_rcode(resp: &[u8]) -> u16 {
        u16::from_be_bytes([resp[2], resp[3]]) & 0x000f
    }

    fn response_ancount(resp: &[u8]) -> u16 {
        u16::from_be_bytes([resp[6], resp[7]])
    }

    fn first_answer_type(resp: &[u8]) -> Option<u16> {
        let (_qname, pos) = super::parse_qname(resp, 12)?;
        let answer_start = pos + 4;
        if answer_start + 4 > resp.len() {
            return None;
        }
        Some(u16::from_be_bytes([
            resp[answer_start + 2],
            resp[answer_start + 3],
        ]))
    }

    #[tokio::test]
    async fn returns_a_answer_for_known_record() {
        let dns = DnsService::new();
        dns.create_zone("example.com", 1).expect("zone should create");
        dns.verify_zone("example.com", 1).expect("zone should verify");
        dns.add_record(
            "example.com",
            NewRecordInput {
                name: "www".to_string(),
                record_type: RecordType::A,
                value: "1.2.3.4".to_string(),
                ttl: 60,
            },
            1,
        )
        .expect("record should create");

        let req = build_query("www.example.com", 1);
        let response_bytes = build_response(&req, &dns).await.expect("build response");

        // response flags in bytes 2..4 contain rcode low 4 bits
        assert_eq!(response_rcode(&response_bytes), 0); // NoError

        // answer count is bytes 6..8
        assert_eq!(response_ancount(&response_bytes), 1);
    }

    #[tokio::test]
    async fn returns_nxdomain_for_unknown_zone() {
        let dns = DnsService::new();
        let req = build_query("missing.example.com", 1);
        let response_bytes = build_response(&req, &dns).await.expect("build response");
        assert_eq!(response_rcode(&response_bytes), 3); // NXDomain
    }

    #[tokio::test]
    async fn supports_additional_record_types_and_any_query() {
        let dns = DnsService::new();
        dns.create_zone("example.com", 1).expect("zone should create");
        dns.create_zone("in-addr.arpa", 1).expect("zone should create");

        dns.verify_zone("example.com", 1).expect("zone should verify");
        dns.verify_zone("in-addr.arpa", 1).expect("zone should verify");

        let records = vec![
            ("example.com", "@", RecordType::Aaaa, "2001:db8::1", 28u16),
            (
                "example.com",
                "@",
                RecordType::Cname,
                "alias.example.com",
                5u16,
            ),
            ("example.com", "@", RecordType::Txt, "hello", 16u16),
            ("example.com", "@", RecordType::Ns, "ns1.example.com", 2u16),
            (
                "example.com",
                "@",
                RecordType::Mx,
                "10 mail.example.com",
                15u16,
            ),
        ];

        for (zone, name, record_type, value, _wire_type) in records {
            dns.add_record(
                zone,
                NewRecordInput {
                    name: name.to_string(),
                    record_type,
                    value: value.to_string(),
                    ttl: 120,
                },
                1,
            )
            .expect("record should create");
        }

        dns.add_record(
            "in-addr.arpa",
            NewRecordInput {
                name: "1.2.0.192.in-addr.arpa".to_string(),
                record_type: RecordType::Ptr,
                value: "host.example.com".to_string(),
                ttl: 120,
            },
            1,
        )
        .expect("ptr record should create");

        let checks = vec![
            ("example.com", 28u16, 28u16),
            ("example.com", 5u16, 5u16),
            ("example.com", 16u16, 16u16),
            ("example.com", 2u16, 2u16),
            ("example.com", 15u16, 15u16),
            ("1.2.0.192.in-addr.arpa", 12u16, 12u16),
        ];

        for (name, qtype, expected_wire_type) in checks {
            let req = build_query(name, qtype);
            let resp = build_response(&req, &dns).await.expect("build response");
            assert_eq!(response_rcode(&resp), 0);
            assert_eq!(response_ancount(&resp), 1);
            assert_eq!(first_answer_type(&resp), Some(expected_wire_type));
        }

        // ANY should include all `example.com` records we inserted for that name.
        let any_req = build_query("example.com", 255);
        let any_resp = build_response(&any_req, &dns)
            .await
            .expect("build any response");
        assert_eq!(response_rcode(&any_resp), 0);
        assert_eq!(response_ancount(&any_resp), 5);
    }

    #[tokio::test]
    async fn integration_known_record_udp() {
        use std::net::UdpSocket as StdUdp;
        use std::sync::Arc;
        use std::time::Duration;
        use tokio::net::UdpSocket;

        let dns = DnsService::new();
        dns.create_zone("example.com", 1).expect("zone should create");
        dns.verify_zone("example.com", 1).expect("zone should verify");
        dns.add_record(
            "example.com",
            NewRecordInput {
                name: "www".to_string(),
                record_type: RecordType::A,
                value: "1.2.3.4".to_string(),
                ttl: 60,
            },
            1,
        )
        .expect("record should create");

        let dns_arc = Arc::new(dns);

        let s = StdUdp::bind(("127.0.0.1", 0)).expect("bind temp");
        let addr = s.local_addr().expect("local addr");
        drop(s);

        let bind_addr = addr;
        let srv_dns = dns_arc.clone();
        tokio::spawn(async move {
            let _ = run_dns_server(bind_addr, srv_dns).await;
        });

        tokio::time::sleep(Duration::from_millis(100)).await;

        let client = UdpSocket::bind("0.0.0.0:0").await.expect("client bind");
        let q = build_query("www.example.com", 1);
        client.send_to(&q, bind_addr).await.expect("send");
        let mut buf = [0u8; 4096];
        let (n, _) = tokio::time::timeout(Duration::from_secs(2), client.recv_from(&mut buf))
            .await
            .expect("recv timeout")
            .expect("recv");
        let resp = &buf[..n];
        assert_eq!(response_rcode(resp), 0);
        assert_eq!(response_ancount(resp), 1);
    }

    #[tokio::test]
    async fn integration_nxdomain_udp() {
        use std::net::UdpSocket as StdUdp;
        use std::sync::Arc;
        use std::time::Duration;
        use tokio::net::UdpSocket;

        let dns = DnsService::new();

        let s = StdUdp::bind(("127.0.0.1", 0)).expect("bind temp");
        let addr = s.local_addr().expect("local addr");
        drop(s);

        let bind_addr = addr;
        let dns_arc = Arc::new(dns);
        let srv_dns = dns_arc.clone();
        tokio::spawn(async move {
            let _ = run_dns_server(bind_addr, srv_dns).await;
        });

        tokio::time::sleep(Duration::from_millis(100)).await;

        let client = UdpSocket::bind("0.0.0.0:0").await.expect("client bind");
        let q = build_query("missing.example.com", 1);
        client.send_to(&q, bind_addr).await.expect("send");
        let mut buf = [0u8; 4096];
        let (n, _) = tokio::time::timeout(Duration::from_secs(2), client.recv_from(&mut buf))
            .await
            .expect("recv timeout")
            .expect("recv");
        let resp = &buf[..n];
        assert_eq!(response_rcode(resp), 3);
    }

    #[tokio::test]
    async fn integration_udp_record_types() {
        use std::net::UdpSocket as StdUdp;
        use std::sync::Arc;
        use std::time::Duration;
        use tokio::net::UdpSocket;

        let dns = DnsService::new();
        dns.create_zone("example.com", 1).expect("zone should create");
        dns.verify_zone("example.com", 1).expect("zone should verify");

        let records = vec![
            (RecordType::Aaaa, "2001:db8::1", 28u16),
            (RecordType::Cname, "alias.example.com", 5u16),
            (RecordType::Txt, "hello", 16u16),
            (RecordType::Ns, "ns1.example.com", 2u16),
            (RecordType::Mx, "10 mail.example.com", 15u16),
        ];

        for (record_type, value, _qtype) in records.iter().cloned() {
            dns.add_record(
                "example.com",
                NewRecordInput {
                    name: "@".to_string(),
                    record_type,
                    value: value.to_string(),
                    ttl: 60,
                },
                1,
            )
            .expect("record should create");
        }

        let dns_arc = Arc::new(dns);
        let s = StdUdp::bind(("127.0.0.1", 0)).expect("bind temp");
        let addr = s.local_addr().expect("local addr");
        drop(s);

        let bind_addr = addr;
        let srv_dns = dns_arc.clone();
        tokio::spawn(async move {
            let _ = run_dns_server(bind_addr, srv_dns).await;
        });

        tokio::time::sleep(Duration::from_millis(100)).await;

        let client = UdpSocket::bind("0.0.0.0:0").await.expect("client bind");
        for (_record_type, _value, qtype) in records {
            let q = build_query("example.com", qtype);
            client.send_to(&q, bind_addr).await.expect("send");
            let mut buf = [0u8; 4096];
            let (n, _) = tokio::time::timeout(Duration::from_secs(2), client.recv_from(&mut buf))
                .await
                .expect("recv timeout")
                .expect("recv");
            let resp = &buf[..n];
            assert_eq!(response_rcode(resp), 0);
            assert_eq!(response_ancount(resp), 1);
            assert_eq!(first_answer_type(resp), Some(qtype));
        }

        let q_any = build_query("example.com", 255);
        client.send_to(&q_any, bind_addr).await.expect("send");
        let mut buf = [0u8; 4096];
        let (n, _) = tokio::time::timeout(Duration::from_secs(2), client.recv_from(&mut buf))
            .await
            .expect("recv timeout")
            .expect("recv");
        let resp = &buf[..n];
        assert_eq!(response_rcode(resp), 0);
        assert_eq!(response_ancount(resp), 5);
    }
}
