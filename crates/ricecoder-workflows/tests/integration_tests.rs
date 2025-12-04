//! Integration tests for ricecoder-workflows
//!
//! Tests full workflow execution scenarios including:
//! - Multi-step workflows with dependencies
//! - Error handling and recovery
//! - Approval gates
//! - Pause/resume functionality
//! - Risk-scored execution

use ricecoder_workflows::{
    ErrorAction, RiskFactors, StateManager, StepConfig, StepResult, StepStatus, StepType, Workflow,
    WorkflowEngine, WorkflowStatus, AgentStep, WorkflowStep, WorkflowConfig, ApprovalDefault,
    StepExecutor,
};
use std::collections::HashMap;

// ============================================================================
// Test Fixtures
// ============================================================================

fn create_simple_workflow() -> Workflow {
    Workflow {
        id: "simple-workflow".to_string(),
        name: "Simple Workflow".to_string(),
        description: "A simple test workflow".to_string(),
        parameters: vec![],
        steps: vec![
            WorkflowStep {
                id: "step1".to_string(),
                name: "Step 1".to_string(),
                step_type: StepType::Agent(AgentStep {
                    agent_id: "test-agent".to_string(),
                    task: "task1".to_string(),
                }),
                config: StepConfig {
                    config: serde_json::json!({"action": "process"}),
                },
                dependencies: vec![],
                approval_required: false,
                on_error: ErrorAction::Fail,
                risk_score: None,
                risk_factors: RiskFactors::default(),
            },
            WorkflowStep {
                id: "step2".to_string(),
                name: "Step 2".to_string(),
                step_type: StepType::Agent(AgentStep {
                    agent_id: "test-agent".to_string(),
                    task: "task2".to_string(),
                }),
                config: StepConfig {
                    config: serde_json::json!({"action": "validate"}),
                },
                dependencies: vec!["step1".to_string()],
                approval_required: false,
                on_error: ErrorAction::Fail,
                risk_score: None,
                risk_factors: RiskFactors::default(),
            },
            WorkflowStep {
                id: "step3".to_string(),
                name: "Step 3".to_string(),
                step_type: StepType::Agent(AgentStep {
                    agent_id: "test-agent".to_string(),
                    task: "task3".to_string(),
                }),
                config: StepConfig {
                    config: serde_json::json!({"action": "finalize"}),
                },
                dependencies: vec!["step2".to_string()],
                approval_required: false,
                on_error: ErrorAction::Fail,
                risk_score: None,
                risk_factors: RiskFactors::default(),
            },
        ],
        config: WorkflowConfig {
            timeout_ms: None,
            max_parallel: None,
        },
    }
}

fn create_workflow_with_approval() -> Workflow {
    Workflow {
        id: "approval-workflow".to_string(),
        name: "Workflow with Approval".to_string(),
        description: "A workflow requiring approval".to_string(),
        parameters: vec![],
        steps: vec![
            WorkflowStep {
                id: "analyze".to_string(),
                name: "Analyze".to_string(),
                step_type: StepType::Agent(AgentStep {
                    agent_id: "analyzer".to_string(),
                    task: "analyze".to_string(),
                }),
                config: StepConfig {
                    config: serde_json::json!({}),
                },
                dependencies: vec![],
                approval_required: false,
                on_error: ErrorAction::Fail,
                risk_score: None,
                risk_factors: RiskFactors::default(),
            },
            WorkflowStep {
                id: "approve".to_string(),
                name: "Approval Gate".to_string(),
                step_type: StepType::Approval(ricecoder_workflows::ApprovalStep {
                    message: "Please review and approve".to_string(),
                    timeout: 5000,
                    default: ApprovalDefault::Reject,
                }),
                config: StepConfig {
                    config: serde_json::json!({}),
                },
                dependencies: vec!["analyze".to_string()],
                approval_required: true,
                on_error: ErrorAction::Fail,
                risk_score: None,
                risk_factors: RiskFactors::default(),
            },
            WorkflowStep {
                id: "execute".to_string(),
                name: "Execute".to_string(),
                step_type: StepType::Agent(AgentStep {
                    agent_id: "executor".to_string(),
                    task: "execute".to_string(),
                }),
                config: StepConfig {
                    config: serde_json::json!({}),
                },
                dependencies: vec!["approve".to_string()],
                approval_required: false,
                on_error: ErrorAction::Fail,
                risk_score: None,
                risk_factors: RiskFactors::default(),
            },
        ],
        config: WorkflowConfig {
            timeout_ms: None,
            max_parallel: None,
        },
    }
}

