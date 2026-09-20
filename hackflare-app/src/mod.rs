pub(crate) mod api;
pub(crate) mod auth;
pub(crate) mod config;
pub(crate) mod frontend;
pub(crate) mod state;

pub(crate) use api::{middlewares, models, services};

use std::net::SocketAddr;

use anyhow::{Context, Result};
use hackflare_dns::{
    DnsConfig,
    ns::{NsConfig, run_with_hickory},
};
use tracing::{error, info};

pub async fn run() -> Result<()> {
    let config = config::from_env().context("invalid environment")?;
    let dns_bind_addr = config.dns_bind_addr;
    let dns_config = DnsConfig::from_env();
    let listener = tokio::net::TcpListener::bind(config.bind_addr)
        .await
        .context("failed to bind application listener")?;
    let state = state::AppState::new(config)
        .await
        .context("failed to initialize application state")?;

    let dns_authority = state.dns_authority.clone();
    let db = state.db.clone();
    std::thread::Builder::new()
        .name("hackflare-dns".into())
        .spawn(move || {
            let ns_config = NsConfig {
                bind_addr: dns_bind_addr.ip().to_string(),
                port: dns_bind_addr.port(),
                zone_file: None,
                database_url: None,
            };
            info!("starting DNS server on {dns_bind_addr}");
            if let Err(error) = run_with_hickory(ns_config, dns_authority, dns_config, Some(db)) {
                error!("DNS server failed: {error}");
            }
        })
        .context("failed to spawn DNS server thread")?;

    let app = frontend::routes::build_router(state.clone())
        .merge(api::routes::build_router(state));
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .context("application server failed")?;
    Ok(())
}
