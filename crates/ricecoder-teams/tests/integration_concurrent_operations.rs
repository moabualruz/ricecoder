/// Integration tests for concurrent operations
///
/// Tests concurrent rule promotions, permission changes, and configuration updates
/// to ensure atomicity and consistency under concurrent access.
///
/// **Feature: ricecoder-teams, Concurrent Operations Tests**
/// **Validates: Requirements 2.1, 2.2, 3.7**

use chrono::Utc;
use ricecoder_teams::{
    TeamManager, TeamMember, TeamRole, TeamStandards, SharedRule, RuleScope,
};
use std::sync::Arc;
use tokio::task::JoinHandle;
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
        steering_docs: Vec::new(),
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
        description: "A test rule for concurrent testing".to_string(),
        scope: RuleScope::Project,
        enforced: true,
        promoted_by: "admin-1".to_string(),
        promoted_at: Utc::now(),
        version: 1,
    }
}

// ============================================================================
// Test Suite 11.2.1: Concurrent Rule Promotions
// ============================================================================

#[tokio::test]
async fn test_concurrent_rule_promotions_from_same_team() {
    // **Validates: Requirements 2.1, 2.2**
    let manager = Arc::new(TeamManager::new());

    // Create team
    let admin = create_test_member("Alice", "alice@example.com", TeamRole::Admin);
    let _team = manager
        .create_team("Concurrent Promotion Team", vec![admin])
        .await
        .expect("Failed to create team");

    let rules_manager = manager.rules_manager();

    // Create multiple rules
    let rules: Vec<SharedRule> = (1..=5)
        .map(|i| create_test_rule(&format!("concurrent-rule-{}", i)))
        .collect();

    // Spawn concurrent promotion tasks
    let mut handles: Vec<JoinHandle<Result<(), String>>> = Vec::new();

    for rule in rules {
        let rules_mgr = rules_manager.clone();
        let rule_clone = rule.clone();

        let handle = tokio::spawn(async move {
            rules_mgr
                .promote_rule(rule_clone, RuleScope::Project, RuleScope::Team)
                .await
                .map_err(|e| format!("Promotion failed: {}", e))
        });

        handles.push(handle);
    }

    // Wait for all promotions to complete
    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await);
    }

    // Verify all promotions succeeded
    for result in results {
        assert!(
            result.is_ok(),
            "Concurrent promotion should succeed: {:?}",
            result
        );
        if let Ok(Ok(_)) = result {
            // Success
        } else {
            panic!("Concurrent promotion failed");
        }
    }
}

#[tokio::test]
async fn test_concurrent_rule_promotions_different_scopes() {
    // **Validates: Requirements 2.1, 2.2**
    let manager = Arc::new(TeamManager::new());

    // Create team
    let admin = create_test_member("Alice", "alice@example.com", TeamRole::Admin);
    let _team = manager
        .create_team("Multi-Scope Promotion Team", vec![admin])
        .await
        .expect("Failed to create team");

    let rules_manager = manager.rules_manager();

    // Create rules for different promotion paths
    let rule1 = create_test_rule("rule-project-to-team");
    let rule2 = create_test_rule("rule-team-to-org");

    let rules_mgr1 = rules_manager.clone();
    let rules_mgr2 = rules_manager.clone();

    // Promote rule1 from Project to Team
    let handle1 = tokio::spawn(async move {
        rules_mgr1
            .promote_rule(rule1, RuleScope::Project, RuleScope::Team)
            .await
    });

    // Promote rule2 from Team to Organization
    let mut rule2_team = rule2.clone();
    rule2_team.scope = RuleScope::Team;

    let handle2 = tokio::spawn(async move {
        rules_mgr2
            .promote_rule(rule2_team, RuleScope::Team, RuleScope::Organization)
            .await
    });

    // Wait for both promotions
    let result1 = handle1.await.expect("Task 1 panicked");
    let result2 = handle2.await.expect("Task 2 panicked");

    assert!(result1.is_ok(), "Project to Team promotion should succeed");
    assert!(result2.is_ok(), "Team to Organization promotion should succeed");
}

