use std::{env, process};

use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    let dotenv_error = dotenvy::dotenv();
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

    if let Err(e) = hackflare_frontend::run().await {
        error!("{:?}", e);
        process::exit(1);
    }
}
