//! Comprehensive error handling system for RiceCoder TUI
//!
//! This module implements:
//! - Error boundary system for component isolation
//! - Crash recovery with auto-save and state restoration
//! - User-friendly error messages and categorization
//! - Retry logic for network failures
//! - Error logging and debugging support

use crate::tea::{AppMessage, AppModel, ReactiveState};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    Low = 1,      // Minor issues, non-blocking
    Medium = 2,   // Noticeable issues, partial functionality loss
    High = 3,     // Significant issues, major functionality loss
    Critical = 4, // System-breaking issues, requires immediate attention
}

/// Error categories for better organization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCategory {
    Network,
    FileSystem,
    Configuration,
    Authentication,
    Validation,
    Rendering,
    State,
    ExternalService,
    UserInput,
    System,
}

/// Comprehensive error type with context and recovery suggestions
#[derive(Debug, Clone)]
pub struct RiceError {
    pub message: String,
    pub category: ErrorCategory,
    pub severity: ErrorSeverity,
    pub context: HashMap<String, String>,
    pub recovery_suggestions: Vec<String>,
    pub timestamp: Instant,
    pub component: String,
    pub operation: String,
    pub user_friendly_message: String,
    pub technical_details: Option<String>,
    pub retryable: bool,
    pub error_id: String,
}

impl RiceError {
    pub fn new(
        message: impl Into<String>,
        category: ErrorCategory,
        severity: ErrorSeverity,
        component: impl Into<String>,
        operation: impl Into<String>,
    ) -> Self {
        let message = message.into();
        let component = component.into();
        let operation = operation.into();
        let error_id = format!("{}_{}", component, uuid::Uuid::new_v4().simple());

        let user_friendly_message = Self::generate_user_friendly_message(&message, category, severity);
        let recovery_suggestions = Self::generate_recovery_suggestions(category, severity);

        Self {
            message,
            category,
            severity,
            context: HashMap::new(),
            recovery_suggestions,
            timestamp: Instant::now(),
            component,
            operation,
            user_friendly_message,
            technical_details: None,
            retryable: Self::is_retryable(category),
            error_id,
        }
    }

    /// Add context information
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }

    /// Add technical details for debugging
    pub fn with_technical_details(mut self, details: impl Into<String>) -> Self {
        self.technical_details = Some(details.into());
        self
    }

    /// Generate user-friendly error message
    fn generate_user_friendly_message(original: &str, category: ErrorCategory, severity: ErrorSeverity) -> String {
        match (category, severity) {
            (ErrorCategory::Network, ErrorSeverity::High) =>
                "Connection lost. Please check your internet connection and try again.".to_string(),
            (ErrorCategory::FileSystem, ErrorSeverity::High) =>
                "Unable to access file. Please check if the file exists and you have permission.".to_string(),
            (ErrorCategory::Configuration, ErrorSeverity::Medium) =>
                "Configuration issue detected. Some features may not work as expected.".to_string(),
            (ErrorCategory::Authentication, ErrorSeverity::High) =>
                "Authentication failed. Please check your credentials and try again.".to_string(),
            (ErrorCategory::Validation, ErrorSeverity::Medium) =>
                "Invalid input provided. Please check your input and try again.".to_string(),
            (ErrorCategory::Rendering, ErrorSeverity::High) =>
                "Display error occurred. The interface may not render correctly.".to_string(),
            (ErrorCategory::State, ErrorSeverity::Critical) =>
                "Application state corrupted. Please restart the application.".to_string(),
            _ => format!("An error occurred: {}", original),
        }
    }

    /// Generate recovery suggestions
    fn generate_recovery_suggestions(category: ErrorCategory, severity: ErrorSeverity) -> Vec<String> {
        match (category, severity) {
            (ErrorCategory::Network, _) => vec![
                "Check your internet connection".to_string(),
                "Try again in a few moments".to_string(),
                "Contact your network administrator if the problem persists".to_string(),
            ],
            (ErrorCategory::FileSystem, _) => vec![
                "Check if the file exists".to_string(),
                "Verify you have read/write permissions".to_string(),
                "Try saving to a different location".to_string(),
            ],
            (ErrorCategory::Configuration, _) => vec![
                "Check configuration file syntax".to_string(),
                "Restore from backup configuration".to_string(),
                "Reset to default settings".to_string(),
            ],
            (ErrorCategory::Authentication, _) => vec![
                "Verify your credentials".to_string(),
                "Check if your account is active".to_string(),
                "Contact support if you continue having issues".to_string(),
            ],
            (ErrorCategory::State, ErrorSeverity::Critical) => vec![
                "Restart the application".to_string(),
                "Clear application cache".to_string(),
                "Reinstall the application if problems persist".to_string(),
            ],
            _ => vec![
                "Try the operation again".to_string(),
                "Restart the application".to_string(),
                "Contact support if the problem continues".to_string(),
            ],
        }
    }

    /// Check if error is retryable
    fn is_retryable(category: ErrorCategory) -> bool {
        matches!(category, ErrorCategory::Network | ErrorCategory::ExternalService)
    }
}

