use std::sync::Arc;

use axum::{
    Router,
    extract::{Query, State},
    http::{HeaderValue, header},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
};
use axum_client_ip::ClientIp;
use axum_extra::extract::CookieJar;
use chrono::{Duration, Utc};
use jsonwebtoken::{Header, Validation};
use rand::{RngExt, distr::Alphanumeric};
use reqwest::{StatusCode, Url};
use serde::Deserialize;
use serde_json::json;
use serde_with::{DurationSeconds, serde_as};
use sqlx::PgPool;
use tower_sessions::{
    Expiry, MemoryStore, Session, SessionManagerLayer,
    cookie::{self, Cookie, SameSite},
};
use uuid::Uuid;

use crate::{
    config::Config,
    models::{HcaUser, JwtClaims},
    services::{user_sessions::UserSessionsService, users::UsersService},
    state::AppState,
};

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

    let url = Url::parse_with_params(path, params)
        .expect("failed to build HCA authorize URL from hardcoded base");

    url.to_string()
}

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

fn make_cookie(
    name: String,
    value: String,
    path: String,
    max_age_seconds: i64,
    is_secure: bool,
) -> cookie::Cookie<'static> {
    let mut c = Cookie::build((name, value))
        .path(path)
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(cookie::time::Duration::seconds(max_age_seconds));
    if is_secure {
        c = c.secure(true);
    }
    c.build()
}

fn make_tokens(
    config: &Config,
    jit: Uuid,
    user_id: &str,
    now: chrono::DateTime<Utc>,
) -> Result<(String, String), (StatusCode, &'static str)> {
    let access_exp = now + chrono::Duration::minutes(config.access_token_minutes);
    let refresh_exp = now + chrono::Duration::days(config.refresh_token_days);

    let access_claims = JwtClaims {
        sub: user_id.to_string(),
        iat: now,
        jit,
        exp: access_exp,
        typ: None,
    };

    let refresh_claims = JwtClaims {
        sub: user_id.to_string(),
        iat: now,
        jit,
        exp: refresh_exp,
        typ: Some("refresh".to_string()),
    };

    let access_token =
        jsonwebtoken::encode(&Header::default(), &access_claims, &config.jwt_encoding_key)
            .map_err(|error| {
                error!(%error, "failed to encode access jwt");
                (StatusCode::INTERNAL_SERVER_ERROR, "jwt_encode_error")
            })?;

    let refresh_token = jsonwebtoken::encode(
        &Header::default(),
        &refresh_claims,
        &config.jwt_encoding_key,
    )
    .map_err(|error| {
        error!(%error, "failed to encode refresh jwt");
        (StatusCode::INTERNAL_SERVER_ERROR, "jwt_encode_error")
    })?;

    Ok((access_token, refresh_token))
}

fn set_cookie_header(response: &mut Response, cookie: &Cookie<'static>) {
    response.headers_mut().append(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie.to_string()).expect("cookie is valid header value"),
    );
}

