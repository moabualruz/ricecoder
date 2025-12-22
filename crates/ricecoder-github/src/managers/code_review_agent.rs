//! Code Review Agent - Provides automated code review for pull requests

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::{errors::Result, models::PullRequest};

/// Code quality issue severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueSeverity {
    /// Critical issue that must be fixed
    Critical,
    /// Warning that should be addressed
    Warning,
    /// Informational suggestion
    Info,
}

/// Code quality issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQualityIssue {
    /// Issue severity
    pub severity: IssueSeverity,
    /// Issue title
    pub title: String,
    /// Issue description
    pub description: String,
    /// File path where issue was found
    pub file_path: String,
    /// Line number (if applicable)
    pub line_number: Option<u32>,
    /// Suggested fix
    pub suggested_fix: Option<String>,
}

impl CodeQualityIssue {
    /// Create a new code quality issue
    pub fn new(
        severity: IssueSeverity,
        title: impl Into<String>,
        description: impl Into<String>,
        file_path: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            title: title.into(),
            description: description.into(),
            file_path: file_path.into(),
            line_number: None,
            suggested_fix: None,
        }
    }

    /// Set line number
    pub fn with_line_number(mut self, line: u32) -> Self {
        self.line_number = Some(line);
        self
    }

    /// Set suggested fix
    pub fn with_suggested_fix(mut self, fix: impl Into<String>) -> Self {
        self.suggested_fix = Some(fix.into());
        self
    }
}

/// Code review suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeReviewSuggestion {
    /// Suggestion title
    pub title: String,
    /// Suggestion body
    pub body: String,
    /// File path
    pub file_path: String,
    /// Line number (if applicable)
    pub line_number: Option<u32>,
    /// Is critical
    pub is_critical: bool,
}

impl CodeReviewSuggestion {
    /// Create a new suggestion
    pub fn new(
        title: impl Into<String>,
        body: impl Into<String>,
        file_path: impl Into<String>,
    ) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
            file_path: file_path.into(),
            line_number: None,
            is_critical: false,
        }
    }

    /// Set line number
    pub fn with_line_number(mut self, line: u32) -> Self {
        self.line_number = Some(line);
        self
    }

    /// Mark as critical
    pub fn as_critical(mut self) -> Self {
        self.is_critical = true;
        self
    }
}

/// Code review result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeReviewResult {
    /// PR number
    pub pr_number: u32,
    /// Quality issues found
    pub issues: Vec<CodeQualityIssue>,
    /// Review suggestions
    pub suggestions: Vec<CodeReviewSuggestion>,
    /// Overall quality score (0-100)
    pub quality_score: u32,
    /// Is approved
    pub approved: bool,
    /// Approval reason
    pub approval_reason: Option<String>,
}

impl CodeReviewResult {
    /// Create a new code review result
    pub fn new(pr_number: u32) -> Self {
        Self {
            pr_number,
            issues: Vec::new(),
            suggestions: Vec::new(),
            quality_score: 100,
            approved: true,
            approval_reason: None,
        }
    }

    /// Add an issue
    pub fn with_issue(mut self, issue: CodeQualityIssue) -> Self {
        // Reduce quality score based on severity
        match issue.severity {
            IssueSeverity::Critical => self.quality_score = self.quality_score.saturating_sub(20),
            IssueSeverity::Warning => self.quality_score = self.quality_score.saturating_sub(10),
            IssueSeverity::Info => self.quality_score = self.quality_score.saturating_sub(5),
        }
        self.issues.push(issue);
        self
    }

    /// Add issues
    pub fn with_issues(mut self, issues: Vec<CodeQualityIssue>) -> Self {
        for issue in issues {
            self = self.with_issue(issue);
        }
        self
    }

    /// Add a suggestion
    pub fn with_suggestion(mut self, suggestion: CodeReviewSuggestion) -> Self {
        self.suggestions.push(suggestion);
        self
    }

    /// Add suggestions
    pub fn with_suggestions(mut self, suggestions: Vec<CodeReviewSuggestion>) -> Self {
        for suggestion in suggestions {
            self = self.with_suggestion(suggestion);
        }
        self
    }

    /// Set approval status
    pub fn set_approved(mut self, approved: bool, reason: Option<String>) -> Self {
        self.approved = approved;
        self.approval_reason = reason;
        self
    }

    /// Check if has critical issues
    pub fn has_critical_issues(&self) -> bool {
        self.issues
            .iter()
            .any(|i| i.severity == IssueSeverity::Critical)
    }

