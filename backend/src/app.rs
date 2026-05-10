use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::domain::auth::{
    AuthError, EmailLoginChallenge, EmailLoginRequest, EmailLoginVerification, LoginInput,
    RegisterInput, Session, User,
};
use crate::domain::dns::{DnsError, NewRecordInput, RecordType, ResolvedRecord, Zone};
use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/v1/ping", get(api_ping))
        .route("/api/v1/auth/register", post(register))
        .route("/api/v1/auth/login", post(login))
        .route("/api/v1/auth/email/request", post(request_email_login))
        .route("/api/v1/auth/email/verify", post(verify_email_login))
        .route("/api/v1/auth/hackclub/url", get(hackclub_login_url))
        .route("/api/v1/auth/hackclub/callback", get(hackclub_callback))
        .route("/api/v1/auth/me", get(me))
        .route("/api/v1/dns/zones", get(list_zones).post(create_zone))
        .route("/api/v1/dns/zones/{zone_name}/records", post(add_record))
        .route("/api/v1/dns/zones/{zone_name}/verify", post(verify_zone))
        .route("/api/v1/dns/records", get(find_records))
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
struct ApiPingResponse {
    status: &'static str,
    service: &'static str,
    database_configured: bool,
}

async fn api_ping(
    State(state): State<AppState>,
    _headers: HeaderMap,
) -> Result<Json<ApiPingResponse>, StatusCode> {
    Ok(Json(ApiPingResponse {
        status: "ok",
        service: "hackflare-backend",
        database_configured: state.config.database_url.is_some(),
    }))
}

#[derive(Serialize)]
struct ErrorResponse {
    error: &'static str,
}

#[derive(Serialize)]
struct HackClubLoginUrlResponse {
    url: String,
}

#[derive(Deserialize)]
struct HackClubCallbackQuery {
    code: String,
}

async fn register(
    State(state): State<AppState>,
    _headers: HeaderMap,
    Json(payload): Json<RegisterInput>,
) -> Result<Json<Session>, (StatusCode, Json<ErrorResponse>)> {
    state
        .auth
        .register(payload)
        .map(Json)
        .map_err(map_auth_error)
}

async fn login(
    State(state): State<AppState>,
    _headers: HeaderMap,
    Json(payload): Json<LoginInput>,
) -> Result<Json<Session>, (StatusCode, Json<ErrorResponse>)> {
    state.auth.login(payload).map(Json).map_err(map_auth_error)
}

async fn request_email_login(
    State(state): State<AppState>,
    _headers: HeaderMap,
    Json(payload): Json<EmailLoginRequest>,
) -> Result<Json<EmailLoginChallenge>, (StatusCode, Json<ErrorResponse>)> {
    state
        .auth
        .request_email_login(payload)
        .map(Json)
        .map_err(map_auth_error)
}

async fn verify_email_login(
    State(state): State<AppState>,
    _headers: HeaderMap,
    Json(payload): Json<EmailLoginVerification>,
) -> Result<Json<Session>, (StatusCode, Json<ErrorResponse>)> {
    state
        .auth
        .verify_email_login(payload)
        .map(Json)
        .map_err(map_auth_error)
}

async fn hackclub_login_url(
    State(state): State<AppState>,
) -> Result<Json<HackClubLoginUrlResponse>, (StatusCode, Json<ErrorResponse>)> {
    let hackclub = state.hackclub_auth.as_ref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        Json(ErrorResponse {
            error: "hackclub_auth_not_configured",
        }),
    ))?;

    let scopes = state
        .config
        .hackclub
        .scopes
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();

    Ok(Json(HackClubLoginUrlResponse {
        url: hackclub.get_oauth_uri(&scopes),
    }))
}

async fn hackclub_callback(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<HackClubCallbackQuery>,
) -> Result<Json<Session>, (StatusCode, Json<ErrorResponse>)> {
    let hackclub = state.hackclub_auth.as_ref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        Json(ErrorResponse {
            error: "hackclub_auth_not_configured",
        }),
    ))?;

    let token = hackclub.exchange_code(query.code).await.map_err(|_| {
        (
            StatusCode::BAD_GATEWAY,
            Json(ErrorResponse {
                error: "hackclub_exchange_failed",
            }),
        )
    })?;

    let claims = hackclub
        .verify_jwt_token(token.id_token)
        .await
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "hackclub_token_invalid",
                }),
            )
        })?;

    let email = claims.email.ok_or((
        StatusCode::BAD_GATEWAY,
        Json(ErrorResponse {
            error: "hackclub_missing_email",
        }),
    ))?;

    state
        .auth
        .sign_in_email(&email)
        .map(Json)
        .map_err(map_auth_error)
}

async fn me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<User>, (StatusCode, Json<ErrorResponse>)> {
    let token = extract_bearer_token(&headers).ok_or((
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
            error: "missing_token",
        }),
    ))?;

    let user = state.auth.get_user_by_token(token).ok_or((
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
            error: "invalid_token",
        }),
    ))?;

    Ok(Json(user))
}

#[derive(Deserialize)]
struct CreateZoneRequest {
    name: String,
}

