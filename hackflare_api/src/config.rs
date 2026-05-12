use std::{env, net::SocketAddr, str::FromStr};

use anyhow::{Context, Result};
use jsonwebtoken::EncodingKey;
use reqwest::Url;

fn env_req<T>(name: &str) -> Result<T>
where
    T: FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    let type_name = std::any::type_name::<T>();
    env::var(name)
        .with_context(|| format!("{name} is required"))?
        .trim()
        .parse::<T>()
        .with_context(|| format!("{name} has an invalid value for type {type_name}"))
}

fn env_or<T>(name: &str, default: T) -> Result<T>
where
    T: FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    let type_name = std::any::type_name::<T>();
    env::var(name)
        .ok()
        // if we have a value, try parsing it
        .map(|s| {
            s.trim()
                .parse::<T>()
                .with_context(|| format!("{name} has an invalid value for type {type_name}"))
        })
        // if we do not have a value, fall back to `default`
        .unwrap_or(Ok(default))
}

#[derive(Debug)]
pub(crate) struct HcaConfig {
    pub(crate) client_id: String,
    pub(crate) client_secret: String,
    /// May be "http://" or "https://".
    pub(crate) redirect_uri: Url,
}

impl HcaConfig {
    pub(crate) fn is_secure(&self) -> bool {
        self.redirect_uri.scheme() == "https"
    }
}

#[derive(Debug)]
pub(crate) struct Config {
    pub(crate) bind_addr: SocketAddr,
    pub(crate) jwt_secret: EncodingKey,
    pub(crate) hca: HcaConfig,
}

pub(crate) fn from_env() -> Result<Config> {
    let redirect_uri: Url = env_req("API_HCA_REDIRECT_URI")?;

    // Validate the scheme
    match redirect_uri.scheme() {
        "http" | "https" => { /* valid */ }
        _ => anyhow::bail!(
            "API_HCA_REDIRECT_URI must use http or https (found {})",
            redirect_uri.scheme()
        ),
    }

    Ok(Config {
        bind_addr: env_or("API_BIND_ADDR", "0.0.0.0:8080".parse().unwrap())?,
        jwt_secret: env_req::<String>("API_JWT_SECRET")
            .map(|s| EncodingKey::from_secret(s.as_bytes()))?,
        hca: HcaConfig {
            client_id: env_req("API_HCA_CLIENT_ID")?,
            client_secret: env_req("API_HCA_CLIENT_SECRET")?,
            redirect_uri,
        },
    })
}
