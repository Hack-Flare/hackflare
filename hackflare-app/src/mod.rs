pub mod api;
pub mod auth;
pub mod client_ip;
pub mod config;
pub mod frontend;
pub mod state;

pub use api::{middlewares, models, services};

use std::net::SocketAddr;

use anyhow::{Context, Result};
use hackflare_dns::ns::{NsConfig, run as run_dns};
#[cfg(feature = "embedded-db")]
use postgresql_embedded::{PostgreSQL, SettingsBuilder, V17};
use reqwest::Url;
use tracing::{error, info};

#[cfg(feature = "embedded-db")]
async fn start_dev_database() -> Result<(PostgreSQL, Url)> {
    let port = {
        let listener = std::net::TcpListener::bind(("127.0.0.1", 0))?;
        listener.local_addr()?.port()
    };
    let settings = SettingsBuilder::new()
        .host("127.0.0.1")
        .port(port)
        .username("postgres")
        .password("postgres")
        .data_dir(std::env::temp_dir().join(format!(
            "hackflare-dev-postgres-{}",
            std::process::id()
        )))
        .temporary(true)
        .timeout(Some(std::time::Duration::from_secs(30)))
        .version(V17.clone())
        .build();
    let mut postgres = PostgreSQL::new(settings);
    postgres.setup().await?;
    postgres.start().await?;
    postgres.create_database("hackflare_dev").await?;
    let database_url = Url::parse(&postgres.settings().url("hackflare_dev"))?;
    Ok((postgres, database_url))
}

pub async fn run(dev_mode: bool) -> Result<()> {
    #[cfg(feature = "embedded-db")]
    let (_embedded_postgres, database_url) = if dev_mode {
        info!("starting embedded PostgreSQL for development");
        let (postgres, database_url) = start_dev_database().await?;
        (Some(postgres), Some(database_url))
    } else {
        (None, None)
    };
    #[cfg(not(feature = "embedded-db"))]
    let database_url: Option<Url> = None;

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
    let listener = tokio::net::TcpListener::bind(config.bind_addr)
        .await
        .context("failed to bind application listener")?;
    let state = state::AppState::new(config)
        .await
        .context("failed to initialize application state")?;

    let dns_authority = state.dns_authority.clone();
    let dns_config = state.dns_config.clone();
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
            if let Err(error) = run_dns(
                ns_config,
                dns_authority,
                dns_config,
                Some(db),
            ) {
                error!("DNS server failed: {error}");
            }
        })
        .context("failed to spawn DNS server thread")?;

    async fn shutdown_signal() {
        if let Err(error) = tokio::signal::ctrl_c().await {
            error!(%error, "failed to install Ctrl-C handler");
        }
    }

    let app = frontend::routes::build_router(state.clone())
        .merge(api::routes::build_router(state.clone()))
        .layer(frontend::cors::layer(&state.config));
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .context("application server failed")?;
    Ok(())
}
