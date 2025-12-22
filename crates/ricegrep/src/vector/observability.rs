use std::{
    collections::{HashMap, HashSet},
    fmt,
    fs::File,
    io,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use dashmap::DashMap;
use prometheus::{Counter, CounterVec, Error, Gauge, Histogram, HistogramOpts, Opts, Registry};
use serde::{Deserialize, Serialize};
use serde_json;
use thiserror::Error;
use unicode_segmentation::UnicodeSegmentation;

use crate::{chunking::Chunk, lexical::LexicalHit};
use sysinfo::{Disks, NetworkData, Networks, System};

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}

#[derive(Debug, Clone)]
pub enum VectorErrorKind {
    Embedding,
    Qdrant(&'static str),
    Health,
}

impl VectorErrorKind {
    fn label(&self) -> &'static str {
        match self {
            VectorErrorKind::Embedding => "embedding",
            VectorErrorKind::Qdrant(_) => "qdrant",
            VectorErrorKind::Health => "health",
        }
    }
}

const VECTOR_METRIC_NAMESPACE: &str = "ricegrep";
const VECTOR_METRIC_SUBSYSTEM: &str = "vector";
const VECTOR_LATENCY_BUCKETS: [f64; 11] = [
    0.0005, 0.001, 0.0025, 0.005, 0.01, 0.02, 0.05, 0.1, 0.2, 0.5, 1.0,
];

#[derive(Debug, Clone)]
pub struct VectorError {
    kind: VectorErrorKind,
    message: String,
}

impl VectorError {
    pub fn embedding(message: String) -> Self {
        Self {
            kind: VectorErrorKind::Embedding,
            message,
        }
    }

    pub fn qdrant(stage: &'static str, message: String) -> Self {
        Self {
            kind: VectorErrorKind::Qdrant(stage),
            message,
        }
    }

    pub fn health(message: String) -> Self {
        Self {
            kind: VectorErrorKind::Health,
            message,
        }
    }
}

impl fmt::Display for VectorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            VectorErrorKind::Embedding => write!(f, "embedding error: {}", self.message),
            VectorErrorKind::Qdrant(stage) => {
                write!(f, "qdrant [{}] error: {}", stage, self.message)
            }
            VectorErrorKind::Health => write!(f, "health error: {}", self.message),
        }
    }
}

impl std::error::Error for VectorError {}

#[derive(Debug)]
pub struct VectorMetrics {
    embedding_requests: Counter,
    embedding_latency: Histogram,
    qdrant_requests: Counter,
    qdrant_latency: Histogram,
    cache_hits: Counter,
    cache_misses: Counter,
    errors: CounterVec,
    cpu_usage: Gauge,
    memory_usage: Gauge,
    disk_usage: Gauge,
    network_sent: Gauge,
    network_received: Gauge,
    index_size: Gauge,
    index_build_duration: Gauge,
}

