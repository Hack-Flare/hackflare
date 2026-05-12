mod config;
mod hca;
mod middlewares;
mod routes;
mod state;

use crate::routes::build_router;
use crate::state::AppState;
use dotenv::dotenv;
use tracing_subscriber::EnvFilter;

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

    let config = match config::from_env() {
        Ok(c) => c,
        Err(e) => {
            error!("invalid .env: {}", e);
            return;
        }
    };
    info!("initialized config");

    let listener = tokio::net::TcpListener::bind(&config.bind_addr)
        .await
        .expect("failed to bind api listener");
    info!("listening on {}", config.bind_addr);

    let state = AppState::new(config);
    let app = build_router(state);

    axum::serve(listener, app)
        .await
        .expect("backend server crashed");
}
