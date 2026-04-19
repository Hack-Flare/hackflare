use crate::dns::Record;
use crate::dns::engine::encode_name_labels;

pub fn encode(r: &Record) -> Option<Vec<u8>> {
    Some(encode_name_labels(&r.data))
}

use crate::dns::registry::Registry;

pub fn register(reg: &mut Registry) {
    reg.register("NS", encode);
}
