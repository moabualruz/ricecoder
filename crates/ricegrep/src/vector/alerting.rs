use std::{
    collections::HashMap,
    fmt,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::vector::observability::{VectorTelemetry, VectorTelemetrySnapshot};

/// Severity levels for alert rules.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// State machine entries for an individual alert rule.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AlertState {
    Inactive,
    Pending,
    Firing,
    Resolved,
}

/// Observable metric tracked by alert rules.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MetricKind {
    AvgEmbeddingLatencySeconds,
    AvgQdrantLatencySeconds,
    CpuUsagePercent,
    MemoryUsageBytes,
    DiskUsageBytes,
    NetworkSentBytes,
    NetworkReceivedBytes,
    CacheMissRate,
    IndexBuildDurationSeconds,
    IndexSizeBytes,
    BenchmarkMrr,
}

impl fmt::Display for MetricKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            MetricKind::AvgEmbeddingLatencySeconds => "embedding_latency_seconds",
            MetricKind::AvgQdrantLatencySeconds => "qdrant_latency_seconds",
            MetricKind::CpuUsagePercent => "cpu_usage_percent",
            MetricKind::MemoryUsageBytes => "memory_usage_bytes",
            MetricKind::DiskUsageBytes => "disk_usage_bytes",
            MetricKind::NetworkSentBytes => "network_sent_bytes",
            MetricKind::NetworkReceivedBytes => "network_received_bytes",
            MetricKind::CacheMissRate => "cache_miss_rate",
            MetricKind::IndexBuildDurationSeconds => "index_build_duration_seconds",
            MetricKind::IndexSizeBytes => "index_size_bytes",
            MetricKind::BenchmarkMrr => "benchmark_mrr",
        };
        write!(f, "{}", label)
    }
}

/// Summary object returned by the alert manager endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertSummary {
    pub name: String,
    pub severity: AlertSeverity,
    pub state: AlertState,
    pub metric: MetricKind,
    pub value: Option<f64>,
    pub threshold: Option<f64>,
    pub description: String,
    pub acknowledged: bool,
    pub acknowledged_by: Option<String>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolution_note: Option<String>,
}

impl AlertSummary {
    fn from_rule(rule: &AlertRule, record: &AlertRecord) -> Self {
        Self {
            name: rule.name.to_string(),
            severity: rule.severity,
            state: record.state,
            metric: rule.metric,
            value: record.last_value,
            threshold: Some(rule.threshold),
            description: rule.description.to_string(),
            acknowledged: record.acknowledged,
            acknowledged_by: record.acknowledged_by.clone(),
            acknowledged_at: record.acknowledged_at,
            resolved_at: record.resolved_at,
            resolution_note: record.resolution_note.clone(),
        }
    }
}

/// Comparison operators used to evaluate alert thresholds.
#[derive(Debug, Clone, Copy)]
pub enum ComparisonOp {
    GreaterThan,
    GreaterEqual,
    LessThan,
    LessEqual,
}

impl ComparisonOp {
    fn matches(&self, value: f64, threshold: f64) -> bool {
        match self {
            ComparisonOp::GreaterThan => value > threshold,
            ComparisonOp::GreaterEqual => value >= threshold,
            ComparisonOp::LessThan => value < threshold,
            ComparisonOp::LessEqual => value <= threshold,
        }
    }
}

/// Definition for an alert rule.
#[derive(Debug, Clone)]
pub struct AlertRule {
    pub name: &'static str,
    pub description: &'static str,
    pub severity: AlertSeverity,
    pub metric: MetricKind,
    pub operator: ComparisonOp,
    pub threshold: f64,
    pub duration: Duration,
}

impl AlertRule {
    fn evaluate(&self, value: Option<f64>) -> bool {
        match value {
            Some(value) => self.operator.matches(value, self.threshold),
            None => false,
        }
    }
}

/// Internal record keeping for alert state machines.
#[derive(Clone)]
struct AlertRecord {
    state: AlertState,
    pending_since: Option<Instant>,
    last_transition: Instant,
    last_value: Option<f64>,
    acknowledged: bool,
    acknowledged_by: Option<String>,
    acknowledged_at: Option<DateTime<Utc>>,
    resolved_at: Option<DateTime<Utc>>,
    resolution_note: Option<String>,
}

impl AlertRecord {
    fn new(now: Instant) -> Self {
        Self {
            state: AlertState::Inactive,
            pending_since: None,
            last_transition: now,
            last_value: None,
            acknowledged: false,
            acknowledged_by: None,
            acknowledged_at: None,
            resolved_at: None,
            resolution_note: None,
        }
    }
}

