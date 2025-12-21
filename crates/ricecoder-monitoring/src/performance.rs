//! Performance monitoring and anomaly detection

use crate::types::*;
use chrono::{DateTime, TimeDelta, Utc};
use dashmap::DashMap;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration as StdDuration;
use tokio::sync::mpsc;
use tokio::time;

/// Global performance metrics storage
static PERFORMANCE_METRICS: Lazy<DashMap<String, Vec<PerformanceMetric>>> = Lazy::new(DashMap::new);

/// Global anomaly storage
static ANOMALIES: Lazy<DashMap<EventId, Anomaly>> = Lazy::new(DashMap::new);

/// Performance monitor
pub struct PerformanceMonitor {
    config: PerformanceConfig,
    shutdown_tx: Option<mpsc::Sender<()>>,
    monitor_task: Option<tokio::task::JoinHandle<()>>,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new(config: PerformanceConfig) -> Self {
        Self {
            config,
            shutdown_tx: None,
            monitor_task: None,
        }
    }

    /// Start the performance monitor
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.config.enabled {
            return Ok(());
        }

        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        let anomaly_detection_enabled = self.config.anomaly_detection_enabled;
        let thresholds = self.config.thresholds.clone();

        let task = tokio::spawn(async move {
            let mut interval = time::interval(StdDuration::from_secs(60)); // Monitor interval

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if let Err(e) = Self::collect_performance_metrics().await {
                            tracing::error!("Failed to collect performance metrics: {}", e);
                        }

                        if anomaly_detection_enabled {
                            if let Err(e) = Self::detect_anomalies(&thresholds).await {
                                tracing::error!("Failed to detect anomalies: {}", e);
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        tracing::info!("Performance monitor shutting down");
                        break;
                    }
                }
            }
        });

        self.monitor_task = Some(task);
        Ok(())
    }

    /// Stop the performance monitor
    pub async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }

        if let Some(task) = self.monitor_task.take() {
            let _ = task.await;
        }

        Ok(())
    }

    /// Record a performance metric
    pub fn record_metric(&self, metric: PerformanceMetric) {
        PERFORMANCE_METRICS
            .entry(metric.name.clone())
            .or_insert_with(Vec::new)
            .push(metric);
    }

    /// Get performance metrics for a given name
    pub fn get_metrics(&self, name: &str, since: Option<DateTime<Utc>>) -> Vec<PerformanceMetric> {
        if let Some(metrics) = PERFORMANCE_METRICS.get(name) {
            if let Some(since) = since {
                metrics
                    .iter()
                    .filter(|m| m.timestamp >= since)
                    .cloned()
                    .collect()
            } else {
                metrics.clone()
            }
        } else {
            Vec::new()
        }
    }

    /// Get performance statistics
    pub fn get_performance_stats(
        &self,
        name: &str,
        since: Option<DateTime<Utc>>,
    ) -> Option<PerformanceStats> {
        let metrics = self.get_metrics(name, since);

        if metrics.is_empty() {
            return None;
        }

        let values: Vec<f64> = metrics.iter().map(|m| m.value).collect();
        let mean = values.iter().sum::<f64>() / values.len() as f64;

        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        let sorted_values = {
            let mut sorted = values.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            sorted
        };

        let p50 = percentile(&sorted_values, 50.0);
        let p95 = percentile(&sorted_values, 95.0);
        let p99 = percentile(&sorted_values, 99.0);

        let min = *values.iter().min_by(|a, b| a.partial_cmp(b).unwrap())?;
        let max = *values.iter().max_by(|a, b| a.partial_cmp(b).unwrap())?;

        Some(PerformanceStats {
            metric_name: name.to_string(),
            count: values.len(),
            mean,
            std_dev,
            min,
            max,
            p50,
            p95,
            p99,
            period_start: since.unwrap_or_else(|| chrono::Utc::now() - chrono::TimeDelta::hours(1)),
            period_end: chrono::Utc::now(),
        })
    }

    /// Collect system performance metrics
    async fn collect_performance_metrics() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let timestamp = chrono::Utc::now();

        // Response time (placeholder - would be collected from actual operations)
        let response_time = PerformanceMetric {
            name: "http.response_time".to_string(),
            value: 150.0, // ms
            unit: "ms".to_string(),
            timestamp,
            tags: HashMap::new(),
        };
        PERFORMANCE_METRICS
            .entry(response_time.name.clone())
            .or_insert_with(Vec::new)
            .push(response_time);

        // Memory usage
        if let Ok(memory_mb) = Self::get_memory_usage_mb() {
            let memory_metric = PerformanceMetric {
                name: "system.memory.usage".to_string(),
                value: memory_mb,
                unit: "MB".to_string(),
                timestamp,
                tags: HashMap::new(),
            };
            PERFORMANCE_METRICS
                .entry(memory_metric.name.clone())
                .or_insert_with(Vec::new)
                .push(memory_metric);
        }

        // CPU usage
        if let Ok(cpu_percent) = Self::get_cpu_usage_percent() {
            let cpu_metric = PerformanceMetric {
                name: "system.cpu.usage".to_string(),
                value: cpu_percent,
                unit: "%".to_string(),
                timestamp,
                tags: HashMap::new(),
            };
            PERFORMANCE_METRICS
                .entry(cpu_metric.name.clone())
                .or_insert_with(Vec::new)
                .push(cpu_metric);
        }

        Ok(())
    }

    /// Get memory usage in MB
    fn get_memory_usage_mb() -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
        let mut system = sysinfo::System::new_all();
        system.refresh_memory();
        Ok(system.used_memory() as f64 / 1024.0 / 1024.0)
    }

    /// Get CPU usage percentage
    fn get_cpu_usage_percent() -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
        let mut system = sysinfo::System::new_all();
        system.refresh_cpu_usage();

        let total_usage: f32 = system.cpus().iter().map(|cpu| cpu.cpu_usage()).sum();
        Ok((total_usage / system.cpus().len() as f32) as f64)
    }

    /// Detect anomalies in performance metrics
    async fn detect_anomalies(
        thresholds: &PerformanceThresholds,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Check response time anomalies
        if let Some(stats) = Self::get_performance_stats_static(
            "http.response_time",
            Some(chrono::Utc::now() - chrono::TimeDelta::hours(1)),
        ) {
            if stats.p95 > thresholds.max_response_time_ms as f64 {
                let anomaly = Anomaly {
                    id: EventId::new_v4(),
                    metric_name: "http.response_time".to_string(),
                    expected_value: stats.mean,
                    actual_value: stats.p95,
                    deviation: ((stats.p95 - stats.mean) / stats.mean) * 100.0,
                    confidence: Self::calculate_anomaly_confidence(&stats),
                    timestamp: chrono::Utc::now(),
                    labels: {
                        let mut labels = HashMap::new();
                        labels.insert(
                            "threshold".to_string(),
                            thresholds.max_response_time_ms.to_string(),
                        );
                        labels
                    },
                };

                ANOMALIES.insert(anomaly.id, anomaly.clone());

                tracing::warn!(
                    anomaly_id = %anomaly.id,
                    metric = %anomaly.metric_name,
                    expected = %anomaly.expected_value,
                    actual = %anomaly.actual_value,
                    deviation = %anomaly.deviation,
                    "Performance anomaly detected"
                );
            }
        }

        // Check memory usage anomalies
        if let Some(stats) = Self::get_performance_stats_static(
            "system.memory.usage",
            Some(chrono::Utc::now() - chrono::TimeDelta::hours(1)),
        ) {
            if stats.p95 > thresholds.max_memory_mb as f64 {
                let anomaly = Anomaly {
                    id: EventId::new_v4(),
                    metric_name: "system.memory.usage".to_string(),
                    expected_value: stats.mean,
                    actual_value: stats.p95,
                    deviation: ((stats.p95 - stats.mean) / stats.mean) * 100.0,
                    confidence: Self::calculate_anomaly_confidence(&stats),
                    timestamp: chrono::Utc::now(),
                    labels: {
                        let mut labels = HashMap::new();
                        labels.insert(
                            "threshold".to_string(),
                            thresholds.max_memory_mb.to_string(),
                        );
                        labels
                    },
                };

                ANOMALIES.insert(anomaly.id, anomaly.clone());

                tracing::warn!(
                    anomaly_id = %anomaly.id,
                    metric = %anomaly.metric_name,
                    expected = %anomaly.expected_value,
                    actual = %anomaly.actual_value,
                    deviation = %anomaly.deviation,
                    "Memory usage anomaly detected"
                );
            }
        }

        Ok(())
    }

    /// Get performance stats (static version for internal use)
    fn get_performance_stats_static(
        name: &str,
        since: Option<DateTime<Utc>>,
    ) -> Option<PerformanceStats> {
        let metrics = if let Some(metrics) = PERFORMANCE_METRICS.get(name) {
            if let Some(since) = since {
                metrics
                    .iter()
                    .filter(|m| m.timestamp >= since)
                    .cloned()
                    .collect()
            } else {
                metrics.clone()
            }
        } else {
            Vec::new()
        };

        if metrics.is_empty() {
            return None;
        }

        let values: Vec<f64> = metrics.iter().map(|m| m.value).collect();
        let mean = values.iter().sum::<f64>() / values.len() as f64;

        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        let sorted_values = {
            let mut sorted = values.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            sorted
        };

        let p50 = percentile(&sorted_values, 50.0);
        let p95 = percentile(&sorted_values, 95.0);
        let p99 = percentile(&sorted_values, 99.0);

        let min = *values.iter().min_by(|a, b| a.partial_cmp(b).unwrap())?;
        let max = *values.iter().max_by(|a, b| a.partial_cmp(b).unwrap())?;

        Some(PerformanceStats {
            metric_name: name.to_string(),
            count: values.len(),
            mean,
            std_dev,
            min,
            max,
            p50,
            p95,
            p99,
            period_start: since.unwrap_or_else(|| chrono::Utc::now() - chrono::TimeDelta::hours(1)),
            period_end: chrono::Utc::now(),
        })
    }

    /// Calculate anomaly confidence based on statistical significance
    fn calculate_anomaly_confidence(stats: &PerformanceStats) -> f64 {
        // Simple confidence calculation based on standard deviation
        let z_score = (stats.p95 - stats.mean) / stats.std_dev;
        // Convert z-score to confidence (simplified)
        let confidence = 1.0 - (-z_score.abs() / 3.0).exp();
        (confidence * 100.0).min(99.9).max(50.0)
    }
}

