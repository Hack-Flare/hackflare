#![allow(clippy::unwrap_used)]

use hackflare_dns::dns::config::*;
use std::time::Duration;

#[test]
fn default_config_has_sensible_defaults() {
    let cfg = DnsConfig::default_config();
    assert!(!cfg.recursion_enabled);
    assert_eq!(cfg.soa_mname, "ns1.hackflare.net.");
    assert_eq!(cfg.soa_rname, "dns.hackflare.net.");
    assert_eq!(cfg.soa_refresh, 3600);
    assert_eq!(cfg.soa_retry, 600);
    assert_eq!(cfg.soa_expire, 604_800);
    assert_eq!(cfg.soa_minimum, 3_600);
    assert_eq!(cfg.soa_ttl, 3600);
    assert_eq!(
        cfg.nameservers,
        vec![
            "ns1.hackflare.net.".to_string(),
            "ns2.hackflare.net.".to_string()
        ]
    );
    assert_eq!(cfg.max_edns_payload_size, 1232);
    assert_eq!(cfg.udp_attempts, 4);
    assert_eq!(cfg.recursion_rounds, 8);
    assert!(!cfg.recursion_debug);
    assert_eq!(cfg.udp_timeout, Duration::from_millis(2500));
}

#[test]
fn env_bool_parses_correctly() {
    assert!(env_bool("NONEXISTENT_VAR_12345", true));
    assert!(!env_bool("NONEXISTENT_VAR_12345", false));
}

#[test]
fn env_list_splits_and_trims() {
    let default = default_nameservers();
    assert_eq!(env_list("NONEXISTENT_VAR_12345", default.clone()), default);
    // comma-separated, entries trimmed, empty entries dropped
    assert_eq!(
        parse_list("ns1.x.net., ns2.x.net."),
        vec!["ns1.x.net.".to_string(), "ns2.x.net.".to_string()]
    );
    assert!(parse_list(",  ,").is_empty());
}
