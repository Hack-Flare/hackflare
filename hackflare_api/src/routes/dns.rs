use axum::{
    Json, Router,
    extract::{Extension, State},
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{delete, get, post, put},
};
use hackflare_dns::dns::authoritative::resolve_ns_authoritative;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{middlewares::auth_middleware, models::CurrentUser, state::AppState};

// ── Response types ──

#[derive(Serialize)]
struct ZoneResponse {
    name: String,
    ns_verified: bool,
}

#[derive(Serialize)]
struct RecordResponse {
    id: String,
    name: String,
    r#type: String,
    value: String,
    ttl: u32,
    status: String,
}

// ── Request types ──

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
    value: String,
    ttl: u32,
}

// ── Helpers ──

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

// ── Ownership helper ──

async fn ensure_zone_ownership(
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

// ── Zone handlers ──

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
    let name = req.name.trim().to_string();
    if name.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "zone name is required"})),
        )
            .into_response();
    }

    // Check zone doesn't already exist globally
    let zones = state.dns_authority.list_zones().await;
    if zones.iter().any(|z| z == &name) {
        return (
            StatusCode::CONFLICT,
            Json(serde_json::json!({"error": "zone already exists"})),
        )
            .into_response();
    }

    if state.dns_authority.create_zone(&name).await {
        // Associate zone with the authenticated user
        let _ = sqlx::query("UPDATE dns_zones SET user_id = $1 WHERE name = $2")
            .bind(&current_user.user.id)
            .bind(&name)
            .execute(&state.db)
            .await;
        (StatusCode::CREATED, Json(serde_json::json!({"name": name}))).into_response()
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "invalid zone name"})),
        )
            .into_response()
    }
}

async fn delete_zone(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    axum::extract::Path(zone_name): axum::extract::Path<String>,
) -> StatusCode {
    if ensure_zone_ownership(&state.db, &zone_name, &current_user.user.id)
        .await
        .is_err()
    {
        return StatusCode::NOT_FOUND;
    }
    if state.dns_authority.delete_zone(&zone_name).await {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

async fn verify_zone(
    State(state): State<AppState>,
    axum::extract::Path(zone_name): axum::extract::Path<String>,
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
        zone_name.clone()
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
        let _ = set_zone_verified(&state.db, &zone_name).await;
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

// ── Record handlers ──

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
    if ensure_zone_ownership(&state.db, &zone_name, &current_user.user.id)
        .await
        .is_err()
    {
        return zone_not_found().into_response();
    }

    if ensure_zone_verified(&state.db, &zone_name).await.is_err() {
        return zone_not_verified().into_response();
    }

    if state
        .dns_authority
        .add_record(&zone_name, &req.name, &req.rtype, req.ttl, &req.value)
        .await
    {
        (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "name": req.name,
                "type": req.rtype,
                "value": req.value,
                "ttl": req.ttl,
                "status": "active"
            })),
        )
            .into_response()
    } else {
        internal_error("failed to create record").into_response()
    }
}

async fn update_record(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    axum::extract::Path((zone_name, record_name, record_type)): axum::extract::Path<(
        String,
        String,
        String,
    )>,
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

    if !state
        .dns_authority
        .remove_record(&zone_name, &record_name, &record_type)
        .await
    {
        return record_not_found().into_response();
    }
    state
        .dns_authority
        .add_record(&zone_name, &record_name, &record_type, req.ttl, &req.value)
        .await;
    (
        StatusCode::OK,
        Json(serde_json::json!({"status": "updated"})),
    )
        .into_response()
}

async fn delete_record(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    axum::extract::Path((zone_name, record_name, record_type)): axum::extract::Path<(
        String,
        String,
        String,
    )>,
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

    if state
        .dns_authority
        .remove_record(&zone_name, &record_name, &record_type)
        .await
    {
        StatusCode::NO_CONTENT.into_response()
    } else {
        record_not_found().into_response()
    }
}

// ── DB helpers ──

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

