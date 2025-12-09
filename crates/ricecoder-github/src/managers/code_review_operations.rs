//! Code Review Operations - Additional code review functionality

use crate::errors::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// Code review metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeReviewMetrics {
    /// Total PRs reviewed
    pub total_prs_reviewed: u32,
    /// PRs approved
    pub prs_approved: u32,
    /// PRs with changes requested
    pub prs_changes_requested: u32,
    /// Average review time (in minutes)
    pub average_review_time: u32,
    /// Average quality score
    pub average_quality_score: f32,
}

impl Default for CodeReviewMetrics {
    fn default() -> Self {
        Self {
            total_prs_reviewed: 0,
            prs_approved: 0,
            prs_changes_requested: 0,
            average_review_time: 0,
            average_quality_score: 0.0,
        }
    }
}

impl CodeReviewMetrics {
    /// Create new metrics
    pub fn new() -> Self {
        Self::default()
    }

    /// Update metrics with new review
    pub fn update_with_review(
        &mut self,
        approved: bool,
        quality_score: u32,
        review_time_minutes: u32,
    ) {
        self.total_prs_reviewed += 1;

        if approved {
            self.prs_approved += 1;
        } else {
            self.prs_changes_requested += 1;
        }

        // Update average review time
        self.average_review_time = (self.average_review_time * (self.total_prs_reviewed - 1)
            + review_time_minutes)
            / self.total_prs_reviewed;

        // Update average quality score
        self.average_quality_score = (self.average_quality_score * (self.total_prs_reviewed - 1) as f32
            + quality_score as f32)
            / self.total_prs_reviewed as f32;
    }

    /// Get approval rate
    pub fn approval_rate(&self) -> f32 {
        if self.total_prs_reviewed == 0 {
            0.0
        } else {
            (self.prs_approved as f32 / self.total_prs_reviewed as f32) * 100.0
        }
    }
}

/// Approval condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalCondition {
    /// Condition name
    pub name: String,
    /// Condition description
    pub description: String,
    /// Is met
    pub is_met: bool,
}

impl ApprovalCondition {
    /// Create new condition
    pub fn new(name: impl Into<String>, description: impl Into<String>, is_met: bool) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            is_met,
        }
    }
}

/// Conditional approval result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalApprovalResult {
    /// PR number
    pub pr_number: u32,
    /// Approval conditions
    pub conditions: Vec<ApprovalCondition>,
    /// Overall approval
    pub approved: bool,
    /// Approval reason
    pub reason: String,
}

impl ConditionalApprovalResult {
    /// Create new result
    pub fn new(pr_number: u32) -> Self {
        Self {
            pr_number,
            conditions: Vec::new(),
            approved: false,
            reason: String::new(),
        }
    }

    /// Add condition
    pub fn with_condition(mut self, condition: ApprovalCondition) -> Self {
        self.conditions.push(condition);
        self
    }

    /// Add conditions
    pub fn with_conditions(mut self, conditions: Vec<ApprovalCondition>) -> Self {
        self.conditions.extend(conditions);
        self
    }

    /// Set approval
    pub fn set_approved(mut self, approved: bool, reason: impl Into<String>) -> Self {
        self.approved = approved;
        self.reason = reason.into();
        self
    }

    /// Check if all conditions are met
    pub fn all_conditions_met(&self) -> bool {
        self.conditions.iter().all(|c| c.is_met)
    }

    /// Get unmet conditions
    pub fn unmet_conditions(&self) -> Vec<&ApprovalCondition> {
        self.conditions.iter().filter(|c| !c.is_met).collect()
    }
}

/// Code Review Operations
pub struct CodeReviewOperations;

impl CodeReviewOperations {
    /// Create a new code review operations instance
    pub fn new() -> Self {
        Self
    }

    /// Post code review suggestion as comment
    pub fn post_suggestion_comment(
        &self,
        pr_number: u32,
        file_path: &str,
        line_number: Option<u32>,
        suggestion: &str,
    ) -> Result<String> {
        debug!(
            pr_number = pr_number,
            file_path = file_path,
            line_number = line_number,
            "Posting code review suggestion"
        );

        let mut comment = "**Code Review Suggestion**\n\n".to_string();
        comment.push_str(&format!("File: `{}`\n", file_path));

        if let Some(line) = line_number {
            comment.push_str(&format!("Line: {}\n\n", line));
        }

        comment.push_str(&format!("**Suggestion:**\n{}\n", suggestion));

        info!(
            pr_number = pr_number,
            comment_length = comment.len(),
            "Suggestion comment created"
        );

        Ok(comment)
    }

