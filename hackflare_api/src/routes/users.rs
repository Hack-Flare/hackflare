use axum::{Extension, Json, Router, middleware, response::IntoResponse, routing::get};
use serde::Serialize;

use crate::{
    middlewares::auth_middleware,
    models::{CurrentUser, db::User},
    state::AppState,
};

#[derive(Serialize)]
struct Me {
    id: String,
    slack_id: String,
    first_name: String,
    last_name: String,
    email: String,
    eligible: bool,
}

impl From<User> for Me {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            slack_id: user.slack_id,
            first_name: user.first_name,
            last_name: user.last_name,
            email: user.email,
            eligible: user.ysws_eligible,
        }
    }
}

async fn me_handler(Extension(current_user): Extension<CurrentUser>) -> impl IntoResponse {
    Json(Me::from(current_user.user))
}

pub(super) fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/me", get(me_handler))
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}
