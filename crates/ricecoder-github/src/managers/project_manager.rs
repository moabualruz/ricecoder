//! Project Manager - Handles GitHub Projects management

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::{
    errors::{GitHubError, Result},
    models::{Issue, ProjectCard, PullRequest},
};

/// Project column status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColumnStatus {
    /// Todo column
    Todo,
    /// In Progress column
    InProgress,
    /// In Review column
    InReview,
    /// Done column
    Done,
}

impl ColumnStatus {
    /// Get the string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            ColumnStatus::Todo => "Todo",
            ColumnStatus::InProgress => "In Progress",
            ColumnStatus::InReview => "In Review",
            ColumnStatus::Done => "Done",
        }
    }
}

/// Project metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetrics {
    /// Total cards
    pub total_cards: u32,
    /// Cards in todo
    pub todo_count: u32,
    /// Cards in progress
    pub in_progress_count: u32,
    /// Cards in review
    pub in_review_count: u32,
    /// Cards done
    pub done_count: u32,
    /// Progress percentage (0-100)
    pub progress_percentage: u32,
}

impl ProjectMetrics {
    /// Calculate progress percentage
    pub fn calculate_progress(&self) -> u32 {
        if self.total_cards == 0 {
            return 0;
        }
        ((self.done_count as f64 / self.total_cards as f64) * 100.0) as u32
    }
}

/// Automation rule for project cards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationRule {
    /// Rule name
    pub name: String,
    /// Trigger condition (e.g., "pr_opened", "issue_labeled")
    pub trigger: String,
    /// Target column status
    pub target_column: ColumnStatus,
    /// Optional filter (e.g., label name)
    pub filter: Option<String>,
}

/// Project status report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStatusReport {
    /// Project name
    pub project_name: String,
    /// Report timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Current metrics
    pub metrics: ProjectMetrics,
    /// Cards by column
    pub cards_by_column: HashMap<String, Vec<ProjectCard>>,
    /// Recent activity
    pub recent_activity: Vec<String>,
}

/// Project Manager
#[derive(Debug, Clone)]
pub struct ProjectManager {
    /// Project ID
    project_id: u64,
    /// Project name
    project_name: String,
    /// Column mappings (status -> column_id)
    column_mappings: HashMap<ColumnStatus, u64>,
    /// Automation rules
    automation_rules: Vec<AutomationRule>,
    /// Cards cache
    cards_cache: HashMap<u64, ProjectCard>,
}

impl ProjectManager {
    /// Create a new ProjectManager
    pub fn new(project_id: u64, project_name: impl Into<String>) -> Self {
        Self {
            project_id,
            project_name: project_name.into(),
            column_mappings: HashMap::new(),
            automation_rules: Vec::new(),
            cards_cache: HashMap::new(),
        }
    }

    /// Set column mapping
    pub fn set_column_mapping(&mut self, status: ColumnStatus, column_id: u64) {
        self.column_mappings.insert(status, column_id);
        debug!("Set column mapping: {:?} -> {}", status, column_id);
    }

    /// Add automation rule
    pub fn add_automation_rule(&mut self, rule: AutomationRule) {
        info!("Adding automation rule: {}", rule.name);
        self.automation_rules.push(rule);
    }

    /// Create a project card from an issue
    pub fn create_card_from_issue(&mut self, issue: &Issue) -> Result<ProjectCard> {
        let card = ProjectCard {
            id: issue.id,
            content_id: issue.id,
            content_type: "Issue".to_string(),
            column_id: self
                .column_mappings
                .get(&ColumnStatus::Todo)
                .copied()
                .ok_or_else(|| GitHubError::config_error("Todo column not configured"))?,
            note: Some(format!("Issue #{}: {}", issue.number, issue.title)),
        };

        self.cards_cache.insert(card.id, card.clone());
        info!(
            "Created project card from issue #{}: {}",
            issue.number, issue.title
        );

        Ok(card)
    }

    /// Create a project card from a PR
    pub fn create_card_from_pr(&mut self, pr: &PullRequest) -> Result<ProjectCard> {
        let card = ProjectCard {
            id: pr.id,
            content_id: pr.id,
            content_type: "PullRequest".to_string(),
            column_id: self
                .column_mappings
                .get(&ColumnStatus::InReview)
                .copied()
                .ok_or_else(|| GitHubError::config_error("In Review column not configured"))?,
            note: Some(format!("PR #{}: {}", pr.number, pr.title)),
        };

        self.cards_cache.insert(card.id, card.clone());
        info!("Created project card from PR #{}: {}", pr.number, pr.title);

        Ok(card)
    }

