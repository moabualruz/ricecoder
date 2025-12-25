//! Circuit breaker pattern implementation for provider resilience
//!
//! AI Provider Implementations - Circuit breaker prevents cascade failures
//!
//! This module implements the circuit breaker pattern to prevent repeated calls
//! to failing providers, allowing them time to recover.

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use std::sync::RwLock;
use tracing::{debug, info, warn};

/// State of the circuit breaker
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed - requests flow through normally
    Closed,
    /// Circuit is open - requests fail immediately
    Open,
    /// Circuit is half-open - limited requests allowed to test recovery
    HalfOpen,
}

impl std::fmt::Display for CircuitState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CircuitState::Closed => write!(f, "Closed"),
            CircuitState::Open => write!(f, "Open"),
            CircuitState::HalfOpen => write!(f, "HalfOpen"),
        }
    }
}

/// Configuration for circuit breaker behavior
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening the circuit
    pub failure_threshold: u32,
    /// Duration to wait before attempting recovery
    pub recovery_timeout: Duration,
    /// Number of successful calls in half-open state before closing
    pub success_threshold: u32,
    /// Window for counting failures (resets after this time without failures)
    pub failure_window: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(30),
            success_threshold: 3,
            failure_window: Duration::from_secs(60),
        }
    }
}

impl CircuitBreakerConfig {
    /// Create a new circuit breaker config
    pub fn new() -> Self {
        Self::default()
    }

    /// Set failure threshold
    pub fn with_failure_threshold(mut self, threshold: u32) -> Self {
        self.failure_threshold = threshold;
        self
    }

    /// Set recovery timeout
    pub fn with_recovery_timeout(mut self, timeout: Duration) -> Self {
        self.recovery_timeout = timeout;
        self
    }

    /// Set success threshold for half-open state
    pub fn with_success_threshold(mut self, threshold: u32) -> Self {
        self.success_threshold = threshold;
        self
    }

    /// Set failure window
    pub fn with_failure_window(mut self, window: Duration) -> Self {
        self.failure_window = window;
        self
    }
}

/// Internal state tracking
struct CircuitBreakerState {
    state: CircuitState,
    failure_count: u32,
    success_count: u32,
    last_failure_time: Option<Instant>,
    opened_at: Option<Instant>,
}

impl Default for CircuitBreakerState {
    fn default() -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            last_failure_time: None,
            opened_at: None,
        }
    }
}

/// Circuit breaker for protecting against cascade failures
///
/// # Usage
///
/// ```ignore
/// let cb = CircuitBreaker::new("openai", CircuitBreakerConfig::default());
///
/// // Check if request can proceed
/// if cb.can_execute() {
///     match make_request().await {
///         Ok(response) => {
///             cb.record_success();
///             Ok(response)
///         }
///         Err(e) => {
///             cb.record_failure();
///             Err(e)
///         }
///     }
/// } else {
///     Err(ProviderError::CircuitOpen)
/// }
/// ```
pub struct CircuitBreaker {
    /// Provider identifier
    provider_id: String,
    /// Configuration
    config: CircuitBreakerConfig,
    /// Internal state
    state: RwLock<CircuitBreakerState>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(provider_id: impl Into<String>, config: CircuitBreakerConfig) -> Self {
        Self {
            provider_id: provider_id.into(),
            config,
            state: RwLock::new(CircuitBreakerState::default()),
        }
    }

    /// Get the provider ID
    pub fn provider_id(&self) -> &str {
        &self.provider_id
    }

    /// Get current circuit state
    pub fn state(&self) -> CircuitState {
        let state = self.state.read().expect("RwLock poisoned");
        self.compute_effective_state(&state)
    }

