//! CodeFile entity representing a source code file

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{errors::*, value_objects::*};

/// File entity representing a source code file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeFile {
    pub id: FileId,
    pub project_id: ProjectId,
    pub relative_path: String,
    pub language: ProgrammingLanguage,
    pub content: String,
    pub size_bytes: usize,
    pub mime_type: MimeType,
    pub last_modified: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

impl CodeFile {
    /// Create a new code file
    pub fn new(
        project_id: ProjectId,
        relative_path: String,
        content: String,
        language: ProgrammingLanguage,
    ) -> DomainResult<Self> {
        let id = FileId::from_path(&relative_path);

        Ok(Self {
            id,
            project_id,
            relative_path: relative_path.clone(),
            language,
            content: content.clone(),
            size_bytes: content.len(),
            mime_type: MimeType::from_path(&relative_path),
            last_modified: Utc::now(),
            metadata: HashMap::new(),
        })
    }

    /// Update file content
    pub fn update_content(&mut self, content: String) {
        self.content = content.clone();
        self.size_bytes = content.len();
        self.last_modified = Utc::now();
    }

    /// Check if file is empty
    pub fn is_empty(&self) -> bool {
        self.content.trim().is_empty()
    }

    /// Get file extension
    pub fn extension(&self) -> Option<&str> {
        std::path::Path::new(&self.relative_path)
            .extension()
            .and_then(|ext| ext.to_str())
    }

    /// Returns the number of lines in the file
    pub fn line_count(&self) -> usize {
        self.content.lines().count()
    }

    /// Add or update a metadata entry
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
        self.last_modified = Utc::now();
    }

    /// Get a specific metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    /// Remove a metadata entry
    pub fn remove_metadata(&mut self, key: &str) -> Option<String> {
        let result = self.metadata.remove(key);
        if result.is_some() {
            self.last_modified = Utc::now();
        }
        result
    }

    /// Check if this is a test file based on path or metadata
    pub fn is_test_file(&self) -> bool {
        self.relative_path.contains("test")
            || self.relative_path.contains("spec")
            || self.relative_path.ends_with("_test.rs")
            || self.relative_path.ends_with(".test.ts")
            || self.relative_path.ends_with(".spec.ts")
            || self.metadata.get("is_test").map(|v| v == "true").unwrap_or(false)
    }

    /// Returns the number of characters in the file
    pub fn char_count(&self) -> usize {
        self.content.chars().count()
    }
}
