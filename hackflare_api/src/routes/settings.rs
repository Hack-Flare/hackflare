use axum::{
    Json, Router,
    extract::{Extension, Path, State},
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{delete, get},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{middlewares::auth_middleware, models::CurrentUser, state::AppState};

#[derive(Serialize)]
pub(super) struct ApiKeyResponse {
    id: String,
    name: String,
    prefix: String,
    created_at: String,
    last_used_at: Option<String>,
    revoked: bool,
}

#[derive(Serialize)]
pub(super) struct CreatedKeyResponse {
    key: ApiKeyResponse,
    raw_key: String,
}

#[derive(Deserialize)]
pub(super) struct CreateKeyRequest {
    name: String,
}

impl From<crate::services::api_keys::ApiKey> for ApiKeyResponse {
    fn from(k: crate::services::api_keys::ApiKey) -> Self {
        Self {
            id: k.id.to_string(),
            name: k.name,
            prefix: k.prefix,
            created_at: k.created_at.to_rfc3339(),
            last_used_at: k.last_used_at.map(|t| t.to_rfc3339()),
            revoked: k.revoked_at.is_some(),
        }
    }
}

pub(super) async fn list_keys(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<ApiKeyResponse>>, StatusCode> {
    let keys = state
        .api_keys
        .list(&current_user.user.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(keys.into_iter().map(Into::into).collect()))
}

pub(super) async fn create_key(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateKeyRequest>,
) -> impl IntoResponse {
    let name = req.name.trim().to_string();
    if name.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "name is required"})),
        )
            .into_response();
    }

    match state.api_keys.create(&current_user.user.id, &name).await {
        Ok((key, raw_key)) => (
            StatusCode::CREATED,
            Json(
                serde_json::to_value(CreatedKeyResponse {
                    key: key.into(),
                    raw_key,
                })
                .unwrap_or_default(),
            ),
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "failed to create key"})),
        )
            .into_response(),
    }
}

pub(super) async fn revoke_key(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> StatusCode {
    match state.api_keys.revoke(id, &current_user.user.id).await {
        Ok(true) => StatusCode::NO_CONTENT,
        Ok(false) => StatusCode::NOT_FOUND,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub(super) fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/api-keys", get(list_keys).post(create_key))
        .route("/api-keys/{id}", delete(revoke_key))
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}