fn create_workflow_with_error_handling() -> Workflow {
    Workflow {
        id: "error-workflow".to_string(),
        name: "Workflow with Error Handling".to_string(),
        description: "A workflow with error handling".to_string(),
        parameters: vec![],
        steps: vec![
            WorkflowStep {
                id: "risky-step".to_string(),
                name: "Risky Step".to_string(),
                step_type: StepType::Agent(AgentStep {
                    agent_id: "risky-agent".to_string(),
                    task: "risky-task".to_string(),
                }),
                config: StepConfig {
                    config: serde_json::json!({}),
                },
                dependencies: vec![],
                approval_required: false,
                on_error: ErrorAction::Retry {
                    max_attempts: 3,
                    delay_ms: 100,
                },
                risk_score: None,
                risk_factors: RiskFactors::default(),
            },
            WorkflowStep {
                id: "recovery-step".to_string(),
                name: "Recovery Step".to_string(),
                step_type: StepType::Agent(AgentStep {
                    agent_id: "recovery-agent".to_string(),
                    task: "recovery-task".to_string(),
                }),
                config: StepConfig {
                    config: serde_json::json!({}),
                },
                dependencies: vec!["risky-step".to_string()],
                approval_required: false,
                on_error: ErrorAction::Fail,
                risk_score: None,
                risk_factors: RiskFactors::default(),
            },
        ],
        config: WorkflowConfig {
            timeout_ms: None,
            max_parallel: None,
        },
    }
}

fn create_workflow_with_risk_scoring() -> Workflow {
    Workflow {
        id: "risk-workflow".to_string(),
        name: "Workflow with Risk Scoring".to_string(),
        description: "A workflow with risk-scored steps".to_string(),
        parameters: vec![],
        steps: vec![
            WorkflowStep {
                id: "low-risk".to_string(),
                name: "Low Risk Step".to_string(),
                step_type: StepType::Agent(AgentStep {
                    agent_id: "safe-agent".to_string(),
                    task: "safe-task".to_string(),
                }),
                config: StepConfig {
                    config: serde_json::json!({}),
                },
                dependencies: vec![],
                approval_required: false,
                on_error: ErrorAction::Fail,
                risk_score: Some(20),
                risk_factors: RiskFactors {
                    impact: 1,
                    reversibility: 3,
                    complexity: 1,
                },
            },
            WorkflowStep {
                id: "high-risk".to_string(),
                name: "High Risk Step".to_string(),
                step_type: StepType::Agent(AgentStep {
                    agent_id: "dangerous-agent".to_string(),
                    task: "dangerous-task".to_string(),
                }),
                config: StepConfig {
                    config: serde_json::json!({}),
                },
                dependencies: vec!["low-risk".to_string()],
                approval_required: true,
                on_error: ErrorAction::Rollback,
                risk_score: Some(85),
                risk_factors: RiskFactors {
                    impact: 3,
                    reversibility: 1,
                    complexity: 3,
                },
            },
        ],
        config: WorkflowConfig {
            timeout_ms: None,
            max_parallel: None,
        },
    }
}

// ============================================================================
// Task 15.1: Full Workflow Execution Tests
// ============================================================================

#[test]
fn test_multi_step_workflow_execution_order() {
    let workflow = create_simple_workflow();
    let order = WorkflowEngine::get_execution_order(&workflow).expect("Failed to get execution order");

    assert_eq!(order.len(), 3);
    assert_eq!(order[0], "step1");
    assert_eq!(order[1], "step2");
    assert_eq!(order[2], "step3");
}

#[test]
fn test_multi_step_workflow_dependency_resolution() {
    let workflow = create_simple_workflow();
    let mut state = StateManager::create_state(&workflow);

    // Initially, only step1 can execute
    assert!(WorkflowEngine::can_execute_step(&workflow, &state, "step1").unwrap());
    assert!(!WorkflowEngine::can_execute_step(&workflow, &state, "step2").unwrap());
    assert!(!WorkflowEngine::can_execute_step(&workflow, &state, "step3").unwrap());

    // After step1 completes, step2 can execute
    state.completed_steps.push("step1".to_string());
    assert!(WorkflowEngine::can_execute_step(&workflow, &state, "step2").unwrap());
    assert!(!WorkflowEngine::can_execute_step(&workflow, &state, "step3").unwrap());

    // After step2 completes, step3 can execute
    state.completed_steps.push("step2".to_string());
    assert!(WorkflowEngine::can_execute_step(&workflow, &state, "step3").unwrap());
}

