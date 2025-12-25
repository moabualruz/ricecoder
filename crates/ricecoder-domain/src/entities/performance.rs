//! Performance metrics and benchmarking entities

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Performance metric for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetric {
    pub id: String,
    pub name: String,
    pub value: f64,
    pub unit: MetricUnit,
    pub timestamp: DateTime<Utc>,
    pub context: HashMap<String, String>,
}

impl PerformanceMetric {
    /// Create a new performance metric
    pub fn new(name: String, value: f64, unit: MetricUnit) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            value,
            unit,
            timestamp: Utc::now(),
            context: HashMap::new(),
        }
    }

    /// Add context
    pub fn add_context(mut self, key: String, value: String) -> Self {
        self.context.insert(key, value);
        self
    }
}

/// Units for performance metrics
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MetricUnit {
    Milliseconds,
    Seconds,
    Bytes,
    Kilobytes,
    Megabytes,
    Count,
    Percentage,
}

/// Performance benchmark
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBenchmark {
    pub id: String,
    pub name: String,
    pub target_value: f64,
    pub tolerance: f64,
    pub unit: MetricUnit,
    pub description: String,
}

impl PerformanceBenchmark {
    /// Create a new benchmark
    pub fn new(
        name: String,
        target_value: f64,
        tolerance: f64,
        unit: MetricUnit,
        description: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            target_value,
            tolerance,
            unit,
            description,
        }
    }

    /// Check if a metric passes this benchmark
    pub fn check(&self, metric: &PerformanceMetric) -> BenchmarkResult {
        if metric.unit != self.unit {
            return BenchmarkResult::Error("Unit mismatch".to_string());
        }

        let deviation = (metric.value - self.target_value).abs() / self.target_value;
        if deviation <= self.tolerance {
            BenchmarkResult::Pass
        } else if deviation <= self.tolerance * 2.0 {
            BenchmarkResult::Warning
        } else {
            BenchmarkResult::Fail
        }
    }
}

/// Benchmark check result
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BenchmarkResult {
    Pass,
    Warning,
    Fail,
    Error(String),
}
