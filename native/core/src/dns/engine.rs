use crate::dns::{DnsManager, Record};
use idna::domain_to_ascii;
use std::env;
use std::net::{Ipv4Addr, Ipv6Addr};

pub struct DnsEngine {
    pub manager: DnsManager,
    pub recursion_enabled: bool,
}

impl DnsEngine {
    pub fn new(manager: DnsManager) -> Self {
        Self {
            manager,
            recursion_enabled: true,
        }
    }

    pub fn handle_query(&self, req: &[u8]) -> Option<Vec<u8>> {
        // Ensure we return a response when possible to avoid clients
        // falling back to TCP due to UDP timeouts on malformed queries.
        if req.len() < 12 {
            let id = if req.len() >= 2 { u16::from_be_bytes([req[0], req[1]]) } else { 0 };
            let req_flags = if req.len() >= 4 { u16::from_be_bytes([req[2], req[3]]) } else { 0 };
            return Some(build_servfail(id, req_flags, self.recursion_enabled, &[]));
        }

        let id = u16::from_be_bytes([req[0], req[1]]);
        let qdcount = u16::from_be_bytes([req[4], req[5]]);
        let ancount = u16::from_be_bytes([req[6], req[7]]);
        let nscount = u16::from_be_bytes([req[8], req[9]]);
        let arcount = u16::from_be_bytes([req[10], req[11]]);
        if qdcount == 0 {
            let req_flags = u16::from_be_bytes([req[2], req[3]]);
            return Some(build_servfail(id, req_flags, self.recursion_enabled, &req[12..]));
        }

        let mut pos = 12usize;
        let (qname, new_pos) = match parse_qname(req, pos) {
            Some((n, p)) => (n, p),
            None => {
                let req_flags = u16::from_be_bytes([req[2], req[3]]);
                // Append OPT if present
                return Some(build_servfail(id, req_flags, self.recursion_enabled, &req[12..]));
            }
        };
        pos = new_pos;
        if pos + 4 > req.len() {
            let req_flags = u16::from_be_bytes([req[2], req[3]]);
            return Some(build_servfail(id, req_flags, self.recursion_enabled, &req[12..pos]));
        }
        let qtype = u16::from_be_bytes([req[pos], req[pos + 1]]);
        let _qclass = u16::from_be_bytes([req[pos + 2], req[pos + 3]]);

        // Parse additional section for EDNS OPT
        let mut client_edns_size: usize = 0;
        let mut client_do: bool = false;
        let pos_after_question = pos + 4;
        let mut rr_pos = pos_after_question;
        // skip answer and authority RRs (usually zero in queries)
        let skip_rrs = ancount as usize + nscount as usize;
        for _ in 0..skip_rrs {
            if rr_pos >= req.len() { break; }
            if let Some((_name, newp)) = parse_qname(req, rr_pos) {
                rr_pos = newp;
            } else { break; }
            if rr_pos + 10 > req.len() { break; }
            let rdlen = u16::from_be_bytes([req[rr_pos + 8], req[rr_pos + 9]]) as usize;
            rr_pos += 10 + rdlen;
        }
        // parse additional records
        for _ in 0..(arcount as usize) {
            if rr_pos >= req.len() { break; }
            if let Some((_name, newp)) = parse_qname(req, rr_pos) {
                rr_pos = newp;
            } else { break; }
            if rr_pos + 10 > req.len() { break; }
            let typ = u16::from_be_bytes([req[rr_pos], req[rr_pos + 1]]);
            let class = u16::from_be_bytes([req[rr_pos + 2], req[rr_pos + 3]]);
            let ttl = u32::from_be_bytes([req[rr_pos + 4], req[rr_pos + 5], req[rr_pos + 6], req[rr_pos + 7]]);
            let rdlen = u16::from_be_bytes([req[rr_pos + 8], req[rr_pos + 9]]) as usize;
            rr_pos += 10;
            if typ == 41 {
                client_edns_size = class as usize;
                // DO bit is in lower 16 bits of ttl (bit 15)
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
            self.manager.find_records(&qname, None)
        } else if qtype_str.is_empty() {
            Vec::new()
        } else {
            self.manager.find_answer_records(&qname, Some(qtype_str))
        };

        if is_ip_literal
            && recs.is_empty()
            && let Some(rn) = reverse_name.as_ref()
        {
            let ptrs = self.manager.find_records(rn, Some("PTR"));
            if !ptrs.is_empty() {
                recs = ptrs;
            } else {
                let mut r = build_nxdomain_with_soa(id, req_flags, self.recursion_enabled, &req[12..pos + 4]);
                if client_edns_size > 0 {
                    append_opt(&mut r, client_edns_size, client_do);
                }
                return Some(r);
            }
        }

        let label_count = qname.split('.').filter(|label| !label.is_empty()).count();
        if recs.is_empty() && reverse_name.is_none() && label_count < 2 {
            let mut r = build_nxdomain_with_soa(id, req_flags, self.recursion_enabled, &req[12..pos + 4]);
            if client_edns_size > 0 { append_opt(&mut r, client_edns_size, client_do); }
            return Some(r);
        }

        let req_flags = u16::from_be_bytes([req[2], req[3]]);
        if recs.is_empty() {
            if let Some(mut resp) = crate::dns::recursive::resolve(&qname, qtype, 6) {
                let id_bytes = id.to_be_bytes();
                if resp.len() >= 2 {
                    resp[0] = id_bytes[0];
                    resp[1] = id_bytes[1];
                }
                if resp.len() >= 4 {
                    let resp_flags = u16::from_be_bytes([resp[2], resp[3]]);
                    let mut new_flags = resp_flags | 0x8000;
                    new_flags &= !0x0400;

                    new_flags |= req_flags & 0x0100;
                    if self.recursion_enabled {
                        new_flags |= 0x0080;
                    }
                    let nf = new_flags.to_be_bytes();
                    resp[2] = nf[0];
                    resp[3] = nf[1];
                }
                return Some(resp);
            }

            if let Some(rn) = reverse_name.as_ref()
                && let Some(mut resp) = crate::dns::recursive::resolve(rn, 12, 6)
            {
                let id_bytes = id.to_be_bytes();
                if resp.len() >= 2 {
                    resp[0] = id_bytes[0];
                    resp[1] = id_bytes[1];
                }
                if resp.len() >= 4 {
                    let resp_flags = u16::from_be_bytes([resp[2], resp[3]]);
                    let mut new_flags = resp_flags | 0x8000;
                    new_flags &= !0x0400;
                    new_flags |= req_flags & 0x0100;
                    if self.recursion_enabled {
                        new_flags |= 0x0080;
                    }
                    let nf = new_flags.to_be_bytes();
                    resp[2] = nf[0];
                    resp[3] = nf[1];
                }
                return Some(resp);
            }

                let mut r = build_servfail(id, req_flags, self.recursion_enabled, &req[12..pos + 4]);
                if client_edns_size > 0 { append_opt(&mut r, client_edns_size, client_do); }
                return Some(r);
        }

        let mut resp: Vec<u8> = Vec::new();
        resp.extend_from_slice(&id.to_be_bytes());

        let mut flags: u16 = 0x8000;
        if !recs.is_empty() {
            flags |= 0x0400;
        }

        flags |= req_flags & 0x0100;
        if self.recursion_enabled {
            flags |= 0x0080;
        }
        let ar_out = if client_edns_size > 0 { 1u16 } else { 0u16 };
        resp.extend_from_slice(&flags.to_be_bytes());
        resp.extend_from_slice(&1u16.to_be_bytes());
        resp.extend_from_slice(&(recs.len() as u16).to_be_bytes());
        resp.extend_from_slice(&0u16.to_be_bytes());
        resp.extend_from_slice(&ar_out.to_be_bytes());

        resp.extend_from_slice(&req[12..pos + 4]);

        for ans in recs {
            resp.extend_from_slice(&0xC00C_u16.to_be_bytes());
            let rtype = map_qtype_to_num(&ans.rtype);
            resp.extend_from_slice(&rtype.to_be_bytes());
            resp.extend_from_slice(&1u16.to_be_bytes());
            resp.extend_from_slice(&ans.ttl.to_be_bytes());

            if let Some(rdata) = crate::dns::records::encode_by_type(&ans.rtype, &ans) {
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
}

fn build_nxdomain_with_soa(
    id: u16,
    req_flags: u16,
    recursion_enabled: bool,
    question_section: &[u8],
) -> Vec<u8> {
    let soa_config = load_soa_config();
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
    resp.extend_from_slice(&soa_config.ttl.to_be_bytes());

    let soa = Record::new(
        ".",
        "SOA",
        soa_config.ttl,
        format!(
            "{} {} {} {} {} {} {}",
            soa_config.mname,
            soa_config.rname,
            soa_config.serial,
            soa_config.refresh,
            soa_config.retry,
            soa_config.expire,
            soa_config.minimum
        ),
    );
    if let Some(rdata) = crate::dns::records::encode_by_type("SOA", &soa) {
        resp.extend_from_slice(&(rdata.len() as u16).to_be_bytes());
        resp.extend_from_slice(&rdata);
    } else {
        resp.extend_from_slice(&0u16.to_be_bytes());
    }

    resp
}

fn build_servfail(id: u16, req_flags: u16, recursion_enabled: bool, question_section: &[u8]) -> Vec<u8> {
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

struct SoaConfig {
    mname: String,
    rname: String,
    serial: u32,
    refresh: u32,
    retry: u32,
    expire: u32,
    minimum: u32,
    ttl: u32,
}

fn load_soa_config() -> SoaConfig {
    SoaConfig {
        mname: env::var("HACKFLARE_DNS_SOA_MNAME")
            .unwrap_or_else(|_| "a.root-servers.net.".to_string()),
        rname: env::var("HACKFLARE_DNS_SOA_RNAME")
            .unwrap_or_else(|_| "nstld.verisign-grs.com.".to_string()),
        serial: env::var("HACKFLARE_DNS_SOA_SERIAL")
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(2026042000),
        refresh: env::var("HACKFLARE_DNS_SOA_REFRESH")
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(1800),
        retry: env::var("HACKFLARE_DNS_SOA_RETRY")
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(900),
        expire: env::var("HACKFLARE_DNS_SOA_EXPIRE")
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(604_800),
        minimum: env::var("HACKFLARE_DNS_SOA_MINIMUM")
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(86_400),
        ttl: env::var("HACKFLARE_DNS_SOA_TTL")
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(86_400),
    }
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

fn map_qtype_to_num(s: &str) -> u16 {
    match s.to_uppercase().as_str() {
        "A" => 1,
        "NS" => 2,
        "CNAME" => 5,
        "SOA" => 6,
        "PTR" => 12,
        "HINFO" => 13,
        "MX" => 15,
        "TXT" => 16,
        "AAAA" => 28,
        "LOC" => 29,
        "SRV" => 33,
        "DS" => 43,
        "SSHFP" => 44,
        "RRSIG" => 46,
        "NSEC" => 47,
        "DNSKEY" => 48,
        "NSEC3" => 50,
        "TLSA" => 52,
        "SVCB" => 64,
        "HTTPS" => 65,
        "ANY" => 255,
        "CAA" => 257,
        _ => 1,
    }
}

pub(crate) fn parse_hex_bytes(s: &str) -> Option<Vec<u8>> {
    let s = s
        .trim_start_matches("0x")
        .replace(|c: char| c.is_whitespace(), "");
    if s.len() % 2 != 0 {
        return None;
    }
    let mut out = Vec::with_capacity(s.len() / 2);
    for i in (0..s.len()).step_by(2) {
        let byte = u8::from_str_radix(&s[i..i + 2], 16).ok()?;
        out.push(byte);
    }
    Some(out)
}

pub(crate) fn parse_qname(buf: &[u8], mut pos: usize) -> Option<(String, usize)> {
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

pub(crate) fn encode_name_labels(name: &str) -> Vec<u8> {
    let ascii = domain_to_ascii(name).unwrap_or_else(|_| name.to_string());
    encode_name_labels_vec(&ascii)
}

fn append_opt(resp: &mut Vec<u8>, client_size: usize, client_do: bool) {
    let server_size: u16 = std::env::var("HACKFLARE_DNS_UDP_SIZE")
        .ok()
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(4096u16);
    let size = std::cmp::min(server_size, client_size as u16);
    // OPT RR: name (root), type 41, class = udp payload size, ttl = (extended rcode<<24)|(version<<16)|flags
    let flags: u16 = if client_do { 0x8000 } else { 0 };
    resp.extend_from_slice(&[0u8]); // root name
    resp.extend_from_slice(&41u16.to_be_bytes());
    resp.extend_from_slice(&size.to_be_bytes());
    let ttl: u32 = flags as u32;
    resp.extend_from_slice(&ttl.to_be_bytes());
    resp.extend_from_slice(&0u16.to_be_bytes());
}

pub(crate) fn encode_name_labels_vec(name: &str) -> Vec<u8> {
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
