use crate::dns::Record;

pub fn encode(r: &Record) -> Option<Vec<u8>> {
    let parts: Vec<&str> = r.data.split_whitespace().collect();
    if parts.len() >= 3 {
        let alg = parts[0].parse::<u8>().ok()?;
        let fptype = parts[1].parse::<u8>().ok()?;
        let fp = crate::dns::engine::parse_hex_bytes(parts[2])?;
        let mut out = Vec::new();
        out.push(alg);
        out.push(fptype);
        out.extend_from_slice(&fp);
        Some(out)
    } else if parts.len() == 1 {
        let fp = crate::dns::engine::parse_hex_bytes(parts[0])?;
        let mut out = Vec::new();
        out.push(0);
        out.push(0);
        out.extend_from_slice(&fp);
        Some(out)
    } else {
        None
    }
}

use crate::dns::registry::Registry;

pub fn register(reg: &mut Registry) {
    reg.register("SSHFP", encode);
}
