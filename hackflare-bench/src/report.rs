use std::path::{Path, PathBuf};

use plotters::prelude::*;
use serde::Serialize;

use crate::monitor::ResourceSample;
use crate::simulator::RunResult;

#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub dir: PathBuf,
    pub json_path: PathBuf,
    pub html_path: PathBuf,
}

/// Expand resource samples into chart points (t, value).
fn to_points(samples: &[ResourceSample], f: fn(&ResourceSample) -> f64) -> Vec<(f64, f64)> {
    samples.iter().map(|s| (s.t, f(s))).collect()
}

fn write_line_chart(
    path: &Path,
    title: &str,
    _y_label: &str,
    points: &[(f64, f64)],
) -> anyhow::Result<()> {
    let root = SVGBackend::new(path, (800, 400)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_x = points
        .iter()
        .map(|(x, _)| *x)
        .fold(0.0f64, f64::max)
        .max(1.0);
    let max_y = points
        .iter()
        .map(|(_, y)| *y)
        .fold(0.0f64, f64::max)
        .max(1.0);

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 20))
        .margin(10)
        .set_label_area_size(LabelAreaPosition::Left, 60)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .build_cartesian_2d(0.0..max_x, 0.0..max_y * 1.1 + 1.0)?;

    chart.configure_mesh().draw()?;

    let style = RED;
    chart.draw_series(LineSeries::new(points.to_vec(), style))?;
    root.present()?;
    Ok(())
}

fn write_report(
    dir: &Path,
    result: &RunResult,
    http_resources: &[ResourceSample],
    dns_resources: &[ResourceSample],
) -> anyhow::Result<Report> {
    std::fs::create_dir_all(dir)?;

    let json_path = dir.join("results.json");
    let json = serde_json::to_string_pretty(result)?;
    std::fs::write(&json_path, json)?;

    let http_cpu_points = to_points(http_resources, |s| s.cpu_percent);
    let http_ram_points = to_points(http_resources, |s| s.ram_mb);
    let latency_points: Vec<(f64, f64)> = result.latency_over_time.clone();

    let http_cpu_path = dir.join("http-cpu.svg");
    let http_ram_path = dir.join("http-ram.svg");
    let throughput_path = dir.join("throughput.svg");
    let latency_path = dir.join("latency.svg");

    write_line_chart(
        &http_cpu_path,
        "HTTP Phase - CPU Usage (%)",
        "%",
        &http_cpu_points,
    )?;
    write_line_chart(
        &http_ram_path,
        "HTTP Phase - RAM Usage (MiB)",
        "MiB",
        &http_ram_points,
    )?;
    write_line_chart(
        &throughput_path,
        "HTTP Throughput (requests / second)",
        "req/s",
        &result.throughput,
    )?;
    write_line_chart(
        &latency_path,
        "HTTP Request Latency (ms)",
        "ms",
        &latency_points,
    )?;

    let mut dns_paths: Vec<(&str, PathBuf)> = Vec::new();
    if let Some(dns) = &result.dns {
        let dns_cpu_path = dir.join("dns-cpu.svg");
        let dns_ram_path = dir.join("dns-ram.svg");
        let dns_throughput_path = dir.join("dns-throughput.svg");
        let dns_hits_path = dir.join("dns-hits.svg");
        let dns_misses_path = dir.join("dns-misses.svg");
        let dns_latency_path = dir.join("dns-latency.svg");

        let dns_cpu_points = to_points(dns_resources, |s| s.cpu_percent);
        let dns_ram_points = to_points(dns_resources, |s| s.ram_mb);

        write_line_chart(
            &dns_cpu_path,
            "DNS Phase - CPU Usage (%)",
            "%",
            &dns_cpu_points,
        )?;
        write_line_chart(
            &dns_ram_path,
            "DNS Phase - RAM Usage (MiB)",
            "MiB",
            &dns_ram_points,
        )?;
        write_line_chart(
            &dns_throughput_path,
            "DNS Throughput (queries / second)",
            "qps",
            &dns.throughput,
        )?;
        write_line_chart(
            &dns_hits_path,
            "DNS Cache Hits / second",
            "hits/s",
            &dns.hits_over_time,
        )?;
        write_line_chart(
            &dns_misses_path,
            "DNS Cache Misses / second",
            "misses/s",
            &dns.misses_over_time,
        )?;
        write_line_chart(
            &dns_latency_path,
            "DNS Query Latency (ms)",
            "ms",
            &dns.latency_over_time,
        )?;

        dns_paths.push(("dns-cpu.svg", dns_cpu_path));
        dns_paths.push(("dns-ram.svg", dns_ram_path));
        dns_paths.push(("dns-throughput.svg", dns_throughput_path));
        dns_paths.push(("dns-hits.svg", dns_hits_path));
        dns_paths.push(("dns-misses.svg", dns_misses_path));
        dns_paths.push(("dns-latency.svg", dns_latency_path));
    }

    let html_path = dir.join("report.html");
    let html = render_html(
        result,
        http_resources,
        dns_resources,
        &http_cpu_path,
        &http_ram_path,
        &throughput_path,
        &latency_path,
        &dns_paths,
    );
    std::fs::write(&html_path, html)?;

    write_webserver_run_script(dir)?;

    Ok(Report {
        dir: dir.to_path_buf(),
        json_path,
        html_path,
    })
}

