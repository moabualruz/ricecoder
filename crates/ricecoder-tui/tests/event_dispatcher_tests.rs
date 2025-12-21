use crate::tea::{AppModel, ReactiveState};
use ricecoder_tui::*;
use std::sync::Arc;
use tokio::sync::RwLock;

mod tests {
    use super::*;

    fn create_test_model() -> AppModel {
        // Create a minimal test model
        AppModel {
            mode: crate::AppMode::Chat,
            previous_mode: crate::AppMode::Chat,
            theme: crate::style::Theme::default(),
            terminal_caps: crate::terminal_state::TerminalCapabilities::default(),
            sessions: crate::tea::SessionState {
                active_session_id: None,
                session_count: 0,
                total_tokens: ricecoder_sessions::TokenUsage::default(),
            },
            commands: crate::tea::TeaCommandState {
                command_history: vec![],
                current_command: "".to_string(),
                command_palette_visible: false,
            },
            ui: crate::tea::UiState {
                focus_manager: crate::accessibility::FocusManager::new(),
                keyboard_nav: crate::accessibility::KeyboardNavigationManager::new(),
                screen_reader: crate::accessibility::ScreenReaderAnnouncer::new(),
                chat_widget: crate::widgets::ChatWidget::new(),
                help_dialog: ricecoder_help::HelpDialog::default_ricecoder(),
                file_picker_visible: false,
                config: ricecoder_storage::TuiConfig::default(),
            },
            pending_operations: std::collections::HashMap::new(),
            subscriptions: vec![],
        }
    }

    #[tokio::test]
    async fn test_event_dispatcher_creation() {
        let dispatcher = EventDispatcher::new();
        let stats = dispatcher.get_stats().await;
        assert_eq!(stats.total_events, 0);
        assert_eq!(stats.processed_events, 0);
    }

    #[tokio::test]
    async fn test_event_dispatch() {
        let dispatcher = EventDispatcher::new();

        // Dispatch an event
        let event_id = dispatcher
            .dispatch_event(
                AppMessage::ModeChanged(crate::AppMode::Command),
                EventPriority::Normal,
                EventSource::UserInput,
            )
            .await
            .unwrap();

        assert!(!event_id.is_empty());

        // Check stats
        let stats = dispatcher.get_stats().await;
        assert_eq!(stats.total_events, 1);
    }

    #[tokio::test]
    async fn test_event_cancellation() {
        let dispatcher = EventDispatcher::new();

        // Dispatch an event
        let event_id = dispatcher
            .dispatch_event(
                AppMessage::ModeChanged(crate::AppMode::Command),
                EventPriority::Normal,
                EventSource::UserInput,
            )
            .await
            .unwrap();

        // Cancel it
        let cancelled = dispatcher.cancel_event(&event_id).await;
        assert!(cancelled);

        // Try to cancel again
        let cancelled_again = dispatcher.cancel_event(&event_id).await;
        assert!(!cancelled_again);
    }

    #[tokio::test]
    async fn test_optimistic_updater() {
        let updater = OptimisticUpdater::new();

        let mut counter = 0;
        let event_id = "test_event".to_string();

        // Apply optimistic update
        updater
            .apply_optimistic(
                event_id.clone(),
                "Test update".to_string(),
                Duration::from_secs(5),
                || counter += 1,
                || counter -= 1,
            )
            .await;

        assert_eq!(counter, 1);

        // Confirm the update
        updater.confirm_update(&event_id).await;

        // Counter should still be 1 (rollback not called)
        assert_eq!(counter, 1);
    }

    #[tokio::test]
    async fn test_optimistic_rollback() {
        let updater = OptimisticUpdater::new();

        let mut counter = 0;
        let event_id = "test_event".to_string();

        // Apply optimistic update
        updater
            .apply_optimistic(
                event_id.clone(),
                "Test update".to_string(),
                Duration::from_secs(5),
                || counter += 1,
                || counter -= 1,
            )
            .await;

        assert_eq!(counter, 0); // Should be rolled back
    }

    #[tokio::test]
    async fn test_loading_manager() {
        let manager = LoadingManager::new();

        // Start loading
        manager
            .start_loading("test_op".to_string(), "Test operation".to_string())
            .await;

        // Check active loadings
        let active = manager.get_active_loadings().await;
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].operation_id, "test_op");
        assert!(manager.has_active_loadings().await);

        // Update progress
        manager.update_progress("test_op", 0.5).await;
        let active = manager.get_active_loadings().await;
        assert_eq!(active[0].progress, Some(0.5));

        // Complete loading
        manager.complete_loading("test_op").await;
        let active = manager.get_active_loadings().await;
        assert_eq!(active.len(), 0);
        assert!(!manager.has_active_loadings().await);
    }

    #[tokio::test]
    async fn test_event_batch_dispatch() {
        let dispatcher = EventDispatcher::new();

        let events = vec![
            AppMessage::ModeChanged(crate::AppMode::Command),
            AppMessage::CommandPaletteToggled,
        ];

        let batch_id = dispatcher
            .dispatch_batch(events, BatchType::Sequential, EventPriority::Normal)
            .await
            .unwrap();

        assert!(!batch_id.is_empty());

        // Check stats
        let stats = dispatcher.get_stats().await;
        assert_eq!(stats.batched_events, 2);
    }

    #[tokio::test]
    async fn test_event_priorities() {
        // Test that priorities are ordered correctly
        assert!(EventPriority::Low < EventPriority::Normal);
        assert!(EventPriority::Normal < EventPriority::High);
        assert!(EventPriority::High < EventPriority::Critical);
    }

    #[tokio::test]
    async fn test_event_sources() {
        // Test event source variants
        assert_eq!(EventSource::UserInput, EventSource::UserInput);
        assert_eq!(EventSource::System, EventSource::System);
        assert_eq!(EventSource::Network, EventSource::Network);
        assert_eq!(EventSource::FileSystem, EventSource::FileSystem);
        assert_eq!(EventSource::Timer, EventSource::Timer);
    }

    #[tokio::test]
    async fn test_batch_types() {
        // Test batch type variants
        assert_eq!(BatchType::Atomic, BatchType::Atomic);
        assert_eq!(BatchType::BestEffort, BatchType::BestEffort);
        assert_eq!(BatchType::Sequential, BatchType::Sequential);
    }
}
