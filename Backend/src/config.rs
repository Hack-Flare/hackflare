use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[derive(Clone, Debug)]
pub struct Config {
    pub bind_addr: SocketAddr,
    pub dns_bind_addr: SocketAddr,
    pub database_url: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        let http_host = env::var("BACKEND_BIND_HOST")
            .ok()
            .and_then(|value| value.parse::<IpAddr>().ok())
            .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));

        let http_port = env::var("BACKEND_BIND_PORT")
            .ok()
            .and_then(|value| value.parse::<u16>().ok())
            .unwrap_or(8080);

        let dns_host = env::var("BACKEND_DNS_BIND_HOST")
            .ok()
            .and_then(|value| value.parse::<IpAddr>().ok())
            .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));

        let dns_port = env::var("BACKEND_DNS_BIND_PORT")
            .ok()
            .and_then(|value| value.parse::<u16>().ok())
            .unwrap_or(5353);

        let database_url = env::var("DATABASE_URL")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

        Self {
            bind_addr: SocketAddr::new(http_host, http_port),
            dns_bind_addr: SocketAddr::new(dns_host, dns_port),
            database_url,
        }
    }
}
