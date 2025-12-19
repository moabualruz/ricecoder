//! Monitoring and telemetry for RiceGrep operations
//!
//! This module provides monitoring capabilities inspired by Refact's
//! StatisticsService and telemetry collection.

use crate::error::RiceGrepError;
use crate::search::{SearchResults, SearchQuery};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::{Duration, Instant};

/// Telemetry record for search operations
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchTelemetryRecord {
    /// Timestamp of the search
    pub timestamp: i64,
    /// Search query details
    pub query: String,
    /// Search duration in milliseconds
    pub duration_ms: u64,
    /// Number of matches found
    pub matches_found: usize,
    /// Files searched
    pub files_searched: usize,
    /// AI features used
    pub ai_used: bool,
    /// AI reranking applied
    pub ai_reranked: bool,
    /// Error occurred
    pub had_error: bool,
    /// Error message if any
    pub error_message: Option<String>,
}

/// Statistics service for collecting and managing telemetry (inspired by Refact's StatisticsService)
pub struct StatisticsService {
    /// In-memory storage of telemetry records
    records: Arc<Mutex<Vec<SearchTelemetryRecord>>>,
    /// Maximum number of records to keep
    max_records: usize,
    /// Performance metrics (inspired by Refact's monitoring)
    performance_metrics: Arc<Mutex<PerformanceMetrics>>,
}

/// Performance metrics (inspired by Refact's monitoring systems)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PerformanceMetrics {
    /// Total searches performed
    pub total_searches: u64,
    /// Total AI-enhanced searches
    pub ai_searches: u64,
    /// Average search time in milliseconds
    pub avg_search_time_ms: f64,
    /// Total search time in milliseconds
    pub total_search_time_ms: u64,
    /// Total matches found
    pub total_matches: u64,
    /// Total files searched
    pub total_files_searched: u64,
    /// Cache hit rate
    pub cache_hit_rate: f64,
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
    /// Last updated timestamp
    pub last_updated: i64,
}

/// Usage analytics for understanding user behavior
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UsageAnalytics {
    /// Feature usage counts
    pub feature_usage: HashMap<String, u64>,
    /// Query pattern analysis
    pub query_patterns: HashMap<String, u64>,
    /// Error frequency by type
    pub error_counts: HashMap<String, u64>,
    /// Performance distribution
    pub performance_distribution: PerformanceDistribution,
    /// User engagement metrics
    pub engagement_metrics: EngagementMetrics,
}

/// Performance distribution statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PerformanceDistribution {
    /// Fast searches (< 100ms)
    pub fast_searches: u64,
    /// Medium searches (100ms - 1s)
    pub medium_searches: u64,
    /// Slow searches (> 1s)
    pub slow_searches: u64,
    /// Very slow searches (> 10s)
    pub very_slow_searches: u64,
}

/// User engagement metrics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EngagementMetrics {
    /// Average session duration
    pub avg_session_duration: Duration,
    /// Feature adoption rates
    pub feature_adoption: HashMap<String, f64>,
    /// User retention patterns
    pub retention_patterns: HashMap<String, u64>,
}

/// Benchmarking suite for performance testing
pub struct BenchmarkSuite {
    /// Benchmark configurations
    benchmarks: Vec<BenchmarkConfig>,
    /// Benchmark results
    results: Arc<Mutex<HashMap<String, BenchmarkResult>>>,
}

/// Benchmark configuration
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    /// Benchmark name
    pub name: String,
    /// Test queries
    pub queries: Vec<String>,
    /// Test files/directories
    pub test_paths: Vec<String>,
    /// Number of iterations
    pub iterations: usize,
    /// Warmup iterations
    pub warmup_iterations: usize,
}