    /// Generate code review summary report
    pub fn generate_summary_report(
        &self,
        pr_number: u32,
        quality_score: u32,
        issues_count: usize,
        suggestions_count: usize,
        approved: bool,
    ) -> Result<String> {
        debug!(
            pr_number = pr_number,
            quality_score = quality_score,
            "Generating code review summary report"
        );

        let mut report = format!("## Code Review Report - PR #{}\n\n", pr_number);

        // Quality score
        report.push_str(&format!("**Quality Score:** {}/100\n\n", quality_score));

        // Status
        let status = if approved { "✅ APPROVED" } else { "❌ NEEDS REVIEW" };
        report.push_str(&format!("**Status:** {}\n\n", status));

        // Issues and suggestions
        report.push_str(&format!("**Issues Found:** {}\n", issues_count));
        report.push_str(&format!("**Suggestions:** {}\n\n", suggestions_count));

        // Recommendation
        if approved {
            report.push_str("This PR meets all quality standards and is ready for merge.\n");
        } else {
            report.push_str("This PR requires attention before merging. Please address the issues and suggestions above.\n");
        }

        info!(
            pr_number = pr_number,
            report_length = report.len(),
            "Summary report generated"
        );

        Ok(report)
    }

    /// Track review metrics
    pub fn track_metrics(
        &self,
        metrics: &mut CodeReviewMetrics,
        approved: bool,
        quality_score: u32,
        review_time_minutes: u32,
    ) -> Result<()> {
        debug!(
            approved = approved,
            quality_score = quality_score,
            review_time_minutes = review_time_minutes,
            "Tracking review metrics"
        );

        metrics.update_with_review(approved, quality_score, review_time_minutes);

        info!(
            total_reviewed = metrics.total_prs_reviewed,
            approval_rate = metrics.approval_rate(),
            "Metrics updated"
        );

        Ok(())
    }

    /// Evaluate conditional approval
    pub fn evaluate_conditional_approval(
        &self,
        pr_number: u32,
        conditions: HashMap<String, bool>,
    ) -> Result<ConditionalApprovalResult> {
        debug!(
            pr_number = pr_number,
            condition_count = conditions.len(),
            "Evaluating conditional approval"
        );

        let mut result = ConditionalApprovalResult::new(pr_number);

        for (name, is_met) in conditions {
            let condition = ApprovalCondition::new(
                &name,
                format!("Condition: {}", name),
                is_met,
            );
            result = result.with_condition(condition);
        }

        // Determine overall approval
        let all_met = result.all_conditions_met();
        let reason = if all_met {
            "All approval conditions are met".to_string()
        } else {
            format!(
                "{} condition(s) not met",
                result.unmet_conditions().len()
            )
        };

        result = result.set_approved(all_met, reason);

        info!(
            pr_number = pr_number,
            approved = result.approved,
            "Conditional approval evaluated"
        );

        Ok(result)
    }

    /// Generate approval checklist
    pub fn generate_approval_checklist(
        &self,
        pr_number: u32,
        conditions: &[ApprovalCondition],
    ) -> Result<String> {
        debug!(
            pr_number = pr_number,
            condition_count = conditions.len(),
            "Generating approval checklist"
        );

        let mut checklist = format!("## Approval Checklist - PR #{}\n\n", pr_number);

        for condition in conditions {
            let checkbox = if condition.is_met { "✅" } else { "❌" };
            checklist.push_str(&format!(
                "{} **{}**: {}\n",
                checkbox, condition.name, condition.description
            ));
        }

        info!(
            pr_number = pr_number,
            checklist_length = checklist.len(),
            "Approval checklist generated"
        );

        Ok(checklist)
    }
}

impl Default for CodeReviewOperations {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_review_metrics_default() {
        let metrics = CodeReviewMetrics::default();
        assert_eq!(metrics.total_prs_reviewed, 0);
        assert_eq!(metrics.prs_approved, 0);
    }

    #[test]
    fn test_code_review_metrics_update() {
        let mut metrics = CodeReviewMetrics::new();
        metrics.update_with_review(true, 85, 30);
        assert_eq!(metrics.total_prs_reviewed, 1);
        assert_eq!(metrics.prs_approved, 1);
        assert_eq!(metrics.average_quality_score, 85.0);
    }

