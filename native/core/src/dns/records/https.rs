use crate::dns::Record;

pub fn encode(r: &Record) -> Option<Vec<u8>> {
    Some(r.data.as_bytes().to_vec())
}

use crate::dns::registry::Registry;

pub fn register(reg: &mut Registry) {
    reg.register("HTTPS", encode);
}