    /// Move card to a column
    pub fn move_card_to_column(
        &mut self,
        card_id: u64,
        target_status: ColumnStatus,
    ) -> Result<ProjectCard> {
        let target_column_id = self
            .column_mappings
            .get(&target_status)
            .copied()
            .ok_or_else(|| {
                GitHubError::config_error(format!(
                    "{} column not configured",
                    target_status.as_str()
                ))
            })?;

        let mut card = self
            .cards_cache
            .get(&card_id)
            .cloned()
            .ok_or_else(|| GitHubError::not_found(format!("Card {} not found", card_id)))?;

        card.column_id = target_column_id;
        self.cards_cache.insert(card_id, card.clone());

        info!(
            "Moved card {} to column {} ({})",
            card_id,
            target_column_id,
            target_status.as_str()
        );

        Ok(card)
    }

    /// Get card by ID
    pub fn get_card(&self, card_id: u64) -> Result<ProjectCard> {
        self.cards_cache
            .get(&card_id)
            .cloned()
            .ok_or_else(|| GitHubError::not_found(format!("Card {} not found", card_id)))
    }

    /// Get all cards
    pub fn get_all_cards(&self) -> Vec<ProjectCard> {
        self.cards_cache.values().cloned().collect()
    }

    /// Calculate project metrics
    pub fn calculate_metrics(&self) -> ProjectMetrics {
        let total_cards = self.cards_cache.len() as u32;
        let mut metrics = ProjectMetrics {
            total_cards,
            todo_count: 0,
            in_progress_count: 0,
            in_review_count: 0,
            done_count: 0,
            progress_percentage: 0,
        };

        let todo_col = self.column_mappings.get(&ColumnStatus::Todo);
        let in_progress_col = self.column_mappings.get(&ColumnStatus::InProgress);
        let in_review_col = self.column_mappings.get(&ColumnStatus::InReview);
        let done_col = self.column_mappings.get(&ColumnStatus::Done);

        for card in self.cards_cache.values() {
            if Some(&card.column_id) == todo_col {
                metrics.todo_count += 1;
            } else if Some(&card.column_id) == in_progress_col {
                metrics.in_progress_count += 1;
            } else if Some(&card.column_id) == in_review_col {
                metrics.in_review_count += 1;
            } else if Some(&card.column_id) == done_col {
                metrics.done_count += 1;
            }
        }

        metrics.progress_percentage = metrics.calculate_progress();
        metrics
    }

    /// Apply automation rules to a card
    pub fn apply_automation_rules(&mut self, card_id: u64, trigger: &str) -> Result<()> {
        let matching_rules: Vec<_> = self
            .automation_rules
            .iter()
            .filter(|rule| rule.trigger == trigger)
            .cloned()
            .collect();

        for rule in matching_rules {
            debug!("Applying automation rule: {}", rule.name);
            self.move_card_to_column(card_id, rule.target_column)?;
        }

        Ok(())
    }

    /// Generate project status report
    pub fn generate_status_report(&self) -> ProjectStatusReport {
        let metrics = self.calculate_metrics();
        let mut cards_by_column: HashMap<String, Vec<ProjectCard>> = HashMap::new();

        for (status, column_id) in &self.column_mappings {
            let cards: Vec<_> = self
                .cards_cache
                .values()
                .filter(|card| card.column_id == *column_id)
                .cloned()
                .collect();
            cards_by_column.insert(status.as_str().to_string(), cards);
        }

        let recent_activity = vec![
            format!("Total cards: {}", metrics.total_cards),
            format!("Todo: {}", metrics.todo_count),
            format!("In Progress: {}", metrics.in_progress_count),
            format!("In Review: {}", metrics.in_review_count),
            format!("Done: {}", metrics.done_count),
            format!("Progress: {}%", metrics.progress_percentage),
        ];

        ProjectStatusReport {
            project_name: self.project_name.clone(),
            timestamp: chrono::Utc::now(),
            metrics,
            cards_by_column,
            recent_activity,
        }
    }

    /// Get project ID
    pub fn project_id(&self) -> u64 {
        self.project_id
    }

    /// Get project name
    pub fn project_name(&self) -> &str {
        &self.project_name
    }

    /// Get column mappings
    pub fn column_mappings(&self) -> &HashMap<ColumnStatus, u64> {
        &self.column_mappings
    }

