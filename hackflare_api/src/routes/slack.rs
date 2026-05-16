use axum::{http::StatusCode, Json};
use serde_json::Value;

pub async fn slack_contact(Json(body): Json<Value>) -> StatusCode {
    let webhook_url = match std::env::var("SLACK_WEBHOOK_URL") {
        Ok(url) => url,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    let client = reqwest::Client::new();
    let res = client
        .post(webhook_url)
        .json(&body)
        .send()
        .await;

    match res {
        Ok(r) if r.status().is_success() => StatusCode::OK,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}