// ── Router ──

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
            "/zones/{zone_name}/records/{record_name}/{record_type}",
            put(update_record).delete(delete_record),
        )
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use std::{net::SocketAddr, str::FromStr};

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use chrono::{Duration, Utc};
    use jsonwebtoken::{DecodingKey, EncodingKey, Header, encode};
    use reqwest::Url;
    use tower::ServiceExt;
    use uuid::Uuid;

    use crate::{
        config::{Config, Environment, HcaConfig},
        models::JwtClaims,
        routes::build_router,
        state::AppState,
    };
    use axum_client_ip::ClientIpSource;

    const TEST_JWT_SECRET: &str = "dGhpcyBpcyBhIHRlc3Qgc2VjcmV0IGZvciB0ZXN0aW5nIHB1cnBvc2Vz";

    struct TestCtx {
        state: AppState,
        jwt: String,
        user_id: String,
        session_id: Uuid,
    }

    impl TestCtx {
        async fn setup() -> Option<Self> {
            let database_url = std::env::var("DATABASE_URL").ok()?;
            let url = Url::parse(&database_url).ok()?;

            let config = Config {
                bind_addr: SocketAddr::from_str("0.0.0.0:0").ok()?,
                dns_bind_addr: SocketAddr::from_str("0.0.0.0:0").ok()?,
                client_ip_source: ClientIpSource::ConnectInfo,
                environment: Environment::Development,
                database_url: url,
                auto_migrate: true,
                jwt_encoding_key: EncodingKey::from_base64_secret(TEST_JWT_SECRET).ok()?,
                jwt_decoding_key: DecodingKey::from_base64_secret(TEST_JWT_SECRET).ok()?,
                hca: HcaConfig {
                    client_id: "test".into(),
                    client_secret: "test".into(),
                    redirect_uri: Url::parse("http://localhost:3000/callback").ok()?,
                },
                slack_webhook_url: None,
                session_inactivity_minutes: 15,
                access_token_minutes: 15,
                refresh_token_days: 30,
                dns_nameservers: vec!["ns1.hackflare.dev".into(), "ns2.hackflare.dev".into()],
                admin_emails: vec![],
                smtp: None,
                frontend_url: None,
            };

            let state = AppState::new(config).await.ok()?;

            let user_id = Uuid::new_v4().to_string();
            let session_id = Uuid::new_v4();
            let now = Utc::now();

            let _ = sqlx::query("DELETE FROM users WHERE id LIKE 'test-%'")
                .execute(&state.db)
                .await;

            sqlx::query(
                r#"
                INSERT INTO users (id, email, slack_id, first_name, last_name, verification_status,
                                   ysws_eligible, hca_access_token, hca_refresh_token, hca_token_expires_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                "#,
            )
            .bind(&user_id)
            .bind(format!("{}@test.com", user_id))
            .bind(format!("slack_{}", user_id))
            .bind("Test")
            .bind("User")
            .bind("verified")
            .bind(true)
            .bind("test_access_token")
            .bind("test_refresh_token")
            .bind(now)
            .execute(&state.db)
            .await
            .ok()?;

            sqlx::query(
                r#"
                INSERT INTO user_sessions (id, user_id, ip_address, expires_at, created_at)
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(session_id)
            .bind(&user_id)
            .bind(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)))
            .bind(now + Duration::hours(1))
            .bind(now)
            .execute(&state.db)
            .await
            .ok()?;

            let claims = JwtClaims {
                sub: user_id.clone(),
                jit: session_id,
                exp: now + Duration::hours(1),
                iat: now,
                typ: None,
            };
            let test_jwt_key = EncodingKey::from_base64_secret(TEST_JWT_SECRET).ok()?;
            let jwt = encode(&Header::default(), &claims, &test_jwt_key).ok()?;

            Some(Self {
                state,
                jwt,
                user_id,
                session_id,
            })
        }

        fn authed_request(
            &self,
            method: &str,
            uri: &str,
            body: Option<&'static str>,
        ) -> Request<Body> {
            let mut builder = Request::builder()
                .method(method)
                .uri(uri)
                .header("Cookie", format!("jwt={}", self.jwt));
            if body.is_some() {
                builder = builder.header("Content-Type", "application/json");
            }
            builder
                .body(body.map(Body::from).unwrap_or_else(Body::empty))
                .unwrap()
        }

        async fn cleanup(&self) {
            let _ = sqlx::query("DELETE FROM user_sessions WHERE id = $1")
                .bind(self.session_id)
                .execute(&self.state.db)
                .await;
            let _ = sqlx::query("DELETE FROM users WHERE id = $1")
                .bind(&self.user_id)
                .execute(&self.state.db)
                .await;
        }
    }

    async fn get_body(response: axum::response::Response) -> serde_json::Value {
        use http_body_util::BodyExt;
        let collected = response.into_body().collect().await.unwrap();
        serde_json::from_slice(&collected.to_bytes()).unwrap()
    }

    // ── Tests ──

    #[tokio::test]
    async fn test_unauthenticated() {
        let Some(ctx) = TestCtx::setup().await else {
            eprintln!("skipping: DATABASE_URL not set or unreachable");
            return;
        };

        let response = build_router(ctx.state.clone())
            .oneshot(
                Request::builder()
                    .uri("/api/v1/dns/zones")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        ctx.cleanup().await;
    }

    #[tokio::test]
    async fn test_create_and_list_zones() {
        let Some(ctx) = TestCtx::setup().await else {
            eprintln!("skipping: DATABASE_URL not set or unreachable");
            return;
        };

        // Create zone
        let response = build_router(ctx.state.clone())
            .oneshot(ctx.authed_request(
                "POST",
                "/api/v1/dns/zones",
                Some(r#"{"name": "test-create.com"}"#),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        // List has zone
        let response = build_router(ctx.state.clone())
            .oneshot(ctx.authed_request("GET", "/api/v1/dns/zones", None))
            .await
            .unwrap();
        let body = get_body(response).await;
        let names: Vec<&str> = body
            .as_array()
            .unwrap()
            .iter()
            .map(|z| z["name"].as_str().unwrap())
            .collect();
        assert!(names.contains(&"test-create.com"));

        // Duplicate zone returns 409
        let response = build_router(ctx.state.clone())
            .oneshot(ctx.authed_request(
                "POST",
                "/api/v1/dns/zones",
                Some(r#"{"name": "test-create.com"}"#),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CONFLICT);

        let zones = ctx.state.dns_authority.list_zones().await;
        assert!(zones.iter().any(|z| z == "test-create.com"));

        // Cleanup
        let _ = ctx.state.dns_authority.delete_zone("test-create.com").await;
        ctx.cleanup().await;
    }

    #[tokio::test]
    async fn test_delete_zone() {
        let Some(ctx) = TestCtx::setup().await else {
            eprintln!("skipping: DATABASE_URL not set or unreachable");
            return;
        };

        ctx.state.dns_authority.create_zone("test-del.com").await;
        sqlx::query("UPDATE dns_zones SET user_id = $1 WHERE name = 'test-del.com'")
            .bind(&ctx.user_id)
            .execute(&ctx.state.db)
            .await
            .ok();

        let response = build_router(ctx.state.clone())
            .oneshot(ctx.authed_request("DELETE", "/api/v1/dns/zones/test-del.com", None))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        assert!(
            !ctx.state
                .dns_authority
                .list_zones()
                .await
                .iter()
                .any(|z| z == "test-del.com")
        );

        let response = build_router(ctx.state.clone())
            .oneshot(ctx.authed_request("DELETE", "/api/v1/dns/zones/nope.com", None))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        ctx.cleanup().await;
    }

    #[tokio::test]
    async fn test_records_crud() {
        let Some(ctx) = TestCtx::setup().await else {
            eprintln!("skipping: DATABASE_URL not set or unreachable");
            return;
        };

        ctx.state.dns_authority.create_zone("test-rec.com").await;
        sqlx::query("UPDATE dns_zones SET user_id = $1 WHERE name = 'test-rec.com'")
            .bind(&ctx.user_id)
            .execute(&ctx.state.db)
            .await
            .ok();
        // Mark zone as verified so record CRUD is allowed
        let _ = sqlx::query("UPDATE dns_zones SET ns_verified = true WHERE name = 'test-rec.com'")
            .execute(&ctx.state.db)
            .await;

        // Create record
        let response = build_router(ctx.state.clone())
            .oneshot(ctx.authed_request(
                "POST",
                "/api/v1/dns/zones/test-rec.com/records",
                Some(r#"{"name":"www","type":"A","value":"1.2.3.4","ttl":300}"#),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        // Verify via authority (add_record returns bool)
        let response = build_router(ctx.state.clone())
            .oneshot(ctx.authed_request("GET", "/api/v1/dns/zones/test-rec.com/records", None))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = get_body(response).await;
        let records = body.as_array().unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0]["name"], "www");
        assert_eq!(records[0]["type"], "A");
        assert_eq!(records[0]["value"], "1.2.3.4");

        // Update
        let response = build_router(ctx.state.clone())
            .oneshot(ctx.authed_request(
                "PUT",
                "/api/v1/dns/zones/test-rec.com/records/www/A",
                Some(r#"{"value":"10.0.0.1","ttl":600}"#),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let response = build_router(ctx.state.clone())
            .oneshot(ctx.authed_request("GET", "/api/v1/dns/zones/test-rec.com/records", None))
            .await
            .unwrap();
        let body = get_body(response).await;
        let rec = &body.as_array().unwrap()[0];
        assert_eq!(rec["value"], "10.0.0.1");
        assert_eq!(rec["ttl"], 600);

        // Delete record
        let response = build_router(ctx.state.clone())
            .oneshot(ctx.authed_request(
                "DELETE",
                "/api/v1/dns/zones/test-rec.com/records/www/A",
                None,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        let response = build_router(ctx.state.clone())
            .oneshot(ctx.authed_request("GET", "/api/v1/dns/zones/test-rec.com/records", None))
            .await
            .unwrap();
        let body = get_body(response).await;
        assert_eq!(body.as_array().unwrap().len(), 0);

        let _ = ctx.state.dns_authority.delete_zone("test-rec.com").await;
        ctx.cleanup().await;
    }

    #[tokio::test]
    async fn test_records_nonexistent_zone() {
        let Some(ctx) = TestCtx::setup().await else {
            eprintln!("skipping: DATABASE_URL not set or unreachable");
            return;
        };

        let response = build_router(ctx.state.clone())
            .oneshot(ctx.authed_request(
                "POST",
                "/api/v1/dns/zones/nope.com/records",
                Some(r#"{"name":"www","type":"A","value":"1.2.3.4","ttl":300}"#),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let response = build_router(ctx.state.clone())
            .oneshot(ctx.authed_request("GET", "/api/v1/dns/zones/nope.com/records", None))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        ctx.cleanup().await;
    }

    #[tokio::test]
    async fn test_records_blocked_on_unverified_zone() {
        let Some(ctx) = TestCtx::setup().await else {
            eprintln!("skipping: DATABASE_URL not set or unreachable");
            return;
        };

        // Create zone (starts unverified)
        ctx.state
            .dns_authority
            .create_zone("test-unverified.com")
            .await;
        sqlx::query("UPDATE dns_zones SET user_id = $1 WHERE name = 'test-unverified.com'")
            .bind(&ctx.user_id)
            .execute(&ctx.state.db)
            .await
            .ok();

        // Create record should be blocked
        let response = build_router(ctx.state.clone())
            .oneshot(ctx.authed_request(
                "POST",
                "/api/v1/dns/zones/test-unverified.com/records",
                Some(r#"{"name":"www","type":"A","value":"1.2.3.4","ttl":300}"#),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        // Update record should be blocked
        ctx.state
            .dns_authority
            .add_record("test-unverified.com", "www", "A", 300, "1.2.3.4")
            .await;
        let response = build_router(ctx.state.clone())
            .oneshot(ctx.authed_request(
                "PUT",
                "/api/v1/dns/zones/test-unverified.com/records/www/A",
                Some(r#"{"value":"10.0.0.1","ttl":600}"#),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        // Delete record should be blocked
        let response = build_router(ctx.state.clone())
            .oneshot(ctx.authed_request(
                "DELETE",
                "/api/v1/dns/zones/test-unverified.com/records/www/A",
                None,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        let _ = ctx
            .state
            .dns_authority
            .delete_zone("test-unverified.com")
            .await;
        ctx.cleanup().await;
    }

    #[tokio::test]
    async fn test_verify_zone() {
        let Some(ctx) = TestCtx::setup().await else {
            eprintln!("skipping: DATABASE_URL not set or unreachable");
            return;
        };

        let response = build_router(ctx.state.clone())
            .oneshot(ctx.authed_request("POST", "/api/v1/dns/zones/example.com/verify", None))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = get_body(response).await;
        assert!(body["verified"].as_bool().is_some());

        ctx.cleanup().await;
    }
}