/// Events emitted as alerts transition between states.
#[derive(Debug, Clone)]
pub struct AlertEvent {
    pub name: String,
    pub severity: AlertSeverity,
    pub state: AlertState,
    pub metric: MetricKind,
    pub value: Option<f64>,
    pub threshold: f64,
    pub message: String,
    pub note: Option<String>,
    pub actor: Option<String>,
}

pub trait Notifier: Send + Sync {
    fn notify(&self, event: &AlertEvent);
}

pub trait RemediationHandler: Send + Sync {
    fn remediate(&self, event: &AlertEvent);
}

pub struct AlertEngine {
    rules: Vec<AlertRule>,
    records: Mutex<HashMap<String, AlertRecord>>,
}

impl AlertEngine {
    pub fn new(rules: Vec<AlertRule>) -> Self {
        Self {
            rules,
            records: Mutex::new(HashMap::new()),
        }
    }

    fn rule_by_name(&self, name: &str) -> Option<&AlertRule> {
        self.rules.iter().find(|rule| rule.name == name)
    }

    pub fn evaluate(&self, snapshot: &VectorTelemetrySnapshot, now: Instant) -> Vec<AlertEvent> {
        let mut events = Vec::new();
        let mut records = self.records.lock().unwrap();
        for rule in &self.rules {
            let record = records
                .entry(rule.name.to_string())
                .or_insert_with(|| AlertRecord::new(now));
            record.last_value = snapshot.metric_value(rule.metric);
            if let Some(event) = self.evaluate_rule(rule, snapshot, now, record) {
                events.push(event);
            }
        }
        events
    }

    fn evaluate_rule(
        &self,
        rule: &AlertRule,
        snapshot: &VectorTelemetrySnapshot,
        now: Instant,
        record: &mut AlertRecord,
    ) -> Option<AlertEvent> {
        let matches = rule.evaluate(snapshot.metric_value(rule.metric));
        let value = snapshot.metric_value(rule.metric);
        let mut emit = None;
        match record.state {
            AlertState::Inactive => {
                if matches {
                    record.state = AlertState::Pending;
                    record.pending_since = Some(now);
                    record.last_transition = now;
                    if rule.duration.is_zero() {
                        record.state = AlertState::Firing;
                        record.pending_since = None;
                        record.last_transition = now;
                        emit = Some(self.build_event(rule, AlertState::Firing, value, None, None));
                    }
                }
            }
            AlertState::Pending => {
                if matches {
                    if let Some(start) = record.pending_since {
                        if now.duration_since(start) >= rule.duration {
                            record.state = AlertState::Firing;
                            record.pending_since = None;
                            record.last_transition = now;
                            emit =
                                Some(self.build_event(rule, AlertState::Firing, value, None, None));
                        }
                    }
                } else {
                    record.state = AlertState::Resolved;
                    record.pending_since = None;
                    record.last_transition = now;
                    emit = Some(self.build_event(rule, AlertState::Resolved, value, None, None));
                }
            }
            AlertState::Firing => {
                if !matches {
                    record.state = AlertState::Resolved;
                    record.last_transition = now;
                    emit = Some(self.build_event(rule, AlertState::Resolved, value, None, None));
                }
            }
            AlertState::Resolved => {
                if matches {
                    record.state = AlertState::Pending;
                    record.pending_since = Some(now);
                    record.last_transition = now;
                } else {
                    record.state = AlertState::Inactive;
                    record.pending_since = None;
                    record.last_transition = now;
                }
            }
        }
        emit
    }

    fn build_event(
        &self,
        rule: &AlertRule,
        state: AlertState,
        value: Option<f64>,
        note: Option<String>,
        actor: Option<String>,
    ) -> AlertEvent {
        let mut description = format!(
            "{} {} {:.3} vs threshold {:.3}",
            rule.name,
            rule.metric,
            value.unwrap_or(0.0),
            rule.threshold
        );
        if let Some(note_value) = &note {
            description = format!("{} (note: {})", description, note_value);
        }
        if let Some(actor_name) = &actor {
            description = format!("{} [actor={}]", description, actor_name);
        }
        AlertEvent {
            name: rule.name.to_string(),
            severity: rule.severity,
            state,
            metric: rule.metric,
            value,
            threshold: rule.threshold,
            message: description,
            note,
            actor,
        }
    }

