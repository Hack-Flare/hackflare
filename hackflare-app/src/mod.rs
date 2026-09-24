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
use pg_embed::{
    pg_enums::PgAuthMethod,
    pg_fetch::{PG_V17, PgFetchSettings},
    postgres::{PgEmbed, PgSettings},
};
use reqwest::Url;
use tracing::{error, info};

async fn start_dev_database() -> Result<(PgEmbed, Url)> {
    let port = {
        let listener = std::net::TcpListener::bind(("127.0.0.1", 0))?;
        listener.local_addr()?.port()
    };
    let pg_settings = PgSettings {
        database_dir: std::env::temp_dir()
            .join(format!("hackflare-dev-postgres-{}", std::process::id())),
        port,
        user: "postgres".to_string(),
        password: "postgres".to_string(),
        auth_method: PgAuthMethod::Plain,
        persistent: false,
        timeout: Some(std::time::Duration::from_secs(30)),
        migration_dir: None,
    };
    let fetch_settings = PgFetchSettings {
        version: PG_V17,
        ..Default::default()
    };
    let mut postgres = PgEmbed::new(pg_settings, fetch_settings).await?;
    postgres.setup().await?;
    postgres.start_db().await?;
    postgres.create_database("hackflare_dev").await?;
    let database_url = Url::parse(&postgres.full_db_uri("hackflare_dev"))?;
    Ok((postgres, database_url))
}

pub async fn run(dev_mode: bool) -> Result<()> {
    let (_embedded_postgres, database_url) = if dev_mode {
        info!("starting embedded PostgreSQL for development");
        let (postgres, database_url) = start_dev_database().await?;
        (Some(postgres), Some(database_url))
    } else {
        (None, None)
    };
    let mut config = match database_url {
        Some(database_url) => config::from_env_with_database_url(Some(database_url)),
        None => config::from_env(),
    }
    .context("invalid environment")?;
    if dev_mode {
        config.auto_migrate = true;
        config.dns_bind_addr = SocketAddr::from(([0, 0, 0, 0], 5454));
    }
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

    async fn shutdown_signal() {
        if let Err(error) = tokio::signal::ctrl_c().await {
            error!(%error, "failed to install Ctrl-C handler");
        }
    }

    let app = frontend::routes::build_router(state.clone()).merge(api::routes::build_router(state));
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .context("application server failed")?;
    Ok(())
}
