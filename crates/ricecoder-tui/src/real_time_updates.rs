//! Real-time updates system for RiceCoder TUI
//!
//! This module implements:
//! - Real-time data streaming architecture
//! - Background operation tracking and management
//! - Progress indicators and reporting
//! - Operation cancellation and coordination
//! - Streaming message rendering with live updates

use crate::error_handling::{ErrorManager, RiceError, ErrorCategory, ErrorSeverity};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc, broadcast};
use tokio::time::{Instant, Duration};
use tokio_util::sync::CancellationToken;

/// Stream data types for different kinds of real-time updates
#[derive(Debug, Clone)]
pub enum StreamData {
    /// Text token for streaming messages
    TextToken(String),
    /// Progress update (0.0 to 1.0)
    ProgressUpdate(f32),
    /// Operation status change
    StatusUpdate(OperationStatus),
    /// Error occurred
    Error(RiceError),
    /// Operation completed successfully
    Completion(String),
}

/// Stream types for different data sources
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StreamType {
    /// AI chat responses
    ChatResponse,
    /// File operations
    FileOperation,
    /// Network requests
    NetworkRequest,
    /// Background processing
    BackgroundTask,
    /// System monitoring
    SystemMonitor,
}

/// Operation status for tracking background operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationStatus {
    /// Operation is queued
    Queued,
    /// Operation is running
    Running,
    /// Operation is paused
    Paused,
    /// Operation completed successfully
    Completed,
    /// Operation failed
    Failed,
    /// Operation was cancelled
    Cancelled,
}

/// Background operation metadata
#[derive(Debug, Clone)]
pub struct OperationInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub operation_type: StreamType,
    pub status: OperationStatus,
    pub progress: Option<f32>,
    pub start_time: Instant,
    pub estimated_duration: Option<Duration>,
    pub cancellation_token: CancellationToken,
}

/// Real-time stream for a specific operation
pub struct RealTimeStream {
    operation_id: String,
    stream_type: StreamType,
    sender: mpsc::UnboundedSender<StreamData>,
    receiver: broadcast::Receiver<StreamData>,
    is_active: Arc<RwLock<bool>>,
    operation_info: Arc<RwLock<OperationInfo>>,
}

impl RealTimeStream {
    /// Create a new real-time stream
    pub fn new(operation_id: String, stream_type: StreamType, name: String, description: String) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let (broadcast_tx, broadcast_rx) = broadcast::channel(100);

        // Forward messages from mpsc to broadcast
        let tx_clone = broadcast_tx.clone();
        tokio::spawn(async move {
            while let Some(data) = rx.recv().await {
                let _ = tx_clone.send(data);
            }
        });

        let operation_info = OperationInfo {
            id: operation_id.clone(),
            name,
            description,
            operation_type: stream_type,
            status: OperationStatus::Queued,
            progress: None,
            start_time: Instant::now(),
            estimated_duration: None,
            cancellation_token: CancellationToken::new(),
        };

        Self {
            operation_id,
            stream_type,
            sender: tx,
            receiver: broadcast_rx,
            is_active: Arc::new(RwLock::new(true)),
            operation_info: Arc::new(RwLock::new(operation_info)),
        }
    }

    /// Send data to the stream
    pub async fn send(&self, data: StreamData) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if *self.is_active.read().await {
            self.sender.send(data)?;
            Ok(())
        } else {
            Err("Stream is not active".into())
        }
    }

    /// Update operation status
    pub async fn update_status(&self, status: OperationStatus) {
        let mut info = self.operation_info.write().await;
        info.status = status;

        let _ = self.send(StreamData::StatusUpdate(status)).await;
    }

    /// Update progress
    pub async fn update_progress(&self, progress: f32) {
        let mut info = self.operation_info.write().await;
        info.progress = Some(progress.clamp(0.0, 1.0));

        let _ = self.send(StreamData::ProgressUpdate(progress)).await;
    }

    /// Mark operation as completed
    pub async fn complete(&self, result: String) {
        self.update_status(OperationStatus::Completed).await;
        let _ = self.send(StreamData::Completion(result)).await;
        *self.is_active.write().await = false;
    }

    /// Mark operation as failed
    pub async fn fail(&self, error: RiceError) {
        self.update_status(OperationStatus::Failed).await;
        let _ = self.send(StreamData::Error(error)).await;
        *self.is_active.write().await = false;
    }

    /// Cancel the operation
    pub async fn cancel(&self) {
        self.operation_info.read().await.cancellation_token.cancel();
        self.update_status(OperationStatus::Cancelled).await;
        *self.is_active.write().await = false;
    }

    /// Get operation info
    pub async fn operation_info(&self) -> OperationInfo {
        self.operation_info.read().await.clone()
    }

    /// Check if stream is active
    pub async fn is_active(&self) -> bool {
        *self.is_active.read().await
    }

    /// Subscribe to stream updates
    pub fn subscribe(&self) -> broadcast::Receiver<StreamData> {
        self.receiver.resubscribe()
    }
}

