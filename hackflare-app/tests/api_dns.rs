#![allow(clippy::unwrap_used)]

use std::{
    net::SocketAddr,
    str::FromStr,
    sync::atomic::{AtomicU16, Ordering},
};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, encode};
use pg_embed::{
    pg_enums::PgAuthMethod,
    pg_fetch::{PG_V17, PgFetchSettings},
    postgres::{PgEmbed, PgSettings},
};
use reqwest::Url;
use tower::ServiceExt;
use uuid::Uuid;

use axum_client_ip::ClientIpSource;
use hackflare_app::{
    api::routes::build_router,
    config::{Config, Environment, HcaConfig},
    models::JwtClaims,
    state::AppState,
};

const TEST_JWT_SECRET: &str = "dGhpcyBpcyBhIHRlc3Qgc2VjcmV0IGZvciB0ZXN0aW5nIHB1cnBvc2Vz";
static NEXT_POSTGRES_PORT: AtomicU16 = AtomicU16::new(55432);

struct TestCtx {
    state: AppState,
    _embedded_postgres: Option<PgEmbed>,
    jwt: String,
    user_id: String,
    session_id: Uuid,
}

impl TestCtx {
    async fn setup() -> Option<Self> {
        let (database_url, embedded_postgres) = match std::env::var("DATABASE_URL") {
            Ok(database_url) => (database_url, None),
            Err(_) => {
                let port = NEXT_POSTGRES_PORT.fetch_add(1, Ordering::Relaxed);
                let pg_settings = PgSettings {
                    database_dir: std::env::temp_dir()
                        .join(format!("hackflare-test-postgres-{port}")),
                    port,
                    user: "postgres".to_string(),
                    password: "postgres".to_string(),
                    auth_method: PgAuthMethod::Plain,
                    persistent: false,
                    timeout: Some(std::time::Duration::from_secs(30)),
                    migration_dir: None,
                };
                let fetch_settings = PgFetchSettings {
                    version: PG_V17,
                    ..Default::default()
                };
                let mut pg = PgEmbed::new(pg_settings, fetch_settings).await.ok()?;
                pg.setup().await.ok()?;
                pg.start_db().await.ok()?;
                pg.create_database("hackflare_test").await.ok()?;
                let database_url = pg.full_db_uri("hackflare_test");
                (database_url, Some(pg))
            }
        };
        let url = Url::parse(&database_url).ok()?;

        let config = Config {
            bind_addr: SocketAddr::from_str("0.0.0.0:0").ok()?,
            dns_bind_addr: SocketAddr::from_str("0.0.0.0:0").ok()?,
            static_dir: std::path::PathBuf::from("static"),
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
            session_inactivity_minutes: 15,
            access_token_minutes: 15,
            refresh_token_days: 30,
            dns_nameservers: vec!["ns1.hackflare.dev".into(), "ns2.hackflare.dev".into()],
            admin_emails: vec![],
            smtp: None,
            domain: None,
            cdn_url: None,
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
                INSERT INTO users (id, email, first_name, last_name, verification_status,
                                   ysws_eligible, hca_access_token, hca_refresh_token, hca_token_expires_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                "#,
            )
            .bind(&user_id)
            .bind(format!("{}@test.com", user_id))
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
            _embedded_postgres: embedded_postgres,
            jwt,
            user_id,
            session_id,
        })
    }

    fn authed_request(&self, method: &str, uri: &str, body: Option<&'static str>) -> Request<Body> {
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

// --- Tests ---

#[tokio::test]
async fn test_unauthenticated() {
    let Some(ctx) = TestCtx::setup().await else {
        eprintln!("skipping: database unavailable");
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
        eprintln!("skipping: database unavailable");
        return;
    };

    // Create zone
    let response = build_router(ctx.state.clone())
        .oneshot(ctx.authed_request(
            "POST",
            "/api/v1/dns/zones",
            Some(r#"{"name": "example.com"}"#),
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
    assert!(names.contains(&"example.com"));

    let response = build_router(ctx.state.clone())
        .oneshot(ctx.authed_request(
            "POST",
            "/api/v1/dns/zones",
            Some(r#"{"name": "example.invalid"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Duplicate zone returns 409
    let response = build_router(ctx.state.clone())
        .oneshot(ctx.authed_request(
            "POST",
            "/api/v1/dns/zones",
            Some(r#"{"name": "example.com"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);

    let zones = ctx.state.dns_authority.list_zones().await;
    assert!(zones.iter().any(|z| z == "example.com"));

    // Cleanup
    let _ = ctx.state.dns_authority.delete_zone("example.com").await;
    ctx.cleanup().await;
}

#[tokio::test]
async fn test_delete_zone() {
    let Some(ctx) = TestCtx::setup().await else {
        eprintln!("skipping: database unavailable");
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
        eprintln!("skipping: database unavailable");
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
    let record_id = records[0]["id"].as_str().unwrap().to_string();

    // Update by record id
    let response = build_router(ctx.state.clone())
        .oneshot(ctx.authed_request(
            "PUT",
            &format!("/api/v1/dns/zones/test-rec.com/records/{record_id}"),
            Some(r#"{"name":"www","type":"A","value":"10.0.0.1","ttl":600}"#),
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
    assert_eq!(rec["id"].as_str().unwrap(), record_id);

    // Delete record by id
    let response = build_router(ctx.state.clone())
        .oneshot(ctx.authed_request(
            "DELETE",
            &format!("/api/v1/dns/zones/test-rec.com/records/{record_id}"),
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
async fn test_delete_duplicate_record_keeps_others() {
    let Some(ctx) = TestCtx::setup().await else {
        eprintln!("skipping: database unavailable");
        return;
    };

    ctx.state.dns_authority.create_zone("test-dup.com").await;
    sqlx::query("UPDATE dns_zones SET user_id = $1 WHERE name = 'test-dup.com'")
        .bind(&ctx.user_id)
        .execute(&ctx.state.db)
        .await
        .ok();
    let _ = sqlx::query("UPDATE dns_zones SET ns_verified = true WHERE name = 'test-dup.com'")
        .execute(&ctx.state.db)
        .await;

    // Two TXT records at the same name (acme dns01 wildcard + root)
    let response = build_router(ctx.state.clone())
        .oneshot(ctx.authed_request(
            "POST",
            "/api/v1/dns/zones/test-dup.com/records",
            Some(r#"{"name":"_acme-challenge","type":"TXT","value":"challenge-one","ttl":60}"#),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let response = build_router(ctx.state.clone())
        .oneshot(ctx.authed_request(
            "POST",
            "/api/v1/dns/zones/test-dup.com/records",
            Some(r#"{"name":"_acme-challenge","type":"TXT","value":"challenge-two","ttl":60}"#),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let response = build_router(ctx.state.clone())
        .oneshot(ctx.authed_request("GET", "/api/v1/dns/zones/test-dup.com/records", None))
        .await
        .unwrap();
    let body = get_body(response).await;
    let records = body.as_array().unwrap();
    assert_eq!(records.len(), 2);
    let first_id = records[0]["id"].as_str().unwrap().to_string();

    // Deleting one record must leave the sibling intact
    let response = build_router(ctx.state.clone())
        .oneshot(ctx.authed_request(
            "DELETE",
            &format!("/api/v1/dns/zones/test-dup.com/records/{first_id}"),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let response = build_router(ctx.state.clone())
        .oneshot(ctx.authed_request("GET", "/api/v1/dns/zones/test-dup.com/records", None))
        .await
        .unwrap();
    let body = get_body(response).await;
    let records = body.as_array().unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0]["value"], "challenge-two");

    // Record ids that don't exist for this zone are 404
    let response = build_router(ctx.state.clone())
        .oneshot(ctx.authed_request(
            "DELETE",
            "/api/v1/dns/zones/test-dup.com/records/aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
            None,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let _ = ctx.state.dns_authority.delete_zone("test-dup.com").await;
    ctx.cleanup().await;
}

#[tokio::test]
async fn test_records_nonexistent_zone() {
    let Some(ctx) = TestCtx::setup().await else {
        eprintln!("skipping: database unavailable");
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
        eprintln!("skipping: database unavailable");
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
            "/api/v1/dns/zones/test-unverified.com/records/00000000-0000-0000-0000-000000000000",
            Some(r#"{"name":"www","type":"A","value":"10.0.0.1","ttl":600}"#),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    // Delete record should be blocked
    let response = build_router(ctx.state.clone())
        .oneshot(ctx.authed_request(
            "DELETE",
            "/api/v1/dns/zones/test-unverified.com/records/00000000-0000-0000-0000-000000000000",
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
        eprintln!("skipping: database unavailable");
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
