# RiceCoder Monitoring

Production monitoring infrastructure for RiceCoder, providing comprehensive metrics collection, error tracking, alerting, performance monitoring, analytics, and compliance reporting.

## DDD Layer

**Infrastructure** - Cross-cutting monitoring infrastructure providing observability across the entire RiceCoder ecosystem.

## Features

### Metrics Collection
- System metrics (CPU, memory, disk usage)
- Application performance metrics
- Custom metric registration and collection
- Multiple export formats (Prometheus, OpenTelemetry)

### Error Tracking & Alerting
- Structured error logging with Sentry integration
- Configurable alert rules and thresholds
- Multiple notification channels (Email, Slack, Webhook, PagerDuty, OpsGenie)
- Incident management and response workflows

### Performance Monitoring
- Response time tracking and analysis
- Memory and CPU usage monitoring
- Anomaly detection with statistical analysis
- Performance profiling and bottleneck identification

### Usage Analytics
- User behavior tracking and analysis
- Feature adoption metrics
- Business intelligence reporting
- Engagement scoring and user segmentation

### Compliance Reporting
- SOC 2, GDPR, HIPAA compliance monitoring
- Automated compliance report generation
- Audit logging and retention
- Data retention policy enforcement

### Dashboards & Visualization
- Configurable dashboard system
- Real-time metrics visualization
- Custom panel types (graphs, tables, gauges, stats)
- JSON and HTML export capabilities

## Usage

### Basic Setup

```rust
use ricecoder_monitoring::*;

// Configure monitoring
let monitoring_config = MonitoringConfig {
    metrics: MetricsConfig {
        enabled: true,
        collection_interval: TimeDelta::from_secs(60),
        retention_period: TimeDelta::from_secs(86400 * 30),
        exporters: vec![],
    },
    alerting: AlertingConfig {
        enabled: true,
        rules: vec![],
        channels: vec![],
    },
    error_tracking: ErrorTrackingConfig {
        enabled: true,
        dsn: Some("your-sentry-dsn".to_string()),
        environment: "production".to_string(),
        release: None,
        sample_rate: 1.0,
    },
    performance: PerformanceConfig {
        enabled: true,
        profiling_enabled: true,
        anomaly_detection_enabled: true,
        thresholds: PerformanceThresholds {
            max_response_time_ms: 500,
            max_memory_mb: 300,
            max_cpu_percent: 80.0,
        },
    },
    analytics: AnalyticsConfig {
        enabled: true,
        tracking_id: None,
        event_buffer_size: 1000,
        flush_interval: TimeDelta::from_secs(300),
    },
    compliance: ComplianceConfig {
        enabled: true,
        standards: vec!["SOC2".to_string(), "GDPR".to_string()],
        reporting_interval: TimeDelta::from_secs(86400),
        audit_log_retention: TimeDelta::from_secs(86400 * 2555),
    },
};

// Initialize components
let mut metrics_collector = MetricsCollector::new(monitoring_config.metrics);
let mut error_tracker = ErrorTracker::new(monitoring_config.error_tracking);
let mut performance_monitor = PerformanceMonitor::new(monitoring_config.performance);
let mut analytics_engine = AnalyticsEngine::new(monitoring_config.analytics);
let compliance_engine = ComplianceEngine::new(monitoring_config.compliance);

// Start monitoring
metrics_collector.start().await?;
error_tracker.start().await?;
performance_monitor.start().await?;
analytics_engine.start().await?;
```

### Recording Metrics

```rust
// Record custom metrics
metrics_collector.record_metric("api.requests", 1.0, HashMap::new());
metrics_collector.record_metric("api.response_time", 150.0, {
    let mut labels = HashMap::new();
    labels.insert("endpoint".to_string(), "/api/users".to_string());
    labels
});

// Use performance timers
let _timer = PerformanceTimer::new("database.query".to_string());
```

### Error Tracking

```rust
// Track errors
let error_event = ErrorEvent {
    id: EventId::new_v4(),
    message: "Database connection failed".to_string(),
    error_type: "DatabaseError".to_string(),
    stack_trace: Some("stack trace here".to_string()),
    user_id: Some("user123".to_string()),
    session_id: Some("session456".to_string()),
    context: HashMap::new(),
    timestamp: Utc::now(),
    severity: Severity::High,
};

error_tracker.track_error(error_event);
```

### Analytics Tracking

```rust
// Track user actions
analytics_engine.track_action(
    Some("user123".to_string()),
    "feature_used",
    {
        let mut props = HashMap::new();
        props.insert("feature".to_string(), json!("code_completion"));
        props.insert("language".to_string(), json!("rust"));
        props
    }
);
```

### Compliance Reporting

```rust
// Generate compliance reports
let report = compliance_engine.generate_compliance_report(
    "SOC2",
    Utc::now() - TimeDelta::days(30),
    Utc::now()
).await?;

println!("Compliance status: {:?}", report.status);
```

## Architecture

The monitoring system is designed with a modular architecture:

- **Metrics Collection**: Handles metric gathering and storage
- **Error Tracking**: Manages error events and alerting
- **Performance Monitoring**: Tracks system and application performance
- **Analytics Engine**: Processes usage data and business intelligence
- **Compliance Engine**: Ensures regulatory compliance
- **Dashboard System**: Provides visualization capabilities

All components are designed to be highly concurrent and can be enabled/disabled independently based on configuration.

## Configuration

The monitoring system uses a centralized configuration structure that allows fine-grained control over each subsystem. All components support runtime reconfiguration and can be hot-swapped without restarting the application.

## Integration

The monitoring crate integrates with the RiceCoder DI container and can be automatically registered by enabling the `monitoring` feature flag. It also provides lifecycle management for proper startup and shutdown procedures.

## Dependencies

- `tokio` - Async runtime
- `prometheus` - Metrics collection
- `sentry` - Error tracking
- `sysinfo` - System monitoring
- `dashmap` - Concurrent data structures
- `sqlx` - Database operations (optional)
- `rusqlite` - SQLite support (optional)

## Testing

The monitoring system includes comprehensive unit tests, integration tests, and property-based tests. Run tests with:

```bash
cargo test -p ricecoder-monitoring
```

## License

This crate is part of the RiceCoder project and follows the same licensing terms.