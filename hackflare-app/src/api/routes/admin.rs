use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    middleware,
    routing::get,
};
use serde::Serialize;
use sqlx::PgPool;

use crate::{
    middlewares::{admin::require_admin, auth_middleware},
    state::AppState,
};

#[derive(Serialize)]
pub(super) struct StatsResponse {
    total_users: i64,
    total_zones: i64,
    total_sessions: i64,
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

pub(super) fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/users", get(list_users))
        .route("/stats", get(get_stats))
        .merge(super::traffic::admin_traffic_routes())
        .layer(middleware::from_fn_with_state(state.clone(), require_admin))
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}
