//! Performance profiling utilities for detailed analysis

use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};

use crate::monitor::{PerformanceMetrics, PerformanceMonitor};

/// Performance profiler for detailed code path analysis
pub struct PerformanceProfiler {
    monitors: HashMap<String, PerformanceMonitor>,
    call_stacks: Vec<String>,
    profile_start: Option<Instant>,
}

impl PerformanceProfiler {
    /// Create a new performance profiler
    pub fn new() -> Self {
        Self {
            monitors: HashMap::new(),
            call_stacks: Vec::new(),
            profile_start: None,
        }
    }

    /// Start profiling session
    pub fn start_profiling(&mut self) {
        self.profile_start = Some(Instant::now());
    }

    /// Stop profiling session and return results
    pub fn stop_profiling(&mut self) -> ProfileResult {
        let total_duration = self
            .profile_start
            .map(|start| start.elapsed())
            .unwrap_or(Duration::from_secs(0));

        let mut metrics = HashMap::new();
        for (name, monitor) in &self.monitors {
            metrics.insert(name.clone(), monitor.get_metrics());
        }

        ProfileResult {
            total_duration,
            metrics,
            call_count: self.call_stacks.len(),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Start timing a specific code path
    pub fn start_timing(&mut self, path_name: &str) {
        let monitor = self
            .monitors
            .entry(path_name.to_string())
            .or_insert_with(|| PerformanceMonitor::new(path_name.to_string()));

        monitor.start();
        self.call_stacks.push(path_name.to_string());
    }

    /// Stop timing the current code path
    pub fn stop_timing(&mut self, path_name: &str) {
        if let Some(monitor) = self.monitors.get_mut(path_name) {
            monitor.stop();
        }

        // Remove from call stack
        if let Some(pos) = self.call_stacks.iter().rposition(|x| x == path_name) {
            self.call_stacks.remove(pos);
        }
    }

    /// Profile a function with automatic timing
    pub fn profile_function<F, R>(&mut self, path_name: &str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        self.start_timing(path_name);
        let result = f();
        self.stop_timing(path_name);
        result
    }

    /// Get current call stack
    pub fn current_call_stack(&self) -> &[String] {
        &self.call_stacks
    }

    /// Clear all profiling data
    pub fn clear(&mut self) {
        self.monitors.clear();
        self.call_stacks.clear();
        self.profile_start = None;
    }
}

/// Profiling result containing detailed performance data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileResult {
    /// Total profiling duration
    pub total_duration: Duration,
    /// Metrics for each profiled path
    pub metrics: HashMap<String, PerformanceMetrics>,
    /// Total number of function calls
    pub call_count: usize,
    /// Timestamp of profiling session
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ProfileResult {
    /// Get the slowest performing path
    pub fn slowest_path(&self) -> Option<(&String, &PerformanceMetrics)> {
        self.metrics
            .iter()
            .max_by_key(|(_, metrics)| metrics.p95_time_ns)
    }

    /// Get the most frequently called path
    pub fn most_frequent_path(&self) -> Option<(&String, &PerformanceMetrics)> {
        self.metrics
            .iter()
            .max_by_key(|(_, metrics)| metrics.sample_size)
    }

    /// Get paths exceeding a time threshold
    pub fn paths_exceeding_threshold(
        &self,
        threshold_ns: u64,
    ) -> Vec<(&String, &PerformanceMetrics)> {
        self.metrics
            .iter()
            .filter(|(_, metrics)| metrics.p95_time_ns > threshold_ns)
            .collect()
    }

    /// Generate a performance report
    pub fn generate_report(&self) -> String {
        let mut report = format!("=== Performance Profile Report ===\n");
        report.push_str(&format!(
            "Total Duration: {:.2}s\n",
            self.total_duration.as_secs_f64()
        ));
        report.push_str(&format!("Total Calls: {}\n", self.call_count));
        report.push_str(&format!("Timestamp: {}\n\n", self.timestamp.to_rfc3339()));

        report.push_str("=== Path Performance ===\n");
        let mut sorted_paths: Vec<_> = self.metrics.iter().collect();
        sorted_paths.sort_by(|a, b| b.1.p95_time_ns.cmp(&a.1.p95_time_ns));

        for (path, metrics) in sorted_paths {
            report.push_str(&format!(
                "{}: P95={:.2}ms, Mean={:.2}ms, Samples={}\n",
                path,
                metrics.p95_time_ns as f64 / 1_000_000.0,
                metrics.mean_time_ns as f64 / 1_000_000.0,
                metrics.sample_size
            ));
        }

        if let Some((path, metrics)) = self.slowest_path() {
            report.push_str(&format!(
                "\nSlowest Path: {} ({:.2}ms P95)\n",
                path,
                metrics.p95_time_ns as f64 / 1_000_000.0
            ));
        }

        report
    }
}

/// RAII-style profiler timer
pub struct ProfilerTimer<'a> {
    profiler: &'a mut PerformanceProfiler,
    path_name: String,
}

impl<'a> ProfilerTimer<'a> {
    /// Create a new profiler timer
    pub fn new(profiler: &'a mut PerformanceProfiler, path_name: impl Into<String>) -> Self {
        let path_name = path_name.into();
        profiler.start_timing(&path_name);
        Self {
            profiler,
            path_name,
        }
    }
}

impl<'a> Drop for ProfilerTimer<'a> {
    fn drop(&mut self) {
        self.profiler.stop_timing(&self.path_name);
    }
}
