//! Domain events infrastructure
//!
//! Domain events for state change tracking
//! Aggregate-specific events

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod project;
pub mod session;
pub mod specification;

/// Base trait for all domain events
///
/// Domain events represent significant occurrences in the domain.
/// They are immutable records of state changes that have already happened.
///
/// # Requirements
///  All aggregates emit domain events on state changes
///
/// # Example
/// ```
/// use ricecoder_domain::events::DomainEvent;
/// use uuid::Uuid;
/// use chrono::{DateTime, Utc};
///
/// struct MyEvent {
///     event_id: Uuid,
///     aggregate_id: Uuid,
///     occurred_at: DateTime<Utc>,
/// }
///
/// impl DomainEvent for MyEvent {
///     fn event_id(&self) -> Uuid { self.event_id }
///     fn aggregate_id(&self) -> Uuid { self.aggregate_id }
///     fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
///     fn event_type(&self) -> &str { "MyEvent" }
/// }
/// ```
pub trait DomainEvent: Send + Sync {
    /// Unique identifier for this event
    fn event_id(&self) -> Uuid;

    /// ID of the aggregate that emitted this event
    fn aggregate_id(&self) -> Uuid;

    /// Timestamp when this event occurred
    fn occurred_at(&self) -> DateTime<Utc>;

    /// Type discriminator for serialization/deserialization
    fn event_type(&self) -> &str;
}

/// Metadata for domain events
///
/// Contains common metadata fields used by all events.
/// Supports event correlation and causation tracking for distributed systems.
///
/// # Fields
/// - `event_id`: Unique identifier for this event
/// - `occurred_at`: Timestamp when event occurred
/// - `causation_id`: ID of the command/event that caused this event (optional)
/// - `correlation_id`: ID to correlate related events across aggregates (optional)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventMetadata {
    /// Unique event identifier
    pub event_id: Uuid,

    /// When the event occurred
    pub occurred_at: DateTime<Utc>,

    /// ID of the command/event that caused this event (for event sourcing)
    pub causation_id: Option<Uuid>,

    /// ID to correlate events across aggregates (for distributed tracing)
    pub correlation_id: Option<Uuid>,
}

impl EventMetadata {
    /// Create new event metadata with generated ID and current timestamp
    pub fn new() -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            causation_id: None,
            correlation_id: None,
        }
    }

    /// Create event metadata with causation tracking
    pub fn with_causation(causation_id: Uuid) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            causation_id: Some(causation_id),
            correlation_id: None,
        }
    }

    /// Create event metadata with full correlation tracking
    pub fn with_correlation(causation_id: Uuid, correlation_id: Uuid) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            causation_id: Some(causation_id),
            correlation_id: Some(correlation_id),
        }
    }
}

impl Default for EventMetadata {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_metadata_new() {
        let metadata = EventMetadata::new();
        assert!(!metadata.event_id.is_nil());
        assert!(metadata.causation_id.is_none());
        assert!(metadata.correlation_id.is_none());
    }

    #[test]
    fn test_event_metadata_with_causation() {
        let causation_id = Uuid::new_v4();
        let metadata = EventMetadata::with_causation(causation_id);
        
        assert!(!metadata.event_id.is_nil());
        assert_eq!(metadata.causation_id, Some(causation_id));
        assert!(metadata.correlation_id.is_none());
    }

    #[test]
    fn test_event_metadata_with_correlation() {
        let causation_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let metadata = EventMetadata::with_correlation(causation_id, correlation_id);
        
        assert!(!metadata.event_id.is_nil());
        assert_eq!(metadata.causation_id, Some(causation_id));
        assert_eq!(metadata.correlation_id, Some(correlation_id));
    }

    #[test]
    fn test_event_metadata_serialization() {
        let metadata = EventMetadata::new();
        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: EventMetadata = serde_json::from_str(&json).unwrap();
        
        assert_eq!(metadata.event_id, deserialized.event_id);
        assert_eq!(metadata.occurred_at, deserialized.occurred_at);
    }
}
