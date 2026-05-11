mod config;
mod hca;
mod state;

use crate::config::Config;
use crate::state::AppState;
use axum::Router;
use dotenv::dotenv;
use tracing_subscriber::EnvFilter;

fn build_router(state: AppState) -> Router {
    Router::new().with_state(state)
}

#[macro_use]
extern crate tracing;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    if let Err(e) = dotenv() {
        error!("failed to load .env files: {}", e)
    }

    let config = Config::from_env();
    let state = AppState::new(config);
    let http_addr = "0.0.0.0:8080";

    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind(http_addr)
        .await
        .expect("failed to bind backend listener");

    info!("backend services listening");

    axum::serve(listener, app)
        .await
        .expect("backend server crashed");
}
