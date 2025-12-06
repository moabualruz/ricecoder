//! Error reporting and logging infrastructure

use crate::error::{Error, ErrorContext, ErrorLogEntry};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// Error reporter for structured error logging and reporting
#[derive(Debug, Clone)]
pub struct ErrorReporter {
    logs: Arc<RwLock<Vec<ErrorLogEntry>>>,
    max_logs: usize,
}

impl ErrorReporter {
    /// Creates a new error reporter
    pub fn new() -> Self {
        Self {
            logs: Arc::new(RwLock::new(Vec::new())),
            max_logs: 1000,
        }
    }

    /// Creates a new error reporter with custom max logs
    pub fn with_max_logs(max_logs: usize) -> Self {
        Self {
            logs: Arc::new(RwLock::new(Vec::new())),
            max_logs,
        }
    }

    /// Reports an error with context
    pub async fn report_error(&self, error: &Error, context: Option<ErrorContext>) {
        let mut log_entry = ErrorLogEntry::new(
            error.error_type().to_string(),
            error.to_string(),
        )
        .with_recoverable(error.is_recoverable());

        if let Some(ctx) = context {
            if let Some(tool_id) = ctx.tool_id {
                log_entry = log_entry.with_tool_id(tool_id);
            }
            if let Some(server_id) = ctx.server_id {
                log_entry = log_entry.with_server_id(server_id);
            }
            if let Some(parameters) = ctx.parameters {
                log_entry = log_entry.with_parameters(parameters);
            }
            if let Some(stack_trace) = ctx.stack_trace {
                log_entry = log_entry.with_stack_trace(stack_trace);
            }
        }

        // Log to tracing
        match error {
            Error::ToolNotFound(_) | Error::PermissionDenied(_) | Error::NamingConflict(_) => {
                error!("Error: {} - {}", log_entry.error_type, log_entry.message);
            }
            Error::TimeoutError(_) | Error::ConnectionError(_) => {
                warn!("Error: {} - {}", log_entry.error_type, log_entry.message);
            }
            _ => {
                info!("Error: {} - {}", log_entry.error_type, log_entry.message);
            }
        }

        // Store in logs
        let mut logs = self.logs.write().await;
        logs.push(log_entry);

        // Trim logs if exceeding max
        if logs.len() > self.max_logs {
            let remove_count = logs.len() - self.max_logs;
            logs.drain(0..remove_count);
        }
    }

    /// Reports an error with retry count
    pub async fn report_error_with_retry(
        &self,
        error: &Error,
        context: Option<ErrorContext>,
        retry_count: u32,
    ) {
        let mut log_entry = ErrorLogEntry::new(
            error.error_type().to_string(),
            error.to_string(),
        )
        .with_recoverable(error.is_recoverable())
        .with_retry_count(retry_count);

        if let Some(ctx) = context {
            if let Some(tool_id) = ctx.tool_id {
                log_entry = log_entry.with_tool_id(tool_id);
            }
            if let Some(server_id) = ctx.server_id {
                log_entry = log_entry.with_server_id(server_id);
            }
            if let Some(parameters) = ctx.parameters {
                log_entry = log_entry.with_parameters(parameters);
            }
            if let Some(stack_trace) = ctx.stack_trace {
                log_entry = log_entry.with_stack_trace(stack_trace);
            }
        }

        warn!(
            "Error (retry {}): {} - {}",
            retry_count, log_entry.error_type, log_entry.message
        );

        let mut logs = self.logs.write().await;
        logs.push(log_entry);

        if logs.len() > self.max_logs {
            let remove_count = logs.len() - self.max_logs;
            logs.drain(0..remove_count);
        }
    }

    /// Gets all error logs
    pub async fn get_logs(&self) -> Vec<ErrorLogEntry> {
        let logs = self.logs.read().await;
        logs.clone()
    }

    /// Gets error logs filtered by error type
    pub async fn get_logs_by_type(&self, error_type: &str) -> Vec<ErrorLogEntry> {
        let logs = self.logs.read().await;
        logs.iter()
            .filter(|log| log.error_type == error_type)
            .cloned()
            .collect()
    }

    /// Gets error logs filtered by tool ID
    pub async fn get_logs_by_tool(&self, tool_id: &str) -> Vec<ErrorLogEntry> {
        let logs = self.logs.read().await;
        logs.iter()
            .filter(|log| log.tool_id.as_deref() == Some(tool_id))
            .cloned()
            .collect()
    }

    /// Gets error logs filtered by server ID
    pub async fn get_logs_by_server(&self, server_id: &str) -> Vec<ErrorLogEntry> {
        let logs = self.logs.read().await;
        logs.iter()
            .filter(|log| log.server_id.as_deref() == Some(server_id))
            .cloned()
            .collect()
    }

