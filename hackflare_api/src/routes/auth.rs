use std::sync::Arc;

use axum::{
    Router,
    extract::{Query, State},
    http::header,
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
};
use chrono::{Duration, Utc};
use jsonwebtoken::Header;
use rand::{RngExt, distr::Alphanumeric};
use reqwest::{StatusCode, Url};
use serde::Deserialize;
use serde_json::json;
use serde_with::{DurationSeconds, serde_as};
use tower_sessions::{
    Expiry, MemoryStore, Session, SessionManagerLayer,
    cookie::{self, Cookie, SameSite},
};

use crate::{config::Config, models::{HcaUser, JwtClaims}, services::users::UsersService, state::AppState};

fn login_redirect(config: &Config, csrf_token: &str) -> String {
    let scopes = "email name profile verification_status slack_id";

    let path = "https://auth.hackclub.com/oauth/authorize";
    let params = [
        ("client_id", config.hca.client_id.as_str()),
        ("redirect_uri", config.hca.redirect_uri.as_str()),
        ("response_type", "code"),
        ("scope", scopes),
        ("state", csrf_token),
    ];

    let url = Url::parse_with_params(path, params).unwrap();

    url.to_string()
}

// TODO: make this a config option?
const SESSION_DURATION_HOURS: i64 = 24;

