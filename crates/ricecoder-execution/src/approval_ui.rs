//! Approval UI components for execution plans
//!
//! Provides UI components for displaying approval requests and handling
//! user decisions (approve/reject). Designed to integrate with the TUI.

use crate::approval::ApprovalSummary;
use crate::models::RiskLevel;

/// Approval UI state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalUIState {
    /// Waiting for user input
    Waiting,
    /// User approved
    Approved,
    /// User rejected
    Rejected,
}

/// Approval UI component for displaying and handling approval requests
#[derive(Debug, Clone)]
pub struct ApprovalUI {
    /// Request ID
    pub request_id: String,
    /// Plan summary
    pub summary: ApprovalSummary,
    /// Current UI state
    pub state: ApprovalUIState,
    /// User comments (if any)
    pub comments: Option<String>,
}

impl ApprovalUI {
    /// Create a new approval UI component
    pub fn new(request_id: String, summary: ApprovalSummary) -> Self {
        ApprovalUI {
            request_id,
            summary,
            state: ApprovalUIState::Waiting,
            comments: None,
        }
    }

    /// Mark as approved
    pub fn approve(&mut self, comments: Option<String>) {
        self.state = ApprovalUIState::Approved;
        self.comments = comments;
    }

    /// Mark as rejected
    pub fn reject(&mut self, comments: Option<String>) {
        self.state = ApprovalUIState::Rejected;
        self.comments = comments;
    }

    /// Get the risk level color for TUI rendering
    ///
    /// Returns a color code suitable for terminal rendering.
    pub fn risk_color(&self) -> &'static str {
        match self.summary.risk_level {
            RiskLevel::Low => "green",
            RiskLevel::Medium => "yellow",
            RiskLevel::High => "red",
            RiskLevel::Critical => "bright-red",
        }
    }

    /// Get the risk level emoji
    pub fn risk_emoji(&self) -> &'static str {
        match self.summary.risk_level {
            RiskLevel::Low => "✓",
            RiskLevel::Medium => "⚠",
            RiskLevel::High => "⚠⚠",
            RiskLevel::Critical => "🚨",
        }
    }

    /// Format the approval UI for display
    ///
    /// Returns a formatted string suitable for terminal display.
    pub fn format_display(&self) -> String {
        let status = match self.state {
            ApprovalUIState::Waiting => "⏳ Waiting for approval",
            ApprovalUIState::Approved => "✅ Approved",
            ApprovalUIState::Rejected => "❌ Rejected",
        };

        let mut display = format!(
            "{}\n\n{} Risk Level: {:?}\n\nPlan: {}\nSteps: {}\nRisk Score: {:.2}\nEstimated Duration: {}s\n\nRisk Factors:\n{}",
            status,
            self.risk_emoji(),
            self.summary.risk_level,
            self.summary.plan_name,
            self.summary.step_count,
            self.summary.risk_score,
            self.summary.estimated_duration_secs,
            self.summary.risk_factors
        );

        if let Some(comments) = &self.comments {
            display.push_str(&format!("\n\nComments: {}", comments));
        }

        display
    }

    /// Get the approval prompt text
    pub fn get_prompt(&self) -> String {
        match self.summary.risk_level {
            RiskLevel::Critical => {
                "⚠️  CRITICAL RISK - This plan has critical risk factors. Review carefully before approving.\n\nApprove? (y/n): ".to_string()
            }
            RiskLevel::High => {
                "⚠️  HIGH RISK - This plan has high risk factors. Review before approving.\n\nApprove? (y/n): ".to_string()
            }
            RiskLevel::Medium => {
                "Plan requires approval.\n\nApprove? (y/n): ".to_string()
            }
            RiskLevel::Low => {
                "Plan is ready for execution.\n\nApprove? (y/n): ".to_string()
            }
        }
    }

    /// Get the approval instructions
    pub fn get_instructions(&self) -> String {
        format!(
            "Plan: {}\nSteps: {}\nRisk Level: {:?}\n\nPress 'y' to approve, 'n' to reject, or 'q' to cancel",
            self.summary.plan_name,
            self.summary.step_count,
            self.summary.risk_level
        )
    }
}

/// Approval UI builder for constructing approval UI components
pub struct ApprovalUIBuilder {
    request_id: Option<String>,
    summary: Option<ApprovalSummary>,
}

impl ApprovalUIBuilder {
    /// Create a new approval UI builder
    pub fn new() -> Self {
        ApprovalUIBuilder {
            request_id: None,
            summary: None,
        }
    }

    /// Set the request ID
    pub fn request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }

    /// Set the approval summary
    pub fn summary(mut self, summary: ApprovalSummary) -> Self {
        self.summary = Some(summary);
        self
    }

    /// Build the approval UI component
    pub fn build(self) -> Result<ApprovalUI, String> {
        let request_id = self
            .request_id
            .ok_or_else(|| "request_id is required".to_string())?;
        let summary = self
            .summary
            .ok_or_else(|| "summary is required".to_string())?;

        Ok(ApprovalUI::new(request_id, summary))
    }
}

