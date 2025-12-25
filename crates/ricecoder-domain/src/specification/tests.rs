//! Specification module tests
//!
//! Tests for Specification aggregate root, Requirements, and Tasks.

use crate::errors::DomainError;
use crate::specification::{SpecStatus, Specification, TaskStatus};
use crate::value_objects::{ProjectId, RequirementId};

fn create_test_specification() -> Specification {
    let project_id = ProjectId::new();
    let (spec, _) = Specification::create(
        project_id,
        "Test Spec".to_string(),
        "Test description".to_string(),
        "1.0.0".to_string(),
    )
    .unwrap();
    spec
}

#[test]
fn test_create_specification_success() {
    let project_id = ProjectId::new();
    let result = Specification::create(
        project_id,
        "My Feature".to_string(),
        "Feature description".to_string(),
        "1.0.0".to_string(),
    );

    assert!(result.is_ok());
    let (spec, event) = result.unwrap();
    assert_eq!(spec.name(), "My Feature");
    assert_eq!(spec.status(), SpecStatus::Draft);
    assert!(spec.requirements().is_empty());
    assert!(spec.tasks().is_empty());
    assert_eq!(event.event_type(), "SpecificationCreated");
}

#[test]
fn test_create_specification_empty_name_fails() {
    let project_id = ProjectId::new();
    let result = Specification::create(
        project_id,
        "".to_string(),
        "Description".to_string(),
        "1.0.0".to_string(),
    );

    assert!(result.is_err());
    match result.err().unwrap() {
        DomainError::ValidationError { field, .. } => assert_eq!(field, "name"),
        _ => panic!("Expected ValidationError"),
    }
}

#[test]
fn test_create_specification_name_too_long_fails() {
    let project_id = ProjectId::new();
    let long_name = "x".repeat(101);
    let result = Specification::create(
        project_id,
        long_name,
        "Description".to_string(),
        "1.0.0".to_string(),
    );

    assert!(result.is_err());
}

#[test]
fn test_add_requirement_success() {
    let mut spec = create_test_specification();

    let result = spec.add_requirement(
        "REQ-001: User Auth".to_string(),
        "User must be able to authenticate".to_string(),
        vec!["AC1: Login form displayed".to_string()],
    );

    assert!(result.is_ok());
    let (req_id, event) = result.unwrap();
    assert_eq!(spec.requirements().len(), 1);
    assert_eq!(spec.requirements()[0].id(), req_id);
    assert_eq!(event.event_type(), "RequirementAdded");
}

#[test]
fn test_add_requirement_without_acceptance_criteria_fails() {
    let mut spec = create_test_specification();

    let result = spec.add_requirement(
        "REQ-001".to_string(),
        "Description".to_string(),
        vec![], // Empty acceptance criteria
    );

    assert!(result.is_err());
    match result.err().unwrap() {
        DomainError::BusinessRuleViolation { rule } => {
            assert!(rule.contains("acceptance criterion"));
        }
        _ => panic!("Expected BusinessRuleViolation"),
    }
}

#[test]
fn test_add_task_success() {
    let mut spec = create_test_specification();

    // First add a requirement
    let (req_id, _) = spec
        .add_requirement(
            "REQ-001".to_string(),
            "Description".to_string(),
            vec!["AC1".to_string()],
        )
        .unwrap();

    // Now add a task referencing the requirement
    let result = spec.add_task(
        "TASK-001: Implement feature".to_string(),
        "Implementation details".to_string(),
        vec![req_id],
    );

    assert!(result.is_ok());
    let (task_id, event) = result.unwrap();
    assert_eq!(spec.tasks().len(), 1);
    assert_eq!(spec.tasks()[0].id(), task_id);
    assert_eq!(spec.tasks()[0].status(), TaskStatus::Pending);
    assert_eq!(event.event_type(), "TaskAdded");
}

#[test]
fn test_add_task_without_requirement_refs_fails() {
    let mut spec = create_test_specification();

    let result = spec.add_task(
        "TASK-001".to_string(),
        "Description".to_string(),
        vec![], // No requirement references
    );

    assert!(result.is_err());
    match result.err().unwrap() {
        DomainError::BusinessRuleViolation { rule } => {
            assert!(rule.contains("reference at least one requirement"));
        }
        _ => panic!("Expected BusinessRuleViolation"),
    }
}

#[test]
fn test_add_task_with_nonexistent_requirement_fails() {
    let mut spec = create_test_specification();

    let fake_req_id = RequirementId::new();
    let result = spec.add_task(
        "TASK-001".to_string(),
        "Description".to_string(),
        vec![fake_req_id],
    );

    assert!(result.is_err());
    match result.err().unwrap() {
        DomainError::BusinessRuleViolation { rule } => {
            assert!(rule.contains("non-existent requirement"));
        }
        _ => panic!("Expected BusinessRuleViolation"),
    }
}

#[test]
fn test_task_lifecycle_start_and_complete() {
    let mut spec = create_test_specification();

    // Add requirement and task
    let (req_id, _) = spec
        .add_requirement("REQ".to_string(), "Desc".to_string(), vec!["AC".to_string()])
        .unwrap();
    let (task_id, _) = spec
        .add_task("TASK".to_string(), "Desc".to_string(), vec![req_id])
        .unwrap();

    // Start the task
    let event = spec.start_task(task_id).unwrap();
    assert_eq!(spec.tasks()[0].status(), TaskStatus::InProgress);
    assert_eq!(event.event_type(), "TaskStarted");

    // Complete the task
    let event = spec.complete_task(task_id).unwrap();
    assert_eq!(spec.tasks()[0].status(), TaskStatus::Completed);
    assert_eq!(event.event_type(), "TaskCompleted");
}

