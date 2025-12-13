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


