use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;

use crate::config::Config;
use crate::state::AppState;

pub fn build_router(config: Config) -> Router {
    let state = AppState::new(config);

    Router::new()
        .route("/health", get(health))
        .route("/internal/v1/ping", get(internal_ping))
        .with_state(state)
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
    visibility: &'static str,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "hackflare-backend",
        visibility: "internal-only",
    })
}

#[derive(Serialize)]
struct InternalPingResponse {
    status: &'static str,
    service: &'static str,
    database_configured: bool,
}

async fn internal_ping(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<InternalPingResponse>, StatusCode> {
    if !is_internal_token_valid(state.config.gateway_internal_token.as_deref(), &headers) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(Json(InternalPingResponse {
        status: "ok",
        service: "hackflare-backend",
        database_configured: state.config.database_url.is_some(),
    }))
}

fn is_internal_token_valid(expected_token: Option<&str>, headers: &HeaderMap) -> bool {
    let Some(expected) = expected_token else {
        return false;
    };

    let Ok(Some(provided)) = headers
        .get("x-internal-token")
        .map(|value| value.to_str())
        .transpose()
    else {
        return false;
    };

    provided == expected
}

#[cfg(test)]
mod tests {
    use super::is_internal_token_valid;
    use axum::http::{HeaderMap, HeaderValue};

    #[test]
    fn validates_matching_token() {
        let mut headers = HeaderMap::new();
        headers.insert("x-internal-token", HeaderValue::from_static("secret"));

        assert!(is_internal_token_valid(Some("secret"), &headers));
    }

    #[test]
    fn rejects_wrong_token() {
        let mut headers = HeaderMap::new();
        headers.insert("x-internal-token", HeaderValue::from_static("wrong"));

        assert!(!is_internal_token_valid(Some("secret"), &headers));
    }

    #[test]
    fn rejects_missing_expected_token() {
        let headers = HeaderMap::new();

        assert!(!is_internal_token_valid(None, &headers));
    }
}
