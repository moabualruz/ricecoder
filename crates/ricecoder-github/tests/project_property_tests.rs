//! Property-based tests for Project Manager
//!
//! These tests verify correctness properties that should hold across all valid inputs

use proptest::prelude::*;
use ricecoder_github::{
    models::{Issue, IssueStatus, PrStatus, ProjectCard, PullRequest},
    ColumnStatus, ProjectManager, ProjectOperations,
};
use std::collections::HashMap;

// Strategy for generating valid project IDs
fn project_id_strategy() -> impl Strategy<Value = u64> {
    1u64..1000000
}

// Strategy for generating valid project names
fn project_name_strategy() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9 \-_]{1,50}"
        .prop_map(|s| s.trim().to_string())
        .prop_filter("name must not be empty", |s| !s.is_empty())
}

// Strategy for generating valid column IDs
fn column_id_strategy() -> impl Strategy<Value = u64> {
    100u64..10000
}

// Strategy for generating issues
fn issue_strategy() -> impl Strategy<Value = Issue> {
    (
        1u64..1000000,
        1u32..10000,
        r"[a-zA-Z0-9 ]{1,50}",
        r"[a-zA-Z0-9 .,]{1,100}",
    )
        .prop_map(|(id, number, title, body)| Issue {
            id,
            number,
            title,
            body,
            labels: vec![],
            assignees: vec![],
            status: IssueStatus::Open,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
}

// Strategy for generating pull requests
fn pull_request_strategy() -> impl Strategy<Value = PullRequest> {
    (
        1u64..1000000,
        1u32..10000,
        r"[a-zA-Z0-9 ]{1,50}",
        r"[a-zA-Z0-9 .,]{1,100}",
        r"feature/[a-z0-9\-]{1,30}",
    )
        .prop_map(|(id, number, title, body, branch)| PullRequest {
            id,
            number,
            title,
            body,
            branch,
            base: "main".to_string(),
            status: PrStatus::Open,
            files: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
}

// **Feature: ricecoder-github, Property 21: Project Card Creation**
// *For any* issue or PR, the system SHALL create a corresponding project card in the configured project.
proptest! {
    #[test]
    fn prop_project_card_creation_from_issue(
        project_id in project_id_strategy(),
        project_name in project_name_strategy(),
        issue in issue_strategy(),
    ) {
        let mut manager = ProjectManager::new(project_id, project_name.clone());
        manager.set_column_mapping(ColumnStatus::Todo, 100);

        let result = manager.create_card_from_issue(&issue);
        prop_assert!(result.is_ok(), "Card creation should succeed");

        let card = result.unwrap();
        prop_assert_eq!(card.content_id, issue.id, "Card content_id should match issue id");
        prop_assert_eq!(card.content_type, "Issue", "Card content_type should be Issue");
        prop_assert_eq!(card.column_id, 100, "Card should be in Todo column");
        prop_assert!(card.note.is_some(), "Card should have a note");
    }

    #[test]
    fn prop_project_card_creation_from_pr(
        project_id in project_id_strategy(),
        project_name in project_name_strategy(),
        pr in pull_request_strategy(),
    ) {
        let mut manager = ProjectManager::new(project_id, project_name.clone());
        manager.set_column_mapping(ColumnStatus::InReview, 101);

        let result = manager.create_card_from_pr(&pr);
        prop_assert!(result.is_ok(), "Card creation should succeed");

        let card = result.unwrap();
        prop_assert_eq!(card.content_id, pr.id, "Card content_id should match PR id");
        prop_assert_eq!(card.content_type, "PullRequest", "Card content_type should be PullRequest");
        prop_assert_eq!(card.column_id, 101, "Card should be in InReview column");
        prop_assert!(card.note.is_some(), "Card should have a note");
    }

    #[test]
    fn prop_card_creation_fails_without_column_mapping(
        project_id in project_id_strategy(),
        project_name in project_name_strategy(),
        issue in issue_strategy(),
    ) {
        let mut manager = ProjectManager::new(project_id, project_name.clone());
        // Don't set column mapping

        let result = manager.create_card_from_issue(&issue);
        prop_assert!(result.is_err(), "Card creation should fail without column mapping");
    }
}

// **Feature: ricecoder-github, Property 22: Card Column Movement**
// *For any* status change, the corresponding project card SHALL move to the correct column.
proptest! {
    #[test]
    fn prop_card_column_movement(
        project_id in project_id_strategy(),
        project_name in project_name_strategy(),
        issue in issue_strategy(),
    ) {
        let mut manager = ProjectManager::new(project_id, project_name.clone());
        manager.set_column_mapping(ColumnStatus::Todo, 100);
        manager.set_column_mapping(ColumnStatus::InProgress, 101);
        manager.set_column_mapping(ColumnStatus::Done, 103);

        let card = manager.create_card_from_issue(&issue).unwrap();
        prop_assert_eq!(card.column_id, 100, "Card should start in Todo");

        let moved_card = manager.move_card_to_column(card.id, ColumnStatus::InProgress).unwrap();
        prop_assert_eq!(moved_card.column_id, 101, "Card should move to InProgress");

        let moved_card = manager.move_card_to_column(card.id, ColumnStatus::Done).unwrap();
        prop_assert_eq!(moved_card.column_id, 103, "Card should move to Done");
    }

    #[test]
    fn prop_card_movement_idempotent(
        project_id in project_id_strategy(),
        project_name in project_name_strategy(),
        issue in issue_strategy(),
    ) {
        let mut manager = ProjectManager::new(project_id, project_name.clone());
        manager.set_column_mapping(ColumnStatus::Todo, 100);
        manager.set_column_mapping(ColumnStatus::InProgress, 101);

        let card = manager.create_card_from_issue(&issue).unwrap();

        let moved_card1 = manager.move_card_to_column(card.id, ColumnStatus::InProgress).unwrap();
        let moved_card2 = manager.move_card_to_column(card.id, ColumnStatus::InProgress).unwrap();

        prop_assert_eq!(moved_card1.column_id, moved_card2.column_id, "Moving to same column twice should be idempotent");
    }
}

// **Feature: ricecoder-github, Property 23: Project Progress Tracking**
// *For any* project, the system SHALL calculate and update progress metrics correctly.
proptest! {
    #[test]
    fn prop_progress_metrics_calculation(
        project_id in project_id_strategy(),
        project_name in project_name_strategy(),
        num_issues in 1usize..20,
    ) {
        let mut manager = ProjectManager::new(project_id, project_name.clone());
        manager.set_column_mapping(ColumnStatus::Todo, 100);
        manager.set_column_mapping(ColumnStatus::Done, 103);

        // Create issues
        for i in 0..num_issues {
            let issue = Issue {
                id: i as u64,
                number: i as u32,
                title: format!("Issue {}", i),
                body: "Test".to_string(),
                labels: vec![],
                assignees: vec![],
                status: IssueStatus::Open,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };
            manager.create_card_from_issue(&issue).unwrap();
        }

        let metrics = manager.calculate_metrics();
        prop_assert_eq!(metrics.total_cards, num_issues as u32, "Total cards should match created issues");
        prop_assert_eq!(metrics.todo_count, num_issues as u32, "All cards should be in Todo initially");
        prop_assert_eq!(metrics.done_count, 0, "No cards should be done initially");
        prop_assert_eq!(metrics.progress_percentage, 0, "Progress should be 0% initially");
    }

    #[test]
    fn prop_progress_percentage_increases_with_done_cards(
        project_id in project_id_strategy(),
        project_name in project_name_strategy(),
        num_issues in 2usize..20,
    ) {
        let mut manager = ProjectManager::new(project_id, project_name.clone());
        manager.set_column_mapping(ColumnStatus::Todo, 100);
        manager.set_column_mapping(ColumnStatus::Done, 103);

        // Create issues
        for i in 0..num_issues {
            let issue = Issue {
                id: i as u64,
                number: i as u32,
                title: format!("Issue {}", i),
                body: "Test".to_string(),
                labels: vec![],
                assignees: vec![],
                status: IssueStatus::Open,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };
            manager.create_card_from_issue(&issue).unwrap();
        }

        let initial_metrics = manager.calculate_metrics();
        prop_assert_eq!(initial_metrics.progress_percentage, 0);

        // Move half to done
        let cards = manager.get_all_cards();
        for (i, card) in cards.iter().enumerate() {
            if i < num_issues / 2 {
                manager.move_card_to_column(card.id, ColumnStatus::Done).unwrap();
            }
        }

        let updated_metrics = manager.calculate_metrics();
        prop_assert!(updated_metrics.progress_percentage > initial_metrics.progress_percentage, "Progress should increase");
        prop_assert!(updated_metrics.progress_percentage <= 100, "Progress should not exceed 100%");
    }
}

// **Feature: ricecoder-github, Property 24: Automation Rule Application**
// *For any* configured automation rule, the system SHALL apply the rule to matching cards.
proptest! {
    #[test]
    fn prop_automation_rule_application(
        project_id in project_id_strategy(),
        project_name in project_name_strategy(),
        issue in issue_strategy(),
    ) {
        let mut manager = ProjectManager::new(project_id, project_name.clone());
        manager.set_column_mapping(ColumnStatus::Todo, 100);
        manager.set_column_mapping(ColumnStatus::InProgress, 101);

        let card = manager.create_card_from_issue(&issue).unwrap();
        prop_assert_eq!(card.column_id, 100);

        // Apply automation rule
        let result = manager.apply_automation_rules(card.id, "pr_opened");
        prop_assert!(result.is_ok(), "Automation rule application should succeed");

        // Verify card is still in original column (no matching rules)
        let updated_card = manager.get_card(card.id).unwrap();
        prop_assert_eq!(updated_card.column_id, 100, "Card should remain in original column without matching rules");
    }
}

// **Feature: ricecoder-github, Property 25: Project Status Report Generation**
// *For any* project, the system SHALL generate a status report containing current metrics and card distribution.
proptest! {
    #[test]
    fn prop_status_report_generation(
        project_id in project_id_strategy(),
        project_name in project_name_strategy(),
        num_issues in 1usize..10,
    ) {
        let mut manager = ProjectManager::new(project_id, project_name.clone());
        manager.set_column_mapping(ColumnStatus::Todo, 100);
        manager.set_column_mapping(ColumnStatus::InProgress, 101);
        manager.set_column_mapping(ColumnStatus::Done, 103);

        // Create issues
        for i in 0..num_issues {
            let issue = Issue {
                id: i as u64,
                number: i as u32,
                title: format!("Issue {}", i),
                body: "Test".to_string(),
                labels: vec![],
                assignees: vec![],
                status: IssueStatus::Open,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };
            manager.create_card_from_issue(&issue).unwrap();
        }

        let report = manager.generate_status_report();
        prop_assert_eq!(report.project_name, project_name, "Report should have correct project name");
        prop_assert_eq!(report.metrics.total_cards, num_issues as u32, "Report should have correct total cards");
        prop_assert!(!report.cards_by_column.is_empty(), "Report should have cards by column");
        prop_assert!(!report.recent_activity.is_empty(), "Report should have recent activity");
    }

    #[test]
    fn prop_status_report_contains_all_columns(
        project_id in project_id_strategy(),
        project_name in project_name_strategy(),
    ) {
        let mut manager = ProjectManager::new(project_id, project_name.clone());
        manager.set_column_mapping(ColumnStatus::Todo, 100);
        manager.set_column_mapping(ColumnStatus::InProgress, 101);
        manager.set_column_mapping(ColumnStatus::InReview, 102);
        manager.set_column_mapping(ColumnStatus::Done, 103);

        let report = manager.generate_status_report();
        prop_assert!(report.cards_by_column.contains_key("Todo"), "Report should contain Todo column");
        prop_assert!(report.cards_by_column.contains_key("In Progress"), "Report should contain In Progress column");
        prop_assert!(report.cards_by_column.contains_key("In Review"), "Report should contain In Review column");
        prop_assert!(report.cards_by_column.contains_key("Done"), "Report should contain Done column");
    }
}
