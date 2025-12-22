//! Project Operations - Handles project automation and reporting

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use super::project_manager::{ColumnStatus, ProjectManager, ProjectStatusReport};
use crate::errors::Result;

/// Automation action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AutomationAction {
    /// Move card to column
    MoveToColumn(ColumnStatus),
    /// Add label
    AddLabel(String),
    /// Remove label
    RemoveLabel(String),
    /// Assign to user
    AssignTo(String),
    /// Add comment
    AddComment(String),
}

/// Automation trigger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AutomationTrigger {
    /// PR opened
    PrOpened,
    /// PR closed
    PrClosed,
    /// PR merged
    PrMerged,
    /// Issue opened
    IssueOpened,
    /// Issue closed
    IssueClosed,
    /// Label added
    LabelAdded(String),
    /// Label removed
    LabelRemoved(String),
}

impl AutomationTrigger {
    /// Get trigger name
    pub fn name(&self) -> String {
        match self {
            AutomationTrigger::PrOpened => "pr_opened".to_string(),
            AutomationTrigger::PrClosed => "pr_closed".to_string(),
            AutomationTrigger::PrMerged => "pr_merged".to_string(),
            AutomationTrigger::IssueOpened => "issue_opened".to_string(),
            AutomationTrigger::IssueClosed => "issue_closed".to_string(),
            AutomationTrigger::LabelAdded(label) => format!("label_added:{}", label),
            AutomationTrigger::LabelRemoved(label) => format!("label_removed:{}", label),
        }
    }
}

/// Automation workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationWorkflow {
    /// Workflow name
    pub name: String,
    /// Trigger
    pub trigger: String,
    /// Actions to perform
    pub actions: Vec<AutomationAction>,
    /// Is enabled
    pub enabled: bool,
}

impl AutomationWorkflow {
    /// Create a new automation workflow
    pub fn new(name: impl Into<String>, trigger: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            trigger: trigger.into(),
            actions: Vec::new(),
            enabled: true,
        }
    }

    /// Add an action
    pub fn with_action(mut self, action: AutomationAction) -> Self {
        self.actions.push(action);
        self
    }

    /// Disable workflow
    pub fn disable(mut self) -> Self {
        self.enabled = false;
        self
    }
}

/// Project report section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSection {
    /// Section title
    pub title: String,
    /// Section content
    pub content: String,
}

/// Detailed project report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedProjectReport {
    /// Report title
    pub title: String,
    /// Report timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Report sections
    pub sections: Vec<ReportSection>,
    /// Summary statistics
    pub summary: HashMap<String, String>,
}

/// Project Operations
#[derive(Debug, Clone)]
pub struct ProjectOperations {
    /// Automation workflows
    workflows: Vec<AutomationWorkflow>,
    /// Report history
    report_history: Vec<DetailedProjectReport>,
}

impl ProjectOperations {
    /// Create new project operations
    pub fn new() -> Self {
        Self {
            workflows: Vec::new(),
            report_history: Vec::new(),
        }
    }

    /// Add automation workflow
    pub fn add_workflow(&mut self, workflow: AutomationWorkflow) {
        info!("Adding automation workflow: {}", workflow.name);
        self.workflows.push(workflow);
    }

    /// Get workflows
    pub fn workflows(&self) -> &[AutomationWorkflow] {
        &self.workflows
    }

    /// Get enabled workflows for trigger
    pub fn get_workflows_for_trigger(&self, trigger: &str) -> Vec<&AutomationWorkflow> {
        self.workflows
            .iter()
            .filter(|w| w.enabled && w.trigger == trigger)
            .collect()
    }

    /// Apply automation workflows
    pub fn apply_workflows(
        &self,
        project_manager: &mut ProjectManager,
        card_id: u64,
        trigger: &str,
    ) -> Result<Vec<AutomationAction>> {
        let workflows = self.get_workflows_for_trigger(trigger);
        let mut applied_actions = Vec::new();

        for workflow in workflows {
            debug!(
                "Applying workflow '{}' for trigger '{}'",
                workflow.name, trigger
            );

            for action in &workflow.actions {
                match action {
                    AutomationAction::MoveToColumn(status) => {
                        project_manager.move_card_to_column(card_id, *status)?;
                        applied_actions.push(action.clone());
                    }
                    _ => {
                        // Other actions would be handled by different systems
                        applied_actions.push(action.clone());
                    }
                }
            }
        }

        Ok(applied_actions)
    }

