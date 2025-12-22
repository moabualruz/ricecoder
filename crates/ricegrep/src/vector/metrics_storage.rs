use chrono::{DateTime, Duration as ChronoDuration, NaiveDateTime, Utc};
use serde::Serialize;
use std::{
    collections::{BTreeMap, VecDeque},
    sync::Mutex,
    time::Duration,
};

use crate::vector::observability::{VectorResourceSnapshot, VectorTelemetrySnapshot};

const DEFAULT_RETENTION_DAYS: i64 = 90;

#[derive(Debug)]
struct MetricAccumulator {
    sum: f64,
    count: u64,
}

impl MetricAccumulator {
    fn new() -> Self {
        Self { sum: 0.0, count: 0 }
    }

    fn add(&mut self, value: f64) {
        self.sum += value;
        self.count += 1;
    }

    fn mean(&self) -> Option<f64> {
        if self.count == 0 {
            None
        } else {
            Some(self.sum / self.count as f64)
        }
    }
}

#[derive(Debug)]
struct BucketAccumulator {
    timestamp: DateTime<Utc>,
    embedding_latency: MetricAccumulator,
    qdrant_latency: MetricAccumulator,
    cache_miss_rate: MetricAccumulator,
    cpu_percent: MetricAccumulator,
    memory_usage: MetricAccumulator,
    disk_usage: MetricAccumulator,
    network_sent: MetricAccumulator,
    network_received: MetricAccumulator,
    index_size: MetricAccumulator,
    index_build_duration: MetricAccumulator,
}

impl BucketAccumulator {
    fn new(timestamp: DateTime<Utc>) -> Self {
        Self {
            timestamp,
            embedding_latency: MetricAccumulator::new(),
            qdrant_latency: MetricAccumulator::new(),
            cache_miss_rate: MetricAccumulator::new(),
            cpu_percent: MetricAccumulator::new(),
            memory_usage: MetricAccumulator::new(),
            disk_usage: MetricAccumulator::new(),
            network_sent: MetricAccumulator::new(),
            network_received: MetricAccumulator::new(),
            index_size: MetricAccumulator::new(),
            index_build_duration: MetricAccumulator::new(),
        }
    }

    fn add_snapshot(&mut self, snapshot: &VectorTelemetrySnapshot) {
        if let Some(latency_ns) = snapshot.avg_embedding_latency_ns {
            self.embedding_latency.add(latency_ns as f64 / 1_000_000.0);
        }
        if let Some(latency_ns) = snapshot.avg_qdrant_latency_ns {
            self.qdrant_latency.add(latency_ns as f64 / 1_000_000.0);
        }
        if let Some(rate) = cache_miss_rate(snapshot) {
            self.cache_miss_rate.add(rate);
        }
        if let Some(resource_snapshot) = &snapshot.resource_snapshot {
            self.cpu_percent.add(resource_snapshot.cpu_percent);
            self.memory_usage
                .add(resource_snapshot.memory_used_bytes as f64);
            self.disk_usage
                .add(resource_snapshot.disk_used_bytes as f64);
            self.network_sent
                .add(resource_snapshot.network_sent_bytes as f64);
            self.network_received
                .add(resource_snapshot.network_received_bytes as f64);
        }
        if let Some(index_size) = snapshot.index_size_bytes {
            self.index_size.add(index_size as f64);
        }
        if let Some(index_build) = snapshot.index_build_duration_seconds {
            self.index_build_duration.add(index_build);
        }
    }

    fn into_entry(self) -> MetricsHistoryEntry {
        MetricsHistoryEntry {
            timestamp: self.timestamp,
            avg_embedding_latency_ms: self.embedding_latency.mean(),
            avg_qdrant_latency_ms: self.qdrant_latency.mean(),
            cache_miss_rate: self.cache_miss_rate.mean(),
            cpu_percent: self.cpu_percent.mean(),
            memory_usage_bytes: self.memory_usage.mean(),
            disk_usage_bytes: self.disk_usage.mean(),
            network_sent_bytes: self.network_sent.mean(),
            network_received_bytes: self.network_received.mean(),
            index_size_bytes: self.index_size.mean(),
            index_build_duration_seconds: self.index_build_duration.mean(),
        }
    }
}

