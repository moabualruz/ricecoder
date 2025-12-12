//! Monitoring and observability for RiceCoder TUI
//!
//! This module provides comprehensive monitoring capabilities including
//! performance metrics, usage analytics, and system observability.

use crate::error::TuiResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// User experience metrics collector
#[derive(Debug)]
pub struct UserExperienceMetrics {
    response_times: Vec<(String, Duration)>,
    error_rates: HashMap<String, (u32, u32)>, // (errors, total_operations)
    user_satisfaction: Vec<f64>, // 1-5 scale
    feature_adoption: HashMap<String, Instant>, // When features were first used
    workflow_efficiency: Vec<WorkflowMetrics>,
}

impl UserExperienceMetrics {
    /// Create new UX metrics collector
    pub fn new() -> Self {
        Self {
            response_times: Vec::new(),
            error_rates: HashMap::new(),
            user_satisfaction: Vec::new(),
            feature_adoption: HashMap::new(),
            workflow_efficiency: Vec::new(),
        }
    }

    /// Record response time for an operation
    pub fn record_response_time(&mut self, operation: &str, duration: Duration) {
        self.response_times.push((operation.to_string(), duration));

        // Keep only last 1000 samples
        if self.response_times.len() > 1000 {
            self.response_times.remove(0);
        }
    }

    /// Record operation result (success/failure)
    pub fn record_operation_result(&mut self, operation: &str, success: bool) {
        let (errors, total) = self.error_rates.entry(operation.to_string())
            .or_insert((0, 0));

        *total += 1;
        if !success {
            *errors += 1;
        }
    }

    /// Record user satisfaction score (1-5 scale)
    pub fn record_satisfaction(&mut self, score: f64) {
        if (1.0..=5.0).contains(&score) {
            self.user_satisfaction.push(score);

            // Keep only last 100 samples
            if self.user_satisfaction.len() > 100 {
                self.user_satisfaction.remove(0);
            }
        }
    }

    /// Record feature adoption (first use)
    pub fn record_feature_adoption(&mut self, feature: &str) {
        self.feature_adoption.entry(feature.to_string())
            .or_insert_with(Instant::now);
    }

    /// Start tracking a workflow
    pub fn start_workflow(&mut self, workflow_name: &str, total_steps: usize) -> WorkflowTracker {
        WorkflowTracker {
            workflow_name: workflow_name.to_string(),
            total_steps,
            start_time: Instant::now(),
            errors_encountered: 0,
        }
    }

    /// Complete a workflow
    pub fn complete_workflow(&mut self, tracker: WorkflowTracker, steps_completed: usize) {
        let metrics = WorkflowMetrics {
            workflow_name: tracker.workflow_name,
            steps_completed,
            total_steps: tracker.total_steps,
            duration: tracker.start_time.elapsed(),
            errors_encountered: tracker.errors_encountered,
        };

        self.workflow_efficiency.push(metrics);
    }

    /// Get average response time for an operation
    pub fn average_response_time(&self, operation: &str) -> Option<Duration> {
        let times: Vec<_> = self.response_times.iter()
            .filter(|(op, _)| op == operation)
            .map(|(_, time)| *time)
            .collect();

        if times.is_empty() {
            None
        } else {
            let total: Duration = times.into_iter().sum();
            Some(total / times.len() as u32)
        }
    }

    /// Get error rate for an operation
    pub fn error_rate(&self, operation: &str) -> Option<f64> {
        self.error_rates.get(operation).map(|(errors, total)| {
            if *total == 0 {
                0.0
            } else {
                *errors as f64 / *total as f64
            }
        })
    }

    /// Get average user satisfaction
    pub fn average_satisfaction(&self) -> Option<f64> {
        if self.user_satisfaction.is_empty() {
            None
        } else {
            Some(self.user_satisfaction.iter().sum::<f64>() / self.user_satisfaction.len() as f64)
        }
    }

