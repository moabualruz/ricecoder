//! Performance baseline management and storage

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use chrono::{DateTime, Utc};
use crate::monitor::PerformanceMetrics;

/// Performance baseline data for a specific test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineData {
    /// Test name identifier
    pub test_name: String,
    /// Mean execution time in nanoseconds
    pub mean_time_ns: u64,
    /// Standard deviation in nanoseconds
    pub std_dev_ns: u64,
    /// 95th percentile time in nanoseconds
    pub p95_time_ns: u64,
    /// 99th percentile time in nanoseconds
    pub p99_time_ns: u64,
    /// Sample size used for baseline
    pub sample_size: usize,
    /// Timestamp when baseline was established
    pub timestamp: DateTime<Utc>,
    /// Target threshold for this baseline
    pub target_threshold_ns: Option<u64>,
}

/// Performance baseline collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBaseline {
    /// Baselines for different tests
    pub baselines: HashMap<String, BaselineData>,
    /// Version of the baseline format
    pub version: String,
    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
}

impl PerformanceBaseline {
    /// Create a new empty performance baseline
    pub fn new() -> Self {
        Self {
            baselines: HashMap::new(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            last_updated: Utc::now(),
        }
    }

    /// Load baseline from JSON file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let baseline: PerformanceBaseline = serde_json::from_str(&content)?;
        Ok(baseline)
    }

    /// Save baseline to JSON file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Add or update a baseline
    pub fn update_baseline(&mut self, test_name: String, metrics: &PerformanceMetrics) {
        let baseline_data = BaselineData {
            test_name: test_name.clone(),
            mean_time_ns: metrics.mean_time_ns,
            std_dev_ns: metrics.std_dev_ns,
            p95_time_ns: metrics.p95_time_ns,
            p99_time_ns: metrics.p99_time_ns,
            sample_size: metrics.sample_size,
            timestamp: Utc::now(),
            target_threshold_ns: self.get_target_threshold(&test_name),
        };

        self.baselines.insert(test_name, baseline_data);
        self.last_updated = Utc::now();
    }

    /// Get baseline for a specific test
    pub fn get_baseline(&self, test_name: &str) -> Option<&BaselineData> {
        self.baselines.get(test_name)
    }

    /// Get target threshold for a test name
    fn get_target_threshold(&self, test_name: &str) -> Option<u64> {
        match test_name {
            "cli_startup" => Some(3_000_000_000), // 3 seconds
            "response_time" => Some(500_000_000),  // 500ms
            "config_loading" => Some(500_000_000), // 500ms
            "provider_init" => Some(1_000_000_000), // 1 second
            "spec_parsing" => Some(1_000_000_000), // 1 second
            "file_operations" => Some(5_000_000_000), // 5 seconds
            _ => None,
        }
    }

    /// Check if current metrics exceed baseline thresholds
    pub fn check_thresholds(&self, test_name: &str, current_metrics: &PerformanceMetrics) -> Vec<String> {
        let mut violations = Vec::new();

        if let Some(baseline) = self.get_baseline(test_name) {
            // Check if current p95 exceeds baseline p95 by more than 10%
            let p95_threshold = baseline.p95_time_ns as f64 * 1.1;
            if current_metrics.p95_time_ns as f64 > p95_threshold {
                violations.push(format!(
                    "P95 time exceeded baseline: {:.2}ms > {:.2}ms",
                    current_metrics.p95_time_ns as f64 / 1_000_000.0,
                    baseline.p95_time_ns as f64 / 1_000_000.0
                ));
            }

            // Check target threshold if defined
            if let Some(target) = baseline.target_threshold_ns {
                if current_metrics.p95_time_ns > target {
                    violations.push(format!(
                        "Target threshold exceeded: {:.2}ms > {:.2}ms",
                        current_metrics.p95_time_ns as f64 / 1_000_000.0,
                        target as f64 / 1_000_000.0
                    ));
                }
            }
        }

        violations
    }
}

impl Default for PerformanceBaseline {
    fn default() -> Self {
        Self::new()
    }
}