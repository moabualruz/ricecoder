use serde_json::to_string_pretty;
use std::{env, path::PathBuf, sync::Arc, time::Duration};

use ricegrep::api::models::BenchmarkSuiteResponse;
use ricegrep::benchmarking::BenchmarkCoordinator;
use ricegrep::performance::BenchmarkHarness;
use ricegrep::vector::{alerting::AlertManager, observability::VectorTelemetry};

#[derive(Debug)]
struct Options {
    index_dir: PathBuf,
    benchmark_root: PathBuf,
    run_suite: bool,
    run_load: bool,
    load_workers: usize,
    load_duration_secs: u64,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            index_dir: PathBuf::from("target/index"),
            benchmark_root: PathBuf::from("benchmark-results"),
            run_suite: false,
            run_load: false,
            load_workers: 4,
            load_duration_secs: 10,
        }
    }
}

impl Options {
    fn parse() -> Self {
        let mut options = Options::default();
        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--index-dir" => {
                    if let Some(value) = args.next() {
                        options.index_dir = PathBuf::from(value);
                    }
                }
                "--benchmark-root" => {
                    if let Some(value) = args.next() {
                        options.benchmark_root = PathBuf::from(value);
                    }
                }
                "--suite" => options.run_suite = true,
                "--load" => options.run_load = true,
                "--load-workers" => {
                    if let Some(value) = args.next() {
                        if let Ok(parsed) = value.parse::<usize>() {
                            options.load_workers = parsed;
                        }
                    }
                }
                "--load-duration" => {
                    if let Some(value) = args.next() {
                        if let Ok(parsed) = value.parse::<u64>() {
                            options.load_duration_secs = parsed;
                        }
                    }
                }
                other => eprintln!("warning: unexpected argument {}", other),
            }
        }
        if !options.run_suite && !options.run_load {
            options.run_suite = true;
        }
        options
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = Options::parse();
    let telemetry = Arc::new(VectorTelemetry::default());
    let alert_manager = Arc::new(AlertManager::new(telemetry.clone()));
    let coordinator = BenchmarkCoordinator::new(
        options.index_dir.clone(),
        options.benchmark_root.clone(),
        BenchmarkHarness::default_queries(),
        alert_manager,
    )?;

    if options.run_suite {
        let results = coordinator.run_suite()?;
        let suite = BenchmarkSuiteResponse {
            summary: format!("completed {} benchmark runs", results.len()),
            results,
        };
        println!("{}", to_string_pretty(&suite)?);
    }

    if options.run_load {
        let load_result = coordinator
            .run_load_test(
                options.load_workers,
                Duration::from_secs(options.load_duration_secs),
            )
            .await?;
        println!("{}", to_string_pretty(&load_result)?);
    }

    Ok(())
}
