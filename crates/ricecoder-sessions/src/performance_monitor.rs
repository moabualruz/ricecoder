//! Session performance monitoring and metrics

use crate::error::{SessionError, SessionResult};
use crate::models::Session;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

/// Session performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetrics {
    /// Session ID
    pub session_id: String,
    /// Total number of messages in session
    pub message_count: u64,
    /// Total tokens used (if available)
    pub total_tokens: Option<u64>,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Total session duration
    pub session_duration_seconds: u64,
    /// Number of tool invocations
    pub tool_invocations: u64,
    /// Number of successful tool executions
    pub successful_tool_executions: u64,
    /// Memory usage estimate in bytes
    pub memory_usage_bytes: u64,
    /// Last activity timestamp
    pub last_activity: SystemTime,
    /// Session creation time
    pub created_at: SystemTime,
}

/// Session performance monitor
#[derive(Debug, Clone)]
pub struct SessionPerformanceMonitor {
    metrics: Arc<std::sync::Mutex<HashMap<String, SessionMetrics>>>,
    global_stats: Arc<GlobalSessionStats>,
}

impl SessionPerformanceMonitor {
    /// Create a new performance monitor
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(std::sync::Mutex::new(HashMap::new())),
            global_stats: Arc::new(GlobalSessionStats::new()),
        }
    }

    /// Record session creation
    pub fn record_session_created(&self, session_id: &str) {
        let mut metrics = self.metrics.lock().unwrap();
        let now = SystemTime::now();

        let session_metrics = SessionMetrics {
            session_id: session_id.to_string(),
            message_count: 0,
            total_tokens: None,
            avg_response_time_ms: 0.0,
            session_duration_seconds: 0,
            tool_invocations: 0,
            successful_tool_executions: 0,
            memory_usage_bytes: 0,
            last_activity: now,
            created_at: now,
        };

        metrics.insert(session_id.to_string(), session_metrics);
        self.global_stats.record_session_created();
    }

    /// Record message sent/received
    pub fn record_message(&self, session_id: &str, response_time_ms: Option<f64>) {
        let mut metrics = self.metrics.lock().unwrap();

        if let Some(session_metrics) = metrics.get_mut(session_id) {
            session_metrics.message_count += 1;
            session_metrics.last_activity = SystemTime::now();

            if let Some(rt) = response_time_ms {
                // Update rolling average
                let total_messages = session_metrics.message_count;
                let current_avg = session_metrics.avg_response_time_ms;
                session_metrics.avg_response_time_ms =
                    (current_avg * (total_messages - 1) as f64 + rt) / total_messages as f64;
            }

            // Update session duration
            if let Ok(duration) = session_metrics.last_activity.duration_since(session_metrics.created_at) {
                session_metrics.session_duration_seconds = duration.as_secs();
            }
        }

        self.global_stats.record_message();
    }

    /// Record tool invocation
    pub fn record_tool_invocation(&self, session_id: &str, success: bool) {
        let mut metrics = self.metrics.lock().unwrap();

        if let Some(session_metrics) = metrics.get_mut(session_id) {
            session_metrics.tool_invocations += 1;
            if success {
                session_metrics.successful_tool_executions += 1;
            }
            session_metrics.last_activity = SystemTime::now();
        }

        self.global_stats.record_tool_invocation(success);
    }

    /// Record token usage
    pub fn record_token_usage(&self, session_id: &str, tokens: u64) {
        let mut metrics = self.metrics.lock().unwrap();

        if let Some(session_metrics) = metrics.get_mut(session_id) {
            session_metrics.total_tokens = Some(
                session_metrics.total_tokens.unwrap_or(0) + tokens
            );
        }

        self.global_stats.record_token_usage(tokens);
    }

    /// Update memory usage estimate
    pub fn update_memory_usage(&self, session_id: &str, bytes: u64) {
        let mut metrics = self.metrics.lock().unwrap();

        if let Some(session_metrics) = metrics.get_mut(session_id) {
            session_metrics.memory_usage_bytes = bytes;
        }
    }

    /// Record session deletion
    pub fn record_session_deleted(&self, session_id: &str) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.remove(session_id);
        self.global_stats.record_session_deleted();
    }

    /// Get metrics for a specific session
    pub fn get_session_metrics(&self, session_id: &str) -> Option<SessionMetrics> {
        let metrics = self.metrics.lock().unwrap();
        metrics.get(session_id).cloned()
    }

    /// Get all session metrics
    pub fn get_all_metrics(&self) -> Vec<SessionMetrics> {
        let metrics = self.metrics.lock().unwrap();
        metrics.values().cloned().collect()
    }

    /// Get global statistics
    pub fn get_global_stats(&self) -> GlobalSessionStatsData {
        self.global_stats.get_data()
    }

    /// Clean up metrics for inactive sessions
    pub fn cleanup_inactive_sessions(&self, inactivity_threshold: Duration) -> usize {
        let mut metrics = self.metrics.lock().unwrap();
        let now = SystemTime::now();
        let threshold = now - inactivity_threshold;

        let initial_count = metrics.len();
        metrics.retain(|_, session_metrics| {
            session_metrics.last_activity > threshold
        });

        let removed_count = initial_count - metrics.len();
        self.global_stats.record_cleanup(removed_count);

        removed_count
    }

    /// Get performance summary
    pub fn get_performance_summary(&self) -> SessionPerformanceSummary {
        let metrics = self.metrics.lock().unwrap();
        let global_stats = self.global_stats.get_data();

        let total_sessions = metrics.len();
        let total_messages: u64 = metrics.values().map(|m| m.message_count).sum();
        let avg_response_time: f64 = if total_messages > 0 {
            metrics.values()
                .map(|m| m.avg_response_time_ms * m.message_count as f64)
                .sum::<f64>() / total_messages as f64
        } else {
            0.0
        };

        let total_tool_invocations: u64 = metrics.values().map(|m| m.tool_invocations).sum();
        let total_memory_usage: u64 = metrics.values().map(|m| m.memory_usage_bytes).sum();

        SessionPerformanceSummary {
            total_active_sessions: total_sessions,
            total_messages,
            average_response_time_ms: avg_response_time,
            total_tool_invocations,
            tool_success_rate: global_stats.tool_success_rate(),
            total_memory_usage_bytes: total_memory_usage,
            global_stats,
        }
    }
}

