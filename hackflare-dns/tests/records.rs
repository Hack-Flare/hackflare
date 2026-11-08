#![allow(clippy::unwrap_used)]

use std::str::FromStr;

use hackflare_dns::dns::config::DnsConfig;
use hackflare_dns::dns::packet::{Message, Protocol};
use hackflare_dns::dns::rr::{RData, RECORD_TYPES, RecordType};
use hackflare_dns::ns::authority::AuthorityStore;

#[test]
fn record_type_names_roundtrip() {
    for name in RECORD_TYPES {
        let record_type = RecordType::from_str(name).unwrap();
        assert_eq!(record_type.to_string(), *name);
        assert_eq!(RecordType::from_u16(record_type.as_u16()), record_type);
    }
}

#[test]
fn parses_and_displays_common_records() {
    let cases = [
        ("A", "192.0.2.1", "192.0.2.1"),
        ("AAAA", "2001:db8::1", "2001:db8::1"),
        ("CNAME", "example.com", "example.com."),
        ("DNAME", "example.com", "example.com."),
        ("MX", "10 mail.example.com", "10 mail.example.com."),
        ("KX", "10 mail.example.com", "10 mail.example.com."),
        ("TXT", "\"v=spf1 ~all\"", "v=spf1 ~all"),
        ("SPF", "\"v=spf1 ~all\"", "v=spf1 ~all"),
        (
            "SRV",
            "10 5 5060 sip.example.com",
            "10 5 5060 sip.example.com.",
        ),
        (
            "CAA",
            "0 issue \"letsencrypt.org\"",
            "0 issue \"letsencrypt.org\"",
        ),
        ("HINFO", "\"x86_64\" \"Linux\"", "\"x86_64\" \"Linux\""),
        (
            "NAPTR",
            "100 10 \"S\" \"SIP+D2U\" \"\" _sip._udp.example.com",
            "100 10 \"S\" \"SIP+D2U\" \"\" _sip._udp.example.com.",
        ),
        ("DS", "12345 13 2 49fd46e6", "12345 13 2 49fd46e6"),
        ("CDS", "12345 13 2 49fd46e6", "12345 13 2 49fd46e6"),
        ("DNSKEY", "257 3 13 YWJj", "257 3 13 YWJj"),
        ("CDNSKEY", "257 3 13 YWJj", "257 3 13 YWJj"),
        ("TLSA", "3 1 1 abcd", "3 1 1 abcd"),
        ("SMIMEA", "3 1 1 abcd", "3 1 1 abcd"),
        ("SSHFP", "1 1 abcd", "1 1 abcd"),
        (
            "SVCB",
            "1 svc.example.com alpn=h2,h3 port=443",
            "1 svc.example.com. alpn=h2,h3 port=443",
        ),
        ("HTTPS", "1 . alpn=h2", "1 . alpn=h2"),
        (
            "URI",
            "10 1 \"https://example.com\"",
            "10 1 \"https://example.com\"",
        ),
        ("CERT", "1 12345 13 YWJj", "1 12345 13 YWJj"),
        ("EUI48", "00-11-22-33-44-55", "00-11-22-33-44-55"),
        (
            "EUI64",
            "00-11-22-33-44-55-66-77",
            "00-11-22-33-44-55-66-77",
        ),
    ];

    for (rtype, input, expected) in cases {
        let record_type = RecordType::from_str(rtype).unwrap();
        let data = RData::parse(record_type, input).unwrap();
        assert_eq!(data.rtype(), record_type, "type mismatch for {rtype}");
        assert_eq!(data.to_string(), expected, "display mismatch for {rtype}");
        assert!(!data.to_wire().is_empty(), "empty wire data for {rtype}");
    }
}

#[test]
fn parses_rfc3597_generic_records() {
    let data = RData::parse(RecordType::Loc, "\\# 4 01020304").unwrap();
    assert_eq!(data.rtype(), RecordType::Loc);
    assert_eq!(data.to_wire(), vec![1, 2, 3, 4]);
    assert_eq!(data.to_string(), "\\# 4 01020304");

    let data = RData::parse(RecordType::Unknown(65280), "\\# 2 BEEF").unwrap();
    assert_eq!(data.rtype(), RecordType::Unknown(65280));
    assert_eq!(data.to_wire(), vec![0xBE, 0xEF]);
}

#[test]
fn rejects_invalid_or_meta_records() {
    assert!(RData::parse(RecordType::A, "not-an-address").is_none());
    assert!(RData::parse(RecordType::Loc, "not generic").is_none());
    assert!(RData::parse(RecordType::Opt, "\\# 0").is_none());
    assert!(RData::parse(RecordType::Any, "\\# 0").is_none());
}

#[tokio::test]
async fn authority_serves_extended_record_types() {
    let store = AuthorityStore::new(DnsConfig::default_config());
    assert!(store.create_zone("example.com").await);

    assert!(
        store
            .add_record(
                "example.com",
                "_sip._tcp",
                "SRV",
                300,
                "10 5 5060 sip.example.com"
            )
            .await
    );
    assert!(
        store
            .add_record(
                "example.com",
                "@",
                "CAA",
                300,
                "0 issue \"letsencrypt.org\""
            )
            .await
    );
    assert!(
        store
            .add_record("example.com", "loc", "LOC", 300, "\\# 4 01020304")
            .await
    );

    let query = Message::query(1, "_sip._tcp.example.com", RecordType::Srv.as_u16());
    let response = store
        .answer(&query, "127.0.0.1:5300".parse().unwrap(), Protocol::Udp)
        .await;
    assert_eq!(response.answers.len(), 1);
    assert_eq!(response.answers[0].rtype, RecordType::Srv.as_u16());

    let query = Message::query(2, "loc.example.com", RecordType::Loc.as_u16());
    let response = store
        .answer(&query, "127.0.0.1:5300".parse().unwrap(), Protocol::Udp)
        .await;
    assert_eq!(response.answers.len(), 1);
    assert_eq!(response.answers[0].rdata, vec![1, 2, 3, 4]);
}