impl fmt::Display for RiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}: {}", self.category, self.severity, self.message)
    }
}

impl std::error::Error for RiceError {}

/// Error boundary for component isolation
pub struct ErrorBoundary<T> {
    component_name: String,
    fallback_component: T,
    error_handler: Box<dyn Fn(&RiceError) + Send + Sync>,
    error_count: Arc<RwLock<usize>>,
    max_errors: usize,
    recovery_strategy: RecoveryStrategy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryStrategy {
    /// Replace with fallback component
    Fallback,
    /// Retry operation
    Retry,
    /// Escalate to parent
    Escalate,
    /// Graceful degradation
    Degrade,
}

impl<T> ErrorBoundary<T>
where
    T: Clone + Send + Sync,
{
    pub fn new(
        component_name: impl Into<String>,
        fallback_component: T,
        error_handler: impl Fn(&RiceError) + Send + Sync + 'static,
    ) -> Self {
        Self {
            component_name: component_name.into(),
            fallback_component,
            error_handler: Box::new(error_handler),
            error_count: Arc::new(RwLock::new(0)),
            max_errors: 3,
            recovery_strategy: RecoveryStrategy::Fallback,
        }
    }

    /// Execute a component operation with error boundary protection
    pub async fn execute<F, R>(&self, operation: F) -> Result<R, RiceError>
    where
        F: FnOnce() -> Result<R, RiceError> + Send,
        R: Send,
    {
        match operation() {
            Ok(result) => {
                // Reset error count on success
                let mut count = self.error_count.write().await;
                *count = 0;
                Ok(result)
            }
            Err(error) => {
                let mut count = self.error_count.write().await;
                *count += 1;

                // Call error handler
                (self.error_handler)(&error);

                // Check if we've exceeded max errors
                if *count >= self.max_errors {
                    match self.recovery_strategy {
                        RecoveryStrategy::Fallback => {
                            return Err(RiceError::new(
                                format!("Component {} failed too many times, using fallback", self.component_name),
                                ErrorCategory::System,
                                ErrorSeverity::High,
                                &self.component_name,
                                "execute",
                            ));
                        }
                        RecoveryStrategy::Retry => {
                            // Could implement retry logic here
                            return Err(error);
                        }
                        RecoveryStrategy::Escalate => {
                            return Err(error);
                        }
                        RecoveryStrategy::Degrade => {
                            // Return fallback result
                            return Err(RiceError::new(
                                format!("Component {} degraded due to errors", self.component_name),
                                ErrorCategory::System,
                                ErrorSeverity::Medium,
                                &self.component_name,
                                "execute",
                            ));
                        }
                    }
                }

                Err(error)
            }
        }
    }

    /// Get the fallback component
    pub fn fallback(&self) -> T {
        self.fallback_component.clone()
    }

    /// Set recovery strategy
    pub fn with_recovery_strategy(mut self, strategy: RecoveryStrategy) -> Self {
        self.recovery_strategy = strategy;
        self
    }

    /// Set maximum error count before recovery
    pub fn with_max_errors(mut self, max_errors: usize) -> Self {
        self.max_errors = max_errors;
        self
    }
}

/// Crash recovery system with auto-save
pub struct CrashRecovery {
    auto_save_interval: Duration,
    last_save: Arc<RwLock<Option<Instant>>>,
    recovery_data: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    crash_reports: Arc<RwLock<Vec<CrashReport>>>,
    max_reports: usize,
}

#[derive(Debug, Clone)]
pub struct CrashReport {
    pub timestamp: Instant,
    pub error_message: String,
    pub stack_trace: Option<String>,
    pub system_info: HashMap<String, String>,
    pub user_actions: Vec<String>,
    pub recovery_successful: bool,
}

impl CrashRecovery {
    pub fn new(auto_save_interval: Duration) -> Self {
        Self {
            auto_save_interval,
            last_save: Arc::new(RwLock::new(None)),
            recovery_data: Arc::new(RwLock::new(HashMap::new())),
            crash_reports: Arc::new(RwLock::new(Vec::new())),
            max_reports: 10,
        }
    }

