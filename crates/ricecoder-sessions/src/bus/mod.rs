//! Event bus for session lifecycle and message events
//!
//! This module provides a type-safe, async event bus for publishing and subscribing
//! to session-related events. It uses tokio's broadcast channel for efficient
//! pub/sub messaging.
//!
//! # Example
//!
//! ```rust
//! use ricecoder_sessions::bus::{EventBus, BusEvent, SessionEvent};
//!
//! # tokio_test::block_on(async {
//! let bus = EventBus::new();
//! let mut subscriber = bus.subscribe();
//!
//! // Publish event
//! bus.publish(BusEvent::Session(SessionEvent::Created {
//!     session_id: "test-123".to_string(),
//! }));
//!
//! // Receive event
//! let event = subscriber.recv().await.unwrap();
//! match event {
//!     BusEvent::Session(SessionEvent::Created { session_id }) => {
//!         println!("Session created: {}", session_id);
//!     }
//!     _ => {}
//! }
//! # });
//! ```

pub mod events;

pub use events::{MessageEvent, SessionEvent, ToolEvent};

use tokio::sync::broadcast;

/// Channel capacity for broadcast events
const CHANNEL_CAPACITY: usize = 1024;

/// Unified event type for all bus events
#[derive(Debug, Clone)]
pub enum BusEvent {
    /// Session lifecycle event
    Session(SessionEvent),
    /// Message update event
    Message(MessageEvent),
    /// Tool execution event
    Tool(ToolEvent),
}

/// Event bus for session lifecycle and message events
///
/// The event bus provides a centralized pub/sub mechanism for session-related events.
/// Components can subscribe to receive all events and filter as needed.
///
/// # Thread Safety
///
/// The event bus is thread-safe and can be cloned to share across threads.
/// Each clone shares the same underlying broadcast channel.
#[derive(Clone, Debug)]
pub struct EventBus {
    sender: broadcast::Sender<BusEvent>,
}

