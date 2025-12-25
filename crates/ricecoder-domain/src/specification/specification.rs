//! Specification Aggregate Root
//!
//! Specification aggregate with full DDD compliance
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

use super::requirement::Requirement;
use super::task::{Task, TaskStatus};

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

    /// Concurrency version for optimistic locking
    concurrency_version: u64,
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
    /// Aggregate root creation with event emission
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
            concurrency_version: 1,
        };

        let event = SpecificationCreated::new(id.as_uuid(), project_id.as_uuid(), version);

        Ok((spec, Box::new(event)))
    }

    /// Reconstitute a specification from persistence
    ///
    /// Bypasses validation since data was validated during original creation.
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: SpecificationId,
        project_id: ProjectId,
        name: String,
        description: String,
        version: String,
        requirements: Vec<Requirement>,
        tasks: Vec<Task>,
        status: SpecStatus,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        concurrency_version: u64,
    ) -> Self {
        Self {
            id,
            project_id,
            name,
            description,
            version,
            requirements,
            tasks,
            status,
            created_at,
            updated_at,
            concurrency_version,
        }
    }

    // ========================================================================
    // Requirement Operations
    // ========================================================================

    /// Add a requirement with validation
    ///
    /// All requirements must have acceptance criteria
    /// Validate completeness when adding requirements
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
        let req_id = requirement.id();
        self.requirements.push(requirement);
        self.updated_at = Utc::now();
        self.concurrency_version += 1;

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
            .find(|r| r.id() == requirement_id)
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
        self.concurrency_version += 1;

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
            .find(|r| r.id() == requirement_id)
            .ok_or_else(|| DomainError::EntityNotFound {
                entity_type: "Requirement".to_string(),
                id: requirement_id.to_string(),
            })?;

        req.approved = true;
        req.updated_at = Utc::now();
        self.updated_at = Utc::now();
        self.concurrency_version += 1;

        let event = RequirementApproved::new(self.id.as_uuid(), requirement_id.as_uuid());

        Ok(Box::new(event))
    }

    // ========================================================================
    // Task Operations
    // ========================================================================

    /// Add a task with traceability validation
    ///
    /// All tasks must trace to at least one requirement
    /// Prevent task addition without requirement references
    /// Validate all requirement references exist
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
            if !self.requirements.iter().any(|r| &r.id() == req_ref) {
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
        let task_id = task.id();
        self.tasks.push(task);
        self.updated_at = Utc::now();
        self.concurrency_version += 1;

        let event = TaskAdded::new(self.id.as_uuid(), task_id.as_uuid(), title);

        Ok((task_id, Box::new(event)))
    }

    /// Start a task
    pub fn start_task(&mut self, task_id: TaskId) -> DomainResult<Box<dyn DomainEvent>> {
        let task = self
            .tasks
            .iter_mut()
            .find(|t| t.id() == task_id)
            .ok_or_else(|| DomainError::EntityNotFound {
                entity_type: "Task".to_string(),
                id: task_id.to_string(),
            })?;

        // Only pending tasks can be started
        if task.status() != TaskStatus::Pending {
            return Err(DomainError::BusinessRuleViolation {
                rule: format!(
                    "Cannot start task in {:?} state, must be Pending",
                    task.status()
                ),
            });
        }

        task.status = TaskStatus::InProgress;
        task.updated_at = Utc::now();
        self.updated_at = Utc::now();
        self.concurrency_version += 1;

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
            .find(|t| t.id() == task_id)
            .ok_or_else(|| DomainError::EntityNotFound {
                entity_type: "Task".to_string(),
                id: task_id.to_string(),
            })?;

        // Only in-progress tasks can be completed
        if task.status() != TaskStatus::InProgress {
            return Err(DomainError::BusinessRuleViolation {
                rule: format!(
                    "Cannot complete task in {:?} state, must be InProgress",
                    task.status()
                ),
            });
        }

        task.status = TaskStatus::Completed;
        task.updated_at = Utc::now();
        self.updated_at = Utc::now();
        self.concurrency_version += 1;

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
        self.concurrency_version += 1;
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
        self.concurrency_version += 1;
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
        self.concurrency_version += 1;
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
        self.concurrency_version += 1;

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
        self.concurrency_version += 1;

        let event = SpecificationImplemented::new(self.id.as_uuid());

        Ok(Box::new(event))
    }

    // ========================================================================
    // Query Methods
    // ========================================================================

    /// Calculate overall completion percentage
    ///
    /// Track completion status across all tasks
    /// Calculate overall completion percentage accurately
    pub fn completion_percentage(&self) -> f32 {
        if self.tasks.is_empty() {
            return 0.0;
        }

        let completed = self
            .tasks
            .iter()
            .filter(|t| t.status() == TaskStatus::Completed)
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
            .filter(|t| t.status() == TaskStatus::Pending)
            .count()
    }

    /// Get count of completed tasks
    pub fn completed_task_count(&self) -> usize {
        self.tasks
            .iter()
            .filter(|t| t.status() == TaskStatus::Completed)
            .count()
    }

    /// Get concurrency version for optimistic locking
    pub fn concurrency_version(&self) -> u64 {
        self.concurrency_version
    }
}