    /// Auto-save application state
    pub async fn auto_save(&self, key: &str, data: Vec<u8>) -> Result<(), RiceError> {
        let mut recovery_data = self.recovery_data.write().await;
        recovery_data.insert(key.to_string(), data);

        let mut last_save = self.last_save.write().await;
        *last_save = Some(Instant::now());

        Ok(())
    }

    /// Check if auto-save is needed
    pub async fn needs_save(&self) -> bool {
        let last_save = self.last_save.read().await;
        match *last_save {
            Some(time) => time.elapsed() >= self.auto_save_interval,
            None => true,
        }
    }

    /// Restore saved state
    pub async fn restore_state(&self, key: &str) -> Option<Vec<u8>> {
        let recovery_data = self.recovery_data.read().await;
        recovery_data.get(key).cloned()
    }

    /// Record a crash
    pub async fn record_crash(
        &self,
        error_message: String,
        stack_trace: Option<String>,
        system_info: HashMap<String, String>,
        user_actions: Vec<String>,
    ) {
        let report = CrashReport {
            timestamp: Instant::now(),
            error_message,
            stack_trace,
            system_info,
            user_actions,
            recovery_successful: false,
        };

        let mut reports = self.crash_reports.write().await;
        reports.push(report);

        // Keep only recent reports
        if reports.len() > self.max_reports {
            reports.remove(0);
        }
    }

    /// Mark recovery as successful
    pub async fn mark_recovery_successful(&self, crash_timestamp: Instant) {
        let mut reports = self.crash_reports.write().await;
        if let Some(report) = reports.iter_mut().find(|r| r.timestamp == crash_timestamp) {
            report.recovery_successful = true;
        }
    }

    /// Get crash reports
    pub async fn get_crash_reports(&self) -> Vec<CrashReport> {
        let reports = self.crash_reports.read().await;
        reports.clone()
    }

    /// Clear recovery data
    pub async fn clear_recovery_data(&self) {
        let mut recovery_data = self.recovery_data.write().await;
        recovery_data.clear();
    }
}

/// Retry mechanism with exponential backoff
pub struct RetryMechanism {
    max_attempts: u32,
    base_delay: Duration,
    max_delay: Duration,
    backoff_factor: f32,
}

impl RetryMechanism {
    pub fn new(max_attempts: u32, base_delay: Duration) -> Self {
        Self {
            max_attempts,
            base_delay,
            max_delay: Duration::from_secs(30),
            backoff_factor: 2.0,
        }
    }

