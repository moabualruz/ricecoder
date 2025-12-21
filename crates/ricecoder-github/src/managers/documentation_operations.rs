//! Documentation Operations - Publishing and maintenance operations

use crate::errors::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// Documentation commit information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationCommit {
    /// Commit message
    pub message: String,
    /// Files changed
    pub files: Vec<String>,
    /// Commit hash (after creation)
    pub hash: Option<String>,
    /// Commit timestamp
    pub timestamp: Option<String>,
}

impl DocumentationCommit {
    /// Create a new documentation commit
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            files: Vec::new(),
            hash: None,
            timestamp: None,
        }
    }

    /// Add a file to the commit
    pub fn with_file(mut self, file: impl Into<String>) -> Self {
        self.files.push(file.into());
        self
    }

    /// Add files to the commit
    pub fn with_files(mut self, files: Vec<String>) -> Self {
        self.files.extend(files);
        self
    }

    /// Set commit hash
    pub fn with_hash(mut self, hash: impl Into<String>) -> Self {
        self.hash = Some(hash.into());
        self
    }

    /// Set commit timestamp
    pub fn with_timestamp(mut self, timestamp: impl Into<String>) -> Self {
        self.timestamp = Some(timestamp.into());
        self
    }
}

/// Documentation template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationTemplate {
    /// Template name
    pub name: String,
    /// Template content
    pub content: String,
    /// Template variables (placeholders)
    pub variables: Vec<String>,
}

impl DocumentationTemplate {
    /// Create a new documentation template
    pub fn new(name: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            content: content.into(),
            variables: Vec::new(),
        }
    }

    /// Add a variable
    pub fn with_variable(mut self, var: impl Into<String>) -> Self {
        self.variables.push(var.into());
        self
    }

    /// Render template with variables
    pub fn render(&self, values: &HashMap<String, String>) -> Result<String> {
        let mut result = self.content.clone();

        for (key, value) in values {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        Ok(result)
    }
}

/// Documentation maintenance task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceTask {
    /// Task name
    pub name: String,
    /// Task description
    pub description: String,
    /// Files affected
    pub files: Vec<String>,
    /// Task status
    pub status: MaintenanceStatus,
    /// Completion percentage (0-100)
    pub progress: u32,
}

impl MaintenanceTask {
    /// Create a new maintenance task
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            files: Vec::new(),
            status: MaintenanceStatus::Pending,
            progress: 0,
        }
    }

    /// Add a file
    pub fn with_file(mut self, file: impl Into<String>) -> Self {
        self.files.push(file.into());
        self
    }

    /// Set status
    pub fn with_status(mut self, status: MaintenanceStatus) -> Self {
        self.status = status;
        self
    }

    /// Set progress
    pub fn with_progress(mut self, progress: u32) -> Self {
        self.progress = progress.min(100);
        self
    }
}

/// Maintenance task status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MaintenanceStatus {
    /// Task is pending
    Pending,
    /// Task is in progress
    InProgress,
    /// Task is completed
    Completed,
    /// Task failed
    Failed,
}

/// Documentation publishing result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishingResult {
    /// Publishing successful
    pub success: bool,
    /// Commit hash
    pub commit_hash: Option<String>,
    /// Files published
    pub files_published: Vec<String>,
    /// Error message if failed
    pub error: Option<String>,
}

impl PublishingResult {
    /// Create a successful publishing result
    pub fn success(commit_hash: impl Into<String>) -> Self {
        Self {
            success: true,
            commit_hash: Some(commit_hash.into()),
            files_published: Vec::new(),
            error: None,
        }
    }

    /// Create a failed publishing result
    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            success: false,
            commit_hash: None,
            files_published: Vec::new(),
            error: Some(error.into()),
        }
    }

    /// Add a published file
    pub fn with_file(mut self, file: impl Into<String>) -> Self {
        self.files_published.push(file.into());
        self
    }
}

/// Documentation Operations
#[derive(Debug, Clone)]
pub struct DocumentationOperations {
    /// Documentation templates
    pub templates: HashMap<String, DocumentationTemplate>,
    /// Maintenance tasks
    pub maintenance_tasks: HashMap<String, MaintenanceTask>,
    /// Commit history
    pub commit_history: Vec<DocumentationCommit>,
}

