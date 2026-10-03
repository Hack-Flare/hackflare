use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub environment: String,
    pub region: String,
    pub mode: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InterfaceConfig {
    pub wan: Option<String>,
    pub lan: Option<String>,
    pub loopback: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DefaultPolicyConfig {
    pub input: String,
    pub forward: String,
    pub output: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NatConfig {
    pub enabled: bool,
    pub masquerade: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitConfig {
    pub enabled: bool,
    pub requests_per_second: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FirewallConfigSection {
    pub name: String,
    pub interfaces: InterfaceConfig,
    pub default_policy: DefaultPolicyConfig,
    pub nat: NatConfig,
    pub logging: LoggingConfig,
    pub allowlist: Vec<String>,
    pub blacklist: Vec<String>,
    pub blocked_ports: Vec<u16>,
    pub rate_limit: RateLimitConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DnsConfigSection {
    pub enabled: bool,
    pub resolver: String,
    pub sinkhole_ip: String,
    pub blocked_domains: Vec<String>,
    pub allowed_tlds: Vec<String>,
    pub log_queries: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub project: ProjectConfig,
    pub firewall: FirewallConfigSection,
    pub dns: DnsConfigSection,
}

pub fn load_config_from_path<P: AsRef<Path>>(path: P) -> anyhow::Result<AppConfig> {
    let content = fs::read_to_string(path)?;
    let config: AppConfig = serde_yaml::from_str(&content)?;
    Ok(config)
}

pub fn load_config() -> anyhow::Result<AppConfig> {
    let candidates = [
        Path::new("config.yaml"),
        Path::new("SRC/config.yaml"),
        Path::new("./config.yaml"),
    ];

    for path in &candidates {
        if path.exists() {
            return load_config_from_path(path);
        }
    }

    anyhow::bail!("No configuration file found. Expected config.yaml or SRC/config.yaml.")
}