    /// Generate detailed report
    pub fn generate_detailed_report(
        &mut self,
        _project_manager: &ProjectManager,
        base_report: &ProjectStatusReport,
    ) -> DetailedProjectReport {
        let mut sections = Vec::new();
        let mut summary = HashMap::new();

        // Overview section
        sections.push(ReportSection {
            title: "Project Overview".to_string(),
            content: format!(
                "Project: {}\nTotal Cards: {}\nProgress: {}%",
                base_report.project_name,
                base_report.metrics.total_cards,
                base_report.metrics.progress_percentage
            ),
        });

        // Status breakdown section
        sections.push(ReportSection {
            title: "Status Breakdown".to_string(),
            content: format!(
                "Todo: {}\nIn Progress: {}\nIn Review: {}\nDone: {}",
                base_report.metrics.todo_count,
                base_report.metrics.in_progress_count,
                base_report.metrics.in_review_count,
                base_report.metrics.done_count
            ),
        });

        // Cards by column section
        let mut cards_content = String::new();
        for (column, cards) in &base_report.cards_by_column {
            cards_content.push_str(&format!("\n{}: {} cards\n", column, cards.len()));
            for card in cards {
                if let Some(note) = &card.note {
                    cards_content.push_str(&format!("  - {}\n", note));
                }
            }
        }
        sections.push(ReportSection {
            title: "Cards by Column".to_string(),
            content: cards_content,
        });

        // Summary statistics
        summary.insert(
            "total_cards".to_string(),
            base_report.metrics.total_cards.to_string(),
        );
        summary.insert(
            "progress_percentage".to_string(),
            base_report.metrics.progress_percentage.to_string(),
        );
        summary.insert(
            "todo_count".to_string(),
            base_report.metrics.todo_count.to_string(),
        );
        summary.insert(
            "in_progress_count".to_string(),
            base_report.metrics.in_progress_count.to_string(),
        );
        summary.insert(
            "in_review_count".to_string(),
            base_report.metrics.in_review_count.to_string(),
        );
        summary.insert(
            "done_count".to_string(),
            base_report.metrics.done_count.to_string(),
        );

        let report = DetailedProjectReport {
            title: format!("Project Report: {}", base_report.project_name),
            timestamp: chrono::Utc::now(),
            sections,
            summary,
        };

        self.report_history.push(report.clone());
        info!("Generated detailed project report");

        report
    }

    /// Get report history
    pub fn report_history(&self) -> &[DetailedProjectReport] {
        &self.report_history
    }

    /// Get latest report
    pub fn latest_report(&self) -> Option<&DetailedProjectReport> {
        self.report_history.last()
    }

    /// Clear report history
    pub fn clear_report_history(&mut self) {
        self.report_history.clear();
    }
}

impl Default for ProjectOperations {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_automation_workflow() {
        let workflow = AutomationWorkflow::new("Test Workflow", "pr_opened")
            .with_action(AutomationAction::MoveToColumn(ColumnStatus::InReview));

        assert_eq!(workflow.name, "Test Workflow");
        assert_eq!(workflow.trigger, "pr_opened");
        assert_eq!(workflow.actions.len(), 1);
        assert!(workflow.enabled);
    }

    #[test]
    fn test_automation_trigger_name() {
        assert_eq!(AutomationTrigger::PrOpened.name(), "pr_opened");
        assert_eq!(AutomationTrigger::PrMerged.name(), "pr_merged");
        assert_eq!(
            AutomationTrigger::LabelAdded("bug".to_string()).name(),
            "label_added:bug"
        );
    }

    #[test]
    fn test_project_operations_add_workflow() {
        let mut ops = ProjectOperations::new();
        let workflow = AutomationWorkflow::new("Test", "pr_opened");
        ops.add_workflow(workflow);

        assert_eq!(ops.workflows().len(), 1);
    }

    #[test]
    fn test_get_workflows_for_trigger() {
        let mut ops = ProjectOperations::new();
        ops.add_workflow(AutomationWorkflow::new("Workflow 1", "pr_opened"));
        ops.add_workflow(AutomationWorkflow::new("Workflow 2", "pr_opened"));
        ops.add_workflow(AutomationWorkflow::new("Workflow 3", "issue_opened"));

        let workflows = ops.get_workflows_for_trigger("pr_opened");
        assert_eq!(workflows.len(), 2);
    }

    #[test]
    fn test_generate_detailed_report() {
        let mut ops = ProjectOperations::new();
        let mut project_manager = ProjectManager::new(1, "Test Project");
        project_manager.set_column_mapping(ColumnStatus::Todo, 100);

        let base_report = project_manager.generate_status_report();
        let detailed_report = ops.generate_detailed_report(&project_manager, &base_report);

        assert_eq!(detailed_report.title, "Project Report: Test Project");
        assert!(!detailed_report.sections.is_empty());
        assert!(!detailed_report.summary.is_empty());
    }

    #[test]
    fn test_report_history() {
        let mut ops = ProjectOperations::new();
        let project_manager = ProjectManager::new(1, "Test Project");
        let base_report = project_manager.generate_status_report();

        ops.generate_detailed_report(&project_manager, &base_report);
        ops.generate_detailed_report(&project_manager, &base_report);

        assert_eq!(ops.report_history().len(), 2);
        assert!(ops.latest_report().is_some());
    }

    #[test]
    fn test_disable_workflow() {
        let workflow = AutomationWorkflow::new("Test", "pr_opened").disable();
        assert!(!workflow.enabled);
    }
}
