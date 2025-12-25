//! Project entity representing a code project being analyzed

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{errors::*, value_objects::*};

/// Core project entity representing a code project being analyzed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: ProjectId,
    pub name: String,
    pub description: Option<String>,
    pub language: ProgrammingLanguage,
    pub root_path: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

impl Project {
    /// Create a new project with validation
    pub fn new(
        name: String,
        language: ProgrammingLanguage,
        root_path: String,
    ) -> DomainResult<Self> {
        Self::validate_name(&name)?;
        Self::validate_path(&root_path)?;

        let now = Utc::now();
        Ok(Self {
            id: ProjectId::new(),
            name,
            description: None,
            language,
            root_path,
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        })
    }

    /// Update project name with validation
    pub fn update_name(&mut self, name: String) -> DomainResult<()> {
        Self::validate_name(&name)?;
        self.name = name;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Update project description
    pub fn update_description(&mut self, description: Option<String>) {
        self.description = description;
        self.updated_at = Utc::now();
    }

    /// Add metadata
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
        self.updated_at = Utc::now();
    }

    /// Validate project name
    fn validate_name(name: &str) -> DomainResult<()> {
        if name.trim().is_empty() {
            return Err(DomainError::InvalidProjectName {
                reason: "Project name cannot be empty".to_string(),
            });
        }

        if name.len() > 100 {
            return Err(DomainError::InvalidProjectName {
                reason: "Project name cannot exceed 100 characters".to_string(),
            });
        }

        // Check for valid characters (alphanumeric, dash, underscore)
        if !regex::Regex::new(r"^[a-zA-Z0-9_-]+$")
            .unwrap()
            .is_match(name)
        {
            return Err(DomainError::InvalidProjectName {
                reason: "Project name can only contain letters, numbers, dashes, and underscores"
                    .to_string(),
            });
        }

        Ok(())
    }

    /// Validate project path
    fn validate_path(path: &str) -> DomainResult<()> {
        if path.trim().is_empty() {
            return Err(DomainError::InvalidFilePath {
                reason: "Project path cannot be empty".to_string(),
            });
        }

        // Basic path validation - could be enhanced
        if path.contains("..") {
            return Err(DomainError::InvalidFilePath {
                reason: "Project path cannot contain '..'".to_string(),
            });
        }

        Ok(())
    }
}
