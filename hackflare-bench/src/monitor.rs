use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSample {
    pub t: f64,
    pub cpu_percent: f64,
    pub ram_mb: f64,
}

/// Find the PID of a running process by scanning /proc for a matching comm name.
pub fn find_pid_by_name(name: &str) -> Option<u32> {
    let proc = fs::read_dir("/proc").ok()?;
    for entry in proc.flatten() {
        let pid_str = entry.file_name().to_string_lossy().into_owned();
        let pid: u32 = match pid_str.parse() {
            Ok(p) => p,
            Err(_) => continue,
        };
        let stat_path = format!("/proc/{pid}/stat");
        let stat = fs::read_to_string(&stat_path).ok()?;
        // stat format: pid (comm) ... - comm is between first '(' and last ')'
        let start = stat.find('(')?;
        let end = stat.rfind(')')?;
        if start < end {
            let comm = &stat[start + 1..end];
            if comm == name {
                return Some(pid);
            }
        }
    }
    None
}

/// Samples CPU and RAM usage of a server process at a fixed interval.
///
/// Runs on a std thread (independent of the tokio runtime) so sampling keeps happening even while the benchmark is hammering the server.
/// Reads `/proc/{pid}/stat` for CPU time and `/proc/{pid}/status` for RSS.
pub struct ProcessMonitor {
    pid: u32,
    interval: Duration,
    samples: Arc<Mutex<Vec<ResourceSample>>>,
    stop: Arc<AtomicBool>,
    start: Instant,
    handle: Option<thread::JoinHandle<()>>,
}

impl ProcessMonitor {
    pub fn new(pid: u32, interval: Duration) -> Self {
        Self {
            pid,
            interval,
            samples: Arc::new(Mutex::new(Vec::new())),
            stop: Arc::new(AtomicBool::new(false)),
            start: Instant::now(),
            handle: None,
        }
    }

    /// Start monitoring. Discovers the server process named "hackflare-app" in /proc;
    /// falls back to the provided pid only if discovery fails.
    pub fn start(&mut self) -> anyhow::Result<()> {
        if self.handle.is_some() {
            anyhow::bail!("monitor already started");
        }
        // Prefer name-based discovery
        let pid = find_pid_by_name("hackflare-app").unwrap_or(self.pid);
        if pid == 0 || !self.proc_exists(pid) {
            anyhow::bail!("could not find hackflare-app process to monitor");
        }
        // Update the stored pid so stop() / sampling use the discovered one.
        self.pid = pid;

        let interval = self.interval;
        let samples = self.samples.clone();
        let stop = self.stop.clone();
        let start = self.start;

        let handle = thread::spawn(move || {
            let mut last_cpu_time: Option<u64> = None;
            let mut last_time: Option<Instant> = None;
            let mut consecutive_misses = 0u32;

            loop {
                if stop.load(Ordering::Relaxed) {
                    break;
                }
                thread::sleep(interval);
                if stop.load(Ordering::Relaxed) {
                    break;
                }

                let now = Instant::now();
                let proc_stat = match fs::read_to_string(format!("/proc/{pid}/stat")) {
                    Ok(s) => s,
                    Err(_) => {
                        consecutive_misses += 1;
                        if consecutive_misses > 10 {
                            break;
                        }
                        continue;
                    }
                };

                let ram_mb = match fs::read_to_string(format!("/proc/{pid}/status")) {
                    Ok(s) => parse_vm_rss_kb(&s) as f64 / 1024.0,
                    Err(_) => 0.0,
                };

                // /proc/{pid}/stat: pid (comm) state ... utime(14) stime(15).
                let rest = proc_stat
                    .rsplit_once(')')
                    .map(|(_, after)| after.trim_start())
                    .unwrap_or(&proc_stat);
                let fields: Vec<&str> = rest.split_whitespace().collect();

                if fields.len() < 13 {
                    consecutive_misses += 1;
                    if consecutive_misses > 10 {
                        break;
                    }
                    continue;
                }
                let utime: u64 = fields[11].parse().unwrap_or(0);
                let stime: u64 = fields[12].parse().unwrap_or(0);
                let total_cpu_time = utime + stime;

                let clock_ticks = 100.0f64;

                let cpu_percent =
                    if let (Some(last_cpu), Some(last_now)) = (last_cpu_time, last_time) {
                        let elapsed = now.duration_since(last_now).as_secs_f64();
                        if elapsed > 0.0 && total_cpu_time >= last_cpu {
                            let delta_ticks = (total_cpu_time - last_cpu) as f64;
                            (delta_ticks / clock_ticks / elapsed) * 100.0
                        } else {
                            0.0
                        }
                    } else {
                        0.0
                    };

                last_cpu_time = Some(total_cpu_time);
                last_time = Some(now);
                consecutive_misses = 0;

                let t = start.elapsed().as_secs_f64();
                samples
                    .lock()
                    .expect("samples mutex poisoned")
                    .push(ResourceSample {
                        t,
                        cpu_percent,
                        ram_mb,
                    });
            }
        });

        self.handle = Some(handle);
        Ok(())
    }

    fn proc_exists(&self, pid: u32) -> bool {
        Path::new(&format!("/proc/{pid}/stat")).exists()
    }

    pub fn stop(&mut self) -> Vec<ResourceSample> {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
        let mut samples = self.samples.lock().expect("samples mutex poisoned");
        samples.sort_by(|a, b| a.t.partial_cmp(&b.t).expect("invalid sample timestamp"));
        std::mem::take(&mut *samples)
    }

    pub fn sample_count(&self) -> usize {
        self.samples.lock().expect("samples mutex poisoned").len()
    }
}

fn parse_vm_rss_kb(status: &str) -> u64 {
    for line in status.lines() {
        if line.starts_with("VmRSS:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                return parts[1].parse().unwrap_or(0);
            }
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_serialization_roundtrip() {
        let sample = ResourceSample {
            t: 0.0,
            cpu_percent: 12.5,
            ram_mb: 256.0,
        };
        let json = serde_json::to_string(&sample).expect("serialization failed");
        let back: ResourceSample = serde_json::from_str(&json).expect("deserialization failed");
        assert_eq!(sample.t, back.t);
        assert_eq!(sample.cpu_percent, back.cpu_percent);
        assert_eq!(sample.ram_mb, back.ram_mb);
    }

    #[test]
    fn parse_vm_rss_kb_works() {
        let status = "Name: test\nVmRSS:   12345 kB\nVmSize: 99999 kB\n";
        assert_eq!(parse_vm_rss_kb(status), 12345);

        let status_no_rss = "Name: test\nVmSize: 99999 kB\n";
        assert_eq!(parse_vm_rss_kb(status_no_rss), 0);
    }
}
