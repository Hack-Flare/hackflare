use crate::dns::engine::{encode_name_labels, encode_name_labels_vec, parse_hex_bytes};
use crate::dns::registry::Registry;
use crate::dns::Record;
use base64::Engine;
use once_cell::sync::Lazy;
use std::net::{Ipv4Addr, Ipv6Addr};

fn encode_ipv4(r: &Record) -> Option<Vec<u8>> {
    if let Ok(ip) = r.data.parse::<Ipv4Addr>() {
        let mut out = Vec::new();
        out.extend_from_slice(&ip.octets());
        Some(out)
    } else {
        None
    }
}

fn encode_ipv6(r: &Record) -> Option<Vec<u8>> {
    if let Ok(ip) = r.data.parse::<Ipv6Addr>() {
        Some(ip.octets().to_vec())
    } else {
        None
    }
}

fn encode_none(_r: &Record) -> Option<Vec<u8>> {
    None
}

fn encode_name(r: &Record) -> Option<Vec<u8>> {
    Some(encode_name_labels(&r.data))
}

fn encode_raw(r: &Record) -> Option<Vec<u8>> {
    Some(r.data.as_bytes().to_vec())
}

fn encode_caa(r: &Record) -> Option<Vec<u8>> {
    let mut parts = r.data.splitn(3, ' ');
    let flags = parts.next()?.parse::<u8>().ok()?
    ;
    let tag = parts.next()?;
    let mut value = parts.next().unwrap_or("");

    if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
        value = &value[1..value.len() - 1];
    }

    let mut out = Vec::new();
    out.push(flags);
    out.push(tag.len().min(255) as u8);
    out.extend_from_slice(tag.as_bytes());
    out.extend_from_slice(value.as_bytes());
    Some(out)
}

fn encode_dnskey(r: &Record) -> Option<Vec<u8>> {
    let parts: Vec<&str> = r.data.split_whitespace().collect();
    if parts.len() == 4 {
        let flags = parts[0].parse::<u16>().ok()?;
        let protocol = parts[1].parse::<u8>().ok()?;
        let algorithm = parts[2].parse::<u8>().ok()?;
        let key = base64::engine::general_purpose::STANDARD.decode(parts[3]).ok()?;
        let mut out = Vec::new();
        out.extend_from_slice(&flags.to_be_bytes());
        out.push(protocol);
        out.push(algorithm);
        out.extend_from_slice(&key);
        Some(out)
    } else if parts.len() == 3 {
        let flags = parts[0].parse::<u16>().ok()?;
        let protocol = 3u8;
        let algorithm = parts[1].parse::<u8>().ok()?;
        let key = base64::engine::general_purpose::STANDARD.decode(parts[2]).ok()?;
        let mut out = Vec::new();
        out.extend_from_slice(&flags.to_be_bytes());
        out.push(protocol);
        out.push(algorithm);
        out.extend_from_slice(&key);
        Some(out)
    } else {
        None
    }
}

fn encode_ds(r: &Record) -> Option<Vec<u8>> {
    let parts: Vec<&str> = r.data.split_whitespace().collect();
    if parts.len() == 4 {
        let keytag = parts[0].parse::<u16>().ok()?;
        let alg = parts[1].parse::<u8>().ok()?;
        let dtype = parts[2].parse::<u8>().ok()?;
        let digest = parse_hex_bytes(parts[3])?;
        let mut out = Vec::new();
        out.extend_from_slice(&keytag.to_be_bytes());
        out.push(alg);
        out.push(dtype);
        out.extend_from_slice(&digest);
        Some(out)
    } else if parts.len() == 1 {
        let digest = parse_hex_bytes(parts[0])?;
        let mut out = Vec::new();
        out.extend_from_slice(&0u16.to_be_bytes());
        out.push(0);
        out.push(0);
        out.extend_from_slice(&digest);
        Some(out)
    } else {
        None
    }
}

fn encode_hinfo(r: &Record) -> Option<Vec<u8>> {
    let mut parts = r.data.splitn(2, ' ');
    let cpu = parts.next().unwrap_or("").as_bytes();
    let os = parts.next().unwrap_or("").as_bytes();
    let mut out = Vec::new();
    out.push(cpu.len().min(255) as u8);
    out.extend_from_slice(cpu);
    out.push(os.len().min(255) as u8);
    out.extend_from_slice(os);
    Some(out)
}

fn encode_mx(r: &Record) -> Option<Vec<u8>> {
    let mut parts = r.data.split_whitespace();
    if let (Some(pref), Some(exchange)) = (parts.next(), parts.next())
        && let Ok(pref_v) = pref.parse::<u16>()
    {
        let mut rdata = Vec::new();
        rdata.extend_from_slice(&pref_v.to_be_bytes());
        rdata.extend_from_slice(&encode_name_labels(exchange));
        return Some(rdata);
    }
    None
}

