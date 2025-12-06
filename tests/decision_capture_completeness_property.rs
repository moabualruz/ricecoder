/// Property-based tests for decision capture completeness
/// **Feature: ricecoder-learning, Property 1: Decision Capture Completeness**
/// **Validates: Requirements 1.1**
///
/// Property: For any user decision made during code generation, the Learning System
/// SHALL capture it with complete metadata (timestamp, context, decision type, input, output).

use proptest::prelude::*;
use ricecoder_learning::{Decision, DecisionContext, DecisionLogger};
use std::path::PathBuf;

/// Strategy for generating valid project paths
fn project_path_strategy() -> impl Strategy<Value = PathBuf> {
    r"/project[0-9]{1,3}" // e.g., /project1, /project42, /project999
        .prop_map(PathBuf::from)
}

/// Strategy for generating valid file paths
fn file_path_strategy() -> impl Strategy<Value = PathBuf> {
    r"/project[0-9]{1,3}/src/[a-z_]{1,20}\.rs" // e.g., /project1/src/main.rs
        .prop_map(PathBuf::from)
}

/// Strategy for generating valid decision types
fn decision_type_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("code_generation".to_string()),
        Just("refactoring".to_string()),
        Just("analysis".to_string()),
        Just("completion".to_string()),
        r"[a-z_]{5,20}".prop_map(|s| s.to_string()),
    ]
}

/// Strategy for generating valid agent types
fn agent_type_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("code_generator".to_string()),
        Just("refactorer".to_string()),
        Just("analyzer".to_string()),
        Just("completer".to_string()),
        r"[a-z_]{5,20}".prop_map(|s| s.to_string()),
    ]
}

/// Strategy for generating valid line numbers
fn line_number_strategy() -> impl Strategy<Value = u32> {
    1u32..10000u32
}

/// Strategy for generating JSON values
fn json_value_strategy() -> impl Strategy<Value = serde_json::Value> {
    prop_oneof![
        Just(serde_json::json!({})),
        Just(serde_json::json!({"key": "value"})),
        Just(serde_json::json!({"nested": {"key": "value"}})),
        Just(serde_json::json!([])),
        Just(serde_json::json!([1, 2, 3])),
    ]
}

/// Strategy for generating complete decisions
fn decision_strategy() -> impl Strategy<Value = Decision> {
    (
        project_path_strategy(),
        file_path_strategy(),
        line_number_strategy(),
        agent_type_strategy(),
        decision_type_strategy(),
        json_value_strategy(),
        json_value_strategy(),
    )
        .prop_map(
            |(project_path, file_path, line_number, agent_type, decision_type, input, output)| {
                let context = DecisionContext {
                    project_path,
                    file_path,
                    line_number,
                    agent_type,
                };

                Decision::new(context, decision_type, input, output)
            },
        )
}

