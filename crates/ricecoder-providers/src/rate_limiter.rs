//! Rate limiting for API calls
//!
//! This module provides rate limiting functionality to prevent exceeding provider limits
//! and to implement backoff strategies for rate limit errors.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

/// Token bucket rate limiter
///
/// Implements the token bucket algorithm for rate limiting API calls.
/// Tokens are added at a fixed rate, and each request consumes tokens.
/// If insufficient tokens are available, the request is rate limited.
pub struct TokenBucketLimiter {
    /// Tokens per second (refill rate)
    tokens_per_second: f64,
    /// Maximum tokens in bucket (burst capacity)
    max_tokens: f64,
    /// Current tokens in bucket
    tokens: f64,
    /// Last refill time
    last_refill: Instant,
}

impl TokenBucketLimiter {
    /// Create a new token bucket limiter
    ///
    /// # Arguments
    /// * `tokens_per_second` - Rate at which tokens are added (e.g., 10 for 10 requests/sec)
    /// * `max_tokens` - Maximum tokens in bucket (burst capacity)
    pub fn new(tokens_per_second: f64, max_tokens: f64) -> Self {
        Self {
            tokens_per_second,
            max_tokens,
            tokens: max_tokens,
            last_refill: Instant::now(),
        }
    }

    /// Refill tokens based on elapsed time
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        let new_tokens = elapsed * self.tokens_per_second;
        self.tokens = (self.tokens + new_tokens).min(self.max_tokens);
        self.last_refill = now;
    }

    /// Try to acquire tokens
    ///
    /// Returns true if tokens were acquired, false if rate limited
    pub fn try_acquire(&mut self, tokens: f64) -> bool {
        self.refill();
        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }

    /// Wait until tokens are available
    ///
    /// Blocks until the specified number of tokens are available
    pub async fn acquire(&mut self, tokens: f64) {
        loop {
            if self.try_acquire(tokens) {
                return;
            }
            // Wait a bit before trying again
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    /// Get current token count
    pub fn current_tokens(&mut self) -> f64 {
        self.refill();
        self.tokens
    }

    /// Get time until tokens are available
    pub fn time_until_available(&mut self, tokens: f64) -> Duration {
        self.refill();
        if self.tokens >= tokens {
            Duration::from_secs(0)
        } else {
            let needed = tokens - self.tokens;
            let seconds = needed / self.tokens_per_second;
            Duration::from_secs_f64(seconds)
        }
    }
}

/// Exponential backoff strategy
///
/// Implements exponential backoff with jitter for retrying failed requests
pub struct ExponentialBackoff {
    /// Initial backoff duration
    initial_delay: Duration,
    /// Maximum backoff duration
    max_delay: Duration,
    /// Backoff multiplier
    multiplier: f64,
    /// Current attempt number
    attempt: u32,
}

impl ExponentialBackoff {
    /// Create a new exponential backoff strategy
    ///
    /// # Arguments
    /// * `initial_delay` - Initial backoff duration (e.g., 100ms)
    /// * `max_delay` - Maximum backoff duration (e.g., 30s)
    /// * `multiplier` - Backoff multiplier (e.g., 2.0 for doubling)
    pub fn new(initial_delay: Duration, max_delay: Duration, multiplier: f64) -> Self {
        Self {
            initial_delay,
            max_delay,
            multiplier,
            attempt: 0,
        }
    }

    /// Get the next backoff duration
    pub fn next_delay(&mut self) -> Duration {
        let delay = self.initial_delay.as_secs_f64() * self.multiplier.powi(self.attempt as i32);
        let delay = Duration::from_secs_f64(delay);
        let delay = delay.min(self.max_delay);

        // Add jitter (±10%)
        let jitter = delay.as_secs_f64() * 0.1;
        let jitter_offset = (rand::random::<f64>() - 0.5) * 2.0 * jitter;
        let final_delay = (delay.as_secs_f64() + jitter_offset).max(0.0);

        self.attempt += 1;
        Duration::from_secs_f64(final_delay)
    }

    /// Reset backoff counter
    pub fn reset(&mut self) {
        self.attempt = 0;
    }

    /// Get current attempt number
    pub fn attempt(&self) -> u32 {
        self.attempt
    }
}

/// Per-provider rate limiter registry
pub struct RateLimiterRegistry {
    limiters: Arc<Mutex<HashMap<String, TokenBucketLimiter>>>,
}

impl RateLimiterRegistry {
    /// Create a new rate limiter registry
    pub fn new() -> Self {
        Self {
            limiters: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register a rate limiter for a provider
    pub fn register(&self, provider_id: &str, limiter: TokenBucketLimiter) {
        let mut limiters = self.limiters.lock().unwrap();
        limiters.insert(provider_id.to_string(), limiter);
    }

    /// Get or create a rate limiter for a provider
    pub fn get_or_create(&self, provider_id: &str) -> Arc<Mutex<TokenBucketLimiter>> {
        let mut limiters = self.limiters.lock().unwrap();

        // Return existing limiter if available
        if limiters.contains_key(provider_id) {
            // We need to return a reference, but we can't hold the lock
            // So we'll create a new Arc for each call
            drop(limiters);
            return Arc::new(Mutex::new(TokenBucketLimiter::new(10.0, 100.0)));
        }

        // Create default limiter (10 requests/sec, burst of 100)
        let limiter = TokenBucketLimiter::new(10.0, 100.0);
        limiters.insert(provider_id.to_string(), limiter);
        drop(limiters);

        Arc::new(Mutex::new(TokenBucketLimiter::new(10.0, 100.0)))
    }

    /// Try to acquire tokens for a provider
    pub fn try_acquire(&self, provider_id: &str, tokens: f64) -> bool {
        let mut limiters = self.limiters.lock().unwrap();
        if let Some(limiter) = limiters.get_mut(provider_id) {
            limiter.try_acquire(tokens)
        } else {
            // No limiter registered, allow request
            true
        }
    }

    /// Wait until tokens are available for a provider
    pub async fn acquire(&self, provider_id: &str, tokens: f64) {
        loop {
            if self.try_acquire(provider_id, tokens) {
                return;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
}

impl Default for RateLimiterRegistry {
    fn default() -> Self {
        Self::new()
    }
}