/// Performance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStats {
    pub metric_name: String,
    pub count: usize,
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}

/// Anomaly detector
pub struct AnomalyDetector {
    baseline_window: StdDuration,
    sensitivity: f64,
}

impl AnomalyDetector {
    /// Create a new anomaly detector
    pub fn new(baseline_window: StdDuration, sensitivity: f64) -> Self {
        Self {
            baseline_window,
            sensitivity,
        }
    }

    /// Detect anomalies in a time series
    pub fn detect_anomalies(&self, metric_name: &str, current_value: f64) -> Vec<Anomaly> {
        let baseline_start = chrono::Utc::now()
            - chrono::TimeDelta::from_std(self.baseline_window)
                .unwrap_or(chrono::TimeDelta::hours(1));

        let baseline_metrics: Vec<f64> = PERFORMANCE_METRICS
            .get(metric_name)
            .map(|metrics| {
                metrics
                    .iter()
                    .filter(|m| m.timestamp >= baseline_start)
                    .map(|m| m.value)
                    .collect()
            })
            .unwrap_or_default();

        if baseline_metrics.len() < 10 {
            return Vec::new(); // Not enough data for baseline
        }

        let mean = baseline_metrics.iter().sum::<f64>() / baseline_metrics.len() as f64;
        let variance = baseline_metrics
            .iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>()
            / baseline_metrics.len() as f64;
        let std_dev = variance.sqrt();

        let z_score = (current_value - mean) / std_dev;

        if z_score.abs() > self.sensitivity {
            let deviation = ((current_value - mean) / mean) * 100.0;
            let confidence = (1.0 - (-z_score.abs() / 3.0).exp()) * 100.0;

            vec![Anomaly {
                id: EventId::new_v4(),
                metric_name: metric_name.to_string(),
                expected_value: mean,
                actual_value: current_value,
                deviation,
                confidence: confidence.min(99.9).max(50.0),
                timestamp: chrono::Utc::now(),
                labels: {
                    let mut labels = HashMap::new();
                    labels.insert("z_score".to_string(), z_score.to_string());
                    labels.insert("sensitivity".to_string(), self.sensitivity.to_string());
                    labels
                },
            }]
        } else {
            Vec::new()
        }
    }