    /// Execute operation with retry logic
    pub async fn execute<F, Fut, T>(&self, mut operation: F) -> Result<T, RiceError>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, RiceError>>,
    {
        let mut attempt = 0;

        loop {
            attempt += 1;

            match operation().await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    if attempt >= self.max_attempts || !error.retryable {
                        return Err(error);
                    }

                    // Calculate delay with exponential backoff
                    let delay = self.calculate_delay(attempt);
                    tracing::warn!("Operation failed (attempt {}), retrying in {:?}", attempt, delay);

                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    /// Calculate delay for exponential backoff
    fn calculate_delay(&self, attempt: u32) -> Duration {
        let delay_ms = self.base_delay.as_millis() as f32 * self.backoff_factor.powi(attempt.saturating_sub(1) as i32);
        let delay_ms = delay_ms.min(self.max_delay.as_millis() as f32) as u64;
        Duration::from_millis(delay_ms)
    }
}

/// Error logger with categorization and filtering
pub struct ErrorLogger {
    logs: Arc<RwLock<Vec<LogEntry>>>,
    max_entries: usize,
    filters: Arc<RwLock<HashMap<ErrorCategory, bool>>>,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: Instant,
    pub level: LogLevel,
    pub error: RiceError,
    pub context: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warning => write!(f, "WARNING"),
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Critical => write!(f, "CRITICAL"),
        }
    }
}

impl ErrorLogger {
    pub fn new(max_entries: usize) -> Self {
        let mut filters = HashMap::new();
        // Enable all categories by default
        for &category in &[
            ErrorCategory::Network,
            ErrorCategory::FileSystem,
            ErrorCategory::Configuration,
            ErrorCategory::Authentication,
            ErrorCategory::Validation,
            ErrorCategory::Rendering,
            ErrorCategory::State,
            ErrorCategory::ExternalService,
            ErrorCategory::UserInput,
            ErrorCategory::System,
        ] {
            filters.insert(category, true);
        }

        Self {
            logs: Arc::new(RwLock::new(Vec::new())),
            max_entries,
            filters: Arc::new(RwLock::new(filters)),
        }
    }

    /// Log an error
    pub async fn log_error(&self, level: LogLevel, error: RiceError, context: HashMap<String, String>) {
        let entry = LogEntry {
            timestamp: Instant::now(),
            level,
            error,
            context,
        };

        let mut logs = self.logs.write().await;
        logs.push(entry);

        // Maintain max entries
        if logs.len() > self.max_entries {
            logs.remove(0);
        }
    }

    /// Get filtered logs
    pub async fn get_logs(&self, category_filter: Option<ErrorCategory>) -> Vec<LogEntry> {
        let logs = self.logs.read().await;
        let filters = self.filters.read().await;

        logs.iter()
            .filter(|entry| {
                category_filter
                    .map(|cat| filters.get(&cat).copied().unwrap_or(true))
                    .unwrap_or(true)
            })
            .cloned()
            .collect()
    }

    /// Set category filter
    pub async fn set_category_filter(&self, category: ErrorCategory, enabled: bool) {
        let mut filters = self.filters.write().await;
        filters.insert(category, enabled);
    }

    /// Export logs for debugging
    pub async fn export_logs(&self) -> String {
        let logs = self.logs.read().await;
        let mut output = String::new();

        for log in logs.iter() {
            output.push_str(&format!(
                "[{}] {} - {}: {}\n",
                log.timestamp.elapsed().as_secs(),
                log.level,
                log.error.category,
                log.error.message
            ));

            for (key, value) in &log.context {
                output.push_str(&format!("  {}: {}\n", key, value));
            }

            if let Some(details) = &log.error.technical_details {
                output.push_str(&format!("  Technical: {}\n", details));
            }

            output.push('\n');
        }

        output
    }

