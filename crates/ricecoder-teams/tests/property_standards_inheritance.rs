use chrono::Utc;
/// Property-based tests for standards inheritance
/// **Feature: ricecoder-teams, Property 1: Standards Inheritance Consistency**
/// **Validates: Requirements 1.6, 1.7**
use proptest::prelude::*;
use ricecoder_teams::{
    config::TeamConfigManager,
    models::{CodeReviewRule, ComplianceRequirement, GovernanceDoc, TeamStandards, Template},
};

/// Strategy for generating random CodeReviewRule
fn arb_code_review_rule() -> impl Strategy<Value = CodeReviewRule> {
    (
        "[a-z0-9]{1,20}",
        "[a-z0-9 ]{1,50}",
        "[a-z0-9 ]{1,100}",
        any::<bool>(),
    )
        .prop_map(|(id, name, description, enabled)| CodeReviewRule {
            id,
            name,
            description,
            enabled,
        })
}

/// Strategy for generating random Template
fn arb_template() -> impl Strategy<Value = Template> {
    (
        "[a-z0-9]{1,20}",
        "[a-z0-9 ]{1,50}",
        "[a-z0-9 ]{1,100}",
        "[a-z0-9 ]{1,200}",
    )
        .prop_map(|(id, name, description, content)| Template {
            id,
            name,
            description,
            content,
        })
}

/// Strategy for generating random GovernanceDoc
fn arb_governance_doc() -> impl Strategy<Value = GovernanceDoc> {
    ("[a-z0-9]{1,20}", "[a-z0-9 ]{1,50}", "[a-z0-9 ]{1,200}")
        .prop_map(|(id, name, content)| GovernanceDoc { id, name, content })
}

/// Strategy for generating random ComplianceRequirement
fn arb_compliance_requirement() -> impl Strategy<Value = ComplianceRequirement> {
    ("[a-z0-9]{1,20}", "[a-z0-9 ]{1,50}", "[a-z0-9 ]{1,100}").prop_map(|(id, name, description)| {
        ComplianceRequirement {
            id,
            name,
            description,
        }
    })
}

