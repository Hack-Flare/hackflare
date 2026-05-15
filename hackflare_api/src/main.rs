use dotenv::dotenv;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

use hackflare_api::{routes::build_router, state::AppState};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    if let Err(e) = dotenv()
        && !e.not_found()
    {
        warn!("failed to load .env files: {}", e)
    }

    let config = match hackflare_api::config::from_env() {
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

    let state = AppState::new(config)
        .await
        .expect("failed to set up app state");
    let app = build_router(state);

    axum::serve(listener, app)
        .await
        .expect("backend server crashed");
}