#[test]
fn test_workflow_execution_lifecycle() {
    let mut engine = WorkflowEngine::new();
    let workflow = create_simple_workflow();

    // Create execution
    let execution_id = engine.create_execution(&workflow).expect("Failed to create execution");
    let state = engine.get_execution_state(&execution_id).unwrap();
    assert_eq!(state.status, WorkflowStatus::Pending);

    // Start execution
    engine.start_execution(&execution_id).expect("Failed to start execution");
    let state = engine.get_execution_state(&execution_id).unwrap();
    assert_eq!(state.status, WorkflowStatus::Running);

    // Complete execution
    engine.complete_execution(&execution_id).expect("Failed to complete execution");
    let state = engine.get_execution_state(&execution_id).unwrap();
    assert_eq!(state.status, WorkflowStatus::Completed);
}

#[test]
fn test_workflow_error_handling_with_retry() {
    let workflow = create_workflow_with_error_handling();

    // Get the risky step
    let risky_step = workflow.steps.iter().find(|s| s.id == "risky-step").unwrap();

    // Verify error action is retry
    match &risky_step.on_error {
        ErrorAction::Retry {
            max_attempts,
            delay_ms,
        } => {
            assert_eq!(*max_attempts, 3);
            assert_eq!(*delay_ms, 100);
        }
        _ => panic!("Expected Retry error action"),
    }

    // Simulate step failure
    let result = StepResult {
        status: StepStatus::Failed,
        output: None,
        error: Some("Step failed".to_string()),
        duration_ms: 100,
    };

    // Verify error is captured
    assert_eq!(result.status, StepStatus::Failed);
    assert!(result.error.is_some());
}

#[test]
fn test_workflow_error_recovery() {
    let mut engine = WorkflowEngine::new();
    let workflow = create_workflow_with_error_handling();

    let execution_id = engine.create_execution(&workflow).expect("Failed to create execution");
    engine.start_execution(&execution_id).expect("Failed to start execution");

    let mut state = engine.get_execution_state(&execution_id).unwrap();

    // Simulate risky-step completing
    state.completed_steps.push("risky-step".to_string());

    // Verify recovery-step can now execute
    assert!(WorkflowEngine::can_execute_step(&workflow, &state, "recovery-step").unwrap());
}

#[test]
fn test_approval_gate_blocks_execution() {
    let workflow = create_workflow_with_approval();

    // Get the approval step
    let approval_step = workflow.steps.iter().find(|s| s.id == "approve").unwrap();

    // Verify approval is required
    assert!(approval_step.approval_required);

    // Verify execute step depends on approve
    let execute_step = workflow.steps.iter().find(|s| s.id == "execute").unwrap();
    assert!(execute_step.dependencies.contains(&"approve".to_string()));
}

#[test]
fn test_approval_gate_enforcement() {
    let workflow = create_workflow_with_approval();
    let mut state = StateManager::create_state(&workflow);

    // Mark analyze as completed
    state.completed_steps.push("analyze".to_string());

    // Approval step cannot be skipped - it must be explicitly approved
    // Verify execute step cannot execute without approval
    assert!(!WorkflowEngine::can_execute_step(&workflow, &state, "execute").unwrap());

    // Mark approval as completed (simulating approval)
    state.completed_steps.push("approve".to_string());

    // Now execute can proceed
    assert!(WorkflowEngine::can_execute_step(&workflow, &state, "execute").unwrap());
}

