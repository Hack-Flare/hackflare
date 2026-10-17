use crate::dns::wire::{encode_name_labels_vec, parse_qname};

pub fn tld_from_name(name: &str) -> Option<String> {
    name.split('.')
        .rev()
        .find(|label| !label.is_empty())
        .map(str::to_ascii_lowercase)
}

const TLD_DELEGATION_MIN_TTL_SECS: u64 = 3600;
const TLD_DELEGATION_MAX_TTL_SECS: u64 = 86400;

pub fn clamp_tld_ttl(ttl_secs: u64) -> u64 {
    ttl_secs.clamp(TLD_DELEGATION_MIN_TTL_SECS, TLD_DELEGATION_MAX_TTL_SECS)
}

#[derive(Debug, Clone, Copy)]
pub struct RecordInfo {
    pub rtype: u16,
    pub pos: usize,
    pub rdlen: usize,
    pub ttl: u32,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct DnsHeader {
    pub id: u16,
    pub flags: u16,
    pub qdcount: u16,
    pub ancount: u16,
    pub nscount: u16,
    pub arcount: u16,
}

impl DnsHeader {
    pub fn from_wire(buf: &[u8]) -> Option<Self> {
        if buf.len() < 12 {
            return None;
        }
        Some(Self {
            id: u16::from_be_bytes([buf[0], buf[1]]),
            flags: u16::from_be_bytes([buf[2], buf[3]]),
            qdcount: u16::from_be_bytes([buf[4], buf[5]]),
            ancount: u16::from_be_bytes([buf[6], buf[7]]),
            nscount: u16::from_be_bytes([buf[8], buf[9]]),
            arcount: u16::from_be_bytes([buf[10], buf[11]]),
        })
    }

    pub fn is_truncated(&self) -> bool {
        self.flags & 0x0200 != 0
    }
}

pub fn build_query(id: u16, qname: &str, qtype: u16) -> Vec<u8> {
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

pub fn response_matches_expected(
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

pub fn parse_rrs(buf: &[u8], mut pos: usize, count: usize) -> Option<Vec<RecordInfo>> {
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        let (_, p2) = parse_qname(buf, pos)?;
        pos = p2;
        if pos + 10 > buf.len() {
            return None;
        }
        let rtype = u16::from_be_bytes([buf[pos], buf[pos + 1]]);
        let class = u16::from_be_bytes([buf[pos + 2], buf[pos + 3]]);
        if class != 1 {
            return None;
        }
        let ttl = u32::from_be_bytes([buf[pos + 4], buf[pos + 5], buf[pos + 6], buf[pos + 7]]);
        let rdlen = u16::from_be_bytes([buf[pos + 8], buf[pos + 9]]) as usize;
        pos += 10;
        if pos + rdlen > buf.len() {
            return None;
        }
        out.push(RecordInfo {
            rtype,
            pos,
            rdlen,
            ttl,
        });
        pos += rdlen;
    }
    Some(out)
}

pub(super) fn extract_ns_and_glue(
    buf: &[u8],
    authority_rrs: &[RecordInfo],
    additional_rrs: &[RecordInfo],
) -> (Vec<String>, Vec<String>) {
    let mut ns_names: Vec<String> = Vec::new();
    let mut glue_ips: Vec<String> = Vec::new();
    for rr in authority_rrs {
        if rr.rtype == 2
            && let Some((name, _)) = parse_qname(buf, rr.pos)
        {
            ns_names.push(name);
        }
    }
    for rr in additional_rrs {
        if rr.rtype == 1 && rr.rdlen == 4 {
            let ip = format!(
                "{}.{}.{}.{}",
                buf[rr.pos],
                buf[rr.pos + 1],
                buf[rr.pos + 2],
                buf[rr.pos + 3]
            );
            glue_ips.push(ip);
        } else if rr.rtype == 28
            && rr.rdlen == 16
            && let Ok(ipv6) = <[u8; 16]>::try_from(&buf[rr.pos..rr.pos + 16])
        {
            glue_ips.push(std::net::Ipv6Addr::from(ipv6).to_string());
        }
    }
    (ns_names, glue_ips)
}