    /// Get workflow completion rate
    pub fn workflow_completion_rate(&self, workflow_name: &str) -> Option<f64> {
        let workflows: Vec<_> = self.workflow_efficiency.iter()
            .filter(|w| w.workflow_name == workflow_name)
            .collect();

        if workflows.is_empty() {
            None
        } else {
            let completed = workflows.iter()
                .filter(|w| w.steps_completed == w.total_steps)
                .count();
            Some(completed as f64 / workflows.len() as f64)
        }
    }

    /// Generate UX metrics report
    pub fn generate_report(&self) -> UserExperienceReport {
        UserExperienceReport {
            average_response_times: self.calculate_avg_response_times(),
            error_rates: self.error_rates.iter()
                .map(|(op, (errors, total))| (op.clone(), *errors as f64 / *total as f64))
                .collect(),
            average_satisfaction: self.average_satisfaction(),
            feature_adoption_count: self.feature_adoption.len(),
            workflow_completion_rates: self.calculate_workflow_completion_rates(),
        }
    }

    fn calculate_avg_response_times(&self) -> HashMap<String, Duration> {
        let mut result = HashMap::new();

        for (operation, _) in &self.response_times {
            if let Some(avg) = self.average_response_time(operation) {
                result.insert(operation.clone(), avg);
            }
        }

        result
    }

    fn calculate_workflow_completion_rates(&self) -> HashMap<String, f64> {
        let mut result = HashMap::new();
        let mut workflow_names = std::collections::HashSet::new();

        for workflow in &self.workflow_efficiency {
            workflow_names.insert(&workflow.workflow_name);
        }

        for name in workflow_names {
            if let Some(rate) = self.workflow_completion_rate(name) {
                result.insert(name.clone(), rate);
            }
        }

        result
    }
}

/// Workflow tracker for measuring workflow efficiency
pub struct WorkflowTracker {
    workflow_name: String,
    total_steps: usize,
    start_time: Instant,
    errors_encountered: usize,
}

impl WorkflowTracker {
    /// Record an error in the workflow
    pub fn record_error(&mut self) {
        self.errors_encountered += 1;
    }
}

#[derive(Debug, Clone)]
pub struct WorkflowMetrics {
    pub workflow_name: String,
    pub steps_completed: usize,
    pub total_steps: usize,
    pub duration: Duration,
    pub errors_encountered: usize,
}

/// Main monitoring system
pub struct MonitoringSystem {
    performance_monitor: PerformanceMonitor,
    usage_analytics: UsageAnalytics,
    metrics_collector: MetricsCollector,
    profiler: PerformanceProfiler,
    ux_metrics: UserExperienceMetrics,
    enabled: bool,
}

impl MonitoringSystem {
    /// Create a new monitoring system
    pub fn new() -> Self {
        Self {
            performance_monitor: PerformanceMonitor::new(),
            usage_analytics: UsageAnalytics::new(),
            metrics_collector: MetricsCollector::new(),
            profiler: PerformanceProfiler::new(),
            ux_metrics: UserExperienceMetrics::new(),
            enabled: true,
        }
    }

    /// Enable or disable monitoring
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Get performance monitor
    pub fn performance(&self) -> &PerformanceMonitor {
        &self.performance_monitor
    }

    /// Get performance monitor mutably
    pub fn performance_mut(&mut self) -> &mut PerformanceMonitor {
        &mut self.performance_monitor
    }

    /// Get usage analytics
    pub fn analytics(&self) -> &UsageAnalytics {
        &self.usage_analytics
    }

    /// Get usage analytics mutably
    pub fn analytics_mut(&mut self) -> &mut UsageAnalytics {
        &mut self.usage_analytics
    }

    /// Get metrics collector
    pub fn metrics(&self) -> &MetricsCollector {
        &self.metrics_collector
    }

    /// Get performance profiler
    pub fn profiler(&self) -> &PerformanceProfiler {
        &self.profiler
    }

    /// Get performance profiler mutably
    pub fn profiler_mut(&mut self) -> &mut PerformanceProfiler {
        &mut self.profiler
    }

    /// Get UX metrics
    pub fn ux_metrics(&self) -> &UserExperienceMetrics {
        &self.ux_metrics
    }

