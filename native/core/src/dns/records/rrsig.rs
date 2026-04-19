use crate::dns::Record;
use base64::Engine;

pub fn encode(r: &Record) -> Option<Vec<u8>> {
    let mut parts = r.data.split_whitespace();
    let type_covered = parts.next()?.parse::<u16>().ok()?;
    let algorithm = parts.next()?.parse::<u8>().ok()?;
    let labels = parts.next()?.parse::<u8>().ok()?;
    let orig_ttl = parts.next()?.parse::<u32>().ok()?;
    let sig_exp = parts.next()?.parse::<u32>().ok()?;
    let sig_inc = parts.next()?.parse::<u32>().ok()?;
    let key_tag = parts.next()?.parse::<u16>().ok()?;
    let signer = parts.next()?;
    let signature_b64 = parts.next()?;
    let sig = base64::engine::general_purpose::STANDARD
        .decode(signature_b64)
        .ok()?;

    let mut out = Vec::new();
    out.extend_from_slice(&type_covered.to_be_bytes());
    out.push(algorithm);
    out.push(labels);
    out.extend_from_slice(&orig_ttl.to_be_bytes());
    out.extend_from_slice(&sig_exp.to_be_bytes());
    out.extend_from_slice(&sig_inc.to_be_bytes());
    out.extend_from_slice(&key_tag.to_be_bytes());

    out.extend_from_slice(&crate::dns::engine::encode_name_labels_vec(signer));
    out.extend_from_slice(&sig);
    Some(out)
}

use crate::dns::registry::Registry;

pub fn register(reg: &mut Registry) {
    reg.register("RRSIG", encode);
}
