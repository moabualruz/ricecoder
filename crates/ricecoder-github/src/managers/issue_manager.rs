//! Issue Management
//!
//! Handles GitHub issue assignment, parsing, and tracking

use crate::errors::{GitHubError, Result};
use crate::models::{Issue, IssueProgressUpdate, IssueStatus};
use regex::Regex;
use serde::{Deserialize, Serialize};

/// Parsed requirement from an issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedRequirement {
    /// Requirement ID
    pub id: String,
    /// Requirement description
    pub description: String,
    /// Acceptance criteria
    pub acceptance_criteria: Vec<String>,
    /// Priority level
    pub priority: String,
}

/// Implementation plan generated from issue requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationPlan {
    /// Plan ID
    pub id: String,
    /// Issue number this plan is for
    pub issue_number: u32,
    /// List of tasks
    pub tasks: Vec<PlanTask>,
    /// Estimated effort (in story points)
    pub estimated_effort: u32,
    /// Generated timestamp
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

/// A task in an implementation plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanTask {
    /// Task ID
    pub id: String,
    /// Task description
    pub description: String,
    /// Related requirements
    pub related_requirements: Vec<String>,
    /// Estimated effort
    pub estimated_effort: u32,
    /// Dependencies on other tasks
    pub dependencies: Vec<String>,
}

/// Issue Manager for handling GitHub issues
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct IssueManager {
    /// GitHub token for API access
    token: String,
    /// Owner of the repository
    owner: String,
    /// Repository name
    repo: String,
}

impl IssueManager {
    /// Create a new IssueManager
    pub fn new(token: String, owner: String, repo: String) -> Self {
        IssueManager { token, owner, repo }
    }

    /// Parse an issue input (URL or issue number)
    ///
    /// Accepts formats like:
    /// - "123" (issue number)
    /// - "https://github.com/owner/repo/issues/123"
    /// - "owner/repo#123"
    pub fn parse_issue_input(&self, input: &str) -> Result<u32> {
        // Try parsing as plain number
        if let Ok(number) = input.parse::<u32>() {
            return Ok(number);
        }

        // Try parsing GitHub URL
        if let Some(captures) = Regex::new(r"issues/(\d+)")
            .ok()
            .and_then(|re| re.captures(input))
        {
            if let Ok(number) = captures.get(1).unwrap().as_str().parse::<u32>() {
                return Ok(number);
            }
        }

        // Try parsing owner/repo#number format
        if let Some(captures) = Regex::new(r"#(\d+)")
            .ok()
            .and_then(|re| re.captures(input))
        {
            if let Ok(number) = captures.get(1).unwrap().as_str().parse::<u32>() {
                return Ok(number);
            }
        }

        Err(GitHubError::invalid_input(format!(
            "Invalid issue input format: {}. Expected issue number, GitHub URL, or owner/repo#number",
            input
        )))
    }

    /// Extract requirements from an issue description
    ///
    /// Looks for patterns like:
    /// - "## Requirement 1: ..."
    /// - "### Acceptance Criteria"
    /// - "- [ ] ..." (checkboxes)
    pub fn extract_requirements(&self, issue_body: &str) -> Result<Vec<ParsedRequirement>> {
        let mut requirements = Vec::new();

        // Split by requirement headers
        let requirement_pattern = Regex::new(r"##\s+Requirement\s+(\d+):\s*(.+)")
            .map_err(|e| GitHubError::invalid_input(format!("Regex error: {}", e)))?;

        let matches: Vec<_> = requirement_pattern.captures_iter(issue_body).collect();

        for (idx, req_match) in matches.iter().enumerate() {
            let req_id = req_match.get(1).map(|m| m.as_str()).unwrap_or("0");
            let req_start = req_match.get(2).unwrap().start();

            // Get content until next requirement or end of string
            let req_end = if idx + 1 < matches.len() {
                matches[idx + 1].get(0).unwrap().start()
            } else {
                issue_body.len()
            };

            let req_content = &issue_body[req_start..req_end];

            // Extract description (first line after header)
            let description = req_content
                .lines()
                .next()
                .unwrap_or("")
                .trim()
                .to_string();

            // Extract acceptance criteria (lines starting with "- ")
            let criteria_pattern = Regex::new(r"- \[.\]\s+(.+)")
                .map_err(|e| GitHubError::invalid_input(format!("Regex error: {}", e)))?;

            let acceptance_criteria: Vec<String> = criteria_pattern
                .captures_iter(req_content)
                .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
                .collect();

            // Extract priority if present
            let priority = if req_content.contains("Priority: HIGH") {
                "HIGH".to_string()
            } else if req_content.contains("Priority: MEDIUM") {
                "MEDIUM".to_string()
            } else {
                "MEDIUM".to_string()
            };

            requirements.push(ParsedRequirement {
                id: format!("REQ-{}", req_id),
                description,
                acceptance_criteria,
                priority,
            });
        }

        // If no structured requirements found, treat entire body as one requirement
        if requirements.is_empty() && !issue_body.is_empty() {
            requirements.push(ParsedRequirement {
                id: "REQ-1".to_string(),
                description: issue_body.lines().next().unwrap_or("").to_string(),
                acceptance_criteria: vec![],
                priority: "MEDIUM".to_string(),
            });
        }

        Ok(requirements)
    }