    /// Get UX metrics mutably
    pub fn ux_metrics_mut(&mut self) -> &mut UserExperienceMetrics {
        &mut self.ux_metrics
    }

    /// Record a frame render
    pub fn record_frame_render(&mut self, render_time: Duration) {
        if self.enabled {
            self.performance_monitor.record_frame_render(render_time);
            self.metrics_collector.record_metric("frame_render_time", render_time.as_micros() as f64);
        }
    }

    /// Record a user action
    pub fn record_user_action(&mut self, action: &str, context: Option<&str>) {
        if self.enabled {
            self.usage_analytics.record_action(action, context);
            self.metrics_collector.increment_counter("user_actions");
        }
    }

    /// Record memory usage
    pub fn record_memory_usage(&mut self, bytes: usize) {
        if self.enabled {
            self.performance_monitor.record_memory_usage(bytes);
            self.metrics_collector.record_metric("memory_usage", bytes as f64);
        }
    }

    /// Generate monitoring report
    pub fn generate_report(&self) -> MonitoringReport {
        MonitoringReport {
            performance: self.performance_monitor.generate_report(),
            analytics: self.usage_analytics.generate_report(),
            ux_metrics: self.ux_metrics.generate_report(),
            metrics: self.metrics_collector.get_snapshot(),
            timestamp: std::time::SystemTime::now(),
        }
    }

    /// Export metrics for external monitoring systems
    pub fn export_metrics(&self) -> HashMap<String, serde_json::Value> {
        let mut metrics = HashMap::new();

        // Performance metrics
        if let Some(fps) = self.performance_monitor.current_fps() {
            metrics.insert("performance.fps".to_string(), serde_json::json!(fps));
        }
        if let Some(avg_render_time) = self.performance_monitor.average_render_time() {
            metrics.insert("performance.avg_render_time_ms".to_string(), serde_json::json!(avg_render_time.as_millis()));
        }

        // Usage metrics
        let analytics = self.usage_analytics.generate_report();
        metrics.insert("usage.total_actions".to_string(), serde_json::json!(analytics.total_actions));
        metrics.insert("usage.unique_features".to_string(), serde_json::json!(analytics.unique_features_used));

        // System metrics
        for (key, value) in self.metrics_collector.get_snapshot() {
            metrics.insert(format!("system.{}", key), serde_json::json!(value));
        }

        metrics
    }
}

impl Default for MonitoringSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance monitoring subsystem
pub struct PerformanceMonitor {
    frame_times: Vec<Duration>,
    memory_usage: Vec<(Instant, usize)>,
    fps_calculator: FpsCalculator,
    max_samples: usize,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new() -> Self {
        Self {
            frame_times: Vec::new(),
            memory_usage: Vec::new(),
            fps_calculator: FpsCalculator::new(),
            max_samples: 1000, // Keep last 1000 samples
        }
    }

    /// Record a frame render time
    pub fn record_frame_render(&mut self, render_time: Duration) {
        self.frame_times.push(render_time);
        self.fps_calculator.record_frame();

        // Maintain sample limit
        if self.frame_times.len() > self.max_samples {
            self.frame_times.remove(0);
        }
    }

    /// Record memory usage
    pub fn record_memory_usage(&mut self, bytes: usize) {
        let now = Instant::now();
        self.memory_usage.push((now, bytes));

        // Maintain sample limit and remove old entries
        if self.memory_usage.len() > self.max_samples {
            self.memory_usage.remove(0);
        }

        // Remove entries older than 5 minutes
        let five_minutes_ago = now - Duration::from_secs(300);
        self.memory_usage.retain(|(time, _)| *time > five_minutes_ago);
    }

    /// Get current FPS
    pub fn current_fps(&self) -> Option<f64> {
        self.fps_calculator.current_fps()
    }

    /// Get average render time
    pub fn average_render_time(&self) -> Option<Duration> {
        if self.frame_times.is_empty() {
            return None;
        }

        let total: Duration = self.frame_times.iter().sum();
        Some(total / self.frame_times.len() as u32)
    }

    /// Get peak memory usage in the last period
    pub fn peak_memory_usage(&self) -> Option<usize> {
        self.memory_usage.iter().map(|(_, mem)| *mem).max()
    }

