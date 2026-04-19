use crate::dns::Record;

pub fn encode(r: &Record) -> Option<Vec<u8>> {
    let parts: Vec<&str> = r.data.split_whitespace().collect();
    if parts.len() == 4 {
        let keytag = parts[0].parse::<u16>().ok()?;
        let alg = parts[1].parse::<u8>().ok()?;
        let dtype = parts[2].parse::<u8>().ok()?;
        let digest = crate::dns::engine::parse_hex_bytes(parts[3])?;
        let mut out = Vec::new();
        out.extend_from_slice(&keytag.to_be_bytes());
        out.push(alg);
        out.push(dtype);
        out.extend_from_slice(&digest);
        Some(out)
    } else if parts.len() == 1 {
        let digest = crate::dns::engine::parse_hex_bytes(parts[0])?;
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

use crate::dns::registry::Registry;

pub fn register(reg: &mut Registry) {
    reg.register("DS", encode);
}
