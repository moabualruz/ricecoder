//! Health checking for LSP servers

use std::time::{Duration, Instant};

use tracing::{debug, warn};

use crate::types::HealthStatus;

/// Performs health checks on LSP servers
pub struct HealthChecker {
    /// Last successful health check time
    last_check: Option<Instant>,
    /// Health check interval
    check_interval: Duration,
    /// Number of consecutive failures
    failure_count: u32,
    /// Maximum consecutive failures before marking unhealthy
    max_failures: u32,
}

impl HealthChecker {
    /// Create a new health checker
    pub fn new(check_interval: Duration) -> Self {
        Self {
            last_check: None,
            check_interval,
            failure_count: 0,
            max_failures: 3,
        }
    }

    /// Check if a health check is due
    pub fn is_check_due(&self) -> bool {
        match self.last_check {
            None => true,
            Some(last) => last.elapsed() >= self.check_interval,
        }
    }

    /// Record a successful health check
    pub fn record_success(&mut self, latency: Duration) -> HealthStatus {
        self.last_check = Some(Instant::now());
        self.failure_count = 0;
        debug!(latency_ms = latency.as_millis(), "Health check passed");
        HealthStatus::Healthy { latency }
    }

    /// Record a failed health check
    pub fn record_failure(&mut self, reason: String) -> HealthStatus {
        self.failure_count += 1;
        self.last_check = Some(Instant::now());

        warn!(
            failure_count = self.failure_count,
            max_failures = self.max_failures,
            reason = %reason,
            "Health check failed"
        );

        HealthStatus::Unhealthy { reason }
    }

    /// Check if the server should be marked as unhealthy
    pub fn is_unhealthy(&self) -> bool {
        self.failure_count >= self.max_failures
    }

    /// Reset health check state
    pub fn reset(&mut self) {
        self.last_check = None;
        self.failure_count = 0;
    }

    /// Get the number of consecutive failures
    pub fn failure_count(&self) -> u32 {
        self.failure_count
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new(Duration::from_secs(30))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_checker_creation() {
        let checker = HealthChecker::new(Duration::from_secs(30));
        assert!(checker.is_check_due());
        assert_eq!(checker.failure_count(), 0);
        assert!(!checker.is_unhealthy());
    }

    #[test]
    fn test_health_check_success() {
        let mut checker = HealthChecker::new(Duration::from_secs(30));
        let status = checker.record_success(Duration::from_millis(50));

        match status {
            HealthStatus::Healthy { latency } => {
                assert_eq!(latency, Duration::from_millis(50));
            }
            _ => panic!("Expected Healthy status"),
        }

        assert_eq!(checker.failure_count(), 0);
        assert!(!checker.is_unhealthy());
    }

    #[test]
    fn test_health_check_failures() {
        let mut checker = HealthChecker::new(Duration::from_secs(30));

        // First failure
        checker.record_failure("timeout".to_string());
        assert_eq!(checker.failure_count(), 1);
        assert!(!checker.is_unhealthy());

        // Second failure
        checker.record_failure("timeout".to_string());
        assert_eq!(checker.failure_count(), 2);
        assert!(!checker.is_unhealthy());

        // Third failure - should mark as unhealthy
        checker.record_failure("timeout".to_string());
        assert_eq!(checker.failure_count(), 3);
        assert!(checker.is_unhealthy());
    }

    #[test]
    fn test_health_check_reset() {
        let mut checker = HealthChecker::new(Duration::from_secs(30));

        checker.record_failure("timeout".to_string());
        assert_eq!(checker.failure_count(), 1);

        checker.reset();
        assert_eq!(checker.failure_count(), 0);
        assert!(checker.is_check_due());
    }
}
