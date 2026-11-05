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
    resources: &[ResourceSample],
) -> anyhow::Result<Report> {
    std::fs::create_dir_all(dir)?;

    let json_path = dir.join("results.json");
    let json = serde_json::to_string_pretty(result)?;
    std::fs::write(&json_path, json)?;

    let cpu_points = to_points(resources, |s| s.cpu_percent);
    let ram_points = to_points(resources, |s| s.ram_mb);
    let latency_points: Vec<(f64, f64)> = result.latency_over_time.clone();

    let cpu_path = dir.join("cpu.svg");
    let ram_path = dir.join("ram.svg");
    let throughput_path = dir.join("throughput.svg");
    let latency_path = dir.join("latency.svg");

    write_line_chart(&cpu_path, "CPU Usage (%)", "%", &cpu_points)?;
    write_line_chart(&ram_path, "RAM Usage (MiB)", "MiB", &ram_points)?;
    write_line_chart(
        &throughput_path,
        "Throughput (requests / second)",
        "req/s",
        &result.throughput,
    )?;
    write_line_chart(&latency_path, "Request Latency (ms)", "ms", &latency_points)?;

    let html_path = dir.join("report.html");
    let html = render_html(
        result,
        resources,
        &cpu_path,
        &ram_path,
        &throughput_path,
        &latency_path,
    );
    std::fs::write(&html_path, html)?;

    write_webserver_run_script(&dir)?;

    Ok(Report {
        dir: dir.to_path_buf(),
        json_path,
        html_path,
    })
}

fn write_webserver_run_script(
    dir: &Path,
) -> anyhow::Result<()> {
    let script_path = dir.join("webserver-run.sh");
    let script = format!(
        r#"#!/bin/bash
        # Run a simple webserver for the report.html file
        # Usage: ./webserver-run.sh
        set -euo pipefail
        cd "$(dirname "$0")"
        echo "Serving report.html at http://localhost:8000"
        python3 -m http.server 8000
"#,
    );
    std::fs::write(&script_path, script)?;
    Ok(())
}

fn render_html(
    result: &RunResult,
    resources: &[ResourceSample],
    cpu_path: &Path,
    ram_path: &Path,
    throughput_path: &Path,
    latency_path: &Path,
) -> String {
    let avg_cpu = if resources.is_empty() {
        0.0
    } else {
        resources.iter().map(|s| s.cpu_percent).sum::<f64>() / resources.len() as f64
    };
    let max_cpu = resources
        .iter()
        .map(|s| s.cpu_percent)
        .fold(0.0f64, f64::max);
    let avg_ram = if resources.is_empty() {
        0.0
    } else {
        resources.iter().map(|s| s.ram_mb).sum::<f64>() / resources.len() as f64
    };
    let max_ram = resources.iter().map(|s| s.ram_mb).fold(0.0f64, f64::max);

    let error_rate = if result.total_requests > 0 {
        result.failed_requests as f64 / result.total_requests as f64 * 100.0
    } else {
        0.0
    };

    format!(
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
  <div class="chart"><h2>CPU Usage</h2><img src="{cpu}"></div>
  <div class="chart"><h2>RAM Usage</h2><img src="{ram}"></div>
</div>
<div class="row">
  <div class="chart"><h2>Throughput</h2><img src="{throughput}"></div>
  <div class="chart"><h2>Latency Over Time</h2><img src="{latency}"></div>
</div>

<h2>Raw Data</h2>
<p class="mono">See <span class="mono">results.json</span> for per-request and per-sample data.</p>
</body>
</html>"#,
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
        cpu = cpu_path
            .file_name()
            .expect("cpu path has a name")
            .to_string_lossy(),
        ram = ram_path
            .file_name()
            .expect("ram path has a name")
            .to_string_lossy(),
        throughput = throughput_path
            .file_name()
            .expect("throughput path has a name")
            .to_string_lossy(),
        latency = latency_path
            .file_name()
            .expect("latency path has a name")
            .to_string_lossy(),
    )
}

pub fn write(
    dir: &Path,
    result: &RunResult,
    resources: &[ResourceSample],
) -> anyhow::Result<Report> {
    write_report(dir, result, resources)
}
