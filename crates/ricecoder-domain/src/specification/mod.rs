//! Specification Aggregate Root
//!
//! REQ-DOMAIN-003: Specification aggregate with full DDD compliance
//! - Aggregate root with encapsulated Requirement and Task entities
//! - Invariant enforcement (requirements have AC, tasks trace to requirements)
//! - Domain event emission for all state changes
//! - Immutable identity
//! - Completion percentage calculation

use crate::errors::{DomainError, DomainResult};
use crate::events::specification::*;
use crate::events::DomainEvent;
use crate::value_objects::{ProjectId, RequirementId, SpecificationId, TaskId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ============================================================================
// Specification Aggregate Root
// ============================================================================

/// Specification Aggregate Root
///
/// Manages feature specifications with full traceability from requirements to tasks.
/// All operations emit domain events for auditability and event sourcing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Specification {
    /// Immutable identity
    id: SpecificationId,

    /// Parent project reference
    project_id: ProjectId,

    /// Specification name (1-100 chars)
    name: String,

    /// Description of the specification
    description: String,

    /// Version string (e.g., "1.0.0")
    version: String,

    /// Encapsulated requirements
    requirements: Vec<Requirement>,

    /// Encapsulated tasks
    tasks: Vec<Task>,

    /// Specification lifecycle status
    status: SpecStatus,

    /// Creation timestamp (immutable)
    created_at: DateTime<Utc>,

    /// Last update timestamp
    updated_at: DateTime<Utc>,
}

/// Specification lifecycle states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpecStatus {
    /// Initial draft state
    Draft,
    /// Requirements have been defined
    RequirementsComplete,
    /// Design has been documented
    DesignComplete,
    /// Tasks have been planned
    TasksPlanned,
    /// Implementation in progress
    InProgress,
    /// All tasks completed
    Complete,
    /// Specification archived
    Archived,
}

impl Specification {
    // ========================================================================
    // Factory Methods
    // ========================================================================

    /// Create a new specification
    ///
    /// REQ-DOMAIN-003.1: Aggregate root creation with event emission
    pub fn create(
        project_id: ProjectId,
        name: String,
        description: String,
        version: String,
    ) -> DomainResult<(Self, Box<dyn DomainEvent>)> {
        // Validate name (1-100 chars)
        if name.is_empty() {
            return Err(DomainError::ValidationError {
                field: "name".to_string(),
                reason: "Specification name cannot be empty".to_string(),
            });
        }
        if name.len() > 100 {
            return Err(DomainError::ValidationError {
                field: "name".to_string(),
                reason: "Specification name must be 100 characters or less".to_string(),
            });
        }

        let id = SpecificationId::new();
        let now = Utc::now();

        let spec = Self {
            id,
            project_id,
            name: name.clone(),
            description,
            version: version.clone(),
            requirements: Vec::new(),
            tasks: Vec::new(),
            status: SpecStatus::Draft,
            created_at: now,
            updated_at: now,
        };

        let event = SpecificationCreated::new(id.as_uuid(), project_id.as_uuid(), version);

        Ok((spec, Box::new(event)))
    }

    // ========================================================================
    // Requirement Operations
    // ========================================================================

    /// Add a requirement with validation
    ///
    /// REQ-DOMAIN-003.3: All requirements must have acceptance criteria
    /// REQ-DOMAIN-003.5: Validate completeness when adding requirements
    pub fn add_requirement(
        &mut self,
        title: String,
        description: String,
        acceptance_criteria: Vec<String>,
    ) -> DomainResult<(RequirementId, Box<dyn DomainEvent>)> {
        // Business rule: Requirements can only be added in Draft or RequirementsComplete
        if self.status != SpecStatus::Draft && self.status != SpecStatus::RequirementsComplete {
            return Err(DomainError::BusinessRuleViolation {
                rule: format!(
                    "Cannot add requirements in {:?} state",
                    self.status
                ),
            });
        }

        // Business rule: All requirements must have at least one acceptance criterion
        if acceptance_criteria.is_empty() {
            return Err(DomainError::BusinessRuleViolation {
                rule: "Requirement must have at least one acceptance criterion".to_string(),
            });
        }

        // Validate title
        if title.is_empty() {
            return Err(DomainError::ValidationError {
                field: "title".to_string(),
                reason: "Requirement title cannot be empty".to_string(),
            });
        }

        let requirement = Requirement::new(title.clone(), description, acceptance_criteria);
        let req_id = requirement.id;
        self.requirements.push(requirement);
        self.updated_at = Utc::now();

        let event = RequirementAdded::new(self.id.as_uuid(), req_id.as_uuid(), title);

        Ok((req_id, Box::new(event)))
    }

