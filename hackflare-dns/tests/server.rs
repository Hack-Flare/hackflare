#![allow(clippy::unwrap_used)]

use hackflare_dns::ns::server::*;

use hackflare_dns::dns::config::DnsConfig;
use hackflare_dns::ns::config::NsConfig;

#[test]
fn nameserver_constructors_set_expected_state() {
    let config = NsConfig {
        bind_addr: "127.0.0.1".to_string(),
        port: 5300,
        zone_file: Some("zones.json".to_string()),
        database_url: None,
    };

    let empty = Nameserver::new(config).unwrap();
    assert_eq!(empty.config.bind_addr, "127.0.0.1");
    assert_eq!(empty.config.port, 5300);
    assert_eq!(empty.config.zone_file.as_deref(), Some("zones.json"));
    assert!(empty.db_pool.is_none());
    assert!(empty.list_zones().is_empty());

    let with_engine =
        Nameserver::with_dns_config(NsConfig::default(), DnsConfig::default_config()).unwrap();
    assert!(with_engine.db_pool.is_none());
    assert!(with_engine.list_zones().is_empty());
}
