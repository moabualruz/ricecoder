//! Project Aggregate Root
//!
//! REQ-DOMAIN-001: Project aggregate with full DDD compliance
//! - Aggregate root with encapsulated entities
//! - Invariant enforcement
//! - Domain event emission
//! - Immutable identity

use crate::errors::{DomainError, DomainResult};
use crate::events::project::*;
use crate::events::DomainEvent;
use crate::value_objects::{ProjectId, ProgrammingLanguage};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Project Aggregate Root
///
/// Encapsulates all project-related business logic and invariants.
/// All operations emit domain events for auditability and event sourcing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// Immutable identity
    id: ProjectId,

    /// Project name (must be non-empty, max 100 chars)
    name: String,

    /// Description (optional, max 1000 chars)
    description: Option<String>,

    /// Primary programming language
    language: ProgrammingLanguage,

    /// Root path (must be valid, no ".." for security)
    root_path: String,

    /// Creation timestamp (immutable)
    created_at: DateTime<Utc>,

    /// Last update timestamp
    updated_at: DateTime<Utc>,

    /// Arbitrary metadata (key-value pairs)
    metadata: HashMap<String, String>,

    /// Lifecycle status
    status: ProjectStatus,
}

/// Project lifecycle status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectStatus {
    Active,
    Archived,
    Deleted,
}

impl Project {
    /// Create new project with invariant validation
    ///
    /// # Invariants
    /// - Name: 1-100 chars, alphanumeric + hyphen/underscore/space
    /// - Path: Non-empty, no ".." sequences
    ///
    /// # Returns
    /// Tuple of (Project, Vec<DomainEvent>) for event sourcing
    ///
    /// # Errors
    /// Returns DomainError if invariants violated
    pub fn create(
        name: String,
        language: ProgrammingLanguage,
        root_path: String,
        description: Option<String>,
    ) -> DomainResult<(Self, Vec<Box<dyn DomainEvent>>)> {
        // REQ-DOMAIN-001.2: Enforce name constraints
        Self::validate_name(&name)?;

        // REQ-DOMAIN-001.3: Enforce path constraints
        Self::validate_path(&root_path)?;

        let id = ProjectId::new();
        let now = Utc::now();

        let project = Self {
            id,
            name: name.clone(),
            description: description.clone(),
            language,
            root_path: root_path.clone(),
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
            status: ProjectStatus::Active,
        };

        // REQ-DOMAIN-001.5: Emit domain events
        let event = ProjectCreated::new(id.as_uuid(), name, description);
        let events: Vec<Box<dyn DomainEvent>> = vec![Box::new(event)];

        Ok((project, events))
    }

    /// Rename project
    ///
    /// # Invariants
    /// - New name must be different from current
    /// - Name must pass validation
    pub fn rename(&mut self, new_name: String) -> DomainResult<Vec<Box<dyn DomainEvent>>> {
        if self.name == new_name {
            return Err(DomainError::ValidationError {
                field: "name".into(),
                reason: "Name unchanged".into(),
            });
        }

        Self::validate_name(&new_name)?;

        let old_name = self.name.clone();
        self.name = new_name.clone();
        self.updated_at = Utc::now();

        let event = ProjectUpdated::new(self.id.as_uuid(), new_name, self.description.clone());
        Ok(vec![Box::new(event)])
    }

    /// Update description
    pub fn update_description(
        &mut self,
        description: Option<String>,
    ) -> DomainResult<Vec<Box<dyn DomainEvent>>> {
        self.description = description.clone();
        self.updated_at = Utc::now();

        let event = ProjectUpdated::new(
            self.id.as_uuid(),
            self.name.clone(),
            description,
        );
        Ok(vec![Box::new(event)])
    }

    /// Archive project
    ///
    /// # Business Rules
    /// - Only active projects can be archived
    pub fn archive(&mut self) -> DomainResult<Vec<Box<dyn DomainEvent>>> {
        if self.status != ProjectStatus::Active {
            return Err(DomainError::BusinessRuleViolation {
                rule: "Only active projects can be archived".into(),
            });
        }

        self.status = ProjectStatus::Archived;
        self.updated_at = Utc::now();

        let event = ProjectArchived::new(self.id.as_uuid());
        Ok(vec![Box::new(event)])
    }

    /// Restore archived project
    ///
    /// # Business Rules
    /// - Only archived projects can be restored
    pub fn restore(&mut self) -> DomainResult<Vec<Box<dyn DomainEvent>>> {
        if self.status != ProjectStatus::Archived {
            return Err(DomainError::BusinessRuleViolation {
                rule: "Only archived projects can be restored".into(),
            });
        }

        self.status = ProjectStatus::Active;
        self.updated_at = Utc::now();

        let event = ProjectRestored::new(self.id.as_uuid());
        Ok(vec![Box::new(event)])
    }

