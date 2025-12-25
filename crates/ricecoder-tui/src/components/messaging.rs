//! Component messaging system for inter-component communication

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use crate::model::AppMessage;
use super::ComponentId;

/// Subscription to component messages
#[derive(Debug, Clone)]
pub struct ComponentSubscription {
    pub subscriber_id: ComponentId,
    pub message_types: Vec<String>,
    pub priority: i32,
}

impl ComponentSubscription {
    pub fn new(subscriber_id: ComponentId) -> Self {
        Self {
            subscriber_id,
            message_types: Vec::new(),
            priority: 0,
        }
    }

    pub fn with_message_types(mut self, types: Vec<String>) -> Self {
        self.message_types = types;
        self
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
}

/// Message envelope for component communication
#[derive(Debug, Clone)]
pub struct ComponentMessage {
    pub source: ComponentId,
    pub target: Option<ComponentId>,
    pub message: AppMessage,
    pub timestamp: std::time::Instant,
}

impl ComponentMessage {
    pub fn new(source: ComponentId, message: AppMessage) -> Self {
        Self {
            source,
            target: None,
            message,
            timestamp: std::time::Instant::now(),
        }
    }

    pub fn with_target(mut self, target: ComponentId) -> Self {
        self.target = Some(target);
        self
    }
}

/// Message bus for component communication
pub struct ComponentMessageBus {
    subscriptions: Arc<RwLock<HashMap<ComponentId, ComponentSubscription>>>,
    message_tx: mpsc::UnboundedSender<ComponentMessage>,
    message_rx: Arc<RwLock<mpsc::UnboundedReceiver<ComponentMessage>>>,
}

impl ComponentMessageBus {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            message_tx: tx,
            message_rx: Arc::new(RwLock::new(rx)),
        }
    }

    /// Subscribe a component to receive messages
    pub async fn subscribe(&self, subscription: ComponentSubscription) {
        let mut subs = self.subscriptions.write().await;
        subs.insert(subscription.subscriber_id.clone(), subscription);
    }

    /// Unsubscribe a component
    pub async fn unsubscribe(&self, id: &ComponentId) {
        let mut subs = self.subscriptions.write().await;
        subs.remove(id);
    }

    /// Send a message to a specific component
    pub fn send(&self, message: ComponentMessage) -> Result<(), String> {
        self.message_tx
            .send(message)
            .map_err(|e| format!("Failed to send message: {}", e))
    }

    /// Broadcast a message to all subscribers
    pub fn broadcast(&self, source: ComponentId, message: AppMessage) -> Result<(), String> {
        let msg = ComponentMessage::new(source, message);
        self.send(msg)
    }

    /// Get pending messages (non-blocking)
    pub async fn try_receive(&self) -> Option<ComponentMessage> {
        let mut rx = self.message_rx.write().await;
        rx.try_recv().ok()
    }

    /// Get all subscriptions
    pub async fn subscriptions(&self) -> Vec<ComponentSubscription> {
        let subs = self.subscriptions.read().await;
        subs.values().cloned().collect()
    }
}

impl Default for ComponentMessageBus {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ComponentMessageBus {
    fn clone(&self) -> Self {
        // Create a new bus that shares subscriptions but has separate channels
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            subscriptions: Arc::clone(&self.subscriptions),
            message_tx: tx,
            message_rx: Arc::new(RwLock::new(rx)),
        }
    }
}
