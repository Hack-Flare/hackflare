use std::path::PathBuf;
use std::time::Duration;

use clap::Parser;

/// Hackflare load benchmarking tool.
#[derive(Debug, Clone, Parser)]
#[command(name = "hackflare-bench", about = "Hackflare load benchmarker")]
pub struct Config {
    /// Target URL to benchmark (e.g. http://127.0.0.1:18080/api/v1/health)
    #[arg(long, default_value = "http://127.0.0.1:18080/api/v1/health")]
    pub target: String,

    /// Number of concurrent simulated HTTP users
    #[arg(long, default_value = "100")]
    pub users: usize,

    /// Test duration in seconds
    #[arg(long, default_value = "60")]
    pub duration_secs: u64,

    /// Ramp-up time in seconds (users are started spread across this window)
    #[arg(long, default_value = "10")]
    pub ramp_up_secs: u64,

    /// Output directory for the JSON and HTML reports
    #[arg(long, default_value = "./bench-results")]
    pub output_dir: PathBuf,

    /// Path to a pre-built hackflare-app binary to self-host and monitor.
    /// When set, the binary is launched in dev mode (embedded PostgreSQL) and its CPU/RAM is sampled during the test.
    #[arg(long)]
    pub binary: Option<PathBuf>,

    /// Port the self-hosted binary binds to (HTTP)
    #[arg(long, default_value = "18080")]
    pub port: u16,

    /// Run the self-hosted binary in dev mode (embedded PostgreSQL)
    #[arg(long, default_value = "false")]
    pub dev: bool,

    /// CPU/RAM sample interval in milliseconds
    #[arg(long, default_value = "250")]
    pub sample_interval_ms: u64,

    /// HTTP request timeout in seconds
    #[arg(long, default_value = "10")]
    pub timeout_secs: u64,

    // DNS benchmarking

    /// Also run a DNS query benchmark against a DNS server.
    #[arg(long, default_value = "false")]
    pub dns: bool,

    /// DNS server address to query (e.g. 127.0.0.1:5454).
    /// Defaults to the dev-mode DNS port when self-hosting.
    #[arg(long, default_value = "127.0.0.1:5454")]
    pub dns_server: String,

    /// Number of concurrent simulated DNS clients
    #[arg(long, default_value = "100")]
    pub dns_users: usize,

    /// DNS test duration in seconds
    #[arg(long, default_value = "60")]
    pub dns_duration_secs: u64,

    /// DNS ramp-up time in seconds
    #[arg(long, default_value = "5")]
    pub dns_ramp_up_secs: u64,

    /// Query name expected to resolve quickly (recursive/cached answer).
    #[arg(long, default_value = "hackflare.net")]
    pub dns_qname_hit: String,

    /// Query name expected to be a cache miss / NXDOMAIN
    #[arg(long, default_value = "nonexistent.hackflare.net")]
    pub dns_qname_miss: String,

    /// DNS query type (1 = A, 2 = NS, 6 = SOA, 28 = AAAA)
    #[arg(long, default_value = "1")]
    pub dns_qtype: u16,

    /// Fraction of DNS queries sent to the cache-hit name (0.0 - 1.0)
    #[arg(long, default_value = "0.5")]
    pub dns_hit_ratio: f64,

    /// DNS request timeout in seconds
    #[arg(long, default_value = "5")]
    pub dns_timeout_secs: u64,

    /// DNS server port to use when self-hosting
    #[arg(long, default_value = "5454")]
    pub dns_port: u16,
}

impl Config {
    pub fn duration(&self) -> Duration {
        Duration::from_secs(self.duration_secs)
    }

    pub fn sample_interval(&self) -> Duration {
        Duration::from_millis(self.sample_interval_ms)
    }

    pub fn request_timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_secs)
    }

    pub fn dns_duration(&self) -> Duration {
        Duration::from_secs(self.dns_duration_secs)
    }

    pub fn dns_timeout(&self) -> Duration {
        Duration::from_secs(self.dns_timeout_secs)
    }
}
