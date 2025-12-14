//! Performance monitoring and metrics collection

use crate::error::{ActivityLogError, ActivityLogResult};
use crate::events::{ActivityEvent, EventCategory, LogLevel};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Performance metrics for monitoring system health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Total events processed
    pub total_events: u64,
    /// Events processed per second
    pub events_per_second: f64,
    /// Average processing time per event in microseconds
    pub avg_processing_time_us: f64,
    /// Peak events per second
    pub peak_events_per_second: f64,
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
    /// Error rate (0.0 to 1.0)
    pub error_rate: f64,
    /// Queue depth (events waiting to be processed)
    pub queue_depth: u64,
    /// Last updated timestamp
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Performance monitor for tracking system metrics
pub struct PerformanceMonitor {
    metrics: Arc<RwLock<PerformanceMetrics>>,
    start_time: Instant,
    event_count: AtomicU64,
    error_count: AtomicU64,
    processing_times: RwLock<Vec<Duration>>,
    max_samples: usize,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(PerformanceMetrics {
                total_events: 0,
                events_per_second: 0.0,
                avg_processing_time_us: 0.0,
                peak_events_per_second: 0.0,
                memory_usage_bytes: 0,
                error_rate: 0.0,
                queue_depth: 0,
                last_updated: chrono::Utc::now(),
            })),
            start_time: Instant::now(),
            event_count: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
            processing_times: RwLock::new(Vec::new()),
            max_samples: 1000,
        }
    }

    /// Record an event processing operation
    pub async fn record_event_processing(&self, processing_time: Duration, was_error: bool) {
        // Update counters
        self.event_count.fetch_add(1, Ordering::Relaxed);
        if was_error {
            self.error_count.fetch_add(1, Ordering::Relaxed);
        }

        // Store processing time
        let mut times = self.processing_times.write().await;
        times.push(processing_time);

        // Maintain sample limit
        if times.len() > self.max_samples {
            times.remove(0);
        }

        // Update metrics
        self.update_metrics().await;
    }

    /// Record queue depth
    pub async fn record_queue_depth(&self, depth: u64) {
        let mut metrics = self.metrics.write().await;
        metrics.queue_depth = depth;
        metrics.last_updated = chrono::Utc::now();
    }

    /// Get current performance metrics
    pub async fn get_metrics(&self) -> PerformanceMetrics {
        self.metrics.read().await.clone()
    }

    /// Check if performance thresholds are exceeded
    pub async fn check_thresholds(&self, thresholds: &PerformanceThresholds) -> Vec<PerformanceAlert> {
        let metrics = self.metrics.read().await;
        let mut alerts = Vec::new();

        if metrics.events_per_second > thresholds.max_events_per_second {
            alerts.push(PerformanceAlert::HighThroughput {
                current: metrics.events_per_second,
                threshold: thresholds.max_events_per_second,
            });
        }

        if metrics.avg_processing_time_us > thresholds.max_avg_processing_time_us {
            alerts.push(PerformanceAlert::SlowProcessing {
                current: metrics.avg_processing_time_us,
                threshold: thresholds.max_avg_processing_time_us,
            });
        }

        if metrics.error_rate > thresholds.max_error_rate {
            alerts.push(PerformanceAlert::HighErrorRate {
                current: metrics.error_rate,
                threshold: thresholds.max_error_rate,
            });
        }

        if metrics.queue_depth > thresholds.max_queue_depth {
            alerts.push(PerformanceAlert::QueueBacklog {
                current: metrics.queue_depth,
                threshold: thresholds.max_queue_depth,
            });
        }

        if metrics.memory_usage_bytes > thresholds.max_memory_usage_bytes {
            alerts.push(PerformanceAlert::HighMemoryUsage {
                current: metrics.memory_usage_bytes,
                threshold: thresholds.max_memory_usage_bytes,
            });
        }

        alerts
    }

    /// Update performance metrics
    async fn update_metrics(&self) {
        let mut metrics = self.metrics.write().await;
        let total_events = self.event_count.load(Ordering::Relaxed);
        let error_count = self.error_count.load(Ordering::Relaxed);

        metrics.total_events = total_events;

        // Calculate events per second
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            metrics.events_per_second = total_events as f64 / elapsed;
            metrics.peak_events_per_second = metrics.peak_events_per_second.max(metrics.events_per_second);
        }

        // Calculate average processing time
        let times = self.processing_times.read().await;
        if !times.is_empty() {
            let total_time: Duration = times.iter().sum();
            metrics.avg_processing_time_us = total_time.as_micros() as f64 / times.len() as f64;
        }

        // Calculate error rate
        if total_events > 0 {
            metrics.error_rate = error_count as f64 / total_events as f64;
        }

        // Estimate memory usage (simplified)
        metrics.memory_usage_bytes = (total_events * std::mem::size_of::<ActivityEvent>() as u64) +
                                   (times.len() as u64 * std::mem::size_of::<Duration>() as u64);

        metrics.last_updated = chrono::Utc::now();
    }
}