    pub fn summaries(&self) -> Vec<AlertSummary> {
        let records = self.records.lock().unwrap();
        self.rules
            .iter()
            .map(|rule| {
                let record = records
                    .get(rule.name)
                    .cloned()
                    .unwrap_or_else(|| AlertRecord::new(Instant::now()));
                AlertSummary::from_rule(rule, &record)
            })
            .collect()
    }

    pub fn summary_for(&self, rule_name: &str) -> Option<AlertSummary> {
        let rule = self.rule_by_name(rule_name)?;
        let records = self.records.lock().unwrap();
        let record = records
            .get(rule.name)
            .cloned()
            .unwrap_or_else(|| AlertRecord::new(Instant::now()));
        Some(AlertSummary::from_rule(rule, &record))
    }

    pub fn acknowledge(&self, rule_name: &str, actor: Option<String>) -> Option<AlertEvent> {
        let rule = self.rule_by_name(rule_name)?;
        let now = Instant::now();
        let mut records = self.records.lock().unwrap();
        let record = records
            .entry(rule.name.to_string())
            .or_insert_with(|| AlertRecord::new(now));
        record.acknowledged = true;
        record.acknowledged_at = Some(Utc::now());
        record.acknowledged_by = actor.clone();
        record.last_transition = now;
        Some(self.build_event(
            rule,
            record.state,
            record.last_value,
            Some("acknowledged manually".into()),
            actor,
        ))
    }

    pub fn resolve(
        &self,
        rule_name: &str,
        note: Option<String>,
        actor: Option<String>,
    ) -> Option<AlertEvent> {
        let rule = self.rule_by_name(rule_name)?;
        let now = Instant::now();
        let mut records = self.records.lock().unwrap();
        let record = records
            .entry(rule.name.to_string())
            .or_insert_with(|| AlertRecord::new(now));
        record.state = AlertState::Resolved;
        record.resolved_at = Some(Utc::now());
        record.resolution_note = note.clone();
        record.pending_since = None;
        record.last_transition = now;
        if actor.is_some() {
            record.acknowledged = true;
            record.acknowledged_by = actor.clone();
            record.acknowledged_at = record.resolved_at;
        }
        Some(self.build_event(
            rule,
            AlertState::Resolved,
            record.last_value,
            note.or_else(|| Some("resolved manually".into())),
            actor,
        ))
    }
}

impl VectorTelemetrySnapshot {
    fn metric_value(&self, metric: MetricKind) -> Option<f64> {
        match metric {
            MetricKind::AvgEmbeddingLatencySeconds => self
                .avg_embedding_latency_ns
                .map(|ns| ns as f64 / 1_000_000_000.0),
            MetricKind::AvgQdrantLatencySeconds => self
                .avg_qdrant_latency_ns
                .map(|ns| ns as f64 / 1_000_000_000.0),
            MetricKind::CpuUsagePercent => self
                .resource_snapshot
                .as_ref()
                .map(|snapshot| snapshot.cpu_percent),
            MetricKind::MemoryUsageBytes => self
                .resource_snapshot
                .as_ref()
                .map(|snapshot| snapshot.memory_used_bytes as f64),
            MetricKind::DiskUsageBytes => self
                .resource_snapshot
                .as_ref()
                .map(|snapshot| snapshot.disk_used_bytes as f64),
            MetricKind::NetworkSentBytes => self
                .resource_snapshot
                .as_ref()
                .map(|snapshot| snapshot.network_sent_bytes as f64),
            MetricKind::NetworkReceivedBytes => self
                .resource_snapshot
                .as_ref()
                .map(|snapshot| snapshot.network_received_bytes as f64),
            MetricKind::CacheMissRate => {
                let hits = self.cache_hits as f64;
                let misses = self.cache_misses as f64;
                let total = hits + misses;
                if total == 0.0 {
                    None
                } else {
                    Some(misses / total)
                }
            }
            MetricKind::IndexBuildDurationSeconds => self.index_build_duration_seconds,
            MetricKind::IndexSizeBytes => self.index_size_bytes.map(|v| v as f64),
            MetricKind::BenchmarkMrr => None,
        }
    }
}

pub struct AlertManager {
    engine: AlertEngine,
    telemetry: Arc<VectorTelemetry>,
    notifiers: Vec<Arc<dyn Notifier>>,
    remediators: Vec<Arc<dyn RemediationHandler>>,
}