fn write_webserver_run_script(dir: &Path) -> anyhow::Result<()> {
    let script_path = dir.join("webserver-run.sh");
    let script = r#"#!/bin/bash
        # Run a simple webserver for the report.html file
        # Usage: ./webserver-run.sh
        set -euo pipefail
        cd "$(dirname "$0")"
        echo "Serving report.html at http://localhost:8000"
        python3 -m http.server 8000
"#
    .to_string();
    std::fs::write(&script_path, script)?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn render_html(
    result: &RunResult,
    http_resources: &[ResourceSample],
    dns_resources: &[ResourceSample],
    http_cpu_path: &Path,
    http_ram_path: &Path,
    throughput_path: &Path,
    latency_path: &Path,
    dns_paths: &[(&str, PathBuf)],
) -> String {
    let avg_cpu = if http_resources.is_empty() {
        0.0
    } else {
        http_resources.iter().map(|s| s.cpu_percent).sum::<f64>() / http_resources.len() as f64
    };
    let max_cpu = http_resources
        .iter()
        .map(|s| s.cpu_percent)
        .fold(0.0f64, f64::max);
    let avg_ram = if http_resources.is_empty() {
        0.0
    } else {
        http_resources.iter().map(|s| s.ram_mb).sum::<f64>() / http_resources.len() as f64
    };
    let max_ram = http_resources
        .iter()
        .map(|s| s.ram_mb)
        .fold(0.0f64, f64::max);

    let error_rate = if result.total_requests > 0 {
        result.failed_requests as f64 / result.total_requests as f64 * 100.0
    } else {
        0.0
    };

    let mut html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Hackflare Benchmark Report</title>
<style>
  * {{ box-sizing: border-box; }}
  body {{ font-family: ui-sans-serif, system-ui, -apple-system, Segoe UI, Roboto, sans-serif; margin: 24px; color: #1a1a1a; }}
  h1 {{ font-size: 24px; margin: 0 0 8px 0; }}
  h2 {{ font-size: 18px; margin: 24px 0 8px 0; }}
  .meta {{ color: #666; font-size: 14px; margin-bottom: 24px; }}
  .summary {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 16px; margin-bottom: 32px; }}
  .card {{ border: 1px solid #e5e5e5; border-radius: 8px; padding: 16px; background: #fafafa; }}
  .card .label {{ font-size: 12px; color: #666; text-transform: uppercase; letter-spacing: 0.05em; }}
  .card .value {{ font-size: 22px; font-weight: 700; margin-top: 4px; }}
  .chart {{ border: 1px solid #e5e5e5; border-radius: 8px; margin-bottom: 24px; }}
  .chart img {{ display: block; width: 100%; height: auto; }}
  .row {{ display: grid; grid-template-columns: 1fr 1fr; gap: 24px; }}
  @media (max-width: 768px) {{ .row {{ grid-template-columns: 1fr; }} }}
  .mono {{ font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: 13px; }}
</style>
</head>
<body>
<h1>Hackflare Benchmark Report</h1>
<div class="meta">
  Target: <span class="mono">{target}</span> &nbsp;|&nbsp;
  Users: {users} &nbsp;|&nbsp;
  Duration: {duration_secs}s &nbsp;|&nbsp;
  Ramp-up: {ramp_up_secs}s
</div>

<div class="summary">
  <div class="card"><div class="label">Total Requests</div><div class="value">{total_requests}</div></div>
  <div class="card"><div class="label">Successful</div><div class="value">{successful}</div></div>
  <div class="card"><div class="label">Failed</div><div class="value">{failed}</div></div>
  <div class="card"><div class="label">Error Rate</div><div class="value">{error_rate:.2}%</div></div>
  <div class="card"><div class="label">Requests / sec</div><div class="value">{rps:.2}</div></div>
  <div class="card"><div class="label">Avg CPU</div><div class="value">{avg_cpu:.2}%</div></div>
  <div class="card"><div class="label">Peak CPU</div><div class="value">{max_cpu:.2}%</div></div>
  <div class="card"><div class="label">Avg RAM</div><div class="value">{avg_ram:.2} MiB</div></div>
  <div class="card"><div class="label">Peak RAM</div><div class="value">{max_ram:.2} MiB</div></div>
  <div class="card"><div class="label">Latency p50</div><div class="value">{p50:.2} ms</div></div>
  <div class="card"><div class="label">Latency p95</div><div class="value">{p95:.2} ms</div></div>
  <div class="card"><div class="label">Latency p99</div><div class="value">{p99:.2} ms</div></div>
</div>

<div class="row">
  <div class="chart"><h2>HTTP Phase - CPU Usage</h2><img src="{http_cpu}"></div>
  <div class="chart"><h2>HTTP Phase - RAM Usage</h2><img src="{http_ram}"></div>
</div>
<div class="row">
  <div class="chart"><h2>HTTP Throughput</h2><img src="{throughput}"></div>
  <div class="chart"><h2>HTTP Latency Over Time</h2><img src="{latency}"></div>
</div>"#,
        target = result.target,
        users = result.users,
        duration_secs = result.duration_secs,
        ramp_up_secs = result.ramp_up_secs,
        total_requests = result.total_requests,
        successful = result.successful_requests,
        failed = result.failed_requests,
        error_rate = error_rate,
        rps = result.requests_per_second,
        avg_cpu = avg_cpu,
        max_cpu = max_cpu,
        avg_ram = avg_ram,
        max_ram = max_ram,
        p50 = result.latency.p50_ms,
        p95 = result.latency.p95_ms,
        p99 = result.latency.p99_ms,
        http_cpu = file_name(http_cpu_path),
        http_ram = file_name(http_ram_path),
        throughput = file_name(throughput_path),
        latency = file_name(latency_path),
    );

    if let Some(dns) = &result.dns {
        let dns_error_rate = if dns.total > 0 {
            dns.failures as f64 / dns.total as f64 * 100.0
        } else {
            0.0
        };
        let dns_avg_cpu = if dns_resources.is_empty() {
            0.0
        } else {
            dns_resources.iter().map(|s| s.cpu_percent).sum::<f64>() / dns_resources.len() as f64
        };
        let dns_max_cpu = dns_resources
            .iter()
            .map(|s| s.cpu_percent)
            .fold(0.0f64, f64::max);
        let dns_avg_ram = if dns_resources.is_empty() {
            0.0
        } else {
            dns_resources.iter().map(|s| s.ram_mb).sum::<f64>() / dns_resources.len() as f64
        };
        let dns_max_ram = dns_resources
            .iter()
            .map(|s| s.ram_mb)
            .fold(0.0f64, f64::max);
        html.push_str(&format!(
            r#"
<h2>DNS Benchmark</h2>
<div class="meta">
  Server: <span class="mono">{server}</span> &nbsp;|&nbsp;
  Users: {dns_users} &nbsp;|&nbsp;
  Duration: {dns_duration}s &nbsp;|&nbsp;
  Hit ratio target: {dns_hit_ratio:.0}
</div>
<div class="summary">
  <div class="card"><div class="label">Total Queries</div><div class="value">{dns_total}</div></div>
  <div class="card"><div class="label">Cache Hits</div><div class="value">{dns_hits}</div></div>
  <div class="card"><div class="label">Cache Misses</div><div class="value">{dns_misses}</div></div>
  <div class="card"><div class="label">Failures</div><div class="value">{dns_failures}</div></div>
  <div class="card"><div class="label">Query Error Rate</div><div class="value">{dns_error_rate:.2}%</div></div>
  <div class="card"><div class="label">Queries / sec</div><div class="value">{dns_rps:.2}</div></div>
  <div class="card"><div class="label">Actual Hit Ratio</div><div class="value">{dns_hit_ratio_actual:.2}</div></div>
  <div class="card"><div class="label">Avg CPU</div><div class="value">{dns_avg_cpu:.2}%</div></div>
  <div class="card"><div class="label">Peak CPU</div><div class="value">{dns_max_cpu:.2}%</div></div>
  <div class="card"><div class="label">Avg RAM</div><div class="value">{dns_avg_ram:.2} MiB</div></div>
  <div class="card"><div class="label">Peak RAM</div><div class="value">{dns_max_ram:.2} MiB</div></div>
  <div class="card"><div class="label">Latency p50</div><div class="value">{dns_p50:.2} ms</div></div>
  <div class="card"><div class="label">Latency p95</div><div class="value">{dns_p95:.2} ms</div></div>
  <div class="card"><div class="label">Latency p99</div><div class="value">{dns_p99:.2} ms</div></div>
</div>
<div class="row">
  <div class="chart"><h2>DNS Phase - CPU Usage</h2><img src="{dns_cpu_img}"></div>
  <div class="chart"><h2>DNS Phase - RAM Usage</h2><img src="{dns_ram_img}"></div>
</div>
<div class="row">
  <div class="chart"><h2>DNS Throughput</h2><img src="{dns_throughput_img}"></div>
  <div class="chart"><h2>DNS Cache Hits / s</h2><img src="{dns_hits_img}"></div>
</div>
<div class="row">
  <div class="chart"><h2>DNS Cache Misses / s</h2><img src="{dns_misses_img}"></div>
  <div class="chart"><h2>DNS Latency Over Time</h2><img src="{dns_latency_img}"></div>
</div>"#,
            server = dns.server,
            dns_users = dns.users,
            dns_duration = dns.duration_secs,
            dns_hit_ratio = dns.hit_ratio * 100.0,
            dns_total = dns.total,
            dns_hits = dns.hits,
            dns_misses = dns.misses,
            dns_failures = dns.failures,
            dns_error_rate = dns_error_rate,
            dns_rps = dns.requests_per_second,
            dns_hit_ratio_actual = dns.hit_ratio,
            dns_avg_cpu = dns_avg_cpu,
            dns_max_cpu = dns_max_cpu,
            dns_avg_ram = dns_avg_ram,
            dns_max_ram = dns_max_ram,
            dns_p50 = dns.latency.p50_ms,
            dns_p95 = dns.latency.p95_ms,
            dns_p99 = dns.latency.p99_ms,
            dns_cpu_img = dns_paths
                .iter()
                .find(|(name, _)| *name == "dns-cpu.svg")
                .map(|(_, p)| file_name(p))
                .unwrap_or_default(),
            dns_ram_img = dns_paths
                .iter()
                .find(|(name, _)| *name == "dns-ram.svg")
                .map(|(_, p)| file_name(p))
                .unwrap_or_default(),
            dns_throughput_img = dns_paths
                .iter()
                .find(|(name, _)| *name == "dns-throughput.svg")
                .map(|(_, p)| file_name(p))
                .unwrap_or_default(),
            dns_hits_img = dns_paths
                .iter()
                .find(|(name, _)| *name == "dns-hits.svg")
                .map(|(_, p)| file_name(p))
                .unwrap_or_default(),
            dns_misses_img = dns_paths
                .iter()
                .find(|(name, _)| *name == "dns-misses.svg")
                .map(|(_, p)| file_name(p))
                .unwrap_or_default(),
            dns_latency_img = dns_paths
                .iter()
                .find(|(name, _)| *name == "dns-latency.svg")
                .map(|(_, p)| file_name(p))
                .unwrap_or_default(),
        ));
    }

    html.push_str(
        r#"
<h2>Raw Data</h2>
<p class="mono">See <span class="mono">results.json</span> for per-request and per-sample data.</p>
</body>
</html>"#,
    );

    html
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .expect("path has a file name")
        .to_string_lossy()
        .to_string()
}

pub fn write(
    dir: &Path,
    result: &RunResult,
    http_resources: &[ResourceSample],
    dns_resources: &[ResourceSample],
) -> anyhow::Result<Report> {
    write_report(dir, result, http_resources, dns_resources)
}