    /// Update an existing requirement
    pub fn update_requirement(
        &mut self,
        requirement_id: RequirementId,
        title: Option<String>,
        description: Option<String>,
        acceptance_criteria: Option<Vec<String>>,
    ) -> DomainResult<Box<dyn DomainEvent>> {
        // Find the requirement
        let req = self
            .requirements
            .iter_mut()
            .find(|r| r.id == requirement_id)
            .ok_or_else(|| DomainError::EntityNotFound {
                entity_type: "Requirement".to_string(),
                id: requirement_id.to_string(),
            })?;

        // Update fields if provided
        if let Some(t) = title {
            if t.is_empty() {
                return Err(DomainError::ValidationError {
                    field: "title".to_string(),
                    reason: "Requirement title cannot be empty".to_string(),
                });
            }
            req.title = t;
        }
        if let Some(d) = description {
            req.description = d;
        }
        if let Some(ac) = acceptance_criteria {
            if ac.is_empty() {
                return Err(DomainError::BusinessRuleViolation {
                    rule: "Requirement must have at least one acceptance criterion".to_string(),
                });
            }
            req.acceptance_criteria = ac;
        }

        req.updated_at = Utc::now();
        self.updated_at = Utc::now();

        let event = RequirementUpdated::new(self.id.as_uuid(), requirement_id.as_uuid());

        Ok(Box::new(event))
    }

    /// Approve a requirement
    pub fn approve_requirement(
        &mut self,
        requirement_id: RequirementId,
    ) -> DomainResult<Box<dyn DomainEvent>> {
        let req = self
            .requirements
            .iter_mut()
            .find(|r| r.id == requirement_id)
            .ok_or_else(|| DomainError::EntityNotFound {
                entity_type: "Requirement".to_string(),
                id: requirement_id.to_string(),
            })?;

        req.approved = true;
        req.updated_at = Utc::now();
        self.updated_at = Utc::now();

        let event = RequirementApproved::new(self.id.as_uuid(), requirement_id.as_uuid());

        Ok(Box::new(event))
    }

    // ========================================================================
    // Task Operations
    // ========================================================================

    /// Add a task with traceability validation
    ///
    /// REQ-DOMAIN-003.4: All tasks must trace to at least one requirement
    /// REQ-DOMAIN-003.8: Prevent task addition without requirement references
    /// REQ-DOMAIN-003.9: Validate all requirement references exist
    pub fn add_task(
        &mut self,
        title: String,
        description: String,
        requirement_refs: Vec<RequirementId>,
    ) -> DomainResult<(TaskId, Box<dyn DomainEvent>)> {
        // Business rule: Tasks must trace to at least one requirement
        if requirement_refs.is_empty() {
            return Err(DomainError::BusinessRuleViolation {
                rule: "Task must reference at least one requirement".to_string(),
            });
        }

        // Business rule: All referenced requirements must exist
        for req_ref in &requirement_refs {
            if !self.requirements.iter().any(|r| &r.id == req_ref) {
                return Err(DomainError::BusinessRuleViolation {
                    rule: format!("Task references non-existent requirement: {}", req_ref),
                });
            }
        }

        // Validate title
        if title.is_empty() {
            return Err(DomainError::ValidationError {
                field: "title".to_string(),
                reason: "Task title cannot be empty".to_string(),
            });
        }

        let task = Task::new(title.clone(), description, requirement_refs);
        let task_id = task.id;
        self.tasks.push(task);
        self.updated_at = Utc::now();

        let event = TaskAdded::new(self.id.as_uuid(), task_id.as_uuid(), title);

        Ok((task_id, Box::new(event)))
    }

