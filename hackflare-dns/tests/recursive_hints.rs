#![allow(clippy::unwrap_used)]

use hackflare_dns::dns::recursive::hints::*;

#[test]
fn root_hint_parser_deduplicates_ips() {
    let hints = parse_root_hints("; comment\n198.41.0.4 198.41.0.4\n170.247.170.2\n");
    assert_eq!(
        hints,
        vec!["170.247.170.2".to_string(), "198.41.0.4".to_string()]
    );
}
