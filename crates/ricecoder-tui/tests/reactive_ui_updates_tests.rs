use ricecoder_tui::*;
use crate::tea::AppModel;
use std::sync::Arc;
use tokio::sync::RwLock;

mod tests {
    use super::*;

    #[tokio::test]
    async fn test_reactive_renderer_creation() {
        let model = AppModel::default();
        let reactive_state = Arc::new(RwLock::new(ReactiveState::new(model)));
        let renderer = ReactiveRenderer::new(reactive_state, Duration::from_millis(100));

        assert!(!renderer.is_running().await);
    }

    #[tokio::test]
    async fn test_reactive_renderer_start_stop() {
        let model = AppModel::default();
        let reactive_state = Arc::new(RwLock::new(ReactiveState::new(model)));
        let renderer = ReactiveRenderer::new(reactive_state, Duration::from_millis(100));

        renderer.start().await.unwrap();
        assert!(renderer.is_running().await);

        renderer.stop().await;
        // Give a moment for the task to stop
        tokio::time::sleep(Duration::from_millis(10)).await;
        assert!(!renderer.is_running().await);
    }

    #[tokio::test]
    async fn test_live_data_synchronizer_creation() {
        let error_manager = ErrorManager::new();
        let synchronizer = LiveDataSynchronizer::new(error_manager);

        assert!(!synchronizer.is_running().await);
    }

    #[tokio::test]
    async fn test_conflict_resolver() {
        let mut resolver = ConflictResolver::new();

        resolver.set_strategy("test_file".to_string(), ConflictResolution::UseLocal);

        let conflict = ConflictInfo {
            resource_id: "test_file".to_string(),
            local_version: vec![1, 2, 3],
            remote_version: vec![4, 5, 6],
            conflict_type: ConflictType::FileModified,
        };

        let strategy = resolver.resolve(&conflict);
        assert_eq!(strategy, ConflictResolution::UseLocal);
    }

    #[tokio::test]
    async fn test_reactive_ui_coordinator_creation() {
        let model = AppModel::default();
        let reactive_state = Arc::new(RwLock::new(ReactiveState::new(model)));
        let error_manager = ErrorManager::new();
        let real_time_updates = RealTimeUpdates::new(error_manager.clone());

        let coordinator = ReactiveUICoordinator::new(
            reactive_state,
            real_time_updates,
            error_manager,
        );

        assert!(!coordinator.is_running().await);
    }
}