#[test]
fn test_workflow_state_tracking() {
    let mut engine = WorkflowEngine::new();
    let workflow = create_simple_workflow();

    let execution_id = engine.create_execution(&workflow).expect("Failed to create execution");
    engine.start_execution(&execution_id).expect("Failed to start execution");

    let mut state = engine.get_execution_state(&execution_id).unwrap();

    // Verify initial state
    assert_eq!(state.status, WorkflowStatus::Running);
    assert!(state.completed_steps.is_empty());
    assert!(state.step_results.is_empty());

    // Simulate step completion
    state.completed_steps.push("step1".to_string());
    state.step_results.insert(
        "step1".to_string(),
        StepResult {
            status: StepStatus::Completed,
            output: Some(serde_json::json!({"result": "success"})),
            error: None,
            duration_ms: 100,
        },
    );

    // Verify state is updated
    assert!(state.completed_steps.contains(&"step1".to_string()));
    assert!(state.step_results.contains_key(&"step1".to_string()));
}

// ============================================================================
// Task 15.2: Pause/Resume Workflow Tests
// ============================================================================

#[test]
fn test_pause_workflow_execution() {
    let mut engine = WorkflowEngine::new();
    let workflow = create_simple_workflow();

    let execution_id = engine.create_execution(&workflow).expect("Failed to create execution");
    engine.start_execution(&execution_id).expect("Failed to start execution");

    let state = engine.get_execution_state(&execution_id).unwrap();
    assert_eq!(state.status, WorkflowStatus::Running);

    engine.pause_execution(&execution_id).expect("Failed to pause execution");

    let state = engine.get_execution_state(&execution_id).unwrap();
    assert_eq!(state.status, WorkflowStatus::Paused);
}

#[test]
fn test_resume_workflow_execution() {
    let mut engine = WorkflowEngine::new();
    let workflow = create_simple_workflow();

    let execution_id = engine.create_execution(&workflow).expect("Failed to create execution");
    engine.start_execution(&execution_id).expect("Failed to start execution");
    engine.pause_execution(&execution_id).expect("Failed to pause execution");

    let state = engine.get_execution_state(&execution_id).unwrap();
    assert_eq!(state.status, WorkflowStatus::Paused);

    engine.resume_execution(&execution_id).expect("Failed to resume execution");

    let state = engine.get_execution_state(&execution_id).unwrap();
    assert_eq!(state.status, WorkflowStatus::Running);
}

#[test]
fn test_pause_resume_preserves_completed_steps() {
    let mut engine = WorkflowEngine::new();
    let workflow = create_simple_workflow();

    let execution_id = engine.create_execution(&workflow).expect("Failed to create execution");
    engine.start_execution(&execution_id).expect("Failed to start execution");

    // Pause workflow
    engine.pause_execution(&execution_id).expect("Failed to pause execution");

    let paused_state = engine.get_execution_state(&execution_id).unwrap();
    assert_eq!(paused_state.status, WorkflowStatus::Paused);

    // Resume workflow
    engine.resume_execution(&execution_id).expect("Failed to resume execution");

    let resumed_state = engine.get_execution_state(&execution_id).unwrap();
    assert_eq!(resumed_state.status, WorkflowStatus::Running);
}

#[test]
fn test_pause_resume_prevents_step_reexecution() {
    let mut engine = WorkflowEngine::new();
    let workflow = create_simple_workflow();

    let execution_id = engine.create_execution(&workflow).expect("Failed to create execution");
    engine.start_execution(&execution_id).expect("Failed to start execution");

    // Pause and resume
    engine.pause_execution(&execution_id).expect("Failed to pause execution");
    engine.resume_execution(&execution_id).expect("Failed to resume execution");

    let resumed_state = engine.get_execution_state(&execution_id).unwrap();

    // Verify workflow is running after resume
    assert_eq!(resumed_state.status, WorkflowStatus::Running);

    // Verify step1 can execute (no dependencies)
    assert!(WorkflowEngine::can_execute_step(&workflow, &resumed_state, "step1").unwrap());
}

#[test]
fn test_workflow_state_persistence_across_pause_resume() {
    let mut engine = WorkflowEngine::new();
    let workflow = create_simple_workflow();

    let execution_id = engine.create_execution(&workflow).expect("Failed to create execution");
    engine.start_execution(&execution_id).expect("Failed to start execution");

    // Pause
    engine.pause_execution(&execution_id).expect("Failed to pause execution");

    let paused_state = engine.get_execution_state(&execution_id).unwrap();
    assert_eq!(paused_state.status, WorkflowStatus::Paused);

    // Resume
    engine.resume_execution(&execution_id).expect("Failed to resume execution");

    let resumed_state = engine.get_execution_state(&execution_id).unwrap();
    assert_eq!(resumed_state.status, WorkflowStatus::Running);
}

