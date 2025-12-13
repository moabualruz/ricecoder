use ricecoder_tui::*;
use std::collections::HashMap;

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