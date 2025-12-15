use clap::Parser;
use ricecoder_benchmark::cli::{Cli, run_cli};
use std::process;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(e) = run_cli(cli).await {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}