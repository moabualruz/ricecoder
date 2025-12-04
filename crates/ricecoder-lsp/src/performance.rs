//! Performance monitoring and optimization utilities
//!
//! This module provides performance tracking, metrics collection, and optimization
//! utilities for the LSP server.

use std::time::{Instant, Duration};
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use tracing::{info, warn};

/// Performance metrics for a specific operation
#[derive(Debug, Clone)]
pub struct OperationMetrics {
    /// Operation name
    pub name: String,
    /// Total number of operations
    pub count: u64,
    /// Total time spent in milliseconds
    pub total_time_ms: f64,
    /// Minimum time in milliseconds
    pub min_time_ms: f64,
    /// Maximum time in milliseconds
    pub max_time_ms: f64,
}

impl OperationMetrics {
    /// Calculate average time in milliseconds
    pub fn avg_time_ms(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.total_time_ms / self.count as f64
        }
    }
}

/// Performance tracker for measuring operation times
pub struct PerformanceTracker {
    /// Metrics by operation name
    metrics: Arc<RwLock<HashMap<String, OperationMetrics>>>,
    /// Performance targets (operation name -> max time in ms)
    targets: Arc<RwLock<HashMap<String, f64>>>,
}

impl PerformanceTracker {
    /// Create a new performance tracker
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            targets: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Set performance target for an operation
    pub fn set_target(&self, operation: String, max_time_ms: f64) {
        let mut targets = self.targets.write().unwrap();
        targets.insert(operation, max_time_ms);
    }

    /// Record operation time
    pub fn record(&self, operation: String, duration: Duration) {
        let time_ms = duration.as_secs_f64() * 1000.0;
        
        let mut metrics = self.metrics.write().unwrap();
        let entry = metrics.entry(operation.clone())
            .or_insert_with(|| OperationMetrics {
                name: operation.clone(),
                count: 0,
                total_time_ms: 0.0,
                min_time_ms: f64::MAX,
                max_time_ms: 0.0,
            });
        
        entry.count += 1;
        entry.total_time_ms += time_ms;
        entry.min_time_ms = entry.min_time_ms.min(time_ms);
        entry.max_time_ms = entry.max_time_ms.max(time_ms);
        
        // Check if target was exceeded
        let targets = self.targets.read().unwrap();
        if let Some(&target) = targets.get(&operation) {
            if time_ms > target {
                warn!(
                    "Performance target exceeded for {}: {:.2}ms > {:.2}ms",
                    operation, time_ms, target
                );
            }
        }
    }

    /// Get metrics for an operation
    pub fn get_metrics(&self, operation: &str) -> Option<OperationMetrics> {
        let metrics = self.metrics.read().unwrap();
        metrics.get(operation).cloned()
    }

    /// Get all metrics
    pub fn all_metrics(&self) -> Vec<OperationMetrics> {
        let metrics = self.metrics.read().unwrap();
        metrics.values().cloned().collect()
    }

    /// Clear all metrics
    pub fn clear(&self) {
        let mut metrics = self.metrics.write().unwrap();
        metrics.clear();
    }

    /// Print performance report
    pub fn print_report(&self) {
        let metrics = self.metrics.read().unwrap();
        
        if metrics.is_empty() {
            info!("No performance metrics recorded");
            return;
        }
        
        info!("=== Performance Report ===");
        for (_, metric) in metrics.iter() {
            info!(
                "{}: count={}, avg={:.2}ms, min={:.2}ms, max={:.2}ms",
                metric.name,
                metric.count,
                metric.avg_time_ms(),
                metric.min_time_ms,
                metric.max_time_ms
            );
        }
    }
}

impl Default for PerformanceTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Timer for measuring operation duration
pub struct Timer {
    start: Instant,
    operation: String,
    tracker: Arc<PerformanceTracker>,
}

impl Timer {
    /// Create a new timer
    pub fn new(operation: String, tracker: Arc<PerformanceTracker>) -> Self {
        Self {
            start: Instant::now(),
            operation,
            tracker,
        }
    }

    /// Stop the timer and record the duration
    pub fn stop(self) {
        let duration = self.start.elapsed();
        let operation = self.operation.clone();
        self.tracker.record(operation, duration);
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        let operation = self.operation.clone();
        self.tracker.record(operation, duration);
    }
}

/// Performance analyzer for identifying bottlenecks
pub struct PerformanceAnalyzer {
    tracker: Arc<PerformanceTracker>,
}

impl PerformanceAnalyzer {
    /// Create a new performance analyzer
    pub fn new(tracker: Arc<PerformanceTracker>) -> Self {
        Self { tracker }
    }

