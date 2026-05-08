mod app;
mod config;
mod domain;
mod state;

use crate::app::build_router;
use crate::config::Config;
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,backend=debug".into()),
        )
        .init();

    let config = Config::from_env();
    let bind_addr = config.bind_addr;

    let app = build_router(config);
    let listener = tokio::net::TcpListener::bind(bind_addr)
        .await
        .expect("failed to bind backend listener");

    info!(%bind_addr, "backend listening");

    axum::serve(listener, app)
        .await
        .expect("backend server crashed");
}