/// Real-time updates coordinator
pub struct RealTimeUpdates {
    streams: Arc<RwLock<HashMap<String, Arc<RealTimeStream>>>>,
    error_manager: ErrorManager,
    update_sender: broadcast::Sender<(String, StreamData)>,
    update_receiver: broadcast::Receiver<(String, StreamData)>,
}

impl RealTimeUpdates {
    /// Create a new real-time updates coordinator
    pub fn new(error_manager: ErrorManager) -> Self {
        let (tx, rx) = broadcast::channel(1000);

        Self {
            streams: Arc::new(RwLock::new(HashMap::new())),
            error_manager,
            update_sender: tx,
            update_receiver: rx,
        }
    }

    /// Create a new real-time stream
    pub async fn create_stream(&self, operation_id: String, stream_type: StreamType, name: String, description: String) -> Arc<RealTimeStream> {
        let stream = Arc::new(RealTimeStream::new(operation_id.clone(), stream_type, name, description));

        let mut streams = self.streams.write().await;
        streams.insert(operation_id, Arc::clone(&stream));

        stream
    }

    /// Get a stream by operation ID
    pub async fn get_stream(&self, operation_id: &str) -> Option<Arc<RealTimeStream>> {
        let streams = self.streams.read().await;
        streams.get(operation_id).cloned()
    }

    /// Remove a completed stream
    pub async fn remove_stream(&self, operation_id: &str) {
        let mut streams = self.streams.write().await;
        streams.remove(operation_id);
    }

    /// Get all active streams
    pub async fn active_streams(&self) -> Vec<Arc<RealTimeStream>> {
        let streams = self.streams.read().await;
        streams.values().cloned().collect()
    }

    /// Get streams by type
    pub async fn streams_by_type(&self, stream_type: StreamType) -> Vec<Arc<RealTimeStream>> {
        let streams = self.streams.read().await;
        streams.values()
            .filter(|stream| stream.stream_type == stream_type)
            .cloned()
            .collect()
    }

    /// Cancel all streams of a specific type
    pub async fn cancel_by_type(&self, stream_type: StreamType) {
        let streams = self.streams_by_type(stream_type).await;
        for stream in streams {
            stream.cancel().await;
        }
    }

    /// Get global update receiver
    pub fn global_receiver(&self) -> broadcast::Receiver<(String, StreamData)> {
        self.update_receiver.resubscribe()
    }

    /// Process stream updates and forward to global channel
    pub async fn process_updates(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut global_rx = self.global_receiver();

        loop {
            match global_rx.recv().await {
                Ok((operation_id, data)) => {
                    // Handle different data types
                    match &data {
                        StreamData::Error(error) => {
                            let _ = self.error_manager.handle_error(error.clone()).await;
                        }
                        StreamData::Completion(_) => {
                            // Could trigger cleanup or notifications
                            tracing::info!("Operation {} completed", operation_id);
                        }
                        _ => {}
                    }

                    // Forward to any listeners
                    tracing::debug!("Real-time update: {} - {:?}", operation_id, data);
                }
                Err(broadcast::error::RecvError::Closed) => {
                    break;
                }
                Err(broadcast::error::RecvError::Lagged(_)) => {
                    tracing::warn!("Real-time updates receiver lagged");
                }
            }
        }

        Ok(())
    }

