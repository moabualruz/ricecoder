//! Unit tests for Project Manager
//!
//! These tests verify specific functionality and edge cases

use ricecoder_github::{
    models::{Issue, IssueStatus, PullRequest, PrStatus},
    AutomationRule, ColumnStatus, ProjectManager,
};

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
    manager.set_column_mapping(ColumnStatus::InProgress, 101);

    let mappings = manager.column_mappings();
    assert_eq!(mappings.get(&ColumnStatus::Todo), Some(&100));
    assert_eq!(mappings.get(&ColumnStatus::InProgress), Some(&101));
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

    manager.add_automation_rule(rule.clone());
    assert_eq!(manager.automation_rules().len(), 1);
    assert_eq!(manager.automation_rules()[0].name, "Test Rule");
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
        status: IssueStatus::Open,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let card = manager.create_card_from_issue(&issue).unwrap();
    assert_eq!(card.content_id, 1);
    assert_eq!(card.content_type, "Issue");
    assert_eq!(card.column_id, 100);
    assert!(card.note.is_some());
    assert!(card.note.unwrap().contains("Issue #1"));
}

#[test]
fn test_create_card_from_pr() {
    let mut manager = ProjectManager::new(1, "Test Project");
    manager.set_column_mapping(ColumnStatus::InReview, 101);

    let pr = PullRequest {
        id: 2,
        number: 2,
        title: "Test PR".to_string(),
        body: "Test body".to_string(),
        branch: "feature/test".to_string(),
        base: "main".to_string(),
        status: PrStatus::Open,
        files: vec![],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let card = manager.create_card_from_pr(&pr).unwrap();
    assert_eq!(card.content_id, 2);
    assert_eq!(card.content_type, "PullRequest");
    assert_eq!(card.column_id, 101);
    assert!(card.note.is_some());
    assert!(card.note.unwrap().contains("PR #2"));
}

#[test]
fn test_create_card_fails_without_column_mapping() {
    let mut manager = ProjectManager::new(1, "Test Project");
    // Don't set column mapping

    let issue = Issue {
        id: 1,
        number: 1,
        title: "Test Issue".to_string(),
        body: "Test body".to_string(),
        labels: vec![],
        assignees: vec![],
        status: IssueStatus::Open,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let result = manager.create_card_from_issue(&issue);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Todo column not configured"));
}

#[test]
fn test_move_card_to_column() {
    let mut manager = ProjectManager::new(1, "Test Project");
    manager.set_column_mapping(ColumnStatus::Todo, 100);
    manager.set_column_mapping(ColumnStatus::InProgress, 101);
    manager.set_column_mapping(ColumnStatus::Done, 103);

    let issue = Issue {
        id: 1,
        number: 1,
        title: "Test Issue".to_string(),
        body: "Test body".to_string(),
        labels: vec![],
        assignees: vec![],
        status: IssueStatus::Open,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let card = manager.create_card_from_issue(&issue).unwrap();
    assert_eq!(card.column_id, 100);

    let moved_card = manager
        .move_card_to_column(card.id, ColumnStatus::InProgress)
        .unwrap();
    assert_eq!(moved_card.column_id, 101);

    let moved_card = manager
        .move_card_to_column(card.id, ColumnStatus::Done)
        .unwrap();
    assert_eq!(moved_card.column_id, 103);
}

#[test]
fn test_move_card_fails_without_target_column_mapping() {
    let mut manager = ProjectManager::new(1, "Test Project");
    manager.set_column_mapping(ColumnStatus::Todo, 100);

    let issue = Issue {
        id: 1,
        number: 1,
        title: "Test Issue".to_string(),
        body: "Test body".to_string(),
        labels: vec![],
        assignees: vec![],
        status: IssueStatus::Open,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let card = manager.create_card_from_issue(&issue).unwrap();

    let result = manager.move_card_to_column(card.id, ColumnStatus::InProgress);
    assert!(result.is_err());
}

#[test]
fn test_get_card() {
    let mut manager = ProjectManager::new(1, "Test Project");
    manager.set_column_mapping(ColumnStatus::Todo, 100);

    let issue = Issue {
        id: 1,
        number: 1,
        title: "Test Issue".to_string(),
        body: "Test body".to_string(),
        labels: vec![],
        assignees: vec![],
        status: IssueStatus::Open,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let card = manager.create_card_from_issue(&issue).unwrap();
    let retrieved_card = manager.get_card(card.id).unwrap();
    assert_eq!(retrieved_card.id, card.id);
    assert_eq!(retrieved_card.content_id, 1);
}

#[test]
fn test_get_card_not_found() {
    let mut manager = ProjectManager::new(1, "Test Project");
    manager.set_column_mapping(ColumnStatus::Todo, 100);
    let result = manager.get_card(999);
    assert!(result.is_err());
}

#[test]
fn test_get_all_cards() {
    let mut manager = ProjectManager::new(1, "Test Project");
    manager.set_column_mapping(ColumnStatus::Todo, 100);

    let issue1 = Issue {
        id: 1,
        number: 1,
        title: "Issue 1".to_string(),
        body: "Body 1".to_string(),
        labels: vec![],
        assignees: vec![],
        status: IssueStatus::Open,
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
        status: IssueStatus::Open,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    manager.create_card_from_issue(&issue1).unwrap();
    manager.create_card_from_issue(&issue2).unwrap();

    let cards = manager.get_all_cards();
    assert_eq!(cards.len(), 2);
}

#[test]
fn test_calculate_metrics_empty_project() {
    let mut manager = ProjectManager::new(1, "Test Project");
    manager.set_column_mapping(ColumnStatus::Todo, 100);

    let metrics = manager.calculate_metrics();
    assert_eq!(metrics.total_cards, 0);
    assert_eq!(metrics.todo_count, 0);
    assert_eq!(metrics.done_count, 0);
    assert_eq!(metrics.progress_percentage, 0);
}

#[test]
fn test_calculate_metrics_with_cards() {
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
        status: IssueStatus::Open,
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
        status: IssueStatus::Closed,
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
fn test_apply_automation_rules() {
    let mut manager = ProjectManager::new(1, "Test Project");
    manager.set_column_mapping(ColumnStatus::Todo, 100);

    let issue = Issue {
        id: 1,
        number: 1,
        title: "Test Issue".to_string(),
        body: "Test body".to_string(),
        labels: vec![],
        assignees: vec![],
        status: IssueStatus::Open,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let card = manager.create_card_from_issue(&issue).unwrap();

    // Apply automation rules (no matching rules, so card should stay in place)
    let result = manager.apply_automation_rules(card.id, "pr_opened");
    assert!(result.is_ok());

    let updated_card = manager.get_card(card.id).unwrap();
    assert_eq!(updated_card.column_id, 100);
}

#[test]
fn test_generate_status_report() {
    let mut manager = ProjectManager::new(1, "Test Project");
    manager.set_column_mapping(ColumnStatus::Todo, 100);
    manager.set_column_mapping(ColumnStatus::InProgress, 101);
    manager.set_column_mapping(ColumnStatus::InReview, 102);
    manager.set_column_mapping(ColumnStatus::Done, 103);

    let issue = Issue {
        id: 1,
        number: 1,
        title: "Test Issue".to_string(),
        body: "Test body".to_string(),
        labels: vec![],
        assignees: vec![],
        status: IssueStatus::Open,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    manager.create_card_from_issue(&issue).unwrap();

    let report = manager.generate_status_report();
    assert_eq!(report.project_name, "Test Project");
    assert_eq!(report.metrics.total_cards, 1);
    assert!(!report.cards_by_column.is_empty());
    assert!(!report.recent_activity.is_empty());
}

#[test]
fn test_status_report_contains_all_columns() {
    let mut manager = ProjectManager::new(1, "Test Project");
    manager.set_column_mapping(ColumnStatus::Todo, 100);
    manager.set_column_mapping(ColumnStatus::InProgress, 101);
    manager.set_column_mapping(ColumnStatus::InReview, 102);
    manager.set_column_mapping(ColumnStatus::Done, 103);

    let report = manager.generate_status_report();
    assert!(report.cards_by_column.contains_key("Todo"));
    assert!(report.cards_by_column.contains_key("In Progress"));
    assert!(report.cards_by_column.contains_key("In Review"));
    assert!(report.cards_by_column.contains_key("Done"));
}

#[test]
fn test_multiple_cards_in_different_columns() {
    let mut manager = ProjectManager::new(1, "Test Project");
    manager.set_column_mapping(ColumnStatus::Todo, 100);
    manager.set_column_mapping(ColumnStatus::InProgress, 101);
    manager.set_column_mapping(ColumnStatus::Done, 103);

    let issue1 = Issue {
        id: 1,
        number: 1,
        title: "Issue 1".to_string(),
        body: "Body 1".to_string(),
        labels: vec![],
        assignees: vec![],
        status: IssueStatus::Open,
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
        status: IssueStatus::Open,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let issue3 = Issue {
        id: 3,
        number: 3,
        title: "Issue 3".to_string(),
        body: "Body 3".to_string(),
        labels: vec![],
        assignees: vec![],
        status: IssueStatus::Closed,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let _card1 = manager.create_card_from_issue(&issue1).unwrap();
    let card2 = manager.create_card_from_issue(&issue2).unwrap();
    let card3 = manager.create_card_from_issue(&issue3).unwrap();

    manager
        .move_card_to_column(card2.id, ColumnStatus::InProgress)
        .unwrap();
    manager
        .move_card_to_column(card3.id, ColumnStatus::Done)
        .unwrap();

    let metrics = manager.calculate_metrics();
    assert_eq!(metrics.total_cards, 3);
    assert_eq!(metrics.todo_count, 1);
    assert_eq!(metrics.in_progress_count, 1);
    assert_eq!(metrics.done_count, 1);
    assert_eq!(metrics.progress_percentage, 33);
}
