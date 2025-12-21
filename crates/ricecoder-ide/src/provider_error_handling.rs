//! Error handling for provider chain
//!
//! This module provides comprehensive error handling for the provider chain,
//! including graceful fallback, error recovery, and clear error messages.

use crate::error::{IdeError, IdeResult};
use std::fmt;
use tracing::{debug, error, warn};

/// Provider chain error context
#[derive(Debug, Clone)]
pub struct ProviderErrorContext {
    /// The language being processed
    pub language: String,
    /// The operation being performed (e.g., "completion", "diagnostics")
    pub operation: String,
    /// The provider that failed
    pub provider_name: String,
    /// The underlying error message
    pub error_message: String,
    /// Whether this is a recoverable error
    pub is_recoverable: bool,
}

impl fmt::Display for ProviderErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Provider '{}' failed for {} operation on language '{}': {}",
            self.provider_name, self.operation, self.language, self.error_message
        )
    }
}

/// Provider chain error handler
pub struct ProviderErrorHandler;

impl ProviderErrorHandler {
    /// Handle LSP server failure with graceful fallback
    pub fn handle_lsp_failure(
        language: &str,
        operation: &str,
        error: &IdeError,
    ) -> IdeResult<ProviderErrorContext> {
        debug!(
            "LSP server failed for {} operation on language '{}': {}",
            operation, language, error
        );

        let context = ProviderErrorContext {
            language: language.to_string(),
            operation: operation.to_string(),
            provider_name: "external_lsp".to_string(),
            error_message: error.to_string(),
            is_recoverable: true,
        };

        Ok(context)
    }

    /// Handle configuration error with remediation
    pub fn handle_config_error(error: &IdeError) -> String {
        warn!("Configuration error: {}", error);

        // Configuration errors are already detailed in the error message
        error.to_string()
    }

    /// Handle IDE communication error with retry logic
    pub fn handle_communication_error(
        ide_type: &str,
        error: &IdeError,
        retry_count: u32,
        max_retries: u32,
    ) -> IdeResult<()> {
        if retry_count < max_retries {
            debug!(
                "IDE communication error for '{}' (retry {}/{}): {}",
                ide_type, retry_count, max_retries, error
            );
            Ok(())
        } else {
            error!(
                "IDE communication error for '{}' after {} retries: {}",
                ide_type, max_retries, error
            );
            Err(IdeError::communication_error(format!(
                "Failed to communicate with {} IDE after {} retries: {}",
                ide_type, max_retries, error
            )))
        }
    }

    /// Handle timeout error with suggestions
    pub fn handle_timeout_error(language: &str, operation: &str, timeout_ms: u64) -> IdeError {
        warn!(
            "Timeout for {} operation on language '{}' after {}ms",
            operation, language, timeout_ms
        );

        IdeError::timeout(timeout_ms)
    }

    /// Create a fallback suggestion based on error context
    pub fn create_fallback_suggestion(context: &ProviderErrorContext) -> String {
        format!(
            "Provider '{}' failed for {} operation on language '{}'. \
             Falling back to next available provider in the chain. \
             If all providers fail, generic text-based features will be used.",
            context.provider_name, context.operation, context.language
        )
    }

    /// Create a recovery suggestion based on error type
    pub fn create_recovery_suggestion(error: &IdeError) -> String {
        match error {
            IdeError::LspError(msg) => {
                format!(
                    "LSP server error: {}. \
                     Recovery steps:\n\
                     1. Check if the LSP server is installed and running\n\
                     2. Verify the LSP server command in your configuration\n\
                     3. Check the LSP server logs for more details\n\
                     4. Try restarting the LSP server",
                    msg
                )
            }
            IdeError::ConfigError(msg) => {
                format!(
                    "Configuration error: {}. \
                     Recovery steps:\n\
                     1. Check your configuration file for syntax errors\n\
                     2. Verify all required fields are present\n\
                     3. Check the configuration documentation\n\
                     4. Try using the default configuration",
                    msg
                )
            }
            IdeError::ConfigValidationError(msg) => {
                format!(
                    "Configuration validation error: {}. \
                     Recovery steps:\n\
                     1. Review the validation error message\n\
                     2. Follow the remediation steps provided\n\
                     3. Verify your configuration against the schema\n\
                     4. Check the configuration documentation",
                    msg
                )
            }
            IdeError::PathResolutionError(msg) => {
                format!(
                    "Path resolution error: {}. \
                     Recovery steps:\n\
                     1. Check that the path exists\n\
                     2. Verify the path is readable\n\
                     3. Use absolute paths instead of relative paths\n\
                     4. Check for permission issues",
                    msg
                )
            }
            IdeError::CommunicationError(msg) => {
                format!(
                    "IDE communication error: {}. \
                     Recovery steps:\n\
                     1. Check that the IDE is running\n\
                     2. Verify the IDE is connected to ricecoder\n\
                     3. Check the IDE logs for errors\n\
                     4. Try restarting the IDE",
                    msg
                )
            }
            IdeError::Timeout(ms) => {
                format!(
                    "Operation timeout after {}ms. \
                     Recovery steps:\n\
                     1. Increase the timeout value in your configuration\n\
                     2. Check system resources (CPU, memory)\n\
                     3. Check network connectivity\n\
                     4. Try the operation again",
                    ms
                )
            }
            _ => {
                format!(
                    "Error: {}. \
                     Recovery steps:\n\
                     1. Check the error message for details\n\
                     2. Review the logs for more information\n\
                     3. Try the operation again\n\
                     4. Contact support if the issue persists",
                    error
                )
            }
        }
    }

