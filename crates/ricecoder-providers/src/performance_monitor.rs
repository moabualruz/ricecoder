//! Provider performance monitoring and optimization

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use crate::error::ProviderError;
use crate::models::ModelInfo;

/// Performance metrics for a provider
#[derive(Debug, Clone)]
pub struct ProviderMetrics {
    /// Provider ID
    pub provider_id: String,
    /// Total number of requests made
    pub total_requests: u64,
    /// Number of successful requests
    pub successful_requests: u64,
    /// Number of failed requests
    pub failed_requests: u64,
    /// Total response time across all requests (milliseconds)
    pub total_response_time_ms: u64,
    /// Average response time (milliseconds)
    pub avg_response_time_ms: f64,
    /// Minimum response time (milliseconds)
    pub min_response_time_ms: u64,
    /// Maximum response time (milliseconds)
    pub max_response_time_ms: u64,
    /// Total tokens used
    pub total_tokens: u64,
    /// Total cost incurred
    pub total_cost: f64,
    /// Last request timestamp
    pub last_request_time: Option<SystemTime>,
    /// Error rate (0.0 to 1.0)
    pub error_rate: f64,
    /// Requests per second (averaged over time window)
    pub requests_per_second: f64,
    /// Token throughput (tokens per second)
    pub tokens_per_second: f64,
}

impl ProviderMetrics {
    /// Create new metrics for a provider
    pub fn new(provider_id: String) -> Self {
        Self {
            provider_id,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            total_response_time_ms: 0,
            avg_response_time_ms: 0.0,
            min_response_time_ms: u64::MAX,
            max_response_time_ms: 0,
            total_tokens: 0,
            total_cost: 0.0,
            last_request_time: None,
            error_rate: 0.0,
            requests_per_second: 0.0,
            tokens_per_second: 0.0,
        }
    }

    /// Calculate success rate (0.0 to 1.0)
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.successful_requests as f64 / self.total_requests as f64
        }
    }

    /// Calculate efficiency score (combines speed, reliability, and cost)
    pub fn efficiency_score(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            let speed_score = 1.0 / (1.0 + self.avg_response_time_ms / 1000.0); // Normalize to 0-1
            let reliability_score = self.success_rate();
            let cost_efficiency = if self.total_cost > 0.0 {
                1.0 / (1.0 + self.total_cost / self.total_requests as f64)
            } else {
                1.0
            };

            // Weighted combination
            (speed_score * 0.4) + (reliability_score * 0.4) + (cost_efficiency * 0.2)
        }
    }

    /// Check if provider is performing well
    pub fn is_performing_well(&self, thresholds: &PerformanceThresholds) -> bool {
        self.avg_response_time_ms <= thresholds.max_avg_response_time_ms as f64
            && self.error_rate <= thresholds.max_error_rate
            && self.success_rate() >= thresholds.min_success_rate
    }
}

/// Performance thresholds for provider evaluation
#[derive(Debug, Clone)]
pub struct PerformanceThresholds {
    pub max_avg_response_time_ms: u64,
    pub max_error_rate: f64,
    pub min_success_rate: f64,
    pub max_cost_per_request: f64,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            max_avg_response_time_ms: 5000, // 5 seconds
            max_error_rate: 0.1,            // 10% error rate
            min_success_rate: 0.9,          // 90% success rate
            max_cost_per_request: 0.01,     // $0.01 per request
        }
    }
}

/// Performance monitor for tracking provider metrics
pub struct ProviderPerformanceMonitor {
    metrics: Arc<std::sync::Mutex<HashMap<String, ProviderMetrics>>>,
    thresholds: PerformanceThresholds,
    time_window: Duration,
}

impl ProviderPerformanceMonitor {
    /// Create a new performance monitor
    pub fn new(thresholds: PerformanceThresholds, time_window: Duration) -> Self {
        Self {
            metrics: Arc::new(std::sync::Mutex::new(HashMap::new())),
            thresholds,
            time_window,
        }
    }

