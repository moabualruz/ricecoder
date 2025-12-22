//! Error recovery and resilience mechanisms

use std::time::Duration;

use tracing::{debug, error, info, warn};

use crate::error::{Error, Result};

/// Exponential backoff configuration for reconnection attempts
#[derive(Debug, Clone)]
pub struct BackoffConfig {
    /// Initial delay in milliseconds
    pub initial_delay_ms: u64,
    /// Maximum delay in milliseconds
    pub max_delay_ms: u64,
    /// Multiplier for exponential backoff
    pub multiplier: f64,
    /// Maximum number of retries
    pub max_retries: u32,
}

impl BackoffConfig {
    /// Creates a new backoff configuration with default values
    pub fn new() -> Self {
        Self {
            initial_delay_ms: 100,
            max_delay_ms: 30000,
            multiplier: 2.0,
            max_retries: 5,
        }
    }

    /// Sets the initial delay
    pub fn with_initial_delay(mut self, delay_ms: u64) -> Self {
        self.initial_delay_ms = delay_ms;
        self
    }

    /// Sets the maximum delay
    pub fn with_max_delay(mut self, delay_ms: u64) -> Self {
        self.max_delay_ms = delay_ms;
        self
    }

    /// Sets the multiplier
    pub fn with_multiplier(mut self, multiplier: f64) -> Self {
        self.multiplier = multiplier;
        self
    }

    /// Sets the maximum retries
    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// Calculates the delay for a given retry attempt
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        let delay = (self.initial_delay_ms as f64 * self.multiplier.powi(attempt as i32)) as u64;
        let delay = delay.min(self.max_delay_ms);
        Duration::from_millis(delay)
    }
}

impl Default for BackoffConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Error recovery strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryStrategy {
    /// Retry the operation
    Retry,
    /// Fail immediately
    Fail,
    /// Use fallback/cached data
    Fallback,
    /// Continue with available resources
    GracefulDegradation,
}

/// Determines the recovery strategy for an error
pub fn determine_recovery_strategy(error: &Error) -> RecoveryStrategy {
    match error {
        // Recoverable errors should retry
        Error::TimeoutError(_) => RecoveryStrategy::Retry,
        Error::ConnectionError(_) => RecoveryStrategy::Retry,
        Error::ServerDisconnected(_) => RecoveryStrategy::Retry,
        Error::ExecutionInterrupted => RecoveryStrategy::Retry,

        // Server errors might be recoverable
        Error::ServerError(_) => RecoveryStrategy::Retry,

        // Permanent errors should fail
        Error::ToolNotFound(_) => RecoveryStrategy::Fail,
        Error::PermissionDenied(_) => RecoveryStrategy::Fail,
        Error::NamingConflict(_) => RecoveryStrategy::Fail,
        Error::MultipleNamingConflicts(_) => RecoveryStrategy::Fail,

        // Validation errors should fail
        Error::ValidationError(_) => RecoveryStrategy::Fail,
        Error::ParameterValidationError(_) => RecoveryStrategy::Fail,
        Error::OutputValidationError(_) => RecoveryStrategy::Fail,
        Error::InvalidToolParameters(_) => RecoveryStrategy::Fail,
        Error::InvalidToolOutput(_) => RecoveryStrategy::Fail,

        // Configuration errors might be recoverable with fallback
        Error::ConfigError(_) => RecoveryStrategy::Fallback,
        Error::ConfigValidationError(_) => RecoveryStrategy::Fallback,
        Error::StorageError(_) => RecoveryStrategy::Fallback,

        // Other errors should use graceful degradation
        _ => RecoveryStrategy::GracefulDegradation,
    }
}

/// Retry handler for operations with exponential backoff
pub struct RetryHandler {
    config: BackoffConfig,
}

impl RetryHandler {
    /// Creates a new retry handler with default configuration
    pub fn new() -> Self {
        Self {
            config: BackoffConfig::new(),
        }
    }

    /// Creates a new retry handler with custom configuration
    pub fn with_config(config: BackoffConfig) -> Self {
        Self { config }
    }

    /// Executes an operation with retry logic
    ///
    /// # Arguments
    /// * `operation_name` - Name of the operation for logging
    /// * `operation` - Async closure that performs the operation
    ///
    /// # Returns
    /// Result of the operation or error if all retries fail
    pub async fn execute_with_retry<F, T>(
        &self,
        operation_name: &str,
        mut operation: F,
    ) -> Result<T>
    where
        F: FnMut() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T>>>>,
    {
        let mut attempt = 0;

        loop {
            debug!(
                "Executing operation '{}' (attempt {}/{})",
                operation_name,
                attempt + 1,
                self.config.max_retries + 1
            );

            match operation().await {
                Ok(result) => {
                    if attempt > 0 {
                        info!(
                            "Operation '{}' succeeded after {} retries",
                            operation_name, attempt
                        );
                    }
                    return Ok(result);
                }
                Err(err) => {
                    if !err.is_recoverable() {
                        debug!(
                            "Operation '{}' failed with non-recoverable error: {}",
                            operation_name, err
                        );
                        return Err(err);
                    }

                    if attempt >= self.config.max_retries {
                        error!(
                            "Operation '{}' failed after {} retries: {}",
                            operation_name, attempt, err
                        );
                        return Err(Error::MaxRetriesExceeded(format!(
                            "Operation '{}' failed after {} retries: {}",
                            operation_name, attempt, err
                        )));
                    }

                    let delay = self.config.calculate_delay(attempt);
                    warn!(
                        "Operation '{}' failed (attempt {}): {}. Retrying in {:?}",
                        operation_name,
                        attempt + 1,
                        err,
                        delay
                    );

                    tokio::time::sleep(delay).await;
                    attempt += 1;
                }
            }
        }
    }

