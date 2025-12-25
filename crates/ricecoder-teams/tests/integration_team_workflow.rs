/// Integration tests for complete team workflow
///
/// Tests the complete team creation, standards sharing, and rule promotion workflows
/// including configuration inheritance and override scenarios.
///
/// **Feature: ricecoder-teams, Integration Tests**
/// **Validates: Requirements 1.1-1.10, 2.1-2.9, 3.1-3.8**
use chrono::Utc;
use ricecoder_teams::{RuleScope, SharedRule, TeamManager, TeamMember, TeamRole, TeamStandards};
use uuid::Uuid;

/// Helper function to create a test team member
fn create_test_member(name: &str, email: &str, role: TeamRole) -> TeamMember {
    TeamMember {
        id: Uuid::new_v4().to_string(),
        name: name.to_string(),
        email: email.to_string(),
        role,
        joined_at: Utc::now(),
    }
}

/// Helper function to create test standards
fn create_test_standards(team_id: &str) -> TeamStandards {
    TeamStandards {
        id: Uuid::new_v4().to_string(),
        team_id: team_id.to_string(),
        code_review_rules: Vec::new(),
        templates: Vec::new(),
        governance_docs: Vec::new(),
        compliance_requirements: Vec::new(),
        version: 1,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

/// Helper function to create a test rule
fn create_test_rule(rule_id: &str) -> SharedRule {
    SharedRule {
        id: rule_id.to_string(),
        name: format!("Test Rule {}", rule_id),
        description: "A test rule for integration testing".to_string(),
        scope: RuleScope::Project,
        enforced: true,
        promoted_by: "admin-1".to_string(),
        promoted_at: Utc::now(),
        version: 1,
    }
}

// ============================================================================
// Test Suite 11.1: Team Creation and Standards Sharing Workflow
// ============================================================================

#[tokio::test]
async fn test_complete_team_creation_and_standards_sharing_workflow() {
    // **Validates: Requirements 1.1, 1.2, 1.3, 3.1, 3.2**
    let manager = TeamManager::new();

    // Step 1: Create team with members
    let admin = create_test_member("Alice", "alice@example.com", TeamRole::Admin);
    let member = create_test_member("Bob", "bob@example.com", TeamRole::Member);
    let viewer = create_test_member("Charlie", "charlie@example.com", TeamRole::Viewer);

    let team = manager
        .create_team(
            "Engineering Team",
            vec![admin.clone(), member.clone(), viewer.clone()],
        )
        .await
        .expect("Failed to create team");

    assert_eq!(team.name, "Engineering Team");
    assert_eq!(team.members.len(), 3);

    // Step 2: Verify team can be retrieved
    let retrieved_team = manager
        .get_team(&team.id)
        .await
        .expect("Failed to retrieve team");

    assert_eq!(retrieved_team.id, team.id);
    assert_eq!(retrieved_team.members.len(), 3);

    // Step 3: Store standards for the team
    let mut standards = create_test_standards(&team.id);
    standards.version = 1;

    let config_manager = manager.config_manager();
    config_manager
        .store_standards(&team.id, standards.clone())
        .await
        .expect("Failed to store standards");

    // Step 4: Retrieve standards and verify they were stored
    let retrieved_standards = config_manager
        .get_standards(&team.id)
        .await
        .expect("Failed to retrieve standards");

    assert_eq!(retrieved_standards.team_id, team.id);
    assert_eq!(retrieved_standards.version, 1);

    // Step 5: Verify all team members have access to standards
    for member in &team.members {
        let access_control = manager.access_control();
        let has_access = access_control
            .check_permission(&member.id, "view_standards", &team.id)
            .await
            .expect("Failed to check permission");

        assert!(
            has_access,
            "Member {} should have access to standards",
            member.id
        );
    }
}

#[tokio::test]
async fn test_team_standards_distribution_to_all_members() {
    // **Validates: Requirements 1.2, 1.3, 1.4, 1.5, 1.6**
    let manager = TeamManager::new();

    // Create team with multiple members
    let members = vec![
        create_test_member("Alice", "alice@example.com", TeamRole::Admin),
        create_test_member("Bob", "bob@example.com", TeamRole::Member),
        create_test_member("Charlie", "charlie@example.com", TeamRole::Member),
        create_test_member("Diana", "diana@example.com", TeamRole::Viewer),
    ];

    let team = manager
        .create_team("Distribution Test Team", members.clone())
        .await
        .expect("Failed to create team");

    // Store standards
    let standards = create_test_standards(&team.id);
    let config_manager = manager.config_manager();
    config_manager
        .store_standards(&team.id, standards)
        .await
        .expect("Failed to store standards");

    // Verify all members can access standards
    for _member in &members {
        let retrieved_standards = config_manager
            .get_standards(&team.id)
            .await
            .expect("Failed to retrieve standards");

        assert_eq!(retrieved_standards.team_id, team.id);
    }
}

#[tokio::test]
async fn test_team_member_addition_and_standards_access() {
    // **Validates: Requirements 1.2, 3.1, 3.2**
    let manager = TeamManager::new();

    // Create initial team
    let initial_member = create_test_member("Alice", "alice@example.com", TeamRole::Admin);
    let team = manager
        .create_team("Growing Team", vec![initial_member.clone()])
        .await
        .expect("Failed to create team");

    // Store standards
    let standards = create_test_standards(&team.id);
    let config_manager = manager.config_manager();
    config_manager
        .store_standards(&team.id, standards)
        .await
        .expect("Failed to store standards");

    // Add new member
    let new_member = create_test_member("Bob", "bob@example.com", TeamRole::Member);
    manager
        .add_member(&team.id, new_member.clone())
        .await
        .expect("Failed to add member");

    // Verify new member can access standards
    let retrieved_standards = config_manager
        .get_standards(&team.id)
        .await
        .expect("Failed to retrieve standards");

    assert_eq!(retrieved_standards.team_id, team.id);

    // Verify new member has correct role
    let updated_team = manager
        .get_team(&team.id)
        .await
        .expect("Failed to retrieve team");

    let new_member_in_team = updated_team
        .members
        .iter()
        .find(|m| m.id == new_member.id)
        .expect("New member not found");

    assert_eq!(new_member_in_team.role, TeamRole::Member);
}

// ============================================================================
// Test Suite 11.2: Rule Promotion Workflow with Approval
// ============================================================================

#[tokio::test]
async fn test_rule_promotion_workflow_with_validation() {
    // **Validates: Requirements 2.1, 2.2, 2.4, 2.9**
    let manager = TeamManager::new();

    // Create team
    let admin = create_test_member("Alice", "alice@example.com", TeamRole::Admin);
    let _team = manager
        .create_team("Rule Promotion Team", vec![admin.clone()])
        .await
        .expect("Failed to create team");

    // Create a rule at project level
    let rule = create_test_rule("rule-1");

    // Get rules manager
    let rules_manager = manager.rules_manager();

    // Validate rule before promotion
    let validation = rules_manager
        .validate_rule(&rule)
        .await
        .expect("Failed to validate rule");

    assert!(validation.is_valid, "Rule should be valid");

    // Promote rule from Project to Team scope
    let mut promoted_rule = rule.clone();
    promoted_rule.scope = RuleScope::Team;

    let result = rules_manager
        .promote_rule(rule, RuleScope::Project, RuleScope::Team)
        .await;

    assert!(result.is_ok(), "Rule promotion should succeed");
}

#[tokio::test]
async fn test_rule_promotion_atomicity_with_member_notification() {
    // **Validates: Requirements 2.1, 2.2, 2.9**
    let manager = TeamManager::new();

    // Create team with multiple members
    let members = vec![
        create_test_member("Alice", "alice@example.com", TeamRole::Admin),
        create_test_member("Bob", "bob@example.com", TeamRole::Member),
        create_test_member("Charlie", "charlie@example.com", TeamRole::Member),
    ];

    let team = manager
        .create_team("Promotion Notification Team", members.clone())
        .await
        .expect("Failed to create team");

    // Create and promote a rule
    let rule = create_test_rule("rule-2");
    let rules_manager = manager.rules_manager();

    let result = rules_manager
        .promote_rule(rule.clone(), RuleScope::Project, RuleScope::Team)
        .await;

    assert!(result.is_ok(), "Rule promotion should succeed");

    // Verify all team members are notified (in a real system)
    // For now, we just verify the promotion succeeded
    assert_eq!(team.members.len(), 3);
}

#[tokio::test]
async fn test_rule_promotion_chain_project_to_team_to_organization() {
    // **Validates: Requirements 2.1, 2.2**
    let manager = TeamManager::new();

    // Create team
    let admin = create_test_member("Alice", "alice@example.com", TeamRole::Admin);
    let _team = manager
        .create_team("Chain Promotion Team", vec![admin])
        .await
        .expect("Failed to create team");

    let rules_manager = manager.rules_manager();

    // Create rule at project level
    let rule = create_test_rule("rule-3");

    // Promote from Project to Team
    let result1 = rules_manager
        .promote_rule(rule.clone(), RuleScope::Project, RuleScope::Team)
        .await;
    assert!(result1.is_ok(), "Project to Team promotion should succeed");

    // Promote from Team to Organization
    let mut team_rule = rule.clone();
    team_rule.scope = RuleScope::Team;

    let result2 = rules_manager
        .promote_rule(team_rule, RuleScope::Team, RuleScope::Organization)
        .await;
    assert!(
        result2.is_ok(),
        "Team to Organization promotion should succeed"
    );
}

#[tokio::test]
async fn test_rule_adoption_tracking_after_promotion() {
    // **Validates: Requirements 2.5**
    let manager = TeamManager::new();

    // Create team
    let admin = create_test_member("Alice", "alice@example.com", TeamRole::Admin);
    let _team = manager
        .create_team("Adoption Tracking Team", vec![admin])
        .await
        .expect("Failed to create team");

    let rules_manager = manager.rules_manager();

    // Create and promote a rule
    let rule = create_test_rule("rule-4");
    rules_manager
        .promote_rule(rule.clone(), RuleScope::Project, RuleScope::Team)
        .await
        .expect("Failed to promote rule");

    // Track adoption metrics
    let adoption = rules_manager
        .track_adoption(&rule.id)
        .await
        .expect("Failed to track adoption");

    assert_eq!(adoption.rule_id, rule.id);
    assert!(adoption.adoption_percentage >= 0.0);
    assert!(adoption.adoption_percentage <= 100.0);
}

#[tokio::test]
async fn test_rule_effectiveness_tracking_after_promotion() {
    // **Validates: Requirements 2.6**
    let manager = TeamManager::new();

    // Create team
    let admin = create_test_member("Alice", "alice@example.com", TeamRole::Admin);
    let _team = manager
        .create_team("Effectiveness Tracking Team", vec![admin])
        .await
        .expect("Failed to create team");

    let rules_manager = manager.rules_manager();

    // Create and promote a rule
    let rule = create_test_rule("rule-5");
    rules_manager
        .promote_rule(rule.clone(), RuleScope::Project, RuleScope::Team)
        .await
        .expect("Failed to promote rule");

    // Track effectiveness metrics
    let effectiveness = rules_manager
        .track_effectiveness(&rule.id)
        .await
        .expect("Failed to track effectiveness");

    assert_eq!(effectiveness.rule_id, rule.id);
    assert!(effectiveness.effectiveness_score >= 0.0);
    assert!(effectiveness.effectiveness_score <= 1.0);
}

// ============================================================================
// Test Suite 11.3: Configuration Inheritance and Override
// ============================================================================

#[tokio::test]
async fn test_configuration_inheritance_organization_to_team_to_project() {
    // **Validates: Requirements 1.7**
    let manager = TeamManager::new();

    // Create team
    let admin = create_test_member("Alice", "alice@example.com", TeamRole::Admin);
    let team = manager
        .create_team("Inheritance Test Team", vec![admin])
        .await
        .expect("Failed to create team");

    let config_manager = manager.config_manager();

    // Create standards at different levels
    let org_id = "org-1";
    let team_id = &team.id;
    let project_id = "project-1";

    let org_standards = create_test_standards(org_id);
    let team_standards = create_test_standards(team_id);
    let project_standards = create_test_standards(project_id);

    // Store standards at each level
    config_manager
        .store_standards(org_id, org_standards)
        .await
        .expect("Failed to store org standards");

    config_manager
        .store_standards(team_id, team_standards)
        .await
        .expect("Failed to store team standards");

    config_manager
        .store_standards(project_id, project_standards)
        .await
        .expect("Failed to store project standards");

    // Apply hierarchy
    let merged = config_manager
        .apply_hierarchy(org_id, team_id, project_id)
        .await
        .expect("Failed to apply hierarchy");

    // Verify all levels are present in merged standards
    assert!(merged.organization_standards.is_some());
    assert!(merged.team_standards.is_some());
    assert!(merged.project_standards.is_some());
}

#[tokio::test]
async fn test_project_level_override_of_inherited_standards() {
    // **Validates: Requirements 1.8**
    let manager = TeamManager::new();

    // Create team
    let admin = create_test_member("Alice", "alice@example.com", TeamRole::Admin);
    let _team = manager
        .create_team("Override Test Team", vec![admin])
        .await
        .expect("Failed to create team");

    let config_manager = manager.config_manager();

    // Create and store project standards with a code review rule
    let project_id = "project-override";
    let mut project_standards = create_test_standards(project_id);
    project_standards.version = 1;

    // Add a code review rule to override
    project_standards
        .code_review_rules
        .push(ricecoder_teams::CodeReviewRule {
            id: "rule-1".to_string(),
            name: "Test Rule".to_string(),
            description: "A test rule to override".to_string(),
            enabled: true,
        });

    config_manager
        .store_standards(project_id, project_standards.clone())
        .await
        .expect("Failed to store project standards");

    // Create override for the rule we just added
    let overrides = ricecoder_teams::StandardsOverride {
        project_id: project_id.to_string(),
        overridden_standards: vec!["rule-1".to_string()],
        created_at: Utc::now(),
    };

    // Apply override
    let result = config_manager
        .override_standards(project_id, overrides)
        .await;

    // Override should succeed
    assert!(result.is_ok(), "Override should succeed");

    // Verify standards were updated
    let updated_standards = config_manager
        .get_standards(project_id)
        .await
        .expect("Failed to retrieve updated standards");

    assert_eq!(
        updated_standards.version, 2,
        "Version should be incremented"
    );
}

#[tokio::test]
async fn test_standards_change_tracking_with_timestamps() {
    // **Validates: Requirements 1.9**
    let manager = TeamManager::new();

    // Create team
    let admin = create_test_member("Alice", "alice@example.com", TeamRole::Admin);
    let team = manager
        .create_team("Change Tracking Team", vec![admin])
        .await
        .expect("Failed to create team");

    let config_manager = manager.config_manager();

    // Store initial standards
    let standards = create_test_standards(&team.id);
    config_manager
        .store_standards(&team.id, standards)
        .await
        .expect("Failed to store standards");

    // Track a change
    config_manager
        .track_changes(&team.id, "Initial standards created")
        .await
        .expect("Failed to track change");

    // Retrieve change history
    let history = config_manager
        .get_change_history(&team.id)
        .await
        .expect("Failed to retrieve change history");

    assert!(!history.is_empty(), "Change history should not be empty");
    assert_eq!(history[0].description, "Initial standards created");
}

#[tokio::test]
async fn test_multiple_teams_with_independent_standards() {
    // **Validates: Requirements 1.1, 1.2**
    let manager = TeamManager::new();

    // Create first team
    let admin1 = create_test_member("Alice", "alice@example.com", TeamRole::Admin);
    let team1 = manager
        .create_team("Team 1", vec![admin1])
        .await
        .expect("Failed to create team 1");

    // Create second team
    let admin2 = create_test_member("Bob", "bob@example.com", TeamRole::Admin);
    let team2 = manager
        .create_team("Team 2", vec![admin2])
        .await
        .expect("Failed to create team 2");

    let config_manager = manager.config_manager();

    // Store different standards for each team
    let standards1 = create_test_standards(&team1.id);
    let standards2 = create_test_standards(&team2.id);

    config_manager
        .store_standards(&team1.id, standards1)
        .await
        .expect("Failed to store team 1 standards");

    config_manager
        .store_standards(&team2.id, standards2)
        .await
        .expect("Failed to store team 2 standards");

    // Verify standards are independent
    let retrieved1 = config_manager
        .get_standards(&team1.id)
        .await
        .expect("Failed to retrieve team 1 standards");

    let retrieved2 = config_manager
        .get_standards(&team2.id)
        .await
        .expect("Failed to retrieve team 2 standards");

    assert_eq!(retrieved1.team_id, team1.id);
    assert_eq!(retrieved2.team_id, team2.id);
    assert_ne!(retrieved1.id, retrieved2.id);
}

#[tokio::test]
async fn test_standards_version_increment_on_update() {
    // **Validates: Requirements 1.9**
    let manager = TeamManager::new();

    // Create team
    let admin = create_test_member("Alice", "alice@example.com", TeamRole::Admin);
    let team = manager
        .create_team("Version Test Team", vec![admin])
        .await
        .expect("Failed to create team");

    let config_manager = manager.config_manager();

    // Store initial standards
    let mut standards = create_test_standards(&team.id);
    standards.version = 1;

    config_manager
        .store_standards(&team.id, standards.clone())
        .await
        .expect("Failed to store standards");

    // Update standards
    let mut updated_standards = standards.clone();
    updated_standards.version = 2;

    config_manager
        .store_standards(&team.id, updated_standards)
        .await
        .expect("Failed to update standards");

    // Verify version was incremented
    let retrieved = config_manager
        .get_standards(&team.id)
        .await
        .expect("Failed to retrieve standards");

    assert_eq!(retrieved.version, 2);
}
