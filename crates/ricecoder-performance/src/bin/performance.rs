use clap::{Parser, Subcommand};
use ricecoder_performance::{
    PerformanceValidator, PerformanceBaseline, PerformanceRegressionDetector,
    PerformanceProfiler, OptimizationPipeline, EnterpriseMonitor, AlertConfig,
    AlertDestination, AlertSeverity, create_default_pipeline, EnterpriseSimulator
};
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
    /// Profile application performance
    Profile {
        /// Path to ricecoder binary
        #[arg(short, long)]
        binary: PathBuf,

        /// Output profile report file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Run optimization pipeline
    Optimize {
        /// Path to ricecoder binary
        #[arg(short, long)]
        binary: PathBuf,

        /// Path to baseline file (optional)
        #[arg(short, long)]
        baseline: Option<PathBuf>,

        /// Output optimization report file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Enterprise monitoring mode
    Monitor {
        /// Path to ricecoder binary
        #[arg(short, long)]
        binary: PathBuf,

        /// Path to baseline file
        #[arg(short, long)]
        baseline: PathBuf,

        /// Monitoring interval in seconds
        #[arg(short, long, default_value = "300")]
        interval: u64,

        /// Slack webhook URL for alerts
        #[arg(long)]
        slack_webhook: Option<String>,

        /// Email recipients for alerts (comma-separated)
        #[arg(long)]
        email_recipients: Option<String>,
    },
    /// Run enterprise workload simulation
    Simulate {
        /// Path to ricecoder binary
        #[arg(short, long)]
        binary: PathBuf,

        /// Simulation duration in seconds
        #[arg(short, long, default_value = "300")]
        duration: u64,

        /// Output simulation report file
        #[arg(short, long)]
        output: Option<PathBuf>,
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

        Commands::Profile { binary, output } => {
            let mut profiler = PerformanceProfiler::new();

            // Profile key operations
            profiler.start_profiling();

            // Profile CLI startup
            profiler.profile_function("cli_startup", || {
                let _ = std::process::Command::new(&binary)
                    .arg("--version")
                    .output();
            });

            // Profile config loading
            profiler.profile_function("config_loading", || {
                let _ = std::fs::read_to_string("dummy_config.yaml");
            });

            let profile_result = profiler.stop_profiling();

            // Generate and output report
            let report = profile_result.generate_report();

            match output {
                Some(path) => {
                    std::fs::write(&path, &report)?;
                    println!("✅ Profile report saved to: {}", path.display());
                }
                None => {
                    println!("{}", report);
                }
            }
        }

        Commands::Optimize { binary, baseline, output } => {
            let baseline_data = baseline
                .map(|path| PerformanceBaseline::load_from_file(path))
                .transpose()?;

            let mut pipeline = create_default_pipeline();
            if let Some(baseline) = baseline_data {
                pipeline = pipeline.with_baseline(baseline);
            }

            // Run optimization on a sample workload
            let optimization_result = pipeline.run_optimization(|profiler| {
                // Simulate some work to profile
                profiler.profile_function("sample_workload", || {
                    let _ = std::process::Command::new(&binary)
                        .arg("--help")
                        .output();
                });
            }).await;

            // Generate optimization report
            let mut report = format!("=== Performance Optimization Report ===\n");
            report.push_str(&format!("Total Duration: {:.2}s\n", optimization_result.profile_result.total_duration.as_secs_f64()));
            report.push_str(&format!("Expected Improvement: {:.1}%\n\n", optimization_result.expected_improvement_percent));

            report.push_str("=== Applied Optimizations ===\n");
            for opt in &optimization_result.applied_optimizations {
                report.push_str(&format!("• {}: {} (+{:.1}%)\n",
                    opt.name, opt.description, opt.expected_improvement_percent));
            }

            report.push_str("\n=== Optimization Suggestions ===\n");
            for suggestion in &optimization_result.optimization_suggestions {
                let priority_icon = match suggestion.priority {
                    ricecoder_performance::OptimizationPriority::Critical => "🚨",
                    ricecoder_performance::OptimizationPriority::High => "⚠️",
                    ricecoder_performance::OptimizationPriority::Medium => "ℹ️",
                    ricecoder_performance::OptimizationPriority::Low => "📝",
                };
                report.push_str(&format!("{} {}: {} (Expected: +{:.1}%)\n",
                    priority_icon, suggestion.title, suggestion.description, suggestion.expected_improvement_percent));
            }

            match output {
                Some(path) => {
                    std::fs::write(&path, &report)?;
                    println!("✅ Optimization report saved to: {}", path.display());
                }
                None => {
                    println!("{}", report);
                }
            }
        }

        Commands::Monitor { binary, baseline, interval, slack_webhook, email_recipients } => {
            let baseline_data = PerformanceBaseline::load_from_file(baseline)?;

            // Configure alert destinations
            let mut destinations = vec![AlertDestination::Console];

            if let Some(webhook_url) = slack_webhook {
                destinations.push(AlertDestination::Slack { webhook_url });
            }

            if let Some(recipients_str) = email_recipients {
                // Note: In a real implementation, you'd need proper SMTP config
                let recipients: Vec<String> = recipients_str.split(',').map(|s| s.trim().to_string()).collect();
                destinations.push(AlertDestination::Email {
                    smtp_config: ricecoder_performance::SmtpConfig {
                        host: "smtp.example.com".to_string(),
                        port: 587,
                        username: "alerts@example.com".to_string(),
                        password: "password".to_string(),
                        use_tls: true,
                    },
                    recipients,
                });
            }

            let alert_config = AlertConfig {
                destinations,
                minimum_severity: AlertSeverity::Medium,
                cooldown_seconds: 3600, // 1 hour
            };

            let mut monitor = EnterpriseMonitor::new(alert_config);

            println!("🚀 Starting enterprise performance monitoring...");
            println!("📊 Monitoring interval: {} seconds", interval);
            println!("🎯 Press Ctrl+C to stop\n");

            loop {
                // Run performance validation
                let validator = PerformanceValidator::new(
                    binary.to_string_lossy().to_string(),
                    Some(baseline_data.clone()),
                );

                let results = validator.run_all_validations().await?;
                let metrics: Vec<_> = results.iter().map(|r| r.metrics.clone()).collect();

                // Monitor performance and check for alerts
                let alerts = monitor.monitor_performance(&metrics).await;

                if !alerts.is_empty() {
                    println!("🚨 {} alerts generated", alerts.len());
                }

                // Monitor validation results
                let validation_alerts = monitor.monitor_validation(&results).await;

                if !validation_alerts.is_empty() {
                    println!("❌ {} validation alerts generated", validation_alerts.len());
                }

                // Generate periodic report
                let report = monitor.generate_report();
                println!("📈 Performance Report:");
                println!("{}", report);

                tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
            }
        }

        Commands::Simulate { binary: _, duration, output } => {
            let simulator = EnterpriseSimulator::enterprise_scale();
            let simulation_duration = std::time::Duration::from_secs(duration);

            println!("🚀 Starting enterprise workload simulation for {:?}", simulation_duration);
            let result = simulator.run_simulation(simulation_duration).await?;

            // Generate and output report
            let report = result.generate_report();

            match output {
                Some(path) => {
                    std::fs::write(&path, &report)?;
                    println!("✅ Simulation report saved to: {}", path.display());
                }
                None => {
                    println!("{}", report);
                }
            }

            // Check if targets were met
            if result.meets_enterprise_targets() {
                println!("✅ Enterprise performance targets met!");
            } else {
                println!("❌ Enterprise performance targets not met. See report above.");
                std::process::exit(1);
            }
        }
    }

    Ok(())
}