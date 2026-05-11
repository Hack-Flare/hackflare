use axum::Router;
use tower_http::trace::TraceLayer;

use crate::{config::Config, state::AppState};

mod auth;

fn v1_routes(config: &Config) -> Router<AppState> {
    Router::new().nest("/auth", auth::routes(config))
}

pub(crate) fn build_router(state: AppState) -> Router {
    Router::new()
        .nest("/api/v1", v1_routes(&state.config))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