/// Benchmark result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BenchmarkResult {
    /// Benchmark name
    pub name: String,
    /// Total execution time
    pub total_time: Duration,
    /// Average query time
    pub avg_query_time: Duration,
    /// Min query time
    pub min_query_time: Duration,
    /// Max query time
    pub max_query_time: Duration,
    /// Queries per second
    pub qps: f64,
    /// Memory usage in MB
    pub memory_usage_mb: f64,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl StatisticsService {
    /// Create a new statistics service (inspired by Refact's StatisticsService)
    pub fn new(max_records: usize) -> Self {
        Self {
            records: Arc::new(Mutex::new(Vec::new())),
            max_records,
            performance_metrics: Arc::new(Mutex::new(PerformanceMetrics {
                total_searches: 0,
                ai_searches: 0,
                avg_search_time_ms: 0.0,
                total_search_time_ms: 0,
                total_matches: 0,
                total_files_searched: 0,
                cache_hit_rate: 0.0,
                memory_usage_bytes: 0,
                last_updated: chrono::Utc::now().timestamp(),
            })),
        }
    }

    /// Record a search operation
    pub async fn record_search(
        &self,
        query: &SearchQuery,
        results: &Result<SearchResults, RiceGrepError>,
        duration: std::time::Duration,
    ) {
        let record = SearchTelemetryRecord {
            timestamp: chrono::Utc::now().timestamp(),
            query: query.pattern.clone(),
            duration_ms: duration.as_millis() as u64,
            matches_found: results.as_ref().map(|r| r.total_matches).unwrap_or(0),
            files_searched: results.as_ref().map(|r| r.files_searched).unwrap_or(0),
            ai_used: query.ai_enhanced,
            ai_reranked: results.as_ref().map(|r| r.ai_reranked).unwrap_or(false),
            had_error: results.is_err(),
            error_message: results.as_ref().err().map(|e| e.to_string()),
        };

        let mut records = self.records.lock().await;
        records.push(record);

        // Keep only the most recent records
        let excess = records.len().saturating_sub(self.max_records);
        if excess > 0 {
            records.drain(0..excess);
        }
    }

    /// Get recent telemetry records
    pub async fn get_recent_records(&self, limit: usize) -> Vec<SearchTelemetryRecord> {
        let records = self.records.lock().await;
        records.iter().rev().take(limit).cloned().collect()
    }

    /// Get statistics summary
    pub async fn get_summary(&self) -> HashMap<String, serde_json::Value> {
        let records = self.records.lock().await;
        let total_searches = records.len();
        let ai_searches = records.iter().filter(|r| r.ai_used).count();
        let error_searches = records.iter().filter(|r| r.had_error).count();
        let avg_duration = if !records.is_empty() {
            records.iter().map(|r| r.duration_ms).sum::<u64>() / records.len() as u64
        } else {
            0
        };

        let mut summary = HashMap::new();
        summary.insert("total_searches".to_string(), total_searches.into());
        summary.insert("ai_searches".to_string(), ai_searches.into());
        summary.insert("error_searches".to_string(), error_searches.into());
        summary.insert("avg_duration_ms".to_string(), avg_duration.into());
        summary
    }

    /// Export telemetry data (inspired by Refact's telemetry export)
    pub async fn export_telemetry(&self) -> Result<String, RiceGrepError> {
        let records = self.records.lock().await;
        serde_json::to_string_pretty(&*records)
            .map_err(|e| RiceGrepError::Config { message: format!("Failed to serialize telemetry: {}", e) })
    }

    /// Get performance metrics (inspired by Refact's monitoring)
    pub async fn get_performance_metrics(&self) -> PerformanceMetrics {
        self.performance_metrics.lock().await.clone()
    }

    /// Update performance metrics
    pub async fn update_performance_metrics(&self, search_time_ms: u64, matches_found: usize, ai_used: bool) {
        let mut metrics = self.performance_metrics.lock().await;
        metrics.total_searches += 1;
        metrics.total_search_time_ms += search_time_ms;
        metrics.total_matches += matches_found as u64;

        if ai_used {
            metrics.ai_searches += 1;
        }

        // Update average search time
        metrics.avg_search_time_ms = metrics.total_search_time_ms as f64 / metrics.total_searches as f64;

        // Update memory usage (simplified)
        metrics.memory_usage_bytes = self.estimate_memory_usage().await;

        metrics.last_updated = chrono::Utc::now().timestamp();
    }

    /// Estimate memory usage (simplified version of Refact's monitoring)
    async fn estimate_memory_usage(&self) -> u64 {
        // In a real implementation, this would use system APIs to measure memory usage
        // For now, return a placeholder
        let records = self.records.lock().await;
        (records.len() * std::mem::size_of::<SearchTelemetryRecord>()) as u64
    }

    /// Health check (inspired by Refact's watchdog system)
    pub async fn health_check(&self) -> HealthStatus {
        let metrics = self.performance_metrics.lock().await;
        let records = self.records.lock().await;

        let is_healthy = records.len() <= self.max_records &&
                        metrics.last_updated > chrono::Utc::now().timestamp() - 300; // 5 minutes

        HealthStatus {
            healthy: is_healthy,
            total_searches: metrics.total_searches,
            avg_response_time_ms: metrics.avg_search_time_ms,
            memory_usage_bytes: metrics.memory_usage_bytes,
            last_activity: metrics.last_updated,
        }
    }
}

