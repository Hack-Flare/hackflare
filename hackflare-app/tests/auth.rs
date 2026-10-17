use hackflare_app::auth::routes::make_cookie;

#[test]
fn cookies_are_secure_only_for_https_deployments() {
    let secure = make_cookie("jwt".into(), "token".into(), "/".into(), 60, true);
    let local = make_cookie("jwt".into(), "token".into(), "/".into(), 60, false);

    assert!(secure.to_string().contains("; Secure"));
    assert!(!local.to_string().contains("; Secure"));
}
