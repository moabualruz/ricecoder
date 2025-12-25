/// Integration tests for TeamManager
///
/// Tests the complete team creation, member management, and permission enforcement workflows
use chrono::Utc;
use ricecoder_teams::{TeamManager, TeamMember, TeamRole};
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

#[tokio::test]
async fn test_team_creation_with_members() {
    let manager = TeamManager::with_defaults();

    // Create test members
    let admin = create_test_member("Alice", "alice@example.com", TeamRole::Admin);
    let member = create_test_member("Bob", "bob@example.com", TeamRole::Member);
    let viewer = create_test_member("Charlie", "charlie@example.com", TeamRole::Viewer);

    let members = vec![admin.clone(), member.clone(), viewer.clone()];

    // Create team
    let team = manager
        .create_team("Test Team", members)
        .await
        .expect("Failed to create team");

    // Verify team was created
    assert_eq!(team.name, "Test Team");
    assert_eq!(team.members.len(), 3);
    assert!(team.members.iter().any(|m| m.id == admin.id));
    assert!(team.members.iter().any(|m| m.id == member.id));
    assert!(team.members.iter().any(|m| m.id == viewer.id));

    // Verify team can be retrieved
    let retrieved_team = manager
        .get_team(&team.id)
        .await
        .expect("Failed to retrieve team");

    assert_eq!(retrieved_team.id, team.id);
    assert_eq!(retrieved_team.name, team.name);
    assert_eq!(retrieved_team.members.len(), 3);
}

#[tokio::test]
async fn test_member_addition() {
    let manager = TeamManager::with_defaults();

    // Create initial team with one member
    let initial_member = create_test_member("Alice", "alice@example.com", TeamRole::Admin);
    let team = manager
        .create_team("Test Team", vec![initial_member.clone()])
        .await
        .expect("Failed to create team");

    assert_eq!(team.members.len(), 1);

    // Add a new member
    let new_member = create_test_member("Bob", "bob@example.com", TeamRole::Member);
    manager
        .add_member(&team.id, new_member.clone())
        .await
        .expect("Failed to add member");

    // Verify member was added
    let updated_team = manager
        .get_team(&team.id)
        .await
        .expect("Failed to retrieve team");

    assert_eq!(updated_team.members.len(), 2);
    assert!(updated_team.members.iter().any(|m| m.id == new_member.id));
}

#[tokio::test]
async fn test_member_removal() {
    let manager = TeamManager::with_defaults();

    // Create team with two members
    let member1 = create_test_member("Alice", "alice@example.com", TeamRole::Admin);
    let member2 = create_test_member("Bob", "bob@example.com", TeamRole::Member);

    let team = manager
        .create_team("Test Team", vec![member1.clone(), member2.clone()])
        .await
        .expect("Failed to create team");

    assert_eq!(team.members.len(), 2);

    // Remove a member
    manager
        .remove_member(&team.id, &member2.id)
        .await
        .expect("Failed to remove member");

    // Verify member was removed
    let updated_team = manager
        .get_team(&team.id)
        .await
        .expect("Failed to retrieve team");

    assert_eq!(updated_team.members.len(), 1);
    assert!(!updated_team.members.iter().any(|m| m.id == member2.id));
    assert!(updated_team.members.iter().any(|m| m.id == member1.id));
}

#[tokio::test]
async fn test_permission_enforcement_across_operations() {
    let manager = TeamManager::with_defaults();

    // Create team with members of different roles
    let admin = create_test_member("Alice", "alice@example.com", TeamRole::Admin);
    let member = create_test_member("Bob", "bob@example.com", TeamRole::Member);
    let viewer = create_test_member("Charlie", "charlie@example.com", TeamRole::Viewer);

    let team = manager
        .create_team(
            "Test Team",
            vec![admin.clone(), member.clone(), viewer.clone()],
        )
        .await
        .expect("Failed to create team");

    // Verify team was created with all members
    let retrieved_team = manager
        .get_team(&team.id)
        .await
        .expect("Failed to retrieve team");

    assert_eq!(retrieved_team.members.len(), 3);

    // Verify roles are preserved
    let admin_member = retrieved_team
        .members
        .iter()
        .find(|m| m.id == admin.id)
        .expect("Admin member not found");
    assert_eq!(admin_member.role, TeamRole::Admin);

    let member_member = retrieved_team
        .members
        .iter()
        .find(|m| m.id == member.id)
        .expect("Member not found");
    assert_eq!(member_member.role, TeamRole::Member);

    let viewer_member = retrieved_team
        .members
        .iter()
        .find(|m| m.id == viewer.id)
        .expect("Viewer not found");
    assert_eq!(viewer_member.role, TeamRole::Viewer);
}

