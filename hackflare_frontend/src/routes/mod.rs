//! Route wiring for the frontend server

use axum::{
    Router,
    routing::{any, get},
};
use tower_http::services::ServeDir;

pub mod dash;
pub mod handlers;

use crate::{proxy::proxy, state::AppState};

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(handlers::home))
        .route("/auth", get(handlers::auth_redirect))
        .route("/login", get(handlers::login_get).post(handlers::login_post))
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
        .route("/dash", get(dash::index))
        .route("/dash/{section}", get(dash::section))
        .route("/dash/domains/{domain}/{sub}", get(dash::domain_sub))
        .route("/api/{*path}", any(proxy))
        .nest_service(
            "/static",
            ServeDir::new(state.config.static_dir.clone()),
        )
        .fallback(handlers::not_found)
        .with_state(state)
}