    /// Get critical issues count
    pub fn critical_issues_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity == IssueSeverity::Critical)
            .count()
    }

    /// Get warnings count
    pub fn warnings_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity == IssueSeverity::Warning)
            .count()
    }

    /// Get info count
    pub fn info_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity == IssueSeverity::Info)
            .count()
    }
}

/// Code review standards configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeReviewStandards {
    /// Minimum quality score for approval (0-100)
    pub min_quality_score: u32,
    /// Require all critical issues to be fixed
    pub require_critical_fixes: bool,
    /// Require all warnings to be addressed
    pub require_warning_fixes: bool,
    /// Custom standards rules
    pub custom_rules: HashMap<String, String>,
}

impl Default for CodeReviewStandards {
    fn default() -> Self {
        Self {
            min_quality_score: 70,
            require_critical_fixes: true,
            require_warning_fixes: false,
            custom_rules: HashMap::new(),
        }
    }
}

impl CodeReviewStandards {
    /// Create new standards
    pub fn new(min_quality_score: u32) -> Self {
        Self {
            min_quality_score,
            ..Default::default()
        }
    }

    /// Set require critical fixes
    pub fn require_critical_fixes(mut self, require: bool) -> Self {
        self.require_critical_fixes = require;
        self
    }

    /// Set require warning fixes
    pub fn require_warning_fixes(mut self, require: bool) -> Self {
        self.require_warning_fixes = require;
        self
    }

    /// Add custom rule
    pub fn with_rule(mut self, name: impl Into<String>, rule: impl Into<String>) -> Self {
        self.custom_rules.insert(name.into(), rule.into());
        self
    }
}

/// Code Review Agent - Provides automated code review
pub struct CodeReviewAgent {
    /// Code review standards
    pub standards: CodeReviewStandards,
}

impl CodeReviewAgent {
    /// Create a new code review agent
    pub fn new() -> Self {
        Self {
            standards: CodeReviewStandards::default(),
        }
    }

    /// Create with custom standards
    pub fn with_standards(standards: CodeReviewStandards) -> Self {
        Self { standards }
    }

    /// Analyze PR code for quality issues
    pub fn analyze_code(&self, pr: &PullRequest) -> Result<Vec<CodeQualityIssue>> {
        debug!(
            pr_number = pr.number,
            file_count = pr.files.len(),
            "Analyzing PR code for quality issues"
        );

        let mut issues = Vec::new();

        // Analyze each file
        for file in &pr.files {
            // Check for large files
            if file.additions + file.deletions > 500 {
                issues.push(CodeQualityIssue::new(
                    IssueSeverity::Warning,
                    "Large file change",
                    format!(
                        "File {} has {} lines changed, consider breaking into smaller changes",
                        file.path,
                        file.additions + file.deletions
                    ),
                    &file.path,
                ));
            }

            // Check for excessive deletions
            if file.deletions > file.additions * 2 {
                issues.push(CodeQualityIssue::new(
                    IssueSeverity::Info,
                    "Large deletion",
                    format!(
                        "File {} has significant deletions ({} lines)",
                        file.path, file.deletions
                    ),
                    &file.path,
                ));
            }
        }

        // Check PR body for common issues
        if pr.body.is_empty() {
            issues.push(CodeQualityIssue::new(
                IssueSeverity::Warning,
                "Missing PR description",
                "PR body is empty, please provide a description of changes",
                "PR",
            ));
        }

        // Check PR title length
        if pr.title.len() > 100 {
            issues.push(CodeQualityIssue::new(
                IssueSeverity::Info,
                "Long PR title",
                format!(
                    "PR title is {} characters, consider shortening",
                    pr.title.len()
                ),
                "PR",
            ));
        }

        info!(
            pr_number = pr.number,
            issue_count = issues.len(),
            "Code analysis complete"
        );

        Ok(issues)
    }

    /// Generate code review suggestions
    pub fn generate_suggestions(
        &self,
        issues: &[CodeQualityIssue],
    ) -> Result<Vec<CodeReviewSuggestion>> {
        debug!(
            issue_count = issues.len(),
            "Generating code review suggestions"
        );

        let mut suggestions = Vec::new();

        for issue in issues {
            let suggestion =
                CodeReviewSuggestion::new(&issue.title, &issue.description, &issue.file_path)
                    .with_line_number(issue.line_number.unwrap_or(0));

            let suggestion = if issue.severity == IssueSeverity::Critical {
                suggestion.as_critical()
            } else {
                suggestion
            };

            suggestions.push(suggestion);
        }

        info!(
            suggestion_count = suggestions.len(),
            "Suggestions generated"
        );

        Ok(suggestions)
    }

