//! Application layer events
//!
//! Application events represent use-case-level facts that are significant
//! to external consumers. They differ from domain events in that they
//! represent completed use cases rather than internal state changes.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Application-level event
///
/// These events represent completed use cases and are suitable
/// for external consumers (webhooks, audit logs, notifications).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApplicationEvent {
    // === Project Events ===
    
    /// Project was successfully created
    ProjectCreated {
        project_id: String,
        name: String,
        timestamp: DateTime<Utc>,
    },

    /// Project was updated
    ProjectUpdated {
        project_id: String,
        timestamp: DateTime<Utc>,
    },

    /// Project was archived
    ProjectArchived {
        project_id: String,
        timestamp: DateTime<Utc>,
    },

    /// Project was deleted
    ProjectDeleted {
        project_id: String,
        timestamp: DateTime<Utc>,
    },

    /// Project analysis completed
    ProjectAnalysisCompleted {
        project_id: String,
        timestamp: DateTime<Utc>,
    },

    // === Session Events ===
    
    /// Session was started
    SessionStarted {
        session_id: String,
        project_id: String,
        timestamp: DateTime<Utc>,
    },

    /// Session was completed
    SessionCompleted {
        session_id: String,
        message_count: usize,
        timestamp: DateTime<Utc>,
    },

    // === Specification Events ===
    
    /// Specification was created
    SpecificationCreated {
        specification_id: String,
        project_id: String,
        name: String,
        timestamp: DateTime<Utc>,
    },

    /// Specification was approved
    SpecificationApproved {
        specification_id: String,
        timestamp: DateTime<Utc>,
    },

    /// Specification was completed
    SpecificationCompleted {
        specification_id: String,
        completion_percentage: f32,
        timestamp: DateTime<Utc>,
    },
}

impl ApplicationEvent {
    /// Get the event type name
    pub fn event_type(&self) -> &'static str {
        match self {
            ApplicationEvent::ProjectCreated { .. } => "ProjectCreated",
            ApplicationEvent::ProjectUpdated { .. } => "ProjectUpdated",
            ApplicationEvent::ProjectArchived { .. } => "ProjectArchived",
            ApplicationEvent::ProjectDeleted { .. } => "ProjectDeleted",
            ApplicationEvent::ProjectAnalysisCompleted { .. } => "ProjectAnalysisCompleted",
            ApplicationEvent::SessionStarted { .. } => "SessionStarted",
            ApplicationEvent::SessionCompleted { .. } => "SessionCompleted",
            ApplicationEvent::SpecificationCreated { .. } => "SpecificationCreated",
            ApplicationEvent::SpecificationApproved { .. } => "SpecificationApproved",
            ApplicationEvent::SpecificationCompleted { .. } => "SpecificationCompleted",
        }
    }

    /// Get the event timestamp
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            ApplicationEvent::ProjectCreated { timestamp, .. } => *timestamp,
            ApplicationEvent::ProjectUpdated { timestamp, .. } => *timestamp,
            ApplicationEvent::ProjectArchived { timestamp, .. } => *timestamp,
            ApplicationEvent::ProjectDeleted { timestamp, .. } => *timestamp,
            ApplicationEvent::ProjectAnalysisCompleted { timestamp, .. } => *timestamp,
            ApplicationEvent::SessionStarted { timestamp, .. } => *timestamp,
            ApplicationEvent::SessionCompleted { timestamp, .. } => *timestamp,
            ApplicationEvent::SpecificationCreated { timestamp, .. } => *timestamp,
            ApplicationEvent::SpecificationApproved { timestamp, .. } => *timestamp,
            ApplicationEvent::SpecificationCompleted { timestamp, .. } => *timestamp,
        }
    }

    /// Generate a unique event ID
    pub fn generate_event_id() -> String {
        Uuid::new_v4().to_string()
    }
}

/// Event publisher port
///
/// Infrastructure Layer provides implementations for:
/// - In-process event handlers
/// - Message queues
/// - Webhooks
/// - Audit logging
#[async_trait]
pub trait EventPublisher: Send + Sync {
    /// Publish an application event
    async fn publish(&self, event: ApplicationEvent);

    /// Publish a domain event (forwarded from aggregates)
    async fn publish_domain_event(&self, event: Box<dyn ricecoder_domain::events::DomainEvent>);
}

/// No-op event publisher for testing
pub struct NoOpEventPublisher;

#[async_trait]
impl EventPublisher for NoOpEventPublisher {
    async fn publish(&self, _event: ApplicationEvent) {
        // No-op: events are discarded
    }

    async fn publish_domain_event(&self, _event: Box<dyn ricecoder_domain::events::DomainEvent>) {
        // No-op: events are discarded
    }
}

/// In-memory event collector for testing
#[derive(Default)]
pub struct InMemoryEventPublisher {
    events: std::sync::Mutex<Vec<ApplicationEvent>>,
}

impl InMemoryEventPublisher {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get all collected events
    pub fn events(&self) -> Vec<ApplicationEvent> {
        self.events.lock().unwrap().clone()
    }

    /// Clear collected events
    pub fn clear(&self) {
        self.events.lock().unwrap().clear();
    }
}

#[async_trait]
impl EventPublisher for InMemoryEventPublisher {
    async fn publish(&self, event: ApplicationEvent) {
        self.events.lock().unwrap().push(event);
    }

    async fn publish_domain_event(&self, _event: Box<dyn ricecoder_domain::events::DomainEvent>) {
        // Domain events are not stored in this simple implementation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type() {
        let event = ApplicationEvent::ProjectCreated {
            project_id: "123".into(),
            name: "test".into(),
            timestamp: Utc::now(),
        };
        assert_eq!(event.event_type(), "ProjectCreated");
    }

    #[tokio::test]
    async fn test_in_memory_publisher() {
        let publisher = InMemoryEventPublisher::new();
        
        publisher
            .publish(ApplicationEvent::ProjectCreated {
                project_id: "123".into(),
                name: "test".into(),
                timestamp: Utc::now(),
            })
            .await;

        let events = publisher.events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type(), "ProjectCreated");
    }
}
