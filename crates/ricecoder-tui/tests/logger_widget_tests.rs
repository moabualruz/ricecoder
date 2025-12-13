use ricecoder_tui::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Trace < LogLevel::Debug);
        assert!(LogLevel::Debug < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Warn);
        assert!(LogLevel::Warn < LogLevel::Error);
    }

    #[test]
    fn test_log_level_as_str() {
        assert_eq!(LogLevel::Debug.as_str(), "DEBUG");
        assert_eq!(LogLevel::Info.as_str(), "INFO");
        assert_eq!(LogLevel::Error.as_str(), "ERROR");
    }

    #[test]
    fn test_log_entry_creation() {
        let entry = LogEntry::new(LogLevel::Info, "Test message");
        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.message, "Test message");
        assert!(entry.module.is_none());
    }

    #[test]
    fn test_log_entry_with_module() {
        let entry = LogEntry::new(LogLevel::Debug, "Debug message")
            .with_module("test_module");
        assert_eq!(entry.module, Some("test_module".to_string()));
    }

    #[test]
    fn test_log_entry_format() {
        let entry = LogEntry::new(LogLevel::Info, "Test message")
            .with_module("app");
        let formatted = entry.format();
        assert!(formatted.contains("INFO"));
        assert!(formatted.contains("Test message"));
        assert!(formatted.contains("app"));
    }

    #[test]
    fn test_logger_widget_creation() {
        let logger = LoggerWidget::new(100);
        assert_eq!(logger.log_count(), 0);
        assert_eq!(logger.scroll_offset(), 0);
    }

    #[test]
    fn test_logger_widget_log() {
        let mut logger = LoggerWidget::new(100);
        logger.debug("Debug message");
        logger.info("Info message");
        logger.warn("Warning message");
        logger.error("Error message");

        assert_eq!(logger.log_count(), 4);
    }

    #[test]
    fn test_logger_widget_max_logs() {
        let mut logger = LoggerWidget::new(3);
        logger.info("Message 1");
        logger.info("Message 2");
        logger.info("Message 3");
        logger.info("Message 4");

        assert_eq!(logger.log_count(), 3);
    }

    #[test]
    fn test_logger_widget_clear() {
        let mut logger = LoggerWidget::new(100);
        logger.info("Message 1");
        logger.info("Message 2");

        logger.clear();
        assert_eq!(logger.log_count(), 0);
    }

    #[test]
    fn test_logger_widget_filter_by_level() {
        let mut logger = LoggerWidget::new(100);
        logger.debug("Debug message");
        logger.info("Info message");
        logger.warn("Warning message");

        logger.set_min_level(LogLevel::Info);
        let filtered = logger.filtered_logs();
        assert_eq!(filtered.len(), 2); // Info and Warning
    }

    #[test]
    fn test_logger_widget_search_filter() {
        let mut logger = LoggerWidget::new(100);
        logger.info("Hello world");
        logger.info("Goodbye world");
        logger.info("Hello there");

        logger.set_search_filter(Some("Hello".to_string()));
        let filtered = logger.filtered_logs();
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_logger_widget_scroll() {
        let mut logger = LoggerWidget::new(100);
        for i in 0..10 {
            logger.info(format!("Message {}", i));
        }

        logger.scroll_down(5);
        assert_eq!(logger.scroll_offset(), 1);

        logger.scroll_up();
        assert_eq!(logger.scroll_offset(), 0);

        logger.scroll_to_bottom(5);
        assert!(logger.is_at_bottom(5));
    }

    #[test]
    fn test_logger_widget_visible_logs() {
        let mut logger = LoggerWidget::new(100);
        for i in 0..10 {
            logger.info(format!("Message {}", i));
        }

        let visible = logger.visible_logs(5);
        assert_eq!(visible.len(), 5);
    }

    #[test]
    fn test_logger_widget_scroll_percentage() {
        let mut logger = LoggerWidget::new(100);
        for i in 0..10 {
            logger.info(format!("Message {}", i));
        }

        assert_eq!(logger.scroll_percentage(5), 0);

        logger.scroll_to_bottom(5);
        assert_eq!(logger.scroll_percentage(5), 100);
    }

    #[test]
    fn test_logger_widget_auto_scroll() {
        let mut logger = LoggerWidget::new(100);
        logger.set_auto_scroll(true);

        for i in 0..10 {
            logger.info(format!("Message {}", i));
        }

        assert!(logger.is_at_bottom(10));
    }

    #[test]
    fn test_logger_widget_title() {
        let mut logger = LoggerWidget::new(100);
        assert_eq!(logger.title(), "Logs");

        logger.set_title("Debug Logs");
        assert_eq!(logger.title(), "Debug Logs");
    }

    #[test]
    fn test_logger_widget_borders() {
        let mut logger = LoggerWidget::new(100);
        assert!(logger.show_borders());

        logger.set_show_borders(false);
        assert!(!logger.show_borders());
    }
}