#![allow(clippy::unwrap_used)]

use hackflare_dns::dns::recursive::message::*;
use hackflare_dns::dns::wire::encode_name_labels_vec;

#[test]
fn tld_from_name_extracts_tld() {
    assert_eq!(tld_from_name("WWW.Example.COM."), Some("com".to_string()));
}

#[test]
fn clamp_tld_ttl_clamps_values() {
    assert_eq!(clamp_tld_ttl(10), 3600);
    assert_eq!(clamp_tld_ttl(999_999), 86400);
}

#[test]
fn build_query_and_response_matching() {
    let mut response = build_query(0x1234, "example.com", 1);
    response[2] |= 0x80;
    assert!(response_matches_expected(
        &response,
        0x1234,
        "example.com",
        1
    ));
    assert!(!response_matches_expected(
        &response,
        0x9999,
        "example.com",
        1
    ));
}

#[test]
fn parse_rrs_accepts_class_in() {
    let name = encode_name_labels_vec("www.example.com");
    let mut rr = name.clone();
    rr.extend_from_slice(&1u16.to_be_bytes());
    rr.extend_from_slice(&1u16.to_be_bytes());
    rr.extend_from_slice(&300u32.to_be_bytes());
    rr.extend_from_slice(&4u16.to_be_bytes());
    rr.extend_from_slice(&[192, 168, 1, 1]);
    let result = parse_rrs(&rr, 0, 1);
    assert!(result.is_some());
}

#[test]
fn parse_rrs_rejects_non_in_class() {
    let name = encode_name_labels_vec("www.example.com");
    let mut rr = name.clone();
    rr.extend_from_slice(&1u16.to_be_bytes());
    rr.extend_from_slice(&3u16.to_be_bytes());
    rr.extend_from_slice(&300u32.to_be_bytes());
    rr.extend_from_slice(&4u16.to_be_bytes());
    rr.extend_from_slice(&[192, 168, 1, 1]);
    let result = parse_rrs(&rr, 0, 1);
    assert!(result.is_none());
}

#[test]
fn dns_header_parses_from_wire() {
    let mut bytes = vec![0u8; 12];
    bytes[0] = 0x12;
    bytes[1] = 0x34; // id = 0x1234
    bytes[2] = 0x81;
    bytes[3] = 0x80; // flags: QR + RD + RA
    bytes[4] = 0x00;
    bytes[5] = 0x01; // qdcount = 1
    bytes[6] = 0x00;
    bytes[7] = 0x02; // ancount = 2
    let hdr = DnsHeader::from_wire(&bytes).expect("complete DNS header should parse");
    assert_eq!(hdr.id, 0x1234);
    assert_eq!(hdr.qdcount, 1);
    assert_eq!(hdr.ancount, 2);
    assert!(!hdr.is_truncated());
}

#[test]
fn dns_header_detects_truncation() {
    let mut bytes = vec![0u8; 12];
    bytes[2] = 0x02;
    bytes[3] = 0x00; // TC bit set
    let hdr = DnsHeader::from_wire(&bytes).expect("complete DNS header should parse");
    assert!(hdr.is_truncated());
}

#[test]
fn dns_header_rejects_short_buffer() {
    assert!(DnsHeader::from_wire(&[0u8; 11]).is_none());
}
