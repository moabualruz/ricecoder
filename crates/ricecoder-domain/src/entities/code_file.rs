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
}