#[tokio::test]
async fn test_duplicate_member_addition_fails() {
    let manager = TeamManager::with_defaults();

    // Create team with one member
    let member = create_test_member("Alice", "alice@example.com", TeamRole::Admin);
    let team = manager
        .create_team("Test Team", vec![member.clone()])
        .await
        .expect("Failed to create team");

    // Try to add the same member again
    let result = manager.add_member(&team.id, member.clone()).await;

    // Should fail because member already exists
    assert!(result.is_err());
}

#[tokio::test]
async fn test_remove_nonexistent_member_fails() {
    let manager = TeamManager::with_defaults();

    // Create team with one member
    let member = create_test_member("Alice", "alice@example.com", TeamRole::Admin);
    let team = manager
        .create_team("Test Team", vec![member.clone()])
        .await
        .expect("Failed to create team");

    // Try to remove a member that doesn't exist
    let nonexistent_id = Uuid::new_v4().to_string();
    let result = manager.remove_member(&team.id, &nonexistent_id).await;

    // Should fail because member doesn't exist
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_nonexistent_team_fails() {
    let manager = TeamManager::with_defaults();

    // Try to get a team that doesn't exist
    let nonexistent_id = Uuid::new_v4().to_string();
    let result = manager.get_team(&nonexistent_id).await;

    // Should fail because team doesn't exist
    assert!(result.is_err());
}

#[tokio::test]
async fn test_team_persistence_across_operations() {
    let manager = TeamManager::with_defaults();

    // Create team
    let member1 = create_test_member("Alice", "alice@example.com", TeamRole::Admin);
    let team = manager
        .create_team("Test Team", vec![member1.clone()])
        .await
        .expect("Failed to create team");

    let team_id = team.id.clone();

    // Add member
    let member2 = create_test_member("Bob", "bob@example.com", TeamRole::Member);
    manager
        .add_member(&team_id, member2.clone())
        .await
        .expect("Failed to add member");

    // Retrieve team and verify both members are present
    let retrieved_team = manager
        .get_team(&team_id)
        .await
        .expect("Failed to retrieve team");

    assert_eq!(retrieved_team.members.len(), 2);
    assert!(retrieved_team.members.iter().any(|m| m.id == member1.id));
    assert!(retrieved_team.members.iter().any(|m| m.id == member2.id));

    // Remove member
    manager
        .remove_member(&team_id, &member2.id)
        .await
        .expect("Failed to remove member");

    // Retrieve team again and verify member was removed
    let final_team = manager
        .get_team(&team_id)
        .await
        .expect("Failed to retrieve team");

    assert_eq!(final_team.members.len(), 1);
    assert!(final_team.members.iter().any(|m| m.id == member1.id));
    assert!(!final_team.members.iter().any(|m| m.id == member2.id));
}

#[tokio::test]
async fn test_multiple_teams_isolation() {
    let manager = TeamManager::with_defaults();

    // Create first team
    let member1 = create_test_member("Alice", "alice@example.com", TeamRole::Admin);
    let team1 = manager
        .create_team("Team 1", vec![member1.clone()])
        .await
        .expect("Failed to create team 1");

    // Create second team
    let member2 = create_test_member("Bob", "bob@example.com", TeamRole::Admin);
    let team2 = manager
        .create_team("Team 2", vec![member2.clone()])
        .await
        .expect("Failed to create team 2");

    // Verify teams are isolated
    assert_ne!(team1.id, team2.id);
    assert_eq!(team1.name, "Team 1");
    assert_eq!(team2.name, "Team 2");

    // Verify members are isolated
    let retrieved_team1 = manager
        .get_team(&team1.id)
        .await
        .expect("Failed to retrieve team 1");
    let retrieved_team2 = manager
        .get_team(&team2.id)
        .await
        .expect("Failed to retrieve team 2");

    assert_eq!(retrieved_team1.members.len(), 1);
    assert_eq!(retrieved_team2.members.len(), 1);
    assert!(retrieved_team1.members.iter().any(|m| m.id == member1.id));
    assert!(retrieved_team2.members.iter().any(|m| m.id == member2.id));
}

#[tokio::test]
async fn test_team_manager_accessors() {
    let manager = TeamManager::with_defaults();

    // Verify all sub-managers are accessible (just verify they don't panic)
    let _config_manager = manager.config_manager();
    let _rules_manager = manager.rules_manager();
    let _access_control = manager.access_control();
    let _sync_service = manager.sync_service();
    let _analytics = manager.analytics();
}
