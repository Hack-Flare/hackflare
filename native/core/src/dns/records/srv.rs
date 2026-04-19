use crate::dns::Record;
use crate::dns::engine::encode_name_labels;

pub fn encode(r: &Record) -> Option<Vec<u8>> {
    let mut parts = r.data.split_whitespace();
    if let (Some(pri), Some(w), Some(port), Some(target)) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    {
        if let (Ok(pri_v), Ok(w_v), Ok(port_v)) =
            (pri.parse::<u16>(), w.parse::<u16>(), port.parse::<u16>())
        {
            let mut rdata = Vec::new();
            rdata.extend_from_slice(&pri_v.to_be_bytes());
            rdata.extend_from_slice(&w_v.to_be_bytes());
            rdata.extend_from_slice(&port_v.to_be_bytes());
            rdata.extend_from_slice(&encode_name_labels(target));
            return Some(rdata);
        }
    }
    None
}

use crate::dns::registry::Registry;

pub fn register(reg: &mut Registry) {
    reg.register("SRV", encode);
}
