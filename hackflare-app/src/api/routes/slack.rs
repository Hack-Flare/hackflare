use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};
use reqwest::Url;
use serde_json::Value;

use crate::{config::Config, state::AppState};

pub async fn slack_contact(
    State(app_state): State<AppState>,
    State(config): State<Arc<Config>>,
    Json(body): Json<Value>,
) -> StatusCode {
    // Read from config_overrides first, fall back to env Config
    let webhook_override = app_state
        .config_overrides
        .get_override("SLACK_WEBHOOK_URL")
        .await
        .ok()
        .flatten();

    let webhook_url = webhook_override
        .and_then(|o| Url::parse(&o.value).ok())
        .or_else(|| config.slack_webhook_url.clone());

    let Some(webhook_url) = webhook_url else {
        return StatusCode::INTERNAL_SERVER_ERROR;
    };

    let client = reqwest::Client::new();
    let res = client.post(webhook_url).json(&body).send().await;

    match res {
        Ok(r) if r.status().is_success() => StatusCode::OK,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