impl VectorMetrics {
    pub fn register(registry: &Registry) -> Result<Arc<Self>, Error> {
        let embedding_requests = Counter::with_opts(Self::counter_opts(
            "embedding_requests_total",
            "Total embedding batches processed by the vector pipeline",
        ))?;
        registry.register(Box::new(embedding_requests.clone()))?;

        let embedding_latency = Histogram::with_opts(Self::histogram_opts(
            "embedding_latency_seconds",
            "Latency of embedding batch inference",
        ))?;
        registry.register(Box::new(embedding_latency.clone()))?;

        let qdrant_requests = Counter::with_opts(Self::counter_opts(
            "qdrant_requests_total",
            "Total Qdrant search and management calls",
        ))?;
        registry.register(Box::new(qdrant_requests.clone()))?;

        let qdrant_latency = Histogram::with_opts(Self::histogram_opts(
            "qdrant_latency_seconds",
            "Latency of Qdrant operations",
        ))?;
        registry.register(Box::new(qdrant_latency.clone()))?;

        let cache_hits = Counter::with_opts(Self::counter_opts(
            "cache_hits_total",
            "Count of embedding cache hits",
        ))?;
        registry.register(Box::new(cache_hits.clone()))?;

        let cache_misses = Counter::with_opts(Self::counter_opts(
            "cache_misses_total",
            "Count of embedding cache misses",
        ))?;
        registry.register(Box::new(cache_misses.clone()))?;

        let errors = CounterVec::new(
            Self::counter_opts("errors_total", "Vector pipeline errors by kind"),
            &["kind"],
        )?;
        registry.register(Box::new(errors.clone()))?;

        let cpu_usage = Gauge::with_opts(Self::gauge_opts(
            "cpu_usage_percent",
            "Global CPU usage percentage",
        ))?;
        registry.register(Box::new(cpu_usage.clone()))?;

        let memory_usage = Gauge::with_opts(Self::gauge_opts(
            "memory_usage_bytes",
            "Process memory usage in bytes",
        ))?;
        registry.register(Box::new(memory_usage.clone()))?;

        let disk_usage = Gauge::with_opts(Self::gauge_opts(
            "disk_usage_bytes",
            "Total disk usage in bytes",
        ))?;
        registry.register(Box::new(disk_usage.clone()))?;

        let network_sent = Gauge::with_opts(Self::gauge_opts(
            "network_sent_bytes",
            "Total bytes transmitted by the host",
        ))?;
        registry.register(Box::new(network_sent.clone()))?;

        let network_received = Gauge::with_opts(Self::gauge_opts(
            "network_received_bytes",
            "Total bytes received by the host",
        ))?;
        registry.register(Box::new(network_received.clone()))?;

        let index_size = Gauge::with_opts(Self::gauge_opts(
            "index_size_bytes",
            "Combined index size in bytes",
        ))?;
        registry.register(Box::new(index_size.clone()))?;

        let index_build_duration = Gauge::with_opts(Self::gauge_opts(
            "index_build_duration_seconds",
            "Duration of the most recent index build or refresh",
        ))?;
        registry.register(Box::new(index_build_duration.clone()))?;

        Ok(Arc::new(Self {
            embedding_requests,
            embedding_latency,
            qdrant_requests,
            qdrant_latency,
            cache_hits,
            cache_misses,
            errors,
            cpu_usage,
            memory_usage,
            disk_usage,
            network_sent,
            network_received,
            index_size,
            index_build_duration,
        }))
    }

    fn counter_opts(name: &str, help: &str) -> Opts {
        Opts::new(name, help)
            .namespace(VECTOR_METRIC_NAMESPACE)
            .subsystem(VECTOR_METRIC_SUBSYSTEM)
    }

    fn gauge_opts(name: &str, help: &str) -> Opts {
        Opts::new(name, help)
            .namespace(VECTOR_METRIC_NAMESPACE)
            .subsystem(VECTOR_METRIC_SUBSYSTEM)
    }

    fn histogram_opts(name: &str, help: &str) -> HistogramOpts {
        HistogramOpts::new(name, help)
            .namespace(VECTOR_METRIC_NAMESPACE)
            .subsystem(VECTOR_METRIC_SUBSYSTEM)
            .buckets(VECTOR_LATENCY_BUCKETS.to_vec())
    }

    pub fn observe_embedding(&self, latency: Duration, chunk_count: usize) {
        self.embedding_requests.inc_by(chunk_count as f64);
        self.embedding_latency.observe(latency.as_secs_f64());
    }

    pub fn observe_qdrant(&self, latency: Duration) {
        self.qdrant_requests.inc();
        self.qdrant_latency.observe(latency.as_secs_f64());
    }

    pub fn record_cache_hit(&self) {
        self.cache_hits.inc();
    }

    pub fn record_cache_miss(&self) {
        self.cache_misses.inc();
    }

    pub fn record_error(&self, kind: &VectorErrorKind) {
        if let Ok(counter) = self.errors.get_metric_with_label_values(&[kind.label()]) {
            counter.inc();
        }
    }

    pub fn record_resource_usage(&self, snapshot: &VectorResourceSnapshot) {
        self.cpu_usage.set(snapshot.cpu_percent);
        self.memory_usage.set(snapshot.memory_used_bytes as f64);
        self.disk_usage.set(snapshot.disk_used_bytes as f64);
        self.network_sent.set(snapshot.network_sent_bytes as f64);
        self.network_received
            .set(snapshot.network_received_bytes as f64);
    }