impl AlertManager {
    pub fn new(telemetry: Arc<VectorTelemetry>) -> Self {
        let engine = AlertEngine::new(Self::default_rules());
        let notifiers: Vec<Arc<dyn Notifier>> = vec![
            Arc::new(EmailNotifier::new("ops@ricegrep.example".to_string())),
            Arc::new(SlackNotifier::new("#grain-ops".to_string())),
            Arc::new(PagerDutyNotifier::new("ricegrep-incidents".to_string())),
        ];
        let remediators: Vec<Arc<dyn RemediationHandler>> = vec![
            Arc::new(CircuitBreakerRemediator),
            Arc::new(RestartRemediator),
        ];
        Self {
            engine,
            telemetry,
            notifiers,
            remediators,
        }
    }

    fn default_rules() -> Vec<AlertRule> {
        vec![
            AlertRule {
                name: "HighEmbeddingLatency",
                description: "Embedding latency exceeds 50ms for >10s",
                severity: AlertSeverity::Warning,
                metric: MetricKind::AvgEmbeddingLatencySeconds,
                operator: ComparisonOp::GreaterThan,
                threshold: 0.05,
                duration: Duration::from_secs(10),
            },
            AlertRule {
                name: "HighQdrantLatency",
                description: "Qdrant latency exceeds 150ms for >15s",
                severity: AlertSeverity::Warning,
                metric: MetricKind::AvgQdrantLatencySeconds,
                operator: ComparisonOp::GreaterThan,
                threshold: 0.15,
                duration: Duration::from_secs(15),
            },
            AlertRule {
                name: "HighCpuUsage",
                description: "CPU utilization above 85%",
                severity: AlertSeverity::Warning,
                metric: MetricKind::CpuUsagePercent,
                operator: ComparisonOp::GreaterThan,
                threshold: 85.0,
                duration: Duration::from_secs(20),
            },
            AlertRule {
                name: "HighMemoryUsage",
                description: "Memory usage above 1.5GiB",
                severity: AlertSeverity::Critical,
                metric: MetricKind::MemoryUsageBytes,
                operator: ComparisonOp::GreaterThan,
                threshold: 1.5 * 1024.0 * 1024.0 * 1024.0,
                duration: Duration::from_secs(30),
            },
            AlertRule {
                name: "IndexBuildSlow",
                description: "Index build duration exceeds 10s",
                severity: AlertSeverity::Warning,
                metric: MetricKind::IndexBuildDurationSeconds,
                operator: ComparisonOp::GreaterThan,
                threshold: 10.0,
                duration: Duration::from_secs(20),
            },
            AlertRule {
                name: "CacheMissRateHigh",
                description: "Cache miss rate above 40%",
                severity: AlertSeverity::Info,
                metric: MetricKind::CacheMissRate,
                operator: ComparisonOp::GreaterThan,
                threshold: 0.4,
                duration: Duration::from_secs(60),
            },
        ]
    }

    pub fn check_alerts(&self) -> Vec<AlertSummary> {
        let snapshot = self.telemetry.snapshot();
        let now = Instant::now();
        let events = self.engine.evaluate(&snapshot, now);
        for event in &events {
            self.dispatch_event(event);
        }
        self.engine.summaries()
    }

    fn dispatch_event(&self, event: &AlertEvent) {
        match event.state {
            AlertState::Firing => {
                for notifier in &self.notifiers {
                    notifier.notify(event);
                }
                for remediation in &self.remediators {
                    remediation.remediate(event);
                }
            }
            AlertState::Resolved => {
                for notifier in &self.notifiers {
                    notifier.notify(event);
                }
            }
            _ => {}
        }
    }

    pub fn fire_custom_alert(
        &self,
        name: &str,
        severity: AlertSeverity,
        metric: MetricKind,
        value: Option<f64>,
        threshold: f64,
        note: Option<String>,
    ) {
        let message = note
            .as_ref()
            .cloned()
            .unwrap_or_else(|| format!("custom alert {}", name));
        let event = AlertEvent {
            name: name.to_string(),
            severity,
            state: AlertState::Firing,
            metric,
            value,
            threshold,
            message,
            note,
            actor: None,
        };
        self.dispatch_event(&event);
    }

    pub fn current_summary(&self) -> Vec<AlertSummary> {
        self.engine.summaries()
    }

    pub fn acknowledge_alert(
        &self,
        rule_name: &str,
        actor: Option<String>,
    ) -> Option<AlertSummary> {
        if let Some(event) = self.engine.acknowledge(rule_name, actor.clone()) {
            self.dispatch_event(&event);
        }
        self.engine.summary_for(rule_name)
    }

