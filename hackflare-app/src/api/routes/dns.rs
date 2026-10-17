use axum::{
    Json, Router,
    extract::{Extension, Path, State},
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{delete, get, post, put},
};
use hackflare_dns::dns::authoritative::resolve_ns_authoritative;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{middlewares::auth_middleware, models::CurrentUser, state::AppState};

// --- Response types ---

#[derive(Serialize)]
struct ZoneResponse {
    name: String,
    ns_verified: bool,
}
#[derive(Serialize)]
pub(crate) struct RecordResponse {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) r#type: String,
    pub(crate) value: String,
    pub(crate) ttl: u32,
    pub(crate) status: String,
}

// --- Request types ---

#[derive(Deserialize)]
struct CreateZoneRequest {
    name: String,
}

#[derive(Deserialize)]
struct CreateRecordRequest {
    name: String,
    #[serde(rename = "type")]
    rtype: String,
    value: String,
    ttl: u32,
}

#[derive(Deserialize)]
struct UpdateRecordRequest {
    name: String,
    #[serde(rename = "type")]
    rtype: String,
    value: String,
    ttl: u32,
}

// --- Helpers ---

async fn is_zone_verified(db: &PgPool, zone_name: &str) -> Result<bool, sqlx::Error> {
    let row: Option<(bool,)> = sqlx::query_as("SELECT ns_verified FROM dns_zones WHERE name = $1")
        .bind(zone_name)
        .fetch_optional(db)
        .await?;
    Ok(row.map(|r| r.0).unwrap_or(false))
}

async fn set_zone_verified(db: &PgPool, zone_name: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE dns_zones SET ns_verified = true, updated_at = now() WHERE name = $1")
        .bind(zone_name)
        .execute(db)
        .await?;
    Ok(())
}

fn zone_not_found() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({"error": "zone not found"})),
    )
}

fn record_not_found() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({"error": "record not found"})),
    )
}

fn zone_not_verified() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::FORBIDDEN,
        Json(
            serde_json::json!({"error": "zone not verified, record edits are blocked until NS delegation is verified"}),
        ),
    )
}

fn internal_error(msg: &str) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({"error": msg})),
    )
}

// --- Ownership helper ---

pub(crate) async fn ensure_zone_ownership(
    db: &PgPool,
    zone_name: &str,
    user_id: &str,
) -> Result<(), StatusCode> {
    let result: Option<String> =
        sqlx::query_scalar("SELECT user_id FROM dns_zones WHERE name = $1")
            .bind(zone_name)
            .fetch_optional(db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match result.as_deref() {
        Some(owner) if owner == user_id => Ok(()),
        _ => Err(StatusCode::NOT_FOUND),
    }
}

async fn ensure_zone_verified(db: &PgPool, zone_name: &str) -> Result<(), StatusCode> {
    if !is_zone_verified(db, zone_name).await.unwrap_or(false) {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(())
}

// --- Zone handlers ---

async fn list_zones(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Json<Vec<ZoneResponse>> {
    let rows: Vec<(String, bool)> =
        sqlx::query_as("SELECT name, ns_verified FROM dns_zones WHERE user_id = $1 ORDER BY name")
            .bind(&current_user.user.id)
            .fetch_all(&state.db)
            .await
            .unwrap_or_default();
    Json(
        rows.into_iter()
            .map(|(name, ns_verified)| ZoneResponse { name, ns_verified })
            .collect(),
    )
}

async fn create_zone(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateZoneRequest>,
) -> impl IntoResponse {
    match add_zone(&state, &current_user.user.id, &req.name).await {
        Ok(name) => (StatusCode::CREATED, Json(serde_json::json!({"name": name}))).into_response(),
        Err(DnsActionError::ZoneExists) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({"error": "zone already exists"})),
        )
            .into_response(),
        Err(DnsActionError::NameRequired) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "zone name is required"})),
        )
            .into_response(),
        Err(_) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "invalid zone name"})),
        )
            .into_response(),
    }
}

async fn delete_zone(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    axum::extract::Path(zone_name): axum::extract::Path<String>,
) -> StatusCode {
    match remove_zone(&state, &current_user.user.id, &zone_name).await {
        Ok(()) => StatusCode::NO_CONTENT,
        Err(_) => StatusCode::NOT_FOUND,
    }
}

