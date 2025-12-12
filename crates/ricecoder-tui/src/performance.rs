//! Performance optimization utilities for the TUI
//!
//! This module provides utilities for optimizing rendering performance,
//! including lazy loading of message history, efficient diff rendering,
//! memory management, and 60 FPS rendering targets.

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

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

/// 60 FPS rendering performance tracker
#[derive(Debug)]
pub struct RenderPerformanceTracker {
    /// Target frame time for 60 FPS (16.67ms)
    pub target_frame_time_ms: f64,
    /// Last frame render time
    pub last_frame_time: Instant,
    /// Frame time history for averaging
    pub frame_times: VecDeque<Duration>,
    /// Maximum frames to keep in history
    pub max_history_size: usize,
    /// Number of frames rendered
    pub frame_count: u64,
    /// Number of dropped frames (over target time)
    pub dropped_frames: u64,
}

impl Default for RenderPerformanceTracker {
    fn default() -> Self {
        Self {
            target_frame_time_ms: 1000.0 / 60.0, // 16.67ms for 60 FPS
            last_frame_time: Instant::now(),
            frame_times: VecDeque::with_capacity(60),
            max_history_size: 60,
            frame_count: 0,
            dropped_frames: 0,
        }
    }
}

impl RenderPerformanceTracker {
    /// Create a new render performance tracker
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a frame render time
    pub fn record_frame(&mut self, render_time: Duration) {
        self.frame_count += 1;

        // Check if frame was dropped (over target time)
        if render_time.as_millis() as f64 > self.target_frame_time_ms {
            self.dropped_frames += 1;
        }

        // Add to history
        self.frame_times.push_back(render_time);
        if self.frame_times.len() > self.max_history_size {
            self.frame_times.pop_front();
        }

        self.last_frame_time = Instant::now();
    }

    /// Get average frame time
    pub fn average_frame_time(&self) -> Duration {
        if self.frame_times.is_empty() {
            return Duration::from_millis(0);
        }

        let total: Duration = self.frame_times.iter().sum();
        total / self.frame_times.len() as u32
    }

    /// Get current FPS
    pub fn current_fps(&self) -> f64 {
        if self.frame_times.is_empty() {
            return 0.0;
        }

        let avg_frame_time = self.average_frame_time();
        if avg_frame_time.as_nanos() == 0 {
            return 0.0;
        }

        1_000_000_000.0 / avg_frame_time.as_nanos() as f64
    }

    /// Check if performance is meeting 60 FPS target
    pub fn is_meeting_target(&self) -> bool {
        self.current_fps() >= 55.0 // Allow some tolerance
    }

    /// Get performance metrics
    pub fn metrics(&self) -> RenderPerformanceMetrics {
        RenderPerformanceMetrics {
            current_fps: self.current_fps(),
            average_frame_time_ms: self.average_frame_time().as_millis() as f64,
            target_fps: 60.0,
            frame_count: self.frame_count,
            dropped_frames: self.dropped_frames,
            drop_rate: if self.frame_count > 0 {
                self.dropped_frames as f64 / self.frame_count as f64
            } else {
                0.0
            },
        }
    }
}

/// Render performance metrics
#[derive(Debug, Clone)]
pub struct RenderPerformanceMetrics {
    pub current_fps: f64,
    pub average_frame_time_ms: f64,
    pub target_fps: f64,
    pub frame_count: u64,
    pub dropped_frames: u64,
    pub drop_rate: f64,
}

/// Memory usage tracker
#[derive(Debug)]
pub struct MemoryTracker {
    /// Current memory usage in bytes
    pub current_usage: Arc<AtomicU64>,
    /// Peak memory usage in bytes
    pub peak_usage: Arc<AtomicU64>,
    /// Memory limit in bytes (0 = no limit)
    pub memory_limit: u64,
    /// Cache sizes by component
    pub cache_sizes: Arc<RwLock<HashMap<String, u64>>>,
}

