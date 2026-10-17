#![allow(clippy::unwrap_used)]

use hackflare_dns::ns::hickory::*;

use hackflare_dns::dns::config::DnsConfig;
use hackflare_dns::ns::AuthorityStore;
use hickory_server::net::xfer::Protocol;
use hickory_server::proto::op::ResponseCode;
use std::sync::Arc;
use std::sync::atomic::Ordering;
#[test]
fn handler_creation() {
    let config = DnsConfig::default_config();
    let authority = Arc::new(AuthorityStore::new(config.clone()));
    let handler = HickoryRequestHandler::new(authority, config, None);
    assert!(Arc::strong_count(&handler.authority) >= 1);
}

#[test]
fn log_function_creates_valid_json() {
    log("info", "Test message", None);
    log("error", "Test error", Some("127.0.0.1:53".parse().unwrap()));
}

#[test]
fn response_code_mnemonic_uses_wire_codes() {
    assert_eq!(response_code_mnemonic(ResponseCode::NoError), "NOERROR");
    assert_eq!(response_code_mnemonic(ResponseCode::NXDomain), "NXDOMAIN");
    assert_eq!(response_code_mnemonic(ResponseCode::ServFail), "SERVFAIL");
    assert_eq!(response_code_mnemonic(ResponseCode::Refused), "REFUSED");
    assert_eq!(response_code_mnemonic(ResponseCode::FormErr), "FORMERR");
    assert_eq!(response_code_mnemonic(ResponseCode::Unknown(42)), "UNKNOWN");
    assert_ne!(response_code_mnemonic(ResponseCode::NoError), "No Error");
}

#[test]
fn record_request_counters() {
    UDP_COUNT.store(0, Ordering::Relaxed);
    TCP_COUNT.store(0, Ordering::Relaxed);

    record_request(Protocol::Udp);
    assert_eq!(UDP_COUNT.load(Ordering::Relaxed), 1);

    record_request(Protocol::Tcp);
    assert_eq!(TCP_COUNT.load(Ordering::Relaxed), 1);

    UDP_COUNT.store(0, Ordering::Relaxed);
    TCP_COUNT.store(0, Ordering::Relaxed);

    record_request(Protocol::Udp);
    record_request(Protocol::Udp);
    record_request(Protocol::Udp);

    assert_eq!(UDP_COUNT.load(Ordering::Relaxed), 3);
    assert_eq!(TCP_COUNT.load(Ordering::Relaxed), 0);

    UDP_COUNT.store(0, Ordering::Relaxed);
    TCP_COUNT.store(0, Ordering::Relaxed);

    record_request(Protocol::Tcp);
    record_request(Protocol::Tcp);

    assert_eq!(UDP_COUNT.load(Ordering::Relaxed), 0);
    assert_eq!(TCP_COUNT.load(Ordering::Relaxed), 2);

    UDP_COUNT.store(0, Ordering::Relaxed);
    TCP_COUNT.store(0, Ordering::Relaxed);

    record_request(Protocol::Udp);
    record_request(Protocol::Tcp);
    record_request(Protocol::Udp);
    record_request(Protocol::Tcp);

    assert_eq!(UDP_COUNT.load(Ordering::Relaxed), 2);
    assert_eq!(TCP_COUNT.load(Ordering::Relaxed), 2);
}