    pub fn resolve_alert(
        &self,
        rule_name: &str,
        note: Option<String>,
        actor: Option<String>,
    ) -> Option<AlertSummary> {
        if let Some(event) = self.engine.resolve(rule_name, note.clone(), actor.clone()) {
            self.dispatch_event(&event);
        }
        self.engine.summary_for(rule_name)
    }
}

struct EmailNotifier {
    address: String,
}

impl EmailNotifier {
    fn new(address: String) -> Self {
        Self { address }
    }
}

impl Notifier for EmailNotifier {
    fn notify(&self, event: &AlertEvent) {
        info!(
            "email notifier {} sending {} alert: {} (state={:?})",
            self.address, event.name, event.message, event.state
        );
    }
}

struct SlackNotifier {
    channel: String,
}

impl SlackNotifier {
    fn new(channel: String) -> Self {
        Self { channel }
    }
}

impl Notifier for SlackNotifier {
    fn notify(&self, event: &AlertEvent) {
        info!(
            "slack notifier {} posting {} alert: {}",
            self.channel, event.name, event.message
        );
    }
}

struct PagerDutyNotifier {
    service: String,
}

impl PagerDutyNotifier {
    fn new(service: String) -> Self {
        Self { service }
    }
}

impl Notifier for PagerDutyNotifier {
    fn notify(&self, event: &AlertEvent) {
        info!(
            "pagerduty notifier [{}] triggered {} alert (state={:?})",
            self.service, event.name, event.state
        );
    }
}

struct CircuitBreakerRemediator;

impl RemediationHandler for CircuitBreakerRemediator {
    fn remediate(&self, event: &AlertEvent) {
        warn!(
            "circuit breaker activated by alert {} (state={:?})",
            event.name, event.state
        );
    }
}

struct RestartRemediator;

impl RemediationHandler for RestartRemediator {
    fn remediate(&self, event: &AlertEvent) {
        warn!(
            "triggering component restart as remediation for alert {}",
            event.name
        );
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::*;
    use crate::vector::observability::VectorTelemetry;

    #[test]
    fn engine_fires_when_threshold_exceeded() {
        let telemetry = VectorTelemetry::default();
        telemetry.record_embedding(Duration::from_millis(60), 1);
        let snapshot = telemetry.snapshot();
        let rule = AlertRule {
            name: "TestLatency",
            description: "test",
            severity: AlertSeverity::Warning,
            metric: MetricKind::AvgEmbeddingLatencySeconds,
            operator: ComparisonOp::GreaterThan,
            threshold: 0.0,
            duration: Duration::from_secs(0),
        };
        let engine = AlertEngine::new(vec![rule]);
        let events = engine.evaluate(&snapshot, Instant::now());
        assert!(events.iter().any(|event| event.state == AlertState::Firing));
    }

    #[test]
    fn acknowledge_records_actor_and_summary() {
        let telemetry = VectorTelemetry::default();
        let rules = vec![AlertRule {
            name: "TestAlert",
            description: "test ack",
            severity: AlertSeverity::Warning,
            metric: MetricKind::CpuUsagePercent,
            operator: ComparisonOp::GreaterThan,
            threshold: 0.0,
            duration: Duration::from_secs(0),
        }];
        let engine = AlertEngine::new(rules);
        let ack_event = engine.acknowledge("TestAlert", Some("ops-team".to_string()));
        assert!(ack_event.is_some());
        let summary = engine.summary_for("TestAlert").unwrap();
        assert!(summary.acknowledged);
        assert_eq!(summary.acknowledged_by.unwrap(), "ops-team");
    }

    #[test]
    fn resolve_marks_alert_resolved_and_notes() {
        let telemetry = VectorTelemetry::default();
        let rules = vec![AlertRule {
            name: "TestResolve",
            description: "test resolve",
            severity: AlertSeverity::Critical,
            metric: MetricKind::MemoryUsageBytes,
            operator: ComparisonOp::GreaterThan,
            threshold: 0.0,
            duration: Duration::from_secs(0),
        }];
        let engine = AlertEngine::new(rules);
        let resolved = engine.resolve(
            "TestResolve",
            Some("system restart".to_string()),
            Some("ops-team".to_string()),
        );
        assert!(resolved.is_some());
        let summary = engine.summary_for("TestResolve").unwrap();
        assert_eq!(summary.state, AlertState::Resolved);
        assert_eq!(summary.resolution_note.as_deref(), Some("system restart"));
        assert!(summary.resolved_at.is_some());
    }
}
