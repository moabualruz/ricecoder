use std::{thread, time::Duration};

use ricecoder_tui::*;

mod tests {
    use super::*;

    #[test]
    fn test_performance_monitor_fps() {
        let mut monitor = PerformanceMonitor::new();

        // Simulate 60 frames per second for 1 second
        for _ in 0..60 {
            monitor.record_frame_render(Duration::from_millis(16)); // ~60 FPS
            thread::sleep(Duration::from_millis(16));
        }

        let fps = monitor.current_fps();
        assert!(fps.is_some());
        assert!((fps.unwrap() - 60.0).abs() < 5.0); // Allow some variance
    }

    #[test]
    fn test_usage_analytics_actions() {
        let mut analytics = UsageAnalytics::new();

        analytics.record_action("open_file", Some("project"));
        analytics.record_action("open_file", Some("user"));
        analytics.record_action("save_file", None);

        let report = analytics.generate_report();
        assert_eq!(report.total_actions, 3);
        assert_eq!(report.unique_actions, 2);
        assert_eq!(report.unique_features_used, 2);
    }

    #[test]
    fn test_metrics_collector() {
        let mut collector = MetricsCollector::new();

        collector.increment_counter("test_counter");
        collector.increment_counter("test_counter");
        collector.set_gauge("test_gauge", 42.0);
        collector.record_metric("test_histogram", 1.0);
        collector.record_metric("test_histogram", 2.0);
        collector.record_metric("test_histogram", 3.0);

        let snapshot = collector.get_snapshot();
        assert_eq!(snapshot.get("counter.test_counter"), Some(&2.0));
        assert_eq!(snapshot.get("gauge.test_gauge"), Some(&42.0));
        assert_eq!(snapshot.get("histogram.test_histogram.avg"), Some(&2.0));
    }

    #[test]
    fn test_monitoring_system_integration() {
        let mut system = MonitoringSystem::new();

        // Record some metrics
        system.record_frame_render(Duration::from_millis(16));
        system.record_user_action("test_action", Some("test_context"));
        system.record_memory_usage(1024 * 1024); // 1MB

        let report = system.generate_report();

        // Check that metrics were recorded
        assert!(report.performance.total_frames > 0);
        assert_eq!(report.analytics.total_actions, 1);
        assert!(report.metrics.contains_key("system.memory_usage"));
    }
}