pub(crate) async fn check_zone_propagation(
    state: &AppState,
    zone_name: &str,
) -> Json<serde_json::Value> {
    let ns_targets: Vec<String> = state
        .config
        .dns_nameservers
        .iter()
        .map(|ns| format!("{ns}."))
        .collect();

    let qname = if zone_name.ends_with('.') {
        zone_name.trim_end_matches('.').to_string()
    } else {
        zone_name.to_string()
    };

    let dns_config = hackflare_dns::DnsConfig::from_env();
    let ns_names = match resolve_ns_authoritative(&qname, &dns_config) {
        Ok(names) => names,
        Err(e) => {
            return Json(serde_json::json!({
                "verified": false,
                "message": format!("Authoritative NS lookup failed: {e}")
            }));
        }
    };

    if ns_names.is_empty() {
        return Json(serde_json::json!({
            "verified": false,
            "message": "No NS records found for this domain"
        }));
    }

    let matched: Vec<&str> = ns_targets
        .iter()
        .map(|t| t.trim_end_matches('.'))
        .filter(|target| {
            ns_names
                .iter()
                .any(|ns| ns.trim_end_matches('.').eq_ignore_ascii_case(target))
        })
        .collect();

    if !matched.is_empty() {
        // Persist verification status so record edits are unblocked
        let _ = set_zone_verified(&state.db, zone_name).await;

        // Notify the zone owner
        if let Ok(Some((owner_id,))) = sqlx::query_as::<_, (String,)>(
            "SELECT user_id FROM dns_zones WHERE name = $1 AND user_id IS NOT NULL",
        )
        .bind(zone_name)
        .fetch_optional(&state.db)
        .await
        {
            let _ = crate::api::services::notifications::create_notification(
                &state.db,
                &owner_id,
                "Domain Verified",
                &format!(
                    "{} has been verified. You can now manage DNS records.",
                    zone_name
                ),
                "domain_verified",
                Some(&format!("/dash/domains/{}/dns", zone_name)),
            )
            .await;
        }

        Json(serde_json::json!({
            "verified": true,
            "message": format!("Nameserver verification passed: {} matched", matched.join(", "))
        }))
    } else {
        Json(serde_json::json!({
            "verified": false,
            "message": format!(
                "Expected nameservers: {}. Found: {}",
                ns_targets.join(", "),
                ns_names.join(", ")
            )
        }))
    }
}

async fn verify_zone(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(zone_name): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    ensure_zone_ownership(&state.db, &zone_name, &current_user.user.id).await?;
    Ok(check_zone_propagation(&state, &zone_name).await)
}

// --- Record handlers ---

async fn list_records(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    axum::extract::Path(zone_name): axum::extract::Path<String>,
) -> Result<Json<Vec<RecordResponse>>, StatusCode> {
    ensure_zone_ownership(&state.db, &zone_name, &current_user.user.id).await?;

    let records = get_records_from_db(&state.db, &zone_name)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(records))
}

async fn create_record(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    axum::extract::Path(zone_name): axum::extract::Path<String>,
    Json(req): Json<CreateRecordRequest>,
) -> impl IntoResponse {
    match add_record(
        &state,
        &current_user.user.id,
        &zone_name,
        &req.name,
        &req.rtype,
        req.ttl,
        &req.value,
    )
    .await
    {
        Err(DnsActionError::ZoneNotFound) => zone_not_found().into_response(),
        Err(DnsActionError::ZoneNotVerified) => zone_not_verified().into_response(),
        Err(_) => internal_error("failed to create record").into_response(),
        Ok(()) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "name": req.name,
                "type": req.rtype,
                "value": req.value,
                "ttl": req.ttl,
                "status": "active"
            })),
        )
            .into_response(),
    }
}

