#[macro_use]
extern crate tracing;

pub mod api;
pub mod config;
pub mod models;
pub mod pages;
pub mod proxy;
pub mod routes;
pub mod state;

use anyhow::Context;

/// Bind the listener, build the router, and serve until shutdown.
pub async fn run() -> anyhow::Result<()> {
    let config = config::from_env().context("invalid frontend config")?;
    info!(
        dev = config.dev,
        proxy_target = %config.api_proxy_target,
        static_dir = %config.static_dir.display(),
        "frontend config loaded"
    );

    let listener = tokio::net::TcpListener::bind(&config.bind_addr)
        .await
        .context("failed to bind frontend listener")?;
    info!("listening on {}", config.bind_addr);

    let state = state::AppState::new(config);
    let app = routes::build_router(state);

    axum::serve(listener, app)
        .await
        .context("frontend server failed")
}
