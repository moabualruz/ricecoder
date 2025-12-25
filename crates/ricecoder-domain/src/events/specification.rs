//! Specification aggregate domain events
//!
//! Specification aggregate events

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{DomainEvent, EventMetadata};

/// Event emitted when a specification is created
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpecificationCreated {
    /// Event metadata
    pub metadata: EventMetadata,
    
    /// Specification aggregate ID
    pub specification_id: Uuid,
    
    /// Associated project ID
    pub project_id: Uuid,
    
    /// Specification version
    pub version: String,
}

impl SpecificationCreated {
    /// Create new SpecificationCreated event
    pub fn new(specification_id: Uuid, project_id: Uuid, version: String) -> Self {
        Self {
            metadata: EventMetadata::new(),
            specification_id,
            project_id,
            version,
        }
    }
}

impl DomainEvent for SpecificationCreated {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn aggregate_id(&self) -> Uuid {
        self.specification_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn event_type(&self) -> &str {
        "SpecificationCreated"
    }
}

/// Event emitted when a requirement is added
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RequirementAdded {
    /// Event metadata
    pub metadata: EventMetadata,
    
    /// Specification aggregate ID
    pub specification_id: Uuid,
    
    /// Requirement ID
    pub requirement_id: Uuid,
    
    /// Requirement title
    pub title: String,
}

impl RequirementAdded {
    /// Create new RequirementAdded event
    pub fn new(specification_id: Uuid, requirement_id: Uuid, title: String) -> Self {
        Self {
            metadata: EventMetadata::new(),
            specification_id,
            requirement_id,
            title,
        }
    }
}

impl DomainEvent for RequirementAdded {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn aggregate_id(&self) -> Uuid {
        self.specification_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn event_type(&self) -> &str {
        "RequirementAdded"
    }
}

/// Event emitted when a requirement is updated
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RequirementUpdated {
    /// Event metadata
    pub metadata: EventMetadata,
    
    /// Specification aggregate ID
    pub specification_id: Uuid,
    
    /// Requirement ID
    pub requirement_id: Uuid,
}

impl RequirementUpdated {
    /// Create new RequirementUpdated event
    pub fn new(specification_id: Uuid, requirement_id: Uuid) -> Self {
        Self {
            metadata: EventMetadata::new(),
            specification_id,
            requirement_id,
        }
    }
}

impl DomainEvent for RequirementUpdated {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn aggregate_id(&self) -> Uuid {
        self.specification_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn event_type(&self) -> &str {
        "RequirementUpdated"
    }
}

/// Event emitted when a requirement is approved
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RequirementApproved {
    /// Event metadata
    pub metadata: EventMetadata,
    
    /// Specification aggregate ID
    pub specification_id: Uuid,
    
    /// Requirement ID
    pub requirement_id: Uuid,
}

impl RequirementApproved {
    /// Create new RequirementApproved event
    pub fn new(specification_id: Uuid, requirement_id: Uuid) -> Self {
        Self {
            metadata: EventMetadata::new(),
            specification_id,
            requirement_id,
        }
    }
}

impl DomainEvent for RequirementApproved {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn aggregate_id(&self) -> Uuid {
        self.specification_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn event_type(&self) -> &str {
        "RequirementApproved"
    }
}

/// Event emitted when a task is added
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskAdded {
    /// Event metadata
    pub metadata: EventMetadata,
    
    /// Specification aggregate ID
    pub specification_id: Uuid,
    
    /// Task ID
    pub task_id: Uuid,
    
    /// Task title
    pub title: String,
}

impl TaskAdded {
    /// Create new TaskAdded event
    pub fn new(specification_id: Uuid, task_id: Uuid, title: String) -> Self {
        Self {
            metadata: EventMetadata::new(),
            specification_id,
            task_id,
            title,
        }
    }
}

impl DomainEvent for TaskAdded {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn aggregate_id(&self) -> Uuid {
        self.specification_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn event_type(&self) -> &str {
        "TaskAdded"
    }
}

/// Event emitted when a task is started
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskStarted {
    /// Event metadata
    pub metadata: EventMetadata,
    
    /// Specification aggregate ID
    pub specification_id: Uuid,
    
    /// Task ID
    pub task_id: Uuid,
}

impl TaskStarted {
    /// Create new TaskStarted event
    pub fn new(specification_id: Uuid, task_id: Uuid) -> Self {
        Self {
            metadata: EventMetadata::new(),
            specification_id,
            task_id,
        }
    }
}

impl DomainEvent for TaskStarted {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn aggregate_id(&self) -> Uuid {
        self.specification_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn event_type(&self) -> &str {
        "TaskStarted"
    }
}

/// Event emitted when a task is completed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskCompleted {
    /// Event metadata
    pub metadata: EventMetadata,
    
    /// Specification aggregate ID
    pub specification_id: Uuid,
    
