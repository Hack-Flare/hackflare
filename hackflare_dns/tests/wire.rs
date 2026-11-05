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

#[test]
fn packet_roundtrip_message_with_answer_and_edns() {
    use hackflare_dns::dns::packet::{Edns, Message, Record, TYPE_A};

    let mut message = Message::query(42, "www.example.com", TYPE_A);
    message.answers.push(Record {
        name: "www.example.com".to_string(),
        rtype: TYPE_A,
        class: 1,
        ttl: 300,
        rdata: vec![192, 0, 2, 1],
        rdata_offset: 0,
    });
    message.edns = Some(Edns {
        udp_payload_size: 1232,
        extended_rcode: 0,
        version: 0,
        flags: 0x8000,
        options: Vec::new(),
    });

    let bytes = message.encode().unwrap();
    let parsed = Message::parse(&bytes).unwrap();

    assert_eq!(parsed.header.id, 42);
    assert_eq!(parsed.questions.len(), 1);
    assert_eq!(parsed.questions[0].name, "www.example.com");
    assert_eq!(parsed.answers.len(), 1);
    assert_eq!(parsed.answers[0].rdata, vec![192, 0, 2, 1]);
    assert_eq!(parsed.edns.unwrap().udp_payload_size, 1232);
}

#[test]
fn packet_roundtrip_compressed_owner_names() {
    use hackflare_dns::dns::packet::{Message, Record, TYPE_A};

    let mut message = Message::query(1, "www.example.com", TYPE_A);
    for last in 1..=2 {
        message.answers.push(Record {
            name: "www.example.com".to_string(),
            rtype: TYPE_A,
            class: 1,
            ttl: 300,
            rdata: vec![192, 0, 2, last],
            rdata_offset: 0,
        });
    }

    let bytes = message.encode().unwrap();
    let parsed = Message::parse(&bytes).unwrap();
    assert_eq!(parsed.answers.len(), 2);
    assert_eq!(parsed.answers[0].name, "www.example.com");
    assert_eq!(parsed.answers[1].name, "www.example.com");
}

#[test]
fn packet_udp_truncation_keeps_question_and_sets_tc() {
    use hackflare_dns::dns::packet::{Message, Record, TYPE_A};

    let mut message = Message::query(1, "www.example.com", TYPE_A);
    for last in 0..40 {
        message.answers.push(Record {
            name: "www.example.com".to_string(),
            rtype: TYPE_A,
            class: 1,
            ttl: 300,
            rdata: vec![192, 0, 2, last],
            rdata_offset: 0,
        });
    }

    let bytes = message.encode_udp(512).unwrap();
    assert!(bytes.len() <= 512);
    let parsed = Message::parse(&bytes).unwrap();
    assert!(parsed.header.truncated());
    assert_eq!(parsed.questions.len(), 1);
    assert!(parsed.answers.is_empty());
}

#[test]
fn packet_parse_rejects_short_header() {
    use hackflare_dns::dns::packet::{Message, PacketError};

    assert_eq!(
        Message::parse(&[0u8; 11]).unwrap_err(),
        PacketError::Truncated
    );
}

#[test]
fn packet_response_to_echoes_query_flags() {
    use hackflare_dns::dns::packet::{Message, Rcode, TYPE_A};

    let query = Message::query(9, "example.com", TYPE_A);
    let response = Message::response_to(&query);

    assert_eq!(response.header.id, 9);
    assert!(response.header.is_response());
    assert!(response.header.recursion_desired());
    assert_eq!(response.header.rcode(), Rcode::NoError);
    assert_eq!(response.questions, query.questions);
}