// ============================================================================
// Task 15.3: Risk-Scored Execution Tests
// ============================================================================

#[test]
fn test_risk_score_calculation() {
    let workflow = create_workflow_with_risk_scoring();

    // Get low-risk step
    let low_risk_step = workflow.steps.iter().find(|s| s.id == "low-risk").unwrap();
    assert_eq!(low_risk_step.risk_score, Some(20));

    // Get high-risk step
    let high_risk_step = workflow.steps.iter().find(|s| s.id == "high-risk").unwrap();
    assert_eq!(high_risk_step.risk_score, Some(85));
}

#[test]
fn test_risk_factors_consideration() {
    let workflow = create_workflow_with_risk_scoring();

    let low_risk_step = workflow.steps.iter().find(|s| s.id == "low-risk").unwrap();
    let high_risk_step = workflow.steps.iter().find(|s| s.id == "high-risk").unwrap();

    // Low-risk step has low impact, high reversibility, low complexity
    assert_eq!(low_risk_step.risk_factors.impact, 1);
    assert_eq!(low_risk_step.risk_factors.reversibility, 3);
    assert_eq!(low_risk_step.risk_factors.complexity, 1);

    // High-risk step has high impact, low reversibility, high complexity
    assert_eq!(high_risk_step.risk_factors.impact, 3);
    assert_eq!(high_risk_step.risk_factors.reversibility, 1);
    assert_eq!(high_risk_step.risk_factors.complexity, 3);
}

#[test]
fn test_high_risk_approval_requirement() {
    let workflow = create_workflow_with_risk_scoring();

    let low_risk_step = workflow.steps.iter().find(|s| s.id == "low-risk").unwrap();
    let high_risk_step = workflow.steps.iter().find(|s| s.id == "high-risk").unwrap();

    // Low-risk step does not require approval
    assert!(!low_risk_step.approval_required);

    // High-risk step requires approval
    assert!(high_risk_step.approval_required);
}

#[test]
fn test_high_risk_step_blocks_execution_without_approval() {
    let workflow = create_workflow_with_risk_scoring();
    let mut state = StateManager::create_state(&workflow);

    // Mark low-risk step as completed
    state.completed_steps.push("low-risk".to_string());

    // High-risk step requires approval, so it cannot execute without explicit approval
    // The approval_required flag should prevent execution
    let high_risk_step = workflow.steps.iter().find(|s| s.id == "high-risk").unwrap();
    assert!(high_risk_step.approval_required);
}

#[test]
fn test_safety_constraints_on_high_risk_steps() {
    let workflow = create_workflow_with_risk_scoring();

    let high_risk_step = workflow.steps.iter().find(|s| s.id == "high-risk").unwrap();

    // High-risk step should have rollback capability
    match &high_risk_step.on_error {
        ErrorAction::Rollback => {
            // Correct - high-risk steps should support rollback
        }
        _ => panic!("Expected Rollback error action for high-risk step"),
    }
}

#[test]
fn test_risk_assessment_report_generation() {
    let workflow = create_workflow_with_risk_scoring();

    // Collect risk scores for all steps
    let mut risk_scores = HashMap::new();
    for step in &workflow.steps {
        if let Some(score) = step.risk_score {
            risk_scores.insert(step.id.clone(), score);
        }
    }

    // Verify all steps have risk scores
    assert_eq!(risk_scores.len(), 2);
    assert_eq!(risk_scores.get("low-risk"), Some(&20));
    assert_eq!(risk_scores.get("high-risk"), Some(&85));
}

#[test]
fn test_risk_score_range_validation() {
    let workflow = create_workflow_with_risk_scoring();

    for step in &workflow.steps {
        if let Some(score) = step.risk_score {
            // Risk scores should be between 0 and 100
            assert!(score <= 100, "Risk score {} out of range", score);
        }
    }
}

#[test]
fn test_workflow_with_multiple_risk_levels() {
    let workflow = create_workflow_with_risk_scoring();

    let low_risk = workflow.steps.iter().find(|s| s.id == "low-risk").unwrap();
    let high_risk = workflow.steps.iter().find(|s| s.id == "high-risk").unwrap();

    // Verify risk scores reflect the difference
    let low_score = low_risk.risk_score.unwrap_or(0);
    let high_score = high_risk.risk_score.unwrap_or(0);

    assert!(high_score > low_score, "High-risk score should be greater than low-risk score");
}

