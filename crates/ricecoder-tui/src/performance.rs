//! Performance optimization utilities for the TUI
//!
//! This module provides utilities for optimizing rendering performance,
//! including lazy loading of message history, efficient diff rendering,
//! memory management, and 60 FPS rendering targets.

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use crate::CancellationToken;

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
            let key_to_remove = cache
                .iter()
                .min_by_key(|(_, item)| (item.access_count, item.last_accessed))
                .map(|(key, _)| key.clone());

            if let Some(key) = key_to_remove {
                if let Some(removed_item) = cache.remove(&key) {
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
            max_size: self.max_size,
            size_bytes: self.current_size.load(Ordering::Relaxed) as u64,
            hit_rate: 0.0, // TODO: Implement hit rate tracking
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entries: usize,
    pub max_size: usize,
    pub size_bytes: u64,
    pub hit_rate: f64,
}

/// Background job system for async processing
pub struct JobQueue {
    /// Job queue with priority ordering
    queue: std::collections::BinaryHeap<Job>,
    /// Active jobs being processed
    active_jobs: std::collections::HashMap<JobId, ActiveJob>,
    /// Job completion callbacks
    completion_callbacks: std::collections::HashMap<JobId, Box<dyn Fn(JobResult) + Send + Sync>>,
    /// Progress reporter for job progress tracking
    progress_reporter: Arc<RwLock<ProgressReporter>>,
    /// Maximum concurrent jobs
    max_concurrent: usize,
    /// Job ID counter
    next_id: u64,
}

impl std::fmt::Debug for JobQueue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JobQueue")
            .field("queue", &self.queue)
            .field("active_jobs", &self.active_jobs)
            .field("completion_callbacks", &format!("<{} callbacks>", self.completion_callbacks.len()))
            .field("progress_reporter", &"<progress_reporter>")
            .field("max_concurrent", &self.max_concurrent)
            .field("next_id", &self.next_id)
            .finish()
    }
}

#[derive(Debug)]
pub struct Job {
    pub id: JobId,
    pub priority: JobPriority,
    pub task: JobTask,
    pub created_at: std::time::Instant,
}

