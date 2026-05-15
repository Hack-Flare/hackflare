use std::env;
use std::path::PathBuf;
use std::time::Duration;

/// Configuration for the DNS engine and recursive resolver.
/// All environment variables are loaded once at startup with sensible defaults.
#[derive(Debug, Clone)]
pub struct DnsConfig {
    /// Enable recursive resolution (default: false)
    pub recursion_enabled: bool,

    /// SOA record fields (defaults suitable for a secondary nameserver)
    /// SOA MNAME (primary nameserver) — default: `"ns.example.com."`
    pub soa_mname: String,
    /// SOA RNAME (responsible person) — default: `"admin.example.com."`
    pub soa_rname: String,
    /// SOA serial number — default: `"2024010101"`
    pub soa_serial: String,
    /// SOA refresh interval (seconds) — default: `"3600"`
    pub soa_refresh: String,
    /// SOA retry interval (seconds) — default: `"1800"`
    pub soa_retry: String,
    /// SOA expire interval (seconds) — default: `"604800"`
    pub soa_expire: String,
    /// SOA minimum TTL (seconds) — default: `"86400"`
    pub soa_minimum: String,
    /// SOA record TTL (seconds) — default: `"3600"`
    pub soa_ttl: String,

    /// UDP/Recursive resolver settings
    /// UDP payload size (EDNS) — default: 512
    pub udp_size: u16,
    /// UDP attempts per upstream server — default: 4
    pub udp_attempts: usize,
    /// UDP timeout per attempt — default: 2500 ms
    pub udp_timeout: Duration,
    /// Maximum recursion rounds — default: 8
    pub recursion_rounds: usize,
    /// Enable debug logging for recursive resolver — default: false
    pub recursion_debug: bool,
    /// Path to root hints file (optional, uses hardcoded defaults if not set)
    pub root_hints_file: Option<PathBuf>,
    /// `PostgreSQL` database URL for loading root hints from DB (optional)
    pub database_url: Option<String>,
}

impl DnsConfig {
    /// Load configuration from environment variables with sensible defaults.
    pub fn from_env() -> Self {
        Self {
            recursion_enabled: env_bool("HACKFLARE_DNS_RECURSION_ENABLED", false),
            soa_mname: env_string("HACKFLARE_DNS_SOA_MNAME", "ns.example.com."),
            soa_rname: env_string("HACKFLARE_DNS_SOA_RNAME", "admin.example.com."),
            soa_serial: env_string("HACKFLARE_DNS_SOA_SERIAL", "2024010101"),
            soa_refresh: env_string("HACKFLARE_DNS_SOA_REFRESH", "3600"),
            soa_retry: env_string("HACKFLARE_DNS_SOA_RETRY", "1800"),
            soa_expire: env_string("HACKFLARE_DNS_SOA_EXPIRE", "604800"),
            soa_minimum: env_string("HACKFLARE_DNS_SOA_MINIMUM", "86400"),
            soa_ttl: env_string("HACKFLARE_DNS_SOA_TTL", "3600"),
            udp_size: env_u16("HACKFLARE_DNS_UDP_SIZE", 512),
            udp_attempts: env_usize("HACKFLARE_DNS_UDP_ATTEMPTS", 4).max(1),
            udp_timeout: Duration::from_millis(env_u64("HACKFLARE_DNS_UDP_TIMEOUT_MS", 2500)),
            recursion_rounds: env_usize("HACKFLARE_DNS_RECURSION_ROUNDS", 8).max(1),
            recursion_debug: env_bool("HACKFLARE_DNS_RECURSION_DEBUG", false),
            root_hints_file: env::var("HACKFLARE_ROOT_HINTS_FILE")
                .ok()
                .map(PathBuf::from),
            database_url: env::var("DATABASE_URL").ok(),
        }
    }

    /// Create a new config with all defaults (useful for testing).
    #[must_use]
    pub fn default_config() -> Self {
        Self {
            recursion_enabled: false,
            soa_mname: "ns.example.com.".to_string(),
            soa_rname: "admin.example.com.".to_string(),
            soa_serial: "2024010101".to_string(),
            soa_refresh: "3600".to_string(),
            soa_retry: "1800".to_string(),
            soa_expire: "604800".to_string(),
            soa_minimum: "86400".to_string(),
            soa_ttl: "3600".to_string(),
            udp_size: 512,
            udp_attempts: 4,
            udp_timeout: Duration::from_millis(2500),
            recursion_rounds: 8,
            recursion_debug: false,
            root_hints_file: None,
            database_url: None,
        }
    }
}

// Helper functions for environment variable parsing
fn env_bool(name: &str, default: bool) -> bool {
    env::var(name).ok().map_or(default, |v| {
        let v = v.trim().to_ascii_lowercase();
        v == "1" || v == "true" || v == "yes" || v == "on"
    })
}

fn env_string(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_string())
}

fn env_u16(name: &str, default: u16) -> u16 {
    env::var(name)
        .ok()
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(default)
}

fn env_u64(name: &str, default: u64) -> u64 {
    env::var(name)
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(default)
}

fn env_usize(name: &str, default: usize) -> usize {
    env::var(name)
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_sensible_defaults() {
        let cfg = DnsConfig::default_config();
        assert!(!cfg.recursion_enabled);
        assert_eq!(cfg.soa_mname, "ns.example.com.");
        assert_eq!(cfg.udp_size, 512);
        assert_eq!(cfg.udp_attempts, 4);
        assert_eq!(cfg.recursion_rounds, 8);
        assert!(!cfg.recursion_debug);
        assert_eq!(cfg.udp_timeout, Duration::from_millis(2500));
    }

    #[test]
    fn env_bool_parses_correctly() {
        assert!(env_bool("NONEXISTENT_VAR_12345", true));
        assert!(!env_bool("NONEXISTENT_VAR_12345", false));
    }
}