async fn list_zones(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<Zone>>, (StatusCode, Json<ErrorResponse>)> {
    let token = extract_bearer_token(&headers).ok_or((
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
            error: "missing_token",
        }),
    ))?;

    let user = state.auth.get_user_by_token(token).ok_or((
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
            error: "invalid_token",
        }),
    ))?;

    Ok(Json(state.dns.list_zones(user.id)))
}

async fn create_zone(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateZoneRequest>,
) -> Result<Json<Zone>, (StatusCode, Json<ErrorResponse>)> {
    let token = extract_bearer_token(&headers).ok_or((
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
            error: "missing_token",
        }),
    ))?;

    let user = state.auth.get_user_by_token(token).ok_or((
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
            error: "invalid_token",
        }),
    ))?;

    state
        .dns
        .create_zone(&payload.name, user.id)
        .map(Json)
        .map_err(map_dns_error)
}

#[derive(Deserialize)]
struct ZonePath {
    zone_name: String,
}

async fn add_record(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Path(path): axum::extract::Path<ZonePath>,
    Json(payload): Json<NewRecordInput>,
) -> Result<Json<Zone>, (StatusCode, Json<ErrorResponse>)> {
    let token = extract_bearer_token(&headers).ok_or((
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
            error: "missing_token",
        }),
    ))?;

    let user = state.auth.get_user_by_token(token).ok_or((
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
            error: "invalid_token",
        }),
    ))?;

    state
        .dns
        .add_record(&path.zone_name, payload, user.id)
        .map(Json)
        .map_err(map_dns_error)
}

async fn verify_zone(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Path(path): axum::extract::Path<ZonePath>,
) -> Result<Json<Zone>, (StatusCode, Json<ErrorResponse>)> {
    let token = extract_bearer_token(&headers).ok_or((
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
            error: "missing_token",
        }),
    ))?;

    let user = state.auth.get_user_by_token(token).ok_or((
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
            error: "invalid_token",
        }),
    ))?;

    state
        .dns
        .verify_zone(&path.zone_name, user.id)
        .map(Json)
        .map_err(map_dns_error)
}

#[derive(Deserialize)]
struct FindRecordsQuery {
    name: String,
    #[serde(rename = "type")]
    record_type: Option<RecordType>,
}

async fn find_records(
    State(state): State<AppState>,
    _headers: HeaderMap,
    axum::extract::Query(query): axum::extract::Query<FindRecordsQuery>,
) -> Result<Json<Vec<ResolvedRecord>>, StatusCode> {
    Ok(Json(state.dns.find_records(&query.name, query.record_type)))
}

fn map_auth_error(error: AuthError) -> (StatusCode, Json<ErrorResponse>) {
    let (status, message) = match error {
        AuthError::InvalidEmail => (StatusCode::BAD_REQUEST, "invalid_email"),
        AuthError::InvalidPassword => (StatusCode::BAD_REQUEST, "invalid_password"),
        AuthError::EmailAlreadyExists => (StatusCode::CONFLICT, "email_already_exists"),
        AuthError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "invalid_credentials"),
        AuthError::PasswordHashFailure => {
            (StatusCode::INTERNAL_SERVER_ERROR, "password_hash_failure")
        }
        AuthError::InvalidEmailCode => (StatusCode::UNAUTHORIZED, "invalid_email_code"),
        AuthError::EmailCodeExpired => (StatusCode::UNAUTHORIZED, "email_code_expired"),
        AuthError::HackClubNotConfigured => (
            StatusCode::SERVICE_UNAVAILABLE,
            "hackclub_auth_not_configured",
        ),
        AuthError::HackClubExchangeFailed => (StatusCode::BAD_GATEWAY, "hackclub_exchange_failed"),
        AuthError::HackClubTokenInvalid => (StatusCode::UNAUTHORIZED, "hackclub_token_invalid"),
        AuthError::HackClubMissingEmail => (StatusCode::BAD_GATEWAY, "hackclub_missing_email"),
    };

    (status, Json(ErrorResponse { error: message }))
}

fn map_dns_error(error: DnsError) -> (StatusCode, Json<ErrorResponse>) {
    let (status, message) = match error {
        DnsError::InvalidZoneName => (StatusCode::BAD_REQUEST, "invalid_zone_name"),
        DnsError::ZoneAlreadyExists => (StatusCode::CONFLICT, "zone_already_exists"),
        DnsError::ZoneNotFound => (StatusCode::NOT_FOUND, "zone_not_found"),
        DnsError::InvalidRecordName => (StatusCode::BAD_REQUEST, "invalid_record_name"),
        DnsError::InvalidRecordValue => (StatusCode::BAD_REQUEST, "invalid_record_value"),
        DnsError::InvalidRecordTtl => (StatusCode::BAD_REQUEST, "invalid_record_ttl"),
        DnsError::ZoneNotVerified => (StatusCode::FORBIDDEN, "zone_not_verified"),
        DnsError::Unauthorized => (StatusCode::FORBIDDEN, "unauthorized"),
    };

    (status, Json(ErrorResponse { error: message }))
}

fn extract_bearer_token(headers: &HeaderMap) -> Option<&str> {
    let raw = headers.get("authorization")?.to_str().ok()?;
    raw.strip_prefix("Bearer ")
}
