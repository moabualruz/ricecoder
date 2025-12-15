//! Performance regression detection and alerting

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::baseline::{PerformanceBaseline, BaselineData};
use crate::monitor::PerformanceMetrics;

/// Regression alert types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegressionAlert {
    /// Performance degradation beyond threshold
    PerformanceDegradation {
        test_name: String,
        baseline_p95_ns: u64,
        current_p95_ns: u64,
        degradation_percent: f64,
        threshold_percent: f64,
    },
    /// Memory usage increase
    MemoryRegression {
        test_name: String,
        baseline_memory_bytes: u64,
        current_memory_bytes: u64,
        increase_percent: f64,
    },
    /// Target threshold exceeded
    TargetExceeded {
        test_name: String,
        target_ns: u64,
        current_p95_ns: u64,
        exceed_percent: f64,
    },
}

/// Regression detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionConfig {
    /// Performance degradation threshold (percentage)
    pub performance_threshold_percent: f64,
    /// Memory increase threshold (percentage)
    pub memory_threshold_percent: f64,
    /// Whether to enable alerting
    pub enable_alerting: bool,
    /// Alert cooldown period in seconds
    pub alert_cooldown_seconds: u64,
}

impl Default for RegressionConfig {
    fn default() -> Self {
        Self {
            performance_threshold_percent: 10.0, // 10% degradation
            memory_threshold_percent: 20.0,      // 20% memory increase
            enable_alerting: true,
            alert_cooldown_seconds: 3600,        // 1 hour
        }
    }
}

/// Regression detector
pub struct RegressionDetector {
    baseline: PerformanceBaseline,
    config: RegressionConfig,
    last_alert_times: std::collections::HashMap<String, DateTime<Utc>>,
}

impl RegressionDetector {
    /// Create a new regression detector
    pub fn new(baseline: PerformanceBaseline, config: RegressionConfig) -> Self {
        Self {
            baseline,
            config,
            last_alert_times: std::collections::HashMap::new(),
        }
    }

    /// Detect regressions in current metrics
    pub fn detect_regressions(&mut self, metrics: &[PerformanceMetrics]) -> Vec<RegressionAlert> {
        let mut alerts = Vec::new();
        let now = Utc::now();

        for metric in metrics {
            // Check for performance regression
            if let Some(baseline_data) = self.baseline.get_baseline(&metric.test_name) {
                let alerts_for_test = self.check_performance_regression(baseline_data, metric, now);
                alerts.extend(alerts_for_test);
            }

            // Check for memory regression
            if let Some(baseline_data) = self.baseline.get_baseline(&metric.test_name) {
                if let Some(alert) = self.check_memory_regression(baseline_data, metric, now) {
                    alerts.push(alert);
                }
            }

            // Check target thresholds
            if let Some(baseline_data) = self.baseline.get_baseline(&metric.test_name) {
                if let Some(target) = baseline_data.target_threshold_ns {
                    if let Some(alert) = self.check_target_exceeded(&metric.test_name, target, metric, now) {
                        alerts.push(alert);
                    }
                }
            }
        }

        alerts
    }

    /// Check for performance regression
    fn check_performance_regression(
        &mut self,
        baseline: &BaselineData,
        current: &PerformanceMetrics,
        now: DateTime<Utc>,
    ) -> Vec<RegressionAlert> {
        let mut alerts = Vec::new();

        // Check if we're within cooldown period
        if let Some(last_alert) = self.last_alert_times.get(&format!("perf_{}", current.test_name)) {
            if (now - *last_alert).num_seconds() < self.config.alert_cooldown_seconds as i64 {
                return alerts;
            }
        }

        let degradation_percent = ((current.p95_time_ns as f64 - baseline.p95_time_ns as f64)
            / baseline.p95_time_ns as f64) * 100.0;

        if degradation_percent > self.config.performance_threshold_percent {
            self.last_alert_times.insert(format!("perf_{}", current.test_name), now);
            alerts.push(RegressionAlert::PerformanceDegradation {
                test_name: current.test_name.clone(),
                baseline_p95_ns: baseline.p95_time_ns,
                current_p95_ns: current.p95_time_ns,
                degradation_percent,
                threshold_percent: self.config.performance_threshold_percent,
            });
        }

        alerts
    }

    /// Check for memory regression
    fn check_memory_regression(
        &mut self,
        baseline: &BaselineData,
        current: &PerformanceMetrics,
        now: DateTime<Utc>,
    ) -> Option<RegressionAlert> {
        // Check if we're within cooldown period
        if let Some(last_alert) = self.last_alert_times.get(&format!("mem_{}", current.test_name)) {
            if (now - *last_alert).num_seconds() < self.config.alert_cooldown_seconds as i64 {
                return None;
            }
        }

        // Calculate memory baseline (simplified - using a fixed baseline for now)
        let baseline_memory = 50 * 1024 * 1024; // 50MB baseline
        let increase_percent = ((current.peak_memory_bytes as f64 - baseline_memory as f64)
            / baseline_memory as f64) * 100.0;

        if increase_percent > self.config.memory_threshold_percent {
            self.last_alert_times.insert(format!("mem_{}", current.test_name), now);
            return Some(RegressionAlert::MemoryRegression {
                test_name: current.test_name.clone(),
                baseline_memory_bytes: baseline_memory,
                current_memory_bytes: current.peak_memory_bytes,
                increase_percent,
            });
        }

        None
    }

    /// Check if target threshold is exceeded
    fn check_target_exceeded(
        &mut self,
        test_name: &str,
        target_ns: u64,
        current: &PerformanceMetrics,
        now: DateTime<Utc>,
    ) -> Option<RegressionAlert> {
        // Check if we're within cooldown period
        if let Some(last_alert) = self.last_alert_times.get(&format!("target_{}", test_name)) {
            if (now - *last_alert).num_seconds() < self.config.alert_cooldown_seconds as i64 {
                return None;
            }
        }

        if current.p95_time_ns > target_ns {
            let exceed_percent = ((current.p95_time_ns as f64 - target_ns as f64)
                / target_ns as f64) * 100.0;

            self.last_alert_times.insert(format!("target_{}", test_name), now);
            return Some(RegressionAlert::TargetExceeded {
                test_name: test_name.to_string(),
                target_ns,
                current_p95_ns: current.p95_time_ns,
                exceed_percent,
            });
        }

        None
    }

    /// Update baseline with new measurements
    pub fn update_baseline(&mut self, metrics: &[PerformanceMetrics]) {
        for metric in metrics {
            self.baseline.update_baseline(metric.test_name.clone(), metric);
        }
    }

    /// Get current baseline
    pub fn baseline(&self) -> &PerformanceBaseline {
        &self.baseline
    }

    /// Get configuration
    pub fn config(&self) -> &RegressionConfig {
        &self.config
    }
}