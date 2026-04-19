use crate::dns::Record;
use crate::dns::engine::encode_name_labels_vec;

pub fn encode(r: &Record) -> Option<Vec<u8>> {
    let mut parts = r.data.split_whitespace();
    if let (
        Some(mname),
        Some(rname),
        Some(serial),
        Some(refresh),
        Some(retry),
        Some(expire),
        Some(minimum),
    ) = (
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
    ) {
        if let (Ok(serial_v), Ok(refresh_v), Ok(retry_v), Ok(expire_v), Ok(min_v)) = (
            serial.parse::<u32>(),
            refresh.parse::<u32>(),
            retry.parse::<u32>(),
            expire.parse::<u32>(),
            minimum.parse::<u32>(),
        ) {
            let mut rdata = Vec::new();
            rdata.extend_from_slice(&encode_name_labels_vec(mname));
            rdata.extend_from_slice(&encode_name_labels_vec(rname));
            rdata.extend_from_slice(&serial_v.to_be_bytes());
            rdata.extend_from_slice(&refresh_v.to_be_bytes());
            rdata.extend_from_slice(&retry_v.to_be_bytes());
            rdata.extend_from_slice(&expire_v.to_be_bytes());
            rdata.extend_from_slice(&min_v.to_be_bytes());
            return Some(rdata);
        }
    }
    None
}

use crate::dns::registry::Registry;

pub fn register(reg: &mut Registry) {
    reg.register("SOA", encode);
}