    /// Log error with context
    pub fn log_error_with_context(context: &ProviderErrorContext) {
        if context.is_recoverable {
            debug!("Recoverable error: {}", context);
        } else {
            error!("Non-recoverable error: {}", context);
        }
    }
}

/// Provider chain error recovery strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryStrategy {
    /// Retry the operation with the same provider
    Retry,
    /// Fall back to the next provider in the chain
    Fallback,
    /// Use generic fallback provider
    GenericFallback,
    /// Fail and return error to caller
    Fail,
}

impl RecoveryStrategy {
    /// Determine recovery strategy based on error type
    pub fn from_error(error: &IdeError) -> Self {
        match error {
            IdeError::Timeout(_) => RecoveryStrategy::Retry,
            IdeError::LspError(_) => RecoveryStrategy::Fallback,
            IdeError::ProviderError(_) => RecoveryStrategy::Fallback,
            IdeError::CommunicationError(_) => RecoveryStrategy::Retry,
            IdeError::ConfigError(_) => RecoveryStrategy::Fail,
            IdeError::ConfigValidationError(_) => RecoveryStrategy::Fail,
            IdeError::PathResolutionError(_) => RecoveryStrategy::Fail,
            _ => RecoveryStrategy::Fallback,
        }
    }

    /// Get description of recovery strategy
    pub fn description(&self) -> &'static str {
        match self {
            RecoveryStrategy::Retry => "Retrying operation with same provider",
            RecoveryStrategy::Fallback => "Falling back to next provider in chain",
            RecoveryStrategy::GenericFallback => "Using generic fallback provider",
            RecoveryStrategy::Fail => "Failing and returning error to caller",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_error_context_display() {
        let context = ProviderErrorContext {
            language: "rust".to_string(),
            operation: "completion".to_string(),
            provider_name: "external_lsp".to_string(),
            error_message: "LSP server not found".to_string(),
            is_recoverable: true,
        };

        let display = context.to_string();
        assert!(display.contains("external_lsp"));
        assert!(display.contains("completion"));
        assert!(display.contains("rust"));
        assert!(display.contains("LSP server not found"));
    }

    #[test]
    fn test_recovery_strategy_from_timeout_error() {
        let error = IdeError::timeout(5000);
        let strategy = RecoveryStrategy::from_error(&error);
        assert_eq!(strategy, RecoveryStrategy::Retry);
    }

    #[test]
    fn test_recovery_strategy_from_lsp_error() {
        let error = IdeError::lsp_error("Server not found");
        let strategy = RecoveryStrategy::from_error(&error);
        assert_eq!(strategy, RecoveryStrategy::Fallback);
    }

    #[test]
    fn test_recovery_strategy_from_config_error() {
        let error = IdeError::config_error("Invalid configuration");
        let strategy = RecoveryStrategy::from_error(&error);
        assert_eq!(strategy, RecoveryStrategy::Fail);
    }

    #[test]
    fn test_recovery_strategy_from_communication_error() {
        let error = IdeError::communication_error("Connection lost");
        let strategy = RecoveryStrategy::from_error(&error);
        assert_eq!(strategy, RecoveryStrategy::Retry);
    }

    #[test]
    fn test_recovery_strategy_description() {
        assert_eq!(
            RecoveryStrategy::Retry.description(),
            "Retrying operation with same provider"
        );
        assert_eq!(
            RecoveryStrategy::Fallback.description(),
            "Falling back to next provider in chain"
        );
        assert_eq!(
            RecoveryStrategy::GenericFallback.description(),
            "Using generic fallback provider"
        );
        assert_eq!(
            RecoveryStrategy::Fail.description(),
            "Failing and returning error to caller"
        );
    }

    #[test]
    fn test_fallback_suggestion() {
        let context = ProviderErrorContext {
            language: "rust".to_string(),
            operation: "completion".to_string(),
            provider_name: "external_lsp".to_string(),
            error_message: "Server error".to_string(),
            is_recoverable: true,
        };

        let suggestion = ProviderErrorHandler::create_fallback_suggestion(&context);
        assert!(suggestion.contains("external_lsp"));
        assert!(suggestion.contains("completion"));
        assert!(suggestion.contains("Falling back"));
    }

    #[test]
    fn test_recovery_suggestion_for_lsp_error() {
        let error = IdeError::lsp_error("Server not found");
        let suggestion = ProviderErrorHandler::create_recovery_suggestion(&error);
        assert!(suggestion.contains("LSP server error"));
        assert!(suggestion.contains("Recovery steps"));
        assert!(suggestion.contains("installed"));
    }

    #[test]
    fn test_recovery_suggestion_for_config_error() {
        let error = IdeError::config_error("Invalid YAML");
        let suggestion = ProviderErrorHandler::create_recovery_suggestion(&error);
        assert!(suggestion.contains("Configuration error"));
        assert!(suggestion.contains("Recovery steps"));
        assert!(suggestion.contains("syntax errors"));
    }

    #[test]
    fn test_recovery_suggestion_for_timeout() {
        let error = IdeError::timeout(5000);
        let suggestion = ProviderErrorHandler::create_recovery_suggestion(&error);
        assert!(suggestion.contains("timeout"));
        assert!(suggestion.contains("5000"));
        assert!(suggestion.contains("Recovery steps"));
    }
}
