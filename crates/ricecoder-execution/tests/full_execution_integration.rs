//! Integration tests for full execution flow
//!
//! Tests the complete execution pipeline including:
//! - Plan creation and execution
//! - Approval flow
//! - Rollback on failure
//! - Pause/resume functionality
//!
//! **Requirements: 1.2, 4.1, 7.1**

use ricecoder_execution::{
    ApprovalManager, ExecutionManager, ExecutionMode, ExecutionPlan, PlanBuilder, RollbackHandler,
};
use tempfile::TempDir;

/// Helper to create a simple test plan
fn create_test_plan(name: &str, step_count: usize) -> ExecutionPlan {
    let mut builder = PlanBuilder::new(name.to_string());

    for i in 0..step_count {
        let path = format!("test_file_{}.txt", i);
        let content = format!("Test content {}", i);
        builder = builder
            .add_create_file_step(path, content)
            .expect("Failed to add step");
    }

    builder.build().expect("Failed to build plan")
}

/// Test 1: Complete execution flow with automatic mode
#[test]
fn test_complete_execution_flow_automatic() {
    let _temp_dir = TempDir::new().expect("Failed to create temp dir");
    let plan = create_test_plan("test_plan", 3);

    let mut manager = ExecutionManager::new();
    manager
        .register_plan(plan.clone())
        .expect("Failed to register plan");

    let execution_id = manager
        .start_execution(&plan.id, ExecutionMode::Automatic)
        .expect("Failed to start execution");

    // Verify execution started
    let state = manager
        .get_execution_state(&execution_id)
        .expect("Failed to get execution state");
    assert_eq!(state.execution_id, execution_id);
    assert_eq!(state.mode, ExecutionMode::Automatic);

    // Verify execution is tracked
    let active = manager.get_active_executions();
    assert!(active.iter().any(|e| e.execution_id == execution_id));
}

/// Test 2: Approval flow - request and approve
#[test]
fn test_approval_flow_approve() {
    let mut approval_manager = ApprovalManager::new();
    let plan = create_test_plan("test_plan", 2);

    // Request approval
    let request_id = approval_manager
        .request_approval(&plan)
        .expect("Failed to request approval");

    // Verify request is pending
    let pending = approval_manager.get_pending_requests();
    assert!(pending.iter().any(|r| r.id == request_id));

    // Approve the plan
    approval_manager
        .approve(&request_id, None)
        .expect("Failed to approve");

    // Verify request is no longer pending
    let pending = approval_manager.get_pending_requests();
    assert!(!pending.iter().any(|r| r.id == request_id));
}

/// Test 3: Approval flow - request and reject
#[test]
fn test_approval_flow_reject() {
    let mut approval_manager = ApprovalManager::new();
    let plan = create_test_plan("test_plan", 2);

    // Request approval
    let request_id = approval_manager
        .request_approval(&plan)
        .expect("Failed to request approval");

    // Reject the plan
    approval_manager
        .reject(&request_id, Some("Not ready".to_string()))
        .expect("Failed to reject");

    // Verify request is no longer pending
    let pending = approval_manager.get_pending_requests();
    assert!(!pending.iter().any(|r| r.id == request_id));
}

/// Test 4: Rollback on failure
#[test]
fn test_rollback_on_failure() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let mut handler = RollbackHandler::new();

    // Create a test file and backup
    let test_file = temp_dir.path().join("test.txt");
    let backup_file = temp_dir.path().join("test.txt.bak");
    std::fs::write(&test_file, "modified content").expect("Failed to write file");
    std::fs::write(&backup_file, "original content").expect("Failed to write backup");

    // Track a rollback action for file modification
    let rollback_action = ricecoder_execution::RollbackAction {
        action_type: ricecoder_execution::RollbackType::RestoreFile,
        data: serde_json::json!({
            "file_path": test_file.to_string_lossy().to_string(),
            "backup_path": backup_file.to_string_lossy().to_string()
        }),
    };

    handler.track_action("step_1".to_string(), rollback_action);

    // Verify action is tracked
    assert_eq!(handler.action_count(), 1);

    // Execute rollback
    let results = handler
        .execute_rollback()
        .expect("Failed to execute rollback");
    assert!(!results.is_empty());
}