    /// Get detected anomalies
    pub fn get_anomalies(&self, since: Option<DateTime<Utc>>) -> Vec<Anomaly> {
        let mut anomalies: Vec<_> = ANOMALIES
            .iter()
            .filter_map(|entry| {
                let anomaly = entry.value();
                if let Some(since) = since {
                    if anomaly.timestamp >= since {
                        Some(anomaly.clone())
                    } else {
                        None
                    }
                } else {
                    Some(anomaly.clone())
                }
            })
            .collect();

        anomalies.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        anomalies
    }
}

/// Performance profiler
pub struct PerformanceProfiler {
    traces: Arc<RwLock<HashMap<String, Vec<TraceEvent>>>>,
}

impl PerformanceProfiler {
    pub fn new() -> Self {
        Self {
            traces: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start profiling a function
    pub fn start_trace(&self, name: &str) -> TraceHandle {
        let start_time = std::time::Instant::now();

        TraceHandle {
            name: name.to_string(),
            start_time,
            profiler: Arc::clone(&self.traces),
        }
    }

    /// Get trace events for a function
    pub fn get_traces(&self, name: &str) -> Vec<TraceEvent> {
        self.traces.read().get(name).cloned().unwrap_or_default()
    }

    /// Get performance bottlenecks
    pub fn get_bottlenecks(&self, threshold_ms: u64) -> Vec<Bottleneck> {
        let traces = self.traces.read();
        let mut bottlenecks = Vec::new();

        for (name, events) in traces.iter() {
            let slow_events: Vec<_> = events
                .iter()
                .filter(|e| e.duration_ms > threshold_ms)
                .cloned()
                .collect();

            if !slow_events.is_empty() {
                let avg_duration = slow_events.iter().map(|e| e.duration_ms).sum::<u64>()
                    / slow_events.len() as u64;

                bottlenecks.push(Bottleneck {
                    function_name: name.clone(),
                    slow_call_count: slow_events.len(),
                    avg_duration_ms: avg_duration,
                    max_duration_ms: slow_events.iter().map(|e| e.duration_ms).max().unwrap_or(0),
                    threshold_ms,
                });
            }
        }

        bottlenecks.sort_by(|a, b| b.avg_duration_ms.cmp(&a.avg_duration_ms));
        bottlenecks
    }
}

/// Trace handle for RAII-style profiling
pub struct TraceHandle {
    name: String,
    start_time: std::time::Instant,
    profiler: Arc<RwLock<HashMap<String, Vec<TraceEvent>>>>,
}

impl Drop for TraceHandle {
    fn drop(&mut self) {
        let duration = self.start_time.elapsed();
        let duration_ms = duration.as_millis() as u64;

        let event = TraceEvent {
            timestamp: chrono::Utc::now(),
            duration_ms,
            thread_id: std::thread::current().id(),
        };

        let mut traces = self.profiler.write();
        traces
            .entry(self.name.clone())
            .or_insert_with(Vec::new)
            .push(event);
    }
}

/// Trace event
#[derive(Debug, Clone)]
pub struct TraceEvent {
    pub timestamp: DateTime<Utc>,
    pub duration_ms: u64,
    pub thread_id: std::thread::ThreadId,
}

/// Performance bottleneck
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bottleneck {
    pub function_name: String,
    pub slow_call_count: usize,
    pub avg_duration_ms: u64,
    pub max_duration_ms: u64,
    pub threshold_ms: u64,
}

/// Calculate percentile from sorted values
fn percentile(sorted_values: &[f64], p: f64) -> f64 {
    if sorted_values.is_empty() {
        return 0.0;
    }

    let index = (p / 100.0 * (sorted_values.len() - 1) as f64) as usize;
    sorted_values[index]
}
