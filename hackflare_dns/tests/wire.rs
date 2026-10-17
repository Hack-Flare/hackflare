#![allow(clippy::unwrap_used)]

use hackflare_dns::dns::wire::*;

#[test]
fn wire_encodes_labels() {
    assert_eq!(
        encode_name_labels_vec("www.example.com"),
        vec![
            3, b'w', b'w', b'w', 7, b'e', b'x', b'a', b'm', b'p', b'l', b'e', 3, b'c', b'o', b'm',
            0
        ]
    );
}

#[test]
fn wire_roundtrip_qname() {
    let parsed = parse_qname(&encode_name_labels_vec("www.example.com"), 0)
        .expect("encoded name should parse");
    assert_eq!(parsed.0, "www.example.com");
}

#[test]
fn parse_qname_rejects_pointer_loop() {
    // Two pointers pointing at each other: pos 0 → pos 12, pos 12 → pos 0
    let wire = [
        0xC0, 0x0C, // pointer to offset 12
        // padding bytes
        b'x', b'y', b'z', 0, 0, 0, 0, 0, 0, 0, 0xC0, 0x00, // pointer to offset 0
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
        3, b'w', b'w', b'w', 0xC0, 0xFF, // pointer to offset 255 (out of bounds)
    ];
    assert!(parse_qname(&wire, 4).is_none());
}