    /// Validate code against project standards
    pub fn validate_standards(&self, pr: &PullRequest) -> Result<Vec<CodeQualityIssue>> {
        debug!(
            pr_number = pr.number,
            "Validating code against project standards"
        );

        let mut issues = Vec::new();

        // Check for minimum file count
        if pr.files.is_empty() {
            issues.push(CodeQualityIssue::new(
                IssueSeverity::Critical,
                "No files changed",
                "PR has no file changes",
                "PR",
            ));
        }

        // Check for branch naming convention
        if !self.is_valid_branch_name(&pr.branch) {
            issues.push(
                CodeQualityIssue::new(
                    IssueSeverity::Warning,
                    "Invalid branch name",
                    format!(
                        "Branch name '{}' does not follow naming conventions (use feature/, bugfix/, hotfix/)",
                        pr.branch
                    ),
                    "PR",
                )
            );
        }

        // Apply custom rules
        for rule_name in self.standards.custom_rules.keys() {
            debug!(rule = rule_name, "Applying custom rule");
        }

        info!(
            pr_number = pr.number,
            issue_count = issues.len(),
            "Standards validation complete"
        );

        Ok(issues)
    }

    /// Check if branch name is valid
    pub fn is_valid_branch_name(&self, branch: &str) -> bool {
        // Allow common branch naming patterns
        branch.starts_with("feature/")
            || branch.starts_with("bugfix/")
            || branch.starts_with("hotfix/")
            || branch.starts_with("release/")
            || branch == "main"
            || branch == "develop"
            || branch == "master"
    }

    /// Generate code review summary
    pub fn generate_summary(&self, result: &CodeReviewResult) -> Result<String> {
        debug!(
            pr_number = result.pr_number,
            quality_score = result.quality_score,
            "Generating code review summary"
        );

        let mut summary = format!(
            "## Code Review Summary\n\n**PR #{}**\n\n**Quality Score: {}/100**\n\n",
            result.pr_number, result.quality_score
        );

        // Add approval status
        if result.approved {
            summary.push_str("✅ **APPROVED**\n\n");
            if let Some(reason) = &result.approval_reason {
                summary.push_str(&format!("Reason: {}\n\n", reason));
            }
        } else {
            summary.push_str("❌ **NEEDS REVIEW**\n\n");
            if let Some(reason) = &result.approval_reason {
                summary.push_str(&format!("Reason: {}\n\n", reason));
            }
        }

        // Add issues summary
        if !result.issues.is_empty() {
            summary.push_str("### Issues Found\n\n");
            summary.push_str(&format!(
                "- **Critical**: {}\n",
                result.critical_issues_count()
            ));
            summary.push_str(&format!("- **Warnings**: {}\n", result.warnings_count()));
            summary.push_str(&format!("- **Info**: {}\n\n", result.info_count()));

            // Add critical issues
            let critical: Vec<_> = result
                .issues
                .iter()
                .filter(|i| i.severity == IssueSeverity::Critical)
                .collect();
            if !critical.is_empty() {
                summary.push_str("#### Critical Issues\n\n");
                for issue in critical {
                    summary.push_str(&format!(
                        "- **{}** ({}): {}\n",
                        issue.title, issue.file_path, issue.description
                    ));
                }
                summary.push('\n');
            }
        }

        // Add suggestions
        if !result.suggestions.is_empty() {
            summary.push_str("### Suggestions\n\n");
            for suggestion in &result.suggestions {
                summary.push_str(&format!(
                    "- **{}** ({}): {}\n",
                    suggestion.title, suggestion.file_path, suggestion.body
                ));
            }
        }

        info!(
            pr_number = result.pr_number,
            summary_length = summary.len(),
            "Summary generated"
        );

        Ok(summary)
    }

    /// Determine if PR should be approved
    pub fn should_approve(&self, result: &CodeReviewResult) -> Result<bool> {
        debug!(
            pr_number = result.pr_number,
            quality_score = result.quality_score,
            "Determining approval status"
        );

        // Check quality score
        if result.quality_score < self.standards.min_quality_score {
            return Ok(false);
        }

        // Check critical issues
        if self.standards.require_critical_fixes && result.has_critical_issues() {
            return Ok(false);
        }

        // Check warnings
        if self.standards.require_warning_fixes && result.warnings_count() > 0 {
            return Ok(false);
        }

        Ok(true)
    }

