use crate::dns::Record;

pub fn encode(r: &Record) -> Option<Vec<u8>> {
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

use crate::dns::registry::Registry;

pub fn register(reg: &mut Registry) {
    reg.register("HINFO", encode);
}
