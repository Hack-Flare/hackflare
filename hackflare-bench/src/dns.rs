use std::net::UdpSocket;
use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::config::Config;

const DNS_CLASS_IN: u16 = 1;
const MAX_DNS_PAYLOAD: usize = 4096;

/// Flag bits in the DNS header flags field.
const FLAG_AA: u16 = 0x0400;
const FLAG_RA: u16 = 0x0080;
const RCODE_MASK: u16 = 0x000F;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsLatencyStats {
    pub min_ms: f64,
    pub max_ms: f64,
    pub mean_ms: f64,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsResult {
    pub server: String,
    pub users: usize,
    pub duration_secs: u64,
    pub total: u64,
    pub hits: u64,
    pub misses: u64,
    pub failures: u64,
    pub requests_per_second: f64,
    pub hit_ratio: f64,
    pub latency: DnsLatencyStats,
    /// Per-second request count: (seconds since start, count).
    pub throughput: Vec<(f64, f64)>,
    /// Per-second cache-hit count: (seconds since start, hits).
    pub hits_over_time: Vec<(f64, f64)>,
    /// Per-second cache-miss count: (seconds since start, misses).
    pub misses_over_time: Vec<(f64, f64)>,
    /// Per-second mean query latency: (seconds since start, mean latency ms).
    pub latency_over_time: Vec<(f64, f64)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DnsResponse {
    pub id: u16,
    pub flags: u16,
    pub rcode: u16,
    pub aa: bool,
    pub ra: bool,
    pub ancount: u16,
    pub qdcount: u16,
}

/// Build a minimal standard DNS query (recursion desired, one question).
pub fn build_query(id: u16, name: &str, qtype: u16) -> Vec<u8> {
    let mut out = Vec::with_capacity(64);
    out.extend_from_slice(&id.to_be_bytes());
    out.extend_from_slice(&0x0100u16.to_be_bytes()); // RD
    out.extend_from_slice(&1u16.to_be_bytes()); // qdcount
    out.extend_from_slice(&0u16.to_be_bytes()); // ancount
    out.extend_from_slice(&0u16.to_be_bytes()); // nscount
    out.extend_from_slice(&0u16.to_be_bytes()); // arcount
    encode_name(name, &mut out);
    out.extend_from_slice(&qtype.to_be_bytes());
    out.extend_from_slice(&DNS_CLASS_IN.to_be_bytes());
    out
}

fn encode_name(name: &str, out: &mut Vec<u8>) {
    let trimmed = name.trim_end_matches('.');
    if trimmed.is_empty() {
        out.push(0);
        return;
    }
    for label in trimmed.split('.') {
        let bytes = label.as_bytes();
        out.push(bytes.len() as u8);
        out.extend_from_slice(bytes);
    }
    out.push(0);
}

/// Skip over a DNS (possibly compressed) name starting at `pos`
fn skip_name(buf: &[u8], mut pos: usize) -> anyhow::Result<usize> {
    loop {
        anyhow::ensure!(pos < buf.len(), "truncated DNS name");
        let len = buf[pos] as usize;
        pos += 1;
        if len == 0 {
            break;
        }
        if len & 0xC0 != 0 {
            // Name compression pointer: jump to the target offset.
            anyhow::ensure!(pos < buf.len(), "truncated DNS pointer");
            let target = ((len & 0x3F) << 8) | (buf[pos] as usize);
            return Ok(target);
        }
        pos += len;
    }
    Ok(pos)
}

/// Parse a DNS response header and return the relevant fields.
pub fn parse_response(buf: &[u8]) -> anyhow::Result<DnsResponse> {
    anyhow::ensure!(buf.len() >= 12, "truncated DNS response");
    let id = u16::from_be_bytes([buf[0], buf[1]]);
    let flags = u16::from_be_bytes([buf[2], buf[3]]);
    let qdcount = u16::from_be_bytes([buf[4], buf[5]]);
    let ancount = u16::from_be_bytes([buf[6], buf[7]]);
    let rcode = flags & RCODE_MASK;
    let aa = (flags & FLAG_AA) != 0;
    let ra = (flags & FLAG_RA) != 0;

    // Skip the question section to validate the message is well-formed.
    let mut pos = 12;
    for _ in 0..qdcount {
        pos = skip_name(buf, pos)?;
        pos += 4; // qtype + qclass
    }

    Ok(DnsResponse {
        id,
        flags,
        rcode,
        aa,
        ra,
        ancount,
        qdcount,
    })
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

pub async fn simulate_dns(server: &str, config: &Config) -> DnsResult {
    let duration = config.dns_duration();
    let duration_secs = config.dns_duration_secs as usize;
    let users = config.dns_users.max(1);
    let hit_ratio = config.dns_hit_ratio.clamp(0.0, 1.0);
    let qname_hit = config.dns_qname_hit.clone();
    let qname_miss = config.dns_qname_miss.clone();
    let qtype = config.dns_qtype;

    let socket = UdpSocket::bind("0.0.0.0:0").expect("failed to bind DNS client socket");
    socket
        .connect(server)
        .expect("failed to connect DNS client socket");
    socket
        .set_read_timeout(Some(config.dns_timeout()))
        .expect("failed to set socket read timeout");

    let start = Instant::now();

    let total = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
    let hits = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
    let misses = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
    let failures = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));

    let buckets: Vec<std::sync::Arc<std::sync::atomic::AtomicU64>> = (0..duration_secs.max(1))
        .map(|_| std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0)))
        .collect();
    let hit_buckets: Vec<std::sync::Arc<std::sync::atomic::AtomicU64>> = (0..duration_secs.max(1))
        .map(|_| std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0)))
        .collect();
    let miss_buckets: Vec<std::sync::Arc<std::sync::atomic::AtomicU64>> = (0..duration_secs.max(1))
        .map(|_| std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0)))
        .collect();
    let latency_buckets: Vec<std::sync::Arc<std::sync::Mutex<Vec<f64>>>> = (0..duration_secs
        .max(1))
        .map(|_| std::sync::Arc::new(std::sync::Mutex::new(Vec::with_capacity(2_000))))
        .collect();
    let global_latencies: std::sync::Arc<std::sync::Mutex<Vec<f64>>> =
        std::sync::Arc::new(std::sync::Mutex::new(Vec::with_capacity(200_000)));

    let mut handles = Vec::with_capacity(users);
    for user_id in 0..users {
        let socket = socket.try_clone().expect("failed to clone socket");
        let total = total.clone();
        let hits = hits.clone();
        let misses = misses.clone();
        let failures = failures.clone();
        let buckets = buckets.clone();
        let hit_buckets = hit_buckets.clone();
        let miss_buckets = miss_buckets.clone();
        let latency_buckets = latency_buckets.clone();
        let global_latencies = global_latencies.clone();

        let qname = if (user_id as f64 / users as f64) < hit_ratio {
            qname_hit.clone()
        } else {
            qname_miss.clone()
        };

        handles.push(tokio::spawn(async move {
            let mut id: u16 = 0;
            loop {
                let elapsed = start.elapsed();
                if elapsed >= duration {
                    break;
                }
                id = id.wrapping_add(1);
                let query = build_query(id, qname.as_str(), qtype);
                let req_start = Instant::now();

                let result = (|| -> anyhow::Result<DnsResponse> {
                    socket.send(&query)?;
                    let mut buf = [0u8; MAX_DNS_PAYLOAD];
                    let len = socket.recv(&mut buf)?;
                    parse_response(&buf[..len])
                })();

                let latency_ms = req_start.elapsed().as_secs_f64() * 1000.0;
                let sec = elapsed.as_secs() as usize;
                let hit = matches!(&result, Ok(r) if r.aa);

                total.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                if hit {
                    hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                } else if result.is_ok() {
                    misses.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                } else {
                    failures.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }

                if sec < duration_secs.max(1) {
                    buckets[sec].fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    if hit {
                        hit_buckets[sec].fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    } else if result.is_ok() {
                        miss_buckets[sec].fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    }

                    let mut bucket = latency_buckets[sec]
                        .lock()
                        .expect("latency bucket mutex poisoned");
                    if bucket.len() < 2_000 {
                        bucket.push(latency_ms);
                    }

                    let mut globals = global_latencies
                        .lock()
                        .expect("global latency mutex poisoned");
                    if globals.len() < 200_000 {
                        globals.push(latency_ms);
                    }
                }
            }
        }));
    }

    for handle in handles {
        let _ = handle.await;
    }

    let total_requests = total.load(std::sync::atomic::Ordering::Relaxed);
    let hit_count = hits.load(std::sync::atomic::Ordering::Relaxed);
    let miss_count = misses.load(std::sync::atomic::Ordering::Relaxed);
    let failure_count = failures.load(std::sync::atomic::Ordering::Relaxed);
    let secs = config.dns_duration_secs as f64;
    let rps = total_requests as f64 / secs;

    let throughput: Vec<(f64, f64)> = buckets
        .iter()
        .enumerate()
        .map(|(i, b)| {
            (
                i as f64,
                b.load(std::sync::atomic::Ordering::Relaxed) as f64,
            )
        })
        .collect();
    let hits_over_time: Vec<(f64, f64)> = hit_buckets
        .iter()
        .enumerate()
        .map(|(i, b)| {
            (
                i as f64,
                b.load(std::sync::atomic::Ordering::Relaxed) as f64,
            )
        })
        .collect();
    let misses_over_time: Vec<(f64, f64)> = miss_buckets
        .iter()
        .enumerate()
        .map(|(i, b)| {
            (
                i as f64,
                b.load(std::sync::atomic::Ordering::Relaxed) as f64,
            )
        })
        .collect();
    let latency_over_time: Vec<(f64, f64)> = latency_buckets
        .iter()
        .enumerate()
        .filter_map(|(i, b)| {
            let samples = b.lock().expect("latency bucket mutex poisoned");
            if samples.is_empty() {
                None
            } else {
                Some((i as f64, samples.iter().sum::<f64>() / samples.len() as f64))
            }
        })
        .collect();

    let mut latencies: Vec<f64> = global_latencies
        .lock()
        .expect("global latency mutex poisoned")
        .clone();
    latencies.sort_by(|a, b| a.partial_cmp(b).expect("invalid latency"));

    let latency = if latencies.is_empty() {
        DnsLatencyStats {
            min_ms: 0.0,
            max_ms: 0.0,
            mean_ms: 0.0,
            p50_ms: 0.0,
            p95_ms: 0.0,
            p99_ms: 0.0,
        }
    } else {
        let mean = latencies.iter().sum::<f64>() / latencies.len() as f64;
        DnsLatencyStats {
            min_ms: latencies[0],
            max_ms: latencies[latencies.len() - 1],
            mean_ms: mean,
            p50_ms: percentile(&latencies, 50.0),
            p95_ms: percentile(&latencies, 95.0),
            p99_ms: percentile(&latencies, 99.0),
        }
    };

    let hit_ratio = if total_requests > 0 {
        hit_count as f64 / total_requests as f64
    } else {
        0.0
    };

    DnsResult {
        server: server.to_string(),
        users,
        duration_secs: config.dns_duration_secs,
        total: total_requests,
        hits: hit_count,
        misses: miss_count,
        failures: failure_count,
        requests_per_second: rps,
        hit_ratio,
        latency,
        throughput,
        hits_over_time,
        misses_over_time,
        latency_over_time,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_and_parse_roundtrip() {
        let query = build_query(42, "www.example.com", 1);
        // The response is fabricated: id=42, flags with AA set, rcode NOERROR,
        // one answer.
        let flags: u16 = 0x8000 | FLAG_AA | 0x0100; // QR+AA+RD
        let mut response = Vec::with_capacity(12 + query.len());
        response.extend_from_slice(&42u16.to_be_bytes());
        response.extend_from_slice(&flags.to_be_bytes());
        response.extend_from_slice(&1u16.to_be_bytes()); // qdcount
        response.extend_from_slice(&1u16.to_be_bytes()); // ancount
        response.extend_from_slice(&0u16.to_be_bytes());
        response.extend_from_slice(&0u16.to_be_bytes());
        response.extend_from_slice(&query[12..]); // copy the question section

        let parsed = parse_response(&response).expect("valid response");
        assert_eq!(parsed.id, 42);
        assert!(parsed.aa, "expected authoritative answer (cache hit)");
        assert_eq!(parsed.rcode, 0);
        assert_eq!(parsed.ancount, 1);

        // Non-authoritative response => cache miss.
        let flags_no_aa: u16 = 0x8000 | 0x0100;
        let mut response2 = Vec::new();
        response2.extend_from_slice(&43u16.to_be_bytes());
        response2.extend_from_slice(&flags_no_aa.to_be_bytes());
        response2.extend_from_slice(&1u16.to_be_bytes());
        response2.extend_from_slice(&0u16.to_be_bytes());
        response2.extend_from_slice(&0u16.to_be_bytes());
        response2.extend_from_slice(&0u16.to_be_bytes());
        response2.extend_from_slice(&query[12..]);
        let parsed2 = parse_response(&response2).expect("valid response");
        assert!(
            !parsed2.aa,
            "expected non-authoritative answer (cache miss)"
        );
    }

    #[test]
    fn percentile_edges() {
        let empty: Vec<f64> = Vec::new();
        assert_eq!(percentile(&empty, 50.0), 0.0);
        let sorted = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(percentile(&sorted, 50.0), 3.0);
        assert_eq!(percentile(&sorted, 100.0), 5.0);
    }
}