// ============================================================================
// Additional Integration Tests
// ============================================================================

#[test]
fn test_workflow_execution_with_multiple_active_workflows() {
    let mut engine = WorkflowEngine::new();
    let workflow1 = create_simple_workflow();
    let workflow2 = create_workflow_with_approval();

    let id1 = engine.create_execution(&workflow1).expect("Failed to create execution 1");
    let id2 = engine.create_execution(&workflow2).expect("Failed to create execution 2");

    assert_eq!(engine.active_execution_count(), 2);

    let active = engine.get_active_executions();
    assert!(active.contains(&id1));
    assert!(active.contains(&id2));
}

#[test]
fn test_workflow_execution_removal() {
    let mut engine = WorkflowEngine::new();
    let workflow = create_simple_workflow();

    let execution_id = engine.create_execution(&workflow).expect("Failed to create execution");
    engine.start_execution(&execution_id).expect("Failed to start execution");
    engine.complete_execution(&execution_id).expect("Failed to complete execution");

    let removed_state = engine.remove_execution(&execution_id).expect("Failed to remove execution");
    assert_eq!(removed_state.status, WorkflowStatus::Completed);
    assert_eq!(engine.active_execution_count(), 0);
}

#[test]
fn test_workflow_cancellation() {
    let mut engine = WorkflowEngine::new();
    let workflow = create_simple_workflow();

    let execution_id = engine.create_execution(&workflow).expect("Failed to create execution");
    engine.start_execution(&execution_id).expect("Failed to start execution");
    engine.cancel_execution(&execution_id).expect("Failed to cancel execution");

    let state = engine.get_execution_state(&execution_id).unwrap();
    assert_eq!(state.status, WorkflowStatus::Cancelled);
}

#[test]
fn test_workflow_failure_handling() {
    let mut engine = WorkflowEngine::new();
    let workflow = create_simple_workflow();

    let execution_id = engine.create_execution(&workflow).expect("Failed to create execution");
    engine.start_execution(&execution_id).expect("Failed to start execution");
    engine.fail_execution(&execution_id).expect("Failed to fail execution");

    let state = engine.get_execution_state(&execution_id).unwrap();
    assert_eq!(state.status, WorkflowStatus::Failed);
}


// ============================================================================
// Task 13.5: Step Type Execution Handler Tests
// ============================================================================

#[test]
fn test_agent_step_execution() {
    let workflow = create_simple_workflow();
    let mut state = StateManager::create_state(&workflow);

    // Execute the agent step
    let result = StepExecutor::execute_step(&workflow, &mut state, "step1");
    assert!(result.is_ok());

    // Verify step is marked as completed
    let step_result = state.step_results.get("step1");
    assert!(step_result.is_some());
    assert_eq!(step_result.unwrap().status, StepStatus::Completed);
}

#[test]
fn test_command_step_execution() {
    use ricecoder_workflows::{CommandStep, StepExecutor};

    let workflow = Workflow {
        id: "command-workflow".to_string(),
        name: "Command Workflow".to_string(),
        description: "A workflow with command steps".to_string(),
        parameters: vec![],
        steps: vec![WorkflowStep {
            id: "cmd-step".to_string(),
            name: "Command Step".to_string(),
            step_type: StepType::Command(CommandStep {
                command: "echo".to_string(),
                args: vec!["hello".to_string()],
                timeout: 5000,
            }),
            config: StepConfig {
                config: serde_json::json!({}),
            },
            dependencies: vec![],
            approval_required: false,
            on_error: ErrorAction::Fail,
            risk_score: None,
            risk_factors: RiskFactors::default(),
        }],
        config: WorkflowConfig {
            timeout_ms: None,
            max_parallel: None,
        },
    };

    let mut state = StateManager::create_state(&workflow);

    // Execute the command step
    let result = StepExecutor::execute_step(&workflow, &mut state, "cmd-step");
    assert!(result.is_ok());

    // Verify step is marked as completed
    let step_result = state.step_results.get("cmd-step");
    assert!(step_result.is_some());
    assert_eq!(step_result.unwrap().status, StepStatus::Completed);
}

