use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use rand::{RngExt, distr::Alphanumeric};
use reqwest::{StatusCode, Url};
use serde::{Deserialize, Serialize};
use tower_sessions::{
    Expiry, MemoryStore, Session, SessionManagerLayer,
    cookie::{SameSite, time::Duration},
};

use crate::{
    config::{Config, HcaConfig},
    state::AppState,
};

fn hca_login_redirect(config: &HcaConfig, state: &str) -> String {
    let scopes = "email name profile verification_status slack_id";

    // Parse the base authorization endpoint
    let mut url = Url::parse("https://auth.hackclub.com/oauth/authorize").unwrap();

    // Add parameters safely
    url.query_pairs_mut()
        .append_pair("client_id", &config.client_id)
        .append_pair("redirect_uri", config.redirect_uri.as_str())
        .append_pair("response_type", "code")
        .append_pair("scope", scopes)
        .append_pair("state", state);

    url.to_string()
}

#[derive(Serialize)]
struct LoginResponse {
    redirect: String,
}

#[derive(Debug, Deserialize)]
struct AuthCallback {
    code: String,
    state: String,
}

#[derive(Deserialize)]
struct HcaTokenResponse {
    access_token: String,
}

#[derive(Debug, Deserialize)]
pub struct HcaResponse {
    pub identity: HcaUser,
    pub scopes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HcaUser {
    id: String,

    ysws_eligible: bool,
    verification_status: String,

    first_name: String,
    last_name: String,

    primary_email: String,

    slack_id: String,
}

/// Generate a random alphanumeric string that is `len` characters long.
fn random_string(len: usize) -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

async fn login_handler(State(state): State<AppState>, session: Session) -> Json<LoginResponse> {
    debug!("login request");
    let csrf_state = random_string(32);

    session
        .insert("hca_state", &csrf_state)
        .await
        .expect("failed to insert state into session");
    trace!(%csrf_state, "persisted login state");

    let redirect = hca_login_redirect(&state.config.hca, &csrf_state);
    Json(LoginResponse { redirect })
}

// TODO: what is someone forges a request to a foreign provider? => does security hold?
async fn callback_handler(
    State(state): State<AppState>,
    session: Session,
    Path(provider): Path<String>,
    Query(query): Query<AuthCallback>,
) -> Result<Json<HcaUser>, (StatusCode, &'static str)> {
    let csrf_state: String = session
        .remove("hca_state")
        .await
        .expect("failed to get state from session")
        .ok_or((StatusCode::BAD_REQUEST, "missing_state_session"))?;

    if query.state != csrf_state {
        return Err((StatusCode::BAD_REQUEST, "invalid_state"));
    }

    trace!(provider, query.code, query.state, "got auth callback");

    let payload = serde_json::json!({
        "client_id": state.config.hca.client_id,
        "client_secret": state.config.hca.client_secret,
        "redirect_uri": state.config.hca.redirect_uri.to_string(),
        "code": query.code,
        "grant_type": "authorization_code",
    });

    let response = state
        .http_client
        .post("https://auth.hackclub.com/oauth/token")
        .json(&payload)
        .send()
        .await
        .map_err(|e| {
            error!(%e, "hca token exchange request failed");
            (StatusCode::INTERNAL_SERVER_ERROR, "exchange_failed")
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        error!(?status, %body, "HCA token exchange rejected");
        return Err((StatusCode::BAD_REQUEST, "hca_rejected_exchange"));
    }

    let token_response = response.json::<HcaTokenResponse>().await.map_err(|e| {
        error!(%e, "failed to parse HCA success JSON");
        (StatusCode::INTERNAL_SERVER_ERROR, "token_parse_failed")
    })?;

    let user_response = state
        .http_client
        .get("https://auth.hackclub.com/api/v1/me")
        .bearer_auth(&token_response.access_token)
        .send()
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "identity_request_failed"))?;

    if !user_response.status().is_success() {
        let status = user_response.status();
        let error_info = user_response.text().await.unwrap_or_default();
        error!(%status, %error_info, "HCA identity API error");
        return Err((StatusCode::UNAUTHORIZED, "hca_identity_denied"));
    }

    let hca_response = user_response.json::<HcaResponse>().await.map_err(|e| {
        error!(%e, "Failed to parse HCA User JSON");
        (StatusCode::INTERNAL_SERVER_ERROR, "invalid_user_data")
    })?;

    let user_info = hca_response.identity;

    session
        .insert("user_id", user_info.slack_id.clone())
        .await
        .unwrap();
    info!(user_info.first_name, user_info.last_name, ?hca_response.scopes, "Login successful");

    // TODO: Store Token in Database

    Ok(Json(user_info))
}

pub(super) fn routes(config: &Config) -> Router<AppState> {
    let is_secure = config.hca.is_secure();

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        // TODO: make this duration a config option
        .with_expiry(Expiry::OnInactivity(Duration::minutes(15)))
        .with_secure(is_secure)
        .with_same_site(if is_secure {
            // strict on https (prod)
            SameSite::Strict
        } else {
            // lax on http (dev)
            SameSite::Lax
        });

    debug!(is_secure, "setting up auth routes");

    Router::new()
        .route("/login", get(login_handler))
        .route("/callback/{provider}", get(callback_handler))
        .layer(session_layer)
}