/// Health status for monitoring (inspired by Refact's health checks)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HealthStatus {
    /// Whether the service is healthy
    pub healthy: bool,
    /// Total searches performed
    pub total_searches: u64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
    /// Last activity timestamp
    pub last_activity: i64,
}

impl StatisticsService {
    /// Get usage analytics
    pub async fn get_usage_analytics(&self) -> Result<UsageAnalytics, RiceGrepError> {
        let records = self.records.lock().await;

        let mut feature_usage = HashMap::new();
        let mut query_patterns = HashMap::new();
        let mut error_counts = HashMap::new();
        let mut performance_dist = PerformanceDistribution {
            fast_searches: 0,
            medium_searches: 0,
            slow_searches: 0,
            very_slow_searches: 0,
        };

        for record in records.iter() {
            // Feature usage
            if record.ai_used {
                *feature_usage.entry("ai_enhanced".to_string()).or_insert(0) += 1;
            }
            if record.ai_reranked {
                *feature_usage.entry("ai_reranked".to_string()).or_insert(0) += 1;
            }

            // Query patterns (simplified categorization)
            if record.query.contains("function") || record.query.contains("fn ") {
                *query_patterns.entry("function_search".to_string()).or_insert(0) += 1;
            } else if record.query.contains("class") || record.query.contains("struct") {
                *query_patterns.entry("type_search".to_string()).or_insert(0) += 1;
            } else if record.query.contains("error") || record.query.contains("exception") {
                *query_patterns.entry("error_search".to_string()).or_insert(0) += 1;
            } else {
                *query_patterns.entry("general_search".to_string()).or_insert(0) += 1;
            }

            // Error counts
            if record.had_error {
                if let Some(ref error_msg) = record.error_message {
                    if error_msg.contains("regex") {
                        *error_counts.entry("regex_error".to_string()).or_insert(0) += 1;
                    } else if error_msg.contains("file") {
                        *error_counts.entry("file_error".to_string()).or_insert(0) += 1;
                    } else {
                        *error_counts.entry("other_error".to_string()).or_insert(0) += 1;
                    }
                }
            }

            // Performance distribution
            if record.duration_ms < 100 {
                performance_dist.fast_searches += 1;
            } else if record.duration_ms < 1000 {
                performance_dist.medium_searches += 1;
            } else if record.duration_ms < 10000 {
                performance_dist.slow_searches += 1;
            } else {
                performance_dist.very_slow_searches += 1;
            }
        }

        // Calculate feature adoption rates
        let total_searches = records.len() as f64;
        let mut feature_adoption = HashMap::new();
        for (feature, count) in &feature_usage {
            feature_adoption.insert(feature.clone(), *count as f64 / total_searches);
        }

        Ok(UsageAnalytics {
            feature_usage,
            query_patterns,
            error_counts,
            performance_distribution: performance_dist,
            engagement_metrics: EngagementMetrics {
                avg_session_duration: Duration::from_secs(300), // Placeholder
                feature_adoption,
                retention_patterns: HashMap::new(), // Placeholder
            },
        })
    }

    /// Generate performance report
    pub async fn generate_performance_report(&self) -> Result<PerformanceReport, RiceGrepError> {
        let records = self.records.lock().await;
        let metrics = self.performance_metrics.lock().await;

        let mut query_times = Vec::new();
        let mut match_counts = Vec::new();

        for record in records.iter() {
            query_times.push(record.duration_ms);
            match_counts.push(record.matches_found);
        }

        // Calculate statistics
        let avg_query_time = if !query_times.is_empty() {
            query_times.iter().sum::<u64>() as f64 / query_times.len() as f64
        } else {
            0.0
        };

        let median_query_time = if !query_times.is_empty() {
            let mut sorted = query_times.clone();
            sorted.sort();
            sorted[sorted.len() / 2] as f64
        } else {
            0.0
        };

        let p95_query_time = if !query_times.is_empty() {
            let mut sorted = query_times.clone();
            sorted.sort();
            let index = (sorted.len() as f64 * 0.95) as usize;
            sorted[index.min(sorted.len() - 1)] as f64
        } else {
            0.0
        };

        Ok(PerformanceReport {
            total_searches: metrics.total_searches,
            avg_query_time_ms: avg_query_time,
            median_query_time_ms: median_query_time,
            p95_query_time_ms: p95_query_time,
            total_matches: metrics.total_matches,
            avg_matches_per_search: if metrics.total_searches > 0 {
                metrics.total_matches as f64 / metrics.total_searches as f64
            } else {
                0.0
            },
            cache_hit_rate: metrics.cache_hit_rate,
            timestamp: chrono::Utc::now(),
        })
    }
}

