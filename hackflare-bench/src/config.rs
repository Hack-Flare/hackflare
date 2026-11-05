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

    /// Number of concurrent simulated users
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
    #[arg(long)]
    pub binary: Option<PathBuf>,

    /// Port the self-hosted binary binds to
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
}
