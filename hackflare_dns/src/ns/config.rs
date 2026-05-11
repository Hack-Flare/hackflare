use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NsConfig {
    pub bind_addr: String,
    pub port: u16,
    pub zone_file: Option<String>,
    pub database_url: Option<String>,
}

impl NsConfig {
    /// Load configuration from environment variables and defaults.
    pub fn from_env() -> Self {
        Self {
            bind_addr: env::var("HACKFLARE_NS_BIND_ADDR").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("HACKFLARE_NS_PORT")
                .ok()
                .and_then(|v| v.parse::<u16>().ok())
                .unwrap_or(53),
            zone_file: env::var("HACKFLARE_ZONE_FILE").ok(),
            database_url: env::var("DATABASE_URL").ok(),
        }
    }
}

impl Default for NsConfig {
    fn default() -> Self {
        Self {
            bind_addr: "0.0.0.0".to_string(),
            port: 53,
            zone_file: None,
            database_url: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_matches_nameserver_defaults() {
        let config = NsConfig::default();
        assert_eq!(config.bind_addr, "0.0.0.0");
        assert_eq!(config.port, 53);
        assert_eq!(config.zone_file, None);
        assert_eq!(config.database_url, None);
    }
}