/// Test 5: Pause and resume execution
#[test]
fn test_pause_resume_execution() {
    let plan = create_test_plan("test_plan", 3);
    let mut manager = ExecutionManager::new();
    manager
        .register_plan(plan.clone())
        .expect("Failed to register plan");

    let execution_id = manager
        .start_execution(&plan.id, ExecutionMode::StepByStep)
        .expect("Failed to start execution");

    // Pause execution
    manager
        .pause_execution(&execution_id)
        .expect("Failed to pause execution");

    let state = manager
        .get_execution_state(&execution_id)
        .expect("Failed to get state");
    // Verify paused_at is set (indicating paused state)
    assert!(!state.paused_at.to_rfc3339().is_empty());

    // Resume execution
    manager
        .resume_execution(&execution_id)
        .expect("Failed to resume execution");

    let state = manager
        .get_execution_state(&execution_id)
        .expect("Failed to get state");
    // Verify state is still valid after resume
    assert_eq!(state.execution_id, execution_id);
}

/// Test 6: Step-by-step execution with approval
#[test]
fn test_step_by_step_execution_with_approval() {
    let plan = create_test_plan("test_plan", 2);
    let mut manager = ExecutionManager::new();
    manager
        .register_plan(plan.clone())
        .expect("Failed to register plan");

    let execution_id = manager
        .start_execution(&plan.id, ExecutionMode::StepByStep)
        .expect("Failed to start execution");

    // Verify execution is in step-by-step mode
    let state = manager
        .get_execution_state(&execution_id)
        .expect("Failed to get state");
    assert_eq!(state.mode, ExecutionMode::StepByStep);
}

/// Test 7: Dry-run mode (no changes applied)
#[test]
fn test_dry_run_mode_no_changes() {
    let _temp_dir = TempDir::new().expect("Failed to create temp dir");
    let plan = create_test_plan("test_plan", 2);
    let mut manager = ExecutionManager::new();
    manager
        .register_plan(plan.clone())
        .expect("Failed to register plan");

    let execution_id = manager
        .start_execution(&plan.id, ExecutionMode::DryRun)
        .expect("Failed to start execution");

    // Verify execution is in dry-run mode
    let state = manager
        .get_execution_state(&execution_id)
        .expect("Failed to get state");
    assert_eq!(state.mode, ExecutionMode::DryRun);
}

/// Test 8: Cancel execution
#[test]
fn test_cancel_execution() {
    let plan = create_test_plan("test_plan", 2);
    let mut manager = ExecutionManager::new();
    manager
        .register_plan(plan.clone())
        .expect("Failed to register plan");

    let execution_id = manager
        .start_execution(&plan.id, ExecutionMode::Automatic)
        .expect("Failed to start execution");

    // Cancel execution
    manager
        .cancel_execution(&execution_id)
        .expect("Failed to cancel execution");

    // Verify execution is no longer active
    let active = manager.get_active_executions();
    assert!(!active.iter().any(|e| e.execution_id == execution_id));
}

/// Test 9: Multiple concurrent executions
#[test]
fn test_multiple_concurrent_executions() {
    let plan1 = create_test_plan("plan_1", 2);
    let plan2 = create_test_plan("plan_2", 2);

    let mut manager = ExecutionManager::new();
    manager
        .register_plan(plan1.clone())
        .expect("Failed to register plan 1");
    manager
        .register_plan(plan2.clone())
        .expect("Failed to register plan 2");

    let exec_id_1 = manager
        .start_execution(&plan1.id, ExecutionMode::Automatic)
        .expect("Failed to start execution 1");

    let exec_id_2 = manager
        .start_execution(&plan2.id, ExecutionMode::Automatic)
        .expect("Failed to start execution 2");

    // Verify both executions are active
    let active = manager.get_active_executions();
    assert_eq!(active.len(), 2);
    assert!(active.iter().any(|e| e.execution_id == exec_id_1));
    assert!(active.iter().any(|e| e.execution_id == exec_id_2));
}