async fn update_record(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    axum::extract::Path((zone_name, record_id)): axum::extract::Path<(String, uuid::Uuid)>,
    Json(req): Json<UpdateRecordRequest>,
) -> impl IntoResponse {
    if ensure_zone_ownership(&state.db, &zone_name, &current_user.user.id)
        .await
        .is_err()
    {
        return zone_not_found().into_response();
    }

    if ensure_zone_verified(&state.db, &zone_name).await.is_err() {
        return zone_not_verified().into_response();
    }

    let Some((old_name, old_rtype, old_data)) =
        get_record_by_id(&state.db, &zone_name, record_id).await
    else {
        return record_not_found().into_response();
    };

    if !state
        .dns_authority
        .remove_record_value(&zone_name, &old_name, &old_rtype, &old_data)
        .await
    {
        return record_not_found().into_response();
    }

    if !state
        .dns_authority
        .add_record(&zone_name, &req.name, &req.rtype, req.ttl, &req.value)
        .await
    {
        return internal_error("failed to update record").into_response();
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "name": req.name,
            "type": req.rtype,
            "value": req.value,
            "ttl": req.ttl,
            "status": "active"
        })),
    )
        .into_response()
}

async fn delete_record(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    axum::extract::Path((zone_name, record_id)): axum::extract::Path<(String, uuid::Uuid)>,
) -> impl IntoResponse {
    match remove_record(&state, &current_user.user.id, &zone_name, record_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(DnsActionError::ZoneNotFound) => zone_not_found().into_response(),
        Err(DnsActionError::ZoneNotVerified) => zone_not_verified().into_response(),
        Err(_) => record_not_found().into_response(),
    }
}

// --- Shared operations ---
//
// Used by both the JSON API above and the server-rendered dashboard forms, so
// ownership and verification checks live in exactly one place.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DnsActionError {
    NameRequired,
    ZoneExists,
    InvalidZone,
    ZoneLookupFailed,
    ZoneNotFound,
    ZoneNotVerified,
    RecordNotFound,
    Internal,
}

/// Create a zone owned by `user_id`, returning its normalized name.
pub(crate) async fn add_zone(
    state: &AppState,
    user_id: &str,
    name: &str,
) -> Result<String, DnsActionError> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(DnsActionError::NameRequired);
    }

    // Check zone doesn't already exist globally
    let zones = state.dns_authority.list_zones().await;
    if zones.iter().any(|z| z == &name) {
        return Err(DnsActionError::ZoneExists);
    }

    let check_name = name.clone();
    let check_config = state.dns_config.clone();
    let registration = tokio::task::spawn_blocking(move || {
        hackflare_dns::check_domain_registration(&check_name, &check_config)
    })
    .await
    .map_err(|_| DnsActionError::ZoneLookupFailed)?;
    match registration {
        Ok(()) => {}
        Err(hackflare_dns::DomainValidationError::InvalidName) => {
            return Err(DnsActionError::InvalidZone);
        }
        Err(hackflare_dns::DomainValidationError::LookupFailed) => {
            return Err(DnsActionError::ZoneLookupFailed);
        }
    }

    if !state.dns_authority.create_zone(&name).await {
        return Err(DnsActionError::InvalidZone);
    }

    // Associate zone with the authenticated user
    let _ = sqlx::query("UPDATE dns_zones SET user_id = $1 WHERE name = $2")
        .bind(user_id)
        .bind(&name)
        .execute(&state.db)
        .await;

    let _ = crate::api::services::notifications::create_notification(
        &state.db,
        user_id,
        "Domain Added",
        &format!(
            "{} has been added and is pending NS delegation verification.",
            name
        ),
        "domain_added",
        Some("/dash/domains"),
    )
    .await;

    Ok(name)
}

pub(crate) async fn remove_zone(
    state: &AppState,
    user_id: &str,
    zone_name: &str,
) -> Result<(), DnsActionError> {
    ensure_zone_ownership(&state.db, zone_name, user_id)
        .await
        .map_err(|_| DnsActionError::ZoneNotFound)?;
    if state.dns_authority.delete_zone(zone_name).await {
        Ok(())
    } else {
        Err(DnsActionError::ZoneNotFound)
    }
}