    /// Get current memory usage (most recent)
    pub fn current_memory_usage(&self) -> Option<usize> {
        self.memory_usage.last().map(|(_, mem)| *mem)
    }

    /// Generate performance report
    pub fn generate_report(&self) -> PerformanceReport {
        PerformanceReport {
            current_fps: self.current_fps(),
            average_render_time: self.average_render_time(),
            peak_memory_usage: self.peak_memory_usage(),
            current_memory_usage: self.current_memory_usage(),
            total_frames: self.frame_times.len(),
            memory_samples: self.memory_usage.len(),
        }
    }
}

/// FPS calculator using exponential moving average
pub struct FpsCalculator {
    frame_times: Vec<Instant>,
    ema_fps: Option<f64>,
    alpha: f64, // Smoothing factor
}

impl FpsCalculator {
    pub fn new() -> Self {
        Self {
            frame_times: Vec::new(),
            ema_fps: None,
            alpha: 0.1, // 10% smoothing
        }
    }

    pub fn record_frame(&mut self) {
        let now = Instant::now();
        self.frame_times.push(now);

        // Keep only recent frames (last 60 seconds)
        let one_minute_ago = now - Duration::from_secs(60);
        self.frame_times.retain(|time| *time > one_minute_ago);

        // Calculate FPS using recent frames
        if self.frame_times.len() >= 2 {
            let time_span = now.duration_since(*self.frame_times.first().unwrap());
            let frame_count = self.frame_times.len() as f64;
            let fps = frame_count / time_span.as_secs_f64();

            // Apply exponential moving average
            self.ema_fps = Some(match self.ema_fps {
                Some(prev_fps) => self.alpha * fps + (1.0 - self.alpha) * prev_fps,
                None => fps,
            });
        }
    }

    pub fn current_fps(&self) -> Option<f64> {
        self.ema_fps
    }
}

/// Anonymous usage statistics (privacy-preserving)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnonymousStatistics {
    /// Random session ID (not linked to user identity)
    pub session_id: String,
    /// Application version
    pub app_version: String,
    /// Platform information (OS, architecture)
    pub platform: String,
    /// Session start time (anonymized)
    pub session_start: u64, // Unix timestamp
    /// Total session duration in seconds
    pub session_duration: u64,
    /// Feature usage counts (anonymized)
    pub feature_usage: HashMap<String, u32>,
    /// Performance metrics (anonymized)
    pub performance_metrics: PerformanceMetrics,
    /// Error counts by category (anonymized)
    pub error_counts: HashMap<String, u32>,
    /// UI interaction patterns (anonymized)
    pub ui_patterns: UiPatterns,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub avg_fps: f64,
    pub avg_render_time_ms: f64,
    pub peak_memory_mb: f64,
    pub total_actions: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPatterns {
    pub mode_switches: u32,
    pub command_palette_uses: u32,
    pub file_operations: u32,
    pub search_operations: u32,
}

/// Usage analytics subsystem
pub struct UsageAnalytics {
    actions: HashMap<String, ActionStats>,
    sessions: Vec<SessionData>,
    features_used: std::collections::HashSet<String>,
    start_time: Instant,
    anonymous_stats: Option<AnonymousStatistics>,
    collect_anonymous: bool,
}

#[derive(Debug, Clone)]
pub struct ActionStats {
    pub count: usize,
    pub first_used: Instant,
    pub last_used: Instant,
    pub contexts: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SessionData {
    pub start_time: Instant,
    pub duration: Option<Duration>,
    pub actions_performed: usize,
    pub features_used: Vec<String>,
}

impl UsageAnalytics {
    /// Create new usage analytics
    pub fn new() -> Self {
        Self {
            actions: HashMap::new(),
            sessions: Vec::new(),
            features_used: std::collections::HashSet::new(),
            start_time: Instant::now(),
            anonymous_stats: None,
            collect_anonymous: false,
        }
    }