#[test]
fn test_cannot_start_already_started_task() {
    let mut spec = create_test_specification();

    let (req_id, _) = spec
        .add_requirement("REQ".to_string(), "Desc".to_string(), vec!["AC".to_string()])
        .unwrap();
    let (task_id, _) = spec
        .add_task("TASK".to_string(), "Desc".to_string(), vec![req_id])
        .unwrap();

    // Start the task
    spec.start_task(task_id).unwrap();

    // Try to start again
    let result = spec.start_task(task_id);
    assert!(result.is_err());
}

#[test]
fn test_completion_percentage() {
    let mut spec = create_test_specification();

    // No tasks = 0%
    assert_eq!(spec.completion_percentage(), 0.0);

    // Add requirement and tasks
    let (req_id, _) = spec
        .add_requirement("REQ".to_string(), "Desc".to_string(), vec!["AC".to_string()])
        .unwrap();

    let (task1_id, _) = spec
        .add_task("TASK-1".to_string(), "Desc".to_string(), vec![req_id])
        .unwrap();
    let (task2_id, _) = spec
        .add_task("TASK-2".to_string(), "Desc".to_string(), vec![req_id])
        .unwrap();

    // 0 of 2 complete = 0%
    assert_eq!(spec.completion_percentage(), 0.0);

    // Complete task 1 (1 of 2 = 50%)
    spec.start_task(task1_id).unwrap();
    spec.complete_task(task1_id).unwrap();
    assert_eq!(spec.completion_percentage(), 50.0);

    // Complete task 2 (2 of 2 = 100%)
    spec.start_task(task2_id).unwrap();
    spec.complete_task(task2_id).unwrap();
    assert_eq!(spec.completion_percentage(), 100.0);
}

#[test]
fn test_approve_specification() {
    let mut spec = create_test_specification();

    // Add and complete tasks
    let (req_id, _) = spec
        .add_requirement("REQ".to_string(), "Desc".to_string(), vec!["AC".to_string()])
        .unwrap();
    let (task_id, _) = spec
        .add_task("TASK".to_string(), "Desc".to_string(), vec![req_id])
        .unwrap();

    spec.start_task(task_id).unwrap();
    spec.complete_task(task_id).unwrap();

    // Approve
    let event = spec.approve().unwrap();
    assert_eq!(spec.status(), SpecStatus::Complete);
    assert_eq!(event.event_type(), "SpecificationApproved");
}

#[test]
fn test_cannot_approve_incomplete_specification() {
    let mut spec = create_test_specification();

    // Add tasks but don't complete them
    let (req_id, _) = spec
        .add_requirement("REQ".to_string(), "Desc".to_string(), vec!["AC".to_string()])
        .unwrap();
    spec.add_task("TASK".to_string(), "Desc".to_string(), vec![req_id])
        .unwrap();

    // Try to approve
    let result = spec.approve();
    assert!(result.is_err());
    match result.err().unwrap() {
        DomainError::BusinessRuleViolation { rule } => {
            assert!(rule.contains("incomplete tasks"));
        }
        _ => panic!("Expected BusinessRuleViolation"),
    }
}

#[test]
fn test_update_requirement() {
    let mut spec = create_test_specification();

    let (req_id, _) = spec
        .add_requirement(
            "Original Title".to_string(),
            "Original Desc".to_string(),
            vec!["AC1".to_string()],
        )
        .unwrap();

    // Update title
    let event = spec
        .update_requirement(req_id, Some("Updated Title".to_string()), None, None)
        .unwrap();

    assert_eq!(spec.requirements()[0].title(), "Updated Title");
    assert_eq!(event.event_type(), "RequirementUpdated");
}

#[test]
fn test_approve_requirement() {
    let mut spec = create_test_specification();

    let (req_id, _) = spec
        .add_requirement("REQ".to_string(), "Desc".to_string(), vec!["AC".to_string()])
        .unwrap();

    assert!(!spec.requirements()[0].is_approved());

    let event = spec.approve_requirement(req_id).unwrap();
    assert!(spec.requirements()[0].is_approved());
    assert_eq!(event.event_type(), "RequirementApproved");
}

#[test]
fn test_archive_completed_specification() {
    let mut spec = create_test_specification();

    // Add, complete, and approve
    let (req_id, _) = spec
        .add_requirement("REQ".to_string(), "Desc".to_string(), vec!["AC".to_string()])
        .unwrap();
    let (task_id, _) = spec
        .add_task("TASK".to_string(), "Desc".to_string(), vec![req_id])
        .unwrap();

    spec.start_task(task_id).unwrap();
    spec.complete_task(task_id).unwrap();
    spec.approve().unwrap();

    // Archive
    let event = spec.archive().unwrap();
    assert_eq!(spec.status(), SpecStatus::Archived);
    assert_eq!(event.event_type(), "SpecificationImplemented");
}

#[test]
fn test_cannot_archive_incomplete_specification() {
    // Spec is in Draft state
    let result = Specification::create(
        ProjectId::new(),
        "Test".to_string(),
        "Desc".to_string(),
        "1.0.0".to_string(),
    )
    .unwrap()
    .0;

    // Try to archive
    let result = {
        let mut spec = result;
        spec.archive()
    };
    assert!(result.is_err());
}