    /// Gets the count of error logs
    pub async fn log_count(&self) -> usize {
        let logs = self.logs.read().await;
        logs.len()
    }

    /// Clears all error logs
    pub async fn clear_logs(&self) {
        let mut logs = self.logs.write().await;
        logs.clear();
        info!("Error logs cleared");
    }

    /// Gets error statistics
    pub async fn get_statistics(&self) -> ErrorStatistics {
        let logs = self.logs.read().await;

        let total_errors = logs.len();
        let recoverable_errors = logs.iter().filter(|l| l.is_recoverable).count();
        let permanent_errors = total_errors - recoverable_errors;

        let mut error_type_counts = std::collections::HashMap::new();
        for log in logs.iter() {
            *error_type_counts.entry(log.error_type.clone()).or_insert(0) += 1;
        }

        let mut tool_error_counts = std::collections::HashMap::new();
        for log in logs.iter() {
            if let Some(tool_id) = &log.tool_id {
                *tool_error_counts.entry(tool_id.clone()).or_insert(0) += 1;
            }
        }

        let mut server_error_counts = std::collections::HashMap::new();
        for log in logs.iter() {
            if let Some(server_id) = &log.server_id {
                *server_error_counts.entry(server_id.clone()).or_insert(0) += 1;
            }
        }

        ErrorStatistics {
            total_errors,
            recoverable_errors,
            permanent_errors,
            error_type_counts,
            tool_error_counts,
            server_error_counts,
        }
    }
}

impl Default for ErrorReporter {
    fn default() -> Self {
        Self::new()
    }
}

/// Error statistics
#[derive(Debug, Clone)]
pub struct ErrorStatistics {
    pub total_errors: usize,
    pub recoverable_errors: usize,
    pub permanent_errors: usize,
    pub error_type_counts: std::collections::HashMap<String, usize>,
    pub tool_error_counts: std::collections::HashMap<String, usize>,
    pub server_error_counts: std::collections::HashMap<String, usize>,
}

impl ErrorStatistics {
    /// Gets the most common error type
    pub fn most_common_error_type(&self) -> Option<(String, usize)> {
        self.error_type_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(error_type, count)| (error_type.clone(), *count))
    }

    /// Gets the most problematic tool
    pub fn most_problematic_tool(&self) -> Option<(String, usize)> {
        self.tool_error_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(tool_id, count)| (tool_id.clone(), *count))
    }

    /// Gets the most problematic server
    pub fn most_problematic_server(&self) -> Option<(String, usize)> {
        self.server_error_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(server_id, count)| (server_id.clone(), *count))
    }

    /// Gets the error recovery rate
    pub fn recovery_rate(&self) -> f64 {
        if self.total_errors == 0 {
            0.0
        } else {
            (self.recoverable_errors as f64 / self.total_errors as f64) * 100.0
        }
    }
}

/// User-friendly error message formatter
pub struct ErrorMessageFormatter;

impl ErrorMessageFormatter {
    /// Formats an error for user display
    pub fn format_for_user(error: &Error) -> String {
        error.user_message()
    }

    /// Formats an error with context for user display
    pub fn format_with_context(error: &Error, context: &ErrorContext) -> String {
        let mut message = error.user_message();

        if let Some(tool_id) = &context.tool_id {
            message.push_str(&format!("\nTool: {}", tool_id));
        }

        if let Some(parameters) = &context.parameters {
            message.push_str(&format!("\nParameters: {}", parameters));
        }

        if let Some(server_id) = &context.server_id {
            message.push_str(&format!("\nServer: {}", server_id));
        }

        message
    }

