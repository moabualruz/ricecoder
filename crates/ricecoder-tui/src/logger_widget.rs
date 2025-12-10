//! Logger widget for displaying debug and info logs
//!
//! This module provides a widget for displaying application logs with filtering by level,
//! scrolling, and search capabilities.

use std::collections::VecDeque;

/// Log level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// Trace level
    Trace,
    /// Debug level
    Debug,
    /// Info level
    Info,
    /// Warning level
    Warn,
    /// Error level
    Error,
}

impl LogLevel {
    /// Get the string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }

    /// Get the color code for the level
    pub fn color_code(&self) -> &'static str {
        match self {
            LogLevel::Trace => "\x1b[90m",    // Gray
            LogLevel::Debug => "\x1b[36m",    // Cyan
            LogLevel::Info => "\x1b[32m",     // Green
            LogLevel::Warn => "\x1b[33m",     // Yellow
            LogLevel::Error => "\x1b[31m",    // Red
        }
    }
}

/// Log entry
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// Log level
    pub level: LogLevel,
    /// Log message
    pub message: String,
    /// Timestamp
    pub timestamp: String,
    /// Source module
    pub module: Option<String>,
}

impl LogEntry {
    /// Create a new log entry
    pub fn new(level: LogLevel, message: impl Into<String>) -> Self {
        Self {
            level,
            message: message.into(),
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
            module: None,
        }
    }

    /// Set the module
    pub fn with_module(mut self, module: impl Into<String>) -> Self {
        self.module = Some(module.into());
        self
    }

    /// Get the formatted log line
    pub fn format(&self) -> String {
        let module = self.module.as_deref().unwrap_or("app");
        format!(
            "[{}] {} {} - {}",
            self.timestamp, self.level.as_str(), module, self.message
        )
    }
}

/// Logger widget for displaying logs
pub struct LoggerWidget {
    /// Log entries (stored in a deque for efficient removal from front)
    logs: VecDeque<LogEntry>,
    /// Maximum number of logs to keep
    max_logs: usize,
    /// Current scroll offset
    scroll_offset: usize,
    /// Minimum log level to display
    min_level: LogLevel,
    /// Search filter
    search_filter: Option<String>,
    /// Title for the widget
    title: String,
    /// Whether to show borders
    show_borders: bool,
    /// Whether to auto-scroll to bottom
    auto_scroll: bool,
}

impl LoggerWidget {
    /// Create a new logger widget
    pub fn new(max_logs: usize) -> Self {
        Self {
            logs: VecDeque::new(),
            max_logs,
            scroll_offset: 0,
            min_level: LogLevel::Debug,
            search_filter: None,
            title: "Logs".to_string(),
            show_borders: true,
            auto_scroll: true,
        }
    }

    /// Add a log entry
    pub fn log(&mut self, entry: LogEntry) {
        self.logs.push_back(entry);

        // Remove oldest logs if we exceed max
        while self.logs.len() > self.max_logs {
            self.logs.pop_front();
        }

        // Auto-scroll to bottom
        if self.auto_scroll {
            self.scroll_to_bottom(10);
        }
    }

    /// Log a debug message
    pub fn debug(&mut self, message: impl Into<String>) {
        self.log(LogEntry::new(LogLevel::Debug, message));
    }

    /// Log an info message
    pub fn info(&mut self, message: impl Into<String>) {
        self.log(LogEntry::new(LogLevel::Info, message));
    }

    /// Log a warning message
    pub fn warn(&mut self, message: impl Into<String>) {
        self.log(LogEntry::new(LogLevel::Warn, message));
    }

    /// Log an error message
    pub fn error(&mut self, message: impl Into<String>) {
        self.log(LogEntry::new(LogLevel::Error, message));
    }

    /// Clear all logs
    pub fn clear(&mut self) {
        self.logs.clear();
        self.scroll_offset = 0;
    }

    /// Get the number of logs
    pub fn log_count(&self) -> usize {
        self.logs.len()
    }

    /// Get filtered logs
    pub fn filtered_logs(&self) -> Vec<&LogEntry> {
        self.logs
            .iter()
            .filter(|entry| {
                // Filter by level
                if entry.level < self.min_level {
                    return false;
                }

                // Filter by search term
                if let Some(ref search) = self.search_filter {
                    if !entry.message.to_lowercase().contains(&search.to_lowercase()) {
                        return false;
                    }
                }

                true
            })
            .collect()
    }

    /// Get visible logs
    pub fn visible_logs(&self, height: usize) -> Vec<&LogEntry> {
        let filtered = self.filtered_logs();
        filtered
            .iter()
            .skip(self.scroll_offset)
            .take(height)
            .copied()
            .collect()
    }

    /// Set the minimum log level
    pub fn set_min_level(&mut self, level: LogLevel) {
        self.min_level = level;
    }

    /// Set the search filter
    pub fn set_search_filter(&mut self, filter: Option<String>) {
        self.search_filter = filter;
    }

    /// Scroll up
    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    /// Scroll down
    pub fn scroll_down(&mut self, height: usize) {
        let filtered = self.filtered_logs();
        let max_scroll = filtered.len().saturating_sub(height);
        if self.scroll_offset < max_scroll {
            self.scroll_offset += 1;
        }
    }

    /// Scroll to top
    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    /// Scroll to bottom
    pub fn scroll_to_bottom(&mut self, height: usize) {
        let filtered = self.filtered_logs();
        let max_scroll = filtered.len().saturating_sub(height);
        self.scroll_offset = max_scroll;
    }

    /// Get the current scroll offset
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Set auto-scroll
    pub fn set_auto_scroll(&mut self, auto_scroll: bool) {
        self.auto_scroll = auto_scroll;
    }

    /// Set the title
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
    }

    /// Set whether to show borders
    pub fn set_show_borders(&mut self, show: bool) {
        self.show_borders = show;
    }

    /// Get the title
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Check if borders are shown
    pub fn show_borders(&self) -> bool {
        self.show_borders
    }

    /// Get the scroll percentage
    pub fn scroll_percentage(&self, height: usize) -> u8 {
        let filtered = self.filtered_logs();
        if filtered.is_empty() {
            return 100;
        }

        let max_scroll = filtered.len().saturating_sub(height);
        if max_scroll == 0 {
            return 100;
        }

        ((self.scroll_offset as f32 / max_scroll as f32) * 100.0) as u8
    }

    /// Check if at the top
    pub fn is_at_top(&self) -> bool {
        self.scroll_offset == 0
    }

    /// Check if at the bottom
    pub fn is_at_bottom(&self, height: usize) -> bool {
        let filtered = self.filtered_logs();
        let max_scroll = filtered.len().saturating_sub(height);
        self.scroll_offset >= max_scroll
    }

    /// Get all logs
    pub fn logs(&self) -> &VecDeque<LogEntry> {
        &self.logs
    }

    /// Get the minimum log level
    pub fn min_level(&self) -> LogLevel {
        self.min_level
    }

    /// Get the search filter
    pub fn search_filter(&self) -> Option<&str> {
        self.search_filter.as_deref()
    }
}

impl Default for LoggerWidget {
    fn default() -> Self {
        Self::new(1000)
    }
}

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
