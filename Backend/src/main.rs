mod app;
mod config;
mod domain;
mod recursive;
mod nameserver;
mod state;

use crate::app::build_router;
use crate::config::Config;
use crate::nameserver::run_dns_server;
use crate::state::AppState;
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
    let http_bind_addr = config.bind_addr;
    let dns_bind_addr = config.dns_bind_addr;

    let state = AppState::new(config);
    let dns_state = state.dns.clone();

    tokio::spawn(async move {
        if let Err(err) = run_dns_server(dns_bind_addr, dns_state).await {
            tracing::error!(%err, "dns nameserver terminated");
        }
    });

    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind(http_bind_addr)
        .await
        .expect("failed to bind backend listener");

    info!(%http_bind_addr, %dns_bind_addr, "backend services listening");

    axum::serve(listener, app)
        .await
        .expect("backend server crashed");
}