/// Strategy for generating random TeamStandards
fn arb_team_standards(team_id: &str) -> impl Strategy<Value = TeamStandards> {
    let team_id = team_id.to_string();
    (
        "[a-z0-9]{1,20}",
        prop::collection::vec(arb_code_review_rule(), 0..3),
        prop::collection::vec(arb_template(), 0..3),
        prop::collection::vec(arb_governance_doc(), 0..3),
        prop::collection::vec(arb_compliance_requirement(), 0..3),
        1u32..10u32,
    )
        .prop_map(
            move |(id, rules, templates, docs, compliance, version)| TeamStandards {
                id,
                team_id: team_id.clone(),
                code_review_rules: rules,
                templates,
                governance_docs: docs,
                compliance_requirements: compliance,
                version,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        )
}

proptest! {
    /// Property 1: Standards Inheritance Consistency
    /// For any organization, team, and project standards hierarchies,
    /// the final merged standards SHALL be the result of merging in priority order
    /// with project-level overrides taking precedence.
    #[test]
    fn prop_standards_inheritance_consistency(
        org_standards in arb_team_standards("org-1"),
        team_standards in arb_team_standards("team-1"),
        project_standards in arb_team_standards("project-1"),
    ) {
        // Verify that organization standards are included in the merge
        let merged = TeamConfigManager::merge_standards_hierarchy(
            Some(org_standards.clone()),
            Some(team_standards.clone()),
            Some(project_standards.clone()),
        ).expect("Merge should succeed");

        // Property: Final standards should contain rules from all levels
        // (since we're extending, not replacing)
        let total_org_rules = org_standards.code_review_rules.len();
        let total_team_rules = team_standards.code_review_rules.len();
        let total_project_rules = project_standards.code_review_rules.len();
        let expected_total = total_org_rules + total_team_rules + total_project_rules;

        prop_assert_eq!(
            merged.code_review_rules.len(),
            expected_total,
            "Merged standards should contain all rules from all levels"
        );

        // Property: Project standards version should be used (highest priority)
        prop_assert_eq!(
            merged.version,
            project_standards.version,
            "Project standards version should take precedence"
        );

        // Property: Updated timestamp should be set to current time (approximately)
        let now = Utc::now();
        let time_diff = (now - merged.updated_at).num_seconds();
        prop_assert!(
            time_diff >= 0 && time_diff <= 5,
            "Updated timestamp should be recent"
        );
    }

    /// Property: Standards inheritance with partial hierarchy
    /// When some levels are missing, the merge should still work correctly
    #[test]
    fn prop_standards_inheritance_partial_hierarchy(
        org_standards in arb_team_standards("org-1"),
        project_standards in arb_team_standards("project-1"),
    ) {
        // Merge with only organization and project (no team)
        let merged = TeamConfigManager::merge_standards_hierarchy(
            Some(org_standards.clone()),
            None,
            Some(project_standards.clone()),
        ).expect("Merge should succeed");

        // Property: Should contain rules from both org and project
        let expected_total = org_standards.code_review_rules.len()
            + project_standards.code_review_rules.len();
        prop_assert_eq!(
            merged.code_review_rules.len(),
            expected_total,
            "Merged standards should contain rules from org and project"
        );

        // Property: Project version should be used
        prop_assert_eq!(
            merged.version,
            project_standards.version,
            "Project standards version should take precedence"
        );
    }

    /// Property: Standards inheritance with only organization
    /// When only organization standards exist, they should be used as-is
    #[test]
    fn prop_standards_inheritance_org_only(
        org_standards in arb_team_standards("org-1"),
    ) {
        let merged = TeamConfigManager::merge_standards_hierarchy(
            Some(org_standards.clone()),
            None,
            None,
        ).expect("Merge should succeed");

        // Property: Should contain exactly the organization rules
        prop_assert_eq!(
            merged.code_review_rules.len(),
            org_standards.code_review_rules.len(),
            "Merged standards should contain only org rules"
        );

        // Property: Version should match organization
        prop_assert_eq!(
            merged.version,
            org_standards.version,
            "Version should match organization standards"
        );
    }

    /// Property: Standards inheritance with empty hierarchy
    /// When no standards exist, a default should be created
    #[test]
    fn prop_standards_inheritance_empty_hierarchy(_unit in Just(())) {
        let merged = TeamConfigManager::merge_standards_hierarchy(
            None,
            None,
            None,
        ).expect("Merge should succeed");

        // Property: Should have default values
        prop_assert_eq!(
            merged.code_review_rules.len(),
            0,
            "Empty hierarchy should result in empty rules"
        );

        prop_assert_eq!(
            merged.version,
            1,
            "Default version should be 1"
        );
    }

    /// Property: Standards inheritance preserves all content types
    /// All types of standards (rules, templates, docs, compliance) should be preserved
    #[test]
    fn prop_standards_inheritance_preserves_all_types(
        org_standards in arb_team_standards("org-1"),
        team_standards in arb_team_standards("team-1"),
        project_standards in arb_team_standards("project-1"),
    ) {
        let merged = TeamConfigManager::merge_standards_hierarchy(
            Some(org_standards.clone()),
            Some(team_standards.clone()),
            Some(project_standards.clone()),
        ).expect("Merge should succeed");

        // Property: All content types should be present
        let expected_rules = org_standards.code_review_rules.len()
            + team_standards.code_review_rules.len()
            + project_standards.code_review_rules.len();
        let expected_templates = org_standards.templates.len()
            + team_standards.templates.len()
            + project_standards.templates.len();
        let expected_docs = org_standards.governance_docs.len()
            + team_standards.governance_docs.len()
            + project_standards.governance_docs.len();
        let expected_compliance = org_standards.compliance_requirements.len()
            + team_standards.compliance_requirements.len()
            + project_standards.compliance_requirements.len();

        prop_assert_eq!(
            merged.code_review_rules.len(),
            expected_rules,
            "All code review rules should be preserved"
        );

        prop_assert_eq!(
            merged.templates.len(),
            expected_templates,
            "All templates should be preserved"
        );

        prop_assert_eq!(
            merged.governance_docs.len(),
            expected_docs,
            "All Governance docs should be preserved"
        );

        prop_assert_eq!(
            merged.compliance_requirements.len(),
            expected_compliance,
            "All compliance requirements should be preserved"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_standards_hierarchy_basic() {
        let org = TeamStandards {
            id: "org-1".to_string(),
            team_id: "org".to_string(),
            code_review_rules: vec![CodeReviewRule {
                id: "rule-1".to_string(),
                name: "Rule 1".to_string(),
                description: "Org rule".to_string(),
                enabled: true,
            }],
            templates: vec![],
            governance_docs: vec![],
            compliance_requirements: vec![],
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let team = TeamStandards {
            id: "team-1".to_string(),
            team_id: "team".to_string(),
            code_review_rules: vec![CodeReviewRule {
                id: "rule-2".to_string(),
                name: "Rule 2".to_string(),
                description: "Team rule".to_string(),
                enabled: true,
            }],
            templates: vec![],
            governance_docs: vec![],
            compliance_requirements: vec![],
            version: 2,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let project = TeamStandards {
            id: "project-1".to_string(),
            team_id: "project".to_string(),
            code_review_rules: vec![CodeReviewRule {
                id: "rule-3".to_string(),
                name: "Rule 3".to_string(),
                description: "Project rule".to_string(),
                enabled: true,
            }],
            templates: vec![],
            governance_docs: vec![],
            compliance_requirements: vec![],
            version: 3,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let merged =
            TeamConfigManager::merge_standards_hierarchy(Some(org), Some(team), Some(project))
                .expect("Merge should succeed");

        // Should have all 3 rules
        assert_eq!(merged.code_review_rules.len(), 3);
        // Project version should be used
        assert_eq!(merged.version, 3);
    }

    #[test]
    fn test_merge_standards_hierarchy_with_none() {
        let org = TeamStandards {
            id: "org-1".to_string(),
            team_id: "org".to_string(),
            code_review_rules: vec![CodeReviewRule {
                id: "rule-1".to_string(),
                name: "Rule 1".to_string(),
                description: "Org rule".to_string(),
                enabled: true,
            }],
            templates: vec![],
            governance_docs: vec![],
            compliance_requirements: vec![],
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let merged = TeamConfigManager::merge_standards_hierarchy(Some(org.clone()), None, None)
            .expect("Merge should succeed");

        // Should have only org rule
        assert_eq!(merged.code_review_rules.len(), 1);
        // Org version should be used
        assert_eq!(merged.version, 1);
    }

    #[test]
    fn test_merge_standards_hierarchy_all_none() {
        let merged = TeamConfigManager::merge_standards_hierarchy(None, None, None)
            .expect("Merge should succeed");

        // Should have default values
        assert_eq!(merged.code_review_rules.len(), 0);
        assert_eq!(merged.version, 1);
    }
}
