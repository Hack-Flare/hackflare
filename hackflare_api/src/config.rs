use std::{
    env::{self, VarError},
    net::SocketAddr,
    str::FromStr,
};

use anyhow::{Context, Result};
use derive_more::{Display, Error};
use jsonwebtoken::{DecodingKey, EncodingKey};
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
    match env::var(name) {
        // if we have a value, try parsing it
        Ok(s) => s
            .trim()
            .parse::<T>()
            .with_context(|| format!("{name} has an invalid value for type {type_name}")),
        // if we do not have a value, fall back to `default`
        Err(VarError::NotPresent) => Ok(default),
        Err(VarError::NotUnicode(s)) => {
            anyhow::bail!("{name} contains invalid unicode: {}", s.display())
        }
    }
}

#[derive(Debug, Display, Error)]
#[display(
    "invalid environment: `{}`, expected `development` or `production`",
    _0
)]
pub struct EnvironmentParseError(#[error(not(source))] String);

#[derive(Debug, PartialEq, Eq, Display)]
pub(crate) enum Environment {
    Production,
    Development,
}

impl Environment {
    pub(crate) const fn is_prod(&self) -> bool {
        matches!(self, Environment::Production)
    }

    pub(crate) const fn is_dev(&self) -> bool {
        matches!(self, Environment::Development)
    }
}

impl FromStr for Environment {
    type Err = EnvironmentParseError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.as_ref() {
            "production" => Ok(Self::Production),
            "development" => Ok(Self::Development),
            _ => Err(EnvironmentParseError(s.to_string())),
        }
    }
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
pub struct Config {
    pub bind_addr: SocketAddr,
    pub(crate) environment: Environment,
    pub(crate) database_url: Url,
    pub(crate) auto_migrate: bool,
    pub(crate) jwt_encoding_key: EncodingKey,
    pub(crate) jwt_decoding_key: DecodingKey,
    pub(crate) hca: HcaConfig,
}

impl Config {}

pub fn from_env() -> Result<Config> {
    let redirect_uri: Url = env_req("API_HCA_REDIRECT_URI")?;
    let database_url: Url = env_req("DATABASE_URL")?;

    match database_url.scheme() {
        "postgres" => { /* valid */ }
        other => anyhow::bail!("DATABASE_URL must use postgres (found {})", other),
    }

    // Validate the scheme
    match redirect_uri.scheme() {
        "http" | "https" => { /* valid */ }
        other => anyhow::bail!(
            "API_HCA_REDIRECT_URI must use http or https (found {})",
            other
        ),
    }

    let environment = env_or("API_ENVIRONMENT", Environment::Production)?;

    if environment == Environment::Production && redirect_uri.scheme() != "https" {
        warn!("running in production but redirect URI is not HTTPS");
    }

    let auto_migrate = env_or("API_AUTO_MIGRATE", environment.is_prod())?;

    let jwt_secret = env_req::<String>("API_JWT_SECRET")?;

    Ok(Config {
        bind_addr: env_or("API_BIND_ADDR", "0.0.0.0:8080".parse().unwrap())?,
        database_url,
        auto_migrate,
        environment,
        jwt_encoding_key: EncodingKey::from_base64_secret(&jwt_secret)?,
        jwt_decoding_key: DecodingKey::from_base64_secret(&jwt_secret)?,
        hca: HcaConfig {
            client_id: env_req("API_HCA_CLIENT_ID")?,
            client_secret: env_req("API_HCA_CLIENT_SECRET")?,
            redirect_uri,
        },
    })
}