    /// Start a task
    pub fn start_task(&mut self, task_id: TaskId) -> DomainResult<Box<dyn DomainEvent>> {
        let task = self
            .tasks
            .iter_mut()
            .find(|t| t.id == task_id)
            .ok_or_else(|| DomainError::EntityNotFound {
                entity_type: "Task".to_string(),
                id: task_id.to_string(),
            })?;

        // Only pending tasks can be started
        if task.status != TaskStatus::Pending {
            return Err(DomainError::BusinessRuleViolation {
                rule: format!(
                    "Cannot start task in {:?} state, must be Pending",
                    task.status
                ),
            });
        }

        task.status = TaskStatus::InProgress;
        task.updated_at = Utc::now();
        self.updated_at = Utc::now();

        // Update spec status to InProgress if not already
        if self.status != SpecStatus::InProgress && self.status != SpecStatus::Complete {
            self.status = SpecStatus::InProgress;
        }

        let event = TaskStarted::new(self.id.as_uuid(), task_id.as_uuid());

        Ok(Box::new(event))
    }

    /// Complete a task
    pub fn complete_task(&mut self, task_id: TaskId) -> DomainResult<Box<dyn DomainEvent>> {
        let task = self
            .tasks
            .iter_mut()
            .find(|t| t.id == task_id)
            .ok_or_else(|| DomainError::EntityNotFound {
                entity_type: "Task".to_string(),
                id: task_id.to_string(),
            })?;

        // Only in-progress tasks can be completed
        if task.status != TaskStatus::InProgress {
            return Err(DomainError::BusinessRuleViolation {
                rule: format!(
                    "Cannot complete task in {:?} state, must be InProgress",
                    task.status
                ),
            });
        }

        task.status = TaskStatus::Completed;
        task.updated_at = Utc::now();
        self.updated_at = Utc::now();

        let event = TaskCompleted::new(self.id.as_uuid(), task_id.as_uuid());

        Ok(Box::new(event))
    }

    // ========================================================================
    // Status Operations
    // ========================================================================