    #[test]
    fn test_code_review_metrics_approval_rate() {
        let mut metrics = CodeReviewMetrics::new();
        metrics.update_with_review(true, 85, 30);
        metrics.update_with_review(false, 60, 45);
        assert_eq!(metrics.approval_rate(), 50.0);
    }

    #[test]
    fn test_approval_condition_creation() {
        let condition = ApprovalCondition::new("Test", "Test condition", true);
        assert_eq!(condition.name, "Test");
        assert!(condition.is_met);
    }

    #[test]
    fn test_conditional_approval_result_creation() {
        let result = ConditionalApprovalResult::new(123);
        assert_eq!(result.pr_number, 123);
        assert!(result.conditions.is_empty());
    }

    #[test]
    fn test_conditional_approval_result_with_conditions() {
        let condition1 = ApprovalCondition::new("Test1", "Description1", true);
        let condition2 = ApprovalCondition::new("Test2", "Description2", false);
        let result = ConditionalApprovalResult::new(123)
            .with_condition(condition1)
            .with_condition(condition2);
        assert_eq!(result.conditions.len(), 2);
    }

    #[test]
    fn test_conditional_approval_result_all_conditions_met() {
        let condition = ApprovalCondition::new("Test", "Description", true);
        let result = ConditionalApprovalResult::new(123)
            .with_condition(condition);
        assert!(result.all_conditions_met());
    }

    #[test]
    fn test_conditional_approval_result_unmet_conditions() {
        let condition1 = ApprovalCondition::new("Test1", "Description1", true);
        let condition2 = ApprovalCondition::new("Test2", "Description2", false);
        let result = ConditionalApprovalResult::new(123)
            .with_condition(condition1)
            .with_condition(condition2);
        assert_eq!(result.unmet_conditions().len(), 1);
    }

    #[test]
    fn test_code_review_operations_creation() {
        let ops = CodeReviewOperations::new();
        assert_eq!(std::mem::size_of_val(&ops), 0); // Zero-sized type
    }

    #[test]
    fn test_post_suggestion_comment() {
        let ops = CodeReviewOperations::new();
        let comment = ops
            .post_suggestion_comment(123, "src/main.rs", Some(42), "Use better variable name")
            .unwrap();
        assert!(comment.contains("Code Review Suggestion"));
        assert!(comment.contains("src/main.rs"));
        assert!(comment.contains("42"));
    }

    #[test]
    fn test_generate_summary_report() {
        let ops = CodeReviewOperations::new();
        let report = ops
            .generate_summary_report(123, 85, 2, 3, true)
            .unwrap();
        assert!(report.contains("PR #123"));
        assert!(report.contains("85/100"));
        assert!(report.contains("APPROVED"));
    }

    #[test]
    fn test_track_metrics() {
        let ops = CodeReviewOperations::new();
        let mut metrics = CodeReviewMetrics::new();
        ops.track_metrics(&mut metrics, true, 85, 30).unwrap();
        assert_eq!(metrics.total_prs_reviewed, 1);
    }

    #[test]
    fn test_evaluate_conditional_approval() {
        let ops = CodeReviewOperations::new();
        let mut conditions = HashMap::new();
        conditions.insert("Quality".to_string(), true);
        conditions.insert("Tests".to_string(), true);
        let result = ops.evaluate_conditional_approval(123, conditions).unwrap();
        assert!(result.approved);
    }

    #[test]
    fn test_evaluate_conditional_approval_not_met() {
        let ops = CodeReviewOperations::new();
        let mut conditions = HashMap::new();
        conditions.insert("Quality".to_string(), true);
        conditions.insert("Tests".to_string(), false);
        let result = ops.evaluate_conditional_approval(123, conditions).unwrap();
        assert!(!result.approved);
    }

    #[test]
    fn test_generate_approval_checklist() {
        let ops = CodeReviewOperations::new();
        let conditions = vec![
            ApprovalCondition::new("Quality", "Quality check", true),
            ApprovalCondition::new("Tests", "Test coverage", false),
        ];
        let checklist = ops.generate_approval_checklist(123, &conditions).unwrap();
        assert!(checklist.contains("PR #123"));
        assert!(checklist.contains("✅"));
        assert!(checklist.contains("❌"));
    }
}