    /// Executes an operation with retry logic and timeout
    pub async fn execute_with_retry_and_timeout<F, T>(
        &self,
        operation_name: &str,
        timeout_ms: u64,
        mut operation: F,
    ) -> Result<T>
    where
        F: FnMut() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T>>>>,
    {
        let timeout = Duration::from_millis(timeout_ms);
        let mut attempt = 0;

        loop {
            debug!(
                "Executing operation '{}' with timeout {:?} (attempt {}/{})",
                operation_name,
                timeout,
                attempt + 1,
                self.config.max_retries + 1
            );

            match tokio::time::timeout(timeout, operation()).await {
                Ok(Ok(result)) => {
                    if attempt > 0 {
                        info!(
                            "Operation '{}' succeeded after {} retries",
                            operation_name, attempt
                        );
                    }
                    return Ok(result);
                }
                Ok(Err(err)) => {
                    if !err.is_recoverable() {
                        debug!(
                            "Operation '{}' failed with non-recoverable error: {}",
                            operation_name, err
                        );
                        return Err(err);
                    }

                    if attempt >= self.config.max_retries {
                        error!(
                            "Operation '{}' failed after {} retries: {}",
                            operation_name, attempt, err
                        );
                        return Err(Error::MaxRetriesExceeded(format!(
                            "Operation '{}' failed after {} retries: {}",
                            operation_name, attempt, err
                        )));
                    }

                    let delay = self.config.calculate_delay(attempt);
                    warn!(
                        "Operation '{}' failed (attempt {}): {}. Retrying in {:?}",
                        operation_name,
                        attempt + 1,
                        err,
                        delay
                    );

                    tokio::time::sleep(delay).await;
                    attempt += 1;
                }
                Err(_) => {
                    if attempt >= self.config.max_retries {
                        error!(
                            "Operation '{}' timed out after {} retries",
                            operation_name, attempt
                        );
                        return Err(Error::TimeoutError(timeout_ms));
                    }

                    let delay = self.config.calculate_delay(attempt);
                    warn!(
                        "Operation '{}' timed out (attempt {}). Retrying in {:?}",
                        operation_name,
                        attempt + 1,
                        delay
                    );

                    tokio::time::sleep(delay).await;
                    attempt += 1;
                }
            }
        }
    }
}

impl Default for RetryHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Graceful degradation handler for managing partial failures
#[derive(Debug, Clone)]
pub struct GracefulDegradationHandler {
    /// List of available resources
    available_resources: Vec<String>,
    /// List of unavailable resources
    unavailable_resources: Vec<String>,
}

impl GracefulDegradationHandler {
    /// Creates a new graceful degradation handler
    pub fn new() -> Self {
        Self {
            available_resources: Vec::new(),
            unavailable_resources: Vec::new(),
        }
    }

    /// Marks a resource as available
    pub fn mark_available(&mut self, resource_id: String) {
        self.available_resources.push(resource_id.clone());
        self.unavailable_resources.retain(|r| r != &resource_id);
        info!("Resource marked as available: {}", resource_id);
    }

    /// Marks a resource as unavailable
    pub fn mark_unavailable(&mut self, resource_id: String) {
        self.unavailable_resources.push(resource_id.clone());
        self.available_resources.retain(|r| r != &resource_id);
        warn!("Resource marked as unavailable: {}", resource_id);
    }

    /// Gets the list of available resources
    pub fn get_available_resources(&self) -> Vec<String> {
        self.available_resources.clone()
    }

    /// Gets the list of unavailable resources
    pub fn get_unavailable_resources(&self) -> Vec<String> {
        self.unavailable_resources.clone()
    }

    /// Checks if any resources are available
    pub fn has_available_resources(&self) -> bool {
        !self.available_resources.is_empty()
    }

    /// Gets the availability percentage
    pub fn availability_percentage(&self) -> f64 {
        let total = self.available_resources.len() + self.unavailable_resources.len();
        if total == 0 {
            0.0
        } else {
            (self.available_resources.len() as f64 / total as f64) * 100.0
        }
    }
}

