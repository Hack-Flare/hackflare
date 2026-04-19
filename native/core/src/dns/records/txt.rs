use crate::dns::Record;

pub fn encode(r: &Record) -> Option<Vec<u8>> {
    let chunks: Vec<&str> = r.data.split('\n').collect();
    let mut rdata = Vec::new();
    for c in chunks {
        let b = c.as_bytes();
        let len = b.len().min(255) as u8;
        rdata.push(len);
        rdata.extend_from_slice(&b[..len as usize]);
    }
    Some(rdata)
}

use crate::dns::registry::Registry;

pub fn register(reg: &mut Registry) {
    reg.register("TXT", encode);
}