    pub fn record_index_stats(&self, stats: VectorIndexStats) {
        self.index_size.set(stats.index_size_bytes as f64);
        self.index_build_duration
            .set(stats.index_build_duration_seconds);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VectorResourceSnapshot {
    pub cpu_percent: f64,
    pub memory_used_bytes: u64,
    pub disk_used_bytes: u64,
    pub network_sent_bytes: u64,
    pub network_received_bytes: u64,
}

#[derive(Clone, Debug)]
pub struct VectorIndexStats {
    pub index_size_bytes: u64,
    pub index_build_duration_seconds: f64,
}

pub struct SystemResourceSampler {
    system: System,
}

impl SystemResourceSampler {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        Self { system }
    }

    pub fn refresh(&mut self) -> VectorResourceSnapshot {
        self.system.refresh_cpu();
        self.system.refresh_memory();

        let cpu_percent = self.system.global_cpu_info().cpu_usage() as f64;
        let memory_used_bytes = self.system.used_memory().saturating_mul(1024);
        let disks = Disks::new_with_refreshed_list();
        let disk_used_bytes: u64 = disks
            .list()
            .iter()
            .map(|disk| disk.total_space().saturating_sub(disk.available_space()))
            .sum();
        let networks = Networks::new_with_refreshed_list();
        let (network_sent_bytes, network_received_bytes): (u64, u64) = networks
            .list()
            .values()
            .fold((0, 0), |(sent_acc, recv_acc), net| {
                (
                    sent_acc.saturating_add(net.total_transmitted()),
                    recv_acc.saturating_add(net.total_received()),
                )
            });

        VectorResourceSnapshot {
            cpu_percent,
            memory_used_bytes,
            disk_used_bytes,
            network_sent_bytes,
            network_received_bytes,
        }
    }
}

#[derive(Debug, Default)]
pub struct VectorTelemetry {
    embedding_count: AtomicU64,
    embedding_ns: AtomicU64,
    qdrant_count: AtomicU64,
    qdrant_ns: AtomicU64,
    last_error: Mutex<Option<String>>,
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
    index_stats: Mutex<Option<VectorIndexStats>>,
    metrics: Mutex<Option<Arc<VectorMetrics>>>,
    resource_snapshot: Mutex<Option<VectorResourceSnapshot>>,
}

impl VectorTelemetry {
    pub fn attach_metrics(&self, metrics: Arc<VectorMetrics>) {
        let mut guard = self.metrics.lock().unwrap();
        *guard = Some(metrics);
    }

    pub fn record_embedding(&self, latency: Duration, chunks: usize) {
        self.embedding_count
            .fetch_add(chunks as u64, Ordering::Relaxed);
        self.embedding_ns
            .fetch_add(latency.as_nanos() as u64, Ordering::Relaxed);
        if let Some(metrics) = self.metrics_arc() {
            metrics.observe_embedding(latency, chunks);
        }
    }

    pub fn record_qdrant(&self, latency: Duration) {
        self.qdrant_count.fetch_add(1, Ordering::Relaxed);
        self.qdrant_ns
            .fetch_add(latency.as_nanos() as u64, Ordering::Relaxed);
        if let Some(metrics) = self.metrics_arc() {
            metrics.observe_qdrant(latency);
        }
    }

