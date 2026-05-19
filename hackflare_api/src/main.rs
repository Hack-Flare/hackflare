use std::{env, net::SocketAddr};

use anyhow::{Context, Result};
use dotenvy::dotenv;
use hackflare_dns::{
    DnsConfig,
    ns::{NsConfig, run_with_hickory},
};
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

use hackflare_api::{routes::build_router, state::AppState};

async fn run() -> Result<()> {
    let config = hackflare_api::config::from_env().context("invalid .env")?;
    info!("initialized config");

    let dns_bind_addr = config.dns_bind_addr;
    let dns_config = DnsConfig::from_env();

    let listener = tokio::net::TcpListener::bind(&config.bind_addr)
        .await
        .context("failed to bind api listener")?;
    info!("listening on {}", config.bind_addr);

    let state = AppState::new(config)
        .await
        .context("failed to set up app state")?;

    // Spawn the DNS server on a background thread
    let dns_authority = state.dns_authority.clone();
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
            if let Err(e) = run_with_hickory(ns_config, dns_authority, dns_config) {
                error!("DNS server failed: {e}");
            } else {
                info!("DNS server stopped");
            }
        })
        .context("failed to spawn DNS server thread")?;
    info!("DNS server thread spawned");

    let app = build_router(state);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    let dotenv_error = dotenv();
    let is_production = matches!(
        env::var("API_ENVIRONMENT").as_deref(),
        Ok("production" | "staging")
    );

    let env_filter = EnvFilter::from_default_env();
    if is_production {
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(env_filter)
            .init();
    } else {
        tracing_subscriber::fmt().with_env_filter(env_filter).init();
    }
    info!(is_production, "tracing initialized");

    if let Err(e) = dotenv_error
        && !e.not_found()
    {
        warn!("failed to load .env files: {}", e)
    }

    if let Err(e) = run().await {
        error!("{:?}", e);
        std::process::exit(1);
    }
}
