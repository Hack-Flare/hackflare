use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::domain::auth::{AuthError, LoginInput, RegisterInput, Session, User};
use crate::domain::dns::{DnsError, NewRecordInput, RecordType, ResolvedRecord, Zone};
use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/v1/ping", get(api_ping))
        .route("/api/v1/auth/register", post(register))
        .route("/api/v1/auth/login", post(login))
        .route("/api/v1/auth/me", get(me))
        .route("/api/v1/dns/zones", get(list_zones).post(create_zone))
        .route("/api/v1/dns/zones/{zone_name}/records", post(add_record))
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
    _headers: HeaderMap,
) -> Result<Json<Vec<Zone>>, StatusCode> {
    Ok(Json(state.dns.list_zones()))
}

async fn create_zone(
    State(state): State<AppState>,
    _headers: HeaderMap,
    Json(payload): Json<CreateZoneRequest>,
) -> Result<Json<Zone>, (StatusCode, Json<ErrorResponse>)> {
    state
        .dns
        .create_zone(&payload.name)
        .map(Json)
        .map_err(map_dns_error)
}

#[derive(Deserialize)]
struct ZonePath {
    zone_name: String,
}

async fn add_record(
    State(state): State<AppState>,
    _headers: HeaderMap,
    axum::extract::Path(path): axum::extract::Path<ZonePath>,
    Json(payload): Json<NewRecordInput>,
) -> Result<Json<Zone>, (StatusCode, Json<ErrorResponse>)> {
    state
        .dns
        .add_record(&path.zone_name, payload)
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
    };

    (status, Json(ErrorResponse { error: message }))
}

fn extract_bearer_token(headers: &HeaderMap) -> Option<&str> {
    let raw = headers.get("authorization")?.to_str().ok()?;
    raw.strip_prefix("Bearer ")
}
