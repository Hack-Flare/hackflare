use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct Config {
    pub bind_addr: SocketAddr,
    pub dns_bind_addr: SocketAddr,
    pub database_url: Option<String>,
    pub email: EmailConfig,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct EmailConfig {
    pub from_name: String,
    pub from_address: String,
    pub reply_to: Option<String>,
    pub smtp_host: Option<String>,
    pub smtp_port: u16,
    pub smtp_username: Option<String>,
    pub smtp_password: Option<String>,
    pub smtp_starttls: bool,
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

        let email = EmailConfig::from_env();

        Self {
            bind_addr: SocketAddr::new(http_host, http_port),
            dns_bind_addr: SocketAddr::new(dns_host, dns_port),
            database_url,
            email,
        }
    }
}

impl EmailConfig {
    pub fn from_env() -> Self {
        let from_name = env::var("BACKEND_EMAIL_FROM_NAME")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "Hackflare".to_string());

        let from_address = env::var("BACKEND_EMAIL_FROM_ADDRESS")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "no-reply@hackflare.local".to_string());

        let reply_to = env::var("BACKEND_EMAIL_REPLY_TO")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

        let smtp_host = env::var("BACKEND_EMAIL_SMTP_HOST")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

        let smtp_port = env::var("BACKEND_EMAIL_SMTP_PORT")
            .ok()
            .and_then(|value| value.parse::<u16>().ok())
            .unwrap_or(587);

        let smtp_username = env::var("BACKEND_EMAIL_SMTP_USERNAME")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

        let smtp_password = env::var("BACKEND_EMAIL_SMTP_PASSWORD")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

        let smtp_starttls = env::var("BACKEND_EMAIL_SMTP_STARTTLS")
            .ok()
            .map(|value| matches!(value.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
            .unwrap_or(true);

        Self {
            from_name,
            from_address,
            reply_to,
            smtp_host,
            smtp_port,
            smtp_username,
            smtp_password,
            smtp_starttls,
        }
    }
}
