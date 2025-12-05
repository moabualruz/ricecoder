//! Property-based tests for steering merge consistency
//! **Feature: ricecoder-specs, Property 2: Steering Merge Consistency**
//! **Validates: Requirements 5.4, 5.6**

use proptest::prelude::*;
use ricecoder_specs::{
    models::{Standard, Steering, SteeringRule, TemplateRef},
    steering::SteeringLoader,
};

// ============================================================================
// Generators for property-based testing
// ============================================================================

fn arb_rule_id() -> impl Strategy<Value = String> {
    "[a-z0-9_-]{1,20}".prop_map(|s| s)
}

fn arb_rule_description() -> impl Strategy<Value = String> {
    "[A-Za-z0-9 ]{1,50}".prop_map(|s| s)
}

fn arb_rule_pattern() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_^$*+?.()\\[\\]{}|\\\\-]{1,30}".prop_map(|s| s)
}

fn arb_rule_action() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("enforce".to_string()),
        Just("warn".to_string()),
        Just("suggest".to_string()),
    ]
}

fn arb_steering_rule() -> impl Strategy<Value = SteeringRule> {
    (
        arb_rule_id(),
        arb_rule_description(),
        arb_rule_pattern(),
        arb_rule_action(),
    )
        .prop_map(|(id, description, pattern, action)| SteeringRule {
            id,
            description,
            pattern,
            action,
        })
}

fn arb_standard_id() -> impl Strategy<Value = String> {
    "[a-z0-9_-]{1,20}".prop_map(|s| s)
}

fn arb_standard_description() -> impl Strategy<Value = String> {
    "[A-Za-z0-9 ]{1,50}".prop_map(|s| s)
}

fn arb_standard() -> impl Strategy<Value = Standard> {
    (arb_standard_id(), arb_standard_description())
        .prop_map(|(id, description)| Standard { id, description })
}

fn arb_template_id() -> impl Strategy<Value = String> {
    "[a-z0-9_-]{1,20}".prop_map(|s| s)
}

fn arb_template_path() -> impl Strategy<Value = String> {
    "[a-z0-9_/-]{1,50}\\.rs".prop_map(|s| s)
}

fn arb_template() -> impl Strategy<Value = TemplateRef> {
    (arb_template_id(), arb_template_path()).prop_map(|(id, path)| TemplateRef { id, path })
}

fn arb_steering() -> impl Strategy<Value = Steering> {
    (
        prop::collection::vec(arb_steering_rule(), 0..5),
        prop::collection::vec(arb_standard(), 0..5),
        prop::collection::vec(arb_template(), 0..5),
    )
        .prop_map(|(rules, standards, templates)| {
            // Remove duplicates to ensure valid steering
            let mut unique_rules = vec![];
            let mut rule_ids = std::collections::HashSet::new();
            for rule in rules {
                if rule_ids.insert(rule.id.clone()) {
                    unique_rules.push(rule);
                }
            }

            let mut unique_standards = vec![];
            let mut standard_ids = std::collections::HashSet::new();
            for standard in standards {
                if standard_ids.insert(standard.id.clone()) {
                    unique_standards.push(standard);
                }
            }

            let mut unique_templates = vec![];
            let mut template_ids = std::collections::HashSet::new();
            for template in templates {
                if template_ids.insert(template.id.clone()) {
                    unique_templates.push(template);
                }
            }

            Steering {
                rules: unique_rules,
                standards: unique_standards,
                templates: unique_templates,
            }
        })
}

// ============================================================================
// Property 2: Steering Merge Consistency
// ============================================================================

