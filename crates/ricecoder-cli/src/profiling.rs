/// Performance profiling utilities for ricecoder CLI
///
/// This module provides utilities for profiling and measuring performance of CLI operations.
/// It includes timing measurements, memory tracking, and performance reporting.

use std::time::{Duration, Instant};
use std::collections::HashMap;
use tracing::{info, debug};

/// Performance metrics for a single operation
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Operation name
    pub name: String,
    /// Total duration
    pub duration: Duration,
    /// Number of iterations
    pub iterations: u64,
    /// Average duration per iteration
    pub avg_duration: Duration,
    /// Peak memory usage (if available)
    pub peak_memory: Option<u64>,
}

impl PerformanceMetrics {
    /// Create new performance metrics
    pub fn new(name: String, duration: Duration, iterations: u64) -> Self {
        let avg_duration = if iterations > 0 {
            Duration::from_nanos(duration.as_nanos() as u64 / iterations)
        } else {
            Duration::ZERO
        };

        Self {
            name,
            duration,
            iterations,
            avg_duration,
            peak_memory: None,
        }
    }

    /// Set peak memory usage
    pub fn with_peak_memory(mut self, peak_memory: u64) -> Self {
        self.peak_memory = Some(peak_memory);
        self
    }

    /// Check if performance meets target
    pub fn meets_target(&self, target: Duration) -> bool {
        self.avg_duration <= target
    }

    /// Format metrics as string
    pub fn format(&self) -> String {
        let mut result = format!(
            "{}: {:.2}ms avg ({:.2}ms total, {} iterations)",
            self.name,
            self.avg_duration.as_secs_f64() * 1000.0,
            self.duration.as_secs_f64() * 1000.0,
            self.iterations
        );

        if let Some(peak_mem) = self.peak_memory {
            result.push_str(&format!(", peak memory: {:.2}MB", peak_mem as f64 / 1024.0 / 1024.0));
        }

        result
    }
}

/// Performance profiler for measuring operation timing
pub struct PerformanceProfiler {
    /// Recorded metrics
    metrics: HashMap<String, Vec<PerformanceMetrics>>,
}

impl PerformanceProfiler {
    /// Create new profiler
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
        }
    }

    /// Record a single operation timing
    pub fn record(&mut self, name: String, duration: Duration) {
        let metrics = PerformanceMetrics::new(name.clone(), duration, 1);
        self.metrics.entry(name).or_insert_with(Vec::new).push(metrics);
    }

    /// Record multiple iterations
    pub fn record_iterations(&mut self, name: String, total_duration: Duration, iterations: u64) {
        let metrics = PerformanceMetrics::new(name.clone(), total_duration, iterations);
        self.metrics.entry(name).or_insert_with(Vec::new).push(metrics);
    }

    /// Get all recorded metrics
    pub fn metrics(&self) -> &HashMap<String, Vec<PerformanceMetrics>> {
        &self.metrics
    }

    /// Get average metrics for an operation
    pub fn average_metrics(&self, name: &str) -> Option<PerformanceMetrics> {
        let metrics = self.metrics.get(name)?;
        if metrics.is_empty() {
            return None;
        }

        let total_duration: Duration = metrics.iter().map(|m| m.duration).sum();
        let total_iterations: u64 = metrics.iter().map(|m| m.iterations).sum();
        let peak_memory = metrics.iter().filter_map(|m| m.peak_memory).max();

        let mut avg = PerformanceMetrics::new(name.to_string(), total_duration, total_iterations);
        if let Some(peak_mem) = peak_memory {
            avg = avg.with_peak_memory(peak_mem);
        }

        Some(avg)
    }

    /// Print all metrics
    pub fn print_summary(&self) {
        info!("=== Performance Summary ===");
        for (name, metrics) in &self.metrics {
            if let Some(avg) = self.average_metrics(name) {
                info!("{}", avg.format());
            }
        }
    }

    /// Clear all metrics
    pub fn clear(&mut self) {
        self.metrics.clear();
    }
}

impl Default for PerformanceProfiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Timer for measuring operation duration
pub struct Timer {
    start: Instant,
    name: String,
}

impl Timer {
    /// Create new timer
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        debug!("Starting timer: {}", name);
        Self {
            start: Instant::now(),
            name,
        }
    }

    /// Get elapsed duration
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Get elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> f64 {
        self.elapsed().as_secs_f64() * 1000.0
    }

    /// Log elapsed time and return duration
    pub fn stop(self) -> Duration {
        let elapsed = self.elapsed();
        info!("{}: {:.2}ms", self.name, elapsed.as_secs_f64() * 1000.0);
        elapsed
    }

    /// Stop and check if within target
    pub fn stop_with_target(self, target: Duration) -> (Duration, bool) {
        let elapsed = self.elapsed();
        let meets_target = elapsed <= target;
        let status = if meets_target { "✓" } else { "✗" };
        info!(
            "{}: {:.2}ms {} (target: {:.2}ms)",
            self.name,
            elapsed.as_secs_f64() * 1000.0,
            status,
            target.as_secs_f64() * 1000.0
        );
        (elapsed, meets_target)
    }
}