impl EventBus {
    /// Create a new event bus with default capacity (1024 events)
    ///
    /// # Example
    ///
    /// ```rust
    /// use ricecoder_sessions::bus::EventBus;
    ///
    /// let bus = EventBus::new();
    /// ```
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(CHANNEL_CAPACITY);
        Self { sender }
    }

    /// Create a new event bus with custom capacity
    ///
    /// # Arguments
    ///
    /// * `capacity` - Maximum number of events to buffer
    ///
    /// # Example
    ///
    /// ```rust
    /// use ricecoder_sessions::bus::EventBus;
    ///
    /// let bus = EventBus::with_capacity(2048);
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Publish an event to all subscribers
    ///
    /// This is a non-blocking operation. If the channel is full, the oldest
    /// event will be dropped.
    ///
    /// # Arguments
    ///
    /// * `event` - Event to publish
    ///
    /// # Example
    ///
    /// ```rust
    /// use ricecoder_sessions::bus::{EventBus, BusEvent, SessionEvent};
    ///
    /// let bus = EventBus::new();
    /// bus.publish(BusEvent::Session(SessionEvent::Created {
    ///     session_id: "test-123".to_string(),
    /// }));
    /// ```
    pub fn publish(&self, event: BusEvent) {
        // Ignore errors - it's ok if there are no subscribers
        let _ = self.sender.send(event);
    }

    /// Subscribe to receive events
    ///
    /// Returns a receiver that will receive all future events published to the bus.
    /// The receiver will not receive events published before subscription.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ricecoder_sessions::bus::EventBus;
    ///
    /// # tokio_test::block_on(async {
    /// let bus = EventBus::new();
    /// let mut subscriber = bus.subscribe();
    ///
    /// // Receive events
    /// while let Ok(event) = subscriber.recv().await {
    ///     println!("Received: {:?}", event);
    /// }
    /// # });
    /// ```
    pub fn subscribe(&self) -> broadcast::Receiver<BusEvent> {
        self.sender.subscribe()
    }

    /// Get the number of active subscribers
    ///
    /// # Example
    ///
    /// ```rust
    /// use ricecoder_sessions::bus::EventBus;
    ///
    /// let bus = EventBus::new();
    /// assert_eq!(bus.subscriber_count(), 0);
    ///
    /// let _sub1 = bus.subscribe();
    /// let _sub2 = bus.subscribe();
    /// assert_eq!(bus.subscriber_count(), 2);
    /// ```
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_bus_publish_subscribe() {
        let bus = EventBus::new();
        let mut subscriber = bus.subscribe();

        // Publish event
        bus.publish(BusEvent::Session(SessionEvent::Created {
            session_id: "test-123".to_string(),
        }));

        // Receive event
        let event = subscriber.recv().await.unwrap();
        match event {
            BusEvent::Session(SessionEvent::Created { session_id }) => {
                assert_eq!(session_id, "test-123");
            }
            _ => panic!("unexpected event"),
        }
    }

    #[tokio::test]
    async fn test_event_bus_multiple_subscribers() {
        let bus = EventBus::new();
        let mut sub1 = bus.subscribe();
        let mut sub2 = bus.subscribe();

        assert_eq!(bus.subscriber_count(), 2);

        // Publish event
        bus.publish(BusEvent::Session(SessionEvent::Updated {
            session_id: "test-456".to_string(),
        }));

        // Both subscribers receive event
        let event1 = sub1.recv().await.unwrap();
        let event2 = sub2.recv().await.unwrap();

        match (event1, event2) {
            (
                BusEvent::Session(SessionEvent::Updated { session_id: id1 }),
                BusEvent::Session(SessionEvent::Updated { session_id: id2 }),
            ) => {
                assert_eq!(id1, "test-456");
                assert_eq!(id2, "test-456");
            }
            _ => panic!("unexpected events"),
        }
    }

    #[tokio::test]
    async fn test_event_bus_message_events() {
        let bus = EventBus::new();
        let mut subscriber = bus.subscribe();

        bus.publish(BusEvent::Message(MessageEvent::PartUpdated {
            message_id: "msg-1".to_string(),
            part_index: 0,
            delta: Some("hello".to_string()),
        }));

        let event = subscriber.recv().await.unwrap();
        match event {
            BusEvent::Message(MessageEvent::PartUpdated {
                message_id,
                part_index,
                delta,
            }) => {
                assert_eq!(message_id, "msg-1");
                assert_eq!(part_index, 0);
                assert_eq!(delta, Some("hello".to_string()));
            }
            _ => panic!("unexpected event"),
        }
    }

    #[tokio::test]
    async fn test_event_bus_tool_events() {
        let bus = EventBus::new();
        let mut subscriber = bus.subscribe();

        bus.publish(BusEvent::Tool(ToolEvent::Completed {
            call_id: "call-1".to_string(),
            tool_name: "grep".to_string(),
        }));

        let event = subscriber.recv().await.unwrap();
        match event {
            BusEvent::Tool(ToolEvent::Completed {
                call_id,
                tool_name,
            }) => {
                assert_eq!(call_id, "call-1");
                assert_eq!(tool_name, "grep");
            }
            _ => panic!("unexpected event"),
        }
    }

    #[tokio::test]
    async fn test_event_bus_custom_capacity() {
        let bus = EventBus::with_capacity(10);
        assert_eq!(bus.subscriber_count(), 0);

        let _sub = bus.subscribe();
        assert_eq!(bus.subscriber_count(), 1);
    }

    #[test]
    fn test_event_bus_no_subscribers() {
        let bus = EventBus::new();

        // Should not panic when publishing with no subscribers
        bus.publish(BusEvent::Session(SessionEvent::Idle {
            session_id: "test".to_string(),
        }));

        assert_eq!(bus.subscriber_count(), 0);
    }

    #[tokio::test]
    async fn test_event_bus_clone() {
        let bus1 = EventBus::new();
        let bus2 = bus1.clone();

        let mut sub1 = bus1.subscribe();
        let mut sub2 = bus2.subscribe();

        // Publish from cloned bus
        bus2.publish(BusEvent::Session(SessionEvent::Created {
            session_id: "clone-test".to_string(),
        }));

        // Both subscribers receive event
        let event1 = sub1.recv().await.unwrap();
        let event2 = sub2.recv().await.unwrap();

        match (event1, event2) {
            (
                BusEvent::Session(SessionEvent::Created { session_id: id1 }),
                BusEvent::Session(SessionEvent::Created { session_id: id2 }),
            ) => {
                assert_eq!(id1, "clone-test");
                assert_eq!(id2, "clone-test");
            }
            _ => panic!("unexpected events"),
        }
    }
}