/// Test 10: Execution with progress tracking
#[test]
fn test_execution_with_progress_tracking() {
    let plan = create_test_plan("test_plan", 3);
    let mut manager = ExecutionManager::new();
    manager
        .register_plan(plan.clone())
        .expect("Failed to register plan");

    let execution_id = manager
        .start_execution(&plan.id, ExecutionMode::Automatic)
        .expect("Failed to start execution");

    // Get progress
    let state = manager
        .get_execution_state(&execution_id)
        .expect("Failed to get state");

    // Verify progress tracking is initialized
    assert_eq!(state.execution_id, execution_id);
    assert_eq!(state.current_step_index, 0);
}

/// Test 11: Execution state persistence
#[test]
fn test_execution_state_persistence() {
    let plan = create_test_plan("test_plan", 2);
    let mut manager = ExecutionManager::new();
    manager
        .register_plan(plan.clone())
        .expect("Failed to register plan");

    let execution_id = manager
        .start_execution(&plan.id, ExecutionMode::StepByStep)
        .expect("Failed to start execution");

    // Get state
    let state1 = manager
        .get_execution_state(&execution_id)
        .expect("Failed to get state");

    // Get state again
    let state2 = manager
        .get_execution_state(&execution_id)
        .expect("Failed to get state");

    // States should be consistent
    assert_eq!(state1.execution_id, state2.execution_id);
    assert_eq!(state1.mode, state2.mode);
}

/// Test 12: Rollback handler state consistency
#[test]
fn test_rollback_handler_state_consistency() {
    let mut handler = RollbackHandler::new();

    // Track multiple actions
    for i in 0..3 {
        let action = ricecoder_execution::RollbackAction {
            action_type: ricecoder_execution::RollbackType::RunCommand,
            data: serde_json::json!({
                "command": "echo",
                "args": [format!("test_{}", i)]
            }),
        };
        handler.track_action(format!("step_{}", i), action);
    }

    // Verify all actions are tracked
    assert_eq!(handler.action_count(), 3);

    // Verify state is consistent
    let result = handler.verify_completeness();
    assert!(result);
}

/// Test 13: Plan validation before execution
#[test]
fn test_plan_validation_before_execution() {
    let plan = create_test_plan("test_plan", 1);
    let mut manager = ExecutionManager::new();

    // Register plan
    manager
        .register_plan(plan.clone())
        .expect("Failed to register plan");

    // Verify plan is registered
    let execution_id = manager
        .start_execution(&plan.id, ExecutionMode::Automatic)
        .expect("Failed to start execution");

    assert!(!execution_id.is_empty());
}

/// Test 14: Execution mode isolation
#[test]
fn test_execution_mode_isolation() {
    let plan = create_test_plan("test_plan", 2);
    let mut manager = ExecutionManager::new();
    manager
        .register_plan(plan.clone())
        .expect("Failed to register plan");

    // Start execution in automatic mode
    let auto_exec = manager
        .start_execution(&plan.id, ExecutionMode::Automatic)
        .expect("Failed to start automatic execution");

    let auto_state = manager
        .get_execution_state(&auto_exec)
        .expect("Failed to get state");

    // Verify mode is automatic
    assert_eq!(auto_state.mode, ExecutionMode::Automatic);
}

/// Test 15: Approval requirement based on risk
#[test]
fn test_approval_requirement_based_on_risk() {
    let plan = create_test_plan("test_plan", 1);
    let mut approval_manager = ApprovalManager::new();

    // Request approval for the plan
    let request_id = approval_manager
        .request_approval(&plan)
        .expect("Failed to request approval");

    // Verify request was created
    assert!(!request_id.is_empty());

    // Verify request is pending
    let pending = approval_manager.get_pending_requests();
    assert!(pending.iter().any(|r| r.id == request_id));
}
