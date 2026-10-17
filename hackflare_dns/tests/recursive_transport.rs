#![allow(clippy::unwrap_used)]

use hackflare_dns::dns::recursive::transport::*;

#[test]
fn socket_target_formats_ipv4_and_ipv6() {
    assert_eq!(socket_target("192.0.2.1"), "192.0.2.1:53");
    assert_eq!(socket_target("2001:db8::1"), "[2001:db8::1]:53");
}