    /// Enable anonymous statistics collection
    pub fn enable_anonymous_collection(&mut self, app_version: &str, platform: &str) {
        self.collect_anonymous = true;
        self.anonymous_stats = Some(AnonymousStatistics {
            session_id: generate_session_id(),
            app_version: app_version.to_string(),
            platform: platform.to_string(),
            session_start: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            session_duration: 0,
            feature_usage: HashMap::new(),
            performance_metrics: PerformanceMetrics {
                avg_fps: 0.0,
                avg_render_time_ms: 0.0,
                peak_memory_mb: 0.0,
                total_actions: 0,
            },
            error_counts: HashMap::new(),
            ui_patterns: UiPatterns {
                mode_switches: 0,
                command_palette_uses: 0,
                file_operations: 0,
                search_operations: 0,
            },
        });
    }

    /// Disable anonymous statistics collection
    pub fn disable_anonymous_collection(&mut self) {
        self.collect_anonymous = false;
        self.anonymous_stats = None;
    }

    /// Check if anonymous collection is enabled
    pub fn is_anonymous_collection_enabled(&self) -> bool {
        self.collect_anonymous
    }

    /// Record a user action
    pub fn record_action(&mut self, action: &str, context: Option<&str>) {
        let now = Instant::now();

        let stats = self.actions.entry(action.to_string()).or_insert_with(|| ActionStats {
            count: 0,
            first_used: now,
            last_used: now,
            contexts: Vec::new(),
        });

        stats.count += 1;
        stats.last_used = now;

        if let Some(ctx) = context {
            if !stats.contexts.contains(&ctx.to_string()) {
                stats.contexts.push(ctx.to_string());
            }
        }

        // Track feature usage
        self.features_used.insert(action.to_string());

        // Record anonymous statistics if enabled
        if self.collect_anonymous {
            if let Some(stats) = &mut self.anonymous_stats {
                *stats.feature_usage.entry(action.to_string()).or_insert(0) += 1;
                stats.performance_metrics.total_actions += 1;

                // Track UI patterns
                match action {
                    "mode_switch" => stats.ui_patterns.mode_switches += 1,
                    "command_palette" => stats.ui_patterns.command_palette_uses += 1,
                    "file_open" | "file_save" => stats.ui_patterns.file_operations += 1,
                    "search" => stats.ui_patterns.search_operations += 1,
                    _ => {}
                }
            }
        }
    }

    /// Start a new session
    pub fn start_session(&mut self) {
        self.sessions.push(SessionData {
            start_time: Instant::now(),
            duration: None,
            actions_performed: 0,
            features_used: Vec::new(),
        });
    }

    /// End the current session
    pub fn end_session(&mut self) {
        if let Some(session) = self.sessions.last_mut() {
            if session.duration.is_none() {
                session.duration = Some(Instant::now().duration_since(session.start_time));
                session.actions_performed = self.actions.values().map(|s| s.count).sum();
                session.features_used = self.features_used.iter().cloned().collect();
            }
        }

        // Finalize anonymous statistics
        if let Some(stats) = &mut self.anonymous_stats {
            stats.session_duration = self.start_time.elapsed().as_secs();
        }
    }

    /// Update performance metrics for anonymous statistics
    pub fn update_performance_metrics(&mut self, avg_fps: f64, avg_render_time_ms: f64, peak_memory_mb: f64) {
        if let Some(stats) = &mut self.anonymous_stats {
            stats.performance_metrics.avg_fps = avg_fps;
            stats.performance_metrics.avg_render_time_ms = avg_render_time_ms;
            stats.performance_metrics.peak_memory_mb = peak_memory_mb;
        }
    }

    /// Record an error for anonymous statistics
    pub fn record_error(&mut self, error_category: &str) {
        if let Some(stats) = &mut self.anonymous_stats {
            *stats.error_counts.entry(error_category.to_string()).or_insert(0) += 1;
        }
    }

    /// Get anonymous statistics (if collection is enabled)
    pub fn get_anonymous_statistics(&self) -> Option<&AnonymousStatistics> {
        self.anonymous_stats.as_ref()
    }

