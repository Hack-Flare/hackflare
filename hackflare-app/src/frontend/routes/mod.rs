//! Route wiring for the frontend server

use axum::{
    Router,
    routing::get,
};
use tower_http::services::ServeDir;

pub mod dash;
pub mod handlers;
pub mod public;

use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(handlers::home))
        .route("/auth", get(handlers::auth_redirect))
        .route(
            "/login",
            get(handlers::login_get).post(handlers::login_post),
        )
        .route(
            "/register",
            get(handlers::register_get).post(handlers::register_post),
        )
        .route(
            "/forgot-password",
            get(handlers::forgot_get).post(handlers::forgot_post),
        )
        .route(
            "/reset-password",
            get(handlers::reset_get).post(handlers::reset_post),
        )
        .route("/auth/hackclub", get(handlers::hackclub))
        .route("/logout", get(handlers::logout))
        .route("/health", get(handlers::health))
        .route("/docs", get(public::docs))
        .route("/ourteam", get(public::team))
        .route("/dash", get(dash::index))
        .route("/dash/domains", get(dash::domains_get).post(dash::domains_post))
        .route("/dash/settings", get(dash::settings_get).post(dash::settings_post))
        .route(
            "/dash/notifications",
            get(dash::notifications_get).post(dash::notifications_post),
        )
        .route("/dash/{section}", get(dash::section))
        .route(
            "/dash/domains/{domain}/dns",
            get(dash::dns_get).post(dash::dns_post),
        )
        .route("/dash/domains/{domain}/{sub}", get(dash::domain_sub))
        .nest_service("/static", ServeDir::new(state.config.static_dir.clone()))
        .fallback(handlers::not_found)
        .with_state(state)
}