impl Default for ApprovalUIBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_summary() -> ApprovalSummary {
        ApprovalSummary {
            plan_id: "plan1".to_string(),
            plan_name: "Test Plan".to_string(),
            step_count: 3,
            risk_level: RiskLevel::High,
            risk_score: 1.8,
            risk_factors: "- file_count: 1 file modified".to_string(),
            estimated_duration_secs: 60,
            requires_approval: true,
        }
    }

    #[test]
    fn test_create_approval_ui() {
        let summary = create_test_summary();
        let ui = ApprovalUI::new("req1".to_string(), summary);

        assert_eq!(ui.request_id, "req1");
        assert_eq!(ui.state, ApprovalUIState::Waiting);
        assert_eq!(ui.summary.plan_name, "Test Plan");
    }

    #[test]
    fn test_approve() {
        let summary = create_test_summary();
        let mut ui = ApprovalUI::new("req1".to_string(), summary);

        ui.approve(Some("Looks good".to_string()));

        assert_eq!(ui.state, ApprovalUIState::Approved);
        assert_eq!(ui.comments, Some("Looks good".to_string()));
    }

    #[test]
    fn test_reject() {
        let summary = create_test_summary();
        let mut ui = ApprovalUI::new("req1".to_string(), summary);

        ui.reject(Some("Needs changes".to_string()));

        assert_eq!(ui.state, ApprovalUIState::Rejected);
        assert_eq!(ui.comments, Some("Needs changes".to_string()));
    }

    #[test]
    fn test_risk_color() {
        let mut summary = create_test_summary();
        summary.risk_level = RiskLevel::Low;
        let ui = ApprovalUI::new("req1".to_string(), summary);
        assert_eq!(ui.risk_color(), "green");

        let mut summary = create_test_summary();
        summary.risk_level = RiskLevel::Medium;
        let ui = ApprovalUI::new("req1".to_string(), summary);
        assert_eq!(ui.risk_color(), "yellow");

        let mut summary = create_test_summary();
        summary.risk_level = RiskLevel::High;
        let ui = ApprovalUI::new("req1".to_string(), summary);
        assert_eq!(ui.risk_color(), "red");

        let mut summary = create_test_summary();
        summary.risk_level = RiskLevel::Critical;
        let ui = ApprovalUI::new("req1".to_string(), summary);
        assert_eq!(ui.risk_color(), "bright-red");
    }

    #[test]
    fn test_risk_emoji() {
        let mut summary = create_test_summary();
        summary.risk_level = RiskLevel::Low;
        let ui = ApprovalUI::new("req1".to_string(), summary);
        assert_eq!(ui.risk_emoji(), "✓");

        let mut summary = create_test_summary();
        summary.risk_level = RiskLevel::Medium;
        let ui = ApprovalUI::new("req1".to_string(), summary);
        assert_eq!(ui.risk_emoji(), "⚠");

        let mut summary = create_test_summary();
        summary.risk_level = RiskLevel::High;
        let ui = ApprovalUI::new("req1".to_string(), summary);
        assert_eq!(ui.risk_emoji(), "⚠⚠");

        let mut summary = create_test_summary();
        summary.risk_level = RiskLevel::Critical;
        let ui = ApprovalUI::new("req1".to_string(), summary);
        assert_eq!(ui.risk_emoji(), "🚨");
    }

    #[test]
    fn test_format_display() {
        let summary = create_test_summary();
        let ui = ApprovalUI::new("req1".to_string(), summary);

        let display = ui.format_display();
        assert!(display.contains("Waiting for approval"));
        assert!(display.contains("Test Plan"));
        assert!(display.contains("Steps: 3"));
    }

    #[test]
    fn test_get_prompt_critical() {
        let mut summary = create_test_summary();
        summary.risk_level = RiskLevel::Critical;
        let ui = ApprovalUI::new("req1".to_string(), summary);

        let prompt = ui.get_prompt();
        assert!(prompt.contains("CRITICAL RISK"));
    }

    #[test]
    fn test_get_prompt_high() {
        let mut summary = create_test_summary();
        summary.risk_level = RiskLevel::High;
        let ui = ApprovalUI::new("req1".to_string(), summary);

        let prompt = ui.get_prompt();
        assert!(prompt.contains("HIGH RISK"));
    }

    #[test]
    fn test_get_instructions() {
        let summary = create_test_summary();
        let ui = ApprovalUI::new("req1".to_string(), summary);

        let instructions = ui.get_instructions();
        assert!(instructions.contains("Test Plan"));
        assert!(instructions.contains("Steps: 3"));
    }

    #[test]
    fn test_approval_ui_builder() {
        let summary = create_test_summary();
        let ui = ApprovalUIBuilder::new()
            .request_id("req1".to_string())
            .summary(summary)
            .build()
            .unwrap();

        assert_eq!(ui.request_id, "req1");
        assert_eq!(ui.state, ApprovalUIState::Waiting);
    }

    #[test]
    fn test_approval_ui_builder_missing_request_id() {
        let summary = create_test_summary();
        let result = ApprovalUIBuilder::new().summary(summary).build();

        assert!(result.is_err());
    }

    #[test]
    fn test_approval_ui_builder_missing_summary() {
        let result = ApprovalUIBuilder::new()
            .request_id("req1".to_string())
            .build();

        assert!(result.is_err());
    }
}