impl Default for MemoryTracker {
    fn default() -> Self {
        Self {
            current_usage: Arc::new(AtomicU64::new(0)),
            peak_usage: Arc::new(AtomicU64::new(0)),
            memory_limit: 0, // No limit by default
            cache_sizes: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl MemoryTracker {
    /// Create a new memory tracker
    pub fn new() -> Self {
        Self::default()
    }

    /// Set memory limit
    pub fn with_limit(mut self, limit_bytes: u64) -> Self {
        self.memory_limit = limit_bytes;
        self
    }

    /// Update memory usage
    pub fn update_usage(&self, bytes: u64) {
        self.current_usage.store(bytes, Ordering::Relaxed);
        let current_peak = self.peak_usage.load(Ordering::Relaxed);
        if bytes > current_peak {
            self.peak_usage.store(bytes, Ordering::Relaxed);
        }
    }

    /// Add to memory usage
    pub fn add_usage(&self, bytes: u64) {
        let current = self.current_usage.fetch_add(bytes, Ordering::Relaxed);
        let new_total = current + bytes;
        let current_peak = self.peak_usage.load(Ordering::Relaxed);
        if new_total > current_peak {
            self.peak_usage.store(new_total, Ordering::Relaxed);
        }
    }

    /// Subtract from memory usage
    pub fn subtract_usage(&self, bytes: u64) {
        self.current_usage.fetch_sub(bytes, Ordering::Relaxed);
    }

    /// Update cache size for a component
    pub async fn update_cache_size(&self, component: &str, size: u64) {
        let mut cache_sizes = self.cache_sizes.write().await;
        cache_sizes.insert(component.to_string(), size);
    }

    /// Get current memory usage
    pub fn current_usage(&self) -> u64 {
        self.current_usage.load(Ordering::Relaxed)
    }

    /// Get peak memory usage
    pub fn peak_usage(&self) -> u64 {
        self.peak_usage.load(Ordering::Relaxed)
    }

    /// Check if memory limit is exceeded
    pub fn is_limit_exceeded(&self) -> bool {
        if self.memory_limit == 0 {
            return false;
        }
        self.current_usage() > self.memory_limit
    }

    /// Get memory usage breakdown
    pub async fn usage_breakdown(&self) -> HashMap<String, u64> {
        let mut breakdown = HashMap::new();
        let cache_sizes = self.cache_sizes.read().await;

        breakdown.insert("total".to_string(), self.current_usage());
        breakdown.insert("peak".to_string(), self.peak_usage());

        for (component, size) in cache_sizes.iter() {
            breakdown.insert(format!("cache_{}", component), *size);
        }

        breakdown
    }
}

/// Virtual scrolling manager for large content
#[derive(Debug)]
pub struct VirtualScrollManager {
    /// Total number of items
    pub total_items: usize,
    /// Number of visible items
    pub visible_items: usize,
    /// Current scroll offset
    pub scroll_offset: usize,
    /// Item height cache for faster calculations
    pub item_heights: HashMap<usize, u16>,
    /// Average item height for estimation
    pub average_item_height: u16,
}

impl VirtualScrollManager {
    /// Create a new virtual scroll manager
    pub fn new(total_items: usize, visible_items: usize) -> Self {
        Self {
            total_items,
            visible_items,
            scroll_offset: 0,
            item_heights: HashMap::new(),
            average_item_height: 1, // Default to 1 line per item
        }
    }

    /// Set scroll offset
    pub fn set_scroll_offset(&mut self, offset: usize) {
        self.scroll_offset = offset.min(self.max_scroll_offset());
    }

    /// Scroll up by one item
    pub fn scroll_up(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
    }

    /// Scroll down by one item
    pub fn scroll_down(&mut self, amount: usize) {
        self.scroll_offset = (self.scroll_offset + amount).min(self.max_scroll_offset());
    }

    /// Get maximum scroll offset
    pub fn max_scroll_offset(&self) -> usize {
        self.total_items.saturating_sub(self.visible_items)
    }

    /// Get visible item range
    pub fn visible_range(&self) -> (usize, usize) {
        let start = self.scroll_offset;
        let end = (start + self.visible_items).min(self.total_items);
        (start, end)
    }

    /// Check if item is visible
    pub fn is_visible(&self, index: usize) -> bool {
        let (start, end) = self.visible_range();
        index >= start && index < end
    }

    /// Record item height for performance optimization
    pub fn record_item_height(&mut self, index: usize, height: u16) {
        self.item_heights.insert(index, height);
        // Update average height
        let total_height: u32 = self.item_heights.values().map(|&h| h as u32).sum();
        self.average_item_height = (total_height / self.item_heights.len() as u32) as u16;
    }

    /// Estimate total content height
    pub fn estimated_total_height(&self) -> usize {
        self.total_items * self.average_item_height as usize
    }

    /// Get scroll percentage (0.0 to 1.0)
    pub fn scroll_percentage(&self) -> f64 {
        if self.total_items <= self.visible_items {
            0.0
        } else {
            self.scroll_offset as f64 / self.max_scroll_offset() as f64
        }
    }
}

/// Content cache for expensive operations
#[derive(Debug)]
pub struct ContentCache<T> {
    /// Cached content by key
    cache: Arc<RwLock<HashMap<String, CachedItem<T>>>>,
    /// Maximum cache size
    max_size: usize,
    /// Current cache size
    current_size: Arc<AtomicU64>,
}

#[derive(Debug)]
struct CachedItem<T> {
    data: T,
    last_accessed: Instant,
    access_count: u64,
    size_bytes: u64,
}

impl<T> ContentCache<T> {
    /// Create a new content cache
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            max_size,
            current_size: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Get cached item
    pub async fn get(&self, key: &str) -> Option<T>
    where
        T: Clone,
    {
        let mut cache = self.cache.write().await;
        if let Some(item) = cache.get_mut(key) {
            item.last_accessed = Instant::now();
            item.access_count += 1;
            Some(item.data.clone())
        } else {
            None
        }
    }

    /// Put item in cache
    pub async fn put(&self, key: String, data: T, size_bytes: u64) {
        let mut cache = self.cache.write().await;

        // Remove old entries if cache is full
        while cache.len() >= self.max_size {
            if let Some((key_to_remove, _)) = cache
                .iter()
                .min_by_key(|(_, item)| (item.access_count, item.last_accessed))
            {
                if let Some(removed_item) = cache.remove(key_to_remove) {
                    self.current_size.fetch_sub(removed_item.size_bytes, Ordering::Relaxed);
                }
            } else {
                break;
            }
        }

        let item = CachedItem {
            data,
            last_accessed: Instant::now(),
            access_count: 0,
            size_bytes,
        };

        self.current_size.fetch_add(size_bytes, Ordering::Relaxed);
        cache.insert(key, item);
    }

    /// Clear cache
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        self.current_size.store(0, Ordering::Relaxed);
    }

    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        CacheStats {
            entries: cache.len(),
            total_size_bytes: self.current_size.load(Ordering::Relaxed),
            max_entries: self.max_size,
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entries: usize,
    pub total_size_bytes: u64,
    pub max_entries: usize,
}

/// Configurable history limits
#[derive(Debug, Clone)]
pub struct HistoryLimits {
    /// Maximum number of messages in history
    pub max_messages: usize,
    /// Maximum number of sessions to keep
    pub max_sessions: usize,
    /// Maximum number of undo operations
    pub max_undo_steps: usize,
    /// Maximum cache size in bytes
    pub max_cache_size: u64,
    /// Maximum memory usage in bytes
    pub max_memory_usage: u64,
}

impl Default for HistoryLimits {
    fn default() -> Self {
        Self {
            max_messages: 10000,
            max_sessions: 100,
            max_undo_steps: 100,
            max_cache_size: 100 * 1024 * 1024, // 100MB
            max_memory_usage: 500 * 1024 * 1024, // 500MB
        }
    }
}

impl HistoryLimits {
    /// Create history limits with custom values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum messages
    pub fn with_max_messages(mut self, max: usize) -> Self {
        self.max_messages = max;
        self
    }

    /// Set maximum sessions
    pub fn with_max_sessions(mut self, max: usize) -> Self {
        self.max_sessions = max;
        self
    }

    /// Set maximum undo steps
    pub fn with_max_undo_steps(mut self, max: usize) -> Self {
        self.max_undo_steps = max;
        self
    }

    /// Set maximum cache size
    pub fn with_max_cache_size(mut self, max_bytes: u64) -> Self {
        self.max_cache_size = max_bytes;
        self
    }

    /// Set maximum memory usage
    pub fn with_max_memory_usage(mut self, max_bytes: u64) -> Self {
        self.max_memory_usage = max_bytes;
        self
    }
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