fn cache_miss_rate(snapshot: &VectorTelemetrySnapshot) -> Option<f64> {
    let hits = snapshot.cache_hits as f64;
    let misses = snapshot.cache_misses as f64;
    let total = hits + misses;
    if total == 0.0 {
        None
    } else {
        Some(misses / total)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MetricsHistoryEntry {
    pub timestamp: DateTime<Utc>,
    pub avg_embedding_latency_ms: Option<f64>,
    pub avg_qdrant_latency_ms: Option<f64>,
    pub cache_miss_rate: Option<f64>,
    pub cpu_percent: Option<f64>,
    pub memory_usage_bytes: Option<f64>,
    pub disk_usage_bytes: Option<f64>,
    pub network_sent_bytes: Option<f64>,
    pub network_received_bytes: Option<f64>,
    pub index_size_bytes: Option<f64>,
    pub index_build_duration_seconds: Option<f64>,
}

pub struct MetricsStorage {
    retention: Duration,
    aggregation_bucket: Duration,
    history: Mutex<VecDeque<(DateTime<Utc>, VectorTelemetrySnapshot)>>,
}

impl MetricsStorage {
    pub fn new(retention: Duration, aggregation_bucket: Duration) -> Self {
        Self {
            retention,
            aggregation_bucket,
            history: Mutex::new(VecDeque::new()),
        }
    }

    pub fn record(&self, snapshot: VectorTelemetrySnapshot) {
        let mut history = self.history.lock().unwrap();
        history.push_back((Utc::now(), snapshot));
        self.prune(&mut history);
    }

    fn prune(&self, history: &mut VecDeque<(DateTime<Utc>, VectorTelemetrySnapshot)>) {
        let now = Utc::now();
        let retention = ChronoDuration::from_std(self.retention)
            .unwrap_or_else(|_| ChronoDuration::days(DEFAULT_RETENTION_DAYS));
        let cutoff = now - retention;
        while let Some((timestamp, _)) = history.front() {
            if *timestamp < cutoff {
                history.pop_front();
            } else {
                break;
            }
        }
    }

    pub fn aggregated_history(
        &self,
        lookback: Duration,
        bucket: Duration,
    ) -> Vec<MetricsHistoryEntry> {
        let bucket_ms = bucket.as_millis() as i64;
        if bucket_ms <= 0 {
            return Vec::new();
        }

        let now = Utc::now();
        let retention = ChronoDuration::from_std(self.retention)
            .unwrap_or_else(|_| ChronoDuration::days(DEFAULT_RETENTION_DAYS));
        let retention_start = now - retention;

        let lookback_duration =
            ChronoDuration::from_std(lookback).unwrap_or_else(|_| ChronoDuration::hours(1));
        let window_start = {
            let candidate = now - lookback_duration;
            if candidate < retention_start {
                retention_start
            } else {
                candidate
            }
        };

        let history = self.history.lock().unwrap();
        let mut buckets: BTreeMap<i64, BucketAccumulator> = BTreeMap::new();
        for (timestamp, snapshot) in history.iter() {
            if *timestamp < window_start {
                continue;
            }
            let bucket_key = (timestamp.timestamp_millis() / bucket_ms) * bucket_ms;
            let bucket_start = match NaiveDateTime::from_timestamp_millis(bucket_key) {
                Some(naive) => DateTime::<Utc>::from_utc(naive, Utc),
                None => continue,
            };
            let accumulator = buckets
                .entry(bucket_key)
                .or_insert_with(|| BucketAccumulator::new(bucket_start));
            accumulator.add_snapshot(snapshot);
        }

        buckets
            .into_iter()
            .map(|(_, bucket)| bucket.into_entry())
            .collect()
    }

    pub fn aggregated_history_default(&self, lookback: Duration) -> Vec<MetricsHistoryEntry> {
        self.aggregated_history(lookback, self.aggregation_bucket)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    fn sample_snapshot() -> VectorTelemetrySnapshot {
        VectorTelemetrySnapshot {
            embeddings_total: 10,
            avg_embedding_latency_ns: Some(500_000),
            qdrant_calls: 5,
            avg_qdrant_latency_ns: Some(1_000_000),
            last_error: None,
            cache_hits: 3,
            cache_misses: 1,
            index_size_bytes: Some(4_000_000),
            index_build_duration_seconds: Some(2.5),
            resource_snapshot: Some(VectorResourceSnapshot {
                cpu_percent: 42.0,
                memory_used_bytes: 512_000,
                disk_used_bytes: 1_000_000,
                network_sent_bytes: 10_000,
                network_received_bytes: 8_000,
            }),
        }
    }

    #[test]
    fn aggregated_history_returns_bucketed_values() {
        let storage = MetricsStorage::new(Duration::from_secs(3600), Duration::from_secs(60));
        storage.record(sample_snapshot());
        let history = storage.aggregated_history_default(Duration::from_secs(3600));
        assert_eq!(history.len(), 1);
        let entry = &history[0];
        assert_eq!(entry.avg_embedding_latency_ms, Some(0.5));
        assert_eq!(entry.avg_qdrant_latency_ms, Some(1.0));
        assert_eq!(entry.cache_miss_rate, Some(0.25));
        assert_eq!(entry.cpu_percent, Some(42.0));
        assert_eq!(entry.memory_usage_bytes, Some(512_000.0));
        assert_eq!(entry.disk_usage_bytes, Some(1_000_000.0));
        assert_eq!(entry.index_size_bytes, Some(4_000_000.0));
        assert_eq!(entry.index_build_duration_seconds, Some(2.5));
    }
}
