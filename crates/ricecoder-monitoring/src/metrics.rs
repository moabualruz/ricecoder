//! Metrics collection and management

use crate::types::*;
use chrono::{DateTime, Utc, TimeDelta};
use dashmap::DashMap;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration as StdDuration, Instant};
use tokio::sync::mpsc;
use tokio::time;

/// Global metrics registry
static METRICS_REGISTRY: Lazy<DashMap<String, Arc<MetricDefinition>>> = Lazy::new(DashMap::new);

/// Global metrics storage
static METRICS_STORAGE: Lazy<DashMap<String, Vec<DataPoint>>> = Lazy::new(DashMap::new);

/// Metrics collector
pub struct MetricsCollector {
    config: MetricsConfig,
    shutdown_tx: Option<mpsc::Sender<()>>,
    collection_task: Option<tokio::task::JoinHandle<()>>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new(config: MetricsConfig) -> Self {
        Self {
            config,
            shutdown_tx: None,
            collection_task: None,
        }
    }

    /// Start the metrics collector
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.config.enabled {
            return Ok(());
        }

        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        let collection_interval = self.config.collection_interval.to_std().unwrap();

        let task = tokio::spawn(async move {
            let mut interval = time::interval(collection_interval);

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if let Err(e) = Self::collect_system_metrics().await {
                            tracing::error!("Failed to collect system metrics: {}", e);
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        tracing::info!("Metrics collector shutting down");
                        break;
                    }
                }
            }
        });

        self.collection_task = Some(task);
        Ok(())
    }

    /// Stop the metrics collector
    pub async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }

        if let Some(task) = self.collection_task.take() {
            let _ = task.await;
        }

        Ok(())
    }

    /// Register a metric
    pub fn register_metric(&self, metric: Metric) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let definition = Arc::new(MetricDefinition {
            metric,
            created_at: chrono::Utc::now(),
        });

        METRICS_REGISTRY.insert(definition.metric.name.clone(), definition);
        Ok(())
    }

    /// Record a metric value
    pub fn record_metric(&self, name: &str, value: f64, labels: HashMap<String, String>) {
        let point = DataPoint {
            timestamp: chrono::Utc::now(),
            value,
            labels,
        };

        METRICS_STORAGE
            .entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(point);
    }

    /// Get metric data points
    pub fn get_metric_data(&self, name: &str, since: Option<DateTime<Utc>>) -> Vec<DataPoint> {
        if let Some(points) = METRICS_STORAGE.get(name) {
            if let Some(since) = since {
                points.iter()
                    .filter(|p| p.timestamp >= since)
                    .cloned()
                    .collect()
            } else {
                points.clone()
            }
        } else {
            Vec::new()
        }
    }

    /// Get all registered metrics
    pub fn get_registered_metrics(&self) -> Vec<Metric> {
        METRICS_REGISTRY
            .iter()
            .map(|entry| entry.value().metric.clone())
            .collect()
    }

    /// Collect system metrics
    async fn collect_system_metrics() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // CPU usage
        if let Ok(cpu_usage) = Self::get_cpu_usage() {
            METRICS_STORAGE
                .entry("system.cpu.usage".to_string())
                .or_insert_with(Vec::new)
                .push(DataPoint {
                    timestamp: chrono::Utc::now(),
                    value: cpu_usage,
                    labels: HashMap::new(),
                });
        }

        // Memory usage
        if let Ok(mem_usage) = Self::get_memory_usage() {
            METRICS_STORAGE
                .entry("system.memory.usage".to_string())
                .or_insert_with(Vec::new)
                .push(DataPoint {
                    timestamp: chrono::Utc::now(),
                    value: mem_usage as f64,
                    labels: HashMap::new(),
                });
        }

        // Disk usage
        if let Ok(disk_usage) = Self::get_disk_usage() {
            METRICS_STORAGE
                .entry("system.disk.usage".to_string())
                .or_insert_with(Vec::new)
                .push(DataPoint {
                    timestamp: chrono::Utc::now(),
                    value: disk_usage as f64,
                    labels: HashMap::new(),
                });
        }

        Ok(())
    }

    /// Get CPU usage percentage
    fn get_cpu_usage() -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
        // Use sysinfo for cross-platform CPU monitoring
        let mut system = sysinfo::System::new_all();
        system.refresh_cpu_usage();

        let total_usage: f32 = system.cpus().iter().map(|cpu| cpu.cpu_usage()).sum();
        let avg_usage = total_usage / system.cpus().len() as f32;

        Ok(avg_usage as f64)
    }

    /// Get memory usage in bytes
    fn get_memory_usage() -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        let mut system = sysinfo::System::new_all();
        system.refresh_memory();

        Ok(system.used_memory())
    }

    /// Get disk usage in bytes
    fn get_disk_usage() -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        let disks = sysinfo::Disks::new_with_refreshed_list();

        let total: u64 = disks.iter().map(|disk| disk.total_space()).sum();
        let available: u64 = disks.iter().map(|disk| disk.available_space()).sum();

        Ok(total - available)
    }
}

/// Metric definition with metadata
struct MetricDefinition {
    metric: Metric,
    created_at: DateTime<Utc>,
}