    /// Create an implementation plan from parsed requirements
    pub fn create_implementation_plan(
        &self,
        issue_number: u32,
        requirements: Vec<ParsedRequirement>,
    ) -> Result<ImplementationPlan> {
        let mut tasks = Vec::new();
        let mut total_effort = 0u32;

        for (idx, req) in requirements.iter().enumerate() {
            let task_id = format!("TASK-{}-{}", issue_number, idx + 1);
            let estimated_effort = match req.priority.as_str() {
                "HIGH" => 8,
                "MEDIUM" => 5,
                "LOW" => 3,
                _ => 5,
            };

            total_effort += estimated_effort;

            tasks.push(PlanTask {
                id: task_id,
                description: req.description.clone(),
                related_requirements: vec![req.id.clone()],
                estimated_effort,
                dependencies: if idx > 0 {
                    vec![format!("TASK-{}-{}", issue_number, idx)]
                } else {
                    vec![]
                },
            });
        }

        Ok(ImplementationPlan {
            id: format!("PLAN-{}", issue_number),
            issue_number,
            tasks,
            estimated_effort: total_effort,
            generated_at: chrono::Utc::now(),
        })
    }

    /// Format a progress update message for posting to an issue
    pub fn format_progress_update(&self, update: &IssueProgressUpdate) -> String {
        let status_emoji = match update.status {
            IssueStatus::Open => "🔴",
            IssueStatus::InProgress => "🟡",
            IssueStatus::Closed => "🟢",
        };

        let progress_bar = self.create_progress_bar(update.progress_percentage);

        format!(
            "{} **Progress Update**\n\n\
             Status: {}\n\
             Progress: {}\n\n\
             {}\n\n\
             _Updated at: {}_",
            status_emoji,
            format!("{:?}", update.status),
            progress_bar,
            update.message,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        )
    }

    /// Create a progress bar string
    fn create_progress_bar(&self, percentage: u32) -> String {
        let filled = (percentage / 10) as usize;
        let empty = 10 - filled;
        let bar = format!(
            "[{}{}] {}%",
            "█".repeat(filled),
            "░".repeat(empty),
            percentage
        );
        bar
    }

    /// Format a PR closure message for an issue
    pub fn format_pr_closure_message(&self, pr_number: u32, pr_title: &str) -> String {
        format!(
            "Closes #{} - {}\n\nThis PR implements the requirements from this issue.",
            pr_number, pr_title
        )
    }

    /// Validate that an issue has required fields
    pub fn validate_issue(&self, issue: &Issue) -> Result<()> {
        if issue.title.is_empty() {
            return Err(GitHubError::invalid_input("Issue title cannot be empty"));
        }

        if issue.body.is_empty() {
            return Err(GitHubError::invalid_input("Issue body cannot be empty"));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_manager() -> IssueManager {
        IssueManager::new(
            "test_token".to_string(),
            "test_owner".to_string(),
            "test_repo".to_string(),
        )
    }

    #[test]
    fn test_parse_issue_input_number() {
        let manager = create_test_manager();
        assert_eq!(manager.parse_issue_input("123").unwrap(), 123);
    }

    #[test]
    fn test_parse_issue_input_url() {
        let manager = create_test_manager();
        let url = "https://github.com/owner/repo/issues/456";
        assert_eq!(manager.parse_issue_input(url).unwrap(), 456);
    }

    #[test]
    fn test_parse_issue_input_hash_format() {
        let manager = create_test_manager();
        let input = "owner/repo#789";
        assert_eq!(manager.parse_issue_input(input).unwrap(), 789);
    }

    #[test]
    fn test_parse_issue_input_invalid() {
        let manager = create_test_manager();
        assert!(manager.parse_issue_input("invalid").is_err());
    }

    #[test]
    fn test_extract_requirements_empty() {
        let manager = create_test_manager();
        let requirements = manager.extract_requirements("").unwrap();
        assert!(requirements.is_empty());
    }

    #[test]
    fn test_extract_requirements_simple() {
        let manager = create_test_manager();
        let body = "This is a simple requirement";
        let requirements = manager.extract_requirements(body).unwrap();
        assert_eq!(requirements.len(), 1);
        assert_eq!(requirements[0].description, "This is a simple requirement");
    }

    #[test]
    fn test_create_implementation_plan() {
        let manager = create_test_manager();
        let requirements = vec![
            ParsedRequirement {
                id: "REQ-1".to_string(),
                description: "Implement feature X".to_string(),
                acceptance_criteria: vec!["Criterion 1".to_string()],
                priority: "HIGH".to_string(),
            },
            ParsedRequirement {
                id: "REQ-2".to_string(),
                description: "Add tests".to_string(),
                acceptance_criteria: vec![],
                priority: "MEDIUM".to_string(),
            },
        ];

        let plan = manager.create_implementation_plan(123, requirements).unwrap();
        assert_eq!(plan.issue_number, 123);
        assert_eq!(plan.tasks.len(), 2);
        assert!(plan.estimated_effort > 0);
    }

    #[test]
    fn test_format_progress_update() {
        let manager = create_test_manager();
        let update = IssueProgressUpdate {
            issue_number: 123,
            message: "Working on implementation".to_string(),
            status: IssueStatus::InProgress,
            progress_percentage: 50,
        };

        let formatted = manager.format_progress_update(&update);
        assert!(formatted.contains("Progress Update"));
        assert!(formatted.contains("50%"));
    }

    #[test]
    fn test_validate_issue_valid() {
        let manager = create_test_manager();
        let issue = Issue {
            id: 1,
            number: 123,
            title: "Test Issue".to_string(),
            body: "Test body".to_string(),
            labels: vec![],
            assignees: vec![],
            status: IssueStatus::Open,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        assert!(manager.validate_issue(&issue).is_ok());
    }

    #[test]
    fn test_validate_issue_empty_title() {
        let manager = create_test_manager();
        let issue = Issue {
            id: 1,
            number: 123,
            title: "".to_string(),
            body: "Test body".to_string(),
            labels: vec![],
            assignees: vec![],
            status: IssueStatus::Open,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        assert!(manager.validate_issue(&issue).is_err());
    }
}
