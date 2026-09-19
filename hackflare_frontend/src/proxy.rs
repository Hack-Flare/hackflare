//! Reverse proxy for `/api/*` requests to the backend API.

use std::collections::HashSet;

use axum::{
    body::{Body, to_bytes},
    extract::{Path, RawQuery, State},
    http::{
        Method, StatusCode,
        header::{ACCEPT_ENCODING, CONNECTION, CONTENT_LENGTH, HOST, HeaderMap, HeaderName},
    },
    response::{IntoResponse, Response},
};

use crate::state::AppState;

/// Headers that must never be forwarded across the proxy boundary.
fn hop_by_hop() -> HashSet<HeaderName> {
    [
        CONNECTION,
        HeaderName::from_static("keep-alive"),
        HeaderName::from_static("proxy-authenticate"),
        HeaderName::from_static("proxy-authorization"),
        HeaderName::from_static("te"),
        HeaderName::from_static("trailer"),
        HeaderName::from_static("transfer-encoding"),
        HeaderName::from_static("upgrade"),
        HOST,
        CONTENT_LENGTH,
        ACCEPT_ENCODING,
    ]
    .into_iter()
    .collect()
}

pub async fn proxy(
    State(state): State<AppState>,
    Path(path): Path<String>,
    RawQuery(query): RawQuery,
    method: Method,
    headers: HeaderMap,
    body: Body,
) -> Response {
    let skip = hop_by_hop();

    let mut backend_request = match query {
        Some(query) if !query.is_empty() => state.http_client.request(
            method,
            format!(
                "{}/api/{path}?{query}",
                state.config.api_proxy_target.trim_end_matches('/')
            ),
        ),
        _ => state.http_client.request(
            method,
            format!(
                "{}/api/{path}",
                state.config.api_proxy_target.trim_end_matches('/')
            ),
        ),
    };

    for (name, value) in headers.iter() {
        if skip.contains(name) {
            continue;
        }
        backend_request = backend_request.header(name, value);
    }

    let body_bytes = match to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(e) => {
            tracing::error!(error = %e, "failed to read proxy request body");
            return (StatusCode::BAD_REQUEST, "failed to read request body").into_response();
        }
    };
    let backend_request = backend_request.body(reqwest::Body::from(body_bytes.to_vec()));

    match backend_request.send().await {
        Ok(backend_response) => {
            let status = backend_response.status();
            let mut response_headers = HeaderMap::new();
            for (name, value) in backend_response.headers() {
                if skip.contains(name) {
                    continue;
                }
                response_headers.append(name, value.clone());
            }

            let response_body = match backend_response.bytes().await {
                Ok(bytes) => bytes.to_vec(),
                Err(e) => {
                    tracing::error!(error = %e, "failed to read proxy response body");
                    return (StatusCode::BAD_GATEWAY, "upstream read failed").into_response();
                }
            };

            let mut response = (status, response_body).into_response();
            *response.headers_mut() = response_headers;
            response
        }
        Err(e) => {
            tracing::error!(error = %e, "upstream request failed");
            (
                StatusCode::BAD_GATEWAY,
                axum::Json(serde_json::json!({"error": format!("upstream error: {e}")})),
            )
                .into_response()
        }
    }
}
