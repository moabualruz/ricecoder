use clap::{Parser, Subcommand};
use clap::{Parser, Subcommand};
use ricecoder_performance::{PerformanceValidator, PerformanceBaseline, PerformanceRegressionDetector};
use std::path::PathBuf;

/// RiceCoder Performance Validation Tool
#[derive(Parser)]
#[command(name = "ricecoder-performance")]
#[command(about = "Performance validation and regression detection for RiceCoder")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate performance against targets
    Validate {
        /// Path to ricecoder binary
        #[arg(short, long)]
        binary: PathBuf,

        /// Path to baseline file (optional)
        #[arg(short, long)]
        baseline: Option<PathBuf>,
    },
    /// Update performance baselines
    UpdateBaseline {
        /// Path to ricecoder binary
        #[arg(short, long)]
        binary: PathBuf,

        /// Path to baseline file
        #[arg(short, long)]
        baseline: PathBuf,
    },
    /// Check for performance regressions
    CheckRegression {
        /// Path to ricecoder binary
        #[arg(short, long)]
        binary: PathBuf,

        /// Path to baseline file
        #[arg(short, long)]
        baseline: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Validate { binary, baseline } => {
            let baseline_data = if let Some(path) = baseline {
                Some(PerformanceBaseline::load_from_file(path)?)
            } else {
                None
            };

            let validator = PerformanceValidator::new(
                binary.to_string_lossy().to_string(),
                baseline_data,
            );

            let results = validator.run_all_validations().await?;

            println!("=== Performance Validation Results ===");
            let mut all_passed = true;

            for result in results {
                let status = if result.passed { "✅ PASS" } else { "❌ FAIL" };
                println!("\n{}: {}", result.test_name, status);

                if !result.messages.is_empty() {
                    for message in result.messages {
                        println!("  ⚠️  {}", message);
                    }
                }

                println!("  📊 P95: {:.2}ms",
                    result.metrics.p95_time_ns as f64 / 1_000_000.0);
                println!("  🧠 Memory: {:.1}MB",
                    result.metrics.peak_memory_bytes as f64 / (1024.0 * 1024.0));

                if !result.passed {
                    all_passed = false;
                }
            }

            println!("\n=== Summary ===");
            if all_passed {
                println!("✅ All performance targets met!");
            } else {
                println!("❌ Some performance targets not met. See details above.");
                std::process::exit(1);
            }
        }

        Commands::UpdateBaseline { binary, baseline } => {
            let validator = PerformanceValidator::new(
                binary.to_string_lossy().to_string(),
                None,
            );

            let results = validator.run_all_validations().await?;
            let mut baseline_data = PerformanceBaseline::new();

            for result in results {
                baseline_data.update_baseline(result.test_name, &result.metrics);
            }

            baseline_data.save_to_file(baseline)?;
            println!("✅ Performance baseline updated successfully!");
        }

        Commands::CheckRegression { binary, baseline } => {
            let baseline_data = PerformanceBaseline::load_from_file(baseline)?;
            let validator = PerformanceValidator::new(
                binary.to_string_lossy().to_string(),
                Some(baseline_data.clone()),
            );

            let results = validator.run_all_validations().await?;
            let mut detector = PerformanceRegressionDetector::new(baseline_data);

            let metrics: Vec<_> = results.iter().map(|r| r.metrics.clone()).collect();
            let regressions = detector.detect(&metrics);

            if regressions.is_empty() {
                println!("✅ No performance regressions detected!");
            } else {
                println!("❌ Performance regressions detected:");
                for regression in regressions {
                    match regression {
                        ricecoder_performance::RegressionAlert::PerformanceDegradation {
                            test_name, degradation_percent, ..
                        } => {
                            println!("  📉 {}: {:.1}% performance degradation", test_name, degradation_percent);
                        }
                        ricecoder_performance::RegressionAlert::MemoryRegression {
                            test_name, increase_percent, ..
                        } => {
                            println!("  🧠 {}: {:.1}% memory increase", test_name, increase_percent);
                        }
                        ricecoder_performance::RegressionAlert::TargetExceeded {
                            test_name, exceed_percent, ..
                        } => {
                            println!("  🎯 {}: {:.1}% over target threshold", test_name, exceed_percent);
                        }
                    }
                }
                std::process::exit(1);
            }
        }
    }

    Ok(())
}