    /// Get statistics about active operations
    pub async fn get_statistics(&self) -> RealTimeStats {
        let streams = self.active_streams().await;
        let mut stats = RealTimeStats::default();

        for stream in streams {
            let info = stream.operation_info().await;
            stats.total_operations += 1;

            match info.status {
                OperationStatus::Queued => stats.queued_operations += 1,
                OperationStatus::Running => stats.running_operations += 1,
                OperationStatus::Paused => stats.paused_operations += 1,
                OperationStatus::Completed => stats.completed_operations += 1,
                OperationStatus::Failed => stats.failed_operations += 1,
                OperationStatus::Cancelled => stats.cancelled_operations += 1,
            }

            match info.operation_type {
                StreamType::ChatResponse => stats.chat_operations += 1,
                StreamType::FileOperation => stats.file_operations += 1,
                StreamType::NetworkRequest => stats.network_operations += 1,
                StreamType::BackgroundTask => stats.background_operations += 1,
                StreamType::SystemMonitor => stats.system_operations += 1,
            }
        }

        stats
    }
}

/// Statistics about real-time operations
#[derive(Debug, Clone, Default)]
pub struct RealTimeStats {
    pub total_operations: usize,
    pub queued_operations: usize,
    pub running_operations: usize,
    pub paused_operations: usize,
    pub completed_operations: usize,
    pub failed_operations: usize,
    pub cancelled_operations: usize,
    pub chat_operations: usize,
    pub file_operations: usize,
    pub network_operations: usize,
    pub background_operations: usize,
    pub system_operations: usize,
}

/// Progress indicator widget for real-time updates
pub struct ProgressIndicator {
    operation_id: String,
    stream: Option<Arc<RealTimeStream>>,
    last_progress: Arc<RwLock<Option<f32>>>,
    last_status: Arc<RwLock<OperationStatus>>,
}

impl ProgressIndicator {
    /// Create a new progress indicator
    pub fn new(operation_id: String) -> Self {
        Self {
            operation_id,
            stream: None,
            last_progress: Arc::new(RwLock::new(None)),
            last_status: Arc::new(RwLock::new(OperationStatus::Queued)),
        }
    }

    /// Attach to a real-time stream
    pub fn attach_stream(&mut self, stream: Arc<RealTimeStream>) {
        self.stream = Some(stream);
    }

    /// Update from stream data
    pub async fn update_from_stream(&self, data: &StreamData) {
        match data {
            StreamData::ProgressUpdate(progress) => {
                *self.last_progress.write().await = Some(*progress);
            }
            StreamData::StatusUpdate(status) => {
                *self.last_status.write().await = *status;
            }
            _ => {}
        }
    }

    /// Get current progress (0.0 to 1.0)
    pub async fn progress(&self) -> Option<f32> {
        *self.last_progress.read().await
    }

    /// Get current status
    pub async fn status(&self) -> OperationStatus {
        *self.last_status.read().await
    }

    /// Get progress bar string
    pub async fn progress_bar(&self, width: usize) -> String {
        let progress = self.progress().await.unwrap_or(0.0);
        let filled = (width as f32 * progress) as usize;
        let empty = width.saturating_sub(filled);

        let filled_chars = "█".repeat(filled);
        let empty_chars = "░".repeat(empty);

        format!("[{}{}] {:.1}%", filled_chars, empty_chars, progress * 100.0)
    }

    /// Get status text
    pub async fn status_text(&self) -> String {
        match self.status().await {
            OperationStatus::Queued => "Queued",
            OperationStatus::Running => "Running",
            OperationStatus::Paused => "Paused",
            OperationStatus::Completed => "Completed",
            OperationStatus::Failed => "Failed",
            OperationStatus::Cancelled => "Cancelled",
        }.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

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
        let stream = updates.create_stream(
            "test_stream".to_string(),
            StreamType::BackgroundTask,
            "Background Task".to_string(),
            "Running background work".to_string(),
        ).await;

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
        indicator.update_from_stream(&StreamData::ProgressUpdate(0.75)).await;
        indicator.update_from_stream(&StreamData::StatusUpdate(OperationStatus::Running)).await;

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