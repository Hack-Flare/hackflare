use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[derive(Clone, Debug)]
pub struct Config {
    pub bind_addr: SocketAddr,
    pub gateway_internal_token: Option<String>,
    pub database_url: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        let host = env::var("BACKEND_BIND_HOST")
            .ok()
            .and_then(|value| value.parse::<IpAddr>().ok())
            .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));

        let port = env::var("BACKEND_BIND_PORT")
            .ok()
            .and_then(|value| value.parse::<u16>().ok())
            .unwrap_or(8080);

        let gateway_internal_token = env::var("BACKEND_GATEWAY_TOKEN")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

        let database_url = env::var("DATABASE_URL")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

        Self {
            bind_addr: SocketAddr::new(host, port),
            gateway_internal_token,
            database_url,
        }
    }
}
