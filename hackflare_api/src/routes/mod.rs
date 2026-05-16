use axum::{Router, routing::post};
use tower_http::trace::TraceLayer;

use crate::{config::Config, state::AppState};

pub(crate) mod auth;
pub(crate) mod sessions;
pub mod slack;
pub(crate) mod users;

fn v1_routes(state: AppState, config: &Config) -> Router<AppState> {
    Router::new()
        .nest("/auth", auth::routes(config))
        .nest("/users", users::routes(state.clone()))
        .nest("/sessions", sessions::routes(state))
        .route("/slack/contact", post(slack::slack_contact))
}

pub fn build_router(state: AppState) -> Router {
    let ip_source_extension = state.config.client_ip_source.clone().into_extension();
    Router::new()
        .nest("/api/v1", v1_routes(state.clone(), &state.config))
        .layer(TraceLayer::new_for_http())
        .layer(ip_source_extension)
        .with_state(state)
}
