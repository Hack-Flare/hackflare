mod config;
mod dns;
mod monitor;
mod report;
mod server;
mod simulator;
pub mod summary;

pub use config::Config;
pub use dns::DnsResult;
pub use monitor::{ProcessMonitor, ResourceSample};
pub use report::{Report, write};
pub use simulator::{LatencyStats, RunResult, simulate};

use std::net::UdpSocket;
use std::time::{Duration, Instant};

use crate::dns::{build_query, parse_response};

/// Wait until the self-hosted DNS server answers a query on the given port.
async fn wait_for_dns(addr: &str) -> anyhow::Result<()> {
    const TIMEOUT: Duration = Duration::from_secs(60);
    const POLL: Duration = Duration::from_millis(250);
    const QNAME: &str = "hackflare.net";
    const QTYPE: u16 = 1;

    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect(addr)?;
    socket.set_read_timeout(Some(Duration::from_secs(3)))?;
    let query = build_query(1, QNAME, QTYPE);
    let start = Instant::now();
    while start.elapsed() < TIMEOUT {
        let mut buf = [0u8; 512];
        match socket.send(&query).and_then(|_| socket.recv(&mut buf)) {
            Ok(len) => {
                if parse_response(&buf[..len]).is_ok() {
                    return Ok(());
                }
            }
            Err(_) => {
                tokio::time::sleep(POLL).await;
                continue;
            }
        }
        tokio::time::sleep(POLL).await;
    }
    anyhow::bail!("DNS server at {addr} did not become ready")
}

/// Run a complete benchmark. Phases run one at a time so each gets its own CPU/RAM sample set:
///
/// 1. HTTP health load test (self-hosted app, CPU/RAM monitored)
/// 2. DNS query load test (CPU/RAM monitored) when `--dns` is enabled
pub async fn run(config: Config) -> anyhow::Result<Report> {
    let (target, dns_addr, server) = if let Some(binary) = &config.binary {
        let server = server::Server::spawn(binary, config.port, config.dev)?;
        server.wait_for_health().await?;
        let target = format!("http://127.0.0.1:{}/api/v1/health", config.port);
        let dns_addr = format!("127.0.0.1:{}", config.dns_port);
        if config.dns {
            wait_for_dns(&dns_addr).await?;
        }
        (target, Some(dns_addr), Some(server))
    } else {
        let dns_addr = if config.dns {
            Some(config.dns_server.clone())
        } else {
            None
        };
        (config.target.clone(), dns_addr, None)
    };

    // HTTP load test with dedicated CPU/RAM sampling
    let mut http_monitor = None;
    if let Some(ref s) = server {
        let mut m = ProcessMonitor::new(s.child_id(), config.sample_interval());
        m.start()?;
        http_monitor = Some(m);
    }

    let mut result = simulate(&target, &config).await;

    let http_resources = if let Some(m) = &mut http_monitor {
        m.stop()
    } else {
        Vec::new()
    };
    result.http_resources = http_resources;

    // DNS load test with dedicated CPU/RAM sampling
    let dns_result = if config.dns {
        let addr = dns_addr.expect("dns_addr set when dns enabled");
        let mut dns_monitor = None;
        if let Some(ref s) = server {
            let mut m = ProcessMonitor::new(s.child_id(), config.sample_interval());
            m.start()?;
            dns_monitor = Some(m);
        }

        let dns_result = dns::simulate_dns(&addr, &config).await;

        let dns_resources = if let Some(m) = &mut dns_monitor {
            m.stop()
        } else {
            Vec::new()
        };

        Some((dns_result, dns_resources))
    } else {
        None
    };

    if let Some((dns_result, dns_resources)) = dns_result {
        result.dns = Some(dns_result);
        result.dns_resources = dns_resources;
    }

    if let Some(mut s) = server {
        s.shutdown().await?;
    }

    let report = write(
        &config.output_dir,
        &result,
        &result.http_resources,
        &result.dns_resources,
    )?;
    Ok(report)
}