    /// Get automation rules
    pub fn automation_rules(&self) -> &[AutomationRule] {
        &self.automation_rules
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_project_manager() {
        let manager = ProjectManager::new(1, "Test Project");
        assert_eq!(manager.project_id(), 1);
        assert_eq!(manager.project_name(), "Test Project");
    }

    #[test]
    fn test_set_column_mapping() {
        let mut manager = ProjectManager::new(1, "Test Project");
        manager.set_column_mapping(ColumnStatus::Todo, 100);
        assert_eq!(
            manager.column_mappings().get(&ColumnStatus::Todo),
            Some(&100)
        );
    }

    #[test]
    fn test_add_automation_rule() {
        let mut manager = ProjectManager::new(1, "Test Project");
        let rule = AutomationRule {
            name: "Test Rule".to_string(),
            trigger: "pr_opened".to_string(),
            target_column: ColumnStatus::InReview,
            filter: None,
        };
        manager.add_automation_rule(rule);
        assert_eq!(manager.automation_rules().len(), 1);
    }

    #[test]
    fn test_create_card_from_issue() {
        let mut manager = ProjectManager::new(1, "Test Project");
        manager.set_column_mapping(ColumnStatus::Todo, 100);

        let issue = Issue {
            id: 1,
            number: 1,
            title: "Test Issue".to_string(),
            body: "Test body".to_string(),
            labels: vec![],
            assignees: vec![],
            status: crate::models::IssueStatus::Open,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let card = manager.create_card_from_issue(&issue).unwrap();
        assert_eq!(card.content_id, 1);
        assert_eq!(card.content_type, "Issue");
        assert_eq!(card.column_id, 100);
    }

    #[test]
    fn test_move_card_to_column() {
        let mut manager = ProjectManager::new(1, "Test Project");
        manager.set_column_mapping(ColumnStatus::Todo, 100);
        manager.set_column_mapping(ColumnStatus::InProgress, 101);

        let issue = Issue {
            id: 1,
            number: 1,
            title: "Test Issue".to_string(),
            body: "Test body".to_string(),
            labels: vec![],
            assignees: vec![],
            status: crate::models::IssueStatus::Open,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let card = manager.create_card_from_issue(&issue).unwrap();
        assert_eq!(card.column_id, 100);

        let moved_card = manager
            .move_card_to_column(card.id, ColumnStatus::InProgress)
            .unwrap();
        assert_eq!(moved_card.column_id, 101);
    }

    #[test]
    fn test_calculate_metrics() {
        let mut manager = ProjectManager::new(1, "Test Project");
        manager.set_column_mapping(ColumnStatus::Todo, 100);
        manager.set_column_mapping(ColumnStatus::Done, 103);

        let issue1 = Issue {
            id: 1,
            number: 1,
            title: "Issue 1".to_string(),
            body: "Body 1".to_string(),
            labels: vec![],
            assignees: vec![],
            status: crate::models::IssueStatus::Open,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let issue2 = Issue {
            id: 2,
            number: 2,
            title: "Issue 2".to_string(),
            body: "Body 2".to_string(),
            labels: vec![],
            assignees: vec![],
            status: crate::models::IssueStatus::Closed,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        manager.create_card_from_issue(&issue1).unwrap();
        let card2 = manager.create_card_from_issue(&issue2).unwrap();
        manager
            .move_card_to_column(card2.id, ColumnStatus::Done)
            .unwrap();

        let metrics = manager.calculate_metrics();
        assert_eq!(metrics.total_cards, 2);
        assert_eq!(metrics.todo_count, 1);
        assert_eq!(metrics.done_count, 1);
        assert_eq!(metrics.progress_percentage, 50);
    }

    #[test]
    fn test_generate_status_report() {
        let mut manager = ProjectManager::new(1, "Test Project");
        manager.set_column_mapping(ColumnStatus::Todo, 100);

        let issue = Issue {
            id: 1,
            number: 1,
            title: "Test Issue".to_string(),
            body: "Test body".to_string(),
            labels: vec![],
            assignees: vec![],
            status: crate::models::IssueStatus::Open,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        manager.create_card_from_issue(&issue).unwrap();

        let report = manager.generate_status_report();
        assert_eq!(report.project_name, "Test Project");
        assert_eq!(report.metrics.total_cards, 1);
        assert!(!report.recent_activity.is_empty());
    }
}