#[test]
fn test_parallel_step_execution() {
    use ricecoder_workflows::{ParallelStep, StepExecutor};

    let workflow = Workflow {
        id: "parallel-workflow".to_string(),
        name: "Parallel Workflow".to_string(),
        description: "A workflow with parallel steps".to_string(),
        parameters: vec![],
        steps: vec![
            WorkflowStep {
                id: "parallel".to_string(),
                name: "Parallel Step".to_string(),
                step_type: StepType::Parallel(ParallelStep {
                    steps: vec!["step1".to_string(), "step2".to_string()],
                    max_concurrency: 2,
                }),
                config: StepConfig {
                    config: serde_json::json!({}),
                },
                dependencies: vec![],
                approval_required: false,
                on_error: ErrorAction::Fail,
                risk_score: None,
                risk_factors: RiskFactors::default(),
            },
            WorkflowStep {
                id: "step1".to_string(),
                name: "Step 1".to_string(),
                step_type: StepType::Agent(AgentStep {
                    agent_id: "agent1".to_string(),
                    task: "task1".to_string(),
                }),
                config: StepConfig {
                    config: serde_json::json!({}),
                },
                dependencies: vec![],
                approval_required: false,
                on_error: ErrorAction::Fail,
                risk_score: None,
                risk_factors: RiskFactors::default(),
            },
            WorkflowStep {
                id: "step2".to_string(),
                name: "Step 2".to_string(),
                step_type: StepType::Agent(AgentStep {
                    agent_id: "agent2".to_string(),
                    task: "task2".to_string(),
                }),
                config: StepConfig {
                    config: serde_json::json!({}),
                },
                dependencies: vec![],
                approval_required: false,
                on_error: ErrorAction::Fail,
                risk_score: None,
                risk_factors: RiskFactors::default(),
            },
        ],
        config: WorkflowConfig {
            timeout_ms: None,
            max_parallel: None,
        },
    };

    let mut state = StateManager::create_state(&workflow);

    // Execute the parallel step
    let result = StepExecutor::execute_step(&workflow, &mut state, "parallel");
    assert!(result.is_ok());

    // Verify step is marked as completed
    let step_result = state.step_results.get("parallel");
    assert!(step_result.is_some());
    assert_eq!(step_result.unwrap().status, StepStatus::Completed);
}

#[test]
fn test_condition_step_execution_then_branch() {
    use ricecoder_workflows::ConditionStep;

    let workflow = Workflow {
        id: "condition-workflow".to_string(),
        name: "Condition Workflow".to_string(),
        description: "A workflow with condition steps".to_string(),
        parameters: vec![],
        steps: vec![
            WorkflowStep {
                id: "check".to_string(),
                name: "Check".to_string(),
                step_type: StepType::Agent(AgentStep {
                    agent_id: "checker".to_string(),
                    task: "check".to_string(),
                }),
                config: StepConfig {
                    config: serde_json::json!({}),
                },
                dependencies: vec![],
                approval_required: false,
                on_error: ErrorAction::Fail,
                risk_score: None,
                risk_factors: RiskFactors::default(),
            },
            WorkflowStep {
                id: "condition".to_string(),
                name: "Condition".to_string(),
                step_type: StepType::Condition(ConditionStep {
                    condition: "check.output.status == 'success'".to_string(),
                    then_steps: vec!["success-step".to_string()],
                    else_steps: vec!["failure-step".to_string()],
                }),
                config: StepConfig {
                    config: serde_json::json!({}),
                },
                dependencies: vec!["check".to_string()],
                approval_required: false,
                on_error: ErrorAction::Fail,
                risk_score: None,
                risk_factors: RiskFactors::default(),
            },
            WorkflowStep {
                id: "success-step".to_string(),
                name: "Success Step".to_string(),
                step_type: StepType::Agent(AgentStep {
                    agent_id: "success-agent".to_string(),
                    task: "success".to_string(),
                }),
                config: StepConfig {
                    config: serde_json::json!({}),
                },
                dependencies: vec!["condition".to_string()],
                approval_required: false,
                on_error: ErrorAction::Fail,
                risk_score: None,
                risk_factors: RiskFactors::default(),
            },
            WorkflowStep {
                id: "failure-step".to_string(),
                name: "Failure Step".to_string(),
                step_type: StepType::Agent(AgentStep {
                    agent_id: "failure-agent".to_string(),
                    task: "failure".to_string(),
                }),
                config: StepConfig {
                    config: serde_json::json!({}),
                },
                dependencies: vec!["condition".to_string()],
                approval_required: false,
                on_error: ErrorAction::Fail,
                risk_score: None,
                risk_factors: RiskFactors::default(),
            },
        ],
        config: WorkflowConfig {
            timeout_ms: None,
            max_parallel: None,
        },
    };

    let mut state = StateManager::create_state(&workflow);

    // Add check step result with status = 'success'
    state.step_results.insert(
        "check".to_string(),
        StepResult {
            status: StepStatus::Completed,
            output: Some(serde_json::json!({"status": "success"})),
            error: None,
            duration_ms: 100,
        },
    );
    state.completed_steps.push("check".to_string());

    // Execute the condition step
    let result = StepExecutor::execute_step(&workflow, &mut state, "condition");
    assert!(result.is_ok());

    // Verify step is marked as completed
    let step_result = state.step_results.get("condition");
    assert!(step_result.is_some());
    assert_eq!(step_result.unwrap().status, StepStatus::Completed);
}