impl PartialEq for Job {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Job {}

impl PartialOrd for Job {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Job {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Higher priority first, then by creation time (older first)
        other.priority.cmp(&self.priority)
            .then_with(|| self.created_at.cmp(&other.created_at))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum JobPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

pub enum JobTask {
    /// Generic async function
    Async(Box<dyn FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = JobResult> + Send>> + Send + Sync>),
    /// File operation
    FileOperation { path: std::path::PathBuf, operation: FileOperationType },
    /// Network request
    NetworkRequest { url: String, method: String },
    /// Custom task with data
    Custom { name: String, data: serde_json::Value },
}

impl std::fmt::Debug for JobTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobTask::Async(_) => write!(f, "JobTask::Async(<function>)"),
            JobTask::FileOperation { path, operation } => {
                write!(f, "JobTask::FileOperation {{ path: {:?}, operation: {:?} }}", path, operation)
            }
            JobTask::NetworkRequest { url, method } => {
                write!(f, "JobTask::NetworkRequest {{ url: {:?}, method: {:?} }}", url, method)
            }
            JobTask::Custom { name, data } => {
                write!(f, "JobTask::Custom {{ name: {:?}, data: <json> }}", name)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum FileOperationType {
    Read,
    Write(Vec<u8>),
    Delete,
    Copy(std::path::PathBuf),
}

#[derive(Debug, Clone)]
pub enum JobOutput {
    Data(Vec<u8>),
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct JobId(pub u64);

#[derive(Debug)]
pub struct ActiveJob {
    pub job: Job,
    pub handle: tokio::task::JoinHandle<JobResult>,
    pub cancel_token: CancellationToken,
}

#[derive(Debug, Clone)]
pub enum JobResult {
    Success(serde_json::Value),
    Error(String),
    Cancelled,
}



impl JobQueue {
    /// Create a new job queue
    pub fn new() -> Self {
        Self {
            queue: std::collections::BinaryHeap::new(),
            active_jobs: std::collections::HashMap::new(),
            completion_callbacks: std::collections::HashMap::new(),
            progress_reporter: Arc::new(RwLock::new(ProgressReporter::new())),
            max_concurrent: 5,
            next_id: 0,
        }
    }

    /// Submit a job to the queue
    pub fn submit_job(&mut self, task: JobTask, priority: JobPriority) -> JobId {
        let id = JobId(self.next_id);
        self.next_id += 1;

        let job = Job {
            id: id.clone(),
            priority,
            task,
            created_at: std::time::Instant::now(),
        };

        self.queue.push(job);
        id
    }

    /// Submit a job with completion callback
    pub fn submit_job_with_callback<F>(&mut self, task: JobTask, priority: JobPriority, callback: F) -> JobId
    where
        F: Fn(JobResult) + Send + Sync + 'static,
    {
        let id = self.submit_job(task, priority);
        self.completion_callbacks.insert(id.clone(), Box::new(callback));
        id
    }

    /// Cancel a job
    pub fn cancel_job(&mut self, job_id: &JobId) -> bool {
        if let Some(active_job) = self.active_jobs.remove(job_id) {
            active_job.cancel_token.cancel();
            return true;
        }
        // Remove from queue if not yet started
        self.queue.retain(|job| job.id != *job_id);
        false
    }

    /// Process pending jobs (call this in an async context)
    pub async fn process_jobs(&mut self) {
        // Start new jobs up to the concurrency limit
        while self.active_jobs.len() < self.max_concurrent {
            if let Some(job) = self.queue.pop() {
                self.start_job(job).await;
            } else {
                break;
            }
        }

        // Clean up completed jobs
        let mut completed_jobs = Vec::new();
        for (job_id, active_job) in &self.active_jobs {
            if active_job.handle.is_finished() {
                completed_jobs.push(job_id.clone());
            }
        }

        for job_id in completed_jobs {
            if let Some(active_job) = self.active_jobs.remove(&job_id) {
                let result = active_job.handle.await.unwrap_or(JobResult::Error("Task panicked".to_string()));

                // Call completion callback if registered
                if let Some(callback) = self.completion_callbacks.remove(&job_id) {
                    callback(result);
                }
            }
        }
    }

    /// Start a job execution
    async fn start_job(&mut self, mut job: Job) {
        let cancel_token = CancellationToken::new();
        let cancel_token_clone = cancel_token.clone();
        let progress_reporter = Arc::clone(&self.progress_reporter);
        let job_id = job.id.clone();

        // Take the task out of the job, leaving a dummy task
        let task = std::mem::replace(&mut job.task, JobTask::Custom {
            name: "completed".to_string(),
            data: serde_json::Value::Null,
        });

        let handle = tokio::spawn(async move {
            tokio::select! {
                result = Self::execute_job_task(task, progress_reporter, &job_id) => result,
                _ = cancel_token_clone.cancelled() => JobResult::Cancelled,
            }
        });

        let active_job = ActiveJob {
            job,
            handle,
            cancel_token,
        };

        self.active_jobs.insert(active_job.job.id.clone(), active_job);
    }

    /// Execute a job task with progress reporting
    async fn execute_job_task(task: JobTask, _progress_reporter: Arc<RwLock<ProgressReporter>>, _job_id: &JobId) -> JobResult {
        // TODO: Implement progress reporting
        match task {
            JobTask::Async(future_fn) => {
                let future = future_fn();
                future.await
            }
            JobTask::FileOperation { path, operation } => {
                match operation {
                    FileOperationType::Read => {
                        match tokio::fs::read(&path).await {
                            Ok(data) => JobResult::Success(serde_json::Value::String(String::from_utf8_lossy(&data).to_string())),
                            Err(e) => JobResult::Error(format!("Failed to read file {}: {}", path.display(), e)),
                        }
                    }
                    FileOperationType::Write(data) => {
                        match tokio::fs::write(&path, &data).await {
                            Ok(_) => JobResult::Success(serde_json::Value::Null),
                            Err(e) => JobResult::Error(format!("Failed to write file {}: {}", path.display(), e)),
                        }
                    }
                    FileOperationType::Delete => {
                        match tokio::fs::remove_file(&path).await {
                            Ok(_) => JobResult::Success(serde_json::Value::Null),
                            Err(e) => JobResult::Error(format!("Failed to delete file {}: {}", path.display(), e)),
                        }
                    }
                    FileOperationType::Copy(dest) => {
                        match tokio::fs::copy(&path, &dest).await {
                            Ok(_) => JobResult::Success(serde_json::Value::Null),
                            Err(e) => JobResult::Error(format!("Failed to copy file {} to {}: {}", path.display(), dest.display(), e)),
                        }
                    }
                }
            }
            JobTask::NetworkRequest { url, method } => {
                // TODO: Implement actual network request
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                JobResult::Success(serde_json::Value::String("Network request completed".to_string()))
            }
            JobTask::Custom { name, data } => {
                // TODO: Implement custom task execution
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                JobResult::Success(serde_json::Value::String(format!("Custom task '{}' completed", name)))
            }
        }
    }

    /// Execute file operations
    async fn execute_file_operation(path: std::path::PathBuf, operation: FileOperationType) -> JobResult {
        match operation {
            FileOperationType::Read => {
                match tokio::fs::read(&path).await {
                    Ok(data) => JobResult::Success(serde_json::json!({
                        "operation": "read",
                        "path": path.to_string_lossy(),
                        "size": data.len()
                    })),
                    Err(e) => JobResult::Error(format!("Failed to read file {}: {}", path.display(), e)),
                }
            }
            FileOperationType::Write(data) => {
                match tokio::fs::write(&path, data).await {
                    Ok(()) => JobResult::Success(serde_json::json!({
                        "operation": "write",
                        "path": path.to_string_lossy()
                    })),
                    Err(e) => JobResult::Error(format!("Failed to write file {}: {}", path.display(), e)),
                }
            }
            FileOperationType::Delete => {
                match tokio::fs::remove_file(&path).await {
                    Ok(()) => JobResult::Success(serde_json::json!({
                        "operation": "delete",
                        "path": path.to_string_lossy()
                    })),
                    Err(e) => JobResult::Error(format!("Failed to delete file {}: {}", path.display(), e)),
                }
            }
            FileOperationType::Copy(dest) => {
                match tokio::fs::copy(&path, &dest).await {
                    Ok(bytes) => JobResult::Success(serde_json::json!({
                        "operation": "copy",
                        "from": path.to_string_lossy(),
                        "to": dest.to_string_lossy(),
                        "bytes": bytes
                    })),
                    Err(e) => JobResult::Error(format!("Failed to copy {} to {}: {}", path.display(), dest.display(), e)),
                }
            }
        }
    }

    /// Execute network requests (placeholder)
    async fn execute_network_request(url: String, method: String) -> JobResult {
        // Placeholder for network request execution
        tracing::info!("Executing network request: {} {}", method, url);
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        JobResult::Success(serde_json::json!({
            "method": method,
            "url": url,
            "status": "simulated"
        }))
    }

    /// Get queue statistics
    pub fn stats(&self) -> JobQueueStats {
        JobQueueStats {
            queued_jobs: self.queue.len(),
            active_jobs: self.active_jobs.len(),
            max_concurrent: self.max_concurrent,
            total_submitted: self.next_id as usize,
        }
    }

    /// Get progress reporter
    pub fn progress_reporter(&self) -> &Arc<RwLock<ProgressReporter>> {
        &self.progress_reporter
    }

    /// Get mutable progress reporter
    pub fn progress_reporter_mut(&mut self) -> &Arc<RwLock<ProgressReporter>> {
        &self.progress_reporter
    }

    /// Subscribe to progress updates for a job
    pub async fn subscribe_progress(&self, job_id: &JobId) -> Option<tokio::sync::broadcast::Receiver<ProgressUpdate>> {
        let operation_id = format!("job_{}", job_id.0);
        let reporter = self.progress_reporter.read().await;
        reporter.subscribe(&operation_id)
    }

    /// Clean up completed progress trackers
    pub async fn cleanup_progress(&mut self) {
        let mut reporter = self.progress_reporter.write().await;
        reporter.cleanup_completed();
    }
}

/// Job queue statistics
#[derive(Debug, Clone)]
pub struct JobQueueStats {
    pub queued_jobs: usize,
    pub active_jobs: usize,
    pub max_concurrent: usize,
    pub total_submitted: usize,
}

/// Progress reporting system for background operations
#[derive(Debug)]
pub struct ProgressReporter {
    /// Progress channels for different operations
    channels: std::collections::HashMap<String, tokio::sync::broadcast::Sender<ProgressUpdate>>,
    /// Active progress trackers
    trackers: std::collections::HashMap<String, ProgressTracker>,
}

#[derive(Debug, Clone)]
pub struct ProgressUpdate {
    pub operation_id: String,
    pub progress: f32, // 0.0 to 1.0
    pub message: String,
    pub status: ProgressStatus,
    pub timestamp: std::time::Instant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProgressStatus {
    Starting,
    Running,
    Paused,
    Completed,
    Failed(String),
    Cancelled,
}

#[derive(Debug)]
pub struct ProgressTracker {
    pub operation_id: String,
    pub total_steps: usize,
    pub current_step: usize,
    pub start_time: std::time::Instant,
    pub status: ProgressStatus,
    pub nested_trackers: Vec<ProgressTracker>,
    pub parent_id: Option<String>,
}

impl ProgressReporter {
    /// Create a new progress reporter
    pub fn new() -> Self {
        Self {
            channels: std::collections::HashMap::new(),
            trackers: std::collections::HashMap::new(),
        }
    }

    /// Create a progress tracker for an operation
    pub fn create_tracker(&mut self, operation_id: &str, total_steps: usize) -> &mut ProgressTracker {
        let tracker = ProgressTracker {
            operation_id: operation_id.to_string(),
            total_steps,
            current_step: 0,
            start_time: std::time::Instant::now(),
            status: ProgressStatus::Starting,
            nested_trackers: Vec::new(),
            parent_id: None,
        };

        self.trackers.insert(operation_id.to_string(), tracker);

        // Create broadcast channel for this operation
        let (tx, _) = tokio::sync::broadcast::channel(100);
        self.channels.insert(operation_id.to_string(), tx);

        self.trackers.get_mut(operation_id).unwrap()
    }

    /// Create a nested progress tracker
    pub fn create_nested_tracker(&mut self, parent_id: &str, operation_id: &str, total_steps: usize) -> Option<&mut ProgressTracker> {
        if let Some(parent_tracker) = self.trackers.get_mut(parent_id) {
            let tracker = ProgressTracker {
                operation_id: operation_id.to_string(),
                total_steps,
                current_step: 0,
                start_time: std::time::Instant::now(),
                status: ProgressStatus::Starting,
                nested_trackers: Vec::new(),
                parent_id: Some(parent_id.to_string()),
            };

            parent_tracker.nested_trackers.push(tracker);

            // Create broadcast channel for nested operation
            let (tx, _) = tokio::sync::broadcast::channel(100);
            self.channels.insert(operation_id.to_string(), tx);

            return parent_tracker.nested_trackers.last_mut();
        }
        None
    }

    /// Update progress for an operation
    pub fn update_progress(&mut self, operation_id: &str, step: usize, message: &str) -> Result<(), String> {
        if let Some(tracker) = self.trackers.get_mut(operation_id) {
            tracker.current_step = step.min(tracker.total_steps);
            tracker.status = ProgressStatus::Running;

            let progress = if tracker.total_steps > 0 {
                tracker.current_step as f32 / tracker.total_steps as f32
            } else {
                1.0
            };

            let update = ProgressUpdate {
                operation_id: operation_id.to_string(),
                progress,
                message: message.to_string(),
                status: tracker.status.clone(),
                timestamp: std::time::Instant::now(),
            };

            // Send update via broadcast channel
            if let Some(channel) = self.channels.get(operation_id) {
                let _ = channel.send(update);
            }

            Ok(())
        } else {
            Err(format!("Progress tracker not found: {}", operation_id))
        }
    }

    /// Mark operation as completed
    pub fn complete_operation(&mut self, operation_id: &str, message: &str) -> Result<(), String> {
        if let Some(tracker) = self.trackers.get_mut(operation_id) {
            tracker.status = ProgressStatus::Completed;
            tracker.current_step = tracker.total_steps;

            let update = ProgressUpdate {
                operation_id: operation_id.to_string(),
                progress: 1.0,
                message: message.to_string(),
                status: tracker.status.clone(),
                timestamp: std::time::Instant::now(),
            };

            if let Some(channel) = self.channels.get(operation_id) {
                let _ = channel.send(update);
            }

            Ok(())
        } else {
            Err(format!("Progress tracker not found: {}", operation_id))
        }
    }

    /// Mark operation as failed
    pub fn fail_operation(&mut self, operation_id: &str, error: &str) -> Result<(), String> {
        if let Some(tracker) = self.trackers.get_mut(operation_id) {
            tracker.status = ProgressStatus::Failed(error.to_string());

            let update = ProgressUpdate {
                operation_id: operation_id.to_string(),
                progress: tracker.current_step as f32 / tracker.total_steps as f32,
                message: format!("Failed: {}", error),
                status: tracker.status.clone(),
                timestamp: std::time::Instant::now(),
            };

            if let Some(channel) = self.channels.get(operation_id) {
                let _ = channel.send(update);
            }

            Ok(())
        } else {
            Err(format!("Progress tracker not found: {}", operation_id))
        }
    }

    /// Subscribe to progress updates for an operation
    pub fn subscribe(&self, operation_id: &str) -> Option<tokio::sync::broadcast::Receiver<ProgressUpdate>> {
        self.channels.get(operation_id)
            .map(|tx| tx.subscribe())
    }

    /// Get progress tracker for an operation
    pub fn get_tracker(&self, operation_id: &str) -> Option<&ProgressTracker> {
        self.trackers.get(operation_id)
    }

    /// Get all active trackers
    pub fn active_trackers(&self) -> Vec<&ProgressTracker> {
        self.trackers.values().filter(|t| t.status == ProgressStatus::Running).collect()
    }

    /// Clean up completed trackers
    pub fn cleanup_completed(&mut self) {
        let completed: Vec<String> = self.trackers.iter()
            .filter(|(_, tracker)| {
                matches!(tracker.status, ProgressStatus::Completed | ProgressStatus::Failed(_) | ProgressStatus::Cancelled)
            })
            .map(|(id, _)| id.clone())
            .collect();

        for id in completed {
            self.trackers.remove(&id);
            self.channels.remove(&id);
        }
    }

    /// Get overall progress statistics
    pub fn stats(&self) -> ProgressStats {
        let total_trackers = self.trackers.len();
        let active_trackers = self.active_trackers().len();
        let completed_trackers = self.trackers.values()
            .filter(|t| matches!(t.status, ProgressStatus::Completed))
            .count();
        let failed_trackers = self.trackers.values()
            .filter(|t| matches!(t.status, ProgressStatus::Failed(_)))
            .count();

        ProgressStats {
            total_trackers,
            active_trackers,
            completed_trackers,
            failed_trackers,
        }
    }
}

/// Progress statistics
#[derive(Debug, Clone)]
pub struct ProgressStats {
    pub total_trackers: usize,
    pub active_trackers: usize,
    pub completed_trackers: usize,
    pub failed_trackers: usize,
}

/// Performance profiler for generating flame graphs and profiling data
#[derive(Debug)]
pub struct PerformanceProfiler {
    /// Whether profiling is enabled
    enabled: bool,
    /// Profiling data collection
    spans: Vec<ProfileSpan>,
    /// Current active spans
    active_spans: Vec<ProfileSpan>,
    /// Maximum number of spans to keep
    max_spans: usize,
    /// Start time of profiling session
    session_start: std::time::Instant,
}

#[derive(Debug, Clone)]
pub struct ProfileSpan {
    /// Span name/identifier
    pub name: String,
    /// Start time relative to session start
    pub start_time: std::time::Duration,
    /// Duration of the span
    pub duration: std::time::Duration,
    /// Parent span index (None for root spans)
    pub parent_index: Option<usize>,
    /// Thread ID where span was recorded
    pub thread_id: u64,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

/// CPU usage monitor
#[derive(Debug)]
pub struct CpuMonitor {
    /// CPU usage samples
    samples: VecDeque<CpuSample>,
    /// Maximum samples to keep
    max_samples: usize,
    /// Last CPU measurement time
    last_measurement: std::time::Instant,
}

#[derive(Debug, Clone)]
pub struct CpuSample {
    pub timestamp: std::time::Instant,
    pub user_percent: f64,
    pub system_percent: f64,
    pub total_percent: f64,
    pub thread_count: usize,
}

/// Memory profiler for heap analysis
#[derive(Debug)]
pub struct MemoryProfiler {
    /// Memory allocation samples
    allocations: VecDeque<MemorySample>,
    /// Maximum samples to keep
    max_samples: usize,
    /// Current memory statistics
    current_stats: MemoryStats,
}

#[derive(Debug, Clone)]
pub struct MemorySample {
    pub timestamp: std::time::Instant,
    pub allocated_bytes: u64,
    pub deallocated_bytes: u64,
    pub live_objects: usize,
    pub heap_size: u64,
}

#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub total_allocated: u64,
    pub total_deallocated: u64,
    pub current_live: usize,
    pub peak_heap_size: u64,
    pub allocation_count: u64,
    pub deallocation_count: u64,
}

impl PerformanceProfiler {
    /// Create a new performance profiler
    pub fn new() -> Self {
        Self {
            enabled: false,
            spans: Vec::new(),
            active_spans: Vec::new(),
            max_spans: 10000,
            session_start: std::time::Instant::now(),
        }
    }

    /// Enable profiling
    pub fn enable(&mut self) {
        self.enabled = true;
        self.session_start = std::time::Instant::now();
        self.spans.clear();
        self.active_spans.clear();
    }

    /// Disable profiling
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Check if profiling is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Start a profiling span
    pub fn start_span(profiler: &Arc<Mutex<Self>>, name: &str) -> Option<ProfileSpanHandle> {
        let mut profiler_guard = profiler.lock().unwrap();
        if !profiler_guard.enabled {
            return None;
        }

        let span = ProfileSpan {
            name: name.to_string(),
            start_time: profiler_guard.session_start.elapsed(),
            duration: std::time::Duration::from_nanos(0),
            parent_index: profiler_guard.active_spans.last().map(|_| profiler_guard.spans.len().saturating_sub(1)),
            thread_id: get_current_thread_id(),
            metadata: std::collections::HashMap::new(),
        };

        profiler_guard.active_spans.push(span.clone());
        Some(ProfileSpanHandle {
            profiler: Arc::clone(profiler),
        })
    }

    /// End the current span
    pub fn end_span(&mut self) {
        if !self.enabled {
            return;
        }

        if let Some(span) = self.active_spans.pop() {
            let mut completed_span = span;
            completed_span.duration = self.session_start.elapsed() - completed_span.start_time;

            // Add metadata about duration
            completed_span.metadata.insert(
                "duration_ms".to_string(),
                format!("{:.3}", completed_span.duration.as_millis())
            );

            self.spans.push(completed_span);

            // Limit span count
            if self.spans.len() > self.max_spans {
                // Remove oldest spans (keep most recent)
                let remove_count = self.spans.len() - self.max_spans;
                self.spans.drain(0..remove_count);
            }
        }
    }

    /// Add metadata to current span
    pub fn add_metadata(&mut self, key: &str, value: &str) {
        if let Some(span) = self.active_spans.last_mut() {
            span.metadata.insert(key.to_string(), value.to_string());
        }
    }

    /// Generate flame graph data in folded stack format
    pub fn generate_flame_graph(&self) -> String {
        let mut lines = Vec::new();
        let mut stack = Vec::new();

        for span in &self.spans {
            stack.push(span.name.clone());

            // Format: stack_trace samples
            let stack_trace = stack.join(";");
            let samples = span.duration.as_micros() as u64;

            lines.push(format!("{} {}", stack_trace, samples));
        }

        lines.join("\n")
    }

    /// Get profiling statistics
    pub fn stats(&self) -> ProfileStats {
        let total_spans = self.spans.len();
        let active_spans = self.active_spans.len();
        let session_duration = self.session_start.elapsed();

        let total_duration: std::time::Duration = self.spans.iter()
            .map(|s| s.duration)
            .sum();

        let avg_span_duration = if total_spans > 0 {
            total_duration / total_spans as u32
        } else {
            std::time::Duration::from_nanos(0)
        };

        ProfileStats {
            enabled: self.enabled,
            total_spans,
            active_spans,
            session_duration,
            total_duration,
            avg_span_duration,
        }
    }

    /// Clear all profiling data
    pub fn clear(&mut self) {
        self.spans.clear();
        self.active_spans.clear();
        self.session_start = std::time::Instant::now();
    }

    /// Validate memory safety invariants
    pub fn validate_memory_safety(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Check that active spans don't exceed reasonable limits
        if self.active_spans.len() > self.max_spans {
            return Err(format!("Active spans ({}) exceed maximum allowed ({})",
                             self.active_spans.len(), self.max_spans).into());
        }

        // Check that total spans don't exceed reasonable limits
        if self.spans.len() > self.max_spans * 2 {
            return Err(format!("Total spans ({}) significantly exceed maximum allowed ({})",
                             self.spans.len(), self.max_spans).into());
        }

        // Validate span data integrity
        for span in &self.spans {
            if span.duration.as_nanos() == 0 {
                return Err("Span has zero duration".into());
            }
            if span.start_time > self.session_start.elapsed() {
                return Err("Span start time is in the future".into());
            }
        }

        Ok(())
    }
}

/// RAII handle for profiling spans
pub struct ProfileSpanHandle {
    profiler: Arc<Mutex<PerformanceProfiler>>,
}

impl Drop for ProfileSpanHandle {
    fn drop(&mut self) {
        let mut profiler = self.profiler.lock().unwrap();
        profiler.end_span();
    }
}

/// Profiling statistics
#[derive(Debug, Clone)]
pub struct ProfileStats {
    pub enabled: bool,
    pub total_spans: usize,
    pub active_spans: usize,
    pub session_duration: std::time::Duration,
    pub total_duration: std::time::Duration,
    pub avg_span_duration: std::time::Duration,
}

impl CpuMonitor {
    /// Create a new CPU monitor
    pub fn new() -> Self {
        Self {
            samples: VecDeque::new(),
            max_samples: 100,
            last_measurement: std::time::Instant::now(),
        }
    }

    /// Record a CPU usage sample
    pub fn record_sample(&mut self, user_percent: f64, system_percent: f64, thread_count: usize) {
        let sample = CpuSample {
            timestamp: std::time::Instant::now(),
            user_percent,
            system_percent,
            total_percent: user_percent + system_percent,
            thread_count,
        };

        self.samples.push_back(sample);
        if self.samples.len() > self.max_samples {
            self.samples.pop_front();
        }
        self.last_measurement = std::time::Instant::now();
    }

    /// Get current CPU usage statistics
    pub fn current_stats(&self) -> Option<CpuStats> {
        self.samples.back().map(|sample| CpuStats {
            user_percent: sample.user_percent,
            system_percent: sample.system_percent,
            total_percent: sample.total_percent,
            thread_count: sample.thread_count,
            timestamp: sample.timestamp,
        })
    }

    /// Get CPU usage history
    pub fn history(&self) -> Vec<&CpuSample> {
        self.samples.iter().collect()
    }

    /// Calculate average CPU usage over recent samples
    pub fn average_usage(&self, sample_count: usize) -> Option<CpuStats> {
        let recent_samples: Vec<_> = self.samples.iter().rev().take(sample_count).collect();

        if recent_samples.is_empty() {
            return None;
        }

        let avg_user = recent_samples.iter().map(|s| s.user_percent).sum::<f64>() / recent_samples.len() as f64;
        let avg_system = recent_samples.iter().map(|s| s.system_percent).sum::<f64>() / recent_samples.len() as f64;
        let avg_total = avg_user + avg_system;
        let avg_threads = recent_samples.iter().map(|s| s.thread_count).sum::<usize>() / recent_samples.len();

        Some(CpuStats {
            user_percent: avg_user,
            system_percent: avg_system,
            total_percent: avg_total,
            thread_count: avg_threads,
            timestamp: std::time::Instant::now(),
        })
    }
}

/// CPU usage statistics
#[derive(Debug, Clone)]
pub struct CpuStats {
    pub user_percent: f64,
    pub system_percent: f64,
    pub total_percent: f64,
    pub thread_count: usize,
    pub timestamp: std::time::Instant,
}

impl MemoryProfiler {
    /// Create a new memory profiler
    pub fn new() -> Self {
        Self {
            allocations: VecDeque::new(),
            max_samples: 1000,
            current_stats: MemoryStats {
                total_allocated: 0,
                total_deallocated: 0,
                current_live: 0,
                peak_heap_size: 0,
                allocation_count: 0,
                deallocation_count: 0,
            },
        }
    }

    /// Record a memory allocation
    pub fn record_allocation(&mut self, size: u64, heap_size: u64) {
        self.current_stats.total_allocated += size;
        self.current_stats.current_live += 1;
        self.current_stats.allocation_count += 1;
        self.current_stats.peak_heap_size = self.current_stats.peak_heap_size.max(heap_size);

        let sample = MemorySample {
            timestamp: std::time::Instant::now(),
            allocated_bytes: size,
            deallocated_bytes: 0,
            live_objects: self.current_stats.current_live,
            heap_size,
        };

        self.allocations.push_back(sample);
        if self.allocations.len() > self.max_samples {
            self.allocations.pop_front();
        }
    }

    /// Record a memory deallocation
    pub fn record_deallocation(&mut self, size: u64, heap_size: u64) {
        self.current_stats.total_deallocated += size;
        self.current_stats.current_live = self.current_stats.current_live.saturating_sub(1);
        self.current_stats.deallocation_count += 1;

        let sample = MemorySample {
            timestamp: std::time::Instant::now(),
            allocated_bytes: 0,
            deallocated_bytes: size,
            live_objects: self.current_stats.current_live,
            heap_size,
        };

        self.allocations.push_back(sample);
        if self.allocations.len() > self.max_samples {
            self.allocations.pop_front();
        }
    }

    /// Get current memory statistics
    pub fn current_stats(&self) -> &MemoryStats {
        &self.current_stats
    }

    /// Get memory allocation history
    pub fn history(&self) -> Vec<&MemorySample> {
        self.allocations.iter().collect()
    }

    /// Detect potential memory leaks
    pub fn detect_leaks(&self) -> Vec<MemoryLeak> {
        let mut leaks = Vec::new();

        // Simple heuristic: if live objects keep increasing over time
        let recent_samples: Vec<_> = self.allocations.iter().rev().take(10).collect();

        if recent_samples.len() >= 5 {
            let live_counts: Vec<_> = recent_samples.iter().map(|s| s.live_objects).collect();
            let increasing = live_counts.windows(2).all(|w| w[1] >= w[0]);

            if increasing && *live_counts.last().unwrap() > live_counts[0] * 2 {
                leaks.push(MemoryLeak {
                    severity: LeakSeverity::High,
                    description: "Live object count steadily increasing".to_string(),
                    live_objects: *live_counts.last().unwrap(),
                    trend: "increasing".to_string(),
                });
            }
        }

        leaks
    }
}

/// Memory leak detection result
#[derive(Debug, Clone)]
pub struct MemoryLeak {
    pub severity: LeakSeverity,
    pub description: String,
    pub live_objects: usize,
    pub trend: String,
}

/// Memory leak severity levels
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LeakSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Get current thread ID (simplified implementation)
fn get_current_thread_id() -> u64 {
    // In a real implementation, this would use thread::current().id()
    // For now, return a simple counter or hash
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    std::thread::current().id().hash(&mut hasher);
    hasher.finish()
}

impl ProgressTracker {
    /// Get current progress as a percentage (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        if self.total_steps == 0 {
            return 1.0;
        }
        self.current_step as f32 / self.total_steps as f32
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    /// Estimate remaining time based on current progress
    pub fn estimated_remaining(&self) -> Option<std::time::Duration> {
        if self.current_step == 0 || self.progress() >= 1.0 {
            return None;
        }

        let elapsed = self.elapsed();
        let progress = self.progress();
        let total_estimated = elapsed.div_f32(progress);
        let remaining = total_estimated.saturating_sub(elapsed);

        Some(remaining)
    }

    /// Get combined progress including nested trackers
    pub fn combined_progress(&self) -> f32 {
        if self.nested_trackers.is_empty() {
            return self.progress();
        }

        let self_weight = 0.5; // 50% weight for parent
        let nested_weight = 0.5 / self.nested_trackers.len() as f32; // Equal weight for nested

        let mut total_progress = self.progress() * self_weight;

        for nested in &self.nested_trackers {
            total_progress += nested.combined_progress() * nested_weight;
        }

        total_progress
    }
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