    /// Mark project as deleted (soft delete)
    pub fn delete(&mut self) -> DomainResult<Vec<Box<dyn DomainEvent>>> {
        self.status = ProjectStatus::Deleted;
        self.updated_at = Utc::now();

        let event = ProjectDeleted::new(self.id.as_uuid());
        Ok(vec![Box::new(event)])
    }

    /// Add metadata entry
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
        self.updated_at = Utc::now();
    }

    /// Get metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    // === Getters (REQ-DOMAIN-001.4: Prevent direct access) ===

    pub fn id(&self) -> ProjectId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn language(&self) -> ProgrammingLanguage {
        self.language
    }

    pub fn root_path(&self) -> &str {
        &self.root_path
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn status(&self) -> ProjectStatus {
        self.status
    }

    pub fn is_active(&self) -> bool {
        self.status == ProjectStatus::Active
    }

    pub fn is_archived(&self) -> bool {
        self.status == ProjectStatus::Archived
    }

    pub fn is_deleted(&self) -> bool {
        self.status == ProjectStatus::Deleted
    }

    // === Validation (REQ-DOMAIN-001.2, REQ-DOMAIN-001.3) ===

    fn validate_name(name: &str) -> DomainResult<()> {
        if name.is_empty() || name.len() > 100 {
            return Err(DomainError::InvalidProjectName {
                reason: "Name must be 1-100 characters".into(),
            });
        }

        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == ' ')
        {
            return Err(DomainError::InvalidProjectName {
                reason: "Name can only contain alphanumeric, hyphen, underscore, space".into(),
            });
        }

        Ok(())
    }

    fn validate_path(path: &str) -> DomainResult<()> {
        if path.is_empty() {
            return Err(DomainError::InvalidFilePath {
                reason: "Path cannot be empty".into(),
            });
        }

        // Security: Prevent directory traversal
        if path.contains("..") {
            return Err(DomainError::InvalidFilePath {
                reason: "Path cannot contain '..' (security)".into(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_project_success() {
        let (project, events) = Project::create(
            "test-project".into(),
            ProgrammingLanguage::Rust,
            "/path/to/project".into(),
            Some("Test project".into()),
        )
        .unwrap();

        assert_eq!(project.name(), "test-project");
        assert_eq!(project.language(), ProgrammingLanguage::Rust);
        assert_eq!(project.status(), ProjectStatus::Active);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type(), "ProjectCreated");
    }

    #[test]
    fn test_create_project_invalid_name_empty() {
        let result = Project::create(
            "".into(),
            ProgrammingLanguage::Rust,
            "/path".into(),
            None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_create_project_invalid_name_too_long() {
        let long_name = "a".repeat(101);
        let result = Project::create(
            long_name,
            ProgrammingLanguage::Rust,
            "/path".into(),
            None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_create_project_invalid_path_with_dotdot() {
        let result = Project::create(
            "test".into(),
            ProgrammingLanguage::Rust,
            "../path".into(),
            None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_rename_project() {
        let (mut project, _) = Project::create(
            "old-name".into(),
            ProgrammingLanguage::Rust,
            "/path".into(),
            None,
        )
        .unwrap();

        let events = project.rename("new-name".into()).unwrap();
        assert_eq!(project.name(), "new-name");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type(), "ProjectUpdated");
    }

    #[test]
    fn test_rename_same_name_fails() {
        let (mut project, _) = Project::create(
            "test-name".into(),
            ProgrammingLanguage::Rust,
            "/path".into(),
            None,
        )
        .unwrap();

        let result = project.rename("test-name".into());
        assert!(result.is_err());
    }

    #[test]
    fn test_archive_and_restore() {
        let (mut project, _) = Project::create(
            "test".into(),
            ProgrammingLanguage::Rust,
            "/path".into(),
            None,
        )
        .unwrap();

        // Archive
        let events = project.archive().unwrap();
        assert!(project.is_archived());
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type(), "ProjectArchived");

        // Restore
        let events = project.restore().unwrap();
        assert!(project.is_active());
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type(), "ProjectRestored");
    }

    #[test]
    fn test_cannot_archive_already_archived() {
        let (mut project, _) = Project::create(
            "test".into(),
            ProgrammingLanguage::Rust,
            "/path".into(),
            None,
        )
        .unwrap();

        project.archive().unwrap();
        let result = project.archive();
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_project() {
        let (mut project, _) = Project::create(
            "test".into(),
            ProgrammingLanguage::Rust,
            "/path".into(),
            None,
        )
        .unwrap();

        let events = project.delete().unwrap();
        assert!(project.is_deleted());
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type(), "ProjectDeleted");
    }

    #[test]
    fn test_metadata_operations() {
        let (mut project, _) = Project::create(
            "test".into(),
            ProgrammingLanguage::Rust,
            "/path".into(),
            None,
        )
        .unwrap();

        project.add_metadata("author".into(), "test-user".into());
        assert_eq!(project.get_metadata("author"), Some(&"test-user".to_string()));
        assert_eq!(project.get_metadata("nonexistent"), None);
    }
}
