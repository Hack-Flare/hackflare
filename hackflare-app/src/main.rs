#[macro_use]
extern crate tracing;

include!("mod.rs");

#[tokio::main]
async fn main() {
    let dotenv_error = dotenvy::dotenv();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    if let Err(error) = dotenv_error
        && !error.not_found()
    {
        tracing::warn!(%error, "failed to load .env files");
    }

    let dev_mode = std::env::args().skip(1).any(|arg| arg == "--dev");

    if let Err(error) = run(dev_mode).await {
        tracing::error!(?error, "application failed");
        std::process::exit(1);
    }
}
