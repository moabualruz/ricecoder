//! Domain services - stateless business logic that doesn't belong to entities
//!
//! Domain services are stateless and implement complex domain logic that spans multiple aggregates.
//! They operate on aggregates and value objects, returning domain results.
//!
//! REQ-ARCH-001.3: Domain services are stateless and coordinate aggregate operations

use serde::{Deserialize, Serialize};

use crate::{
    project::Project,
    session::Session,
    specification::Specification,
    errors::*,
    value_objects::*,
};

/// Validation result for domain operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    /// Create valid result
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Create invalid result with errors
    pub fn invalid(errors: Vec<String>) -> Self {
        Self {
            is_valid: false,
            errors,
            warnings: Vec::new(),
        }
    }

    /// Add warnings to result
    pub fn with_warnings(mut self, warnings: Vec<String>) -> Self {
        self.warnings = warnings;
        self
    }

    /// Add error to result
    pub fn add_error(&mut self, error: String) {
        self.is_valid = false;
        self.errors.push(error);
    }

    /// Add warning to result
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
}

/// Domain validation service - stateless validation logic for aggregates
///
/// REQ-ARCH-001.3: Domain services are stateless and coordinate aggregate operations
pub struct ValidationService;

impl ValidationService {
    /// Validate project invariants
    ///
    /// - Project name must be 1-100 characters
    /// - Project name must contain only alphanumeric, hyphens, underscores, spaces
    /// - Project path cannot contain ".."
    /// - Project must be in valid state
    pub fn validate_project(project: &Project) -> ValidationResult {
        let mut result = ValidationResult::valid();

        // Name length validation
        if project.name().is_empty() {
            result.add_error("Project name cannot be empty".to_string());
        } else if project.name().len() > 100 {
            result.add_error("Project name cannot exceed 100 characters".to_string());
        }

        // Name format validation
        for ch in project.name().chars() {
            if !ch.is_alphanumeric() && ch != '-' && ch != '_' && ch != ' ' {
                result.add_error(format!(
                    "Project name contains invalid character '{}'. Only alphanumeric, hyphens, underscores, and spaces allowed",
                    ch
                ));
                break;
            }
        }

        // Path security validation
        if project.root_path().contains("..") {
            result.add_error("Project path cannot contain '..' for security reasons".to_string());
        }

        // Status validation
        if matches!(project.status(), crate::project::ProjectStatus::Deleted) {
            result.add_warning("Project has been deleted and may not be usable".to_string());
        }

        result
    }

    /// Validate session invariants
    ///
    /// - Session must be in valid state
    /// - Session must not have exceeded message limits
    pub fn validate_session(session: &Session) -> ValidationResult {
        let mut result = ValidationResult::valid();

        // Session state validation
        if session.is_archived() {
            result.add_warning("Session has been archived and is read-only".to_string());
        }

        // Message capacity check
        let capacity_percent = (session.message_count() as f32 / session.max_messages() as f32) * 100.0;
        if capacity_percent > 90.0 {
            result.add_warning(format!(
                "Session message capacity is {}% full",
                capacity_percent as u32
            ));
        }

        result
    }

    /// Validate specification invariants
    ///
    /// - Specification name must not be empty
    /// - Specification name cannot exceed 255 characters
    /// - Specification must be in valid status
    pub fn validate_specification(specification: &Specification) -> ValidationResult {
        let mut result = ValidationResult::valid();

        if specification.name().is_empty() {
            result.add_error("Specification name cannot be empty".to_string());
        } else if specification.name().len() > 255 {
            result.add_error("Specification name cannot exceed 255 characters".to_string());
        }

        // Status validation
        match specification.status() {
            crate::specification::SpecStatus::Archived => {
                result.add_warning("Specification has been archived and is read-only".to_string());
            }
            crate::specification::SpecStatus::Draft => {
                // Draft is expected early in lifecycle
            }
            _ => {}
        }

        result
    }
}

/// Domain analysis service - stateless analysis logic
///
/// REQ-ARCH-001.3: Provides analysis operations without external dependencies
pub struct AnalysisService;

impl AnalysisService {
    /// Analyze specification progress
    ///
    /// Evaluates:
    /// - Requirements defined
    /// - Task planning
    /// - Task completion status
    /// - Overall progress percentage
    pub fn analyze_specification_progress(
        specification: &Specification,
    ) -> DomainResult<SpecificationProgress> {
        let has_requirements = specification.requirements().len() > 0;
        let has_tasks = specification.tasks().len() > 0;
        let completion_percentage = specification.completion_percentage();

        let status = match (has_requirements, has_tasks, completion_percentage) {
            (true, true, percent) if (percent - 100.0).abs() < f32::EPSILON => ProgressStatus::Complete,
            (true, true, percent) if percent > 50.0 => ProgressStatus::InProgress,
            (true, true, _) => ProgressStatus::Planned,
            (true, false, _) => ProgressStatus::RequirementsDefined,
            _ => ProgressStatus::Draft,
        };

        Ok(SpecificationProgress {
            status,
            completion_percentage,
            requirement_count: specification.requirements().len() as u32,
            task_count: specification.tasks().len() as u32,
            completed_tasks: specification.completed_task_count() as u32,
        })
    }

    /// Analyze project health
    ///
    /// Evaluates:
    /// - Active/archived status
    /// - Consistency (domain layer ensures this)
    pub fn analyze_project_health(project: &Project) -> ProjectHealth {
        ProjectHealth {
            is_active: project.is_active(),
            is_consistent: true, // Domain layer enforces consistency through aggregates
        }
    }
}

/// Specification progress metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecificationProgress {
    pub status: ProgressStatus,
    pub completion_percentage: f32,
    pub requirement_count: u32,
    pub task_count: u32,
    pub completed_tasks: u32,
}

/// Progress status enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ProgressStatus {
    Draft,
    RequirementsDefined,
    Planned,
    InProgress,
    Complete,
}

/// Project health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectHealth {
    pub is_active: bool,
    pub is_consistent: bool,
}