/// Records for a zone owned by `user_id`, plus whether the zone is verified.
pub(crate) async fn zone_records(
    state: &AppState,
    user_id: &str,
    zone_name: &str,
) -> Result<(bool, Vec<RecordResponse>), DnsActionError> {
    ensure_zone_ownership(&state.db, zone_name, user_id)
        .await
        .map_err(|_| DnsActionError::ZoneNotFound)?;
    let verified = is_zone_verified(&state.db, zone_name)
        .await
        .map_err(|_| DnsActionError::Internal)?;
    let records = get_records_from_db(&state.db, zone_name)
        .await
        .map_err(|_| DnsActionError::Internal)?;
    Ok((verified, records))
}

pub(crate) async fn add_record(
    state: &AppState,
    user_id: &str,
    zone_name: &str,
    name: &str,
    rtype: &str,
    ttl: u32,
    value: &str,
) -> Result<(), DnsActionError> {
    ensure_zone_ownership(&state.db, zone_name, user_id)
        .await
        .map_err(|_| DnsActionError::ZoneNotFound)?;
    ensure_zone_verified(&state.db, zone_name)
        .await
        .map_err(|_| DnsActionError::ZoneNotVerified)?;
    if state
        .dns_authority
        .add_record(zone_name, name, rtype, ttl, value)
        .await
    {
        Ok(())
    } else {
        Err(DnsActionError::Internal)
    }
}

pub(crate) async fn remove_record(
    state: &AppState,
    user_id: &str,
    zone_name: &str,
    record_id: uuid::Uuid,
) -> Result<(), DnsActionError> {
    ensure_zone_ownership(&state.db, zone_name, user_id)
        .await
        .map_err(|_| DnsActionError::ZoneNotFound)?;
    ensure_zone_verified(&state.db, zone_name)
        .await
        .map_err(|_| DnsActionError::ZoneNotVerified)?;
    let (name, rtype, data) = get_record_by_id(&state.db, zone_name, record_id)
        .await
        .ok_or(DnsActionError::RecordNotFound)?;
    if state
        .dns_authority
        .remove_record_value(zone_name, &name, &rtype, &data)
        .await
    {
        Ok(())
    } else {
        Err(DnsActionError::RecordNotFound)
    }
}

// --- DB helpers ---

async fn get_record_by_id(
    db: &PgPool,
    zone_name: &str,
    record_id: uuid::Uuid,
) -> Option<(String, String, String)> {
    let row: Option<(uuid::Uuid, String, String, String)> = sqlx::query_as(
        r#"
        SELECT r.id, r.name, r.rtype, r.data
        FROM dns_records r
        JOIN dns_zones z ON z.id = r.zone_id
        WHERE r.id = $1 AND z.name = $2
        "#,
    )
    .bind(record_id)
    .bind(zone_name)
    .fetch_optional(db)
    .await
    .ok()?;

    row.map(|(_, name, rtype, data)| (name, rtype, data))
}

/// List all records for a zone.
async fn get_records_from_db(
    db: &PgPool,
    zone_name: &str,
) -> Result<Vec<RecordResponse>, sqlx::Error> {
    #[derive(sqlx::FromRow)]
    struct RecordRow {
        id: uuid::Uuid,
        name: String,
        rtype: String,
        data: String,
        ttl: i32,
    }

    let rows = sqlx::query_as::<_, RecordRow>(
        r#"
        SELECT r.id, r.name, r.rtype, r.data, r.ttl
        FROM dns_records r
        JOIN dns_zones z ON z.id = r.zone_id
        WHERE z.name = $1
        ORDER BY r.name, r.rtype
        "#,
    )
    .bind(zone_name)
    .fetch_all(db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| RecordResponse {
            id: r.id.to_string(),
            name: r.name,
            r#type: r.rtype,
            value: r.data,
            ttl: r.ttl as u32,
            status: "active".into(),
        })
        .collect())
}

// --- Router ---

pub(super) fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/zones", get(list_zones).post(create_zone))
        .route("/zones/{zone_name}", delete(delete_zone))
        .route("/zones/{zone_name}/verify", post(verify_zone))
        .route(
            "/zones/{zone_name}/records",
            get(list_records).post(create_record),
        )
        .route(
            "/zones/{zone_name}/records/{record_id}",
            put(update_record).delete(delete_record),
        )
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}
