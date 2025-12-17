//! Core types for the monitoring system

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, TimeDelta};
use std::collections::HashMap;
use uuid::Uuid;

/// Unique identifier for monitoring events
pub type EventId = Uuid;

/// Severity levels for alerts and errors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// Status of monitoring components
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Healthy,
    Warning,
    Error,
    Unknown,
}

/// Metric types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Summary,
}

/// Time series data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
    pub labels: HashMap<String, String>,
}

/// Metric definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub description: String,
    pub metric_type: MetricType,
    pub unit: Option<String>,
    pub labels: Vec<String>,
}

/// Alert rule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub query: String,
    pub threshold: f64,
    pub severity: Severity,
    pub enabled: bool,
    pub cooldown_period: chrono::TimeDelta,
}

/// Alert instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: EventId,
    pub rule_id: String,
    pub message: String,
    pub severity: Severity,
    pub status: AlertStatus,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub labels: HashMap<String, String>,
}

/// Alert status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertStatus {
    Firing,
    Resolved,
    Silenced,
}

/// Error event for tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEvent {
    pub id: EventId,
    pub message: String,
    pub error_type: String,
    pub stack_trace: Option<String>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub context: HashMap<String, serde_json::Value>,
    pub timestamp: DateTime<Utc>,
    pub severity: Severity,
}

/// Performance metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetric {
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub timestamp: DateTime<Utc>,
    pub tags: HashMap<String, String>,
}

/// Usage analytics event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEvent {
    pub id: EventId,
    pub event_type: String,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub properties: HashMap<String, serde_json::Value>,
    pub timestamp: DateTime<Utc>,
}

/// Compliance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub id: EventId,
    pub report_type: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub findings: Vec<ComplianceFinding>,
    pub status: ComplianceStatus,
    pub generated_at: DateTime<Utc>,
}

/// Compliance finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFinding {
    pub rule_id: String,
    pub description: String,
    pub severity: Severity,
    pub status: ComplianceStatus,
    pub evidence: HashMap<String, serde_json::Value>,
}

/// Compliance status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplianceStatus {
    Pass,
    Fail,
    Warning,
    NotApplicable,
}

/// Dashboard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dashboard {
    pub id: String,
    pub name: String,
    pub description: String,
    pub panels: Vec<Panel>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Dashboard panel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Panel {
    pub id: String,
    pub title: String,
    pub panel_type: PanelType,
    pub query: String,
    pub width: u32,
    pub height: u32,
    pub position: PanelPosition,
}

/// Panel type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PanelType {
    Graph,
    Table,
    Gauge,
    Stat,
    Heatmap,
}

/// Panel position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelPosition {
    pub x: u32,
    pub y: u32,
}

/// Anomaly detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    pub id: EventId,
    pub metric_name: String,
    pub expected_value: f64,
    pub actual_value: f64,
    pub deviation: f64,
    pub confidence: f64,
    pub timestamp: DateTime<Utc>,
    pub labels: HashMap<String, String>,
}

/// Business intelligence report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BIReport {
    pub id: EventId,
    pub title: String,
    pub description: String,
    pub query: String,
    pub data: Vec<HashMap<String, serde_json::Value>>,
    pub generated_at: DateTime<Utc>,
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub metrics: MetricsConfig,
    pub alerting: AlertingConfig,
    pub error_tracking: ErrorTrackingConfig,
    pub performance: PerformanceConfig,
    pub analytics: AnalyticsConfig,
    pub compliance: ComplianceConfig,
}

/// Metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub collection_interval: chrono::TimeDelta,
    pub retention_period: chrono::TimeDelta,
    pub exporters: Vec<MetricsExporter>,
}

/// Metrics exporter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsExporter {
    pub exporter_type: ExporterType,
    pub endpoint: Option<String>,
    pub headers: HashMap<String, String>,
}

/// Exporter type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExporterType {
    Prometheus,
    OpenTelemetry,
    StatsD,
    Custom,
}

/// Alerting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertingConfig {
    pub enabled: bool,
    pub rules: Vec<AlertRule>,
    pub channels: Vec<AlertChannel>,
}

/// Alert channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertChannel {
    pub id: String,
    pub channel_type: ChannelType,
    pub config: HashMap<String, String>,
}

/// Channel type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChannelType {
    Email,
    Slack,
    Webhook,
    PagerDuty,
    OpsGenie,
}

/// Error tracking configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorTrackingConfig {
    pub enabled: bool,
    pub dsn: Option<String>,
    pub environment: String,
    pub release: Option<String>,
    pub sample_rate: f32,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub enabled: bool,
    pub profiling_enabled: bool,
    pub anomaly_detection_enabled: bool,
    pub thresholds: PerformanceThresholds,
}

/// Performance thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceThresholds {
    pub max_response_time_ms: u64,
    pub max_memory_mb: u64,
    pub max_cpu_percent: f64,
}

/// Analytics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsConfig {
    pub enabled: bool,
    pub tracking_id: Option<String>,
    pub event_buffer_size: usize,
    pub flush_interval: chrono::TimeDelta,
}

/// Compliance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceConfig {
    pub enabled: bool,
    pub standards: Vec<String>,
    pub reporting_interval: chrono::TimeDelta,
    pub audit_log_retention: chrono::TimeDelta,
}