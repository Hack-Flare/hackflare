use std::time::Instant;

use clap::Parser;

use hackflare_bench::{Config, run};

#[tokio::main]
async fn main() {
    if let Err(error) = run_main().await {
        eprintln!("benchmark failed: {error}");
        std::process::exit(1);
    }
}

async fn run_main() -> anyhow::Result<()> {
    let config = Config::parse();
    let start = Instant::now();

    let report = run(config).await?;

    println!("benchmark complete");
    println!("  report: {}", report.html_path.display());
    println!("  json:   {}", report.json_path.display());
    println!("  elapsed: {:.2}s", start.elapsed().as_secs_f64());
    Ok(())
}