    /// Clear all logs
    pub async fn clear_logs(&self) {
        let mut logs = self.logs.write().await;
        logs.clear();
    }
}

/// Global error manager that coordinates all error handling
#[derive(Clone)]
pub struct ErrorManager {
    pub boundaries: HashMap<String, Box<dyn std::any::Any + Send + Sync>>,
    pub crash_recovery: CrashRecovery,
    pub retry_mechanism: RetryMechanism,
    pub logger: ErrorLogger,
    pub error_counts: Arc<RwLock<HashMap<ErrorCategory, usize>>>,
}

impl ErrorManager {
    pub fn new() -> Self {
        Self {
            boundaries: HashMap::new(),
            crash_recovery: CrashRecovery::new(Duration::from_secs(30)),
            retry_mechanism: RetryMechanism::new(3, Duration::from_millis(500)),
            logger: ErrorLogger::new(1000),
            error_counts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Handle an error with appropriate recovery
    pub async fn handle_error(&self, error: RiceError) -> Result<(), RiceError> {
        // Log the error
        self.logger.log_error(
            match error.severity {
                ErrorSeverity::Low => LogLevel::Info,
                ErrorSeverity::Medium => LogLevel::Warning,
                ErrorSeverity::High => LogLevel::Error,
                ErrorSeverity::Critical => LogLevel::Critical,
            },
            error.clone(),
            HashMap::new(),
        ).await;

        // Update error counts
        {
            let mut counts = self.error_counts.write().await;
            *counts.entry(error.category).or_insert(0) += 1;
        }

        // Handle based on severity
        match error.severity {
            ErrorSeverity::Critical => {
                // Record crash for critical errors
                self.crash_recovery.record_crash(
                    error.message.clone(),
                    error.technical_details.clone(),
                    HashMap::new(),
                    vec![format!("Operation: {}", error.operation)],
                ).await;

                Err(error)
            }
            ErrorSeverity::High => {
                // For high severity, attempt recovery if possible
                if error.retryable {
                    // Could implement retry logic here
                }
                Err(error)
            }
            _ => {
                // For lower severity, log and continue
                Ok(())
            }
        }
    }

    /// Get error statistics
    pub async fn get_error_stats(&self) -> HashMap<ErrorCategory, usize> {
        let counts = self.error_counts.read().await;
        counts.clone()
    }

    /// Check if system is in error state
    pub async fn is_in_error_state(&self) -> bool {
        let counts = self.error_counts.read().await;
        let total_errors: usize = counts.values().sum();
        total_errors > 10 // Arbitrary threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rice_error_creation() {
        let error = RiceError::new(
            "Test error",
            ErrorCategory::System,
            ErrorSeverity::High,
            "test_component",
            "test_operation",
        );

        assert_eq!(error.category, ErrorCategory::System);
        assert_eq!(error.severity, ErrorSeverity::High);
        assert!(error.retryable);
        assert!(!error.error_id.is_empty());
    }

    #[test]
    fn test_error_boundary() {
        let boundary = ErrorBoundary::new(
            "test_component",
            "fallback_value",
            |error| {
                println!("Error handled: {}", error);
            },
        );

        // Test successful operation
        let result = futures::executor::block_on(boundary.execute(|| Ok("success")));
        assert!(result.is_ok());

        // Test failed operation
        let result = futures::executor::block_on(boundary.execute(|| {
            Err(RiceError::new(
                "Test failure",
                ErrorCategory::System,
                ErrorSeverity::Medium,
                "test",
                "test",
            ))
        }));
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_crash_recovery() {
        let recovery = CrashRecovery::new(Duration::from_secs(1));

        // Test auto-save
        recovery.auto_save("test_key", vec![1, 2, 3]).await.unwrap();

        // Test restore
        let data = recovery.restore_state("test_key").await;
        assert_eq!(data, Some(vec![1, 2, 3]));

        // Test crash recording
        recovery.record_crash(
            "Test crash".to_string(),
            Some("stack trace".to_string()),
            HashMap::new(),
            vec!["action1".to_string()],
        ).await;

        let reports = recovery.get_crash_reports().await;
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].error_message, "Test crash");
    }

    #[tokio::test]
    async fn test_retry_mechanism() {
        let retry = RetryMechanism::new(3, Duration::from_millis(10));

        let mut attempts = 0;
        let result = retry.execute(|| async {
            attempts += 1;
            if attempts < 3 {
                Err(RiceError::new(
                    "Retry test",
                    ErrorCategory::Network,
                    ErrorSeverity::Medium,
                    "test",
                    "test",
                ))
            } else {
                Ok("success")
            }
        }).await;

        assert!(result.is_ok());
        assert_eq!(attempts, 3);
    }

    #[tokio::test]
    async fn test_error_logger() {
        let logger = ErrorLogger::new(100);

        let error = RiceError::new(
            "Test error",
            ErrorCategory::System,
            ErrorSeverity::Medium,
            "test",
            "test",
        );

        logger.log_error(LogLevel::Error, error, HashMap::new()).await;

        let logs = logger.get_logs(None).await;
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].level, LogLevel::Error);
    }

    #[tokio::test]
    async fn test_error_manager() {
        let manager = ErrorManager::new();

        let error = RiceError::new(
            "Test error",
            ErrorCategory::System,
            ErrorSeverity::Medium,
            "test",
            "test",
        );

        let result = manager.handle_error(error).await;
        assert!(result.is_ok()); // Non-critical errors are handled

        let stats = manager.get_error_stats().await;
        assert_eq!(stats.get(&ErrorCategory::System), Some(&1));
    }
}