    /// Check if the circuit allows execution
    pub fn can_execute(&self) -> bool {
        let mut state = self.state.write().expect("RwLock poisoned");

        match state.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if recovery timeout has elapsed
                if let Some(opened_at) = state.opened_at {
                    if opened_at.elapsed() >= self.config.recovery_timeout {
                        // Transition to half-open
                        state.state = CircuitState::HalfOpen;
                        state.success_count = 0;
                        info!(
                            "Circuit breaker for {} transitioning to HalfOpen",
                            self.provider_id
                        );
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// Record a successful call
    pub fn record_success(&self) {
        let mut state = self.state.write().expect("RwLock poisoned");
        
        match state.state {
            CircuitState::Closed => {
                // Reset failure count on success
                state.failure_count = 0;
            }
            CircuitState::HalfOpen => {
                state.success_count += 1;
                debug!(
                    "Circuit breaker {} HalfOpen success {}/{}",
                    self.provider_id, state.success_count, self.config.success_threshold
                );
                
                if state.success_count >= self.config.success_threshold {
                    // Transition back to closed
                    state.state = CircuitState::Closed;
                    state.failure_count = 0;
                    state.success_count = 0;
                    state.opened_at = None;
                    info!(
                        "Circuit breaker for {} recovered, transitioning to Closed",
                        self.provider_id
                    );
                }
            }
            CircuitState::Open => {
                // Should not happen if can_execute() is called properly
                warn!(
                    "Circuit breaker {} recorded success while Open",
                    self.provider_id
                );
            }
        }
    }

    /// Record a failed call
    pub fn record_failure(&self) {
        let mut state = self.state.write().expect("RwLock poisoned");
        
        // Reset failure count if outside failure window
        if let Some(last_failure) = state.last_failure_time {
            if last_failure.elapsed() >= self.config.failure_window {
                state.failure_count = 0;
            }
        }
        
        state.failure_count += 1;
        state.last_failure_time = Some(Instant::now());
        
        match state.state {
            CircuitState::Closed => {
                debug!(
                    "Circuit breaker {} failure {}/{}",
                    self.provider_id, state.failure_count, self.config.failure_threshold
                );
                
                if state.failure_count >= self.config.failure_threshold {
                    // Open the circuit
                    state.state = CircuitState::Open;
                    state.opened_at = Some(Instant::now());
                    warn!(
                        "Circuit breaker for {} opened after {} failures",
                        self.provider_id, state.failure_count
                    );
                }
            }
            CircuitState::HalfOpen => {
                // Any failure in half-open immediately reopens
                state.state = CircuitState::Open;
                state.opened_at = Some(Instant::now());
                state.success_count = 0;
                warn!(
                    "Circuit breaker for {} reopened after failure in HalfOpen state",
                    self.provider_id
                );
            }
            CircuitState::Open => {
                // Already open, just update failure tracking
            }
        }
    }

    /// Force the circuit to open
    pub fn force_open(&self) {
        let mut state = self.state.write().expect("RwLock poisoned");
        state.state = CircuitState::Open;
        state.opened_at = Some(Instant::now());
        info!("Circuit breaker for {} force opened", self.provider_id);
    }

    /// Force the circuit to close
    pub fn force_close(&self) {
        let mut state = self.state.write().expect("RwLock poisoned");
        state.state = CircuitState::Closed;
        state.failure_count = 0;
        state.success_count = 0;
        state.opened_at = None;
        info!("Circuit breaker for {} force closed", self.provider_id);
    }

    /// Reset the circuit breaker
    pub fn reset(&self) {
        let mut state = self.state.write().expect("RwLock poisoned");
        *state = CircuitBreakerState::default();
        debug!("Circuit breaker for {} reset", self.provider_id);
    }

    /// Get failure count
    pub fn failure_count(&self) -> u32 {
        self.state.read().expect("RwLock poisoned").failure_count
    }

    /// Get success count (meaningful only in half-open state)
    pub fn success_count(&self) -> u32 {
        self.state.read().expect("RwLock poisoned").success_count
    }

    /// Compute effective state (checking recovery timeout)
    fn compute_effective_state(&self, state: &CircuitBreakerState) -> CircuitState {
        match state.state {
            CircuitState::Open => {
                if let Some(opened_at) = state.opened_at {
                    if opened_at.elapsed() >= self.config.recovery_timeout {
                        return CircuitState::HalfOpen;
                    }
                }
                CircuitState::Open
            }
            other => other,
        }
    }
}

/// Registry of circuit breakers by provider ID
pub struct CircuitBreakerRegistry {
    breakers: RwLock<std::collections::HashMap<String, Arc<CircuitBreaker>>>,
    default_config: CircuitBreakerConfig,
}

impl CircuitBreakerRegistry {
    /// Create a new registry with default config
    pub fn new() -> Self {
        Self {
            breakers: RwLock::new(std::collections::HashMap::new()),
            default_config: CircuitBreakerConfig::default(),
        }
    }

    /// Create a new registry with custom default config
    pub fn with_config(config: CircuitBreakerConfig) -> Self {
        Self {
            breakers: RwLock::new(std::collections::HashMap::new()),
            default_config: config,
        }
    }

    /// Get or create a circuit breaker for a provider
    pub fn get_or_create(&self, provider_id: &str) -> Arc<CircuitBreaker> {
        // Check if exists
        {
            let breakers = self.breakers.read().expect("RwLock poisoned");
            if let Some(cb) = breakers.get(provider_id) {
                return Arc::clone(cb);
            }
        }

        // Create new
        let mut breakers = self.breakers.write().expect("RwLock poisoned");
        // Double-check after acquiring write lock
        if let Some(cb) = breakers.get(provider_id) {
            return Arc::clone(cb);
        }

        let cb = Arc::new(CircuitBreaker::new(provider_id, self.default_config.clone()));
        breakers.insert(provider_id.to_string(), Arc::clone(&cb));
        cb
    }

    /// Register a circuit breaker with custom config
    pub fn register(&self, provider_id: &str, config: CircuitBreakerConfig) -> Arc<CircuitBreaker> {
        let mut breakers = self.breakers.write().expect("RwLock poisoned");
        let cb = Arc::new(CircuitBreaker::new(provider_id, config));
        breakers.insert(provider_id.to_string(), Arc::clone(&cb));
        cb
    }

    /// Get a circuit breaker if it exists
    pub fn get(&self, provider_id: &str) -> Option<Arc<CircuitBreaker>> {
        let breakers = self.breakers.read().expect("RwLock poisoned");
        breakers.get(provider_id).cloned()
    }

    /// Reset all circuit breakers
    pub fn reset_all(&self) {
        let breakers = self.breakers.read().expect("RwLock poisoned");
        for cb in breakers.values() {
            cb.reset();
        }
    }
}

impl Default for CircuitBreakerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_starts_closed() {
        let cb = CircuitBreaker::new("test", CircuitBreakerConfig::default());
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.can_execute());
    }

    #[test]
    fn test_circuit_opens_after_threshold_failures() {
        let config = CircuitBreakerConfig::default().with_failure_threshold(3);
        let cb = CircuitBreaker::new("test", config);

        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);
        
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);
        
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.can_execute());
    }

    #[test]
    fn test_success_resets_failure_count() {
        let config = CircuitBreakerConfig::default().with_failure_threshold(3);
        let cb = CircuitBreaker::new("test", config);

        cb.record_failure();
        cb.record_failure();
        cb.record_success();
        
        assert_eq!(cb.failure_count(), 0);
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_force_open_and_close() {
        let cb = CircuitBreaker::new("test", CircuitBreakerConfig::default());
        
        cb.force_open();
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.can_execute());
        
        cb.force_close();
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.can_execute());
    }

    #[test]
    fn test_reset() {
        let config = CircuitBreakerConfig::default().with_failure_threshold(3);
        let cb = CircuitBreaker::new("test", config);

        cb.record_failure();
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        
        cb.reset();
        assert_eq!(cb.state(), CircuitState::Closed);
        assert_eq!(cb.failure_count(), 0);
    }

    #[test]
    fn test_registry_get_or_create() {
        let registry = CircuitBreakerRegistry::new();
        
        let cb1 = registry.get_or_create("provider1");
        let cb2 = registry.get_or_create("provider1");
        let cb3 = registry.get_or_create("provider2");

        // Same provider should return same instance
        assert!(Arc::ptr_eq(&cb1, &cb2));
        // Different provider should return different instance
        assert!(!Arc::ptr_eq(&cb1, &cb3));
    }

    #[test]
    fn test_half_open_recovers_after_successes() {
        let config = CircuitBreakerConfig::default()
            .with_failure_threshold(2)
            .with_success_threshold(2)
            .with_recovery_timeout(Duration::from_millis(1));
        
        let cb = CircuitBreaker::new("test", config);

        // Open the circuit
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);

        // Wait for recovery timeout
        std::thread::sleep(Duration::from_millis(5));
        
        // Should be half-open now
        assert!(cb.can_execute());
        
        // Record successes
        cb.record_success();
        cb.record_success();
        
        // Should be closed now
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_half_open_reopens_on_failure() {
        let config = CircuitBreakerConfig::default()
            .with_failure_threshold(2)
            .with_recovery_timeout(Duration::from_millis(1));
        
        let cb = CircuitBreaker::new("test", config);

        // Open the circuit
        cb.record_failure();
        cb.record_failure();
        
        // Wait for recovery timeout
        std::thread::sleep(Duration::from_millis(5));
        
        // Should be half-open now
        assert!(cb.can_execute());
        
        // Record failure in half-open
        cb.record_failure();
        
        // Should be open again
        assert_eq!(cb.state(), CircuitState::Open);
    }
}
