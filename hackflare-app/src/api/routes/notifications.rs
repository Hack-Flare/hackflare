use axum::{
    Json, Router,
    extract::{Extension, Path, State},
    http::StatusCode,
    routing::{get, put},
};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    models::{CurrentUser, db::UserNotification},
    services,
    state::AppState,
};

#[derive(Serialize)]
pub(crate) struct UnreadCountResponse {
    count: i64,
}

pub(crate) async fn list(
    State(db): State<sqlx::PgPool>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<UserNotification>>, StatusCode> {
    services::notifications::list_notifications(&db, &current_user.user.id, 50)
        .await
        .map(Json)
        .map_err(|e| {
            tracing::error!("failed to list notifications: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

pub(crate) async fn get_unread_count(
    State(db): State<sqlx::PgPool>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<UnreadCountResponse>, StatusCode> {
    services::notifications::unread_count(&db, &current_user.user.id)
        .await
        .map(|count| Json(UnreadCountResponse { count }))
        .map_err(|e| {
            tracing::error!("failed to get unread count: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

pub(crate) async fn mark_one_read(
    State(db): State<sqlx::PgPool>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<()>, StatusCode> {
    services::notifications::mark_read(&db, id, &current_user.user.id)
        .await
        .map(Json)
        .map_err(|e| {
            tracing::error!("failed to mark notification read: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

pub(crate) async fn mark_all_read(
    State(db): State<sqlx::PgPool>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<()>, StatusCode> {
    services::notifications::mark_all_read(&db, &current_user.user.id)
        .await
        .map(Json)
        .map_err(|e| {
            tracing::error!("failed to mark all notifications read: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list))
        .route("/unread-count", get(get_unread_count))
        .route("/{id}/read", put(mark_one_read))
        .route("/read-all", put(mark_all_read))
}