    /// Export anonymous statistics as JSON (for telemetry)
    pub fn export_anonymous_statistics(&self) -> Option<String> {
        self.anonymous_stats.as_ref()
            .and_then(|stats| serde_json::to_string(stats).ok())
    }

    /// Generate analytics report
    pub fn generate_report(&self) -> AnalyticsReport {
        let total_actions: usize = self.actions.values().map(|s| s.count).sum();
        let unique_actions = self.actions.len();
        let unique_features = self.features_used.len();

        let avg_session_duration = if !self.sessions.is_empty() {
            let total_duration: Duration = self.sessions.iter()
                .filter_map(|s| s.duration)
                .sum();
            total_duration / self.sessions.len() as u32
        } else {
            Duration::new(0, 0)
        };

        AnalyticsReport {
            total_actions,
            unique_actions,
            unique_features_used: unique_features,
            total_sessions: self.sessions.len(),
            average_session_duration: avg_session_duration,
            most_used_actions: self.get_most_used_actions(5),
            session_durations: self.sessions.iter().filter_map(|s| s.duration).collect(),
        }
    }

    /// Get most used actions
    fn get_most_used_actions(&self, limit: usize) -> Vec<(String, usize)> {
        let mut actions: Vec<_> = self.actions.iter()
            .map(|(name, stats)| (name.clone(), stats.count))
            .collect();

        actions.sort_by(|a, b| b.1.cmp(&a.1));
        actions.into_iter().take(limit).collect()
    }
}

/// Metrics collector for system metrics
pub struct MetricsCollector {
    counters: HashMap<String, u64>,
    gauges: HashMap<String, f64>,
    histograms: HashMap<String, Vec<f64>>,
}

impl MetricsCollector {
    /// Create new metrics collector
    pub fn new() -> Self {
        Self {
            counters: HashMap::new(),
            gauges: HashMap::new(),
            histograms: HashMap::new(),
        }
    }

    /// Increment a counter
    pub fn increment_counter(&mut self, name: &str) {
        *self.counters.entry(name.to_string()).or_insert(0) += 1;
    }

    /// Set a gauge value
    pub fn set_gauge(&mut self, name: &str, value: f64) {
        self.gauges.insert(name.to_string(), value);
    }

    /// Record a histogram value
    pub fn record_histogram(&mut self, name: &str, value: f64) {
        self.histograms.entry(name.to_string()).or_insert_with(Vec::new).push(value);

        // Keep only last 1000 samples
        if let Some(samples) = self.histograms.get_mut(name) {
            if samples.len() > 1000 {
                samples.remove(0);
            }
        }
    }

    /// Record a metric (convenience method)
    pub fn record_metric(&mut self, name: &str, value: f64) {
        self.record_histogram(name, value);
        self.set_gauge(&format!("{}_latest", name), value);
    }

    /// Get current snapshot of all metrics
    pub fn get_snapshot(&self) -> HashMap<String, f64> {
        let mut snapshot = HashMap::new();

        // Add counters
        for (name, value) in &self.counters {
            snapshot.insert(format!("counter.{}", name), *value as f64);
        }

        // Add gauges
        for (name, value) in &self.gauges {
            snapshot.insert(format!("gauge.{}", name), *value);
        }

        // Add histogram averages
        for (name, values) in &self.histograms {
            if !values.is_empty() {
                let avg = values.iter().sum::<f64>() / values.len() as f64;
                snapshot.insert(format!("histogram.{}.avg", name), avg);

                let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                snapshot.insert(format!("histogram.{}.min", name), min);

                let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                snapshot.insert(format!("histogram.{}.max", name), max);
            }
        }

        snapshot
    }
}

/// Performance report
#[derive(Debug, Clone)]
pub struct PerformanceReport {
    pub current_fps: Option<f64>,
    pub average_render_time: Option<Duration>,
    pub peak_memory_usage: Option<usize>,
    pub current_memory_usage: Option<usize>,
    pub total_frames: usize,
    pub memory_samples: usize,
}

/// Analytics report
#[derive(Debug, Clone)]
pub struct AnalyticsReport {
    pub total_actions: usize,
    pub unique_actions: usize,
    pub unique_features_used: usize,
    pub total_sessions: usize,
    pub average_session_duration: Duration,
    pub most_used_actions: Vec<(String, usize)>,
    pub session_durations: Vec<Duration>,
}

/// Performance profiler for detailed analysis
pub struct PerformanceProfiler {
    active_profiles: HashMap<String, ProfileSession>,
    completed_profiles: Vec<CompletedProfile>,
}

#[derive(Debug)]
pub struct ProfileSession {
    name: String,
    start_time: Instant,
    checkpoints: Vec<(String, Instant)>,
    memory_samples: Vec<(Instant, usize)>,
}

#[derive(Debug)]
pub struct CompletedProfile {
    pub name: String,
    pub total_duration: Duration,
    pub checkpoints: Vec<(String, Duration)>,
    pub memory_peak: usize,
    pub memory_avg: usize,
}

impl PerformanceProfiler {
    /// Create a new performance profiler
    pub fn new() -> Self {
        Self {
            active_profiles: HashMap::new(),
            completed_profiles: Vec::new(),
        }
    }

