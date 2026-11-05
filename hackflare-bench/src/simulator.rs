use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use serde::Serialize;

use crate::config::Config;
use crate::dns::DnsResult;
use crate::monitor::ResourceSample;

/// Maximum latency samples kept per second for the latency-over-time chart.
const PER_SECOND_LATENCY_CAP: usize = 2_000;
/// Maximum latency samples kept overall for percentile computation.
const GLOBAL_LATENCY_CAP: usize = 200_000;

#[derive(Debug, Clone, Serialize)]
pub struct LatencyStats {
    pub min_ms: f64,
    pub max_ms: f64,
    pub mean_ms: f64,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RunResult {
    pub target: String,
    pub users: usize,
    pub duration_secs: u64,
    pub ramp_up_secs: u64,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub requests_per_second: f64,
    pub latency: LatencyStats,
    /// Per-second request count: (seconds since start, count).
    pub throughput: Vec<(f64, f64)>,
    /// Per-second mean latency: (seconds since start, mean latency ms).
    pub latency_over_time: Vec<(f64, f64)>,

    /// Server CPU/RAM samples collected during the HTTP phase.
    pub http_resources: Vec<ResourceSample>,
    /// Server CPU/RAM samples collected during the DNS phase.
    pub dns_resources: Vec<ResourceSample>,

    /// Optional DNS benchmark results, populated when `--dns` is enabled.
    pub dns: Option<DnsResult>,
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let rank = (p / 100.0) * (sorted.len() as f64 - 1.0);
    let lo = rank.floor() as usize;
    let hi = (rank.ceil() as usize).min(sorted.len() - 1);
    let frac = rank - lo as f64;
    sorted[lo] + (sorted[hi] - sorted[lo]) * frac
}

pub async fn simulate(target: &str, config: &Config) -> RunResult {
    let client = reqwest::Client::builder()
        .timeout(config.request_timeout())
        .build()
        .expect("failed to build http client");

    let start = Instant::now();
    let duration = config.duration();
    let duration_secs = config.duration_secs as usize;
    let users = config.users.max(1);

    let total = Arc::new(AtomicU64::new(0));
    let successful = Arc::new(AtomicU64::new(0));
    let failed = Arc::new(AtomicU64::new(0));

    let buckets: Vec<Arc<AtomicU64>> = (0..duration_secs.max(1))
        .map(|_| Arc::new(AtomicU64::new(0)))
        .collect();
    let latency_buckets: Vec<Arc<Mutex<Vec<f64>>>> = (0..duration_secs.max(1))
        .map(|_| Arc::new(Mutex::new(Vec::new())))
        .collect();
    let latency_count: Vec<Arc<AtomicU64>> = (0..duration_secs.max(1))
        .map(|_| Arc::new(AtomicU64::new(0)))
        .collect();
    let global_latencies: Arc<Mutex<Vec<f64>>> =
        Arc::new(Mutex::new(Vec::with_capacity(GLOBAL_LATENCY_CAP)));

    let mut handles = Vec::with_capacity(users);
    for user_id in 0..users {
        let client = client.clone();
        let target = target.to_string();
        let ramp = config.ramp_up_secs as f64;
        let total = total.clone();
        let successful = successful.clone();
        let failed = failed.clone();
        let buckets = buckets.clone();
        let latency_buckets = latency_buckets.clone();
        let latency_count = latency_count.clone();
        let global_latencies = global_latencies.clone();

        handles.push(tokio::spawn(async move {
            // Stagger user start across the ramp-up window.
            let offset = ramp * (user_id as f64 / users as f64);
            tokio::time::sleep(std::time::Duration::from_secs_f64(offset)).await;

            loop {
                let elapsed = start.elapsed();
                if elapsed >= duration {
                    break;
                }
                let req_start = Instant::now();
                let (_status, ok) = match client.get(&target).send().await {
                    Ok(response) => {
                        let status = response.status().as_u16();
                        (Some(status), status < 500)
                    }
                    Err(_) => (None, false),
                };
                let latency_ms = req_start.elapsed().as_secs_f64() * 1000.0;

                total.fetch_add(1, Ordering::Relaxed);
                if ok {
                    successful.fetch_add(1, Ordering::Relaxed);
                } else {
                    failed.fetch_add(1, Ordering::Relaxed);
                }

                let sec = elapsed.as_secs() as usize;
                if sec < duration_secs {
                    buckets[sec].fetch_add(1, Ordering::Relaxed);

                    let counted = latency_count[sec].fetch_add(1, Ordering::Relaxed);
                    if (counted as usize) < PER_SECOND_LATENCY_CAP {
                        latency_buckets[sec]
                            .lock()
                            .expect("latency bucket mutex poisoned")
                            .push(latency_ms);
                    }

                    let mut globals = global_latencies
                        .lock()
                        .expect("global latency mutex poisoned");
                    if globals.len() < GLOBAL_LATENCY_CAP {
                        globals.push(latency_ms);
                    }
                }
            }
        }));
    }

    for handle in handles {
        let _ = handle.await;
    }

    let total_requests = total.load(Ordering::Relaxed);
    let successful_requests = successful.load(Ordering::Relaxed);
    let failed_requests = failed.load(Ordering::Relaxed);
    let secs = config.duration_secs as f64;
    let rps = total_requests as f64 / secs;

    // Throughput: one point per second.
    let throughput: Vec<(f64, f64)> = buckets
        .iter()
        .enumerate()
        .map(|(i, b)| (i as f64, b.load(Ordering::Relaxed) as f64))
        .collect();

    // Latency over time: mean latency per second that had requests.
    let mut latency_over_time = Vec::with_capacity(duration_secs.max(1));
    for (sec, bucket) in latency_buckets.iter().enumerate() {
        let samples = bucket.lock().expect("latency bucket mutex poisoned");
        if samples.is_empty() {
            continue;
        }
        let mean = samples.iter().sum::<f64>() / samples.len() as f64;
        latency_over_time.push((sec as f64, mean));
    }

    // Overall percentiles from the bounded global latency sample.
    let mut latencies: Vec<f64> = global_latencies
        .lock()
        .expect("global latency mutex poisoned")
        .clone();
    latencies.sort_by(|a, b| a.partial_cmp(b).expect("invalid latency"));

    let latency = if latencies.is_empty() {
        LatencyStats {
            min_ms: 0.0,
            max_ms: 0.0,
            mean_ms: 0.0,
            p50_ms: 0.0,
            p95_ms: 0.0,
            p99_ms: 0.0,
        }
    } else {
        let mean = latencies.iter().sum::<f64>() / latencies.len() as f64;
        LatencyStats {
            min_ms: latencies[0],
            max_ms: latencies[latencies.len() - 1],
            mean_ms: mean,
            p50_ms: percentile(&latencies, 50.0),
            p95_ms: percentile(&latencies, 95.0),
            p99_ms: percentile(&latencies, 99.0),
        }
    };

    RunResult {
        target: target.to_string(),
        users,
        duration_secs: config.duration_secs,
        ramp_up_secs: config.ramp_up_secs,
        total_requests,
        successful_requests,
        failed_requests,
        requests_per_second: rps,
        latency,
        throughput,
        latency_over_time,
        http_resources: Vec::new(),
        dns_resources: Vec::new(),
        dns: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentile_edges() {
        let empty: Vec<f64> = Vec::new();
        assert_eq!(percentile(&empty, 50.0), 0.0);

        let sorted = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(percentile(&sorted, 50.0), 3.0);
        assert_eq!(percentile(&sorted, 100.0), 5.0);
        assert_eq!(percentile(&sorted, 0.0), 1.0);
    }
}