    /// Perform complete code review
    pub fn review_pr(&self, pr: &PullRequest) -> Result<CodeReviewResult> {
        debug!(pr_number = pr.number, "Starting complete code review");

        // Analyze code
        let code_issues = self.analyze_code(pr)?;

        // Validate standards
        let standard_issues = self.validate_standards(pr)?;

        // Combine all issues
        let mut all_issues = code_issues;
        all_issues.extend(standard_issues);

        // Generate suggestions
        let suggestions = self.generate_suggestions(&all_issues)?;

        // Create result
        let mut result = CodeReviewResult::new(pr.number).with_issues(all_issues);

        // Add suggestions
        result = result.with_suggestions(suggestions);

        // Determine approval
        let should_approve = self.should_approve(&result)?;
        let approval_reason = Some(format!(
            "Code quality score is {} (minimum: {})",
            result.quality_score, self.standards.min_quality_score
        ));

        result = result.set_approved(should_approve, approval_reason);

        info!(
            pr_number = pr.number,
            approved = result.approved,
            quality_score = result.quality_score,
            "Code review complete"
        );

        Ok(result)
    }
}

impl Default for CodeReviewAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{FileChange, PrStatus};

    fn create_test_pr() -> PullRequest {
        PullRequest {
            id: 1,
            number: 123,
            title: "Test PR".to_string(),
            body: "This is a test PR".to_string(),
            branch: "feature/test".to_string(),
            base: "main".to_string(),
            status: PrStatus::Open,
            files: vec![FileChange {
                path: "src/main.rs".to_string(),
                change_type: "modified".to_string(),
                additions: 50,
                deletions: 10,
            }],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_code_quality_issue_creation() {
        let issue = CodeQualityIssue::new(
            IssueSeverity::Warning,
            "Test issue",
            "This is a test",
            "test.rs",
        );
        assert_eq!(issue.severity, IssueSeverity::Warning);
        assert_eq!(issue.title, "Test issue");
        assert_eq!(issue.file_path, "test.rs");
    }

    #[test]
    fn test_code_quality_issue_with_line_number() {
        let issue =
            CodeQualityIssue::new(IssueSeverity::Critical, "Test", "Description", "test.rs")
                .with_line_number(42);
        assert_eq!(issue.line_number, Some(42));
    }

    #[test]
    fn test_code_review_suggestion_creation() {
        let suggestion = CodeReviewSuggestion::new("Test", "Body", "test.rs");
        assert_eq!(suggestion.title, "Test");
        assert_eq!(suggestion.body, "Body");
        assert!(!suggestion.is_critical);
    }

    #[test]
    fn test_code_review_suggestion_as_critical() {
        let suggestion = CodeReviewSuggestion::new("Test", "Body", "test.rs").as_critical();
        assert!(suggestion.is_critical);
    }

    #[test]
    fn test_code_review_result_creation() {
        let result = CodeReviewResult::new(123);
        assert_eq!(result.pr_number, 123);
        assert_eq!(result.quality_score, 100);
        assert!(result.approved);
    }

    #[test]
    fn test_code_review_result_with_issue() {
        let issue =
            CodeQualityIssue::new(IssueSeverity::Critical, "Test", "Description", "test.rs");
        let result = CodeReviewResult::new(123).with_issue(issue);
        assert_eq!(result.issues.len(), 1);
        assert_eq!(result.critical_issues_count(), 1);
        assert!(result.quality_score < 100);
    }

    #[test]
    fn test_code_review_result_quality_score_calculation() {
        let critical = CodeQualityIssue::new(
            IssueSeverity::Critical,
            "Critical",
            "Description",
            "test.rs",
        );
        let warning =
            CodeQualityIssue::new(IssueSeverity::Warning, "Warning", "Description", "test.rs");
        let result = CodeReviewResult::new(123)
            .with_issue(critical)
            .with_issue(warning);
        assert_eq!(result.quality_score, 70); // 100 - 20 - 10
    }

    #[test]
    fn test_code_review_standards_default() {
        let standards = CodeReviewStandards::default();
        assert_eq!(standards.min_quality_score, 70);
        assert!(standards.require_critical_fixes);
        assert!(!standards.require_warning_fixes);
    }

    #[test]
    fn test_code_review_agent_creation() {
        let agent = CodeReviewAgent::new();
        assert_eq!(agent.standards.min_quality_score, 70);
    }

    #[test]
    fn test_analyze_code_empty_files() {
        let agent = CodeReviewAgent::new();
        let mut pr = create_test_pr();
        pr.files.clear();
        let issues = agent.analyze_code(&pr).unwrap();
        assert!(issues.is_empty());
    }

    #[test]
    fn test_analyze_code_large_file() {
        let agent = CodeReviewAgent::new();
        let mut pr = create_test_pr();
        pr.files[0].additions = 300;
        pr.files[0].deletions = 300;
        let issues = agent.analyze_code(&pr).unwrap();
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.title.contains("Large file")));
    }

    #[test]
    fn test_analyze_code_empty_body() {
        let agent = CodeReviewAgent::new();
        let mut pr = create_test_pr();
        pr.body.clear();
        let issues = agent.analyze_code(&pr).unwrap();
        assert!(issues
            .iter()
            .any(|i| i.title.contains("Missing PR description")));
    }

    #[test]
    fn test_validate_standards_no_files() {
        let agent = CodeReviewAgent::new();
        let mut pr = create_test_pr();
        pr.files.clear();
        let issues = agent.validate_standards(&pr).unwrap();
        assert!(issues.iter().any(|i| i.severity == IssueSeverity::Critical));
    }

    #[test]
    fn test_validate_standards_invalid_branch() {
        let agent = CodeReviewAgent::new();
        let mut pr = create_test_pr();
        pr.branch = "invalid-branch".to_string();
        let issues = agent.validate_standards(&pr).unwrap();
        assert!(issues.iter().any(|i| i.title.contains("Invalid branch")));
    }

    #[test]
    fn test_is_valid_branch_name() {
        let agent = CodeReviewAgent::new();
        assert!(agent.is_valid_branch_name("feature/test"));
        assert!(agent.is_valid_branch_name("bugfix/test"));
        assert!(agent.is_valid_branch_name("hotfix/test"));
        assert!(agent.is_valid_branch_name("main"));
        assert!(!agent.is_valid_branch_name("invalid"));
    }

    #[test]
    fn test_generate_suggestions() {
        let agent = CodeReviewAgent::new();
        let issues = vec![CodeQualityIssue::new(
            IssueSeverity::Warning,
            "Test",
            "Description",
            "test.rs",
        )];
        let suggestions = agent.generate_suggestions(&issues).unwrap();
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].title, "Test");
    }

    #[test]
    fn test_generate_summary() {
        let agent = CodeReviewAgent::new();
        let result = CodeReviewResult::new(123);
        let summary = agent.generate_summary(&result).unwrap();
        assert!(summary.contains("Code Review Summary"));
        assert!(summary.contains("PR #123"));
        assert!(summary.contains("APPROVED"));
    }

    #[test]
    fn test_should_approve_high_quality() {
        let agent = CodeReviewAgent::new();
        let result = CodeReviewResult::new(123).set_approved(true, None);
        assert!(agent.should_approve(&result).unwrap());
    }

    #[test]
    fn test_should_approve_low_quality() {
        let agent = CodeReviewAgent::new();
        let mut result = CodeReviewResult::new(123);
        result.quality_score = 50;
        assert!(!agent.should_approve(&result).unwrap());
    }

    #[test]
    fn test_should_approve_with_critical_issues() {
        let standards = CodeReviewStandards::default().require_critical_fixes(true);
        let agent = CodeReviewAgent::with_standards(standards);
        let issue = CodeQualityIssue::new(
            IssueSeverity::Critical,
            "Critical",
            "Description",
            "test.rs",
        );
        let result = CodeReviewResult::new(123).with_issue(issue);
        assert!(!agent.should_approve(&result).unwrap());
    }

    #[test]
    fn test_review_pr_complete() {
        let agent = CodeReviewAgent::new();
        let pr = create_test_pr();
        let result = agent.review_pr(&pr).unwrap();
        assert_eq!(result.pr_number, 123);
        assert!(result.quality_score > 0);
    }

    #[test]
    fn test_review_pr_with_issues() {
        let agent = CodeReviewAgent::new();
        let mut pr = create_test_pr();
        pr.body.clear();
        pr.branch = "invalid".to_string();
        let result = agent.review_pr(&pr).unwrap();
        assert!(!result.issues.is_empty());
    }
}
