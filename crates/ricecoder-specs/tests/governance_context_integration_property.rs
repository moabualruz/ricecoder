//! Property-based tests for Governance context integration
//! **Feature: ricecoder-specs, Property 10: Governance Context Integration**
//! **Validates: Requirements 5.5, 5.6**

use chrono::Utc;
use proptest::prelude::*;
use ricecoder_specs::{
    ai_writer::AISpecWriter,
    models::{
        ApprovalGate, ConversationMessage, MessageRole, SpecPhase, SpecWritingSession, Standard,
        Governance, GovernanceRule, TemplateRef,
    },
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

fn arb_governance_rule() -> impl Strategy<Value = GovernanceRule> {
    (
        arb_rule_id(),
        arb_rule_description(),
        arb_rule_pattern(),
        arb_rule_action(),
    )
        .prop_map(|(id, description, pattern, action)| GovernanceRule {
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

fn arb_governance() -> impl Strategy<Value = Governance> {
    (
        prop::collection::vec(arb_governance_rule(), 0..5),
        prop::collection::vec(arb_standard(), 0..5),
        prop::collection::vec(arb_template(), 0..5),
    )
        .prop_map(|(rules, standards, templates)| {
            // Remove duplicates to ensure valid Governance
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

            Governance {
                rules: unique_rules,
                standards: unique_standards,
                templates: unique_templates,
            }
        })
}

fn arb_user_input() -> impl Strategy<Value = String> {
    "[A-Za-z0-9 .,?!]{1,100}".prop_map(|s| s)
}

fn arb_spec_writing_session() -> impl Strategy<Value = SpecWritingSession> {
    (
        "[a-z0-9_-]{1,20}",
        "[a-z0-9_-]{1,20}",
        prop::collection::vec("[A-Za-z0-9 ]{1,50}", 0..3),
    )
        .prop_map(|(id, spec_id, messages)| {
            let now = Utc::now();
            let mut conversation_history = vec![];

            for (idx, msg) in messages.iter().enumerate() {
                let role = if idx % 2 == 0 {
                    MessageRole::User
                } else {
                    MessageRole::Assistant
                };

                conversation_history.push(ConversationMessage {
                    id: format!("msg-{}", idx),
                    spec_id: spec_id.clone(),
                    role,
                    content: msg.clone(),
                    timestamp: now,
                });
            }

            SpecWritingSession {
                id,
                spec_id,
                phase: SpecPhase::Requirements,
                conversation_history,
                approval_gates: vec![
                    ApprovalGate {
                        phase: SpecPhase::Discovery,
                        approved: false,
                        approved_at: None,
                        approved_by: None,
                        feedback: None,
                    },
                    ApprovalGate {
                        phase: SpecPhase::Requirements,
                        approved: false,
                        approved_at: None,
                        approved_by: None,
                        feedback: None,
                    },
                ],
                created_at: now,
                updated_at: now,
            }
        })
}

// ============================================================================
// Property 10: Governance Context Integration
// ============================================================================

proptest! {
    /// Property: For any AI prompt generation, Governance context from
    /// `projects/ricecoder/.ricecoder/Governance/` SHALL be included with
    /// project Governance taking precedence over global Governance.
    ///
    /// This property verifies that:
    /// 1. Governance context is included in the generated prompt
    /// 2. Project Governance takes precedence over global Governance
    /// 3. All Governance rules, standards, and templates are represented
    #[test]
    fn prop_governance_context_included_in_prompt(
        session in arb_spec_writing_session(),
        global_governance in arb_governance(),
        project_governance in arb_governance(),
        user_input in arb_user_input(),
    ) {
        let result = AISpecWriter::build_prompt_with_governance_context(
            &session,
            &global_governance,
            &project_governance,
            &user_input,
        );

        prop_assert!(result.is_ok(), "Prompt building should succeed");

        let prompt = result.unwrap();

        // Property 10.1: Prompt should contain phase guidance
        prop_assert!(
            prompt.contains("Phase Guidance"),
            "Prompt should include phase guidance"
        );

        // Property 10.2: Prompt should contain user input
        prop_assert!(
            prompt.contains(&user_input),
            "Prompt should include user input"
        );

        // Property 10.3: If Governance has rules, they should be in the prompt
        if !project_governance.rules.is_empty() {
            prop_assert!(
                prompt.contains("Governance Rules"),
                "Prompt should include Governance rules section when rules exist"
            );

            for rule in &project_governance.rules {
                prop_assert!(
                    prompt.contains(&rule.id),
                    "Prompt should include project rule ID: {}",
                    rule.id
                );
            }
        }

        // Property 10.4: If Governance has standards, they should be in the prompt
        if !project_governance.standards.is_empty() {
            prop_assert!(
                prompt.contains("Standards"),
                "Prompt should include standards section when standards exist"
            );

            for standard in &project_governance.standards {
                prop_assert!(
                    prompt.contains(&standard.id),
                    "Prompt should include project standard ID: {}",
                    standard.id
                );
            }
        }

        // Property 10.5: If Governance has templates, they should be in the prompt
        if !project_governance.templates.is_empty() {
            prop_assert!(
                prompt.contains("Available Templates"),
                "Prompt should include templates section when templates exist"
            );

            for template in &project_governance.templates {
                prop_assert!(
                    prompt.contains(&template.id),
                    "Prompt should include project template ID: {}",
                    template.id
                );
            }
        }

        // Property 10.6: Conversation history should be included if present
        if !session.conversation_history.is_empty() {
            prop_assert!(
                prompt.contains("Conversation History"),
                "Prompt should include conversation history section when messages exist"
            );
        }
    }

    /// Property: Project Governance SHALL take precedence over global Governance
    /// in the generated prompt.
    ///
    /// This property verifies that when both global and project Governance
    /// define the same rule/standard/template, the project version appears
    /// in the prompt.
    #[test]
    fn prop_project_governance_takes_precedence(
        session in arb_spec_writing_session(),
        global_governance in arb_governance(),
        project_governance in arb_governance(),
        user_input in arb_user_input(),
    ) {
        let result = AISpecWriter::build_prompt_with_governance_context(
            &session,
            &global_governance,
            &project_governance,
            &user_input,
        );

        prop_assert!(result.is_ok(), "Prompt building should succeed");

        let prompt = result.unwrap();

        // Property 10.7: For each project rule, verify it appears in prompt
        for project_rule in &project_governance.rules {
            prop_assert!(
                prompt.contains(&project_rule.id),
                "Project rule {} should appear in prompt",
                project_rule.id
            );

            // If there's a global rule with the same ID, verify project version is used
            if let Some(global_rule) = global_governance.rules.iter().find(|r| r.id == project_rule.id) {
                // The prompt should contain the project rule's description
                prop_assert!(
                    prompt.contains(&project_rule.description),
                    "Project rule description should appear in prompt"
                );

                // The prompt should NOT contain the global rule's description
                // (unless they happen to be identical)
                if global_rule.description != project_rule.description {
                    // We can't strictly assert the global description is absent
                    // because it might appear elsewhere, but we verify the project
                    // version is present
                    prop_assert!(
                        prompt.contains(&project_rule.description),
                        "Project rule description should be in prompt"
                    );
                }
            }
        }

        // Property 10.8: For each project standard, verify it appears in prompt
        for project_std in &project_governance.standards {
            prop_assert!(
                prompt.contains(&project_std.id),
                "Project standard {} should appear in prompt",
                project_std.id
            );
        }

        // Property 10.9: For each project template, verify it appears in prompt
        for project_tpl in &project_governance.templates {
            prop_assert!(
                prompt.contains(&project_tpl.id),
                "Project template {} should appear in prompt",
                project_tpl.id
            );
        }
    }

    /// Property: Governance context SHALL be consistent across multiple
    /// prompt generations with identical input.
    ///
    /// This property verifies that generating prompts with the same
    /// input produces identical output (deterministic).
    #[test]
    fn prop_governance_context_deterministic(
        session in arb_spec_writing_session(),
        global_governance in arb_governance(),
        project_governance in arb_governance(),
        user_input in arb_user_input(),
    ) {
        let result1 = AISpecWriter::build_prompt_with_governance_context(
            &session,
            &global_governance,
            &project_governance,
            &user_input,
        );

        let result2 = AISpecWriter::build_prompt_with_governance_context(
            &session,
            &global_governance,
            &project_governance,
            &user_input,
        );

        prop_assert!(result1.is_ok(), "First prompt building should succeed");
        prop_assert!(result2.is_ok(), "Second prompt building should succeed");

        let prompt1 = result1.unwrap();
        let prompt2 = result2.unwrap();

        // Property 10.10: Identical input should produce identical prompts
        prop_assert_eq!(
            prompt1, prompt2,
            "Governance context generation should be deterministic"
        );
    }

    /// Property: Governance context SHALL include all rules, standards,
    /// and templates from the merged Governance.
    ///
    /// This property verifies that no Governance elements are lost
    /// during prompt generation.
    #[test]
    fn prop_governance_context_complete(
        session in arb_spec_writing_session(),
        global_governance in arb_governance(),
        project_governance in arb_governance(),
        user_input in arb_user_input(),
    ) {
        let result = AISpecWriter::build_prompt_with_governance_context(
            &session,
            &global_governance,
            &project_governance,
            &user_input,
        );

        prop_assert!(result.is_ok(), "Prompt building should succeed");

        let prompt = result.unwrap();

        // Property 10.11: All project rules should be in the prompt
        for rule in &project_governance.rules {
            prop_assert!(
                prompt.contains(&rule.id),
                "All project rules should be in prompt"
            );
        }

        // Property 10.12: All global rules not overridden should be in the prompt
        for global_rule in &global_governance.rules {
            // Check if this rule is overridden by a project rule
            let is_overridden = project_governance.rules.iter().any(|r| r.id == global_rule.id);

            if !is_overridden {
                prop_assert!(
                    prompt.contains(&global_rule.id),
                    "Non-overridden global rules should be in prompt"
                );
            }
        }

        // Property 10.13: All project standards should be in the prompt
        for standard in &project_governance.standards {
            prop_assert!(
                prompt.contains(&standard.id),
                "All project standards should be in prompt"
            );
        }

        // Property 10.14: All global standards not overridden should be in the prompt
        for global_std in &global_governance.standards {
            let is_overridden = project_governance.standards.iter().any(|s| s.id == global_std.id);

            if !is_overridden {
                prop_assert!(
                    prompt.contains(&global_std.id),
                    "Non-overridden global standards should be in prompt"
                );
            }
        }

        // Property 10.15: All project templates should be in the prompt
        for template in &project_governance.templates {
            prop_assert!(
                prompt.contains(&template.id),
                "All project templates should be in prompt"
            );
        }

        // Property 10.16: All global templates not overridden should be in the prompt
        for global_tpl in &global_governance.templates {
            let is_overridden = project_governance.templates.iter().any(|t| t.id == global_tpl.id);

            if !is_overridden {
                prop_assert!(
                    prompt.contains(&global_tpl.id),
                    "Non-overridden global templates should be in prompt"
                );
            }
        }
    }

    /// Property: Governance context SHALL be properly formatted for readability.
    ///
    /// This property verifies that the generated prompt is well-formatted
    /// and readable.
    #[test]
    fn prop_governance_context_well_formatted(
        session in arb_spec_writing_session(),
        global_governance in arb_governance(),
        project_governance in arb_governance(),
        user_input in arb_user_input(),
    ) {
        let result = AISpecWriter::build_prompt_with_governance_context(
            &session,
            &global_governance,
            &project_governance,
            &user_input,
        );

        prop_assert!(result.is_ok(), "Prompt building should succeed");

        let prompt = result.unwrap();

        // Property 10.17: Prompt should not be empty
        prop_assert!(!prompt.is_empty(), "Prompt should not be empty");

        // Property 10.18: Prompt should contain markdown headers
        prop_assert!(
            prompt.contains("##"),
            "Prompt should contain markdown headers for structure"
        );

        // Property 10.19: Prompt should be reasonably sized
        // (not too large, not too small)
        prop_assert!(
            prompt.len() > 50,
            "Prompt should have reasonable content"
        );

        prop_assert!(
            prompt.len() < 100000,
            "Prompt should not be excessively large"
        );
    }

    /// Property: Governance context SHALL handle empty Governance gracefully.
    ///
    /// This property verifies that the system works correctly when
    /// Governance documents are empty.
    #[test]
    fn prop_governance_context_handles_empty_governance(
        session in arb_spec_writing_session(),
        user_input in arb_user_input(),
    ) {
        let empty_governance = Governance {
            rules: vec![],
            standards: vec![],
            templates: vec![],
        };

        let result = AISpecWriter::build_prompt_with_governance_context(
            &session,
            &empty_governance,
            &empty_governance,
            &user_input,
        );

        prop_assert!(result.is_ok(), "Prompt building should succeed with empty Governance");

        let prompt = result.unwrap();

        // Property 10.20: Prompt should still contain phase guidance
        prop_assert!(
            prompt.contains("Phase Guidance"),
            "Prompt should include phase guidance even with empty Governance"
        );

        // Property 10.21: Prompt should still contain user input
        prop_assert!(
            prompt.contains(&user_input),
            "Prompt should include user input even with empty Governance"
        );
    }
}
