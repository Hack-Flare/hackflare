use std::{env, net::SocketAddr, path::PathBuf, str::FromStr};

use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub bind_addr: SocketAddr,
    /// Enables local UI development without a running API or session.
    pub dev: bool,
    /// Base URL of the backend API that `/api/*` requests are proxied to.
    pub api_proxy_target: String,
    /// Directory served at `/static/*`.
    pub static_dir: PathBuf,
}

fn env_or<T>(name: &str, default: T) -> Result<T>
where
    T: FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    let type_name = std::any::type_name::<T>();
    match env::var(name) {
        Ok(s) => s
            .trim()
            .parse::<T>()
            .with_context(|| format!("{name} has an invalid value for type {type_name}")),
        Err(env::VarError::NotPresent) => Ok(default),
        Err(env::VarError::NotUnicode(s)) => {
            anyhow::bail!("{name} contains invalid unicode: {}", s.display())
        }
    }
}

pub fn from_env() -> Result<Config> {
    Ok(Config {
        bind_addr: env_or(
            "FRONTEND_BIND_ADDR",
            "0.0.0.0:3000"
                .parse()
                .context("invalid default frontend bind address")?,
        )?,
        dev: env::args().any(|arg| arg == "--dev"),
        api_proxy_target: env::var("API_PROXY_TARGET")
            .unwrap_or_else(|_| "http://localhost:8080".to_string()),
        static_dir: env_or(
            "FRONTEND_STATIC_DIR",
            PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/static")),
        )?,
    })
}
