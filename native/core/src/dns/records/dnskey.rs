use crate::dns::Record;
use base64::Engine;

pub fn encode(r: &Record) -> Option<Vec<u8>> {
    let parts: Vec<&str> = r.data.split_whitespace().collect();
    if parts.len() == 4 {
        let flags = parts[0].parse::<u16>().ok()?;
        let protocol = parts[1].parse::<u8>().ok()?;
        let algorithm = parts[2].parse::<u8>().ok()?;
        let key = base64::engine::general_purpose::STANDARD
            .decode(parts[3])
            .ok()?;
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
        let key = base64::engine::general_purpose::STANDARD
            .decode(parts[2])
            .ok()?;
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

use crate::dns::registry::Registry;

pub fn register(reg: &mut Registry) {
    reg.register("DNSKEY", encode);
}
