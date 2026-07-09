use std::collections::HashMap;

use axum::{
    Json, Router,
    extract::{Extension, Path, State},
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{get, post, put},
};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{
    config::{HcaConfig, SmtpConfig},
    middlewares::{admin::require_admin, auth_middleware},
    models::CurrentUser,
    services::{config_overrides::ConfigEntry, email::EmailService},
    state::AppState,
};

#[derive(Serialize)]
pub(super) struct StatsResponse {
    total_users: i64,
    total_zones: i64,
    total_sessions: i64,
}

#[derive(Deserialize)]
pub(super) struct UpsertConfigRequest {
    value: String,
}

pub(super) async fn list_config(
    State(state): State<AppState>,
) -> Result<Json<Vec<ConfigEntry>>, StatusCode> {
    let env_map = build_env_map(&state.config);
    let overrides = state
        .config_overrides
        .list_overrides()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let overrides_map: HashMap<String, _> =
        overrides.into_iter().map(|o| (o.key.clone(), o)).collect();

    let entries: Vec<ConfigEntry> =
        crate::services::config_overrides::ConfigOverridesService::get_known_keys()
            .iter()
            .map(|meta| {
                let env_value = env_map.get(meta.key).cloned();
                let ov = overrides_map.get(meta.key);
                let default_value = meta.default_value.map(|s| s.to_string());
                let effective_value = ov
                    .map(|o| o.value.clone())
                    .or_else(|| {
                        if meta.default_override {
                            default_value.clone()
                        } else {
                            env_value.clone()
                        }
                    })
                    .or_else(|| {
                        if meta.default_override {
                            env_value.clone()
                        } else {
                            default_value.clone()
                        }
                    })
                    .unwrap_or_default();

                ConfigEntry {
                    key: meta.key.to_string(),
                    label: meta.label.to_string(),
                    description: meta.description.to_string(),
                    env_value,
                    override_value: ov.map(|o| o.value.clone()),
                    effective_value,
                    default_value,
                    default_override: meta.default_override,
                    editable: meta.editable,
                    requires_restart: meta.requires_restart,
                    updated_at: ov.map(|o| o.updated_at),
                    updated_by: ov.map(|o| o.updated_by.clone()),
                }
            })
            .collect();

    Ok(Json(entries))
}

pub(super) async fn upsert_config(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(key): Path<String>,
    Json(req): Json<UpsertConfigRequest>,
) -> impl IntoResponse {
    if !crate::services::config_overrides::ConfigOverridesService::is_editable(&key) {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "config key is not editable"})),
        )
            .into_response();
    }

    state
        .config_overrides
        .upsert(&key, &req.value, &current_user.user.id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "failed to upsert config"})),
            )
        })
        .ok();

    (StatusCode::OK, Json(serde_json::json!({"status": "ok"}))).into_response()
}

