//! HTTP middleware for retry logic

use std::time::Duration;

use tracing::{debug, warn};

use crate::{error::HttpError, Result};

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum retry attempts
    pub max_attempts: u32,
    /// Initial delay between retries
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Backoff multiplier
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
        }
    }
}

impl RetryConfig {
    /// Calculate delay for given attempt number
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        let delay_ms = self.initial_delay.as_millis() as f64
            * self.backoff_multiplier.powi(attempt as i32);

        let delay = Duration::from_millis(delay_ms as u64);
        std::cmp::min(delay, self.max_delay)
    }
}

/// Retry middleware for HTTP operations
pub struct RetryMiddleware {
    config: RetryConfig,
}

impl RetryMiddleware {
    /// Create new retry middleware
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    /// Execute operation with retry logic
    pub async fn execute<F, Fut, T>(&self, operation: F) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut last_error = None;

        for attempt in 0..=self.config.max_attempts {
            match operation().await {
                Ok(result) => {
                    if attempt > 0 {
                        debug!("Request succeeded after {attempt} retries");
                    }
                    return Ok(result);
                }
                Err(e) => {
                    if !e.is_retryable() {
                        debug!("Non-retryable error: {e}");
                        return Err(e);
                    }

                    if attempt < self.config.max_attempts {
                        let delay = self.config.calculate_delay(attempt);
                        warn!(
                            "Request failed (attempt {}/{}), retrying in {:?}: {}",
                            attempt + 1,
                            self.config.max_attempts,
                            delay,
                            e
                        );
                        tokio::time::sleep(delay).await;
                        last_error = Some(e);
                    } else {
                        return Err(HttpError::RetryLimitExceeded {
                            attempts: attempt + 1,
                        });
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| HttpError::RetryLimitExceeded {
            attempts: self.config.max_attempts,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_config_delay_calculation() {
        let config = RetryConfig::default();

        // Attempt 0: 500ms
        assert_eq!(config.calculate_delay(0), Duration::from_millis(500));

        // Attempt 1: 1000ms (500 * 2^1)
        assert_eq!(config.calculate_delay(1), Duration::from_millis(1000));

        // Attempt 2: 2000ms (500 * 2^2)
        assert_eq!(config.calculate_delay(2), Duration::from_millis(2000));

        // Attempt 3: 4000ms (500 * 2^3)
        assert_eq!(config.calculate_delay(3), Duration::from_millis(4000));
    }

    #[test]
    fn test_max_delay_cap() {
        let config = RetryConfig {
            max_delay: Duration::from_secs(5),
            ..Default::default()
        };

        // Should be capped at max_delay
        assert!(config.calculate_delay(10) <= Duration::from_secs(5));
    }

    #[tokio::test]
    async fn test_retry_success_on_first_attempt() {
        let middleware = RetryMiddleware::new(RetryConfig::default());

        let result = middleware
            .execute(|| async { Ok::<_, HttpError>(42) })
            .await;

        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_retry_success_on_second_attempt() {
        let middleware = RetryMiddleware::new(RetryConfig {
            max_attempts: 2,
            initial_delay: Duration::from_millis(10),
            ..Default::default()
        });

        let mut attempt = 0;
        let result = middleware
            .execute(|| async {
                attempt += 1;
                if attempt == 1 {
                    Err(HttpError::Timeout(Duration::from_secs(1)))
                } else {
                    Ok(42)
                }
            })
            .await;

        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_retry_limit_exceeded() {
        let middleware = RetryMiddleware::new(RetryConfig {
            max_attempts: 2,
            initial_delay: Duration::from_millis(10),
            ..Default::default()
        });

        let result = middleware
            .execute(|| async { Err::<i32, _>(HttpError::Timeout(Duration::from_secs(1))) })
            .await;

        assert!(matches!(
            result,
            Err(HttpError::RetryLimitExceeded { attempts: 3 })
        ));
    }

    #[tokio::test]
    async fn test_non_retryable_error() {
        let middleware = RetryMiddleware::new(RetryConfig::default());

        let result = middleware
            .execute(|| async { Err::<i32, _>(HttpError::InvalidUrl("bad".to_string())) })
            .await;

        assert!(matches!(result, Err(HttpError::InvalidUrl(_))));
    }
}
