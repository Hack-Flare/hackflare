use std::path::Path;
use std::process::{Child, Stdio};

use tokio::time::{Duration, sleep};

/// A self-hosted hackflare-app instance launched for benchmarking.
pub struct Server {
    child: Option<Child>,
    port: u16,
}

impl Server {
    /// Spawn the binary and return a handle.
    pub fn spawn(binary: &Path, port: u16, dev: bool) -> anyhow::Result<Self> {
        let mut command = std::process::Command::new(binary);
        if dev {
            command.arg("--dev");
        }
        command
            .env("BIND_ADDR", format!("127.0.0.1:{port}"))
            // Dummy values sufficient for config validation
            .env("JWT_SECRET", "CC+GwABTHe/pCrp402AvRePRNAfHJMPHawfVTvP0M+0=")
            .env("HCA_CLIENT_ID", "bench")
            .env("HCA_CLIENT_SECRET", "bench")
            .env(
                "HCA_REDIRECT_URI",
                format!("http://127.0.0.1:{port}/api/v1/auth/callback"),
            )
            .env("ENVIRONMENT", "development")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let child = command.spawn()?;
        Ok(Self {
            child: Some(child),
            port,
        })
    }

    /// Wait until `/api/v1/health` returns successfully, up to 60 seconds.
    pub async fn wait_for_health(&self) -> anyhow::Result<()> {
        let url = format!("http://127.0.0.1:{}/api/v1/health", self.port);
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()?;

        let mut attempts = 0;
        loop {
            attempts += 1;
            if attempts > 120 {
                anyhow::bail!("server did not become healthy at {url}");
            }
            match client.get(&url).send().await {
                Ok(response) if response.status().is_success() => return Ok(()),
                _ => sleep(Duration::from_millis(250)).await,
            }
        }
    }

    /// Return the process id of the server, if still running.
    pub fn child_id(&self) -> u32 {
        self.child.as_ref().map(|c| c.id()).unwrap_or(0)
    }

    /// Gracefully shut the server down, killing it if it does not exit.
    pub async fn shutdown(&mut self) -> anyhow::Result<()> {
        let Some(mut child) = self.child.take() else {
            return Ok(());
        };

        let _ = child.kill();

        let _ = child.wait();

        Ok(())
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn child_id_defaults_to_zero_when_none() {
        let server = Server {
            child: None,
            port: 18080,
        };
        assert_eq!(server.child_id(), 0);
    }
}