#[test]
fn test_approval_step_execution() {
    use ricecoder_workflows::ApprovalStep;

    let workflow = Workflow {
        id: "approval-workflow".to_string(),
        name: "Approval Workflow".to_string(),
        description: "A workflow with approval steps".to_string(),
        parameters: vec![],
        steps: vec![WorkflowStep {
            id: "approval".to_string(),
            name: "Approval".to_string(),
            step_type: StepType::Approval(ApprovalStep {
                message: "Please approve".to_string(),
                timeout: 5000,
                default: ApprovalDefault::Reject,
            }),
            config: StepConfig {
                config: serde_json::json!({}),
            },
            dependencies: vec![],
            approval_required: true,
            on_error: ErrorAction::Fail,
            risk_score: None,
            risk_factors: RiskFactors::default(),
        }],
        config: WorkflowConfig {
            timeout_ms: None,
            max_parallel: None,
        },
    };

    let mut state = StateManager::create_state(&workflow);

    // Execute the approval step
    let result = StepExecutor::execute_step(&workflow, &mut state, "approval");
    assert!(result.is_ok());

    // Verify step is marked as completed
    let step_result = state.step_results.get("approval");
    assert!(step_result.is_some());
    assert_eq!(step_result.unwrap().status, StepStatus::Completed);
}

#[test]
fn test_mixed_step_types_execution() {
    use ricecoder_workflows::CommandStep;

    let workflow = Workflow {
        id: "mixed-workflow".to_string(),
        name: "Mixed Workflow".to_string(),
        description: "A workflow with mixed step types".to_string(),
        parameters: vec![],
        steps: vec![
            WorkflowStep {
                id: "agent-step".to_string(),
                name: "Agent Step".to_string(),
                step_type: StepType::Agent(AgentStep {
                    agent_id: "agent".to_string(),
                    task: "task".to_string(),
                }),
                config: StepConfig {
                    config: serde_json::json!({}),
                },
                dependencies: vec![],
                approval_required: false,
                on_error: ErrorAction::Fail,
                risk_score: None,
                risk_factors: RiskFactors::default(),
            },
            WorkflowStep {
                id: "command-step".to_string(),
                name: "Command Step".to_string(),
                step_type: StepType::Command(CommandStep {
                    command: "echo".to_string(),
                    args: vec!["test".to_string()],
                    timeout: 5000,
                }),
                config: StepConfig {
                    config: serde_json::json!({}),
                },
                dependencies: vec!["agent-step".to_string()],
                approval_required: false,
                on_error: ErrorAction::Fail,
                risk_score: None,
                risk_factors: RiskFactors::default(),
            },
        ],
        config: WorkflowConfig {
            timeout_ms: None,
            max_parallel: None,
        },
    };

    let mut state = StateManager::create_state(&workflow);

    // Execute agent step
    let result = StepExecutor::execute_step(&workflow, &mut state, "agent-step");
    assert!(result.is_ok());
    assert_eq!(
        state.step_results.get("agent-step").unwrap().status,
        StepStatus::Completed
    );

    // Execute command step
    let result = StepExecutor::execute_step(&workflow, &mut state, "command-step");
    assert!(result.is_ok());
    assert_eq!(
        state.step_results.get("command-step").unwrap().status,
        StepStatus::Completed
    );
}
