//! Performance monitoring and metrics collection

use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Performance metrics for a test run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Test name
    pub test_name: String,
    /// Mean execution time in nanoseconds
    pub mean_time_ns: u64,
    /// Standard deviation in nanoseconds
    pub std_dev_ns: u64,
    /// 95th percentile time in nanoseconds
    pub p95_time_ns: u64,
    /// 99th percentile time in nanoseconds
    pub p99_time_ns: u64,
    /// Sample size
    pub sample_size: usize,
    /// Memory usage in bytes (peak)
    pub peak_memory_bytes: u64,
    /// CPU usage percentage (average)
    pub avg_cpu_percent: f64,
    /// Timestamp of measurement
    pub timestamp: DateTime<Utc>,
}

/// Performance monitor for collecting metrics
pub struct PerformanceMonitor {
    test_name: String,
    measurements: VecDeque<Duration>,
    max_samples: usize,
    start_time: Option<Instant>,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new(test_name: String) -> Self {
        Self {
            test_name,
            measurements: VecDeque::new(),
            max_samples: 1000,
            start_time: None,
        }
    }

    /// Start timing an operation
    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
    }

    /// Stop timing and record the measurement
    pub fn stop(&mut self) {
        if let Some(start) = self.start_time.take() {
            let duration = start.elapsed();
            self.record_measurement(duration);
        }
    }

    /// Record a manual measurement
    pub fn record_measurement(&mut self, duration: Duration) {
        self.measurements.push_back(duration);

        // Maintain max samples
        if self.measurements.len() > self.max_samples {
            self.measurements.pop_front();
        }
    }

    /// Get current performance metrics
    pub fn get_metrics(&self) -> PerformanceMetrics {
        let sample_size = self.measurements.len();
        if sample_size == 0 {
            return PerformanceMetrics {
                test_name: self.test_name.clone(),
                mean_time_ns: 0,
                std_dev_ns: 0,
                p95_time_ns: 0,
                p99_time_ns: 0,
                sample_size: 0,
                peak_memory_bytes: 0,
                avg_cpu_percent: 0.0,
                timestamp: Utc::now(),
            };
        }

        // Convert durations to nanoseconds
        let mut times_ns: Vec<u64> = self
            .measurements
            .iter()
            .map(|d| d.as_nanos() as u64)
            .collect();
        times_ns.sort();

        // Calculate statistics
        let mean_time_ns = times_ns.iter().sum::<u64>() / sample_size as u64;

        let variance = times_ns
            .iter()
            .map(|t| {
                let diff = *t as i64 - mean_time_ns as i64;
                (diff * diff) as u64
            })
            .sum::<u64>()
            / sample_size as u64;
        let std_dev_ns = (variance as f64).sqrt() as u64;

        let p95_index = (sample_size as f64 * 0.95) as usize;
        let p99_index = (sample_size as f64 * 0.99) as usize;

        let p95_time_ns = times_ns
            .get(p95_index)
            .copied()
            .unwrap_or(*times_ns.last().unwrap());
        let p99_time_ns = times_ns
            .get(p99_index)
            .copied()
            .unwrap_or(*times_ns.last().unwrap());

        PerformanceMetrics {
            test_name: self.test_name.clone(),
            mean_time_ns,
            std_dev_ns,
            p95_time_ns,
            p99_time_ns,
            sample_size,
            peak_memory_bytes: self.get_memory_usage(),
            avg_cpu_percent: self.get_cpu_usage(),
            timestamp: Utc::now(),
        }
    }

    /// Clear all measurements
    pub fn clear(&mut self) {
        self.measurements.clear();
        self.start_time = None;
    }

    /// Get current memory usage (simplified)
    fn get_memory_usage(&self) -> u64 {
        // In a real implementation, this would use system APIs
        // For now, return a placeholder
        50 * 1024 * 1024 // 50MB placeholder
    }

    /// Get current CPU usage (simplified)
    fn get_cpu_usage(&self) -> f64 {
        // In a real implementation, this would use system APIs
        // For now, return a placeholder
        15.0 // 15% placeholder
    }
}

/// RAII-style performance timer
pub struct PerformanceTimer<'a> {
    monitor: &'a mut PerformanceMonitor,
}

impl<'a> PerformanceTimer<'a> {
    /// Create a new performance timer
    pub fn new(monitor: &'a mut PerformanceMonitor) -> Self {
        monitor.start();
        Self { monitor }
    }
}

impl<'a> Drop for PerformanceTimer<'a> {
    fn drop(&mut self) {
        self.monitor.stop();
    }
}
