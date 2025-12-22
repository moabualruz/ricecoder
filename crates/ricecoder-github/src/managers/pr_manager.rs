//! PR Manager - Handles pull request creation and management

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::{
    errors::{GitHubError, Result},
    models::{FileChange, PrStatus, PullRequest},
};

/// PR template configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrTemplate {
    /// Template title (supports placeholders like {{title}}, {{issue_number}})
    pub title_template: String,
    /// Template body (supports placeholders)
    pub body_template: String,
}

impl Default for PrTemplate {
    fn default() -> Self {
        Self {
            title_template: "{{title}}".to_string(),
            body_template:
                "## Description\n\n{{description}}\n\n## Related Issues\n\n{{related_issues}}"
                    .to_string(),
        }
    }
}

/// Task context for PR creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContext {
    /// Task title
    pub title: String,
    /// Task description
    pub description: String,
    /// Related issue numbers
    pub related_issues: Vec<u32>,
    /// Files changed
    pub files: Vec<FileChange>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl TaskContext {
    /// Create a new task context
    pub fn new(title: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: description.into(),
            related_issues: Vec::new(),
            files: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a related issue
    pub fn with_issue(mut self, issue_number: u32) -> Self {
        self.related_issues.push(issue_number);
        self
    }

    /// Add related issues
    pub fn with_issues(mut self, issues: Vec<u32>) -> Self {
        self.related_issues.extend(issues);
        self
    }

    /// Add a file change
    pub fn with_file(mut self, file: FileChange) -> Self {
        self.files.push(file);
        self
    }

    /// Add files
    pub fn with_files(mut self, files: Vec<FileChange>) -> Self {
        self.files.extend(files);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// PR creation options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrOptions {
    /// Branch name for the PR
    pub branch: String,
    /// Base branch (default: main)
    pub base_branch: String,
    /// Is draft PR
    pub draft: bool,
    /// PR template to use
    pub template: Option<PrTemplate>,
}

impl Default for PrOptions {
    fn default() -> Self {
        Self {
            branch: "feature/auto-pr".to_string(),
            base_branch: "main".to_string(),
            draft: false,
            template: None,
        }
    }
}

impl PrOptions {
    /// Create new PR options
    pub fn new(branch: impl Into<String>) -> Self {
        Self {
            branch: branch.into(),
            ..Default::default()
        }
    }

    /// Set as draft
    pub fn as_draft(mut self) -> Self {
        self.draft = true;
        self
    }

    /// Set base branch
    pub fn with_base_branch(mut self, base: impl Into<String>) -> Self {
        self.base_branch = base.into();
        self
    }

    /// Set template
    pub fn with_template(mut self, template: PrTemplate) -> Self {
        self.template = Some(template);
        self
    }
}

/// PR Manager - Handles pull request creation and management
pub struct PrManager {
    /// Default PR template
    default_template: PrTemplate,
}

impl PrManager {
    /// Create a new PR manager
    pub fn new() -> Self {
        Self {
            default_template: PrTemplate::default(),
        }
    }

    /// Create a new PR manager with custom template
    pub fn with_template(template: PrTemplate) -> Self {
        Self {
            default_template: template,
        }
    }

    /// Generate PR title from task context
    pub fn generate_title(
        &self,
        context: &TaskContext,
        template: Option<&PrTemplate>,
    ) -> Result<String> {
        let template = template.unwrap_or(&self.default_template);
        let title = self.apply_template(&template.title_template, context)?;

        if title.is_empty() {
            return Err(GitHubError::invalid_input("Generated PR title is empty"));
        }

        Ok(title)
    }

    /// Generate PR body from task context
    pub fn generate_body(
        &self,
        context: &TaskContext,
        template: Option<&PrTemplate>,
    ) -> Result<String> {
        let template = template.unwrap_or(&self.default_template);
        let body = self.apply_template(&template.body_template, context)?;

        if body.is_empty() {
            return Err(GitHubError::invalid_input("Generated PR body is empty"));
        }

        Ok(body)
    }

    /// Apply template with context variables
    fn apply_template(&self, template: &str, context: &TaskContext) -> Result<String> {
        let mut result = template.to_string();

        // Replace basic placeholders
        result = result.replace("{{title}}", &context.title);
        result = result.replace("{{description}}", &context.description);

        // Replace related issues
        let issues_str = if context.related_issues.is_empty() {
            "None".to_string()
        } else {
            context
                .related_issues
                .iter()
                .map(|n| format!("Closes #{}", n))
                .collect::<Vec<_>>()
                .join("\n")
        };
        result = result.replace("{{related_issues}}", &issues_str);

        // Replace files summary
        let files_summary = if context.files.is_empty() {
            "No files changed".to_string()
        } else {
            format!(
                "{} files changed: {}",
                context.files.len(),
                context
                    .files
                    .iter()
                    .map(|f| f.path.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        result = result.replace("{{files_summary}}", &files_summary);

        // Replace metadata placeholders
        for (key, value) in &context.metadata {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        Ok(result)
    }

    /// Validate PR creation inputs
    pub fn validate_pr_creation(&self, context: &TaskContext, options: &PrOptions) -> Result<()> {
        // Validate context
        if context.title.is_empty() {
            return Err(GitHubError::invalid_input("PR title cannot be empty"));
        }

        if context.title.len() > 256 {
            return Err(GitHubError::invalid_input(
                "PR title cannot exceed 256 characters",
            ));
        }

        // Validate options
        if options.branch.is_empty() {
            return Err(GitHubError::invalid_input("Branch name cannot be empty"));
        }

        if options.base_branch.is_empty() {
            return Err(GitHubError::invalid_input("Base branch cannot be empty"));
        }

        if options.branch == options.base_branch {
            return Err(GitHubError::invalid_input(
                "Branch and base branch cannot be the same",
            ));
        }

        Ok(())
    }

    /// Create a PR from task context
    pub fn create_pr_from_context(
        &self,
        context: TaskContext,
        options: PrOptions,
    ) -> Result<PullRequest> {
        // Validate inputs
        self.validate_pr_creation(&context, &options)?;

        // Generate title and body
        let title = self.generate_title(&context, options.template.as_ref())?;
        let body = self.generate_body(&context, options.template.as_ref())?;

        debug!(
            title = %title,
            branch = %options.branch,
            draft = options.draft,
            "Creating PR from context"
        );

        // Create PR object
        let pr = PullRequest {
            id: 0,     // Will be assigned by GitHub
            number: 0, // Will be assigned by GitHub
            title,
            body,
            branch: options.branch,
            base: options.base_branch,
            status: if options.draft {
                PrStatus::Draft
            } else {
                PrStatus::Open
            },
            files: context.files,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        info!(
            pr_title = %pr.title,
            pr_branch = %pr.branch,
            "PR created from context"
        );

        Ok(pr)
    }

    /// Link PR to related issues
    pub fn link_to_issues(&self, pr: &mut PullRequest, issue_numbers: Vec<u32>) -> Result<()> {
        if issue_numbers.is_empty() {
            return Ok(());
        }

        debug!(issue_count = issue_numbers.len(), "Linking PR to issues");

        // Add close keywords to PR body
        let close_keywords = issue_numbers
            .iter()
            .map(|n| format!("Closes #{}", n))
            .collect::<Vec<_>>()
            .join("\n");

        if !pr.body.contains("Closes #") {
            pr.body.push_str("\n\n");
            pr.body.push_str(&close_keywords);
        }

        info!(issue_count = issue_numbers.len(), "PR linked to issues");

        Ok(())
    }

    /// Validate PR content
    pub fn validate_pr_content(&self, pr: &PullRequest) -> Result<()> {
        if pr.title.is_empty() {
            return Err(GitHubError::invalid_input("PR title cannot be empty"));
        }

        if pr.body.is_empty() {
            return Err(GitHubError::invalid_input("PR body cannot be empty"));
        }

        if pr.branch.is_empty() {
            return Err(GitHubError::invalid_input("PR branch cannot be empty"));
        }

        if pr.base.is_empty() {
            return Err(GitHubError::invalid_input("PR base branch cannot be empty"));
        }

        Ok(())
    }
}

impl Default for PrManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_context_creation() {
        let context = TaskContext::new("Test PR", "This is a test PR");
        assert_eq!(context.title, "Test PR");
        assert_eq!(context.description, "This is a test PR");
        assert!(context.related_issues.is_empty());
        assert!(context.files.is_empty());
    }

    #[test]
    fn test_task_context_with_issues() {
        let context = TaskContext::new("Test PR", "Description")
            .with_issue(123)
            .with_issue(456);
        assert_eq!(context.related_issues.len(), 2);
        assert!(context.related_issues.contains(&123));
        assert!(context.related_issues.contains(&456));
    }

    #[test]
    fn test_pr_options_creation() {
        let options = PrOptions::new("feature/test");
        assert_eq!(options.branch, "feature/test");
        assert_eq!(options.base_branch, "main");
        assert!(!options.draft);
    }

    #[test]
    fn test_pr_options_as_draft() {
        let options = PrOptions::new("feature/test").as_draft();
        assert!(options.draft);
    }

    #[test]
    fn test_pr_manager_creation() {
        let manager = PrManager::new();
        assert_eq!(manager.default_template.title_template, "{{title}}");
    }

    #[test]
    fn test_generate_title_simple() {
        let manager = PrManager::new();
        let context = TaskContext::new("Fix bug", "Fixed a critical bug");
        let title = manager.generate_title(&context, None).unwrap();
        assert_eq!(title, "Fix bug");
    }

    #[test]
    fn test_generate_title_empty() {
        let manager = PrManager::new();
        let context = TaskContext::new("", "Description");
        let result = manager.generate_title(&context, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_body_with_issues() {
        let manager = PrManager::new();
        let context = TaskContext::new("Test", "Description").with_issue(123);
        let body = manager.generate_body(&context, None).unwrap();
        assert!(body.contains("Closes #123"));
    }

    #[test]
    fn test_generate_body_no_issues() {
        let manager = PrManager::new();
        let context = TaskContext::new("Test", "Description");
        let body = manager.generate_body(&context, None).unwrap();
        assert!(body.contains("None"));
    }

    #[test]
    fn test_apply_template_with_metadata() {
        let manager = PrManager::new();
        let context =
            TaskContext::new("Test", "Description").with_metadata("custom_field", "custom_value");
        let template = "Title: {{title}}, Custom: {{custom_field}}";
        let result = manager.apply_template(template, &context).unwrap();
        assert_eq!(result, "Title: Test, Custom: custom_value");
    }

    #[test]
    fn test_validate_pr_creation_success() {
        let manager = PrManager::new();
        let context = TaskContext::new("Test PR", "Description");
        let options = PrOptions::new("feature/test");
        assert!(manager.validate_pr_creation(&context, &options).is_ok());
    }

    #[test]
    fn test_validate_pr_creation_empty_title() {
        let manager = PrManager::new();
        let context = TaskContext::new("", "Description");
        let options = PrOptions::new("feature/test");
        assert!(manager.validate_pr_creation(&context, &options).is_err());
    }

    #[test]
    fn test_validate_pr_creation_same_branch() {
        let manager = PrManager::new();
        let context = TaskContext::new("Test", "Description");
        let options = PrOptions::new("main").with_base_branch("main");
        assert!(manager.validate_pr_creation(&context, &options).is_err());
    }

    #[test]
    fn test_create_pr_from_context() {
        let manager = PrManager::new();
        let context = TaskContext::new("Test PR", "This is a test").with_issue(123);
        let options = PrOptions::new("feature/test");
        let pr = manager.create_pr_from_context(context, options).unwrap();
        assert_eq!(pr.title, "Test PR");
        assert_eq!(pr.branch, "feature/test");
        assert_eq!(pr.base, "main");
        assert!(!pr.body.is_empty());
    }

    #[test]
    fn test_create_pr_draft() {
        let manager = PrManager::new();
        let context = TaskContext::new("Test PR", "Description");
        let options = PrOptions::new("feature/test").as_draft();
        let pr = manager.create_pr_from_context(context, options).unwrap();
        assert_eq!(pr.status, PrStatus::Draft);
    }

    #[test]
    fn test_link_to_issues() {
        let manager = PrManager::new();
        let context = TaskContext::new("Test", "Description");
        let options = PrOptions::new("feature/test");
        let mut pr = manager.create_pr_from_context(context, options).unwrap();
        let original_body = pr.body.clone();
        manager.link_to_issues(&mut pr, vec![123, 456]).unwrap();
        assert!(pr.body.contains("Closes #123"));
        assert!(pr.body.contains("Closes #456"));
        assert!(pr.body.len() > original_body.len());
    }

    #[test]
    fn test_validate_pr_content_success() {
        let manager = PrManager::new();
        let context = TaskContext::new("Test", "Description");
        let options = PrOptions::new("feature/test");
        let pr = manager.create_pr_from_context(context, options).unwrap();
        assert!(manager.validate_pr_content(&pr).is_ok());
    }

    #[test]
    fn test_validate_pr_content_empty_title() {
        let pr = PullRequest {
            id: 0,
            number: 0,
            title: String::new(),
            body: "Body".to_string(),
            branch: "feature/test".to_string(),
            base: "main".to_string(),
            status: PrStatus::Open,
            files: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let manager = PrManager::new();
        assert!(manager.validate_pr_content(&pr).is_err());
    }

    #[test]
    fn test_custom_template() {
        let template = PrTemplate {
            title_template: "CUSTOM: {{title}}".to_string(),
            body_template: "CUSTOM BODY: {{description}}".to_string(),
        };
        let manager = PrManager::with_template(template);
        let context = TaskContext::new("Test", "Description");
        let title = manager.generate_title(&context, None).unwrap();
        assert_eq!(title, "CUSTOM: Test");
    }
}
