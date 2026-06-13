use std::sync::Arc;

use axum::{
    Json, Router,
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
    models::{HcaUser, JwtClaims, db::User},
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

    let user_id = UsersService::upsert_with(
        &mut tx,
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

    let jit = UserSessionsService::create_with(&mut *tx, &user_id, ip_addr, refresh_exp)
        .await
        .map_err(|error| {
            error!(%error, "failed to create session");
            (StatusCode::INTERNAL_SERVER_ERROR, "db_error")
        })?;

    tx.commit().await.map_err(|error| {
        error!(%error, "failed to commit transaction");
        (StatusCode::INTERNAL_SERVER_ERROR, "db_error")
    })?;

    let (access_token, refresh_token) = make_tokens(&config, jit, &user_id, now)?;

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

#[derive(Debug, Deserialize)]
struct RegisterRequest {
    email: String,
    password: String,
    first_name: String,
    last_name: String,
}

#[derive(Debug, Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

async fn register_handler(
    State(state): State<AppState>,
    ClientIp(ip_addr): ClientIp,
    Json(req): Json<RegisterRequest>,
) -> Result<Response, (StatusCode, &'static str)> {
    if req.email.is_empty() || !req.email.contains('@') {
        return Err((StatusCode::BAD_REQUEST, "invalid_email"));
    }
    if req.password.len() < 8 {
        return Err((StatusCode::BAD_REQUEST, "password_too_short"));
    }
    if req.first_name.is_empty() || req.last_name.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "name_required"));
    }

    let existing = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE email = $1")
        .bind(&req.email)
        .fetch_one(&state.db)
        .await
        .map_err(|error| {
            error!(%error, "failed to check existing user");
            (StatusCode::INTERNAL_SERVER_ERROR, "db_error")
        })?;

    if existing > 0 {
        return Err((StatusCode::CONFLICT, "email_already_registered"));
    }

    use argon2::{
        Argon2, PasswordHasher,
        password_hash::{SaltString, rand_core::OsRng},
    };
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(req.password.as_bytes(), &salt)
        .map_err(|error| {
            error!(%error, "failed to hash password");
            (StatusCode::INTERNAL_SERVER_ERROR, "password_hash_error")
        })?
        .to_string();

    let user_id = format!("hf!{}", Uuid::new_v4());
    let now = Utc::now();

    let mut tx = state.db.begin().await.map_err(|error| {
        error!(%error, "failed to start transaction");
        (StatusCode::INTERNAL_SERVER_ERROR, "db_error")
    })?;

    sqlx::query(
        r#"
        INSERT INTO users (id, email, slack_id, first_name, last_name, verification_status, ysws_eligible, password_hash, email_verified, hca_access_token, hca_refresh_token, hca_token_expires_at)
        VALUES ($1, $2, NULL, $3, $4, 'email', false, $5, false, '', '', NOW())
        "#,
    )
    .bind(&user_id)
    .bind(&req.email)
    .bind(&req.first_name)
    .bind(&req.last_name)
    .bind(&password_hash)
    .execute(&mut *tx)
    .await
    .map_err(|error| {
        error!(%error, "failed to insert user");
        (StatusCode::INTERNAL_SERVER_ERROR, "db_error")
    })?;

    let refresh_exp = now + chrono::Duration::days(state.config.refresh_token_days);
    let jit = UserSessionsService::create_with(&mut *tx, &user_id, ip_addr, refresh_exp)
        .await
        .map_err(|error| {
            error!(%error, "failed to create session");
            (StatusCode::INTERNAL_SERVER_ERROR, "db_error")
        })?;

    tx.commit().await.map_err(|error| {
        error!(%error, "failed to commit transaction");
        (StatusCode::INTERNAL_SERVER_ERROR, "db_error")
    })?;

    let (access_token, refresh_token) = make_tokens(&state.config, jit, &user_id, now)?;

    let is_secure = state.config.hca.is_secure();
    let (access_cookie, refresh_cookie) =
        make_auth_cookies(&state.config, access_token, refresh_token, is_secure);

    let mut response = (StatusCode::CREATED, ()).into_response();
    set_cookie_header(&mut response, &access_cookie);
    set_cookie_header(&mut response, &refresh_cookie);
    Ok(response)
}

async fn email_login_handler(
    State(state): State<AppState>,
    ClientIp(ip_addr): ClientIp,
    Json(req): Json<LoginRequest>,
) -> Result<Response, (StatusCode, &'static str)> {
    if req.email.is_empty() || req.password.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "email_and_password_required"));
    }

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1 LIMIT 1")
        .bind(&req.email)
        .fetch_optional(&state.db)
        .await
        .map_err(|error| {
            error!(%error, "failed to look up user");
            (StatusCode::INTERNAL_SERVER_ERROR, "db_error")
        })?
        .ok_or((StatusCode::UNAUTHORIZED, "invalid_email_or_password"))?;

    let Some(ref stored_hash) = user.password_hash else {
        return Err((StatusCode::UNAUTHORIZED, "invalid_email_or_password"));
    };

    use argon2::{Argon2, PasswordHash, PasswordVerifier};
    let parsed_hash = PasswordHash::new(stored_hash).map_err(|error| {
        error!(%error, "failed to parse stored password hash");
        (StatusCode::INTERNAL_SERVER_ERROR, "password_parse_error")
    })?;

    Argon2::default()
        .verify_password(req.password.as_bytes(), &parsed_hash)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "invalid_email_or_password"))?;

    let now = Utc::now();
    let refresh_exp = now + chrono::Duration::days(state.config.refresh_token_days);

    let jit = UserSessionsService::create_with(&state.db, &user.id, ip_addr, refresh_exp)
        .await
        .map_err(|error| {
            error!(%error, "failed to create session");
            (StatusCode::INTERNAL_SERVER_ERROR, "db_error")
        })?;

    let (access_token, refresh_token) = make_tokens(&state.config, jit, &user.id, now)?;

    let is_secure = state.config.hca.is_secure();
    let (access_cookie, refresh_cookie) =
        make_auth_cookies(&state.config, access_token, refresh_token, is_secure);

    let mut response = (StatusCode::OK, ()).into_response();
    set_cookie_header(&mut response, &access_cookie);
    set_cookie_header(&mut response, &refresh_cookie);
    Ok(response)
}

