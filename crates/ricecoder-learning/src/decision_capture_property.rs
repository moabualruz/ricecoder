/// Property-based tests for decision capture completeness
/// **Feature: ricecoder-learning, Property 1: Decision Capture Completeness**
/// **Validates: Requirements 1.1**
///
/// Property: For any user decision made during code generation, the Learning System
/// SHALL capture it with complete metadata (timestamp, context, decision type, input, output).

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{Decision, DecisionContext, DecisionLogger};

    /// Property 1: Decision Capture Completeness
    /// For any decision, when captured, all metadata fields should be preserved
    #[tokio::test]
    async fn prop_decision_capture_preserves_all_metadata() {
        let logger = DecisionLogger::new();

        let context = DecisionContext {
            project_path: PathBuf::from("/project1"),
            file_path: PathBuf::from("/project1/src/main.rs"),
            line_number: 42,
            agent_type: "code_generator".to_string(),
        };

        let decision = Decision::new(
            context.clone(),
            "code_generation".to_string(),
            serde_json::json!({"input": "test"}),
            serde_json::json!({"output": "result"}),
        );

        let decision_id = decision.id.clone();
        let decision_type = decision.decision_type.clone();

        // Capture the decision
        let result = logger.log_decision(decision).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), decision_id);

        // Retrieve the decision
        let retrieved = logger.get_decision(&decision_id).await;
        assert!(retrieved.is_ok());

        let retrieved_decision = retrieved.unwrap();

        // Verify all metadata is preserved
        assert_eq!(retrieved_decision.id, decision_id);
        assert_eq!(retrieved_decision.decision_type, decision_type);
        assert_eq!(
            retrieved_decision.context.project_path,
            context.project_path
        );
        assert_eq!(retrieved_decision.context.file_path, context.file_path);
        assert_eq!(retrieved_decision.context.line_number, context.line_number);
        assert_eq!(retrieved_decision.context.agent_type, context.agent_type);
        assert!(retrieved_decision.timestamp.timestamp() > 0);
    }

    /// Property 1: Decision Capture Completeness (History)
    /// For any sequence of decisions, all should be captured in history
    #[tokio::test]
    async fn prop_all_decisions_appear_in_history() {
        let logger = DecisionLogger::new();

        let mut decision_ids = Vec::new();
        for i in 0..50 {
            let context = DecisionContext {
                project_path: PathBuf::from(format!("/project{}", i % 5)),
                file_path: PathBuf::from(format!("/project{}/src/file{}.rs", i % 5, i)),
                line_number: (i * 10) as u32,
                agent_type: format!("agent_{}", i % 3),
            };

            let decision = Decision::new(
                context,
                format!("type_{}", i % 4),
                serde_json::json!({"index": i}),
                serde_json::json!({"result": i * 2}),
            );

            decision_ids.push(decision.id.clone());
            let result = logger.log_decision(decision).await;
            assert!(result.is_ok());
        }

        // Verify all decisions appear in history
        let history = logger.get_history().await;
        assert_eq!(history.len(), decision_ids.len());

        for (i, decision_id) in decision_ids.iter().enumerate() {
            assert_eq!(history[i].id, *decision_id);
        }
    }

    /// Property 1: Decision Capture Completeness (By Type)
    /// For any sequence of decisions with different types, filtering by type should return all matching decisions
    #[tokio::test]
    async fn prop_decisions_filterable_by_type() {
        let logger = DecisionLogger::new();

        let mut type_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        for i in 0..50 {
            let context = DecisionContext {
                project_path: PathBuf::from("/project"),
                file_path: PathBuf::from("/project/src/main.rs"),
                line_number: i as u32,
                agent_type: "agent".to_string(),
            };

            let decision_type = format!("type_{}", i % 4);
            *type_counts.entry(decision_type.clone()).or_insert(0) += 1;

            let decision = Decision::new(
                context,
                decision_type,
                serde_json::json!({}),
                serde_json::json!({}),
            );

            let result = logger.log_decision(decision).await;
            assert!(result.is_ok());
        }

        // Verify filtering works for each type
        for (decision_type, expected_count) in type_counts {
            let filtered = logger.get_history_by_type(&decision_type).await;
            assert_eq!(filtered.len(), expected_count);

            // Verify all filtered decisions have the correct type
            for decision in filtered {
                assert_eq!(decision.decision_type, decision_type);
            }
        }
    }

    /// Property 1: Decision Capture Completeness (By Context)
    /// For any sequence of decisions with different contexts, filtering by context should return all matching decisions
    #[tokio::test]
    async fn prop_decisions_filterable_by_context() {
        let logger = DecisionLogger::new();

        let context1 = DecisionContext {
            project_path: PathBuf::from("/project1"),
            file_path: PathBuf::from("/project1/src/main.rs"),
            line_number: 10,
            agent_type: "agent1".to_string(),
        };

        let context2 = DecisionContext {
            project_path: PathBuf::from("/project2"),
            file_path: PathBuf::from("/project2/src/lib.rs"),
            line_number: 20,
            agent_type: "agent2".to_string(),
        };

        // Capture decisions for context1
        for i in 0..25 {
            let decision = Decision::new(
                context1.clone(),
                format!("type_{}", i % 3),
                serde_json::json!({}),
                serde_json::json!({}),
            );
            let result = logger.log_decision(decision).await;
            assert!(result.is_ok());
        }

        // Capture decisions for context2
        for i in 0..25 {
            let decision = Decision::new(
                context2.clone(),
                format!("type_{}", i % 3),
                serde_json::json!({}),
                serde_json::json!({}),
            );
            let result = logger.log_decision(decision).await;
            assert!(result.is_ok());
        }

        // Verify filtering works for each context
        let context1_decisions = logger.get_history_by_context(&context1).await;
        assert_eq!(context1_decisions.len(), 25);
        for decision in context1_decisions {
            assert_eq!(decision.context.project_path, context1.project_path);
            assert_eq!(decision.context.file_path, context1.file_path);
        }

        let context2_decisions = logger.get_history_by_context(&context2).await;
        assert_eq!(context2_decisions.len(), 25);
        for decision in context2_decisions {
            assert_eq!(decision.context.project_path, context2.project_path);
            assert_eq!(decision.context.file_path, context2.file_path);
        }
    }

    /// Property 1: Decision Capture Completeness (Replay)
    /// For any sequence of decisions, replaying should return them in the same order
    #[tokio::test]
    async fn prop_replay_preserves_decision_order() {
        let logger = DecisionLogger::new();

        let mut decision_ids = Vec::new();
        for i in 0..50 {
            let context = DecisionContext {
                project_path: PathBuf::from("/project"),
                file_path: PathBuf::from("/project/src/main.rs"),
                line_number: i as u32,
                agent_type: "agent".to_string(),
            };

            let decision = Decision::new(
                context,
                format!("type_{}", i % 4),
                serde_json::json!({}),
                serde_json::json!({}),
            );

            decision_ids.push(decision.id.clone());
            let result = logger.log_decision(decision).await;
            assert!(result.is_ok());
        }

        // Verify replay returns decisions in the same order
        let replayed = logger.replay_decisions().await;
        assert_eq!(replayed.len(), decision_ids.len());

        for (i, decision_id) in decision_ids.iter().enumerate() {
            assert_eq!(replayed[i].id, *decision_id);
        }
    }

    /// Property 1: Decision Capture Completeness (Count)
    /// For any sequence of decisions, the count should match the number captured
    #[tokio::test]
    async fn prop_decision_count_matches_captured() {
        let logger = DecisionLogger::new();

        assert_eq!(logger.decision_count().await, 0);

        for i in 0..50 {
            let context = DecisionContext {
                project_path: PathBuf::from("/project"),
                file_path: PathBuf::from("/project/src/main.rs"),
                line_number: i as u32,
                agent_type: "agent".to_string(),
            };

            let decision = Decision::new(
                context,
                "type".to_string(),
                serde_json::json!({}),
                serde_json::json!({}),
            );

            let result = logger.log_decision(decision).await;
            assert!(result.is_ok());
            assert_eq!(logger.decision_count().await, i + 1);
        }

        assert_eq!(logger.decision_count().await, 50);
    }

    /// Property 1: Decision Capture Completeness (Statistics)
    /// For any sequence of decisions, statistics should accurately reflect captured decisions
    #[tokio::test]
    async fn prop_statistics_accurately_reflect_decisions() {
        let logger = DecisionLogger::new();

        let mut expected_type_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        let mut expected_agent_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        for i in 0..50 {
            let context = DecisionContext {
                project_path: PathBuf::from("/project"),
                file_path: PathBuf::from("/project/src/main.rs"),
                line_number: i as u32,
                agent_type: format!("agent_{}", i % 3),
            };

            let decision_type = format!("type_{}", i % 4);

            *expected_type_counts
                .entry(decision_type.clone())
                .or_insert(0) += 1;
            *expected_agent_counts
                .entry(context.agent_type.clone())
                .or_insert(0) += 1;

            let decision = Decision::new(
                context,
                decision_type,
                serde_json::json!({}),
                serde_json::json!({}),
            );

            let result = logger.log_decision(decision).await;
            assert!(result.is_ok());
        }

        // Get statistics
        let stats = logger.get_statistics().await;

        // Verify total count
        assert_eq!(stats.total_decisions, 50);

        // Verify decision type counts
        for (decision_type, expected_count) in expected_type_counts {
            assert_eq!(
                stats.decision_types.get(&decision_type),
                Some(&expected_count)
            );
        }

        // Verify agent type counts
        for (agent_type, expected_count) in expected_agent_counts {
            assert_eq!(stats.agent_types.get(&agent_type), Some(&expected_count));
        }
    }
}
