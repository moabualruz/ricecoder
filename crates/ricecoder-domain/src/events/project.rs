//! Project aggregate domain events
//!
//! REQ-DOMAIN-001.5: Project aggregate events

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{DomainEvent, EventMetadata};

/// Event emitted when a project is created
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectCreated {
    /// Event metadata
    pub metadata: EventMetadata,
    
    /// Project aggregate ID
    pub project_id: Uuid,
    
    /// Project name
    pub name: String,
    
    /// Optional project description
    pub description: Option<String>,
}

impl ProjectCreated {
    /// Create new ProjectCreated event
    pub fn new(project_id: Uuid, name: String, description: Option<String>) -> Self {
        Self {
            metadata: EventMetadata::new(),
            project_id,
            name,
            description,
        }
    }
}

impl DomainEvent for ProjectCreated {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn aggregate_id(&self) -> Uuid {
        self.project_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn event_type(&self) -> &str {
        "ProjectCreated"
    }
}

/// Event emitted when a project is updated
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectUpdated {
    /// Event metadata
    pub metadata: EventMetadata,
    
    /// Project aggregate ID
    pub project_id: Uuid,
    
    /// Updated name
    pub name: String,
    
    /// Updated description
    pub description: Option<String>,
}

impl ProjectUpdated {
    /// Create new ProjectUpdated event
    pub fn new(project_id: Uuid, name: String, description: Option<String>) -> Self {
        Self {
            metadata: EventMetadata::new(),
            project_id,
            name,
            description,
        }
    }
}

impl DomainEvent for ProjectUpdated {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn aggregate_id(&self) -> Uuid {
        self.project_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn event_type(&self) -> &str {
        "ProjectUpdated"
    }
}

/// Event emitted when a project is archived
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectArchived {
    /// Event metadata
    pub metadata: EventMetadata,
    
    /// Project aggregate ID
    pub project_id: Uuid,
}

impl ProjectArchived {
    /// Create new ProjectArchived event
    pub fn new(project_id: Uuid) -> Self {
        Self {
            metadata: EventMetadata::new(),
            project_id,
        }
    }
}

impl DomainEvent for ProjectArchived {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn aggregate_id(&self) -> Uuid {
        self.project_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn event_type(&self) -> &str {
        "ProjectArchived"
    }
}

/// Event emitted when a project is deleted
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectDeleted {
    /// Event metadata
    pub metadata: EventMetadata,
    
    /// Project aggregate ID
    pub project_id: Uuid,
}

impl ProjectDeleted {
    /// Create new ProjectDeleted event
    pub fn new(project_id: Uuid) -> Self {
        Self {
            metadata: EventMetadata::new(),
            project_id,
        }
    }
}

impl DomainEvent for ProjectDeleted {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn aggregate_id(&self) -> Uuid {
        self.project_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn event_type(&self) -> &str {
        "ProjectDeleted"
    }
}

/// Event emitted when a deleted project is restored
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectRestored {
    /// Event metadata
    pub metadata: EventMetadata,
    
    /// Project aggregate ID
    pub project_id: Uuid,
}

impl ProjectRestored {
    /// Create new ProjectRestored event
    pub fn new(project_id: Uuid) -> Self {
        Self {
            metadata: EventMetadata::new(),
            project_id,
        }
    }
}

impl DomainEvent for ProjectRestored {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn aggregate_id(&self) -> Uuid {
        self.project_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn event_type(&self) -> &str {
        "ProjectRestored"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_created_event() {
        let project_id = Uuid::new_v4();
        let event = ProjectCreated::new(
            project_id,
            "test-project".to_string(),
            Some("A test project".to_string()),
        );

        assert_eq!(event.aggregate_id(), project_id);
        assert_eq!(event.name, "test-project");
        assert_eq!(event.description, Some("A test project".to_string()));
        assert_eq!(event.event_type(), "ProjectCreated");
    }

    #[test]
    fn test_project_updated_event() {
        let project_id = Uuid::new_v4();
        let event = ProjectUpdated::new(
            project_id,
            "updated-project".to_string(),
            Some("Updated description".to_string()),
        );

        assert_eq!(event.aggregate_id(), project_id);
        assert_eq!(event.name, "updated-project");
        assert_eq!(event.event_type(), "ProjectUpdated");
    }

    #[test]
    fn test_project_archived_event() {
        let project_id = Uuid::new_v4();
        let event = ProjectArchived::new(project_id);

        assert_eq!(event.aggregate_id(), project_id);
        assert_eq!(event.event_type(), "ProjectArchived");
    }

    #[test]
    fn test_project_deleted_event() {
        let project_id = Uuid::new_v4();
        let event = ProjectDeleted::new(project_id);

        assert_eq!(event.aggregate_id(), project_id);
        assert_eq!(event.event_type(), "ProjectDeleted");
    }

    #[test]
    fn test_project_restored_event() {
        let project_id = Uuid::new_v4();
        let event = ProjectRestored::new(project_id);

        assert_eq!(event.aggregate_id(), project_id);
        assert_eq!(event.event_type(), "ProjectRestored");
    }

    #[test]
    fn test_event_serialization() {
        let project_id = Uuid::new_v4();
        let event = ProjectCreated::new(
            project_id,
            "test".to_string(),
            None,
        );

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ProjectCreated = serde_json::from_str(&json).unwrap();

        assert_eq!(event.project_id, deserialized.project_id);
        assert_eq!(event.name, deserialized.name);
    }
}