#[derive(Debug, Deserialize)]
struct ForgotPasswordRequest {
    email: String,
}

#[derive(Debug, Deserialize)]
struct ResetPasswordRequest {
    token: String,
    password: String,
}

async fn forgot_password_handler(
    State(state): State<AppState>,
    Json(req): Json<ForgotPasswordRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, &'static str)> {
    if req.email.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "email_required"));
    }

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1 LIMIT 1")
        .bind(&req.email)
        .fetch_optional(&state.db)
        .await
        .map_err(|error| {
            error!(%error, "failed to look up user");
            (StatusCode::INTERNAL_SERVER_ERROR, "db_error")
        })?;

    // Always return 200 to avoid revealing whether the email exists
    let Some(user) = user else {
        return Ok(Json(
            serde_json::json!({"message": "if the email exists, a reset link has been sent"}),
        ));
    };

    // Revoke existing reset tokens for this user, then create a new one
    if let Err(e) = state.password_reset.revoke_all_for_user(&user.id).await {
        error!(%e, "failed to revoke existing tokens");
    }

    let token = match state.password_reset.create_token(&user.id).await {
        Ok(t) => t,
        Err(error) => {
            error!(%error, "failed to create reset token");
            return Err((StatusCode::INTERNAL_SERVER_ERROR, "db_error"));
        }
    };

    // Send email (best-effort)
    if let Some(ref email_svc) = state.email {
        let reset_link = match state.config.frontend_url.as_ref() {
            Some(base) => format!(
                "{}/reset-password?token={}",
                base.as_str().trim_end_matches('/'),
                token
            ),
            None => format!(
                "{}/api/v1/auth/reset-password?token={}",
                state.config.hca.redirect_uri.as_str().trim_end_matches('/'),
                token
            ),
        };

        let subject = "Password Reset for Hackflare";
        let body = format!(
            "Someone requested a password reset for your Hackflare account.\n\n\
             If this was you, click the link below to reset your password:\n\n\
             {reset_link}\n\n\
             This link will expire in 1 hour.\n\n\
             If you didn't request this, you can safely ignore this email."
        );

        if let Err(e) = email_svc.send(&user.email, subject, &body).await {
            error!(%e, "failed to send password reset email");
        }
    } else {
        warn!("password reset requested but SMTP not configured, email not sent");
    }

    Ok(Json(
        serde_json::json!({"message": "if the email exists, a reset link has been sent"}),
    ))
}

async fn reset_password_handler(
    State(state): State<AppState>,
    Json(req): Json<ResetPasswordRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, &'static str)> {
    if req.token.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "token_required"));
    }
    if req.password.len() < 8 {
        return Err((StatusCode::BAD_REQUEST, "password_too_short"));
    }

    let user_id = state
        .password_reset
        .consume_token(&req.token)
        .await
        .map_err(|error| {
            error!(%error, "failed to consume reset token");
            (StatusCode::INTERNAL_SERVER_ERROR, "db_error")
        })?
        .ok_or((StatusCode::BAD_REQUEST, "invalid_or_expired_token"))?;

    use argon2::{
        Argon2, PasswordHasher,
        password_hash::{SaltString, rand_core::OsRng},
    };
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(req.password.as_bytes(), &salt)
        .map_err(|error| {
            error!(%error, "failed to hash password");
            (StatusCode::INTERNAL_SERVER_ERROR, "password_hash_error")
        })?
        .to_string();

    sqlx::query("UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2")
        .bind(&password_hash)
        .bind(&user_id)
        .execute(&state.db)
        .await
        .map_err(|error| {
            error!(%error, "failed to update password");
            (StatusCode::INTERNAL_SERVER_ERROR, "db_error")
        })?;

    // Revoke all sessions for this user after password reset
    if let Err(e) = state.user_sessions.revoke_all_for_user(&user_id).await {
        error!(%e, "failed to revoke sessions after password reset");
    }

    Ok(Json(
        serde_json::json!({"message": "password reset successfully"}),
    ))
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
        .route("/login", get(login_handler).post(email_login_handler))
        .route("/register", post(register_handler))
        .route("/forgot-password", post(forgot_password_handler))
        .route("/reset-password", post(reset_password_handler))
        .route("/callback", get(callback_handler))
        .route("/refresh", post(refresh_handler))
        .route("/logout", post(logout_handler))
        .layer(session_layer)
}
