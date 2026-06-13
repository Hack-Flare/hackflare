use axum::{
    Json, Router,
    extract::{Extension, Path, State},
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{delete, get, put},
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

#[derive(Deserialize)]
struct SetPasswordRequest {
    current_password: Option<String>,
    new_password: String,
}

async fn set_password(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<SetPasswordRequest>,
) -> impl IntoResponse {
    if req.new_password.len() < 8 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "password_too_short"})),
        )
            .into_response();
    }

    // If user already has a password, verify current_password
    if let Some(ref existing_hash) = current_user.user.password_hash {
        let current = req.current_password.as_deref().unwrap_or("");
        use argon2::{Argon2, PasswordHash, PasswordVerifier};
        let parsed = match PasswordHash::new(existing_hash) {
            Ok(p) => p,
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "internal_error"})),
                )
                    .into_response()
            }
        };
        if Argon2::default().verify_password(current.as_bytes(), &parsed).is_err() {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "current_password_incorrect"})),
            )
                .into_response();
        }
    }

    use argon2::{Argon2, PasswordHasher, password_hash::{SaltString, rand_core::OsRng}};
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = match Argon2::default()
        .hash_password(req.new_password.as_bytes(), &salt)
    {
        Ok(h) => h.to_string(),
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "hash_error"})),
            )
                .into_response()
        }
    };

    match sqlx::query("UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2")
        .bind(&password_hash)
        .bind(&current_user.user.id)
        .execute(&state.db)
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({"message": "password_updated"})),
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "db_error"})),
        )
            .into_response(),
    }
}

pub(super) fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/api-keys", get(list_keys).post(create_key))
        .route("/api-keys/{id}", delete(revoke_key))
        .route("/password", put(set_password))
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}