    /// Start profiling a code section
    pub fn start_profile(&mut self, name: &str) {
        let session = ProfileSession {
            name: name.to_string(),
            start_time: Instant::now(),
            checkpoints: vec![("start".to_string(), Instant::now())],
            memory_samples: Vec::new(),
        };

        self.active_profiles.insert(name.to_string(), session);
    }

    /// Add a checkpoint to the current profile
    pub fn checkpoint(&mut self, profile_name: &str, checkpoint_name: &str) {
        if let Some(session) = self.active_profiles.get_mut(profile_name) {
            session.checkpoints.push((checkpoint_name.to_string(), Instant::now()));

            // Sample memory usage at checkpoints
            if let Some(memory) = get_memory_usage() {
                session.memory_samples.push((Instant::now(), memory));
            }
        }
    }

    /// End profiling and get results
    pub fn end_profile(&mut self, name: &str) -> Option<CompletedProfile> {
        if let Some(session) = self.active_profiles.remove(name) {
            let total_duration = session.start_time.elapsed();

            let mut checkpoints = Vec::new();
            for (i, (name, time)) in session.checkpoints.iter().enumerate() {
                if i == 0 {
                    checkpoints.push((name.clone(), Duration::new(0, 0)));
                } else {
                    let duration = time.duration_since(session.checkpoints[i-1].1);
                    checkpoints.push((name.clone(), duration));
                }
            }

            let memory_peak = session.memory_samples.iter()
                .map(|(_, mem)| *mem)
                .max()
                .unwrap_or(0);

            let memory_avg = if session.memory_samples.is_empty() {
                0
            } else {
                session.memory_samples.iter()
                    .map(|(_, mem)| *mem)
                    .sum::<usize>() / session.memory_samples.len()
            };

            let profile = CompletedProfile {
                name: session.name,
                total_duration,
                checkpoints,
                memory_peak,
                memory_avg,
            };

            self.completed_profiles.push(profile.clone());
            Some(profile)
        } else {
            None
        }
    }

    /// Get profiling statistics
    pub fn get_stats(&self) -> ProfilingStats {
        let total_profiles = self.completed_profiles.len();
        let active_profiles = self.active_profiles.len();

        let avg_duration = if !self.completed_profiles.is_empty() {
            let total: Duration = self.completed_profiles.iter()
                .map(|p| p.total_duration)
                .sum();
            total / self.completed_profiles.len() as u32
        } else {
            Duration::new(0, 0)
        };

        ProfilingStats {
            total_profiles,
            active_profiles,
            average_duration: avg_duration,
            slowest_profile: self.completed_profiles.iter()
                .max_by_key(|p| p.total_duration)
                .map(|p| p.name.clone()),
        }
    }