    /// Task ID
    pub task_id: Uuid,
}

impl TaskCompleted {
    /// Create new TaskCompleted event
    pub fn new(specification_id: Uuid, task_id: Uuid) -> Self {
        Self {
            metadata: EventMetadata::new(),
            specification_id,
            task_id,
        }
    }
}

impl DomainEvent for TaskCompleted {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn aggregate_id(&self) -> Uuid {
        self.specification_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn event_type(&self) -> &str {
        "TaskCompleted"
    }
}

/// Event emitted when a specification is approved
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpecificationApproved {
    /// Event metadata
    pub metadata: EventMetadata,
    
    /// Specification aggregate ID
    pub specification_id: Uuid,
}

impl SpecificationApproved {
    /// Create new SpecificationApproved event
    pub fn new(specification_id: Uuid) -> Self {
        Self {
            metadata: EventMetadata::new(),
            specification_id,
        }
    }
}

impl DomainEvent for SpecificationApproved {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn aggregate_id(&self) -> Uuid {
        self.specification_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn event_type(&self) -> &str {
        "SpecificationApproved"
    }
}

/// Event emitted when a specification is implemented
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpecificationImplemented {
    /// Event metadata
    pub metadata: EventMetadata,
    
    /// Specification aggregate ID
    pub specification_id: Uuid,
}

impl SpecificationImplemented {
    /// Create new SpecificationImplemented event
    pub fn new(specification_id: Uuid) -> Self {
        Self {
            metadata: EventMetadata::new(),
            specification_id,
        }
    }
}

impl DomainEvent for SpecificationImplemented {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn aggregate_id(&self) -> Uuid {
        self.specification_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn event_type(&self) -> &str {
        "SpecificationImplemented"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specification_created_event() {
        let spec_id = Uuid::new_v4();
        let project_id = Uuid::new_v4();
        let event = SpecificationCreated::new(
            spec_id,
            project_id,
            "1.0.0".to_string(),
        );

        assert_eq!(event.aggregate_id(), spec_id);
        assert_eq!(event.project_id, project_id);
        assert_eq!(event.version, "1.0.0");
        assert_eq!(event.event_type(), "SpecificationCreated");
    }

    #[test]
    fn test_requirement_added_event() {
        let spec_id = Uuid::new_v4();
        let req_id = Uuid::new_v4();
        let event = RequirementAdded::new(
            spec_id,
            req_id,
            "REQ-001: User Authentication".to_string(),
        );

        assert_eq!(event.aggregate_id(), spec_id);
        assert_eq!(event.requirement_id, req_id);
        assert_eq!(event.title, "REQ-001: User Authentication");
        assert_eq!(event.event_type(), "RequirementAdded");
    }

    #[test]
    fn test_requirement_updated_event() {
        let spec_id = Uuid::new_v4();
        let req_id = Uuid::new_v4();
        let event = RequirementUpdated::new(spec_id, req_id);

        assert_eq!(event.aggregate_id(), spec_id);
        assert_eq!(event.requirement_id, req_id);
        assert_eq!(event.event_type(), "RequirementUpdated");
    }

    #[test]
    fn test_requirement_approved_event() {
        let spec_id = Uuid::new_v4();
        let req_id = Uuid::new_v4();
        let event = RequirementApproved::new(spec_id, req_id);

        assert_eq!(event.aggregate_id(), spec_id);
        assert_eq!(event.requirement_id, req_id);
        assert_eq!(event.event_type(), "RequirementApproved");
    }

    #[test]
    fn test_task_added_event() {
        let spec_id = Uuid::new_v4();
        let task_id = Uuid::new_v4();
        let event = TaskAdded::new(
            spec_id,
            task_id,
            "TASK-001: Implement login".to_string(),
        );

        assert_eq!(event.aggregate_id(), spec_id);
        assert_eq!(event.task_id, task_id);
        assert_eq!(event.title, "TASK-001: Implement login");
        assert_eq!(event.event_type(), "TaskAdded");
    }

    #[test]
    fn test_task_started_event() {
        let spec_id = Uuid::new_v4();
        let task_id = Uuid::new_v4();
        let event = TaskStarted::new(spec_id, task_id);

        assert_eq!(event.aggregate_id(), spec_id);
        assert_eq!(event.task_id, task_id);
        assert_eq!(event.event_type(), "TaskStarted");
    }

    #[test]
    fn test_task_completed_event() {
        let spec_id = Uuid::new_v4();
        let task_id = Uuid::new_v4();
        let event = TaskCompleted::new(spec_id, task_id);

        assert_eq!(event.aggregate_id(), spec_id);
        assert_eq!(event.task_id, task_id);
        assert_eq!(event.event_type(), "TaskCompleted");
    }

    #[test]
    fn test_specification_approved_event() {
        let spec_id = Uuid::new_v4();
        let event = SpecificationApproved::new(spec_id);

        assert_eq!(event.aggregate_id(), spec_id);
        assert_eq!(event.event_type(), "SpecificationApproved");
    }

    #[test]
    fn test_specification_implemented_event() {
        let spec_id = Uuid::new_v4();
        let event = SpecificationImplemented::new(spec_id);

        assert_eq!(event.aggregate_id(), spec_id);
        assert_eq!(event.event_type(), "SpecificationImplemented");
    }

    #[test]
    fn test_event_serialization() {
        let spec_id = Uuid::new_v4();
        let project_id = Uuid::new_v4();
        let event = SpecificationCreated::new(
            spec_id,
            project_id,
            "2.0.0".to_string(),
        );

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: SpecificationCreated = serde_json::from_str(&json).unwrap();

        assert_eq!(event.specification_id, deserialized.specification_id);
        assert_eq!(event.project_id, deserialized.project_id);
        assert_eq!(event.version, deserialized.version);
    }
}
