use crate::dns::Record;
use std::net::Ipv4Addr;

pub fn encode(r: &Record) -> Option<Vec<u8>> {
    if let Ok(ip) = r.data.parse::<Ipv4Addr>() {
        let mut out = Vec::new();
        out.extend_from_slice(&ip.octets());
        Some(out)
    } else {
        None
    }
}

use crate::dns::registry::Registry;

pub fn register(reg: &mut Registry) {
    reg.register("A", encode);
}
