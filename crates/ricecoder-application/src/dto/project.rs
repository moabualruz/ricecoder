//! Project-related DTOs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use ricecoder_domain::project::{Project, ProjectStatus};
use ricecoder_domain::value_objects::ProgrammingLanguage;

/// Command to create a new project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectCommand {
    pub name: String,
    pub root_path: String,
    pub language: ProgrammingLanguage,
    pub description: Option<String>,
}

/// Command to update a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProjectCommand {
    pub project_id: String,
    pub name: Option<String>,
    pub description: Option<String>,
}

/// Command to rename a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenameProjectCommand {
    pub project_id: String,
    pub new_name: String,
}

/// Project summary DTO (list view)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSummaryDto {
    pub id: String,
    pub name: String,
    pub language: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

impl ProjectSummaryDto {
    /// Create from domain aggregate
    pub fn from_domain(project: &Project) -> Self {
        Self {
            id: project.id().to_string(),
            name: project.name().to_string(),
            language: project.language().to_string(),
            status: format!("{:?}", project.status()),
            created_at: project.created_at(),
        }
    }
}

/// Project detail DTO (single view)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDetailDto {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub language: String,
    pub root_path: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
    pub is_archived: bool,
}

impl ProjectDetailDto {
    /// Create from domain aggregate
    pub fn from_domain(project: &Project) -> Self {
        Self {
            id: project.id().to_string(),
            name: project.name().to_string(),
            description: project.description().map(|s| s.to_string()),
            language: project.language().to_string(),
            root_path: project.root_path().to_string(),
            status: format!("{:?}", project.status()),
            created_at: project.created_at(),
            updated_at: project.updated_at(),
            is_active: project.is_active(),
            is_archived: project.is_archived(),
        }
    }
}

/// Project analysis result DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectAnalysisDto {
    pub project_id: String,
    pub is_healthy: bool,
    pub analysis_timestamp: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_project_command_serialization() {
        let cmd = CreateProjectCommand {
            name: "test-project".into(),
            root_path: "/path/to/project".into(),
            language: ProgrammingLanguage::Rust,
            description: Some("A test project".into()),
        };

        let json = serde_json::to_string(&cmd).unwrap();
        let parsed: CreateProjectCommand = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed.name, "test-project");
        assert_eq!(parsed.language, ProgrammingLanguage::Rust);
    }
}