proptest! {
    /// Property 1: Decision Capture Completeness
    /// For any decision, when captured, all metadata fields should be preserved
    #[test]
    fn prop_decision_capture_preserves_all_metadata(decision in decision_strategy()) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let logger = DecisionLogger::new();

            // Capture the decision
            let decision_id = decision.id.clone();
            let decision_type = decision.decision_type.clone();
            let context_project = decision.context.project_path.clone();
            let context_file = decision.context.file_path.clone();
            let context_line = decision.context.line_number;
            let context_agent = decision.context.agent_type.clone();
            let input = decision.input.clone();
            let output = decision.output.clone();

            let result = logger.log_decision(decision).await;

            // Verify capture succeeded
            prop_assert!(result.is_ok());
            prop_assert_eq!(result.unwrap(), decision_id);

            // Retrieve the decision
            let retrieved = logger.get_decision(&decision_id).await;
            prop_assert!(retrieved.is_ok());

            let retrieved_decision = retrieved.unwrap();

            // Verify all metadata is preserved
            prop_assert_eq!(retrieved_decision.id, decision_id);
            prop_assert_eq!(retrieved_decision.decision_type, decision_type);
            prop_assert_eq!(retrieved_decision.context.project_path, context_project);
            prop_assert_eq!(retrieved_decision.context.file_path, context_file);
            prop_assert_eq!(retrieved_decision.context.line_number, context_line);
            prop_assert_eq!(retrieved_decision.context.agent_type, context_agent);
            prop_assert_eq!(retrieved_decision.input, input);
            prop_assert_eq!(retrieved_decision.output, output);

            // Verify timestamp is set
            prop_assert!(retrieved_decision.timestamp.timestamp() > 0);
        });
    }

    /// Property 1: Decision Capture Completeness (History)
    /// For any sequence of decisions, all should be captured in history
    #[test]
    fn prop_all_decisions_appear_in_history(decisions in prop::collection::vec(decision_strategy(), 1..100)) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let logger = DecisionLogger::new();

            let decision_ids: Vec<String> = decisions.iter().map(|d| d.id.clone()).collect();

            // Capture all decisions
            for decision in decisions {
                let result = logger.log_decision(decision).await;
                prop_assert!(result.is_ok());
            }

            // Verify all decisions appear in history
            let history = logger.get_history().await;
            prop_assert_eq!(history.len(), decision_ids.len());

            for (i, decision_id) in decision_ids.iter().enumerate() {
                prop_assert_eq!(history[i].id, *decision_id);
            }
        });
    }

    /// Property 1: Decision Capture Completeness (By Type)
    /// For any sequence of decisions with different types, filtering by type should return all matching decisions
    #[test]
    fn prop_decisions_filterable_by_type(decisions in prop::collection::vec(decision_strategy(), 1..100)) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let logger = DecisionLogger::new();

            // Capture all decisions
            for decision in &decisions {
                let result = logger.log_decision(decision.clone()).await;
                prop_assert!(result.is_ok());
            }

            // For each unique decision type, verify filtering works
            let mut type_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
            for decision in &decisions {
                *type_counts.entry(decision.decision_type.clone()).or_insert(0) += 1;
            }

            for (decision_type, expected_count) in type_counts {
                let filtered = logger.get_history_by_type(&decision_type).await;
                prop_assert_eq!(filtered.len(), expected_count);

                // Verify all filtered decisions have the correct type
                for decision in filtered {
                    prop_assert_eq!(decision.decision_type, decision_type);
                }
            }
        });
    }

    /// Property 1: Decision Capture Completeness (By Context)
    /// For any sequence of decisions with different contexts, filtering by context should return all matching decisions
    #[test]
    fn prop_decisions_filterable_by_context(decisions in prop::collection::vec(decision_strategy(), 1..100)) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let logger = DecisionLogger::new();

            // Capture all decisions
            for decision in &decisions {
                let result = logger.log_decision(decision.clone()).await;
                prop_assert!(result.is_ok());
            }

            // For each unique context, verify filtering works
            let mut context_map: std::collections::HashMap<String, Vec<Decision>> = std::collections::HashMap::new();
            for decision in &decisions {
                let context_key = format!(
                    "{}:{}",
                    decision.context.project_path.display(),
                    decision.context.file_path.display()
                );
                context_map.entry(context_key).or_insert_with(Vec::new).push(decision.clone());
            }

            for (_, context_decisions) in context_map {
                if !context_decisions.is_empty() {
                    let context = &context_decisions[0].context;
                    let filtered = logger.get_history_by_context(context).await;
                    prop_assert_eq!(filtered.len(), context_decisions.len());

                    // Verify all filtered decisions have the correct context
                    for decision in filtered {
                        prop_assert_eq!(decision.context.project_path, context.project_path);
                        prop_assert_eq!(decision.context.file_path, context.file_path);
                    }
                }
            }
        });
    }

    /// Property 1: Decision Capture Completeness (Replay)
    /// For any sequence of decisions, replaying should return them in the same order
    #[test]
    fn prop_replay_preserves_decision_order(decisions in prop::collection::vec(decision_strategy(), 1..100)) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let logger = DecisionLogger::new();

            let decision_ids: Vec<String> = decisions.iter().map(|d| d.id.clone()).collect();

            // Capture all decisions
            for decision in decisions {
                let result = logger.log_decision(decision).await;
                prop_assert!(result.is_ok());
            }

            // Verify replay returns decisions in the same order
            let replayed = logger.replay_decisions().await;
            prop_assert_eq!(replayed.len(), decision_ids.len());

            for (i, decision_id) in decision_ids.iter().enumerate() {
                prop_assert_eq!(replayed[i].id, *decision_id);
            }
        });
    }

    /// Property 1: Decision Capture Completeness (Count)
    /// For any sequence of decisions, the count should match the number captured
    #[test]
    fn prop_decision_count_matches_captured(decisions in prop::collection::vec(decision_strategy(), 1..100)) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let logger = DecisionLogger::new();

            prop_assert_eq!(logger.decision_count().await, 0);

            // Capture all decisions
            for (i, decision) in decisions.iter().enumerate() {
                let result = logger.log_decision(decision.clone()).await;
                prop_assert!(result.is_ok());
                prop_assert_eq!(logger.decision_count().await, i + 1);
            }

            prop_assert_eq!(logger.decision_count().await, decisions.len());
        });
    }

    /// Property 1: Decision Capture Completeness (Statistics)
    /// For any sequence of decisions, statistics should accurately reflect captured decisions
    #[test]
    fn prop_statistics_accurately_reflect_decisions(decisions in prop::collection::vec(decision_strategy(), 1..100)) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let logger = DecisionLogger::new();

            // Capture all decisions
            for decision in &decisions {
                let result = logger.log_decision(decision.clone()).await;
                prop_assert!(result.is_ok());
            }

            // Get statistics
            let stats = logger.get_statistics().await;

            // Verify total count
            prop_assert_eq!(stats.total_decisions, decisions.len());

            // Verify decision type counts
            let mut expected_type_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
            for decision in &decisions {
                *expected_type_counts.entry(decision.decision_type.clone()).or_insert(0) += 1;
            }

            for (decision_type, expected_count) in expected_type_counts {
                prop_assert_eq!(stats.decision_types.get(&decision_type), Some(&expected_count));
            }

            // Verify agent type counts
            let mut expected_agent_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
            for decision in &decisions {
                *expected_agent_counts.entry(decision.context.agent_type.clone()).or_insert(0) += 1;
            }

            for (agent_type, expected_count) in expected_agent_counts {
                prop_assert_eq!(stats.agent_types.get(&agent_type), Some(&expected_count));
            }
        });
    }
}