/// Performance thresholds for alerting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceThresholds {
    /// Maximum events per second
    pub max_events_per_second: f64,
    /// Maximum average processing time in microseconds
    pub max_avg_processing_time_us: f64,
    /// Maximum error rate (0.0 to 1.0)
    pub max_error_rate: f64,
    /// Maximum queue depth
    pub max_queue_depth: u64,
    /// Maximum memory usage in bytes
    pub max_memory_usage_bytes: u64,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            max_events_per_second: 1000.0,
            max_avg_processing_time_us: 10000.0, // 10ms
            max_error_rate: 0.05, // 5%
            max_queue_depth: 10000,
            max_memory_usage_bytes: 100 * 1024 * 1024, // 100MB
        }
    }
}

/// Performance alert types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceAlert {
    /// High event throughput
    HighThroughput { current: f64, threshold: f64 },
    /// Slow event processing
    SlowProcessing { current: f64, threshold: f64 },
    /// High error rate
    HighErrorRate { current: f64, threshold: f64 },
    /// Queue backlog
    QueueBacklog { current: u64, threshold: u64 },
    /// High memory usage
    HighMemoryUsage { current: u64, threshold: u64 },
}

/// Metrics collector for aggregating performance data
pub struct MetricsCollector {
    monitors: RwLock<HashMap<String, Arc<PerformanceMonitor>>>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            monitors: RwLock::new(HashMap::new()),
        }
    }

    /// Register a performance monitor
    pub async fn register_monitor(&self, name: String, monitor: Arc<PerformanceMonitor>) {
        self.monitors.write().await.insert(name, monitor);
    }

    /// Get a monitor by name
    pub async fn get_monitor(&self, name: &str) -> Option<Arc<PerformanceMonitor>> {
        self.monitors.read().await.get(name).cloned()
    }

    /// Get aggregated metrics across all monitors
    pub async fn get_aggregated_metrics(&self) -> AggregatedMetrics {
        let monitors = self.monitors.read().await;
        let mut total_events = 0u64;
        let mut total_errors = 0u64;
        let mut avg_processing_time = 0.0;
        let mut max_queue_depth = 0u64;
        let mut monitor_count = 0;

        for monitor in monitors.values() {
            let metrics = monitor.get_metrics().await;
            total_events += metrics.total_events;
            total_errors += metrics.error_count();
            avg_processing_time += metrics.avg_processing_time_us;
            max_queue_depth = max_queue_depth.max(metrics.queue_depth);
            monitor_count += 1;
        }

        let avg_processing_time = if monitor_count > 0 {
            avg_processing_time / monitor_count as f64
        } else {
            0.0
        };

        let error_rate = if total_events > 0 {
            total_errors as f64 / total_events as f64
        } else {
            0.0
        };

        AggregatedMetrics {
            monitor_count,
            total_events,
            error_rate,
            avg_processing_time_us: avg_processing_time,
            max_queue_depth,
            collected_at: chrono::Utc::now(),
        }
    }

    /// Check all monitors for threshold violations
    pub async fn check_all_thresholds(&self, thresholds: &PerformanceThresholds) -> HashMap<String, Vec<PerformanceAlert>> {
        let monitors = self.monitors.read().await;
        let mut results = HashMap::new();

        for (name, monitor) in monitors.iter() {
            let alerts = monitor.check_thresholds(thresholds).await;
            if !alerts.is_empty() {
                results.insert(name.clone(), alerts);
            }
        }

        results
    }
}

/// Aggregated metrics across multiple monitors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetrics {
    /// Number of monitors
    pub monitor_count: usize,
    /// Total events across all monitors
    pub total_events: u64,
    /// Overall error rate
    pub error_rate: f64,
    /// Average processing time across monitors
    pub avg_processing_time_us: f64,
    /// Maximum queue depth across monitors
    pub max_queue_depth: u64,
    /// Collection timestamp
    pub collected_at: chrono::DateTime<chrono::Utc>,
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceMetrics {
    /// Get error count (derived from error_rate and total_events)
    pub fn error_count(&self) -> u64 {
        (self.error_rate * self.total_events as f64) as u64
    }
}