    pub fn record_error(&self, error: &VectorError) {
        let mut lock = self.last_error.lock().unwrap();
        *lock = Some(error.to_string());
        if let Some(metrics) = self.metrics_arc() {
            metrics.record_error(&error.kind);
        }
    }

    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
        if let Some(metrics) = self.metrics_arc() {
            metrics.record_cache_hit();
        }
    }

    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
        if let Some(metrics) = self.metrics_arc() {
            metrics.record_cache_miss();
        }
    }

    pub fn record_resource_usage(&self, snapshot: VectorResourceSnapshot) {
        let mut guard = self.resource_snapshot.lock().unwrap();
        *guard = Some(snapshot);
    }

    pub fn record_index_stats(&self, stats: VectorIndexStats) {
        if let Some(metrics) = self.metrics_arc() {
            metrics.record_index_stats(stats.clone());
        }
        let mut guard = self.index_stats.lock().unwrap();
        *guard = Some(stats);
    }

    pub fn snapshot(&self) -> VectorTelemetrySnapshot {
        let embedding_total = self.embedding_count.load(Ordering::Relaxed);
        let embedding_latency = self.embedding_ns.load(Ordering::Relaxed);
        let qdrant_total = self.qdrant_count.load(Ordering::Relaxed);
        let qdrant_latency = self.qdrant_ns.load(Ordering::Relaxed);
        let last_error = self.last_error.lock().unwrap().clone();
        let cache_hits = self.cache_hits.load(Ordering::Relaxed);
        let cache_misses = self.cache_misses.load(Ordering::Relaxed);
        let index_stats = self.index_stats.lock().unwrap().clone();
        let resource_snapshot = self.resource_snapshot.lock().unwrap().clone();
        VectorTelemetrySnapshot {
            embeddings_total: embedding_total,
            avg_embedding_latency_ns: if embedding_total == 0 {
                None
            } else {
                Some(embedding_latency / embedding_total.max(1))
            },
            qdrant_calls: qdrant_total,
            avg_qdrant_latency_ns: if qdrant_total == 0 {
                None
            } else {
                Some(qdrant_latency / qdrant_total)
            },
            last_error,
            cache_hits,
            cache_misses,
            index_size_bytes: index_stats.as_ref().map(|stats| stats.index_size_bytes),
            index_build_duration_seconds: index_stats
                .as_ref()
                .map(|stats| stats.index_build_duration_seconds),
            resource_snapshot,
        }
    }

    fn metrics_arc(&self) -> Option<Arc<VectorMetrics>> {
        self.metrics.lock().unwrap().clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorTelemetrySnapshot {
    pub embeddings_total: u64,
    pub avg_embedding_latency_ns: Option<u64>,
    pub qdrant_calls: u64,
    pub avg_qdrant_latency_ns: Option<u64>,
    pub last_error: Option<String>,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub index_size_bytes: Option<u64>,
    pub index_build_duration_seconds: Option<f64>,
    pub resource_snapshot: Option<VectorResourceSnapshot>,
}

#[derive(Debug, Default)]
pub struct VectorHealth {
    last_success_ms: AtomicU64,
    last_failure_ms: AtomicU64,
    last_failure_message: Mutex<Option<String>>,
}

impl VectorHealth {
    pub fn record_success(&self) {
        self.last_success_ms.store(now_millis(), Ordering::Relaxed);
    }

    pub fn record_failure(&self, message: String) {
        self.last_failure_ms.store(now_millis(), Ordering::Relaxed);
        let mut lock = self.last_failure_message.lock().unwrap();
        *lock = Some(message);
    }

    pub fn status(&self) -> VectorHealthStatus {
        let success = self.last_success_ms.load(Ordering::Relaxed);
        let failure = self.last_failure_ms.load(Ordering::Relaxed);
        let message = self.last_failure_message.lock().unwrap().clone();
        VectorHealthStatus {
            healthy: success >= failure,
            last_success_ms: if success == 0 { None } else { Some(success) },
            last_failure_ms: if failure == 0 { None } else { Some(failure) },
            last_failure_message: message,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorHealthStatus {
    pub healthy: bool,
    pub last_success_ms: Option<u64>,
    pub last_failure_ms: Option<u64>,
    pub last_failure_message: Option<String>,
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use prometheus::{proto, Registry};

    #[test]
    fn telemetry_tracks_latency_and_errors() {
        let telemetry = VectorTelemetry::default();
        telemetry.record_embedding(Duration::from_millis(10), 2);
        telemetry.record_qdrant(Duration::from_micros(5));
        let snapshot = telemetry.snapshot();
        assert_eq!(snapshot.embeddings_total, 2);
        assert_eq!(snapshot.qdrant_calls, 1);
        assert!(snapshot.avg_embedding_latency_ns.is_some());
        assert!(snapshot.avg_qdrant_latency_ns.unwrap() > 0);
        assert!(snapshot.last_error.is_none());
        telemetry.record_error(&VectorError::embedding("boom".into()));
        let snapshot = telemetry.snapshot();
        assert_eq!(snapshot.last_error.unwrap(), "embedding error: boom");
    }

    #[test]
    fn telemetry_tracks_cache_events() {
        let telemetry = VectorTelemetry::default();
        telemetry.record_cache_hit();
        telemetry.record_cache_miss();
        telemetry.record_cache_miss();
        let snapshot = telemetry.snapshot();
        assert_eq!(snapshot.cache_hits, 1);
        assert_eq!(snapshot.cache_misses, 2);
    }

    #[test]
    fn health_records_success_and_failure() {
        let health = VectorHealth::default();
        health.record_failure("oops".into());
        let status = health.status();
        assert!(!status.healthy);
        assert!(status.last_failure_message.is_some());
        health.record_success();
        let status = health.status();
        assert!(status.healthy);
    }

    #[test]
    fn telemetry_exports_prometheus_metrics() {
        let registry = Registry::new();
        let metrics = VectorMetrics::register(&registry).unwrap();
        let telemetry = VectorTelemetry::default();
        telemetry.attach_metrics(metrics.clone());

        telemetry.record_embedding(Duration::from_millis(5), 2);
        telemetry.record_qdrant(Duration::from_millis(3));
        telemetry.record_cache_hit();
        telemetry.record_cache_miss();
        telemetry.record_error(&VectorError::qdrant("search", "boom".into()));

        let snapshot = VectorResourceSnapshot {
            cpu_percent: 12.5,
            memory_used_bytes: 123_456,
            disk_used_bytes: 1_000_000,
            network_sent_bytes: 10_000,
            network_received_bytes: 20_000,
        };
        metrics.record_resource_usage(&snapshot);
        let index_stats = VectorIndexStats {
            index_size_bytes: 5_000_000,
            index_build_duration_seconds: 0.75,
        };
        metrics.record_index_stats(index_stats.clone());

        let families = registry.gather();

        assert_eq!(metrics.embedding_requests.get(), 2.0);
        assert!(metrics.embedding_latency.get_sample_count() >= 1);
        assert_eq!(metrics.qdrant_requests.get(), 1.0);
        assert_eq!(metrics.cache_hits.get(), 1.0);
        assert_eq!(metrics.cache_misses.get(), 1.0);
        assert_eq!(
            metrics
                .errors
                .get_metric_with_label_values(&["qdrant"])
                .unwrap()
                .get(),
            1.0
        );

        assert!(
            has_metric(&families, "ricegrep_vector_embedding_requests_total"),
            "embedding counter registered"
        );
        assert!(
            has_metric(&families, "ricegrep_vector_qdrant_requests_total"),
            "qdrant counter registered"
        );
        assert!(
            has_metric(&families, "ricegrep_vector_cache_hits_total"),
            "cache hits counter registered"
        );
        assert!(
            has_metric(&families, "ricegrep_vector_cache_misses_total"),
            "cache misses counter registered"
        );
        assert!(
            has_metric(&families, "ricegrep_vector_errors_total"),
            "error counter registered"
        );

        assert_eq!(metrics.cpu_usage.get(), snapshot.cpu_percent);
        assert_eq!(
            metrics.memory_usage.get(),
            snapshot.memory_used_bytes as f64
        );
        assert_eq!(metrics.disk_usage.get(), snapshot.disk_used_bytes as f64);
        assert_eq!(
            metrics.network_sent.get(),
            snapshot.network_sent_bytes as f64
        );
        assert_eq!(
            metrics.network_received.get(),
            snapshot.network_received_bytes as f64
        );
        assert_eq!(
            metrics.index_size.get(),
            index_stats.index_size_bytes as f64
        );
        assert_eq!(
            metrics.index_build_duration.get(),
            index_stats.index_build_duration_seconds
        );
    }

    fn has_metric(families: &[proto::MetricFamily], name: &str) -> bool {
        families.iter().any(|mf| mf.get_name() == name)
    }

    #[test]
    fn system_resource_sampler_reports_values() {
        let mut sampler = SystemResourceSampler::new();
        let snapshot = sampler.refresh();
        assert!(snapshot.memory_used_bytes > 0);
        assert!(snapshot.cpu_percent >= 0.0 && snapshot.cpu_percent <= 100.0);
    }
}
