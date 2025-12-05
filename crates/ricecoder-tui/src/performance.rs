//! Performance optimization utilities for the TUI
//!
//! This module provides utilities for optimizing rendering performance,
//! including lazy loading of message history and efficient diff rendering.

use std::collections::VecDeque;

/// Configuration for lazy loading
#[derive(Debug, Clone)]
pub struct LazyLoadConfig {
    /// Number of messages to load per chunk
    pub chunk_size: usize,
    /// Maximum number of chunks to keep in memory
    pub max_chunks: usize,
}

impl Default for LazyLoadConfig {
    fn default() -> Self {
        Self {
            chunk_size: 50,
            max_chunks: 10,
        }
    }
}

/// Lazy-loaded message history
#[derive(Debug, Clone)]
pub struct LazyMessageHistory {
    /// Currently loaded messages
    messages: VecDeque<String>,
    /// Configuration
    config: LazyLoadConfig,
    /// Total number of messages (including unloaded)
    total_count: usize,
    /// Index of the first loaded message
    first_loaded_index: usize,
}

impl LazyMessageHistory {
    /// Create a new lazy message history
    pub fn new(config: LazyLoadConfig) -> Self {
        Self {
            messages: VecDeque::with_capacity(config.chunk_size * config.max_chunks),
            config,
            total_count: 0,
            first_loaded_index: 0,
        }
    }

    /// Add a message to the history
    pub fn add_message(&mut self, message: String) {
        self.messages.push_back(message);
        self.total_count += 1;

        // Evict old messages if we exceed max capacity
        let max_capacity = self.config.chunk_size * self.config.max_chunks;
        while self.messages.len() > max_capacity {
            self.messages.pop_front();
            self.first_loaded_index += 1;
        }
    }

    /// Get visible messages (for rendering)
    pub fn visible_messages(&self, start_index: usize, count: usize) -> Vec<&String> {
        let end_index = (start_index + count).min(self.total_count);
        let relative_start = start_index.saturating_sub(self.first_loaded_index);
        let relative_end = end_index.saturating_sub(self.first_loaded_index);

        if relative_start >= self.messages.len() || relative_end <= relative_start {
            return Vec::new();
        }

        self.messages
            .iter()
            .skip(relative_start)
            .take(relative_end - relative_start)
            .collect()
    }

    /// Get total message count
    pub fn total_count(&self) -> usize {
        self.total_count
    }

    /// Get number of loaded messages
    pub fn loaded_count(&self) -> usize {
        self.messages.len()
    }

    /// Clear all messages
    pub fn clear(&mut self) {
        self.messages.clear();
        self.total_count = 0;
        self.first_loaded_index = 0;
    }
}

/// Diff rendering optimizer for large files
#[derive(Debug, Clone)]
pub struct DiffRenderOptimizer {
    /// Maximum lines to render at once
    pub max_lines_per_render: usize,
    /// Whether to use syntax highlighting (can be expensive)
    pub enable_syntax_highlighting: bool,
}

impl Default for DiffRenderOptimizer {
    fn default() -> Self {
        Self {
            max_lines_per_render: 1000,
            enable_syntax_highlighting: true,
        }
    }
}

impl DiffRenderOptimizer {
    /// Create a new diff render optimizer
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a diff is too large for full rendering
    pub fn is_large_diff(&self, total_lines: usize) -> bool {
        total_lines > self.max_lines_per_render * 2
    }

    /// Get the recommended chunk size for rendering
    pub fn recommended_chunk_size(&self, total_lines: usize) -> usize {
        if self.is_large_diff(total_lines) {
            self.max_lines_per_render / 2
        } else {
            total_lines
        }
    }

    /// Disable syntax highlighting for very large diffs
    pub fn should_disable_syntax_highlighting(&self, total_lines: usize) -> bool {
        total_lines > self.max_lines_per_render * 5
    }
}

/// Theme switching performance tracker
#[derive(Debug, Clone, Default)]
pub struct ThemeSwitchPerformance {
    /// Last theme switch time in milliseconds
    pub last_switch_time_ms: u64,
    /// Average theme switch time in milliseconds
    pub average_switch_time_ms: u64,
    /// Number of theme switches
    pub switch_count: u32,
}

impl ThemeSwitchPerformance {
    /// Record a theme switch time
    pub fn record_switch(&mut self, time_ms: u64) {
        self.last_switch_time_ms = time_ms;
        self.average_switch_time_ms = (self.average_switch_time_ms * self.switch_count as u64
            + time_ms)
            / (self.switch_count as u64 + 1);
        self.switch_count += 1;
    }

    /// Check if theme switching is performant
    pub fn is_performant(&self) -> bool {
        self.last_switch_time_ms < 100 && self.average_switch_time_ms < 100
    }
}

#[cfg(test)]
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
}