impl DocumentationOperations {
    /// Create new documentation operations
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
            maintenance_tasks: HashMap::new(),
            commit_history: Vec::new(),
        }
    }

    /// Commit documentation updates to repository
    pub fn commit_documentation(
        &mut self,
        commit: DocumentationCommit,
    ) -> Result<PublishingResult> {
        debug!("Committing documentation: {}", commit.message);

        if commit.files.is_empty() {
            return Ok(PublishingResult::failure("No files to commit"));
        }

        // Simulate commit creation
        let commit_hash = format!("commit_{}", self.commit_history.len() + 1);
        let mut result = PublishingResult::success(&commit_hash);

        for file in &commit.files {
            result = result.with_file(file.clone());
        }

        // Store in history
        let mut stored_commit = commit;
        stored_commit.hash = Some(commit_hash);
        self.commit_history.push(stored_commit);

        info!("Documentation committed successfully");
        Ok(result)
    }

    /// Track documentation coverage and gaps
    pub fn track_coverage(&mut self, task: MaintenanceTask) -> Result<()> {
        debug!("Tracking documentation coverage: {}", task.name);

        self.maintenance_tasks.insert(task.name.clone(), task);

        info!("Documentation coverage tracked");
        Ok(())
    }

    /// Support documentation templates
    pub fn add_template(&mut self, template: DocumentationTemplate) -> Result<()> {
        debug!("Adding documentation template: {}", template.name);

        self.templates.insert(template.name.clone(), template);

        info!("Documentation template added");
        Ok(())
    }

    /// Get template by name
    pub fn get_template(&self, name: &str) -> Option<&DocumentationTemplate> {
        self.templates.get(name)
    }

    /// Render template with values
    pub fn render_template(&self, name: &str, values: &HashMap<String, String>) -> Result<String> {
        debug!("Rendering template: {}", name);

        let template = self.templates.get(name).ok_or_else(|| {
            crate::errors::GitHubError::NotFound(format!("Template '{}' not found", name))
        })?;

        template.render(values)
    }

    /// Get all maintenance tasks
    pub fn get_maintenance_tasks(&self) -> Vec<&MaintenanceTask> {
        self.maintenance_tasks.values().collect()
    }

    /// Get maintenance task by name
    pub fn get_maintenance_task(&self, name: &str) -> Option<&MaintenanceTask> {
        self.maintenance_tasks.get(name)
    }

    /// Update maintenance task status
    pub fn update_task_status(&mut self, name: &str, status: MaintenanceStatus) -> Result<()> {
        debug!("Updating maintenance task status: {} -> {:?}", name, status);

        if let Some(task) = self.maintenance_tasks.get_mut(name) {
            task.status = status;
            info!("Maintenance task status updated");
            Ok(())
        } else {
            Err(crate::errors::GitHubError::NotFound(format!(
                "Task '{}' not found",
                name
            )))
        }
    }

    /// Get commit history
    pub fn get_commit_history(&self) -> &[DocumentationCommit] {
        &self.commit_history
    }

    /// Get latest commit
    pub fn get_latest_commit(&self) -> Option<&DocumentationCommit> {
        self.commit_history.last()
    }
}

impl Default for DocumentationOperations {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_documentation_commit_builder() {
        let commit = DocumentationCommit::new("Update docs")
            .with_file("README.md")
            .with_file("API.md")
            .with_hash("abc123")
            .with_timestamp("2025-01-01T00:00:00Z");

        assert_eq!(commit.message, "Update docs");
        assert_eq!(commit.files.len(), 2);
        assert_eq!(commit.hash, Some("abc123".to_string()));
    }

    #[test]
    fn test_documentation_template_rendering() {
        let mut values = HashMap::new();
        values.insert("project".to_string(), "MyProject".to_string());
        values.insert("version".to_string(), "1.0.0".to_string());

        let template =
            DocumentationTemplate::new("readme", "# {{project}}\n\nVersion: {{version}}");

        let rendered = template.render(&values).expect("Failed to render");
        assert!(rendered.contains("# MyProject"));
        assert!(rendered.contains("Version: 1.0.0"));
    }

    #[test]
    fn test_maintenance_task_builder() {
        let task = MaintenanceTask::new("Update API docs", "Update API documentation")
            .with_file("API.md")
            .with_status(MaintenanceStatus::InProgress)
            .with_progress(50);

        assert_eq!(task.name, "Update API docs");
        assert_eq!(task.status, MaintenanceStatus::InProgress);
        assert_eq!(task.progress, 50);
    }

    #[test]
    fn test_documentation_operations_commit() {
        let mut ops = DocumentationOperations::new();
        let commit = DocumentationCommit::new("Initial commit").with_file("README.md");

        let result = ops.commit_documentation(commit).expect("Failed to commit");
        assert!(result.success);
        assert!(result.commit_hash.is_some());
        assert_eq!(ops.commit_history.len(), 1);
    }

    #[test]
    fn test_documentation_operations_template() {
        let mut ops = DocumentationOperations::new();
        let template = DocumentationTemplate::new("test", "Hello {{name}}");

        ops.add_template(template).expect("Failed to add template");
        assert!(ops.get_template("test").is_some());
    }

    #[test]
    fn test_maintenance_task_tracking() {
        let mut ops = DocumentationOperations::new();
        let task = MaintenanceTask::new("Task 1", "Description");

        ops.track_coverage(task).expect("Failed to track");
        assert_eq!(ops.maintenance_tasks.len(), 1);
    }

    #[test]
    fn test_maintenance_status_update() {
        let mut ops = DocumentationOperations::new();
        let task = MaintenanceTask::new("Task 1", "Description");

        ops.track_coverage(task).expect("Failed to track");
        ops.update_task_status("Task 1", MaintenanceStatus::Completed)
            .expect("Failed to update");

        let updated = ops.get_maintenance_task("Task 1").unwrap();
        assert_eq!(updated.status, MaintenanceStatus::Completed);
    }

    #[test]
    fn test_publishing_result_builder() {
        let result = PublishingResult::success("abc123")
            .with_file("README.md")
            .with_file("API.md");

        assert!(result.success);
        assert_eq!(result.files_published.len(), 2);
    }

    #[test]
    fn test_publishing_result_failure() {
        let result = PublishingResult::failure("Something went wrong");

        assert!(!result.success);
        assert!(result.error.is_some());
    }
}