    /// Record a successful request
    pub fn record_success(
        &self,
        provider_id: &str,
        response_time_ms: u64,
        tokens_used: u64,
        cost: f64,
    ) {
        let mut metrics = self.metrics.lock().unwrap();
        let provider_metrics = metrics
            .entry(provider_id.to_string())
            .or_insert_with(|| ProviderMetrics::new(provider_id.to_string()));

        provider_metrics.total_requests += 1;
        provider_metrics.successful_requests += 1;
        provider_metrics.total_response_time_ms += response_time_ms;
        provider_metrics.total_tokens += tokens_used;
        provider_metrics.total_cost += cost;
        provider_metrics.last_request_time = Some(SystemTime::now());

        // Update min/max response times
        provider_metrics.min_response_time_ms =
            provider_metrics.min_response_time_ms.min(response_time_ms);
        provider_metrics.max_response_time_ms =
            provider_metrics.max_response_time_ms.max(response_time_ms);

        // Recalculate averages
        provider_metrics.avg_response_time_ms =
            provider_metrics.total_response_time_ms as f64 / provider_metrics.total_requests as f64;
        provider_metrics.error_rate =
            provider_metrics.failed_requests as f64 / provider_metrics.total_requests as f64;

        // Calculate throughput metrics (simplified)
        if let Some(last_time) = provider_metrics.last_request_time {
            if let Ok(elapsed) = last_time.elapsed() {
                let seconds = elapsed.as_secs_f64();
                if seconds > 0.0 {
                    provider_metrics.requests_per_second =
                        provider_metrics.total_requests as f64 / seconds;
                    provider_metrics.tokens_per_second =
                        provider_metrics.total_tokens as f64 / seconds;
                }
            }
        }
    }

    /// Record a failed request
    pub fn record_failure(&self, provider_id: &str, response_time_ms: u64) {
        let mut metrics = self.metrics.lock().unwrap();
        let provider_metrics = metrics
            .entry(provider_id.to_string())
            .or_insert_with(|| ProviderMetrics::new(provider_id.to_string()));

        provider_metrics.total_requests += 1;
        provider_metrics.failed_requests += 1;
        provider_metrics.total_response_time_ms += response_time_ms;
        provider_metrics.last_request_time = Some(SystemTime::now());

        // Recalculate averages
        provider_metrics.avg_response_time_ms =
            provider_metrics.total_response_time_ms as f64 / provider_metrics.total_requests as f64;
        provider_metrics.error_rate =
            provider_metrics.failed_requests as f64 / provider_metrics.total_requests as f64;
    }

    /// Get metrics for a provider
    pub fn get_metrics(&self, provider_id: &str) -> Option<ProviderMetrics> {
        let metrics = self.metrics.lock().unwrap();
        metrics.get(provider_id).cloned()
    }

    /// Get all provider metrics
    pub fn get_all_metrics(&self) -> HashMap<String, ProviderMetrics> {
        let metrics = self.metrics.lock().unwrap();
        metrics.clone()
    }

    /// Get the best performing provider based on efficiency score
    pub fn get_best_provider(&self, provider_ids: &[String]) -> Option<String> {
        let metrics = self.metrics.lock().unwrap();

        provider_ids
            .iter()
            .filter_map(|id| metrics.get(id).map(|m| (id.clone(), m.efficiency_score())))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(id, _)| id)
    }

    /// Get providers that are performing well
    pub fn get_performing_providers(&self) -> Vec<String> {
        let metrics = self.metrics.lock().unwrap();
        metrics
            .iter()
            .filter(|(_, m)| m.is_performing_well(&self.thresholds))
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Get performance summary
    pub fn get_performance_summary(&self) -> PerformanceSummary {
        let all_metrics: Vec<ProviderMetrics> = {
            let metrics = self.metrics.lock().unwrap();
            metrics.values().cloned().collect()
        };

        let total_providers = all_metrics.len();
        let total_requests: u64 = all_metrics.iter().map(|m| m.total_requests).sum();
        let total_errors: u64 = all_metrics.iter().map(|m| m.failed_requests).sum();
        let avg_response_time = if !all_metrics.is_empty() {
            all_metrics
                .iter()
                .map(|m| m.avg_response_time_ms)
                .sum::<f64>()
                / all_metrics.len() as f64
        } else {
            0.0
        };

        let performing_providers = all_metrics
            .iter()
            .filter(|m| m.is_performing_well(&self.thresholds))
            .count();

        PerformanceSummary {
            total_providers,
            total_requests,
            total_errors,
            avg_response_time_ms: avg_response_time,
            performing_providers,
            overall_error_rate: if total_requests > 0 {
                total_errors as f64 / total_requests as f64
            } else {
                0.0
            },
        }
    }

    /// Reset metrics for a provider
    pub fn reset_metrics(&self, provider_id: &str) {
        let mut metrics = self.metrics.lock().unwrap();
        if let Some(provider_metrics) = metrics.get_mut(provider_id) {
            *provider_metrics = ProviderMetrics::new(provider_id.to_string());
        }
    }

    /// Clean up old metrics data
    pub fn cleanup_old_data(&self, max_age: Duration) {
        let cutoff_time = SystemTime::now() - max_age;
        let mut metrics = self.metrics.lock().unwrap();

        metrics.retain(|_, provider_metrics| {
            if let Some(last_time) = provider_metrics.last_request_time {
                last_time > cutoff_time
            } else {
                // Keep providers that have never been used
                true
            }
        });
    }
}

