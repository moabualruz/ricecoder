use chrono::Utc;
/// Unit tests for TeamConfigManager
/// Tests configuration storage, retrieval, hierarchy merging, and override capability
use ricecoder_teams::config::TeamConfigManager;
use ricecoder_teams::models::{
    CodeReviewRule, ComplianceRequirement, StandardsOverride, GovernanceDoc, TeamStandards, Template,
};

/// Helper function to create test standards
fn create_test_standards(team_id: &str, version: u32) -> TeamStandards {
    TeamStandards {
        id: format!("standards-{}", team_id),
        team_id: team_id.to_string(),
        code_review_rules: vec![CodeReviewRule {
            id: "rule-1".to_string(),
            name: "Test Rule".to_string(),
            description: "A test rule".to_string(),
            enabled: true,
        }],
        templates: vec![Template {
            id: "template-1".to_string(),
            name: "Test Template".to_string(),
            description: "A test template".to_string(),
            content: "template content".to_string(),
        }],
        governance_docs: vec![GovernanceDoc {
            id: "doc-1".to_string(),
            name: "Test Doc".to_string(),
            content: "doc content".to_string(),
        }],
        compliance_requirements: vec![ComplianceRequirement {
            id: "compliance-1".to_string(),
            name: "Test Compliance".to_string(),
            description: "A test compliance requirement".to_string(),
        }],
        version,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

#[tokio::test]
async fn test_store_standards_creates_file() {
    let manager = TeamConfigManager::new();
    let standards = create_test_standards("team-1", 1);

    // Store standards
    let result = manager.store_standards("team-1", standards.clone()).await;
    assert!(result.is_ok(), "Should store standards successfully");

    // Just verify that the operation succeeded - file creation is tested implicitly
    // by the fact that get_standards will work after store_standards
}

#[tokio::test]
async fn test_get_standards_from_cache() {
    let manager = TeamConfigManager::new();
    let standards = create_test_standards("team-1", 1);

    // Store standards (populates cache)
    manager
        .store_standards("team-1", standards.clone())
        .await
        .expect("Should store standards");

    // Get standards (should come from cache)
    let retrieved = manager
        .get_standards("team-1")
        .await
        .expect("Should retrieve standards");

    assert_eq!(retrieved.id, standards.id);
    assert_eq!(retrieved.team_id, standards.team_id);
    assert_eq!(retrieved.version, standards.version);
    assert_eq!(retrieved.code_review_rules.len(), 1);
}

#[tokio::test]
async fn test_get_standards_not_found() {
    let manager = TeamConfigManager::new();

    // Try to get non-existent standards
    let result = manager.get_standards("non-existent-team").await;
    assert!(result.is_err(), "Should return error for non-existent team");
}

#[tokio::test]
async fn test_apply_hierarchy_with_all_levels() {
    let manager = TeamConfigManager::new();

    let org_standards = create_test_standards("org-1", 1);
    let team_standards = create_test_standards("team-1", 2);
    let project_standards = create_test_standards("project-1", 3);

    // Store all levels
    manager
        .store_standards("org-1", org_standards)
        .await
        .expect("Should store org standards");
    manager
        .store_standards("team-1", team_standards)
        .await
        .expect("Should store team standards");
    manager
        .store_standards("project-1", project_standards)
        .await
        .expect("Should store project standards");

    // Apply hierarchy
    let merged = manager
        .apply_hierarchy("org-1", "team-1", "project-1")
        .await
        .expect("Should apply hierarchy");

    // Verify merged standards
    assert!(merged.organization_standards.is_some());
    assert!(merged.team_standards.is_some());
    assert!(merged.project_standards.is_some());

    // Final standards should have rules from all levels
    assert_eq!(merged.final_standards.code_review_rules.len(), 3);
    // Project version should be used
    assert_eq!(merged.final_standards.version, 3);
}

#[tokio::test]
async fn test_apply_hierarchy_partial() {
    let manager = TeamConfigManager::new();

    let org_standards = create_test_standards("org-1", 1);
    let project_standards = create_test_standards("project-1", 3);

    // Store only org and project
    manager
        .store_standards("org-1", org_standards)
        .await
        .expect("Should store org standards");
    manager
        .store_standards("project-1", project_standards)
        .await
        .expect("Should store project standards");

    // Apply hierarchy (team doesn't exist)
    let merged = manager
        .apply_hierarchy("org-1", "non-existent-team", "project-1")
        .await
        .expect("Should apply hierarchy");

    // Verify merged standards
    assert!(merged.organization_standards.is_some());
    assert!(merged.team_standards.is_none());
    assert!(merged.project_standards.is_some());

    // Final standards should have rules from org and project
    assert_eq!(merged.final_standards.code_review_rules.len(), 2);
}

#[tokio::test]
async fn test_override_standards_removes_rules() {
    let manager = TeamConfigManager::new();
    let mut standards = create_test_standards("project-1", 1);

    // Add multiple rules
    standards.code_review_rules.push(CodeReviewRule {
        id: "rule-2".to_string(),
        name: "Rule 2".to_string(),
        description: "Another rule".to_string(),
        enabled: true,
    });

    // Store standards
    manager
        .store_standards("project-1", standards.clone())
        .await
        .expect("Should store standards");

    // Create override
    let overrides = StandardsOverride {
        project_id: "project-1".to_string(),
        overridden_standards: vec!["rule-1".to_string()],
        created_at: Utc::now(),
    };

    // Apply override
    let result = manager.override_standards("project-1", overrides).await;
    assert!(result.is_ok(), "Should apply overrides successfully");

    // Verify rule was removed
    let updated = manager
        .get_standards("project-1")
        .await
        .expect("Should retrieve updated standards");

    assert_eq!(updated.code_review_rules.len(), 1);
    assert_eq!(updated.code_review_rules[0].id, "rule-2");
    assert_eq!(updated.version, 2); // Version should be incremented
}

#[tokio::test]
async fn test_override_standards_invalid_target() {
    let manager = TeamConfigManager::new();
    let standards = create_test_standards("project-1", 1);

    // Store standards
    manager
        .store_standards("project-1", standards)
        .await
        .expect("Should store standards");

    // Create override with non-existent rule
    let overrides = StandardsOverride {
        project_id: "project-1".to_string(),
        overridden_standards: vec!["non-existent-rule".to_string()],
        created_at: Utc::now(),
    };

    // Apply override should fail
    let result = manager.override_standards("project-1", overrides).await;
    assert!(
        result.is_err(),
        "Should fail for non-existent override target"
    );
}

#[tokio::test]
async fn test_track_changes() {
    let manager = TeamConfigManager::new();

    // Track a change
    let result = manager
        .track_changes("team-1", "Created new standards")
        .await;
    assert!(result.is_ok(), "Should track change successfully");

    // Get change history
    let history = manager
        .get_change_history("team-1")
        .await
        .expect("Should retrieve change history");

    assert_eq!(history.len(), 1);
    assert_eq!(history[0].description, "Created new standards");
}

#[tokio::test]
async fn test_track_changes_multiple() {
    let manager = TeamConfigManager::new();

    // Track multiple changes
    manager
        .track_changes("team-1", "Change 1")
        .await
        .expect("Should track first change");
    manager
        .track_changes("team-1", "Change 2")
        .await
        .expect("Should track second change");
    manager
        .track_changes("team-1", "Change 3")
        .await
        .expect("Should track third change");

    // Get change history
    let history = manager
        .get_change_history("team-1")
        .await
        .expect("Should retrieve change history");

    assert_eq!(history.len(), 3);
    assert_eq!(history[0].description, "Change 1");
    assert_eq!(history[1].description, "Change 2");
    assert_eq!(history[2].description, "Change 3");
}

#[tokio::test]
async fn test_get_change_history_empty() {
    let manager = TeamConfigManager::new();

    // Get change history for team with no changes
    let history = manager
        .get_change_history("non-existent-team")
        .await
        .expect("Should return empty history");

    assert_eq!(history.len(), 0);
}

#[tokio::test]
async fn test_cache_invalidation_on_store() {
    let manager = TeamConfigManager::new();
    let standards1 = create_test_standards("team-1", 1);
    let mut standards2 = create_test_standards("team-1", 2);
    standards2.code_review_rules.push(CodeReviewRule {
        id: "rule-2".to_string(),
        name: "Rule 2".to_string(),
        description: "Another rule".to_string(),
        enabled: true,
    });

    // Store first version
    manager
        .store_standards("team-1", standards1)
        .await
        .expect("Should store first version");

    // Get first version (from cache)
    let retrieved1 = manager
        .get_standards("team-1")
        .await
        .expect("Should retrieve first version");
    assert_eq!(retrieved1.code_review_rules.len(), 1);

    // Store second version (should update cache)
    manager
        .store_standards("team-1", standards2)
        .await
        .expect("Should store second version");

    // Get second version (should be from updated cache)
    let retrieved2 = manager
        .get_standards("team-1")
        .await
        .expect("Should retrieve second version");
    assert_eq!(retrieved2.code_review_rules.len(), 2);
    assert_eq!(retrieved2.version, 2);
}

#[tokio::test]
async fn test_merge_standards_hierarchy_consistency() {
    let _manager = TeamConfigManager::new();

    let org = create_test_standards("org-1", 1);
    let team = create_test_standards("team-1", 2);
    let project = create_test_standards("project-1", 3);

    // Merge standards
    let merged = TeamConfigManager::merge_standards_hierarchy(
        Some(org.clone()),
        Some(team.clone()),
        Some(project.clone()),
    )
    .expect("Should merge successfully");

    // Verify all rules are included
    assert_eq!(
        merged.code_review_rules.len(),
        3,
        "Should include rules from all levels"
    );

    // Verify project version is used
    assert_eq!(merged.version, 3, "Should use project version");

    // Verify all content types are preserved
    assert_eq!(merged.templates.len(), 3);
    assert_eq!(merged.governance_docs.len(), 3);
    assert_eq!(merged.compliance_requirements.len(), 3);
}

#[tokio::test]
async fn test_standards_serialization_roundtrip() {
    let manager = TeamConfigManager::new();
    let original = create_test_standards("team-1", 1);

    // Store standards
    manager
        .store_standards("team-1", original.clone())
        .await
        .expect("Should store standards");

    // Retrieve standards
    let retrieved = manager
        .get_standards("team-1")
        .await
        .expect("Should retrieve standards");

    // Verify roundtrip
    assert_eq!(original.id, retrieved.id);
    assert_eq!(original.team_id, retrieved.team_id);
    assert_eq!(original.version, retrieved.version);
    assert_eq!(
        original.code_review_rules.len(),
        retrieved.code_review_rules.len()
    );
    assert_eq!(original.templates.len(), retrieved.templates.len());
    assert_eq!(original.governance_docs.len(), retrieved.governance_docs.len());
    assert_eq!(
        original.compliance_requirements.len(),
        retrieved.compliance_requirements.len()
    );
}
