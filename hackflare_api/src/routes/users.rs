use axum::{Extension, Json, Router, middleware, response::IntoResponse, routing::get};
use serde_json::json;

use crate::{
    middlewares::{CurrentUser, auth_middleware},
    state::AppState,
};

async fn me_handler(Extension(user): Extension<CurrentUser>) -> impl IntoResponse {
    Json(json!({
        "id": user.claims.sub
    }))
}

pub(super) fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/me", get(me_handler))
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}
