use axum::{
    Extension, Json, Router, extract::State, middleware, response::IntoResponse, routing::get,
};
use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::{middlewares::auth_middleware, models::CurrentUser, state::AppState};

#[derive(Serialize)]
struct Me {
    id: String,
    slack_id: Option<String>,
    first_name: String,
    last_name: String,
    email: String,
    eligible: bool,
    has_password: bool,
    is_admin: bool,
    created_at: DateTime<Utc>,
}

async fn me_handler(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> impl IntoResponse {
    let user = current_user.user;
    let is_admin = state.config.admin_emails.iter().any(|e| e == &user.email);
    Json(Me {
        id: user.id,
        slack_id: user.slack_id,
        first_name: user.first_name,
        last_name: user.last_name,
        email: user.email,
        eligible: user.ysws_eligible,
        has_password: user.password_hash.is_some(),
        is_admin,
        created_at: user.created_at,
    })
}

pub(super) fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/me", get(me_handler))
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}