/// Global session statistics
#[derive(Debug)]
struct GlobalSessionStats {
    total_sessions_created: AtomicU64,
    total_sessions_active: AtomicU64,
    total_messages: AtomicU64,
    total_tool_invocations: AtomicU64,
    successful_tool_invocations: AtomicU64,
    total_tokens_used: AtomicU64,
    cleanup_sessions_removed: AtomicU64,
}

impl GlobalSessionStats {
    fn new() -> Self {
        Self {
            total_sessions_created: AtomicU64::new(0),
            total_sessions_active: AtomicU64::new(0),
            total_messages: AtomicU64::new(0),
            total_tool_invocations: AtomicU64::new(0),
            successful_tool_invocations: AtomicU64::new(0),
            total_tokens_used: AtomicU64::new(0),
            cleanup_sessions_removed: AtomicU64::new(0),
        }
    }

    fn record_session_created(&self) {
        self.total_sessions_created.fetch_add(1, Ordering::Relaxed);
        self.total_sessions_active.fetch_add(1, Ordering::Relaxed);
    }

    fn record_session_deleted(&self) {
        if self.total_sessions_active.load(Ordering::Relaxed) > 0 {
            self.total_sessions_active.fetch_sub(1, Ordering::Relaxed);
        }
    }

    fn record_message(&self) {
        self.total_messages.fetch_add(1, Ordering::Relaxed);
    }

    fn record_tool_invocation(&self, success: bool) {
        self.total_tool_invocations.fetch_add(1, Ordering::Relaxed);
        if success {
            self.successful_tool_invocations.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn record_token_usage(&self, tokens: u64) {
        self.total_tokens_used.fetch_add(tokens, Ordering::Relaxed);
    }

    fn record_cleanup(&self, removed_count: usize) {
        self.cleanup_sessions_removed.fetch_add(removed_count as u64, Ordering::Relaxed);
        if self.total_sessions_active.load(Ordering::Relaxed) >= removed_count as u64 {
            self.total_sessions_active.fetch_sub(removed_count as u64, Ordering::Relaxed);
        }
    }

    fn get_data(&self) -> GlobalSessionStatsData {
        GlobalSessionStatsData {
            total_sessions_created: self.total_sessions_created.load(Ordering::Relaxed),
            total_sessions_active: self.total_sessions_active.load(Ordering::Relaxed),
            total_messages: self.total_messages.load(Ordering::Relaxed),
            total_tool_invocations: self.total_tool_invocations.load(Ordering::Relaxed),
            successful_tool_invocations: self.successful_tool_invocations.load(Ordering::Relaxed),
            total_tokens_used: self.total_tokens_used.load(Ordering::Relaxed),
            cleanup_sessions_removed: self.cleanup_sessions_removed.load(Ordering::Relaxed),
        }
    }
}

/// Global session statistics data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSessionStatsData {
    pub total_sessions_created: u64,
    pub total_sessions_active: u64,
    pub total_messages: u64,
    pub total_tool_invocations: u64,
    pub successful_tool_invocations: u64,
    pub total_tokens_used: u64,
    pub cleanup_sessions_removed: u64,
}

impl GlobalSessionStatsData {
    /// Calculate tool success rate
    pub fn tool_success_rate(&self) -> f64 {
        if self.total_tool_invocations == 0 {
            0.0
        } else {
            self.successful_tool_invocations as f64 / self.total_tool_invocations as f64 * 100.0
        }
    }

    /// Calculate average tokens per message
    pub fn avg_tokens_per_message(&self) -> f64 {
        if self.total_messages == 0 {
            0.0
        } else {
            self.total_tokens_used as f64 / self.total_messages as f64
        }
    }
}

/// Session performance summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionPerformanceSummary {
    pub total_active_sessions: usize,
    pub total_messages: u64,
    pub average_response_time_ms: f64,
    pub total_tool_invocations: u64,
    pub tool_success_rate: f64,
    pub total_memory_usage_bytes: u64,
    pub global_stats: GlobalSessionStatsData,
}

impl Default for SessionPerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}