//! Task entity within a Specification
//!
//! Must trace to at least one requirement
//! Supports hierarchical breakdown

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::value_objects::{RequirementId, TaskId};

/// Task entity within a Specification
///
/// Must trace to at least one requirement
/// Supports hierarchical breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Task identity
    pub(crate) id: TaskId,

    /// Task title (e.g., "TASK-001: Implement login endpoint")
    pub(crate) title: String,

    /// Detailed description
    pub(crate) description: String,

    /// References to requirements this task fulfills
    pub(crate) requirement_refs: Vec<RequirementId>,

    /// Task status
    pub(crate) status: TaskStatus,

    /// Optional parent task for hierarchy
    pub(crate) parent_task_id: Option<TaskId>,

    /// Creation timestamp
    pub(crate) created_at: DateTime<Utc>,

    /// Last update timestamp
    pub(crate) updated_at: DateTime<Utc>,
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
    pub(crate) fn new(title: String, description: String, requirement_refs: Vec<RequirementId>) -> Self {
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

    /// Reconstitute a task from persistence
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: TaskId,
        title: String,
        description: String,
        requirement_refs: Vec<RequirementId>,
        status: TaskStatus,
        parent_task_id: Option<TaskId>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            title,
            description,
            requirement_refs,
            status,
            parent_task_id,
            created_at,
            updated_at,
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
