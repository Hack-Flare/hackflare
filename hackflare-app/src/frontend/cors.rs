use axum::http::{HeaderValue, Method, header};
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::config::Config;

pub(crate) fn layer(config: &Config) -> CorsLayer {
    let origins = config
        .domain
        .iter()
        .chain(config.cdn_url.iter())
        .cloned()
        .filter_map(|url| HeaderValue::try_from(url.as_str()).ok())
        .collect::<Vec<_>>();

    if origins.is_empty() {
        return CorsLayer::new();
    }

    CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::ACCEPT, header::AUTHORIZATION, header::CONTENT_TYPE])
        .expose_headers([header::CONTENT_TYPE])
        .allow_credentials(true)
}
