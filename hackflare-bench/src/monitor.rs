use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use sysinfo::{Pid, ProcessesToUpdate, System};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSample {
    pub t: f64,
    pub cpu_percent: f64,
    pub ram_mb: f64,
}

/// Samples CPU and RAM usage of a server process at a fixed interval.
pub struct ProcessMonitor {
    pid: u32,
    interval: Duration,
    samples: Arc<Mutex<Vec<ResourceSample>>>,
    stop: Arc<AtomicBool>,
    start: Instant,
    handle: Option<tokio::task::JoinHandle<()>>,
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

    /// Begin sampling in the background.
    pub fn start(&mut self) -> anyhow::Result<()> {
        if self.handle.is_some() {
            anyhow::bail!("monitor already started");
        }
        let pid = self.pid;
        let interval = self.interval;
        let samples = self.samples.clone();
        let stop = self.stop.clone();
        let start = self.start;

        let handle = tokio::spawn(async move {
            let mut sys = System::new_all();
            let spid = Pid::from(pid as usize);

            // Prime the CPU calculation, then wait one interval
            sys.refresh_processes(ProcessesToUpdate::All, true);
            let _prime = sys.process(spid);
            tokio::time::sleep(interval).await;

            loop {
                if stop.load(Ordering::Relaxed) {
                    break;
                }
                tokio::time::sleep(interval).await;
                if stop.load(Ordering::Relaxed) {
                    break;
                }

                sys.refresh_processes(ProcessesToUpdate::Some(&[spid]), true);
                let process = match sys.process(spid) {
                    Some(process) => process,
                    None => break,
                };

                let t = start.elapsed().as_secs_f64();
                samples
                    .lock()
                    .expect("samples mutex poisoned")
                    .push(ResourceSample {
                        t,
                        cpu_percent: process.cpu_usage() as f64,
                        ram_mb: process.memory() as f64 / 1_048_576.0,
                    });
            }
        });

        self.handle = Some(handle);
        Ok(())
    }

    /// Stop sampling and return the collected samples.
    pub async fn stop(&mut self) -> Vec<ResourceSample> {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.await;
        }
        let mut samples = self.samples.lock().expect("samples mutex poisoned");
        samples.sort_by(|a, b| a.t.partial_cmp(&b.t).expect("invalid sample timestamp"));
        std::mem::take(&mut *samples)
    }

    pub fn sample_count(&self) -> usize {
        self.samples.lock().expect("samples mutex poisoned").len()
    }
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
}
