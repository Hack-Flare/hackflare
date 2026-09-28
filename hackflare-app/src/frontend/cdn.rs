use axum::{
    body::{Body, to_bytes},
    extract::State,
    http::{Request, header},
    middleware::Next,
    response::Response,
};

use crate::state::AppState;

pub(crate) async fn rewrite_asset_urls(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    let Some(cdn_url) = state.config.cdn_url.as_ref() else {
        return response;
    };
    let is_html = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("text/html"));
    if !is_html {
        return response;
    }

    let (mut parts, body) = response.into_parts();
    let bytes = match to_bytes(body, 4 * 1024 * 1024).await {
        Ok(bytes) => bytes,
        Err(error) => {
            tracing::error!(%error, "failed to read rendered HTML response");
            return Response::from_parts(parts, Body::empty());
        }
    };
    let Ok(html) = String::from_utf8(bytes.to_vec()) else {
        return Response::from_parts(parts, Body::from(bytes));
    };

    let asset_prefix = format!("{}/static/", cdn_url.as_str().trim_end_matches('/'));
    let html = html.replace("/static/", &asset_prefix);
    parts.headers.remove(header::CONTENT_LENGTH);
    response = Response::from_parts(parts, Body::from(html));
    response
}
