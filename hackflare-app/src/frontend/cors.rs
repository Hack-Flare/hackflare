use axum::http::{HeaderValue, Method, header};
use reqwest::Url;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};

use crate::config::Config;

pub(crate) fn layer(config: &Config) -> CorsLayer {
    let Some(domain) = config.domain.as_ref() else {
        return CorsLayer::new();
    };

    let origins = std::iter::once(domain.clone())
        .chain(cdn_origin(domain))
        .filter_map(|url| HeaderValue::try_from(url.as_str()).ok())
        .collect::<Vec<_>>();

    CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any)
        .expose_headers([header::CONTENT_TYPE])
        .allow_credentials(true)
}

fn cdn_origin(domain: &Url) -> Option<Url> {
    let host = domain.host_str()?;
    if host.parse::<std::net::IpAddr>().is_ok() {
        return None;
    }

    let mut cdn = domain.clone();
    cdn.set_host(Some(&format!("cdn.{host}"))).ok()?;
    Some(cdn)
}
