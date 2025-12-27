//! Retry policy with OpenCode-compatible header parsing
//!
//! SSTATE-006: Retry policy parity

use std::time::Duration;

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::{error::SessionResult, models::MessagePart, runtime_state::RuntimeStatus};

/// Retry policy configuration (OpenCode-compatible)
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Initial delay in milliseconds
    pub initial_delay_ms: u64,
    /// Backoff factor (e.g., 2.0 for exponential)
    pub backoff_factor: f64,
    /// Maximum delay without headers (milliseconds)
    pub max_delay_no_headers_ms: u64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            initial_delay_ms: 2000,      // 2 seconds
            backoff_factor: 2.0,          // Double each time
            max_delay_no_headers_ms: 30_000, // 30 seconds
        }
    }
}

/// Error information for retry decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryableError {
    /// Error message
    pub message: String,
    /// HTTP response headers (if available)
    pub response_headers: Option<std::collections::HashMap<String, String>>,
    /// Whether error is retryable
    pub is_retryable: bool,
}

impl RetryPolicy {
    /// Calculate delay for next retry attempt
    ///
    /// Implements OpenCode retry policy:
    /// 1. Check `retry-after-ms` header (milliseconds)
    /// 2. Check `retry-after` header (seconds or HTTP date)
    /// 3. Fall back to exponential backoff with max cap
    pub fn calculate_delay(&self, attempt: u32, error: Option<&RetryableError>) -> Duration {
        if let Some(err) = error {
            if let Some(headers) = &err.response_headers {
                // Check retry-after-ms header
                if let Some(retry_ms) = headers.get("retry-after-ms") {
                    if let Ok(ms) = retry_ms.parse::<u64>() {
                        return Duration::from_millis(ms);
                    }
                }
                
                // Check retry-after header (seconds or HTTP date)
                if let Some(retry_after) = headers.get("retry-after") {
                    // Try parsing as seconds
                    if let Ok(seconds) = retry_after.parse::<f64>() {
                        return Duration::from_secs_f64(seconds);
                    }
                    
                    // Try parsing as HTTP date format
                    if let Ok(parsed) = httpdate::parse_http_date(retry_after) {
                        let now = std::time::SystemTime::now();
                        if let Ok(duration) = parsed.duration_since(now) {
                            return duration;
                        }
                    }
                }
                
                // Headers present but unusable: use exponential backoff
                let delay_ms = self.initial_delay_ms * self.backoff_factor.powi((attempt - 1) as i32) as u64;
                return Duration::from_millis(delay_ms);
            }
        }
        
        // No headers: exponential backoff with max cap
        let delay_ms = self.initial_delay_ms * self.backoff_factor.powi((attempt - 1) as i32) as u64;
        let capped_ms = delay_ms.min(self.max_delay_no_headers_ms);
        Duration::from_millis(capped_ms)
    }
    
    /// Check if error is retryable and return user-facing message
    pub fn is_retryable(&self, error_message: &str) -> Option<String> {
        // Check for known retryable patterns
        let message_lower = error_message.to_lowercase();
        
        if message_lower.contains("overloaded") {
            return Some("Provider is overloaded".to_string());
        }
        
        if message_lower.contains("too many requests") || message_lower.contains("rate_limit") || message_lower.contains("rate limited") {
            return Some("Rate Limited".to_string());
        }
        
        if message_lower.contains("exhausted") || message_lower.contains("unavailable") {
            return Some("Provider is overloaded".to_string());
        }
        
        if message_lower.contains("server_error") || message_lower.contains("no_kv_space") {
            return Some("Provider Server Error".to_string());
        }
        
        None
    }
    
    /// Create retry status for runtime state manager
    pub fn create_retry_status(&self, attempt: u32, error: Option<&RetryableError>) -> SessionResult<RuntimeStatus> {
        let user_message = if let Some(err) = error {
            self.is_retryable(&err.message)
                .unwrap_or_else(|| "Retrying request".to_string())
        } else {
            "Retrying request".to_string()
        };
        
        let delay = self.calculate_delay(attempt, error);
        let next_time = Utc::now() + chrono::Duration::from_std(delay).unwrap();
        
        Ok(RuntimeStatus::Retry {
            attempt,
            message: user_message,
            next: next_time.timestamp_millis(),
        })
    }
    
    /// Sleep for retry delay (async)
    pub async fn sleep(&self, attempt: u32, error: Option<&RetryableError>) {
        let delay = self.calculate_delay(attempt, error);
        tokio::time::sleep(delay).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_exponential_backoff() {
        let policy = RetryPolicy::default();
        
        // Attempt 1: 2000ms
        let delay1 = policy.calculate_delay(1, None);
        assert_eq!(delay1.as_millis(), 2000);
        
        // Attempt 2: 4000ms
        let delay2 = policy.calculate_delay(2, None);
        assert_eq!(delay2.as_millis(), 4000);
        
        // Attempt 3: 8000ms
        let delay3 = policy.calculate_delay(3, None);
        assert_eq!(delay3.as_millis(), 8000);
    }
    
    #[test]
    fn test_max_delay_cap() {
        let policy = RetryPolicy::default();
        
        // Attempt 10 should be capped at 30 seconds
        let delay = policy.calculate_delay(10, None);
        assert_eq!(delay.as_millis(), 30_000);
    }
    
    #[test]
    fn test_retry_after_ms_header() {
        let policy = RetryPolicy::default();
        let mut headers = std::collections::HashMap::new();
        headers.insert("retry-after-ms".to_string(), "5000".to_string());
        
        let error = RetryableError {
            message: "Rate limited".to_string(),
            response_headers: Some(headers),
            is_retryable: true,
        };
        
        let delay = policy.calculate_delay(1, Some(&error));
        assert_eq!(delay.as_millis(), 5000);
    }
    
    #[test]
    fn test_retry_after_seconds_header() {
        let policy = RetryPolicy::default();
        let mut headers = std::collections::HashMap::new();
        headers.insert("retry-after".to_string(), "10.5".to_string());
        
        let error = RetryableError {
            message: "Rate limited".to_string(),
            response_headers: Some(headers),
            is_retryable: true,
        };
        
        let delay = policy.calculate_delay(1, Some(&error));
        assert_eq!(delay.as_secs(), 10);
    }
    
    #[test]
    fn test_retryable_detection() {
        let policy = RetryPolicy::default();
        
        assert_eq!(
            policy.is_retryable("Provider is overloaded"),
            Some("Provider is overloaded".to_string())
        );
        
        assert_eq!(
            policy.is_retryable("Too Many Requests"),
            Some("Rate Limited".to_string())
        );
        
        assert_eq!(
            policy.is_retryable("Connection refused"),
            None
        );
    }
    
    #[tokio::test]
    async fn test_create_retry_status() {
        let policy = RetryPolicy::default();
        let mut headers = std::collections::HashMap::new();
        headers.insert("retry-after-ms".to_string(), "3000".to_string());
        
        let error = RetryableError {
            message: "Rate limited".to_string(),
            response_headers: Some(headers),
            is_retryable: true,
        };
        
        let status = policy.create_retry_status(1, Some(&error)).unwrap();
        match status {
            RuntimeStatus::Retry { attempt, message, next } => {
                assert_eq!(attempt, 1);
                assert_eq!(message, "Rate Limited");
                assert!(next > Utc::now().timestamp_millis());
            }
            _ => panic!("Expected Retry status"),
        }
    }
}
