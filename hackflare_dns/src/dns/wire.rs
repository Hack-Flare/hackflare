/// Encode a domain name into DNS wire format label bytes.
///
/// Each label is prefixed by its length; the name is terminated
/// by a zero-length label.
pub(super) fn encode_name_labels_vec(name: &str) -> Vec<u8> {
    let mut out = Vec::new();
    for label in name.split('.') {
        let l = label.len();
        if l == 0 {
            continue;
        }
        out.push(u8::try_from(l).unwrap_or(63));
        out.extend_from_slice(label.as_bytes());
    }
    out.push(0);
    out
}

const MAX_QNAME_LABELS: usize = 127;

/// Parse a DNS name from a buffer starting at the given position.
///
/// Handles DNS name compression pointers with cycle detection.
/// Returns the parsed name and the new position in the buffer,
/// or `None` on error (malformed, pointer loop, too many labels).
pub(super) fn parse_qname(buf: &[u8], mut pos: usize) -> Option<(String, usize)> {
    let mut labels: Vec<String> = Vec::new();
    let mut jumped = false;
    let mut orig_pos = pos;
    let mut visited = std::collections::HashSet::new();
    loop {
        if pos >= buf.len() {
            return None;
        }
        if labels.len() >= MAX_QNAME_LABELS {
            return None;
        }
        let len = buf[pos];
        if len & 0xC0 == 0xC0 {
            if pos + 1 >= buf.len() {
                return None;
            }
            if !visited.insert(pos) {
                return None;
            }
            let b2 = buf[pos + 1];
            let offset = (u16::from(len) & 0x3F) << 8 | u16::from(b2);
            let offset = offset as usize;
            if offset >= buf.len() {
                return None;
            }
            if !jumped {
                orig_pos = pos + 2;
            }
            pos = offset;
            jumped = true;
            continue;
        }
        let l = len as usize;
        pos += 1;
        if l == 0 {
            break;
        }
        if pos + l > buf.len() {
            return None;
        }
        match std::str::from_utf8(&buf[pos..pos + l]) {
            Ok(s) => labels.push(s.to_string()),
            Err(_) => return None,
        }
        pos += l;
    }
    let name = labels.join(".");
    if jumped {
        Some((name, orig_pos))
    } else {
        Some((name, pos))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_encodes_labels() {
        assert_eq!(
            encode_name_labels_vec("www.example.com"),
            vec![
                3, b'w', b'w', b'w', 7, b'e', b'x', b'a', b'm', b'p', b'l', b'e', 3, b'c', b'o',
                b'm', 0
            ]
        );
    }

    #[test]
    fn wire_roundtrip_qname() {
        let parsed = parse_qname(&encode_name_labels_vec("www.example.com"), 0).unwrap();
        assert_eq!(parsed.0, "www.example.com");
    }

    #[test]
    fn parse_qname_rejects_pointer_loop() {
        // Two pointers pointing at each other: pos 0 → pos 12, pos 12 → pos 0
        let wire = [
            0xC0, 0x0C, // pointer to offset 12
            // padding bytes
            b'x', b'y', b'z', 0, 0, 0, 0, 0, 0, 0,
            0xC0, 0x00, // pointer to offset 0
        ];
        assert!(parse_qname(&wire, 0).is_none());
    }

    #[test]
    fn parse_qname_rejects_self_referencing_pointer() {
        // Pointer that points to itself
        let wire = [
            0xC0, 0x00, // pointer to offset 0 (itself)
        ];
        assert!(parse_qname(&wire, 0).is_none());
    }

    #[test]
    fn parse_qname_rejects_too_many_labels() {
        let mut wire = Vec::new();
        for _ in 0..130 {
            wire.push(1);
            wire.push(b'a');
        }
        wire.push(0);
        assert!(parse_qname(&wire, 0).is_none());
    }

    #[test]
    fn parse_qname_rejects_pointer_to_out_of_bounds() {
        let wire = [
            3, b'w', b'w', b'w',
            0xC0, 0xFF, // pointer to offset 255 (out of bounds)
        ];
        assert!(parse_qname(&wire, 4).is_none());
    }
}