    /// Export profiling data as flame graph data
    pub fn export_flame_graph(&self) -> String {
        let mut output = String::new();

        for profile in &self.completed_profiles {
            for (checkpoint_name, duration) in &profile.checkpoints {
                output.push_str(&format!("{} {}\n",
                    checkpoint_name,
                    duration.as_nanos()
                ));
            }
        }

        output
    }
}

/// Profiling statistics
#[derive(Debug)]
pub struct ProfilingStats {
    pub total_profiles: usize,
    pub active_profiles: usize,
    pub average_duration: Duration,
    pub slowest_profile: Option<String>,
}

/// Generate a random session ID for anonymous statistics
fn generate_session_id() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    std::time::SystemTime::now().hash(&mut hasher);
    std::thread::current().id().hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Get current memory usage (simplified implementation)
fn get_memory_usage() -> Option<usize> {
    // This is a placeholder implementation
    // In a real implementation, this would use platform-specific APIs
    // to get actual memory usage statistics

    // For now, return a dummy value or None
    None
}

/// User experience report
#[derive(Debug)]
pub struct UserExperienceReport {
    pub average_response_times: HashMap<String, Duration>,
    pub error_rates: HashMap<String, f64>,
    pub average_satisfaction: Option<f64>,
    pub feature_adoption_count: usize,
    pub workflow_completion_rates: HashMap<String, f64>,
}

/// Complete monitoring report
#[derive(Debug)]
pub struct MonitoringReport {
    pub performance: PerformanceReport,
    pub analytics: AnalyticsReport,
    pub ux_metrics: UserExperienceReport,
    pub metrics: HashMap<String, f64>,
    pub timestamp: std::time::SystemTime,
}

impl MonitoringReport {
    /// Export report as JSON
    pub fn to_json(&self) -> TuiResult<String> {
        serde_json::to_string_pretty(self).map_err(|e| crate::error::TuiError::Config {
            message: format!("Failed to serialize monitoring report: {}", e),
        })
    }

    /// Export report as human-readable text
    pub fn to_text(&self) -> String {
        let mut output = String::new();

        output.push_str("# RiceCoder TUI Monitoring Report\n\n");

        output.push_str("## Performance Metrics\n");
        if let Some(fps) = self.performance.current_fps {
            output.push_str(&format!("- FPS: {:.1}\n", fps));
        }
        if let Some(avg_time) = self.performance.average_render_time {
            output.push_str(&format!("- Average Render Time: {:.2}ms\n", avg_time.as_millis()));
        }
        if let Some(peak_mem) = self.performance.peak_memory_usage {
            output.push_str(&format!("- Peak Memory: {:.2}MB\n", peak_mem as f64 / 1024.0 / 1024.0));
        }
        output.push_str(&format!("- Total Frames: {}\n", self.performance.total_frames));

        output.push_str("\n## Usage Analytics\n");
        output.push_str(&format!("- Total Actions: {}\n", self.analytics.total_actions));
        output.push_str(&format!("- Unique Actions: {}\n", self.analytics.unique_actions));
        output.push_str(&format!("- Features Used: {}\n", self.analytics.unique_features_used));
        output.push_str(&format!("- Sessions: {}\n", self.analytics.total_sessions));
        output.push_str(&format!("- Avg Session Duration: {:.1}s\n", self.analytics.average_session_duration.as_secs_f64()));

        if !self.analytics.most_used_actions.is_empty() {
            output.push_str("\n## Most Used Actions\n");
            for (action, count) in &self.analytics.most_used_actions {
                output.push_str(&format!("- {}: {}\n", action, count));
            }
        }

        output.push_str("\n## User Experience Metrics\n");
        for (operation, time) in &self.ux_metrics.average_response_times {
            output.push_str(&format!("- {} response time: {:.2}ms\n", operation, time.as_millis()));
        }
        for (operation, rate) in &self.ux_metrics.error_rates {
            output.push_str(&format!("- {} error rate: {:.2}%\n", operation, rate * 100.0));
        }
        if let Some(satisfaction) = self.ux_metrics.average_satisfaction {
            output.push_str(&format!("- Average satisfaction: {:.1}/5\n", satisfaction));
        }
        output.push_str(&format!("- Features adopted: {}\n", self.ux_metrics.feature_adoption_count));

        output.push_str("\n## System Metrics\n");
        for (key, value) in &self.metrics {
            output.push_str(&format!("- {}: {:.2}\n", key, value));
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

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