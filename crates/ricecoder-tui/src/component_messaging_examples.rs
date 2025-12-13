//! Example components demonstrating the component messaging system
//!
//! This module provides example implementations of components that use
//! the messaging system for inter-component communication.

use super::*;
use crate::widgets::*;
use ratatui::layout::Rect;
use ratatui::Frame;

/// Example chat component that sends messages to other components
pub struct MessagingChatWidget {
    base_widget: ChatWidget,
    component_id: ComponentId,
}

impl MessagingChatWidget {
    pub fn new() -> Self {
        Self {
            base_widget: ChatWidget::new(),
            component_id: "chat-widget".to_string(),
        }
    }
}

impl Component for MessagingChatWidget {
    fn id(&self) -> ComponentId {
        self.component_id.clone()
    }

    fn render(&self, frame: &mut Frame, area: Rect, model: &AppModel) {
        self.base_widget.render(frame, area, model);
    }

    fn update(&mut self, message: &AppMessage, model: &AppModel) -> bool {
        // Handle component-specific messages
        match message {
            AppMessage::ComponentMessage { target, .. } if target == &self.component_id => {
                // Handle message targeted at this component
                true
            }
            _ => self.base_widget.update(message, model),
        }
    }

    fn subscriptions(&self) -> Vec<messaging::ComponentSubscription> {
        vec![
            // Subscribe to mode changes
            messaging::ComponentSubscription {
                subscriber_id: self.component_id.clone(),
                filter: messaging::MessageFilter::MessageTypes(vec!["ModeChanged".to_string()]),
                priority: messaging::SubscriptionPriority::Normal,
            },
            // Subscribe to messages from status bar
            messaging::ComponentSubscription {
                subscriber_id: self.component_id.clone(),
                filter: messaging::MessageFilter::Sources(vec!["status-bar".to_string()]),
                priority: messaging::SubscriptionPriority::High,
            },
        ]
    }

    fn send_message(&self, bus: &messaging::ComponentMessageBus, message: AppMessage) {
        // Send message to status bar when chat is updated
        if let AppMessage::SendMessage(_) = message {
            let status_update = AppMessage::ComponentMessage {
                target: "status-bar".to_string(),
                payload: messaging::ComponentMessagePayload::String("Message sent".to_string()),
            };
            // Note: This would be called asynchronously in a real implementation
            // For now, it's just demonstrating the API
        }
    }

    fn receive_messages(&mut self, messages: &[AppMessage]) -> bool {
        let mut handled = false;
        for message in messages {
            match message {
                AppMessage::ModeChanged(mode) => {
                    // React to mode changes
                    handled = true;
                }
                AppMessage::ComponentMessage { payload: messaging::ComponentMessagePayload::String(text), .. } => {
                    // Handle string messages from other components
                    handled = true;
                }
                _ => {}
            }
        }
        handled
    }

    // Delegate other methods to base widget
    fn is_focused(&self) -> bool { self.base_widget.is_focused() }
    fn set_focused(&mut self, focused: bool) { self.base_widget.set_focused(focused); }
    fn is_visible(&self) -> bool { self.base_widget.is_visible() }
    fn set_visible(&mut self, visible: bool) { self.base_widget.set_visible(visible); }
    fn is_enabled(&self) -> bool { self.base_widget.is_enabled() }
    fn set_enabled(&mut self, enabled: bool) { self.base_widget.set_enabled(enabled); }
    fn bounds(&self) -> Rect { self.base_widget.bounds() }
    fn set_bounds(&mut self, bounds: Rect) { self.base_widget.set_bounds(bounds); }
    fn handle_focus(&mut self, direction: FocusDirection) -> FocusResult { self.base_widget.handle_focus(direction) }
    fn children(&self) -> Vec<&dyn Component> { self.base_widget.children() }
    fn children_mut(&mut self) -> Vec<&mut dyn Component> { self.base_widget.children_mut() }
    fn find_child(&self, id: &ComponentId) -> Option<&dyn Component> { self.base_widget.find_child(id) }
    fn find_child_mut(&mut self, id: &ComponentId) -> Option<&mut dyn Component> { self.base_widget.find_child_mut(id) }
    fn add_child(&mut self, child: Box<dyn Component>) { self.base_widget.add_child(child); }
    fn remove_child(&mut self, id: &ComponentId) -> Option<Box<dyn Component>> { self.base_widget.remove_child(id) }
    fn z_index(&self) -> i32 { self.base_widget.z_index() }
    fn set_z_index(&mut self, z_index: i32) { self.base_widget.set_z_index(z_index); }
    fn can_focus(&self) -> bool { self.base_widget.can_focus() }
    fn tab_order(&self) -> Option<usize> { self.base_widget.tab_order() }
    fn set_tab_order(&mut self, order: Option<usize>) { self.base_widget.set_tab_order(order); }
}

