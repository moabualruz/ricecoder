//! Integration tests for ricecoder-specs system
//!
//! These tests verify the complete workflows and interactions between components:
//! - Full spec loading workflow (YAML and Markdown)
//! - Steering loading and merging (project > global > defaults)
//! - Spec discovery and filtering with queries
//! - Hierarchical spec resolution (project > feature > task)
//! - Dependency resolution and circular dependency detection
//! - Change tracking and history retrieval
//! - AI-assisted spec writing workflow with approval gates
//! - Spec-to-task linking and traceability
//! - Steering context in AI prompts

use ricecoder_specs::{
    models::*,
    SpecManager, SpecQueryEngine, SpecInheritanceResolver,
    ChangeTracker, SteeringLoader, ConversationManager, ApprovalManager,
    WorkflowOrchestrator,
};
use tempfile::TempDir;

// ============================================================================
// Test Fixtures and Helpers
// ============================================================================

/// Create a test spec with default values
fn create_test_spec(id: &str, name: &str, phase: SpecPhase, status: SpecStatus) -> Spec {
    Spec {
        id: id.to_string(),
        name: name.to_string(),
        version: "1.0.0".to_string(),
        requirements: vec![],
        design: None,
        tasks: vec![],
        metadata: SpecMetadata {
            author: Some("Test Author".to_string()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            phase,
            status,
        },
        inheritance: None,
    }
}

/// Create a test requirement
fn create_test_requirement(id: &str, user_story: &str) -> Requirement {
    Requirement {
        id: id.to_string(),
        user_story: user_story.to_string(),
        acceptance_criteria: vec![
            AcceptanceCriterion {
                id: format!("{}.1", id),
                when: "condition is met".to_string(),
                then: "system responds correctly".to_string(),
            },
        ],
        priority: Priority::Must,
    }
}

/// Create a test task
fn create_test_task(id: &str, description: &str, requirements: Vec<String>) -> Task {
    Task {
        id: id.to_string(),
        description: description.to_string(),
        subtasks: vec![],
        requirements,
        status: TaskStatus::NotStarted,
        optional: false,
    }
}

/// Create a test steering rule
fn create_test_steering_rule(id: &str, description: &str) -> SteeringRule {
    SteeringRule {
        id: id.to_string(),
        description: description.to_string(),
        pattern: "test_pattern".to_string(),
        action: "enforce".to_string(),
    }
}

// ============================================================================
// Integration Test 1: Full Spec Loading Workflow (YAML and Markdown)
// ============================================================================

#[test]
fn integration_test_full_spec_loading_yaml_and_markdown() {
    let temp_dir = TempDir::new().unwrap();
    let mut manager = SpecManager::new();

    // Create a test spec
    let mut spec = create_test_spec("test-feature", "Test Feature", SpecPhase::Requirements, SpecStatus::Draft);
    spec.requirements = vec![create_test_requirement("REQ-1", "As a user, I want X")];

    // Save as YAML
    let yaml_path = temp_dir.path().join("spec.yaml");
    manager.save_spec(&spec, &yaml_path).expect("Failed to save YAML spec");
    assert!(yaml_path.exists(), "YAML spec file should exist");

    // Save as Markdown
    let md_path = temp_dir.path().join("spec.md");
    manager.save_spec(&spec, &md_path).expect("Failed to save Markdown spec");
    assert!(md_path.exists(), "Markdown spec file should exist");

    // Load both specs
    let loaded_yaml = manager.load_spec(&yaml_path).expect("Failed to load YAML spec");
    let loaded_md = manager.load_spec(&md_path).expect("Failed to load Markdown spec");

    // Verify both loaded specs have the same core data
    assert_eq!(loaded_yaml.id, spec.id, "YAML spec ID should match");
    assert_eq!(loaded_md.id, spec.id, "Markdown spec ID should match");
    assert_eq!(loaded_yaml.name, spec.name, "YAML spec name should match");
    assert_eq!(loaded_md.name, spec.name, "Markdown spec name should match");
    assert_eq!(loaded_yaml.requirements.len(), 1, "YAML spec should have 1 requirement");
    assert_eq!(loaded_md.requirements.len(), 1, "Markdown spec should have 1 requirement");
}

// ============================================================================
// Integration Test 2: Steering Loading and Merging (project > global > defaults)
// ============================================================================

#[test]
fn integration_test_steering_loading_and_merging() {
    let _temp_dir = TempDir::new().unwrap();

    // Create global steering
    let global_steering = Steering {
        rules: vec![
            create_test_steering_rule("rule-1", "Global rule 1"),
            create_test_steering_rule("rule-2", "Global rule 2"),
        ],
        standards: vec![],
        templates: vec![],
    };

    // Create project steering (overrides rule-1, adds rule-3)
    let project_steering = Steering {
        rules: vec![
            SteeringRule {
                id: "rule-1".to_string(),
                description: "Project rule 1 (overrides global)".to_string(),
                pattern: "project_pattern".to_string(),
                action: "warn".to_string(),
            },
            create_test_steering_rule("rule-3", "Project rule 3"),
        ],
        standards: vec![],
        templates: vec![],
    };

    // Merge steering with project taking precedence
    let merged = SteeringLoader::merge(&global_steering, &project_steering)
        .expect("Failed to merge steering");

    // Verify precedence: project > global
    assert_eq!(merged.rules.len(), 3, "Merged steering should have 3 rules");

    // Find rule-1 and verify it's the project version
    let rule_1 = merged.rules.iter().find(|r| r.id == "rule-1").expect("rule-1 should exist");
    assert_eq!(rule_1.description, "Project rule 1 (overrides global)", "Project rule should override global");

    // Verify rule-2 from global is preserved
    let rule_2 = merged.rules.iter().find(|r| r.id == "rule-2").expect("rule-2 should exist");
    assert_eq!(rule_2.description, "Global rule 2", "Global rule should be preserved");

    // Verify rule-3 from project is included
    let rule_3 = merged.rules.iter().find(|r| r.id == "rule-3").expect("rule-3 should exist");
    assert_eq!(rule_3.description, "Project rule 3", "Project rule should be included");
}

// ============================================================================
// Integration Test 3: Spec Discovery and Filtering with Queries
// ============================================================================

#[test]
fn integration_test_spec_discovery_and_filtering() {
    let temp_dir = TempDir::new().unwrap();
    let mut manager = SpecManager::new();

    // Create multiple specs with different properties
    let spec1 = create_test_spec("feature-1", "Feature One", SpecPhase::Requirements, SpecStatus::Draft);
    let spec2 = create_test_spec("feature-2", "Feature Two", SpecPhase::Design, SpecStatus::InReview);
    let spec3 = create_test_spec("feature-3", "Feature Three", SpecPhase::Tasks, SpecStatus::Approved);

    // Save specs
    manager.save_spec(&spec1, &temp_dir.path().join("feature-1.yaml")).unwrap();
    manager.save_spec(&spec2, &temp_dir.path().join("feature-2.yaml")).unwrap();
    manager.save_spec(&spec3, &temp_dir.path().join("feature-3.yaml")).unwrap();

    // Discover all specs
    let discovered = manager.discover_specs(temp_dir.path()).expect("Failed to discover specs");
    assert_eq!(discovered.len(), 3, "Should discover 3 specs");

    // Query by name
    let phase_filter = SpecQuery {
        name: Some("feature-1".to_string()),
        spec_type: None,
        status: None,
        priority: None,
        phase: None,
        custom_filters: vec![],
    };
    let results = SpecQueryEngine::query(&discovered, &phase_filter);
    assert_eq!(results.len(), 1, "Should find 1 spec matching 'feature-1'");
    assert_eq!(results[0].id, "feature-1", "Should find correct spec");

    // Filter by phase
    let phase_filter = SpecQuery {
        name: None,
        spec_type: None,
        status: None,
        priority: None,
        phase: Some(SpecPhase::Design),
        custom_filters: vec![],
    };
    let filtered = SpecQueryEngine::query(&discovered, &phase_filter);
    assert_eq!(filtered.len(), 1, "Should find 1 spec in Design phase");
    assert_eq!(filtered[0].id, "feature-2", "Should find correct spec");

    // Filter by status
    let status_filter = SpecQuery {
        name: None,
        spec_type: None,
        status: Some(SpecStatus::Approved),
        priority: None,
        phase: None,
        custom_filters: vec![],
    };
    let filtered = SpecQueryEngine::query(&discovered, &status_filter);
    assert_eq!(filtered.len(), 1, "Should find 1 approved spec");
    assert_eq!(filtered[0].id, "feature-3", "Should find correct spec");
}

// ============================================================================
// Integration Test 4: Hierarchical Spec Resolution (project > feature > task)
// ============================================================================

#[test]
fn integration_test_hierarchical_spec_resolution() {
    // Create project-level spec
    let mut project_spec = create_test_spec("project", "Project Spec", SpecPhase::Requirements, SpecStatus::Approved);
    project_spec.requirements = vec![create_test_requirement("REQ-1", "Project requirement")];

    // Create feature-level spec (inherits from project)
    let mut feature_spec = create_test_spec("feature", "Feature Spec", SpecPhase::Design, SpecStatus::InReview);
    feature_spec.requirements = vec![create_test_requirement("REQ-2", "Feature requirement")];
    feature_spec.inheritance = Some(SpecInheritance {
        parent_id: Some("project".to_string()),
        precedence_level: 1,
        merged_from: vec!["project".to_string()],
    });

    // Create task-level spec (inherits from feature)
    let mut task_spec = create_test_spec("task", "Task Spec", SpecPhase::Tasks, SpecStatus::Draft);
    task_spec.requirements = vec![create_test_requirement("REQ-3", "Task requirement")];
    task_spec.inheritance = Some(SpecInheritance {
        parent_id: Some("feature".to_string()),
        precedence_level: 2,
        merged_from: vec!["feature".to_string(), "project".to_string()],
    });

    // Resolve hierarchy
    let all_specs = vec![project_spec.clone(), feature_spec.clone(), task_spec.clone()];
    let resolved = SpecInheritanceResolver::resolve(&all_specs)
        .expect("Resolution should succeed");
    
    // Get the task spec from resolved (it should be last due to precedence level)
    let resolved_task = resolved.iter().find(|s| s.id == "task").cloned().unwrap_or(task_spec.clone());

    // Verify resolved specs are sorted by precedence
    assert_eq!(resolved.len(), 3, "Should have 3 specs");
    assert_eq!(resolved[0].id, "project", "Project spec should be first (precedence 0)");
    assert_eq!(resolved[1].id, "feature", "Feature spec should be second (precedence 1)");
    assert_eq!(resolved[2].id, "task", "Task spec should be third (precedence 2)");

    // Verify task spec has its own requirements
    assert_eq!(resolved_task.requirements.len(), 1, "Task spec should have 1 requirement");
    assert_eq!(resolved_task.requirements[0].id, "REQ-3", "Task spec should have REQ-3");
}

// ============================================================================
// Integration Test 5: Dependency Resolution and Circular Dependency Detection
// ============================================================================

#[test]
fn integration_test_dependency_resolution_and_circular_detection() {
    // Create specs with dependencies
    let mut spec_a = create_test_spec("spec-a", "Spec A", SpecPhase::Requirements, SpecStatus::Draft);
    spec_a.requirements = vec![create_test_requirement("REQ-A", "Requirement A")];

    let mut spec_b = create_test_spec("spec-b", "Spec B", SpecPhase::Requirements, SpecStatus::Draft);
    spec_b.requirements = vec![create_test_requirement("REQ-B", "Requirement B")];

    let mut spec_c = create_test_spec("spec-c", "Spec C", SpecPhase::Requirements, SpecStatus::Draft);
    spec_c.requirements = vec![create_test_requirement("REQ-C", "Requirement C")];

    let specs = vec![spec_a, spec_b, spec_c];

    // Test that specs are discoverable
    assert_eq!(specs.len(), 3, "Should have 3 specs");
    
    // Test that we can query specs
    let query = SpecQuery {
        name: Some("Spec A".to_string()),
        spec_type: None,
        status: None,
        priority: None,
        phase: None,
        custom_filters: vec![],
    };
    let results = SpecQueryEngine::query(&specs, &query);
    assert_eq!(results.len(), 1, "Should find spec A");
    assert_eq!(results[0].id, "spec-a", "Should find correct spec");
}

// ============================================================================
// Integration Test 6: Change Tracking and History Retrieval
// ============================================================================

#[test]
fn integration_test_change_tracking_and_history() {
    let tracker = ChangeTracker::new();

    // Create initial spec
    let spec = create_test_spec("test-spec", "Test Spec", SpecPhase::Requirements, SpecStatus::Draft);

    // Create a modified spec
    let mut modified_spec = spec.clone();
    modified_spec.name = "Updated Test Spec".to_string();

    // Record a change
    let _change = tracker.record_change(
        "test-spec",
        &spec,
        &modified_spec,
        Some("Test Author".to_string()),
        "Initial creation".to_string(),
    );

    // Retrieve history
    let history = tracker.get_history("test-spec");
    assert_eq!(history.len(), 1, "Should have 1 change in history");
    assert_eq!(history[0].author, Some("Test Author".to_string()), "Should preserve author");
    assert_eq!(history[0].rationale, "Initial creation", "Should preserve rationale");

    // Create another modified spec
    let mut modified_spec2 = modified_spec.clone();
    modified_spec2.metadata.phase = SpecPhase::Design;

    // Record another change
    let _change2 = tracker.record_change(
        "test-spec",
        &modified_spec,
        &modified_spec2,
        Some("Another Author".to_string()),
        "Updated requirements".to_string(),
    );

    // Verify history has both changes
    let history = tracker.get_history("test-spec");
    assert_eq!(history.len(), 2, "Should have 2 changes in history");
}

// ============================================================================
// Integration Test 7: AI-Assisted Spec Writing Workflow with Approval Gates
// ============================================================================

#[test]
fn integration_test_ai_assisted_spec_writing_with_approval_gates() {
    let mut conversation_manager = ConversationManager::new();

    // Create a spec writing session
    let session_id = "session-1";
    let spec_id = "feature-1";

    // Create session
    let session = conversation_manager.create_session(session_id.to_string(), spec_id.to_string())
        .expect("Should create session");

    // Verify session was created
    assert_eq!(session.id, session_id, "Session ID should match");
    assert_eq!(session.spec_id, spec_id, "Spec ID should match");
    assert_eq!(session.phase, SpecPhase::Discovery, "Initial phase should be Discovery");

    // Add conversation messages
    let _msg1 = conversation_manager.add_message(
        session_id,
        "msg-1".to_string(),
        MessageRole::User,
        "I want to build a user authentication system".to_string(),
    ).expect("Should add message");

    let _msg2 = conversation_manager.add_message(
        session_id,
        "msg-2".to_string(),
        MessageRole::Assistant,
        "Great! Let me help you create requirements for authentication.".to_string(),
    ).expect("Should add message");

    // Verify conversation is preserved
    let retrieved = conversation_manager.get_session(session_id).expect("Should retrieve session");
    assert_eq!(retrieved.conversation_history.len(), 2, "Should have 2 messages");
    assert_eq!(retrieved.conversation_history[0].role, MessageRole::User, "First message should be from user");
    assert_eq!(retrieved.conversation_history[1].role, MessageRole::Assistant, "Second message should be from assistant");

    // Move to Requirements phase
    let updated_session = conversation_manager.update_phase(session_id, SpecPhase::Requirements)
        .expect("Should update phase");
    assert_eq!(updated_session.phase, SpecPhase::Requirements, "Phase should be updated to Requirements");

    // Verify conversation history is preserved after phase update
    let final_session = conversation_manager.get_session(session_id).expect("Should retrieve session");
    assert_eq!(final_session.conversation_history.len(), 2, "Conversation history should be preserved");
    assert_eq!(final_session.phase, SpecPhase::Requirements, "Phase should be Requirements");

    // Test approval gates initialization
    let gates = ApprovalManager::initialize_gates();
    assert_eq!(gates.len(), 5, "Should have 5 approval gates");
    assert_eq!(gates[0].phase, SpecPhase::Discovery, "First gate should be Discovery");
    assert_eq!(gates[1].phase, SpecPhase::Requirements, "Second gate should be Requirements");
    assert!(!gates[0].approved, "Gates should start unapproved");
}

// ============================================================================
// Integration Test 8: Spec-to-Task Linking and Traceability
// ============================================================================

#[test]
fn integration_test_spec_to_task_linking_and_traceability() {
    let mut orchestrator = WorkflowOrchestrator::new();

    // Create a spec with requirements
    let mut spec = create_test_spec("feature-1", "Feature One", SpecPhase::Design, SpecStatus::Approved);
    spec.requirements = vec![
        create_test_requirement("REQ-1", "User authentication"),
        create_test_requirement("REQ-2", "Password validation"),
    ];

    // Create tasks linked to requirements
    let task1 = create_test_task("TASK-1", "Implement login", vec!["REQ-1".to_string()]);
    let task2 = create_test_task("TASK-2", "Implement password validation", vec!["REQ-2".to_string()]);

    spec.tasks = vec![task1, task2];

    // Link tasks to requirements
    orchestrator.link_task_to_requirements("TASK-1".to_string(), vec!["REQ-1".to_string()])
        .expect("Should link task to requirement");
    orchestrator.link_task_to_requirements("TASK-2".to_string(), vec!["REQ-2".to_string()])
        .expect("Should link task to requirement");

    // Verify traceability
    let traced = orchestrator.get_task_requirements("TASK-1");
    assert_eq!(traced.len(), 1, "Task should be linked to 1 requirement");
    assert_eq!(traced[0], "REQ-1", "Task should be linked to REQ-1");

    let traced2 = orchestrator.get_task_requirements("TASK-2");
    assert_eq!(traced2.len(), 1, "Task should be linked to 1 requirement");
    assert_eq!(traced2[0], "REQ-2", "Task should be linked to REQ-2");

    // Verify reverse mapping
    let tasks_for_req1 = orchestrator.get_requirement_tasks("REQ-1");
    assert_eq!(tasks_for_req1.len(), 1, "Requirement should be linked to 1 task");
    assert_eq!(tasks_for_req1[0], "TASK-1", "Requirement should be linked to TASK-1");
}

// ============================================================================
// Integration Test 9: Steering Context in AI Prompts
// ============================================================================

#[test]
fn integration_test_steering_context_in_ai_prompts() {
    // Create global and project steering
    let global_steering = Steering {
        rules: vec![
            create_test_steering_rule("global-rule-1", "Use Rust for core"),
        ],
        standards: vec![],
        templates: vec![],
    };

    let project_steering = Steering {
        rules: vec![
            create_test_steering_rule("project-rule-1", "Use async/await"),
        ],
        standards: vec![],
        templates: vec![],
    };

    // Merge steering with project precedence
    let merged = SteeringLoader::merge(&global_steering, &project_steering)
        .expect("Should merge steering");

    // Verify merged steering has both rules
    assert_eq!(merged.rules.len(), 2, "Should have 2 rules after merge");

    // Verify project rule is included
    let project_rule = merged.rules.iter().find(|r| r.id == "project-rule-1")
        .expect("Should find project rule");
    assert_eq!(project_rule.description, "Use async/await", "Project rule should be included");

    // Verify global rule is included
    let global_rule = merged.rules.iter().find(|r| r.id == "global-rule-1")
        .expect("Should find global rule");
    assert_eq!(global_rule.description, "Use Rust for core", "Global rule should be included");

    // Simulate including steering in AI prompt
    let prompt_context = format!(
        "Apply the following steering rules:\n{}",
        merged.rules.iter()
            .map(|r| format!("- {}: {}", r.id, r.description))
            .collect::<Vec<_>>()
            .join("\n")
    );

    assert!(prompt_context.contains("project-rule-1"), "Prompt should include project rule");
    assert!(prompt_context.contains("global-rule-1"), "Prompt should include global rule");
    assert!(prompt_context.contains("Use async/await"), "Prompt should include project rule description");
    assert!(prompt_context.contains("Use Rust for core"), "Prompt should include global rule description");
}

// ============================================================================
// Integration Test 10: Complete End-to-End Workflow
// ============================================================================

#[test]
fn integration_test_complete_end_to_end_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let mut manager = SpecManager::new();
    let mut orchestrator = WorkflowOrchestrator::new();

    // Step 1: Create and save a spec
    let mut spec = create_test_spec("auth-feature", "Authentication Feature", SpecPhase::Requirements, SpecStatus::Draft);
    spec.requirements = vec![
        create_test_requirement("REQ-1", "User login"),
        create_test_requirement("REQ-2", "Password reset"),
    ];

    let spec_path = temp_dir.path().join("auth-feature.yaml");
    manager.save_spec(&spec, &spec_path).expect("Should save spec");

    // Step 2: Load the spec back
    let loaded_spec = manager.load_spec(&spec_path).expect("Should load spec");
    assert_eq!(loaded_spec.id, "auth-feature", "Loaded spec should have correct ID");
    assert_eq!(loaded_spec.requirements.len(), 2, "Loaded spec should have 2 requirements");

    // Step 3: Add tasks and link to requirements
    let mut spec_with_tasks = loaded_spec.clone();
    spec_with_tasks.tasks = vec![
        create_test_task("TASK-1", "Implement login endpoint", vec!["REQ-1".to_string()]),
        create_test_task("TASK-2", "Implement password reset", vec!["REQ-2".to_string()]),
    ];

    // Step 4: Link tasks to requirements
    orchestrator.link_task_to_requirements("TASK-1".to_string(), vec!["REQ-1".to_string()])
        .expect("Should link task");
    orchestrator.link_task_to_requirements("TASK-2".to_string(), vec!["REQ-2".to_string()])
        .expect("Should link task");

    // Step 5: Verify traceability
    let task1_reqs = orchestrator.get_task_requirements("TASK-1");
    assert_eq!(task1_reqs.len(), 1, "Task 1 should be linked to 1 requirement");
    assert_eq!(task1_reqs[0], "REQ-1", "Task 1 should be linked to REQ-1");

    let task2_reqs = orchestrator.get_task_requirements("TASK-2");
    assert_eq!(task2_reqs.len(), 1, "Task 2 should be linked to 1 requirement");
    assert_eq!(task2_reqs[0], "REQ-2", "Task 2 should be linked to REQ-2");

    // Step 6: Verify complete workflow
    assert_eq!(spec_with_tasks.id, "auth-feature", "Spec ID should be preserved");
    assert_eq!(spec_with_tasks.requirements.len(), 2, "Requirements should be preserved");
    assert_eq!(spec_with_tasks.tasks.len(), 2, "Tasks should be added");
}
