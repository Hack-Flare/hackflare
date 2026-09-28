use axum::http::{HeaderValue, Method, header};
use tower_http::cors::{AllowOrigin, Any, CorsLayer};

use crate::config::Config;

pub(crate) fn layer(config: &Config) -> CorsLayer {
    let Some(frontend_url) = config.frontend_url.as_ref() else {
        return CorsLayer::new();
    };

    let Ok(origin) = HeaderValue::try_from(frontend_url.as_str()) else {
        return CorsLayer::new();
    };

    CorsLayer::new()
        .allow_origin(AllowOrigin::list([origin]))
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any)
        .expose_headers([header::CONTENT_TYPE])
        .allow_credentials(true)
}
