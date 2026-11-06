use axum::{
    Extension, Json, Router,
    extract::State,
    http::StatusCode,
    middleware,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

use crate::{
    middlewares::{admin::require_admin, auth_middleware},
    models::CurrentUser,
    state::AppState,
};

#[derive(Serialize)]
pub(super) struct StatsResponse {
    total_users: i64,
    total_zones: i64,
    total_sessions: i64,
}

pub(super) async fn list_users(
    State(state): State<AppState>,
) -> Result<Json<Vec<UserResponse>>, StatusCode> {
    #[derive(sqlx::FromRow)]
    struct UserRow {
        id: String,
        email: String,
        first_name: String,
        last_name: String,
        verification_status: String,
        is_admin: bool,
        created_at: chrono::DateTime<chrono::Utc>,
    }

    let rows = sqlx::query_as::<_, UserRow>(
        r#"
        SELECT id, email, first_name, last_name, verification_status, is_admin, created_at
        FROM users
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(&state.db)
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
                is_admin: r.is_admin,
                created_at: r.created_at,
            })
            .collect(),
    ))
}

pub(super) async fn get_stats(State(state): State<AppState>) -> Result<Json<StatsResponse>, StatusCode> {
    let total_users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total_zones: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM dns_zones")
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total_sessions: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM user_sessions WHERE revoked_at IS NULL")
            .fetch_one(&state.db)
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
    is_admin: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize)]
pub(super) struct AdminGrantRequest {
    email: String,
    #[serde(default = "default_true")]
    admin: bool,
}

fn default_true() -> bool {
    true
}

pub(super) async fn grant_admin(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(body): Json<AdminGrantRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, &'static str)> {
    let email = body.email.trim();
    if email.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "email_required"));
    }
    if !body.admin && email.eq_ignore_ascii_case(&current_user.user.email) {
        return Err((StatusCode::BAD_REQUEST, "cannot_remove_self"));
    }
    match state.users.set_admin_by_email(email, body.admin).await {
        Ok(true) => Ok(Json(
            serde_json::json!({ "ok": true, "email": email, "admin": body.admin }),
        )),
        Ok(false) => Err((StatusCode::NOT_FOUND, "user_not_found")),
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "internal")),
    }
}

pub(super) fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/users", get(list_users))
        .route("/users/admin", post(grant_admin))
        .route("/stats", get(get_stats))
        .merge(super::traffic::admin_traffic_routes())
        .layer(middleware::from_fn_with_state(state.clone(), require_admin))
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}