/// Example status bar component that receives messages from other components
pub struct MessagingStatusBarWidget {
    base_widget: StatusBarWidget,
    component_id: ComponentId,
    received_messages: Vec<String>,
}

impl MessagingStatusBarWidget {
    pub fn new() -> Self {
        Self {
            base_widget: StatusBarWidget::new(),
            component_id: "status-bar".to_string(),
            received_messages: Vec::new(),
        }
    }
}

impl Component for MessagingStatusBarWidget {
    fn id(&self) -> ComponentId {
        self.component_id.clone()
    }

    fn render(&self, frame: &mut Frame, area: Rect, model: &AppModel) {
        self.base_widget.render(frame, area, model);
    }

    fn update(&mut self, message: &AppMessage, model: &AppModel) -> bool {
        match message {
            AppMessage::ComponentMessage { target, .. } if target == &self.component_id => {
                // Handle message targeted at this component
                true
            }
            _ => self.base_widget.update(message, model),
        }
    }

    fn subscriptions(&self) -> Vec<messaging::ComponentSubscription> {
        vec![
            // Subscribe to all component messages
            messaging::ComponentSubscription {
                subscriber_id: self.component_id.clone(),
                filter: messaging::MessageFilter::MessageTypes(vec!["ComponentMessage".to_string()]),
                priority: messaging::SubscriptionPriority::Normal,
            },
            // Subscribe to token updates
            messaging::ComponentSubscription {
                subscriber_id: self.component_id.clone(),
                filter: messaging::MessageFilter::MessageTypes(vec!["TokensUpdated".to_string()]),
                priority: messaging::SubscriptionPriority::High,
            },
        ]
    }

    fn receive_messages(&mut self, messages: &[AppMessage]) -> bool {
        let mut handled = false;
        for message in messages {
            match message {
                AppMessage::ComponentMessage { payload: messaging::ComponentMessagePayload::String(text), .. } => {
                    self.received_messages.push(text.clone());
                    handled = true;
                }
                AppMessage::TokensUpdated(_) => {
                    // Update token display
                    handled = true;
                }
                _ => {}
            }
        }
        handled
    }

    // Delegate other methods to base widget
    fn is_focused(&self) -> bool { self.base_widget.is_focused() }
    fn set_focused(&mut self, focused: bool) { self.base_widget.set_focused(focused); }
    fn is_visible(&self) -> bool { self.base_widget.is_visible() }
    fn set_visible(&mut self, visible: bool) { self.base_widget.set_visible(visible); }
    fn is_enabled(&self) -> bool { self.base_widget.is_enabled() }
    fn set_enabled(&mut self, enabled: bool) { self.base_widget.set_enabled(enabled); }
    fn bounds(&self) -> Rect { self.base_widget.bounds() }
    fn set_bounds(&mut self, bounds: Rect) { self.base_widget.set_bounds(bounds); }
    fn handle_focus(&mut self, direction: FocusDirection) -> FocusResult { self.base_widget.handle_focus(direction) }
    fn children(&self) -> Vec<&dyn Component> { self.base_widget.children() }
    fn children_mut(&mut self) -> Vec<&mut dyn Component> { self.base_widget.children_mut() }
    fn find_child(&self, id: &ComponentId) -> Option<&dyn Component> { self.base_widget.find_child(id) }
    fn find_child_mut(&mut self, id: &ComponentId) -> Option<&mut dyn Component> { self.base_widget.find_child_mut(id) }
    fn add_child(&mut self, child: Box<dyn Component>) { self.base_widget.add_child(child); }
    fn remove_child(&mut self, id: &ComponentId) -> Option<Box<dyn Component>> { self.base_widget.remove_child(id) }
    fn z_index(&self) -> i32 { self.base_widget.z_index() }
    fn set_z_index(&mut self, z_index: i32) { self.base_widget.set_z_index(z_index); }
    fn can_focus(&self) -> bool { self.base_widget.can_focus() }
    fn tab_order(&self) -> Option<usize> { self.base_widget.tab_order() }
    fn set_tab_order(&mut self, order: Option<usize>) { self.base_widget.set_tab_order(order); }
}

/// Test the component messaging system
#[cfg(test)]
mod messaging_tests {
    use super::*;
    use tokio::runtime::Runtime;

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
                AppMessage::ComponentMessage { payload: messaging::ComponentMessagePayload::String(text), .. } => {
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