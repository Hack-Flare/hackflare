use std::env;

use anyhow::{Context, Result};
use dotenv::dotenv;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

use hackflare_api::{routes::build_router, state::AppState};

async fn run() -> Result<()> {
    let config = hackflare_api::config::from_env().context("invalid .env")?;
    info!("initialized config");

    let listener = tokio::net::TcpListener::bind(&config.bind_addr)
        .await
        .context("failed to bind api listener")?;
    info!("listening on {}", config.bind_addr);

    let state = AppState::new(config)
        .await
        .context("failed to set up app state")?;
    let app = build_router(state);

    axum::serve(listener, app).await?;

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