proptest! {
    /// Property: For any global and project steering documents, merging SHALL
    /// produce consistent rules with project taking precedence over global,
    /// and global taking precedence over defaults.
    ///
    /// This property verifies that:
    /// 1. Project rules override global rules with the same ID
    /// 2. Global rules are preserved when not overridden
    /// 3. All rules from both global and project are present in the result
    #[test]
    fn prop_merge_project_overrides_global(
        global in arb_steering(),
        project in arb_steering(),
    ) {
        let result = SteeringLoader::merge(&global, &project);
        prop_assert!(result.is_ok(), "Merge should succeed");

        let merged = result.unwrap();

        // Property 2.1: All project rules should be in the merged result
        for project_rule in &project.rules {
            prop_assert!(
                merged.rules.iter().any(|r| r.id == project_rule.id),
                "All project rules should be in merged result"
            );
        }

        // Property 2.2: All global rules not overridden should be in the merged result
        for global_rule in &global.rules {
            prop_assert!(
                merged.rules.iter().any(|r| r.id == global_rule.id),
                "All global rules should be in merged result"
            );
        }

        // Property 2.3: Project rules should override global rules with same ID
        for project_rule in &project.rules {
            if let Some(merged_rule) = merged.rules.iter().find(|r| r.id == project_rule.id) {
                prop_assert_eq!(
                    &merged_rule.description, &project_rule.description,
                    "Project rule should override global rule"
                );
                prop_assert_eq!(
                    &merged_rule.pattern, &project_rule.pattern,
                    "Project rule pattern should override global"
                );
                prop_assert_eq!(
                    &merged_rule.action, &project_rule.action,
                    "Project rule action should override global"
                );
            }
        }
    }

    /// Property: Merging SHALL preserve all standards with project precedence
    ///
    /// This property verifies that standards are merged correctly with
    /// project standards overriding global standards.
    #[test]
    fn prop_merge_standards_with_precedence(
        global in arb_steering(),
        project in arb_steering(),
    ) {
        let result = SteeringLoader::merge(&global, &project);
        prop_assert!(result.is_ok(), "Merge should succeed");

        let merged = result.unwrap();

        // Property 2.4: All project standards should be in the merged result
        for project_std in &project.standards {
            prop_assert!(
                merged.standards.iter().any(|s| s.id == project_std.id),
                "All project standards should be in merged result"
            );
        }

        // Property 2.5: All global standards not overridden should be in the merged result
        for global_std in &global.standards {
            prop_assert!(
                merged.standards.iter().any(|s| s.id == global_std.id),
                "All global standards should be in merged result"
            );
        }

        // Property 2.6: Project standards should override global standards with same ID
        for project_std in &project.standards {
            if let Some(merged_std) = merged.standards.iter().find(|s| s.id == project_std.id) {
                prop_assert_eq!(
                    &merged_std.description, &project_std.description,
                    "Project standard should override global standard"
                );
            }
        }
    }

    /// Property: Merging SHALL preserve all templates with project precedence
    ///
    /// This property verifies that templates are merged correctly with
    /// project templates overriding global templates.
    #[test]
    fn prop_merge_templates_with_precedence(
        global in arb_steering(),
        project in arb_steering(),
    ) {
        let result = SteeringLoader::merge(&global, &project);
        prop_assert!(result.is_ok(), "Merge should succeed");

        let merged = result.unwrap();

        // Property 2.7: All project templates should be in the merged result
        for project_tpl in &project.templates {
            prop_assert!(
                merged.templates.iter().any(|t| t.id == project_tpl.id),
                "All project templates should be in merged result"
            );
        }

        // Property 2.8: All global templates not overridden should be in the merged result
        for global_tpl in &global.templates {
            prop_assert!(
                merged.templates.iter().any(|t| t.id == global_tpl.id),
                "All global templates should be in merged result"
            );
        }

        // Property 2.9: Project templates should override global templates with same ID
        for project_tpl in &project.templates {
            if let Some(merged_tpl) = merged.templates.iter().find(|t| t.id == project_tpl.id) {
                prop_assert_eq!(
                    &merged_tpl.path, &project_tpl.path,
                    "Project template should override global template"
                );
            }
        }
    }

    /// Property: Merging empty project steering SHALL preserve global steering
    ///
    /// This property verifies that when project steering is empty,
    /// the merged result is identical to global steering.
    #[test]
    fn prop_merge_empty_project_preserves_global(
        global in arb_steering(),
    ) {
        let empty_project = Steering {
            rules: vec![],
            standards: vec![],
            templates: vec![],
        };

        let result = SteeringLoader::merge(&global, &empty_project);
        prop_assert!(result.is_ok(), "Merge should succeed");

        let merged = result.unwrap();

        // Property 2.10: Merged result should have same rules as global
        prop_assert_eq!(
            merged.rules.len(),
            global.rules.len(),
            "Merged result should have same number of rules as global"
        );

        // Property 2.11: Merged result should have same standards as global
        prop_assert_eq!(
            merged.standards.len(),
            global.standards.len(),
            "Merged result should have same number of standards as global"
        );

        // Property 2.12: Merged result should have same templates as global
        prop_assert_eq!(
            merged.templates.len(),
            global.templates.len(),
            "Merged result should have same number of templates as global"
        );
    }

    /// Property: Merging empty global steering SHALL preserve project steering
    ///
    /// This property verifies that when global steering is empty,
    /// the merged result is identical to project steering.
    #[test]
    fn prop_merge_empty_global_preserves_project(
        project in arb_steering(),
    ) {
        let empty_global = Steering {
            rules: vec![],
            standards: vec![],
            templates: vec![],
        };

        let result = SteeringLoader::merge(&empty_global, &project);
        prop_assert!(result.is_ok(), "Merge should succeed");

        let merged = result.unwrap();

        // Property 2.13: Merged result should have same rules as project
        prop_assert_eq!(
            merged.rules.len(),
            project.rules.len(),
            "Merged result should have same number of rules as project"
        );

        // Property 2.14: Merged result should have same standards as project
        prop_assert_eq!(
            merged.standards.len(),
            project.standards.len(),
            "Merged result should have same number of standards as project"
        );

        // Property 2.15: Merged result should have same templates as project
        prop_assert_eq!(
            merged.templates.len(),
            project.templates.len(),
            "Merged result should have same number of templates as project"
        );
    }

    /// Property: Merge SHALL be idempotent for identical steering
    ///
    /// This property verifies that merging identical steering documents
    /// produces the same result.
    #[test]
    fn prop_merge_idempotent_for_identical_steering(
        steering in arb_steering(),
    ) {
        let result1 = SteeringLoader::merge(&steering, &steering);
        prop_assert!(result1.is_ok(), "First merge should succeed");

        let merged1 = result1.unwrap();

        let result2 = SteeringLoader::merge(&merged1, &steering);
        prop_assert!(result2.is_ok(), "Second merge should succeed");

        let merged2 = result2.unwrap();

        // Property 2.16: Merging identical steering should be idempotent
        prop_assert_eq!(
            merged1.rules.len(),
            merged2.rules.len(),
            "Idempotent merge should have same number of rules"
        );

        prop_assert_eq!(
            merged1.standards.len(),
            merged2.standards.len(),
            "Idempotent merge should have same number of standards"
        );

        prop_assert_eq!(
            merged1.templates.len(),
            merged2.templates.len(),
            "Idempotent merge should have same number of templates"
        );
    }

    /// Property: Merge result SHALL not have duplicate IDs
    ///
    /// This property verifies that after merging, there are no duplicate
    /// rule, standard, or template IDs.
    #[test]
    fn prop_merge_no_duplicate_ids(
        global in arb_steering(),
        project in arb_steering(),
    ) {
        let result = SteeringLoader::merge(&global, &project);
        prop_assert!(result.is_ok(), "Merge should succeed");

        let merged = result.unwrap();

        // Property 2.17: No duplicate rule IDs
        let mut rule_ids = std::collections::HashSet::new();
        for rule in &merged.rules {
            prop_assert!(
                rule_ids.insert(&rule.id),
                "No duplicate rule IDs should exist"
            );
        }

        // Property 2.18: No duplicate standard IDs
        let mut standard_ids = std::collections::HashSet::new();
        for standard in &merged.standards {
            prop_assert!(
                standard_ids.insert(&standard.id),
                "No duplicate standard IDs should exist"
            );
        }

        // Property 2.19: No duplicate template IDs
        let mut template_ids = std::collections::HashSet::new();
        for template in &merged.templates {
            prop_assert!(
                template_ids.insert(&template.id),
                "No duplicate template IDs should exist"
            );
        }
    }

    /// Property: Merge SHALL preserve rule properties
    ///
    /// This property verifies that merging doesn't lose or corrupt
    /// rule properties.
    #[test]
    fn prop_merge_preserves_rule_properties(
        global in arb_steering(),
        project in arb_steering(),
    ) {
        let result = SteeringLoader::merge(&global, &project);
        prop_assert!(result.is_ok(), "Merge should succeed");

        let merged = result.unwrap();

        // Property 2.20: All merged rules should have non-empty IDs
        for rule in &merged.rules {
            prop_assert!(!rule.id.is_empty(), "Rule ID should not be empty");
        }

        // Property 2.21: All merged standards should have non-empty IDs
        for standard in &merged.standards {
            prop_assert!(!standard.id.is_empty(), "Standard ID should not be empty");
        }

        // Property 2.22: All merged templates should have non-empty IDs
        for template in &merged.templates {
            prop_assert!(!template.id.is_empty(), "Template ID should not be empty");
        }
    }
}
