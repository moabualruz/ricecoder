use chrono::{TimeDelta, Utc};

pub mod alerting;
pub mod analytics;
pub mod anomaly_detection;
pub mod compliance;
pub mod dashboards;
pub mod di;
pub mod error_tracking;
pub mod metrics;
pub mod performance;
pub mod reporting;
pub mod types;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::{
        analytics::AnalyticsEngine, compliance::ComplianceEngine, dashboards::DashboardManager,
        error_tracking::ErrorTracker, metrics::MetricsCollector, performance::PerformanceMonitor,
        types::*,
    };

    #[tokio::test]
    async fn test_metrics_collection() {
        let config = MetricsConfig {
            enabled: true,
            collection_interval: chrono::TimeDelta::seconds(1),
            retention_period: chrono::TimeDelta::hours(1),
            exporters: vec![],
        };

        let mut collector = MetricsCollector::new(config);
        collector.start().await.unwrap();

        // Record some metrics
        collector.record_metric("test.counter", 42.0, HashMap::new());
        collector.record_metric("test.gauge", 85.5, HashMap::new());

        // Check metrics were recorded
        let counter_data = collector.get_metric_data("test.counter", None);
        let gauge_data = collector.get_metric_data("test.gauge", None);

        assert_eq!(counter_data.len(), 1);
        assert_eq!(counter_data[0].value, 42.0);
        assert_eq!(gauge_data.len(), 1);
        assert_eq!(gauge_data[0].value, 85.5);

        collector.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_error_tracking() {
        let config = ErrorTrackingConfig {
            enabled: true,
            dsn: None,
            environment: "test".to_string(),
            release: None,
            sample_rate: 1.0_f32,
        };

        let mut tracker = ErrorTracker::new(config);
        tracker.start().await.unwrap();

        let error_event = ErrorEvent {
            id: EventId::new_v4(),
            message: "Test error".to_string(),
            error_type: "TestError".to_string(),
            stack_trace: Some("stack trace here".to_string()),
            user_id: Some("user123".to_string()),
            session_id: Some("session456".to_string()),
            context: {
                let mut ctx = HashMap::new();
                ctx.insert("key".to_string(), serde_json::json!("value"));
                ctx
            },
            timestamp: chrono::Utc::now(),
            severity: Severity::High,
        };

        tracker.track_error(error_event.clone());

        let errors = tracker.get_error_events(None, None, Some(10));
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "Test error");

        tracker.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_performance_monitoring() {
        let config = PerformanceConfig {
            enabled: true,
            profiling_enabled: true,
            anomaly_detection_enabled: false,
            thresholds: PerformanceThresholds {
                max_response_time_ms: 500,
                max_memory_mb: 300,
                max_cpu_percent: 80.0,
            },
        };

        let mut monitor = PerformanceMonitor::new(config);
        monitor.start().await.unwrap();

        // Record a performance metric
        let metric = PerformanceMetric {
            name: "test.response_time".to_string(),
            value: 150.0,
            unit: "ms".to_string(),
            timestamp: chrono::Utc::now(),
            tags: HashMap::new(),
        };

        monitor.record_metric(metric);

        let metrics = monitor.get_metrics("test.response_time", None);
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].value, 150.0);

        monitor.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_analytics() {
        let config = AnalyticsConfig {
            enabled: true,
            tracking_id: None,
            event_buffer_size: 100,
            flush_interval: chrono::TimeDelta::seconds(60),
        };

        let mut engine = AnalyticsEngine::new(config);
        engine.start().await.unwrap();

        let event = UsageEvent {
            id: EventId::new_v4(),
            event_type: "test_action".to_string(),
            user_id: Some("user123".to_string()),
            session_id: Some("session456".to_string()),
            properties: {
                let mut props = HashMap::new();
                props.insert("feature".to_string(), serde_json::json!("test"));
                props
            },
            timestamp: chrono::Utc::now(),
        };

        engine.track_event(event);

        let events = engine.get_usage_events(Some("test_action"), None, None, Some(10));
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "test_action");

        engine.stop().await.unwrap();
    }

    #[test]
    fn test_dashboard_management() {
        let mut manager = DashboardManager::new();

        let dashboard = Dashboard {
            id: "test-dashboard".to_string(),
            name: "Test Dashboard".to_string(),
            description: "A test dashboard".to_string(),
            panels: vec![],
            tags: vec!["test".to_string()],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        manager.create_dashboard(dashboard.clone());

        let retrieved = manager.get_dashboard("test-dashboard").unwrap();
        assert_eq!(retrieved.name, "Test Dashboard");
    }

    #[test]
    fn test_compliance_reporting() {
        let config = ComplianceConfig {
            enabled: true,
            standards: vec!["SOC2".to_string()],
            reporting_interval: chrono::TimeDelta::days(1),
            audit_log_retention: chrono::TimeDelta::days(2555),
        };

        let engine = ComplianceEngine::new(config);

        let reports = engine.get_compliance_reports(Some("SOC2"), Some(10));
        // Should be empty initially
        assert_eq!(reports.len(), 0);
    }
}
