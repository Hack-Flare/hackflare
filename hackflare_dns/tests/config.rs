#![allow(clippy::unwrap_used)]

use hackflare_dns::ns::config::*;

#[test]
fn default_config_matches_nameserver_defaults() {
    let config = NsConfig::default();
    assert_eq!(config.bind_addr, "0.0.0.0");
    assert_eq!(config.port, 53);
    assert_eq!(config.zone_file, None);
    assert_eq!(config.database_url, None);
}