#[tokio::test]
async fn test_concurrent_rule_promotions_with_validation() {
    // **Validates: Requirements 2.1, 2.4**
    let manager = Arc::new(TeamManager::new());

    // Create team
    let admin = create_test_member("Alice", "alice@example.com", TeamRole::Admin);
    let _team = manager
        .create_team("Validation Promotion Team", vec![admin])
        .await
        .expect("Failed to create team");

    let rules_manager = manager.rules_manager();

    // Create multiple rules
    let rules: Vec<SharedRule> = (1..=3)
        .map(|i| create_test_rule(&format!("validated-rule-{}", i)))
        .collect();

    // Spawn concurrent tasks that validate and promote
    let mut handles: Vec<JoinHandle<Result<(), String>>> = Vec::new();

    for rule in rules {
        let rules_mgr = rules_manager.clone();
        let rule_clone = rule.clone();

        let handle = tokio::spawn(async move {
            // Validate rule
            let validation = rules_mgr
                .validate_rule(&rule_clone)
                .await
                .map_err(|e| format!("Validation failed: {}", e))?;

            if !validation.is_valid {
                return Err("Rule validation failed".to_string());
            }

            // Promote rule
            rules_mgr
                .promote_rule(rule_clone, RuleScope::Project, RuleScope::Team)
                .await
                .map_err(|e| format!("Promotion failed: {}", e))
        });

        handles.push(handle);
    }

    // Wait for all tasks
    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await);
    }

    // Verify all succeeded
    for result in results {
        assert!(
            result.is_ok(),
            "Concurrent validation and promotion should succeed: {:?}",
            result
        );
        if let Ok(Ok(_)) = result {
            // Success
        } else {
            panic!("Concurrent validation and promotion failed");
        }
    }
}

// ============================================================================
// Test Suite 11.2.2: Concurrent Permission Changes
// ============================================================================

#[tokio::test]
async fn test_concurrent_role_assignments() {
    // **Validates: Requirements 3.7**
    let manager = Arc::new(TeamManager::new());

    // Create team with initial members
    let admin = create_test_member("Alice", "alice@example.com", TeamRole::Admin);
    let team = manager
        .create_team("Concurrent Role Team", vec![admin])
        .await
        .expect("Failed to create team");

    let access_control = manager.access_control();

    // Create multiple members to assign roles to
    let members: Vec<TeamMember> = (1..=5)
        .map(|i| {
            create_test_member(
                &format!("Member {}", i),
                &format!("member{}@example.com", i),
                TeamRole::Member,
            )
        })
        .collect();

    // Add all members to team
    for member in &members {
        manager
            .add_member(&team.id, member.clone())
            .await
            .expect("Failed to add member");
    }

    // Spawn concurrent role assignment tasks
    let mut handles: Vec<JoinHandle<Result<(), String>>> = Vec::new();

    for (i, member) in members.iter().enumerate() {
        let access_ctrl = access_control.clone();
        let team_id = team.id.clone();
        let member_id = member.id.clone();
        let role = if i % 2 == 0 {
            TeamRole::Member
        } else {
            TeamRole::Viewer
        };

        let handle = tokio::spawn(async move {
            access_ctrl
                .assign_role(&team_id, &member_id, role)
                .await
                .map_err(|e| format!("Role assignment failed: {}", e))
        });

        handles.push(handle);
    }

    // Wait for all assignments
    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await);
    }

    // Verify all succeeded
    for result in results {
        assert!(
            result.is_ok(),
            "Concurrent role assignment should succeed: {:?}",
            result
        );
        if let Ok(Ok(_)) = result {
            // Success
        } else {
            panic!("Concurrent role assignment failed");
        }
    }
}

#[tokio::test]
async fn test_concurrent_permission_checks() {
    // **Validates: Requirements 3.7**
    let manager = Arc::new(TeamManager::new());

    // Create team
    let admin = create_test_member("Alice", "alice@example.com", TeamRole::Admin);
    let team = manager
        .create_team("Permission Check Team", vec![admin.clone()])
        .await
        .expect("Failed to create team");

    let access_control = manager.access_control();

    // Spawn concurrent permission check tasks
    let mut handles: Vec<JoinHandle<Result<bool, String>>> = Vec::new();

    for _i in 0..10 {
        let access_ctrl = access_control.clone();
        let member_id = admin.id.clone();
        let team_id = team.id.clone();

        let handle = tokio::spawn(async move {
            access_ctrl
                .check_permission(&member_id, "view_standards", &team_id)
                .await
                .map_err(|e| format!("Permission check failed: {}", e))
        });

        handles.push(handle);
    }

    // Wait for all checks
    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await);
    }

    // Verify all succeeded
    for result in results {
        assert!(
            result.is_ok(),
            "Concurrent permission check should succeed: {:?}",
            result
        );
        if let Ok(Ok(has_permission)) = result {
            assert!(has_permission, "Admin should have permission");
        } else {
            panic!("Concurrent permission check failed");
        }
    }
}

// ============================================================================
// Test Suite 11.2.3: Concurrent Configuration Updates
// ============================================================================

