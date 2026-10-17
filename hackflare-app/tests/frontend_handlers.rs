use axum::http::HeaderMap;
use hackflare_app::frontend::routes::handlers::{hackclub_target_url, safe_return_to};

#[test]
fn hackclub_target_starts_oauth_flow() {
    let target = hackclub_target_url(&HeaderMap::new(), "/dash");
    assert!(target.starts_with("/api/v1/auth/login?target="));
}

#[test]
fn safe_return_to_keeps_local_paths() {
    assert_eq!(
        safe_return_to(Some("/dash/domains".into())),
        "/dash/domains"
    );
}

#[test]
fn safe_return_to_rejects_external_targets() {
    for value in [
        "//attacker.example",
        "/\\attacker.example",
        "https://attacker.example",
        "dash",
    ] {
        assert_eq!(safe_return_to(Some(value.into())), "/dash", "{value}");
    }
    assert_eq!(safe_return_to(None), "/dash");
}