/// Metrics exporter trait
#[async_trait::async_trait]
pub trait MetricsExporter: Send + Sync {
    async fn export(&self, metrics: Vec<DataPoint>) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// Prometheus exporter
pub struct PrometheusExporter {
    registry: prometheus::Registry,
}

impl PrometheusExporter {
    pub fn new() -> Self {
        Self {
            registry: prometheus::Registry::new(),
        }
    }

    pub fn registry(&self) -> &prometheus::Registry {
        &self.registry
    }
}

#[async_trait::async_trait]
impl MetricsExporter for PrometheusExporter {
    async fn export(&self, _metrics: Vec<DataPoint>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Prometheus metrics are served via HTTP endpoint
        // This would be implemented in a web server
        Ok(())
    }
}

/// OpenTelemetry exporter
pub struct OpenTelemetryExporter {
    // OpenTelemetry meter provider would be configured here
}

impl OpenTelemetryExporter {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl MetricsExporter for OpenTelemetryExporter {
    async fn export(&self, _metrics: Vec<DataPoint>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Export metrics via OpenTelemetry protocol
        Ok(())
    }
}

/// Performance timer for measuring operation duration
pub struct PerformanceTimer {
    name: String,
    start: Instant,
    labels: HashMap<String, String>,
    finished: bool,
}

impl PerformanceTimer {
    pub fn new(name: String) -> Self {
        Self {
            name,
            start: Instant::now(),
            labels: HashMap::new(),
            finished: false,
        }
    }

    pub fn with_labels(mut self, labels: HashMap<String, String>) -> Self {
        self.labels = labels;
        self
    }

    pub fn finish(&mut self) {
        if self.finished {
            return;
        }
        let duration = self.start.elapsed();
        let duration_ms = duration.as_millis() as f64;
        let name = std::mem::take(&mut self.name);
        let labels = std::mem::take(&mut self.labels);

        METRICS_STORAGE
            .entry(name)
            .or_insert_with(Vec::new)
            .push(DataPoint {
                timestamp: chrono::Utc::now(),
                value: duration_ms,
                labels,
            });
        self.finished = true;
    }
}

impl Drop for PerformanceTimer {
    fn drop(&mut self) {
        if self.finished {
            return;
        }
        // Auto-finish if not manually finished
        let duration = self.start.elapsed();
        let duration_ms = duration.as_millis() as f64;
        let name = std::mem::take(&mut self.name);
        let labels = std::mem::take(&mut self.labels);

        METRICS_STORAGE
            .entry(name)
            .or_insert_with(Vec::new)
            .push(DataPoint {
                timestamp: chrono::Utc::now(),
                value: duration_ms,
                labels,
            });
        self.finished = true;
    }
}

/// Counter metric
pub struct Counter {
    name: String,
    value: Arc<RwLock<u64>>,
}

impl Counter {
    pub fn new(name: String) -> Self {
        Self {
            name,
            value: Arc::new(RwLock::new(0)),
        }
    }

    pub fn increment(&self) {
        *self.value.write() += 1;
        self.record_value();
    }

    pub fn add(&self, value: u64) {
        *self.value.write() += value;
        self.record_value();
    }

    pub fn get(&self) -> u64 {
        *self.value.read()
    }

    fn record_value(&self) {
        let value = self.get() as f64;
        METRICS_STORAGE
            .entry(self.name.clone())
            .or_insert_with(Vec::new)
            .push(DataPoint {
                timestamp: chrono::Utc::now(),
                value,
                labels: HashMap::new(),
            });
    }
}

/// Gauge metric
pub struct Gauge {
    name: String,
    value: Arc<RwLock<f64>>,
}

impl Gauge {
    pub fn new(name: String) -> Self {
        Self {
            name,
            value: Arc::new(RwLock::new(0.0)),
        }
    }

    pub fn set(&self, value: f64) {
        *self.value.write() = value;
        self.record_value();
    }

    pub fn increment(&self, value: f64) {
        *self.value.write() += value;
        self.record_value();
    }

    pub fn decrement(&self, value: f64) {
        *self.value.write() -= value;
        self.record_value();
    }

    pub fn get(&self) -> f64 {
        *self.value.read()
    }

    fn record_value(&self) {
        let value = self.get();
        METRICS_STORAGE
            .entry(self.name.clone())
            .or_insert_with(Vec::new)
            .push(DataPoint {
                timestamp: chrono::Utc::now(),
                value,
                labels: HashMap::new(),
            });
    }
}

/// Histogram metric
pub struct Histogram {
    name: String,
    samples: Arc<RwLock<Vec<f64>>>,
    buckets: Vec<f64>,
}

impl Histogram {
    pub fn new(name: String, buckets: Vec<f64>) -> Self {
        Self {
            name,
            samples: Arc::new(RwLock::new(Vec::new())),
            buckets,
        }
    }

    pub fn observe(&self, value: f64) {
        self.samples.write().push(value);
        self.record_sample(value);
    }

    pub fn get_samples(&self) -> Vec<f64> {
        self.samples.read().clone()
    }

    fn record_sample(&self, value: f64) {
        METRICS_STORAGE
            .entry(self.name.clone())
            .or_insert_with(Vec::new)
            .push(DataPoint {
                timestamp: chrono::Utc::now(),
                value,
                labels: HashMap::new(),
            });
    }
}