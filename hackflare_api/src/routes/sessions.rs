use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    middleware,
    routing::get,
};
use reqwest::StatusCode;
use uuid::Uuid;

use crate::{
    middlewares::auth_middleware,
    models::{CurrentUser, db::UserSession},
    services::user_sessions::UserSessionsService,
    state::AppState,
};

async fn sessions_handler(
    Extension(current_user): Extension<CurrentUser>,
    State(sessions): State<UserSessionsService>,
) -> Result<Json<Vec<UserSession>>, (StatusCode, &'static str)> {
    let result = sessions
        .get_all_for_user(&current_user.user.id)
        .await
        .map_err(|error| {
            error!(%error, "failed to get user sessions");
            (StatusCode::INTERNAL_SERVER_ERROR, "db_error")
        })?;
    Ok(Json(result))
}

async fn session_by_id_handler(
    Extension(current_user): Extension<CurrentUser>,
    State(sessions): State<UserSessionsService>,
    Path(id): Path<Uuid>,
) -> Result<Json<UserSession>, (StatusCode, &'static str)> {
    let result = sessions.get_by_id(&id).await.map_err(|error| {
        error!(%error, "failed to get user sessions");
        (StatusCode::INTERNAL_SERVER_ERROR, "db_error")
    })?;

    let Some(session) = result else {
        return Err((StatusCode::NOT_FOUND, "session_not_found"));
    };

    if session.user_id != current_user.user.id {
        return Err((StatusCode::FORBIDDEN, "forbidden"));
    }

    Ok(Json(session))
}

pub(super) fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(sessions_handler))
        .route("/{id}", get(session_by_id_handler))
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}