/// Performance summary across all providers
#[derive(Debug, Clone)]
pub struct PerformanceSummary {
    pub total_providers: usize,
    pub total_requests: u64,
    pub total_errors: u64,
    pub avg_response_time_ms: f64,
    pub performing_providers: usize,
    pub overall_error_rate: f64,
}

impl std::fmt::Display for PerformanceSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Provider Performance Summary:")?;
        writeln!(f, "  Total providers: {}", self.total_providers)?;
        writeln!(f, "  Total requests: {}", self.total_requests)?;
        writeln!(f, "  Total errors: {}", self.total_errors)?;
        writeln!(
            f,
            "  Average response time: {:.2}ms",
            self.avg_response_time_ms
        )?;
        writeln!(
            f,
            "  Performing providers: {}/{}",
            self.performing_providers, self.total_providers
        )?;
        writeln!(
            f,
            "  Overall error rate: {:.2}%",
            self.overall_error_rate * 100.0
        )?;
        Ok(())
    }
}

impl Default for ProviderPerformanceMonitor {
    fn default() -> Self {
        Self::new(PerformanceThresholds::default(), Duration::from_secs(300)) // 5 minutes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_metrics_calculation() {
        let mut metrics = ProviderMetrics::new("test_provider".to_string());

        // Record some successful requests
        metrics.total_requests = 10;
        metrics.successful_requests = 8;
        metrics.failed_requests = 2;
        metrics.total_response_time_ms = 5000; // 5 seconds total
        metrics.avg_response_time_ms = 500.0;
        // Manually set error_rate since it's calculated in record methods
        metrics.error_rate = metrics.failed_requests as f64 / metrics.total_requests as f64;

        assert_eq!(metrics.success_rate(), 0.8);
        assert_eq!(metrics.error_rate, 0.2);
        assert!(metrics.efficiency_score() > 0.0);
    }

    #[test]
    fn test_performance_monitor() {
        let monitor = ProviderPerformanceMonitor::default();

        // Record some requests
        monitor.record_success("provider1", 100, 100, 0.01);
        monitor.record_success("provider1", 200, 150, 0.015);
        monitor.record_failure("provider1", 500);

        let metrics = monitor.get_metrics("provider1").unwrap();
        assert_eq!(metrics.total_requests, 3);
        assert_eq!(metrics.successful_requests, 2);
        assert_eq!(metrics.failed_requests, 1);
        assert_eq!(metrics.total_tokens, 250);
        assert_eq!(metrics.total_cost, 0.025);
    }

    #[test]
    fn test_performance_thresholds() {
        let thresholds = PerformanceThresholds::default();
        let mut metrics = ProviderMetrics::new("test".to_string());

        metrics.avg_response_time_ms = 1000.0; // 1 second
        metrics.error_rate = 0.05; // 5%
        metrics.total_requests = 100;
        metrics.successful_requests = 95; // 95% success rate

        assert!(metrics.is_performing_well(&thresholds));
    }

    #[test]
    fn test_best_provider_selection() {
        let monitor = ProviderPerformanceMonitor::default();

        // Provider 1: Fast but expensive
        monitor.record_success("provider1", 100, 100, 0.02);
        monitor.record_success("provider1", 120, 100, 0.02);

        // Provider 2: Slower but cheaper
        monitor.record_success("provider2", 300, 100, 0.01);
        monitor.record_success("provider2", 350, 100, 0.01);

        let providers = vec!["provider1".to_string(), "provider2".to_string()];
        let best = monitor.get_best_provider(&providers);

        // Provider 1 should be better (faster, even though more expensive)
        // Speed gets 40% weight, cost gets 20% weight in efficiency score
        assert_eq!(best, Some("provider1".to_string()));
    }
}