pub(super) async fn delete_config(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> impl IntoResponse {
    let deleted = state
        .config_overrides
        .delete(&key)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub(super) async fn list_users(
    State(db): State<PgPool>,
) -> Result<Json<Vec<UserResponse>>, StatusCode> {
    #[derive(sqlx::FromRow)]
    struct UserRow {
        id: String,
        email: String,
        first_name: String,
        last_name: String,
        verification_status: String,
        created_at: chrono::DateTime<chrono::Utc>,
    }

    let rows = sqlx::query_as::<_, UserRow>(
        r#"
        SELECT id, email, first_name, last_name, verification_status, created_at
        FROM users
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(&db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(
        rows.into_iter()
            .map(|r| UserResponse {
                id: r.id,
                email: r.email,
                first_name: r.first_name,
                last_name: r.last_name,
                status: r.verification_status,
                created_at: r.created_at,
            })
            .collect(),
    ))
}

pub(super) async fn get_stats(State(db): State<PgPool>) -> Result<Json<StatsResponse>, StatusCode> {
    let total_users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total_zones: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM dns_zones")
        .fetch_one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total_sessions: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM user_sessions WHERE revoked_at IS NULL")
            .fetch_one(&db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(StatsResponse {
        total_users,
        total_zones,
        total_sessions,
    }))
}

#[derive(Serialize)]
pub(super) struct UserResponse {
    id: String,
    email: String,
    first_name: String,
    last_name: String,
    status: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

fn build_env_map(config: &crate::config::Config) -> HashMap<&'static str, String> {
    let mut m = HashMap::new();
    m.insert("API_BIND_ADDR", config.bind_addr.to_string());
    m.insert("API_DNS_BIND_ADDR", config.dns_bind_addr.to_string());
    m.insert("API_ENVIRONMENT", config.environment.to_string());
    m.insert("API_AUTO_MIGRATE", config.auto_migrate.to_string());
    m.insert("API_HCA_CLIENT_ID", config.hca.client_id.clone());
    m.insert("API_HCA_CLIENT_SECRET", "********".to_string());
    m.insert("API_HCA_REDIRECT_URI", config.hca.redirect_uri.to_string());
    m.insert(
        "API_ACCESS_TOKEN_MINUTES",
        config.access_token_minutes.to_string(),
    );
    m.insert(
        "API_REFRESH_TOKEN_DAYS",
        config.refresh_token_days.to_string(),
    );
    m.insert(
        "API_SESSION_INACTIVITY_MINUTES",
        config.session_inactivity_minutes.to_string(),
    );
    m.insert("API_DNS_NAMESERVERS", config.dns_nameservers.join(","));
    m.insert(
        "API_CLIENT_IP_SOURCE",
        format!("{:?}", config.client_ip_source),
    );
    m.insert("API_ADMIN_EMAILS", config.admin_emails.join(","));
    m.insert(
        "SLACK_WEBHOOK_URL",
        config
            .slack_webhook_url
            .as_ref()
            .map(|u| u.to_string())
            .unwrap_or_default(),
    );
    m.insert("DATABASE_URL", "postgres://****@****/****".to_string());
    m.insert("API_JWT_SECRET", "********".to_string());

    m.insert(
        "FRONTEND_URL",
        config
            .frontend_url
            .as_ref()
            .map(|u| u.to_string())
            .unwrap_or_default(),
    );

    if let Some(smtp) = &config.smtp {
        m.insert("SMTP_HOST", smtp.host.clone());
        m.insert("SMTP_PORT", smtp.port.to_string());
        m.insert("SMTP_USERNAME", smtp.username.clone());
        m.insert("SMTP_PASSWORD", "********".to_string());
        m.insert("SMTP_FROM", smtp.from.clone());
    }

    m
}

fn override_or_env(key: &str, overrides: &HashMap<String, String>) -> Option<String> {
    overrides
        .get(key)
        .cloned()
        .filter(|v| !v.is_empty())
        .or_else(|| std::env::var(key).ok())
}

async fn apply_config(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, &'static str)> {
    let overrides = state
        .config_overrides
        .list_overrides()
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "db_error"))?;

    let map: HashMap<String, String> = overrides.into_iter().map(|o| (o.key, o.value)).collect();

    // Update live overrides
    *state.live_overrides.write().await = map.clone();

    // Reload HCA config from overrides
    if let Some(redirect_uri_str) =
        override_or_env("API_HCA_REDIRECT_URI", &map).filter(|v| !v.is_empty())
        && let Ok(redirect_uri) = Url::parse(&redirect_uri_str)
    {
        let hca = HcaConfig {
            client_id: override_or_env("API_HCA_CLIENT_ID", &map)
                .filter(|v| !v.is_empty())
                .unwrap_or_else(|| state.config.hca.client_id.clone()),
            client_secret: override_or_env("API_HCA_CLIENT_SECRET", &map)
                .filter(|v| !v.is_empty())
                .unwrap_or_else(|| state.config.hca.client_secret.clone()),
            redirect_uri,
        };
        *state.live_hca.write().await = hca;
        info!("live HCA config updated from overrides");
    }

    // Try to rebuild email service from overrides + env
    if let (Some(host), Some(username), Some(password), Some(from)) = (
        override_or_env("SMTP_HOST", &map),
        override_or_env("SMTP_USERNAME", &map),
        override_or_env("SMTP_PASSWORD", &map),
        override_or_env("SMTP_FROM", &map),
    ) {
        let port: u16 = override_or_env("SMTP_PORT", &map)
            .and_then(|p| p.parse().ok())
            .unwrap_or(587);
        let smtp = SmtpConfig {
            host,
            port,
            username,
            password,
            from,
        };
        *state.email.write().await = Some(EmailService::new(&smtp));
    } else {
        // SMTP config incomplete -> disable email service
        *state.email.write().await = None;
    }

    Ok(Json(serde_json::json!({"status": "applied"})))
}

pub(super) fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/config", get(list_config))
        .route("/config/apply", post(apply_config))
        .route("/config/{key}", put(upsert_config).delete(delete_config))
        .route("/users", get(list_users))
        .route("/stats", get(get_stats))
        .merge(super::traffic::admin_traffic_routes())
        .layer(middleware::from_fn_with_state(state.clone(), require_admin))
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}
