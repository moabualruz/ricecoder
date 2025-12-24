//! Specification-related DTOs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use ricecoder_domain::specification::{Specification, SpecStatus};

/// Command to create a new specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSpecificationCommand {
    pub project_id: String,
    pub name: String,
    pub description: String,
    pub version: Option<String>,
}

/// Command to add a requirement to a specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddRequirementCommand {
    pub specification_id: String,
    pub title: String,
    pub description: String,
    pub acceptance_criteria: Vec<String>,
}

/// Command to add a task to a specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddTaskCommand {
    pub specification_id: String,
    pub title: String,
    pub description: String,
    pub requirement_ids: Vec<String>,
}

/// Specification summary DTO (list view)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecificationSummaryDto {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub status: String,
    pub completion_percentage: f32,
    pub created_at: DateTime<Utc>,
}

impl SpecificationSummaryDto {
    /// Create from domain aggregate
    pub fn from_domain(spec: &Specification) -> Self {
        Self {
            id: spec.id().to_string(),
            project_id: spec.project_id().to_string(),
            name: spec.name().to_string(),
            status: format!("{:?}", spec.status()),
            completion_percentage: spec.completion_percentage(),
            created_at: spec.created_at(),
        }
    }
}

/// Specification detail DTO (single view)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecificationDetailDto {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub status: String,
    pub requirements: Vec<RequirementDto>,
    pub tasks: Vec<TaskDto>,
    pub completion_percentage: f32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl SpecificationDetailDto {
    /// Create from domain aggregate
    pub fn from_domain(spec: &Specification) -> Self {
        Self {
            id: spec.id().to_string(),
            project_id: spec.project_id().to_string(),
            name: spec.name().to_string(),
            description: spec.description().to_string(),
            version: spec.version().to_string(),
            status: format!("{:?}", spec.status()),
            requirements: spec
                .requirements()
                .iter()
                .map(RequirementDto::from_domain)
                .collect(),
            tasks: spec
                .tasks()
                .iter()
                .map(TaskDto::from_domain)
                .collect(),
            completion_percentage: spec.completion_percentage(),
            created_at: spec.created_at(),
            updated_at: spec.updated_at(),
        }
    }
}

/// Requirement DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementDto {
    pub id: String,
    pub title: String,
    pub description: String,
    pub acceptance_criteria: Vec<String>,
    pub is_approved: bool,
}

impl RequirementDto {
    /// Create from domain entity
    pub fn from_domain(req: &ricecoder_domain::specification::Requirement) -> Self {
        Self {
            id: req.id().to_string(),
            title: req.title().to_string(),
            description: req.description().to_string(),
            acceptance_criteria: req.acceptance_criteria().to_vec(),
            is_approved: req.is_approved(),
        }
    }
}

/// Task DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDto {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub requirement_ids: Vec<String>,
}

impl TaskDto {
    /// Create from domain entity
    pub fn from_domain(task: &ricecoder_domain::specification::Task) -> Self {
        Self {
            id: task.id().to_string(),
            title: task.title().to_string(),
            description: task.description().to_string(),
            status: format!("{:?}", task.status()),
            requirement_ids: task.requirement_refs().iter().map(|r| r.to_string()).collect(),
        }
    }
}
