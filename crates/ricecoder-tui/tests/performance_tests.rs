use ricecoder_tui::*;
use std::time::Duration;

mod tests {
    use super::*;

    #[test]
    fn test_lazy_message_history_add_and_retrieve() {
        let mut history = LazyMessageHistory::new(LazyLoadConfig {
            chunk_size: 10,
            max_chunks: 2,
        });

        for i in 0..15 {
            history.add_message(format!("Message {}", i));
        }

        assert_eq!(history.total_count(), 15);
        assert_eq!(history.loaded_count(), 15);
    }

    #[test]
    fn test_lazy_message_history_eviction() {
        let mut history = LazyMessageHistory::new(LazyLoadConfig {
            chunk_size: 5,
            max_chunks: 2,
        });

        for i in 0..20 {
            history.add_message(format!("Message {}", i));
        }

        // Should only keep last 10 messages (2 chunks of 5)
        assert_eq!(history.loaded_count(), 10);
        assert_eq!(history.total_count(), 20);
    }

    #[test]
    fn test_lazy_message_history_visible_messages() {
        let mut history = LazyMessageHistory::new(LazyLoadConfig {
            chunk_size: 10,
            max_chunks: 2,
        });

        for i in 0..15 {
            history.add_message(format!("Message {}", i));
        }

        let visible = history.visible_messages(5, 5);
        assert_eq!(visible.len(), 5);
    }

    #[test]
    fn test_diff_render_optimizer_large_diff() {
        let optimizer = DiffRenderOptimizer::new();
        assert!(!optimizer.is_large_diff(500));
        assert!(optimizer.is_large_diff(3000));
    }

    #[test]
    fn test_theme_switch_performance_tracking() {
        let mut perf = ThemeSwitchPerformance::default();
        perf.record_switch(50);
        perf.record_switch(75);

        assert_eq!(perf.last_switch_time_ms, 75);
        assert_eq!(perf.switch_count, 2);
        assert!(perf.is_performant());
    }

    #[test]
    fn test_theme_switch_performance_slow() {
        let mut perf = ThemeSwitchPerformance::default();
        perf.record_switch(150);

        assert!(!perf.is_performant());
    }

    #[test]
    fn test_render_performance_tracker() {
        let mut tracker = RenderPerformanceTracker::new();

        // Record some frame times
        tracker.record_frame(Duration::from_millis(10));
        tracker.record_frame(Duration::from_millis(20));
        tracker.record_frame(Duration::from_millis(30));

        assert_eq!(tracker.frame_count, 3);
        assert_eq!(tracker.dropped_frames, 2); // 20ms and 30ms exceed 16.67ms target

        let avg_frame_time = tracker.average_frame_time();
        assert!(avg_frame_time.as_millis() >= 20); // Should be around 20ms average

        let fps = tracker.current_fps();
        assert!(fps > 0.0 && fps < 60.0); // Should be less than 60 FPS
    }

    #[test]
    fn test_memory_tracker() {
        let tracker = MemoryTracker::new();

        tracker.add_usage(1024);
        assert_eq!(tracker.current_usage(), 1024);
        assert_eq!(tracker.peak_usage(), 1024);

        tracker.add_usage(512);
        assert_eq!(tracker.current_usage(), 1536);
        assert_eq!(tracker.peak_usage(), 1536);

        tracker.subtract_usage(256);
        assert_eq!(tracker.current_usage(), 1280);
        assert_eq!(tracker.peak_usage(), 1536); // Peak should remain
    }

    #[test]
    fn test_virtual_scroll_manager() {
        let mut manager = VirtualScrollManager::new(100, 10);

        assert_eq!(manager.visible_range(), (0, 10));
        assert_eq!(manager.max_scroll_offset(), 90);

        manager.scroll_down(5);
        assert_eq!(manager.visible_range(), (5, 15));

        manager.scroll_up(3);
        assert_eq!(manager.visible_range(), (2, 12));

        // Test bounds checking
        manager.scroll_down(200);
        assert_eq!(manager.scroll_offset, 90);
    }

    #[tokio::test]
    async fn test_content_cache() {
        let cache = ContentCache::<String>::new(3);

        // Test cache miss
        assert!(cache.get("key1").await.is_none());

        // Add item
        cache.put("key1".to_string(), "value1".to_string(), 10).await;
        assert_eq!(cache.get("key1").await, Some("value1".to_string()));

        // Test cache eviction
        cache.put("key2".to_string(), "value2".to_string(), 10).await;
        cache.put("key3".to_string(), "value3".to_string(), 10).await;
        cache.put("key4".to_string(), "value4".to_string(), 10).await;

        // Cache should still have key4 but may have evicted older items
        let stats = cache.stats().await;
        assert_eq!(stats.entries, 3); // Should maintain max size
        assert_eq!(stats.max_entries, 3);
    }

    #[test]
    fn test_history_limits() {
        let limits = HistoryLimits::new()
            .with_max_messages(5000)
            .with_max_sessions(50)
            .with_max_cache_size(50 * 1024 * 1024); // 50MB

        assert_eq!(limits.max_messages, 5000);
        assert_eq!(limits.max_sessions, 50);
        assert_eq!(limits.max_cache_size, 50 * 1024 * 1024);
    }
}