impl Default for GracefulDegradationHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    #[test]
    fn test_backoff_config_default() {
        let config = BackoffConfig::new();
        assert_eq!(config.initial_delay_ms, 100);
        assert_eq!(config.max_delay_ms, 30000);
        assert_eq!(config.multiplier, 2.0);
        assert_eq!(config.max_retries, 5);
    }

    #[test]
    fn test_backoff_config_custom() {
        let config = BackoffConfig::new()
            .with_initial_delay(50)
            .with_max_delay(10000)
            .with_multiplier(1.5)
            .with_max_retries(3);

        assert_eq!(config.initial_delay_ms, 50);
        assert_eq!(config.max_delay_ms, 10000);
        assert_eq!(config.multiplier, 1.5);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_calculate_delay() {
        let config = BackoffConfig::new()
            .with_initial_delay(100)
            .with_max_delay(10000)
            .with_multiplier(2.0);

        assert_eq!(config.calculate_delay(0), Duration::from_millis(100));
        assert_eq!(config.calculate_delay(1), Duration::from_millis(200));
        assert_eq!(config.calculate_delay(2), Duration::from_millis(400));
        assert_eq!(config.calculate_delay(3), Duration::from_millis(800));
        assert_eq!(config.calculate_delay(4), Duration::from_millis(1600));
        assert_eq!(config.calculate_delay(5), Duration::from_millis(3200));
        assert_eq!(config.calculate_delay(6), Duration::from_millis(6400));
        assert_eq!(config.calculate_delay(7), Duration::from_millis(10000)); // Capped at max
    }

    #[test]
    fn test_determine_recovery_strategy_retry() {
        assert_eq!(
            determine_recovery_strategy(&Error::TimeoutError(5000)),
            RecoveryStrategy::Retry
        );
        assert_eq!(
            determine_recovery_strategy(&Error::ConnectionError("test".to_string())),
            RecoveryStrategy::Retry
        );
    }

    #[test]
    fn test_determine_recovery_strategy_fail() {
        assert_eq!(
            determine_recovery_strategy(&Error::ToolNotFound("test".to_string())),
            RecoveryStrategy::Fail
        );
        assert_eq!(
            determine_recovery_strategy(&Error::PermissionDenied("test".to_string())),
            RecoveryStrategy::Fail
        );
    }

    #[test]
    fn test_determine_recovery_strategy_fallback() {
        assert_eq!(
            determine_recovery_strategy(&Error::ConfigError("test".to_string())),
            RecoveryStrategy::Fallback
        );
    }

    #[test]
    fn test_retry_handler_default() {
        let handler = RetryHandler::new();
        assert_eq!(handler.config.max_retries, 5);
    }

    #[test]
    fn test_retry_handler_custom_config() {
        let config = BackoffConfig::new().with_max_retries(3);
        let handler = RetryHandler::with_config(config);
        assert_eq!(handler.config.max_retries, 3);
    }

    #[tokio::test]
    async fn test_execute_with_retry_success() {
        let handler = RetryHandler::new();
        let call_count = Arc::new(tokio::sync::Mutex::new(0));
        let call_count_clone = call_count.clone();

        let result = handler
            .execute_with_retry("test_op", || {
                let count = call_count_clone.clone();
                Box::pin(async move {
                    let mut c = count.lock().await;
                    *c += 1;
                    Ok::<i32, Error>(42)
                })
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(*call_count.lock().await, 1);
    }

    #[tokio::test]
    async fn test_execute_with_retry_non_recoverable_error() {
        let handler = RetryHandler::new();
        let call_count = Arc::new(tokio::sync::Mutex::new(0));
        let call_count_clone = call_count.clone();

        let result = handler
            .execute_with_retry("test_op", || {
                let count = call_count_clone.clone();
                Box::pin(async move {
                    let mut c = count.lock().await;
                    *c += 1;
                    Err::<i32, Error>(Error::ToolNotFound("test".to_string()))
                })
            })
            .await;

        assert!(result.is_err());
        assert_eq!(*call_count.lock().await, 1); // Should not retry
    }

    #[test]
    fn test_graceful_degradation_handler() {
        let mut handler = GracefulDegradationHandler::new();

        handler.mark_available("server1".to_string());
        handler.mark_available("server2".to_string());
        handler.mark_unavailable("server3".to_string());

        assert_eq!(handler.get_available_resources().len(), 2);
        assert_eq!(handler.get_unavailable_resources().len(), 1);
        assert!(handler.has_available_resources());
        assert_eq!(handler.availability_percentage(), 66.66666666666666);
    }

    #[test]
    fn test_graceful_degradation_handler_all_unavailable() {
        let mut handler = GracefulDegradationHandler::new();

        handler.mark_unavailable("server1".to_string());
        handler.mark_unavailable("server2".to_string());

        assert!(!handler.has_available_resources());
        assert_eq!(handler.availability_percentage(), 0.0);
    }

    #[test]
    fn test_graceful_degradation_handler_recovery() {
        let mut handler = GracefulDegradationHandler::new();

        handler.mark_unavailable("server1".to_string());
        assert!(!handler.has_available_resources());

        handler.mark_available("server1".to_string());
        assert!(handler.has_available_resources());
        assert_eq!(handler.get_unavailable_resources().len(), 0);
    }
}
