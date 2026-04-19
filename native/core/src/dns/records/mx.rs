use crate::dns::Record;
use crate::dns::engine::encode_name_labels;

pub fn encode(r: &Record) -> Option<Vec<u8>> {
    let mut parts = r.data.splitn(2, ' ');
    if let (Some(pref), Some(exchange)) = (parts.next(), parts.next()) {
        if let Ok(pref_v) = pref.parse::<u16>() {
            let mut out = pref_v.to_be_bytes().to_vec();
            out.extend_from_slice(&encode_name_labels(exchange));
            return Some(out);
        }
    }
    None
}

use crate::dns::registry::Registry;

pub fn register(reg: &mut Registry) {
    reg.register("MX", encode);
}
