//! Performance validation utilities

use crate::baseline::PerformanceBaseline;
use crate::monitor::{PerformanceMetrics, PerformanceMonitor};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::process::Command;

/// Validation result for a performance test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Test name
    pub test_name: String,
    /// Whether validation passed
    pub passed: bool,
    /// Actual metrics
    pub metrics: PerformanceMetrics,
    /// Baseline metrics (if available)
    pub baseline: Option<PerformanceMetrics>,
    /// Validation errors or warnings
    pub messages: Vec<String>,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Performance validator for running validation tests
pub struct PerformanceValidator {
    binary_path: String,
    baseline: Option<PerformanceBaseline>,
}

impl PerformanceValidator {
    /// Create a new performance validator
    pub fn new(binary_path: String, baseline: Option<PerformanceBaseline>) -> Self {
        Self {
            binary_path,
            baseline,
        }
    }

    /// Validate CLI startup time (< 3 seconds)
    pub async fn validate_startup_time(
        &self,
    ) -> Result<ValidationResult, Box<dyn std::error::Error>> {
        let mut monitor = PerformanceMonitor::new("cli_startup".to_string());

        // Run multiple iterations
        for _ in 0..10 {
            monitor.start();

            let result = Command::new(&self.binary_path)
                .arg("--version")
                .output()
                .await?;

            monitor.stop();

            if !result.status.success() {
                return Err(format!("CLI command failed: {:?}", result.status).into());
            }
        }

        let metrics = monitor.get_metrics();
        let baseline_metrics = self
            .baseline
            .as_ref()
            .and_then(|b| b.get_baseline("cli_startup"))
            .map(|b| PerformanceMetrics {
                test_name: b.test_name.clone(),
                mean_time_ns: b.mean_time_ns,
                std_dev_ns: b.std_dev_ns,
                p95_time_ns: b.p95_time_ns,
                p99_time_ns: b.p99_time_ns,
                sample_size: b.sample_size,
                peak_memory_bytes: 0, // Not stored in baseline
                avg_cpu_percent: 0.0, // Not stored in baseline
                timestamp: b.timestamp,
            });

        let target_ns = 3_000_000_000; // 3 seconds
        let passed = metrics.p95_time_ns <= target_ns;

        let mut messages = Vec::new();
        if !passed {
            messages.push(format!(
                "Startup time exceeded target: {:.2}s > 3.0s",
                metrics.p95_time_ns as f64 / 1_000_000_000.0
            ));
        }

        // Check against baseline
        if let Some(baseline) = &baseline_metrics {
            let degradation = ((metrics.p95_time_ns as f64 - baseline.p95_time_ns as f64)
                / baseline.p95_time_ns as f64)
                * 100.0;
            if degradation > 10.0 {
                messages.push(format!(
                    "Performance regression: {:.1}% slower than baseline",
                    degradation
                ));
            }
        }

        Ok(ValidationResult {
            test_name: "cli_startup".to_string(),
            passed,
            metrics,
            baseline: baseline_metrics,
            messages,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Validate response time (< 500ms for typical operations)
    pub async fn validate_response_time(
        &self,
    ) -> Result<ValidationResult, Box<dyn std::error::Error>> {
        let mut monitor = PerformanceMonitor::new("response_time".to_string());

        // Test help command as a typical operation
        for _ in 0..20 {
            monitor.start();

            let result = Command::new(&self.binary_path)
                .arg("--help")
                .output()
                .await?;

            monitor.stop();

            if !result.status.success() {
                return Err(format!("Help command failed: {:?}", result.status).into());
            }
        }

        let metrics = monitor.get_metrics();
        let target_ns = 500_000_000; // 500ms
        let passed = metrics.p95_time_ns <= target_ns;

        let mut messages = Vec::new();
        if !passed {
            messages.push(format!(
                "Response time exceeded target: {:.2}ms > 500ms",
                metrics.p95_time_ns as f64 / 1_000_000.0
            ));
        }

        Ok(ValidationResult {
            test_name: "response_time".to_string(),
            passed,
            metrics,
            baseline: None, // Could be added later
            messages,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Validate memory usage (< 300MB)
    pub async fn validate_memory_usage(
        &self,
    ) -> Result<ValidationResult, Box<dyn std::error::Error>> {
        // This is a simplified memory validation
        // In a real implementation, you'd use system monitoring tools

        let mut monitor = PerformanceMonitor::new("memory_usage".to_string());

        // Simulate memory measurement during operation
        monitor.start();

        let result = Command::new(&self.binary_path)
            .arg("--help")
            .output()
            .await?;

        monitor.stop();

        if !result.status.success() {
            return Err(format!("Command failed: {:?}", result.status).into());
        }

        let metrics = monitor.get_metrics();
        let target_bytes = 300 * 1024 * 1024; // 300MB
        let passed = metrics.peak_memory_bytes <= target_bytes;

        let mut messages = Vec::new();
        if !passed {
            messages.push(format!(
                "Memory usage exceeded target: {:.1}MB > 300MB",
                metrics.peak_memory_bytes as f64 / (1024.0 * 1024.0)
            ));
        }

        Ok(ValidationResult {
            test_name: "memory_usage".to_string(),
            passed,
            metrics,
            baseline: None,
            messages,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Run all validations
    pub async fn run_all_validations(
        &self,
    ) -> Result<Vec<ValidationResult>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();

        results.push(self.validate_startup_time().await?);
        results.push(self.validate_response_time().await?);
        results.push(self.validate_memory_usage().await?);

        Ok(results)
    }
}
