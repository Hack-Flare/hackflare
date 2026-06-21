use axum::{Router, routing::get};
use tower_http::trace::TraceLayer;

use crate::{config::Config, state::AppState};

pub(crate) mod admin;
pub(crate) mod auth;
pub(crate) mod dns;
pub(crate) mod health;
pub(crate) mod logs;
pub(crate) mod sessions;
pub(crate) mod settings;
pub mod slack;
pub(crate) mod traffic;
pub(crate) mod users;

fn v1_routes(state: AppState, config: &Config) -> Router<AppState> {
    Router::new()
        .route("/health", get(health::health))
        .nest("/auth", auth::routes(config))
        .nest("/users", users::routes(state.clone()))
        .nest("/sessions", sessions::routes(state.clone()))
        .nest("/dns", dns::routes(state.clone()))
        .nest("/admin", admin::routes(state.clone()))
        .nest("/settings", settings::routes(state.clone()))
        .nest("/logs", logs::routes(state.clone()))
        .nest(
            "/traffic",
            traffic::user_routes().route_layer(axum::middleware::from_fn_with_state(
                state.clone(),
                crate::middlewares::auth_middleware,
            )),
        )
        .route("/slack/contact", axum::routing::post(slack::slack_contact))
}

pub fn build_router(state: AppState) -> Router {
    let ip_source_extension = state.config.client_ip_source.clone().into_extension();
    Router::new()
        .nest("/api/v1", v1_routes(state.clone(), &state.config))
        .layer(TraceLayer::new_for_http())
        .layer(ip_source_extension)
        .with_state(state)
}