    /// Formats an error for logging
    pub fn format_for_logging(error: &Error, context: Option<&ErrorContext>) -> String {
        let mut message = format!("[{}] {}", error.error_type(), error);

        if let Some(ctx) = context {
            if let Some(tool_id) = &ctx.tool_id {
                message.push_str(&format!(" [tool: {}]", tool_id));
            }
            if let Some(server_id) = &ctx.server_id {
                message.push_str(&format!(" [server: {}]", server_id));
            }
            if let Some(parameters) = &ctx.parameters {
                message.push_str(&format!(" [params: {}]", parameters));
            }
        }

        message
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_error_reporter() {
        let reporter = ErrorReporter::new();
        assert_eq!(reporter.log_count().await, 0);
    }

    #[tokio::test]
    async fn test_report_error() {
        let reporter = ErrorReporter::new();
        let error = Error::ToolNotFound("test-tool".to_string());

        reporter.report_error(&error, None).await;

        assert_eq!(reporter.log_count().await, 1);
        let logs = reporter.get_logs().await;
        assert_eq!(logs[0].error_type, "ToolNotFound");
    }

    #[tokio::test]
    async fn test_report_error_with_context() {
        let reporter = ErrorReporter::new();
        let error = Error::ExecutionError("Tool failed".to_string());
        let context = ErrorContext::new()
            .with_tool_id("test-tool".to_string())
            .with_server_id("server1".to_string());

        reporter.report_error(&error, Some(context)).await;

        let logs = reporter.get_logs().await;
        assert_eq!(logs[0].tool_id, Some("test-tool".to_string()));
        assert_eq!(logs[0].server_id, Some("server1".to_string()));
    }

    #[tokio::test]
    async fn test_get_logs_by_type() {
        let reporter = ErrorReporter::new();

        reporter
            .report_error(&Error::ToolNotFound("tool1".to_string()), None)
            .await;
        reporter
            .report_error(&Error::ToolNotFound("tool2".to_string()), None)
            .await;
        reporter
            .report_error(&Error::ConnectionError("conn".to_string()), None)
            .await;

        let logs = reporter.get_logs_by_type("ToolNotFound").await;
        assert_eq!(logs.len(), 2);
    }

    #[tokio::test]
    async fn test_get_logs_by_tool() {
        let reporter = ErrorReporter::new();

        let context1 = ErrorContext::new().with_tool_id("tool1".to_string());
        let context2 = ErrorContext::new().with_tool_id("tool2".to_string());

        reporter
            .report_error(&Error::ExecutionError("failed".to_string()), Some(context1))
            .await;
        reporter
            .report_error(&Error::ExecutionError("failed".to_string()), Some(context2))
            .await;

        let logs = reporter.get_logs_by_tool("tool1").await;
        assert_eq!(logs.len(), 1);
    }

    #[tokio::test]
    async fn test_get_statistics() {
        let reporter = ErrorReporter::new();

        reporter
            .report_error(&Error::ToolNotFound("tool1".to_string()), None)
            .await;
        reporter
            .report_error(&Error::ToolNotFound("tool2".to_string()), None)
            .await;
        reporter
            .report_error(&Error::ConnectionError("conn".to_string()), None)
            .await;

        let stats = reporter.get_statistics().await;
        assert_eq!(stats.total_errors, 3);
        assert_eq!(stats.error_type_counts.get("ToolNotFound"), Some(&2));
        assert_eq!(stats.error_type_counts.get("ConnectionError"), Some(&1));
    }

    #[tokio::test]
    async fn test_clear_logs() {
        let reporter = ErrorReporter::new();

        reporter
            .report_error(&Error::ToolNotFound("tool1".to_string()), None)
            .await;
        assert_eq!(reporter.log_count().await, 1);

        reporter.clear_logs().await;
        assert_eq!(reporter.log_count().await, 0);
    }

    #[test]
    fn test_error_message_formatter() {
        let error = Error::ToolNotFound("test-tool".to_string());
        let message = ErrorMessageFormatter::format_for_user(&error);
        assert!(message.contains("test-tool"));
    }

    #[test]
    fn test_error_message_formatter_with_context() {
        let error = Error::ExecutionError("failed".to_string());
        let context = ErrorContext::new()
            .with_tool_id("test-tool".to_string())
            .with_server_id("server1".to_string());

        let message = ErrorMessageFormatter::format_with_context(&error, &context);
        assert!(message.contains("test-tool"));
        assert!(message.contains("server1"));
    }

    #[test]
    fn test_error_statistics_most_common() {
        let mut stats = ErrorStatistics {
            total_errors: 3,
            recoverable_errors: 1,
            permanent_errors: 2,
            error_type_counts: std::collections::HashMap::new(),
            tool_error_counts: std::collections::HashMap::new(),
            server_error_counts: std::collections::HashMap::new(),
        };

        stats.error_type_counts.insert("ToolNotFound".to_string(), 2);
        stats.error_type_counts.insert("ConnectionError".to_string(), 1);

        let most_common = stats.most_common_error_type();
        assert_eq!(most_common, Some(("ToolNotFound".to_string(), 2)));
    }

    #[test]
    fn test_error_statistics_recovery_rate() {
        let stats = ErrorStatistics {
            total_errors: 10,
            recoverable_errors: 7,
            permanent_errors: 3,
            error_type_counts: std::collections::HashMap::new(),
            tool_error_counts: std::collections::HashMap::new(),
            server_error_counts: std::collections::HashMap::new(),
        };

        assert_eq!(stats.recovery_rate(), 70.0);
    }
}
