use ricecoder_tui::*;
use std::time::Duration;

mod tests {
    use super::*;

    #[tokio::test]
    async fn test_real_time_stream_creation() {
        let stream = RealTimeStream::new(
            "test_op".to_string(),
            StreamType::ChatResponse,
            "Test Operation".to_string(),
            "A test operation".to_string(),
        );

        assert_eq!(stream.operation_id, "test_op");
        assert_eq!(stream.stream_type, StreamType::ChatResponse);
        assert!(stream.is_active().await);

        let info = stream.operation_info().await;
        assert_eq!(info.name, "Test Operation");
        assert_eq!(info.status, OperationStatus::Queued);
    }

    #[tokio::test]
    async fn test_stream_progress_updates() {
        let stream = RealTimeStream::new(
            "test_op".to_string(),
            StreamType::FileOperation,
            "File Copy".to_string(),
            "Copying files".to_string(),
        );

        // Update progress
        stream.update_progress(0.5).await;

        let info = stream.operation_info().await;
        assert_eq!(info.progress, Some(0.5));

        // Update status
        stream.update_status(OperationStatus::Running).await;

        let info = stream.operation_info().await;
        assert_eq!(info.status, OperationStatus::Running);
    }

    #[tokio::test]
    async fn test_stream_completion() {
        let stream = RealTimeStream::new(
            "test_op".to_string(),
            StreamType::NetworkRequest,
            "API Call".to_string(),
            "Making API request".to_string(),
        );

        stream.complete("Success!".to_string()).await;

        let info = stream.operation_info().await;
        assert_eq!(info.status, OperationStatus::Completed);
        assert!(!stream.is_active().await);
    }

    #[tokio::test]
    async fn test_real_time_updates_coordination() {
        let error_manager = ErrorManager::new();
        let updates = RealTimeUpdates::new(error_manager);

        // Create a stream
        let stream = updates
            .create_stream(
                "test_stream".to_string(),
                StreamType::BackgroundTask,
                "Background Task".to_string(),
                "Running background work".to_string(),
            )
            .await;

        // Get the stream back
        let retrieved = updates.get_stream("test_stream").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().operation_id, "test_stream");

        // Check statistics
        let stats = updates.get_statistics().await;
        assert_eq!(stats.total_operations, 1);
        assert_eq!(stats.background_operations, 1);
    }

    #[tokio::test]
    async fn test_progress_indicator() {
        let indicator = ProgressIndicator::new("test_op".to_string());

        // Initially no progress
        assert_eq!(indicator.progress().await, None);
        assert_eq!(indicator.status().await, OperationStatus::Queued);

        // Update progress
        indicator
            .update_from_stream(&StreamData::ProgressUpdate(0.75))
            .await;
        indicator
            .update_from_stream(&StreamData::StatusUpdate(OperationStatus::Running))
            .await;

        assert_eq!(indicator.progress().await, Some(0.75));
        assert_eq!(indicator.status().await, OperationStatus::Running);

        // Test progress bar
        let bar = indicator.progress_bar(10).await;
        assert!(bar.contains("███████░")); // 7 filled, 3 empty
        assert!(bar.contains("75.0%"));
    }

    #[tokio::test]
    async fn test_stream_cancellation() {
        let stream = RealTimeStream::new(
            "test_op".to_string(),
            StreamType::ChatResponse,
            "Chat".to_string(),
            "AI chat response".to_string(),
        );

        // Cancel the stream
        stream.cancel().await;

        let info = stream.operation_info().await;
        assert_eq!(info.status, OperationStatus::Cancelled);
        assert!(!stream.is_active().await);
    }
}
