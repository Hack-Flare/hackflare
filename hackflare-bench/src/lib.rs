mod config;
mod monitor;
mod report;
mod server;
mod simulator;

pub use config::Config;
pub use monitor::{ProcessMonitor, ResourceSample};
pub use report::{Report, write};
pub use simulator::{LatencyStats, RunResult, simulate};

/// Run a complete benchmark
pub async fn run(config: Config) -> anyhow::Result<Report> {
    let (target, server) = if let Some(binary) = &config.binary {
        let server = server::Server::spawn(binary, config.port, config.dev)?;
        server.wait_for_health().await?;
        let target = format!("http://127.0.0.1:{}/api/v1/health", config.port);
        (target, Some(server))
    } else {
        (config.target.clone(), None)
    };

    // Start sampling the server process CPU/RAM when self-hosting.
    let mut monitor = None;
    if let Some(ref s) = server {
        let mut m = ProcessMonitor::new(s.child_id(), config.sample_interval());
        m.start()?;
        monitor = Some(m);
    }

    let result = simulate(&target, &config).await;

    let resources = if let Some(m) = &mut monitor {
        m.stop().await
    } else {
        Vec::new()
    };

    if let Some(mut s) = server {
        s.shutdown().await?;
    }

    let report = write(&config.output_dir, &result, &resources)?;
    Ok(report)
}