#[tokio::test]
async fn test_concurrent_standards_storage() {
    // **Validates: Requirements 1.1, 1.2**
    let manager = Arc::new(TeamManager::new());

    // Create team
    let admin = create_test_member("Alice", "alice@example.com", TeamRole::Admin);
    let team = manager
        .create_team("Concurrent Storage Team", vec![admin])
        .await
        .expect("Failed to create team");

    let config_manager = manager.config_manager();

    // Spawn concurrent storage tasks
    let mut handles: Vec<JoinHandle<Result<(), String>>> = Vec::new();

    for i in 0..5 {
        let config_mgr = config_manager.clone();
        let team_id = team.id.clone();

        let handle = tokio::spawn(async move {
            let mut standards = create_test_standards(&team_id);
            standards.version = i + 1;

            config_mgr
                .store_standards(&team_id, standards)
                .await
                .map_err(|e| format!("Storage failed: {}", e))
        });

        handles.push(handle);
    }

    // Wait for all storage operations
    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await);
    }

    // Verify all succeeded
    for result in results {
        assert!(
            result.is_ok(),
            "Concurrent storage should succeed: {:?}",
            result
        );
        if let Ok(Ok(_)) = result {
            // Success
        } else {
            panic!("Concurrent storage failed");
        }
    }

    // Verify final standards are retrievable
    let final_standards = config_manager
        .get_standards(&team.id)
        .await
        .expect("Failed to retrieve final standards");

    assert_eq!(final_standards.team_id, team.id);
}

#[tokio::test]
async fn test_concurrent_standards_retrieval() {
    // **Validates: Requirements 1.2**
    let manager = Arc::new(TeamManager::new());

    // Create team
    let admin = create_test_member("Alice", "alice@example.com", TeamRole::Admin);
    let team = manager
        .create_team("Concurrent Retrieval Team", vec![admin])
        .await
        .expect("Failed to create team");

    let config_manager = manager.config_manager();

    // Store standards
    let standards = create_test_standards(&team.id);
    config_manager
        .store_standards(&team.id, standards)
        .await
        .expect("Failed to store standards");

    // Spawn concurrent retrieval tasks
    let mut handles: Vec<JoinHandle<Result<String, String>>> = Vec::new();

    for _ in 0..10 {
        let config_mgr = config_manager.clone();
        let team_id = team.id.clone();

        let handle = tokio::spawn(async move {
            let standards = config_mgr
                .get_standards(&team_id)
                .await
                .map_err(|e| format!("Retrieval failed: {}", e))?;

            Ok(standards.id)
        });

        handles.push(handle);
    }

    // Wait for all retrievals
    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await);
    }

    // Verify all succeeded and got same standards
    let first_id = if let Ok(Ok(id)) = &results[0] {
        id.clone()
    } else {
        panic!("First retrieval failed");
    };

    for result in results {
        assert!(
            result.is_ok(),
            "Concurrent retrieval should succeed: {:?}",
            result
        );
        if let Ok(Ok(id)) = result {
            assert_eq!(id, first_id, "All retrievals should get same standards");
        } else {
            panic!("Concurrent retrieval failed");
        }
    }
}

#[tokio::test]
async fn test_concurrent_member_operations() {
    // **Validates: Requirements 3.1, 3.2**
    let manager = Arc::new(TeamManager::new());

    // Create team
    let admin = create_test_member("Alice", "alice@example.com", TeamRole::Admin);
    let team = manager
        .create_team("Concurrent Member Team", vec![admin])
        .await
        .expect("Failed to create team");

    // Spawn concurrent member addition tasks
    let mut handles: Vec<JoinHandle<Result<(), String>>> = Vec::new();

    for i in 0..5 {
        let mgr = manager.clone();
        let team_id = team.id.clone();

        let handle = tokio::spawn(async move {
            let member = create_test_member(
                &format!("Concurrent Member {}", i),
                &format!("concurrent{}@example.com", i),
                TeamRole::Member,
            );

            mgr.add_member(&team_id, member)
                .await
                .map_err(|e| format!("Add member failed: {}", e))
        });

        handles.push(handle);
    }

    // Wait for all additions
    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await);
    }

    // Verify all succeeded
    for result in results {
        assert!(
            result.is_ok(),
            "Concurrent member addition should succeed: {:?}",
            result
        );
        if let Ok(Ok(_)) = result {
            // Success
        } else {
            panic!("Concurrent member addition failed");
        }
    }

    // Verify final team has all members
    let final_team = manager
        .get_team(&team.id)
        .await
        .expect("Failed to retrieve team");

    assert_eq!(final_team.members.len(), 6); // 1 admin + 5 added members
}
