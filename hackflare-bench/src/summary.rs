use std::fs;
use std::path::PathBuf;

use crate::monitor::ResourceSample;
use crate::simulator::RunResult;

/// Read each directory's `results.json`, then write a combined markdown comparison summary to `output`.
pub fn write_summary(dirs: &[PathBuf], output: PathBuf) -> anyhow::Result<()> {
    let mut results: Vec<(PathBuf, RunResult)> = Vec::new();
    for dir in dirs {
        let json_path = dir.join("results.json");
        let json = fs::read_to_string(&json_path)
            .map_err(|e| anyhow::anyhow!("failed to read {json_path:?}: {e}"))?;
        let result: RunResult = serde_json::from_str(&json)
            .map_err(|e| anyhow::anyhow!("failed to parse {json_path:?}: {e}"))?;
        results.push((dir.clone(), result));
    }

    let mut md = String::new();
    md.push_str("# Hackflare Benchmark Summary\n\n");
    md.push_str("Comparison across benchmark scales.\n\n");

    md.push_str("## HTTP Results\n\n");
    md.push_str(
        "| Scale | Users | Total Req | Successful | Failed | Err % | Req/s | CPU % | RAM MiB |\n",
    );
    md.push_str(
        "|-------|-------|-----------|------------|--------|-------|-------|-------|---------|\n",
    );

    for (dir, r) in &results {
        let scale = dir
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        let error_rate = if r.total_requests > 0 {
            r.failed_requests as f64 / r.total_requests as f64 * 100.0
        } else {
            0.0
        };
        let http_max_cpu = max_cpu(&r.http_resources);
        let http_max_ram = max_ram(&r.http_resources);

        md.push_str(&format!(
            "| {} | {} | {} | {} | {} | {:.2}% | {:.2} | {:.2} | {:.2} |\n",
            scale,
            r.users,
            r.total_requests,
            r.successful_requests,
            r.failed_requests,
            error_rate,
            r.requests_per_second,
            http_max_cpu,
            http_max_ram,
        ));
    }

    md.push_str("\n## HTTP Latency\n\n");
    md.push_str("| Scale | p50 ms | p95 ms | p99 ms |\n");
    md.push_str("|-------|--------|--------|--------|\n");

    for (dir, r) in &results {
        let scale = dir
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        md.push_str(&format!(
            "| {} | {:.2} | {:.2} | {:.2} |\n",
            scale, r.latency.p50_ms, r.latency.p95_ms, r.latency.p99_ms,
        ));
    }

    md.push_str("\n## DNS Results\n\n");
    md.push_str(
        "| Scale | DNS Users | QPS | Hits | Misses | Failures | Hit Ratio | CPU % | RAM MiB |\n",
    );
    md.push_str(
        "|-------|-----------|-----|------|--------|----------|-----------|-------|---------|\n",
    );

    for (dir, r) in &results {
        let scale = dir
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        let dns_qps = r.dns.as_ref().map(|d| d.requests_per_second).unwrap_or(0.0);
        let dns_hits = r.dns.as_ref().map(|d| d.hits).unwrap_or(0);
        let dns_misses = r.dns.as_ref().map(|d| d.misses).unwrap_or(0);
        let dns_failures = r.dns.as_ref().map(|d| d.failures).unwrap_or(0);
        let dns_hit_ratio = r.dns.as_ref().map(|d| d.hit_ratio).unwrap_or(0.0);
        let dns_max_cpu = r
            .dns
            .as_ref()
            .map(|_| max_cpu(&r.dns_resources))
            .unwrap_or(0.0);
        let dns_max_ram = r
            .dns
            .as_ref()
            .map(|_| max_ram(&r.dns_resources))
            .unwrap_or(0.0);
        let dns_users = r.dns.as_ref().map(|d| d.users).unwrap_or(0);

        md.push_str(&format!(
            "| {} | {} | {:.2} | {} | {} | {} | {:.2} | {:.2} | {:.2} |\n",
            scale,
            dns_users,
            dns_qps,
            dns_hits,
            dns_misses,
            dns_failures,
            dns_hit_ratio,
            dns_max_cpu,
            dns_max_ram,
        ));
    }

    md.push_str("\n## Per-Scale Details\n\n");
    for (dir, r) in &results {
        let scale = dir
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        md.push_str(&format!("### {scale}\n\n"));
        md.push_str(&format!("- **Target**: {}\n", r.target));
        md.push_str(&format!("- **Users**: {}\n", r.users));
        md.push_str(&format!(
            "- **Duration**: {}s (ramp-up {}s)\n",
            r.duration_secs, r.ramp_up_secs
        ));
        md.push_str(&format!(
            "- **Total requests**: {} ({} successful, {} failed)\n",
            r.total_requests, r.successful_requests, r.failed_requests
        ));
        md.push_str(&format!(
            "- **Requests / sec**: {:.2}\n",
            r.requests_per_second
        ));
        md.push_str(&format!(
            "- **Latency**: p50={:.2}ms, p95={:.2}ms, p99={:.2}ms\n",
            r.latency.p50_ms, r.latency.p95_ms, r.latency.p99_ms
        ));
        if !r.http_resources.is_empty() {
            md.push_str(&format!(
                "- **HTTP CPU**: avg={:.2}%, max={:.2}%\n",
                avg_cpu(&r.http_resources),
                max_cpu(&r.http_resources)
            ));
            md.push_str(&format!(
                "- **HTTP RAM**: avg={:.2} MiB, max={:.2} MiB\n",
                avg_ram(&r.http_resources),
                max_ram(&r.http_resources)
            ));
        }
        if let Some(dns) = &r.dns {
            md.push_str(&format!(
                "- **DNS queries**: {} ({} hits, {} misses, {} failures)\n",
                dns.total, dns.hits, dns.misses, dns.failures
            ));
            md.push_str(&format!("- **DNS QPS**: {:.2}\n", dns.requests_per_second));
            md.push_str(&format!("- **DNS hit ratio**: {:.2}\n", dns.hit_ratio));
            md.push_str(&format!(
                "- **DNS latency**: p50={:.2}ms, p95={:.2}ms, p99={:.2}ms\n",
                dns.latency.p50_ms, dns.latency.p95_ms, dns.latency.p99_ms
            ));
            if !r.dns_resources.is_empty() {
                md.push_str(&format!(
                    "- **DNS CPU**: avg={:.2}%, max={:.2}%\n",
                    avg_cpu(&r.dns_resources),
                    max_cpu(&r.dns_resources)
                ));
                md.push_str(&format!(
                    "- **DNS RAM**: avg={:.2} MiB, max={:.2} MiB\n",
                    avg_ram(&r.dns_resources),
                    max_ram(&r.dns_resources)
                ));
            }
        }
        md.push('\n');
    }

    fs::write(&output, md)?;
    Ok(())
}

fn avg_cpu(samples: &[ResourceSample]) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    samples.iter().map(|s| s.cpu_percent).sum::<f64>() / samples.len() as f64
}

fn max_cpu(samples: &[ResourceSample]) -> f64 {
    samples.iter().map(|s| s.cpu_percent).fold(0.0f64, f64::max)
}

fn avg_ram(samples: &[ResourceSample]) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    samples.iter().map(|s| s.ram_mb).sum::<f64>() / samples.len() as f64
}

fn max_ram(samples: &[ResourceSample]) -> f64 {
    samples.iter().map(|s| s.ram_mb).fold(0.0f64, f64::max)
}