    /// Identify slow operations (operations exceeding their targets)
    pub fn identify_slow_operations(&self) -> Vec<(String, f64, f64)> {
        let metrics = self.tracker.metrics.read().unwrap();
        let targets = self.tracker.targets.read().unwrap();
        
        let mut slow_ops = Vec::new();
        
        for (name, metric) in metrics.iter() {
            if let Some(&target) = targets.get(name) {
                let avg = metric.avg_time_ms();
                if avg > target {
                    slow_ops.push((name.clone(), avg, target));
                }
            }
        }
        
        slow_ops.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        slow_ops
    }

    /// Get operations sorted by average time (slowest first)
    pub fn slowest_operations(&self, limit: usize) -> Vec<OperationMetrics> {
        let mut metrics = self.tracker.all_metrics();
        metrics.sort_by(|a, b| b.avg_time_ms().partial_cmp(&a.avg_time_ms()).unwrap());
        metrics.into_iter().take(limit).collect()
    }

    /// Get operations sorted by total time (most time spent)
    pub fn most_time_spent(&self, limit: usize) -> Vec<OperationMetrics> {
        let mut metrics = self.tracker.all_metrics();
        metrics.sort_by(|a, b| b.total_time_ms.partial_cmp(&a.total_time_ms).unwrap());
        metrics.into_iter().take(limit).collect()
    }
}

/// Performance optimization recommendations
#[derive(Debug, Clone)]
pub struct OptimizationRecommendation {
    /// Operation name
    pub operation: String,
    /// Current average time in milliseconds
    pub current_time_ms: f64,
    /// Target time in milliseconds
    pub target_time_ms: f64,
    /// Recommendation message
    pub recommendation: String,
}

impl OptimizationRecommendation {
    /// Calculate improvement needed as percentage
    pub fn improvement_needed(&self) -> f64 {
        if self.target_time_ms == 0.0 {
            0.0
        } else {
            ((self.current_time_ms - self.target_time_ms) / self.target_time_ms) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_performance_tracker_record() {
        let tracker = PerformanceTracker::new();
        let duration = Duration::from_millis(100);
        
        tracker.record("test_op".to_string(), duration);
        
        let metrics = tracker.get_metrics("test_op").unwrap();
        assert_eq!(metrics.count, 1);
        assert!(metrics.total_time_ms >= 100.0);
    }

    #[test]
    fn test_performance_tracker_multiple_records() {
        let tracker = PerformanceTracker::new();
        
        tracker.record("test_op".to_string(), Duration::from_millis(100));
        tracker.record("test_op".to_string(), Duration::from_millis(200));
        tracker.record("test_op".to_string(), Duration::from_millis(150));
        
        let metrics = tracker.get_metrics("test_op").unwrap();
        assert_eq!(metrics.count, 3);
        assert!(metrics.avg_time_ms() >= 150.0);
        assert!(metrics.min_time_ms >= 100.0);
        assert!(metrics.max_time_ms >= 200.0);
    }

    #[test]
    fn test_timer_auto_record() {
        let tracker = Arc::new(PerformanceTracker::new());
        
        {
            let _timer = Timer::new("test_op".to_string(), tracker.clone());
            thread::sleep(Duration::from_millis(50));
        }
        
        let metrics = tracker.get_metrics("test_op").unwrap();
        assert_eq!(metrics.count, 1);
        assert!(metrics.total_time_ms >= 50.0);
    }

    #[test]
    fn test_performance_target() {
        let tracker = PerformanceTracker::new();
        tracker.set_target("test_op".to_string(), 100.0);
        
        tracker.record("test_op".to_string(), Duration::from_millis(50));
        
        let metrics = tracker.get_metrics("test_op").unwrap();
        assert_eq!(metrics.count, 1);
    }

    #[test]
    fn test_performance_analyzer_slowest() {
        let tracker = Arc::new(PerformanceTracker::new());
        
        tracker.record("op1".to_string(), Duration::from_millis(100));
        tracker.record("op2".to_string(), Duration::from_millis(200));
        tracker.record("op3".to_string(), Duration::from_millis(150));
        
        let analyzer = PerformanceAnalyzer::new(tracker);
        let slowest = analyzer.slowest_operations(2);
        
        assert_eq!(slowest.len(), 2);
        assert_eq!(slowest[0].name, "op2");
        assert_eq!(slowest[1].name, "op3");
    }

    #[test]
    fn test_optimization_recommendation() {
        let rec = OptimizationRecommendation {
            operation: "test_op".to_string(),
            current_time_ms: 200.0,
            target_time_ms: 100.0,
            recommendation: "Optimize parsing".to_string(),
        };
        
        assert_eq!(rec.improvement_needed(), 100.0);
    }
}
