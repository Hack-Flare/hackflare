use crate::dns::Record;

pub fn encode(r: &Record) -> Option<Vec<u8>> {
    let mut parts = r.data.splitn(3, ' ');
    let flags = parts.next()?.parse::<u8>().ok()?;
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

use crate::dns::registry::Registry;

pub fn register(reg: &mut Registry) {
    reg.register("CAA", encode);
}