impl Default for StatisticsService {
    fn default() -> Self {
        Self::new(1000) // Keep last 1000 records by default
    }
}

/// Performance report
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PerformanceReport {
    /// Total searches performed
    pub total_searches: u64,
    /// Average query time in milliseconds
    pub avg_query_time_ms: f64,
    /// Median query time in milliseconds
    pub median_query_time_ms: f64,
    /// 95th percentile query time in milliseconds
    pub p95_query_time_ms: f64,
    /// Total matches found
    pub total_matches: u64,
    /// Average matches per search
    pub avg_matches_per_search: f64,
    /// Cache hit rate
    pub cache_hit_rate: f64,
    /// Report timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl BenchmarkSuite {
    /// Create a new benchmark suite
    pub fn new() -> Self {
        Self {
            benchmarks: Vec::new(),
            results: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Add a benchmark configuration
    pub fn add_benchmark(&mut self, config: BenchmarkConfig) {
        self.benchmarks.push(config);
    }

    /// Run all benchmarks
    pub async fn run_all_benchmarks(&self) -> Result<HashMap<String, BenchmarkResult>, RiceGrepError> {
        let mut results = HashMap::new();

        for config in &self.benchmarks {
            let result = self.run_benchmark(config).await?;
            results.insert(config.name.clone(), result);
        }

        // Store results
        let mut stored_results = self.results.lock().await;
        for (name, result) in &results {
            stored_results.insert(name.clone(), result.clone());
        }

        Ok(results)
    }

    /// Run a single benchmark
    async fn run_benchmark(&self, config: &BenchmarkConfig) -> Result<BenchmarkResult, RiceGrepError> {
        let start_time = Instant::now();
        let mut query_times = Vec::new();

        // Warmup iterations
        for _ in 0..config.warmup_iterations {
            for query in &config.queries {
                // Create a mock search engine for benchmarking
                // In a real implementation, this would use the actual search engine
                tokio::time::sleep(Duration::from_millis(1)).await; // Mock delay
            }
        }

        // Actual benchmark iterations
        for _ in 0..config.iterations {
            for query in &config.queries {
                let query_start = Instant::now();

                // Mock search operation
                // In a real implementation, this would perform actual searches
                tokio::time::sleep(Duration::from_millis((query.len() % 10) as u64)).await;

                let query_time = query_start.elapsed();
                query_times.push(query_time);
            }
        }

        let total_time = start_time.elapsed();

        // Calculate statistics
        let total_queries = query_times.len() as f64;
        let avg_query_time = query_times.iter().sum::<Duration>() / query_times.len() as u32;
        let min_query_time = query_times.iter().min().unwrap_or(&Duration::ZERO);
        let max_query_time = query_times.iter().max().unwrap_or(&Duration::ZERO);
        let qps = total_queries / total_time.as_secs_f64();

        Ok(BenchmarkResult {
            name: config.name.clone(),
            total_time,
            avg_query_time,
            min_query_time: *min_query_time,
            max_query_time: *max_query_time,
            qps,
            memory_usage_mb: 50.0, // Placeholder
            timestamp: chrono::Utc::now(),
        })
    }

    /// Get benchmark results
    pub async fn get_results(&self) -> HashMap<String, BenchmarkResult> {
        self.results.lock().await.clone()
    }

    /// Get specific benchmark result
    pub async fn get_result(&self, name: &str) -> Option<BenchmarkResult> {
        self.results.lock().await.get(name).cloned()
    }
}

impl Default for BenchmarkSuite {
    fn default() -> Self {
        Self::new()
    }
}