use ricecoder_tui::*;

/// Test the component messaging system
#[cfg(test)]
mod messaging_tests {
    use tokio::runtime::Runtime;

    use super::*;

    #[test]
    fn test_component_messaging() {
        let rt = Runtime::new().unwrap();

        rt.block_on(async {
            // Create message bus
            let bus = messaging::ComponentMessageBus::new();

            // Create components
            let chat = MessagingChatWidget::new();
            let status = MessagingStatusBarWidget::new();

            // Subscribe components
            for subscription in chat.subscriptions() {
                bus.subscribe(subscription).await;
            }
            for subscription in status.subscriptions() {
                bus.subscribe(subscription).await;
            }

            // Send a message from chat to status bar
            let message = AppMessage::ComponentMessage {
                target: "status-bar".to_string(),
                payload: messaging::ComponentMessagePayload::String("Test message".to_string()),
            };
            bus.send_message(message, "chat-widget".to_string()).await;

            // Check that status bar received the message
            let messages = bus.get_messages(&"status-bar".to_string()).await;
            assert_eq!(messages.len(), 1);

            match &messages[0] {
                AppMessage::ComponentMessage {
                    payload: messaging::ComponentMessagePayload::String(text),
                    ..
                } => {
                    assert_eq!(text, "Test message");
                }
                _ => panic!("Unexpected message type"),
            }
        });
    }

    #[test]
    fn test_message_filtering() {
        let filter = messaging::MessageFilter::MessageTypes(vec!["ModeChanged".to_string()]);

        // Test matching message
        let matching_message = AppMessage::ModeChanged(AppMode::Command);
        assert!(filter.matches(&matching_message, &"test-component".to_string()));

        // Test non-matching message
        let non_matching_message = AppMessage::KeyPress(crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Char('a'),
            modifiers: crossterm::event::KeyModifiers::empty(),
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        });
        assert!(!filter.matches(&non_matching_message, &"test-component".to_string()));
    }

    #[test]
    fn test_message_bus_stats() {
        let rt = Runtime::new().unwrap();

        rt.block_on(async {
            let bus = messaging::ComponentMessageBus::new();

            // Subscribe a component
            let subscription = messaging::ComponentSubscription {
                subscriber_id: "test-component".to_string(),
                filter: messaging::MessageFilter::All,
                priority: messaging::SubscriptionPriority::Normal,
            };
            bus.subscribe(subscription).await;

            // Send a message
            let message = AppMessage::ModeChanged(AppMode::Chat);
            bus.send_message(message, "sender".to_string()).await;

            // Check stats
            let stats = bus.get_stats().await;
            assert_eq!(stats.total_subscriptions, 1);
            assert_eq!(stats.active_subscribers, 1);
            assert_eq!(stats.queued_messages, 1);
        });
    }
}
