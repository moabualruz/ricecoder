use ricecoder_cli::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_metrics() {
        let metrics = PerformanceMetrics::new(
            "test_op".to_string(),
            Duration::from_millis(100),
            10,
        );

        assert_eq!(metrics.name, "test_op");
        assert_eq!(metrics.duration, Duration::from_millis(100));
        assert_eq!(metrics.iterations, 10);
        assert_eq!(metrics.avg_duration, Duration::from_millis(10));
    }

    #[test]
    fn test_performance_metrics_meets_target() {
        let metrics = PerformanceMetrics::new(
            "test_op".to_string(),
            Duration::from_millis(100),
            10,
        );

        assert!(metrics.meets_target(Duration::from_millis(20)));
        assert!(!metrics.meets_target(Duration::from_millis(5)));
    }

    #[test]
    fn test_profiler_record() {
        let mut profiler = PerformanceProfiler::new();
        profiler.record("op1".to_string(), Duration::from_millis(10));
        profiler.record("op1".to_string(), Duration::from_millis(20));

        let metrics = profiler.average_metrics("op1");
        assert!(metrics.is_some());
        let metrics = metrics.unwrap();
        assert_eq!(metrics.iterations, 2);
    }

    #[test]
    fn test_timer() {
        let timer = Timer::new("test_timer");
        std::thread::sleep(Duration::from_millis(10));
        let elapsed = timer.elapsed();
        assert!(elapsed >= Duration::from_millis(10));
    }
}