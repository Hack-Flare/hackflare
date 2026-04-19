use crate::dns::Record;
use std::net::Ipv6Addr;

pub fn encode(r: &Record) -> Option<Vec<u8>> {
    if let Ok(ip) = r.data.parse::<Ipv6Addr>() {
        Some(ip.octets().to_vec())
    } else {
        None
    }
}

use crate::dns::registry::Registry;

pub fn register(reg: &mut Registry) {
    reg.register("AAAA", encode);
}
