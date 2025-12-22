use std::process;

use clap::Parser;
use ricecoder_benchmark::cli::{run_cli, Cli};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(e) = run_cli(cli).await {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