#[derive(Debug, Deserialize)]
struct LoginParams {
    #[serde(rename = "target")]
    target_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AuthCallbackParams {
    code: String,
    #[serde(rename = "state")]
    csrf_token: String,
}

#[derive(Deserialize)]
enum TokenType {
    Bearer,
}

#[serde_as]
#[allow(unused)]
#[derive(Deserialize)]
struct HcaTokenResponse {
    access_token: String,
    token_type: TokenType,
    #[serde_as(as = "DurationSeconds<i64>")]
    expires_in: Duration,
    refresh_token: String,
    scope: String,
}

#[derive(Debug, Deserialize)]
struct HcaUserdataResponse {
    identity: HcaUser,
    scopes: Vec<String>,
}

/// Generate a random alphanumeric string that is `len` characters long.
fn random_string(len: usize) -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

const SESSION_CSRF_TOKEN_KEY: &str = "auth::csrf_token";
const SESSION_TARGET_URL_KEY: &str = "auth::target_url";

async fn login_handler(
    State(state): State<AppState>,
    session: Session,
    Query(LoginParams { target_url }): Query<LoginParams>,
) -> Redirect {
    let csrf_token = random_string(32);

    session
        .insert(SESSION_CSRF_TOKEN_KEY, &csrf_token)
        .await
        .expect("failed to set csrf token in session");
    if let Some(target_url) = target_url.as_ref() {
        session
            .insert(SESSION_TARGET_URL_KEY, &target_url)
            .await
            .expect("failed to set target url in session");
    } else {
        session
            .remove::<String>(SESSION_TARGET_URL_KEY)
            .await
            .expect("failed to set target url in session");
    }
    trace!(target_url, "persisted login state");

    let redirect = login_redirect(&state.config, &csrf_token);
    Redirect::to(&redirect)
}

async fn callback_handler(
    State(config): State<Arc<Config>>,
    State(http_client): State<reqwest::Client>,
    State(users): State<UsersService>,
    session: Session,
    Query(query): Query<AuthCallbackParams>,
) -> Result<Response, (StatusCode, &'static str)> {
    let session_csrf_token: String = session
        .remove(SESSION_CSRF_TOKEN_KEY)
        .await
        .expect("failed to get csrf token from session")
        .ok_or((StatusCode::BAD_REQUEST, "missing_auth_state"))?;

    let session_target_url: Option<String> = session
        .remove(SESSION_TARGET_URL_KEY)
        .await
        .expect("failed to get target url from session");

    if query.csrf_token != session_csrf_token {
        // TODO: should this be a warn! event instead?
        debug!(query.csrf_token, session_csrf_token, "csrf token mismatch");
        return Err((StatusCode::BAD_REQUEST, "csrf_token_mismatch"));
    }

    trace!(
        query.code,
        query.csrf_token,
        ?session_target_url,
        "got auth callback"
    );

    let payload = json!({
        "client_id": config.hca.client_id,
        "client_secret": config.hca.client_secret,
        "redirect_uri": config.hca.redirect_uri.to_string(),
        "code": query.code,
        "grant_type": "authorization_code",
    });

    let token_request_sent_at = Utc::now();
    let response = http_client
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

    let token_response = response.json::<HcaTokenResponse>().await.map_err(|error| {
        error!(%error, "failed to parse HCA success JSON");
        (StatusCode::INTERNAL_SERVER_ERROR, "token_parse_failed")
    })?;

    let user_response = http_client
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

    let hca_response = user_response
        .json::<HcaUserdataResponse>()
        .await
        .map_err(|e| {
            error!(%e, "Failed to parse HCA User JSON");
            (StatusCode::INTERNAL_SERVER_ERROR, "invalid_user_data")
        })?;

    let user_info = hca_response.identity;

    debug!(user_info.first_name, user_info.last_name, ?hca_response.scopes, "login successful");

    // NB: we capture the time *before* sending the request - this slightly underestimates
    // the token lifetime, but that's the safer tradeoff: treating a valid token as expired
    // is harmless, while treating an expired token as valid is a security issue.
    let token_expires_at = token_request_sent_at + token_response.expires_in;

    users
        .upsert(
            &user_info,
            &token_response.access_token,
            &token_response.refresh_token,
            token_expires_at,
        )
        .await
        .map_err(|error| {
            error!(%error, "failed to upsert user");
            (StatusCode::INTERNAL_SERVER_ERROR, "db_error")
        })?;

    let now = Utc::now();
    let claims = JwtClaims {
        sub: user_info.id,
        iat: now,
        exp: now + chrono::Duration::hours(SESSION_DURATION_HOURS),
    };

    let token = jsonwebtoken::encode(&Header::default(), &claims, &config.jwt_encoding_key)
        .map_err(|error| {
            error!(%error, "failed to encode jwt");
            (StatusCode::INTERNAL_SERVER_ERROR, "jwt_encode_error")
        })?;

    let is_secure = config.hca.is_secure();

    let cookie = Cookie::build(("jwt", token))
        .path("/")
        .http_only(true)
        .secure(is_secure)
        // NB: use lax to e.g. allow for email links etc. while still protecting from CSRF
        .same_site(SameSite::Lax)
        .max_age(cookie::time::Duration::hours(SESSION_DURATION_HOURS))
        .build();

    // TODO: do we have to validate this cookie? is it not HttpOnly already?
    // would it allow for open redirects to other origins?
    // https://github.com/Hack-Flare/hackflare/pull/34#discussion_r3230192477
    let target_url = session_target_url.as_deref().unwrap_or("/");

    Ok((
        StatusCode::FOUND,
        [
            (header::SET_COOKIE, cookie.to_string().as_str()),
            (header::LOCATION, target_url),
        ],
    )
        .into_response())
}

async fn logout_handler(State(state): State<AppState>) -> Response {
    let is_secure = state.config.hca.is_secure();

    let cookie = Cookie::build(("jwt", ""))
        .path("/")
        .http_only(true)
        .secure(is_secure)
        .same_site(SameSite::Lax)
        .max_age(cookie::time::Duration::ZERO)
        .build();

    (
        StatusCode::NO_CONTENT,
        [(header::SET_COOKIE, cookie.to_string())],
    )
        .into_response()
}

pub(super) fn routes(config: &Config) -> Router<AppState> {
    let is_secure = config.hca.is_secure();

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        // TODO: make this duration a config option
        .with_expiry(Expiry::OnInactivity(cookie::time::Duration::minutes(15)))
        .with_secure(is_secure)
        .with_same_site(SameSite::Lax);

    debug!(is_secure, "setting up auth routes");

    Router::new()
        .route("/login", get(login_handler))
        .route("/callback", get(callback_handler))
        .route("/logout", post(logout_handler))
        .layer(session_layer)
}