    /// Mark requirements as complete
    pub fn mark_requirements_complete(&mut self) -> DomainResult<()> {
        if self.requirements.is_empty() {
            return Err(DomainError::BusinessRuleViolation {
                rule: "Cannot mark requirements complete without any requirements".to_string(),
            });
        }
        self.status = SpecStatus::RequirementsComplete;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Mark design as complete
    pub fn mark_design_complete(&mut self) -> DomainResult<()> {
        if self.status != SpecStatus::RequirementsComplete {
            return Err(DomainError::BusinessRuleViolation {
                rule: "Requirements must be complete before design".to_string(),
            });
        }
        self.status = SpecStatus::DesignComplete;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Mark tasks as planned
    pub fn mark_tasks_planned(&mut self) -> DomainResult<()> {
        if self.tasks.is_empty() {
            return Err(DomainError::BusinessRuleViolation {
                rule: "Cannot mark tasks planned without any tasks".to_string(),
            });
        }
        self.status = SpecStatus::TasksPlanned;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Approve the specification (mark as complete)
    pub fn approve(&mut self) -> DomainResult<Box<dyn DomainEvent>> {
        // All tasks must be completed
        if self.completion_percentage() < 100.0 {
            return Err(DomainError::BusinessRuleViolation {
                rule: "Cannot approve specification with incomplete tasks".to_string(),
            });
        }

        self.status = SpecStatus::Complete;
        self.updated_at = Utc::now();

        let event = SpecificationApproved::new(self.id.as_uuid());

        Ok(Box::new(event))
    }

    /// Archive the specification
    pub fn archive(&mut self) -> DomainResult<Box<dyn DomainEvent>> {
        if self.status != SpecStatus::Complete {
            return Err(DomainError::BusinessRuleViolation {
                rule: "Cannot archive specification that is not complete".to_string(),
            });
        }

        self.status = SpecStatus::Archived;
        self.updated_at = Utc::now();

        let event = SpecificationImplemented::new(self.id.as_uuid());

        Ok(Box::new(event))
    }

    // ========================================================================
    // Query Methods
    // ========================================================================

    /// Calculate overall completion percentage
    ///
    /// REQ-DOMAIN-003.7: Track completion status across all tasks
    /// REQ-DOMAIN-003.10: Calculate overall completion percentage accurately
    pub fn completion_percentage(&self) -> f32 {
        if self.tasks.is_empty() {
            return 0.0;
        }

        let completed = self
            .tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Completed)
            .count();

        (completed as f32 / self.tasks.len() as f32) * 100.0
    }

    /// Get immutable identity
    pub fn id(&self) -> SpecificationId {
        self.id
    }

    /// Get project reference
    pub fn project_id(&self) -> ProjectId {
        self.project_id
    }

    /// Get specification name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get specification description
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Get specification version
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Get current status
    pub fn status(&self) -> SpecStatus {
        self.status
    }

    /// Get read-only access to requirements
    pub fn requirements(&self) -> &[Requirement] {
        &self.requirements
    }

    /// Get read-only access to tasks
    pub fn tasks(&self) -> &[Task] {
        &self.tasks
    }

    /// Get creation timestamp
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Get last update timestamp
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    /// Get count of pending tasks
    pub fn pending_task_count(&self) -> usize {
        self.tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Pending)
            .count()
    }

    /// Get count of completed tasks
    pub fn completed_task_count(&self) -> usize {
        self.tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Completed)
            .count()
    }
}

// ============================================================================
// Requirement Entity
// ============================================================================

/// Requirement entity within a Specification
///
/// REQ-DOMAIN-003.2: Contains title, description, and acceptance criteria
/// REQ-DOMAIN-003.3: Must have at least one acceptance criterion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirement {
    /// Requirement identity
    id: RequirementId,

    /// Requirement title (e.g., "REQ-001: User Authentication")
    title: String,

    /// Detailed description
    description: String,

    /// Acceptance criteria (at least one required)
    acceptance_criteria: Vec<String>,

    /// Whether the requirement has been approved
    approved: bool,

    /// Creation timestamp
    created_at: DateTime<Utc>,

    /// Last update timestamp
    updated_at: DateTime<Utc>,
}

impl Requirement {
    /// Create a new requirement (internal use by Specification aggregate)
    fn new(title: String, description: String, acceptance_criteria: Vec<String>) -> Self {
        let now = Utc::now();
        Self {
            id: RequirementId::new(),
            title,
            description,
            acceptance_criteria,
            approved: false,
            created_at: now,
            updated_at: now,
        }
    }

    /// Get requirement ID
    pub fn id(&self) -> RequirementId {
        self.id
    }

    /// Get requirement title
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Get requirement description
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Get acceptance criteria
    pub fn acceptance_criteria(&self) -> &[String] {
        &self.acceptance_criteria
    }

    /// Check if requirement is approved
    pub fn is_approved(&self) -> bool {
        self.approved
    }

    /// Get creation timestamp
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Get last update timestamp
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

// ============================================================================
// Task Entity
// ============================================================================

/// Task entity within a Specification
///
/// REQ-DOMAIN-003.4: Must trace to at least one requirement
/// REQ-DOMAIN-003.6: Supports hierarchical breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Task identity
    id: TaskId,

    /// Task title (e.g., "TASK-001: Implement login endpoint")
    title: String,

    /// Detailed description
    description: String,

    /// References to requirements this task fulfills
    requirement_refs: Vec<RequirementId>,

    /// Task status
    status: TaskStatus,

