use crate::dns::Record;

pub fn encode(r: &Record) -> Option<Vec<u8>> {
    let parts: Vec<&str> = r.data.split_whitespace().collect();
    if parts.len() >= 4 {
        let usage = parts[0].parse::<u8>().ok()?;
        let selector = parts[1].parse::<u8>().ok()?;
        let mtype = parts[2].parse::<u8>().ok()?;
        let data = crate::dns::engine::parse_hex_bytes(parts[3])?;
        let mut out = Vec::new();
        out.push(usage);
        out.push(selector);
        out.push(mtype);
        out.extend_from_slice(&data);
        Some(out)
    } else {
        None
    }
}

use crate::dns::registry::Registry;

pub fn register(reg: &mut Registry) {
    reg.register("TLSA", encode);
}