fn make_auth_cookies(
    config: &Config,
    access_token: String,
    refresh_token: String,
    is_secure: bool,
) -> (Cookie<'static>, Cookie<'static>) {
    let access = make_cookie(
        "jwt".into(),
        access_token,
        "/".into(),
        config.access_token_minutes * 60,
        is_secure,
    );
    let refresh = make_cookie(
        "refresh_jwt".into(),
        refresh_token,
        "/api/v1/auth".into(),
        config.refresh_token_days * 86400,
        is_secure,
    );
    (access, refresh)
}

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
    State(db): State<PgPool>,
    session: Session,
    Query(query): Query<AuthCallbackParams>,
    ClientIp(ip_addr): ClientIp,
) -> Result<Response, (StatusCode, &'static str)> {
    info!("callback_handler called with code={:?}", query.code);

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
        warn!(query.csrf_token, session_csrf_token, "csrf token mismatch");
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

    let mut tx = db.begin().await.map_err(|error| {
        error!(%error, "failed to start transaction");
        (StatusCode::INTERNAL_SERVER_ERROR, "db_error")
    })?;

    UsersService::upsert_with(
        &mut *tx,
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
    let refresh_exp = now + chrono::Duration::days(config.refresh_token_days);

    let jit = UserSessionsService::create_with(&mut *tx, &user_info.id, ip_addr, refresh_exp)
        .await
        .map_err(|error| {
            error!(%error, "failed to create session");
            (StatusCode::INTERNAL_SERVER_ERROR, "db_error")
        })?;

    tx.commit().await.map_err(|error| {
        error!(%error, "failed to commit transaction");
        (StatusCode::INTERNAL_SERVER_ERROR, "db_error")
    })?;

    let (access_token, refresh_token) = make_tokens(&config, jit, &user_info.id, now)?;

    let is_secure = config.hca.is_secure();
    let (access_cookie, refresh_cookie) =
        make_auth_cookies(&config, access_token, refresh_token, is_secure);

    let target_url = session_target_url
        .as_deref()
        .filter(|u| {
            if u.starts_with('/') && !u.contains("://") && !u.contains("\\") {
                return true;
            }
            // Accept absolute URLs on the same host (for dev: different port)
            if let Ok(url) = Url::parse(u) {
                return url.host_str() == config.hca.redirect_uri.host_str() && !u.contains("\\");
            }
            false
        })
        .unwrap_or("/");

    let mut response = (StatusCode::FOUND, ()).into_response();
    set_cookie_header(&mut response, &access_cookie);
    set_cookie_header(&mut response, &refresh_cookie);
    response.headers_mut().append(
        header::LOCATION,
        HeaderValue::from_str(target_url).expect("target url is valid header value"),
    );

    info!(
        "response set-cookie headers: {:?}",
        response
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .collect::<Vec<_>>(),
    );

    Ok(response)
}

async fn logout_handler(
    State(state): State<AppState>,
    State(sessions): State<UserSessionsService>,
    jar: CookieJar,
) -> Response {
    let is_secure = state.config.hca.is_secure();

    if let Some(jwt) = jar.get("jwt")
        && let Ok(data) = jsonwebtoken::decode::<JwtClaims>(
            jwt.value(),
            &state.config.jwt_decoding_key,
            &Validation::default(),
        )
    {
        let jit = data.claims.jit;
        if let Err(e) = sessions.revoke(&jit).await {
            error!(%e, "failed to revoke session");
        }
    }

    let clear_access = make_cookie("jwt".into(), "".into(), "/".into(), 0, is_secure);
    let clear_refresh = make_cookie(
        "refresh_jwt".into(),
        "".into(),
        "/api/v1/auth".into(),
        0,
        is_secure,
    );

    let mut response = (StatusCode::NO_CONTENT, ()).into_response();
    set_cookie_header(&mut response, &clear_access);
    set_cookie_header(&mut response, &clear_refresh);
    response
}

async fn refresh_handler(
    jar: CookieJar,
    State(config): State<Arc<Config>>,
    State(sessions): State<UserSessionsService>,
) -> Result<Response, (StatusCode, &'static str)> {
    let refresh_jwt = jar
        .get("refresh_jwt")
        .map(|c| c.value().to_owned())
        .ok_or((StatusCode::UNAUTHORIZED, "missing_refresh_token"))?;

    let claims = jsonwebtoken::decode::<JwtClaims>(
        &refresh_jwt,
        &config.jwt_decoding_key,
        &Validation::default(),
    )
    .map_err(|error| {
        debug!(%error, "refresh jwt validation failed");
        (StatusCode::UNAUTHORIZED, "invalid_refresh_token")
    })?
    .claims;

    if claims.typ.as_deref() != Some("refresh") {
        warn!("access token used as refresh token");
        return Err((StatusCode::UNAUTHORIZED, "invalid_token_type"));
    }

    let session = sessions.get_by_id(&claims.jit).await.map_err(|error| {
        error!(%error, "failed to get session during refresh");
        (StatusCode::INTERNAL_SERVER_ERROR, "db_error")
    })?;

    let Some(_session) = session else {
        warn!("session revoked or expired during refresh");
        return Err((StatusCode::UNAUTHORIZED, "session_invalid"));
    };

    let now = Utc::now();
    let (access_token, refresh_token) = make_tokens(&config, claims.jit, &claims.sub, now)?;

    let is_secure = config.hca.is_secure();
    let (access_cookie, refresh_cookie) =
        make_auth_cookies(&config, access_token, refresh_token, is_secure);

    let mut response = (StatusCode::OK, ()).into_response();
    set_cookie_header(&mut response, &access_cookie);
    set_cookie_header(&mut response, &refresh_cookie);
    Ok(response)
}

pub(super) fn routes(config: &Config) -> Router<AppState> {
    let is_secure = config.hca.is_secure();

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_expiry(Expiry::OnInactivity(cookie::time::Duration::minutes(
            config.session_inactivity_minutes,
        )))
        .with_secure(is_secure)
        .with_same_site(SameSite::Lax);

    debug!(is_secure, "setting up auth routes");

    Router::new()
        .route("/login", get(login_handler))
        .route("/callback", get(callback_handler))
        .route("/refresh", post(refresh_handler))
        .route("/logout", post(logout_handler))
        .layer(session_layer)
}