    /// Optional parent task for hierarchy
    parent_task_id: Option<TaskId>,

    /// Creation timestamp
    created_at: DateTime<Utc>,

    /// Last update timestamp
    updated_at: DateTime<Utc>,
}

/// Task lifecycle status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Task not yet started
    Pending,
    /// Task in progress
    InProgress,
    /// Task completed
    Completed,
    /// Task cancelled
    Cancelled,
}

impl Task {
    /// Create a new task (internal use by Specification aggregate)
    fn new(title: String, description: String, requirement_refs: Vec<RequirementId>) -> Self {
        let now = Utc::now();
        Self {
            id: TaskId::new(),
            title,
            description,
            requirement_refs,
            status: TaskStatus::Pending,
            parent_task_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Get task ID
    pub fn id(&self) -> TaskId {
        self.id
    }

    /// Get task title
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Get task description
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Get requirement references
    pub fn requirement_refs(&self) -> &[RequirementId] {
        &self.requirement_refs
    }

    /// Get task status
    pub fn status(&self) -> TaskStatus {
        self.status
    }

    /// Get parent task ID (if hierarchical)
    pub fn parent_task_id(&self) -> Option<TaskId> {
        self.parent_task_id
    }

    /// Get creation timestamp
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Get last update timestamp
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_specification() -> Specification {
        let project_id = ProjectId::new();
        let (spec, _) = Specification::create(
            project_id,
            "Test Spec".to_string(),
            "Test description".to_string(),
            "1.0.0".to_string(),
        )
        .unwrap();
        spec
    }

    #[test]
    fn test_create_specification_success() {
        let project_id = ProjectId::new();
        let result = Specification::create(
            project_id,
            "My Feature".to_string(),
            "Feature description".to_string(),
            "1.0.0".to_string(),
        );

        assert!(result.is_ok());
        let (spec, event) = result.unwrap();
        assert_eq!(spec.name(), "My Feature");
        assert_eq!(spec.status(), SpecStatus::Draft);
        assert!(spec.requirements().is_empty());
        assert!(spec.tasks().is_empty());
        assert_eq!(event.event_type(), "SpecificationCreated");
    }

    #[test]
    fn test_create_specification_empty_name_fails() {
        let project_id = ProjectId::new();
        let result = Specification::create(
            project_id,
            "".to_string(),
            "Description".to_string(),
            "1.0.0".to_string(),
        );

        assert!(result.is_err());
        match result.err().unwrap() {
            DomainError::ValidationError { field, .. } => assert_eq!(field, "name"),
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_create_specification_name_too_long_fails() {
        let project_id = ProjectId::new();
        let long_name = "x".repeat(101);
        let result = Specification::create(
            project_id,
            long_name,
            "Description".to_string(),
            "1.0.0".to_string(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_add_requirement_success() {
        let mut spec = create_test_specification();

        let result = spec.add_requirement(
            "REQ-001: User Auth".to_string(),
            "User must be able to authenticate".to_string(),
            vec!["AC1: Login form displayed".to_string()],
        );

        assert!(result.is_ok());
        let (req_id, event) = result.unwrap();
        assert_eq!(spec.requirements().len(), 1);
        assert_eq!(spec.requirements()[0].id(), req_id);
        assert_eq!(event.event_type(), "RequirementAdded");
    }

    #[test]
    fn test_add_requirement_without_acceptance_criteria_fails() {
        let mut spec = create_test_specification();

        let result = spec.add_requirement(
            "REQ-001".to_string(),
            "Description".to_string(),
            vec![], // Empty acceptance criteria
        );

        assert!(result.is_err());
        match result.err().unwrap() {
            DomainError::BusinessRuleViolation { rule } => {
                assert!(rule.contains("acceptance criterion"));
            }
            _ => panic!("Expected BusinessRuleViolation"),
        }
    }

    #[test]
    fn test_add_task_success() {
        let mut spec = create_test_specification();

        // First add a requirement
        let (req_id, _) = spec
            .add_requirement(
                "REQ-001".to_string(),
                "Description".to_string(),
                vec!["AC1".to_string()],
            )
            .unwrap();

        // Now add a task referencing the requirement
        let result = spec.add_task(
            "TASK-001: Implement feature".to_string(),
            "Implementation details".to_string(),
            vec![req_id],
        );

        assert!(result.is_ok());
        let (task_id, event) = result.unwrap();
        assert_eq!(spec.tasks().len(), 1);
        assert_eq!(spec.tasks()[0].id(), task_id);
        assert_eq!(spec.tasks()[0].status(), TaskStatus::Pending);
        assert_eq!(event.event_type(), "TaskAdded");
    }

    #[test]
    fn test_add_task_without_requirement_refs_fails() {
        let mut spec = create_test_specification();

        let result = spec.add_task(
            "TASK-001".to_string(),
            "Description".to_string(),
            vec![], // No requirement references
        );

        assert!(result.is_err());
        match result.err().unwrap() {
            DomainError::BusinessRuleViolation { rule } => {
                assert!(rule.contains("reference at least one requirement"));
            }
            _ => panic!("Expected BusinessRuleViolation"),
        }
    }

    #[test]
    fn test_add_task_with_nonexistent_requirement_fails() {
        let mut spec = create_test_specification();

        let fake_req_id = RequirementId::new();
        let result = spec.add_task(
            "TASK-001".to_string(),
            "Description".to_string(),
            vec![fake_req_id],
        );

        assert!(result.is_err());
        match result.err().unwrap() {
            DomainError::BusinessRuleViolation { rule } => {
                assert!(rule.contains("non-existent requirement"));
            }
            _ => panic!("Expected BusinessRuleViolation"),
        }
    }

    #[test]
    fn test_task_lifecycle_start_and_complete() {
        let mut spec = create_test_specification();

        // Add requirement and task
        let (req_id, _) = spec
            .add_requirement("REQ".to_string(), "Desc".to_string(), vec!["AC".to_string()])
            .unwrap();
        let (task_id, _) = spec
            .add_task("TASK".to_string(), "Desc".to_string(), vec![req_id])
            .unwrap();

        // Start the task
        let event = spec.start_task(task_id).unwrap();
        assert_eq!(spec.tasks()[0].status(), TaskStatus::InProgress);
        assert_eq!(event.event_type(), "TaskStarted");

        // Complete the task
        let event = spec.complete_task(task_id).unwrap();
        assert_eq!(spec.tasks()[0].status(), TaskStatus::Completed);
        assert_eq!(event.event_type(), "TaskCompleted");
    }

    #[test]
    fn test_cannot_start_already_started_task() {
        let mut spec = create_test_specification();

        let (req_id, _) = spec
            .add_requirement("REQ".to_string(), "Desc".to_string(), vec!["AC".to_string()])
            .unwrap();
        let (task_id, _) = spec
            .add_task("TASK".to_string(), "Desc".to_string(), vec![req_id])
            .unwrap();

        // Start the task
        spec.start_task(task_id).unwrap();

        // Try to start again
        let result = spec.start_task(task_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_completion_percentage() {
        let mut spec = create_test_specification();

        // No tasks = 0%
        assert_eq!(spec.completion_percentage(), 0.0);

        // Add requirement and tasks
        let (req_id, _) = spec
            .add_requirement("REQ".to_string(), "Desc".to_string(), vec!["AC".to_string()])
            .unwrap();

        let (task1_id, _) = spec
            .add_task("TASK-1".to_string(), "Desc".to_string(), vec![req_id])
            .unwrap();
        let (task2_id, _) = spec
            .add_task("TASK-2".to_string(), "Desc".to_string(), vec![req_id])
            .unwrap();

        // 0 of 2 complete = 0%
        assert_eq!(spec.completion_percentage(), 0.0);

        // Complete task 1 (1 of 2 = 50%)
        spec.start_task(task1_id).unwrap();
        spec.complete_task(task1_id).unwrap();
        assert_eq!(spec.completion_percentage(), 50.0);

        // Complete task 2 (2 of 2 = 100%)
        spec.start_task(task2_id).unwrap();
        spec.complete_task(task2_id).unwrap();
        assert_eq!(spec.completion_percentage(), 100.0);
    }

    #[test]
    fn test_approve_specification() {
        let mut spec = create_test_specification();

        // Add and complete tasks
        let (req_id, _) = spec
            .add_requirement("REQ".to_string(), "Desc".to_string(), vec!["AC".to_string()])
            .unwrap();
        let (task_id, _) = spec
            .add_task("TASK".to_string(), "Desc".to_string(), vec![req_id])
            .unwrap();

        spec.start_task(task_id).unwrap();
        spec.complete_task(task_id).unwrap();

        // Approve
        let event = spec.approve().unwrap();
        assert_eq!(spec.status(), SpecStatus::Complete);
        assert_eq!(event.event_type(), "SpecificationApproved");
    }

    #[test]
    fn test_cannot_approve_incomplete_specification() {
        let mut spec = create_test_specification();

        // Add tasks but don't complete them
        let (req_id, _) = spec
            .add_requirement("REQ".to_string(), "Desc".to_string(), vec!["AC".to_string()])
            .unwrap();
        spec.add_task("TASK".to_string(), "Desc".to_string(), vec![req_id])
            .unwrap();

        // Try to approve
        let result = spec.approve();
        assert!(result.is_err());
        match result.err().unwrap() {
            DomainError::BusinessRuleViolation { rule } => {
                assert!(rule.contains("incomplete tasks"));
            }
            _ => panic!("Expected BusinessRuleViolation"),
        }
    }

    #[test]
    fn test_update_requirement() {
        let mut spec = create_test_specification();

        let (req_id, _) = spec
            .add_requirement(
                "Original Title".to_string(),
                "Original Desc".to_string(),
                vec!["AC1".to_string()],
            )
            .unwrap();

        // Update title
        let event = spec
            .update_requirement(
                req_id,
                Some("Updated Title".to_string()),
                None,
                None,
            )
            .unwrap();

        assert_eq!(spec.requirements()[0].title(), "Updated Title");
        assert_eq!(event.event_type(), "RequirementUpdated");
    }

    #[test]
    fn test_approve_requirement() {
        let mut spec = create_test_specification();

        let (req_id, _) = spec
            .add_requirement("REQ".to_string(), "Desc".to_string(), vec!["AC".to_string()])
            .unwrap();

        assert!(!spec.requirements()[0].is_approved());

        let event = spec.approve_requirement(req_id).unwrap();
        assert!(spec.requirements()[0].is_approved());
        assert_eq!(event.event_type(), "RequirementApproved");
    }

    #[test]
    fn test_archive_completed_specification() {
        let mut spec = create_test_specification();

        // Add, complete, and approve
        let (req_id, _) = spec
            .add_requirement("REQ".to_string(), "Desc".to_string(), vec!["AC".to_string()])
            .unwrap();
        let (task_id, _) = spec
            .add_task("TASK".to_string(), "Desc".to_string(), vec![req_id])
            .unwrap();

        spec.start_task(task_id).unwrap();
        spec.complete_task(task_id).unwrap();
        spec.approve().unwrap();

        // Archive
        let event = spec.archive().unwrap();
        assert_eq!(spec.status(), SpecStatus::Archived);
        assert_eq!(event.event_type(), "SpecificationImplemented");
    }

    #[test]
    fn test_cannot_archive_incomplete_specification() {
        let spec = create_test_specification();

        // Spec is in Draft state
        let result = Specification::create(
            ProjectId::new(),
            "Test".to_string(),
            "Desc".to_string(),
            "1.0.0".to_string(),
        )
        .unwrap()
        .0;

        // Try to archive
        let result = {
            let mut spec = result;
            spec.archive()
        };
        assert!(result.is_err());
    }
}