fn encode_rrsig(r: &Record) -> Option<Vec<u8>> {
    let mut parts = r.data.split_whitespace();
    let type_covered = parts.next()?.parse::<u16>().ok()?;
    let algorithm = parts.next()?.parse::<u8>().ok()?;
    let labels = parts.next()?.parse::<u8>().ok()?;
    let orig_ttl = parts.next()?.parse::<u32>().ok()?;
    let sig_exp = parts.next()?.parse::<u32>().ok()?;
    let sig_inc = parts.next()?.parse::<u32>().ok()?;
    let key_tag = parts.next()?.parse::<u16>().ok()?;
    let signer = parts.next()?;
    let signature_b64 = parts.next()?;
    let sig = base64::engine::general_purpose::STANDARD
        .decode(signature_b64)
        .ok()?;

    let mut out = Vec::new();
    out.extend_from_slice(&type_covered.to_be_bytes());
    out.push(algorithm);
    out.push(labels);
    out.extend_from_slice(&orig_ttl.to_be_bytes());
    out.extend_from_slice(&sig_exp.to_be_bytes());
    out.extend_from_slice(&sig_inc.to_be_bytes());
    out.extend_from_slice(&key_tag.to_be_bytes());

    out.extend_from_slice(&encode_name_labels_vec(signer));
    out.extend_from_slice(&sig);
    Some(out)
}

fn encode_soa(r: &Record) -> Option<Vec<u8>> {
    let mut parts = r.data.split_whitespace();
    if let (
        Some(mname),
        Some(rname),
        Some(serial),
        Some(refresh),
        Some(retry),
        Some(expire),
        Some(minimum),
    ) = (
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
    ) && let (Ok(serial_v), Ok(refresh_v), Ok(retry_v), Ok(expire_v), Ok(min_v)) = (
        serial.parse::<u32>(),
        refresh.parse::<u32>(),
        retry.parse::<u32>(),
        expire.parse::<u32>(),
        minimum.parse::<u32>(),
    ) {
        let mut rdata = Vec::new();
        rdata.extend_from_slice(&encode_name_labels_vec(mname));
        rdata.extend_from_slice(&encode_name_labels_vec(rname));
        rdata.extend_from_slice(&serial_v.to_be_bytes());
        rdata.extend_from_slice(&refresh_v.to_be_bytes());
        rdata.extend_from_slice(&retry_v.to_be_bytes());
        rdata.extend_from_slice(&expire_v.to_be_bytes());
        rdata.extend_from_slice(&min_v.to_be_bytes());
        return Some(rdata);
    }
    None
}

fn encode_srv(r: &Record) -> Option<Vec<u8>> {
    let mut parts = r.data.split_whitespace();
    if let (Some(pri), Some(w), Some(port), Some(target)) =
        (parts.next(), parts.next(), parts.next(), parts.next())
        && let (Ok(pri_v), Ok(w_v), Ok(port_v)) =
            (pri.parse::<u16>(), w.parse::<u16>(), port.parse::<u16>())
    {
        let mut rdata = Vec::new();
        rdata.extend_from_slice(&pri_v.to_be_bytes());
        rdata.extend_from_slice(&w_v.to_be_bytes());
        rdata.extend_from_slice(&port_v.to_be_bytes());
        rdata.extend_from_slice(&encode_name_labels(target));
        return Some(rdata);
    }
    None
}

fn encode_txt(r: &Record) -> Option<Vec<u8>> {
    let chunks: Vec<&str> = r.data.split('\n').collect();
    let mut rdata = Vec::new();
    for chunk in chunks {
        let bytes = chunk.as_bytes();
        let len = bytes.len().min(255) as u8;
        rdata.push(len);
        rdata.extend_from_slice(&bytes[..len as usize]);
    }
    Some(rdata)
}

fn register_all(reg: &mut Registry) {
    reg.register("A", encode_ipv4);
    reg.register("NS", encode_name);
    reg.register("CNAME", encode_name);
    reg.register("SOA", encode_soa);
    reg.register("PTR", encode_name);
    reg.register("HINFO", encode_hinfo);
    reg.register("MX", encode_mx);
    reg.register("TXT", encode_txt);
    reg.register("AAAA", encode_ipv6);
    reg.register("LOC", encode_raw);
    reg.register("SRV", encode_srv);
    reg.register("DS", encode_ds);
    reg.register("SSHFP", encode_ds);
    reg.register("RRSIG", encode_rrsig);
    reg.register("NSEC", encode_raw);
    reg.register("DNSKEY", encode_dnskey);
    reg.register("NSEC3", encode_raw);
    reg.register("TLSA", encode_ds);
    reg.register("SVCB", encode_raw);
    reg.register("HTTPS", encode_raw);
    reg.register("ANY", encode_none);
    reg.register("CAA", encode_caa);
    reg.register("UNKNOWN", encode_none);
}

pub(crate) static REGISTRY: Lazy<Registry> = Lazy::new(|| {
    let mut registry = Registry::new();
    register_all(&mut registry);
    registry
});

pub fn encode_by_type(typ: &str, record: &Record) -> Option<Vec<u8>> {
    REGISTRY.get(typ).and_then(|enc| enc(record))
}