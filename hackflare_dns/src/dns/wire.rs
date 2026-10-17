/// Encode a domain name into DNS wire format label bytes.
///
/// Each label is prefixed by its length; the name is terminated
/// by a zero-length label.
pub fn encode_name_labels_vec(name: &str) -> Vec<u8> {
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
pub fn parse_qname(buf: &[u8], mut pos: usize) -> Option<(String, usize)> {
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
