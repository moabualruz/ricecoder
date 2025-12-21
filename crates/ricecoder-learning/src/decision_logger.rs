/// Decision logging component for capturing user decisions with full metadata
use crate::error::{LearningError, Result};
use crate::models::{Decision, DecisionContext};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Captures and stores user decisions with full metadata
pub struct DecisionLogger {
    /// In-memory storage for decisions
    decisions: Arc<RwLock<Vec<Decision>>>,
    /// Index for fast lookup by decision ID
    decision_index: Arc<RwLock<HashMap<String, usize>>>,
    /// Index for fast lookup by context (project_path + file_path)
    context_index: Arc<RwLock<HashMap<String, Vec<usize>>>>,
}

impl DecisionLogger {
    /// Create a new decision logger
    pub fn new() -> Self {
        Self {
            decisions: Arc::new(RwLock::new(Vec::new())),
            decision_index: Arc::new(RwLock::new(HashMap::new())),
            context_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Log a decision with timestamp, context, and type
    pub async fn log_decision(&self, decision: Decision) -> Result<String> {
        let decision_id = decision.id.clone();
        let context_key = self.make_context_key(&decision.context);

        let mut decisions = self.decisions.write().await;
        let index = decisions.len();
        decisions.push(decision);

        // Update indices
        let mut decision_index = self.decision_index.write().await;
        decision_index.insert(decision_id.clone(), index);

        let mut context_index = self.context_index.write().await;
        context_index.entry(context_key).or_default().push(index);

        Ok(decision_id)
    }

    /// Get decision history (all captured decisions)
    pub async fn get_history(&self) -> Vec<Decision> {
        self.decisions.read().await.clone()
    }

    /// Get decision history for a specific context
    pub async fn get_history_by_context(&self, context: &DecisionContext) -> Vec<Decision> {
        let context_key = self.make_context_key(context);
        let context_index = self.context_index.read().await;

        if let Some(indices) = context_index.get(&context_key) {
            let decisions = self.decisions.read().await;
            indices
                .iter()
                .filter_map(|&idx| decisions.get(idx).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get decision history filtered by decision type
    pub async fn get_history_by_type(&self, decision_type: &str) -> Vec<Decision> {
        self.decisions
            .read()
            .await
            .iter()
            .filter(|d| d.decision_type == decision_type)
            .cloned()
            .collect()
    }

    /// Get a specific decision by ID
    pub async fn get_decision(&self, decision_id: &str) -> Result<Decision> {
        let decision_index = self.decision_index.read().await;

        if let Some(&idx) = decision_index.get(decision_id) {
            let decisions = self.decisions.read().await;
            decisions
                .get(idx)
                .cloned()
                .ok_or_else(|| LearningError::DecisionNotFound(decision_id.to_string()))
        } else {
            Err(LearningError::DecisionNotFound(decision_id.to_string()))
        }
    }

    /// Replay decisions for validation (returns decisions in order)
    pub async fn replay_decisions(&self) -> Vec<Decision> {
        self.decisions.read().await.clone()
    }

    /// Replay decisions for a specific context
    pub async fn replay_decisions_for_context(&self, context: &DecisionContext) -> Vec<Decision> {
        self.get_history_by_context(context).await
    }

    /// Get the number of captured decisions
    pub async fn decision_count(&self) -> usize {
        self.decisions.read().await.len()
    }

    /// Clear all decisions
    pub async fn clear(&self) {
        self.decisions.write().await.clear();
        self.decision_index.write().await.clear();
        self.context_index.write().await.clear();
    }

    /// Get statistics about captured decisions
    pub async fn get_statistics(&self) -> DecisionStatistics {
        let decisions = self.decisions.read().await;

        let mut type_counts: HashMap<String, usize> = HashMap::new();
        let mut agent_counts: HashMap<String, usize> = HashMap::new();

        for decision in decisions.iter() {
            *type_counts
                .entry(decision.decision_type.clone())
                .or_insert(0) += 1;
            *agent_counts
                .entry(decision.context.agent_type.clone())
                .or_insert(0) += 1;
        }

        DecisionStatistics {
            total_decisions: decisions.len(),
            decision_types: type_counts,
            agent_types: agent_counts,
        }
    }

    /// Helper function to create a context key for indexing
    fn make_context_key(&self, context: &DecisionContext) -> String {
        format!(
            "{}:{}",
            context.project_path.display(),
            context.file_path.display()
        )
    }
}

impl Default for DecisionLogger {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about captured decisions
#[derive(Debug, Clone)]
pub struct DecisionStatistics {
    /// Total number of decisions captured
    pub total_decisions: usize,
    /// Count of decisions by type
    pub decision_types: HashMap<String, usize>,
    /// Count of decisions by agent type
    pub agent_types: HashMap<String, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_decision(
        decision_type: &str,
        agent_type: &str,
        project_path: &str,
        file_path: &str,
    ) -> Decision {
        let context = DecisionContext {
            project_path: PathBuf::from(project_path),
            file_path: PathBuf::from(file_path),
            line_number: 10,
            agent_type: agent_type.to_string(),
        };

        Decision::new(
            context,
            decision_type.to_string(),
            serde_json::json!({"input": "test"}),
            serde_json::json!({"output": "result"}),
        )
    }

    #[tokio::test]
    async fn test_log_decision() {
        let logger = DecisionLogger::new();

        let decision =
            create_test_decision("code_gen", "agent1", "/project", "/project/src/main.rs");
        let decision_id = decision.id.clone();

        let result = logger.log_decision(decision).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), decision_id);

        assert_eq!(logger.decision_count().await, 1);
    }

    #[tokio::test]
    async fn test_get_history() {
        let logger = DecisionLogger::new();

        let decision1 =
            create_test_decision("code_gen", "agent1", "/project", "/project/src/main.rs");
        let decision2 =
            create_test_decision("refactor", "agent2", "/project", "/project/src/lib.rs");

        logger.log_decision(decision1).await.unwrap();
        logger.log_decision(decision2).await.unwrap();

        let history = logger.get_history().await;
        assert_eq!(history.len(), 2);
    }

    #[tokio::test]
    async fn test_get_history_by_type() {
        let logger = DecisionLogger::new();

        let decision1 =
            create_test_decision("code_gen", "agent1", "/project", "/project/src/main.rs");
        let decision2 =
            create_test_decision("refactor", "agent2", "/project", "/project/src/lib.rs");
        let decision3 =
            create_test_decision("code_gen", "agent1", "/project", "/project/src/utils.rs");

        logger.log_decision(decision1).await.unwrap();
        logger.log_decision(decision2).await.unwrap();
        logger.log_decision(decision3).await.unwrap();

        let code_gen_decisions = logger.get_history_by_type("code_gen").await;
        assert_eq!(code_gen_decisions.len(), 2);

        let refactor_decisions = logger.get_history_by_type("refactor").await;
        assert_eq!(refactor_decisions.len(), 1);
    }

    #[tokio::test]
    async fn test_get_history_by_context() {
        let logger = DecisionLogger::new();

        let context1 = DecisionContext {
            project_path: PathBuf::from("/project1"),
            file_path: PathBuf::from("/project1/src/main.rs"),
            line_number: 10,
            agent_type: "agent1".to_string(),
        };

        let context2 = DecisionContext {
            project_path: PathBuf::from("/project2"),
            file_path: PathBuf::from("/project2/src/main.rs"),
            line_number: 20,
            agent_type: "agent2".to_string(),
        };

        let decision1 = Decision::new(
            context1.clone(),
            "code_gen".to_string(),
            serde_json::json!({}),
            serde_json::json!({}),
        );

        let decision2 = Decision::new(
            context2.clone(),
            "refactor".to_string(),
            serde_json::json!({}),
            serde_json::json!({}),
        );

        let decision3 = Decision::new(
            context1.clone(),
            "code_gen".to_string(),
            serde_json::json!({}),
            serde_json::json!({}),
        );

        logger.log_decision(decision1).await.unwrap();
        logger.log_decision(decision2).await.unwrap();
        logger.log_decision(decision3).await.unwrap();

        let context1_decisions = logger.get_history_by_context(&context1).await;
        assert_eq!(context1_decisions.len(), 2);

        let context2_decisions = logger.get_history_by_context(&context2).await;
        assert_eq!(context2_decisions.len(), 1);
    }

    #[tokio::test]
    async fn test_get_decision() {
        let logger = DecisionLogger::new();

        let decision =
            create_test_decision("code_gen", "agent1", "/project", "/project/src/main.rs");
        let decision_id = decision.id.clone();

        logger.log_decision(decision.clone()).await.unwrap();

        let retrieved = logger.get_decision(&decision_id).await;
        assert!(retrieved.is_ok());
        assert_eq!(retrieved.unwrap().id, decision_id);
    }

    #[tokio::test]
    async fn test_get_decision_not_found() {
        let logger = DecisionLogger::new();

        let result = logger.get_decision("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_replay_decisions() {
        let logger = DecisionLogger::new();

        let decision1 =
            create_test_decision("code_gen", "agent1", "/project", "/project/src/main.rs");
        let decision2 =
            create_test_decision("refactor", "agent2", "/project", "/project/src/lib.rs");

        logger.log_decision(decision1).await.unwrap();
        logger.log_decision(decision2).await.unwrap();

        let replayed = logger.replay_decisions().await;
        assert_eq!(replayed.len(), 2);
        assert_eq!(replayed[0].decision_type, "code_gen");
        assert_eq!(replayed[1].decision_type, "refactor");
    }

    #[tokio::test]
    async fn test_replay_decisions_for_context() {
        let logger = DecisionLogger::new();

        let context = DecisionContext {
            project_path: PathBuf::from("/project"),
            file_path: PathBuf::from("/project/src/main.rs"),
            line_number: 10,
            agent_type: "agent1".to_string(),
        };

        let decision1 = Decision::new(
            context.clone(),
            "code_gen".to_string(),
            serde_json::json!({}),
            serde_json::json!({}),
        );

        let decision2 = Decision::new(
            context.clone(),
            "refactor".to_string(),
            serde_json::json!({}),
            serde_json::json!({}),
        );

        logger.log_decision(decision1).await.unwrap();
        logger.log_decision(decision2).await.unwrap();

        let replayed = logger.replay_decisions_for_context(&context).await;
        assert_eq!(replayed.len(), 2);
    }

    #[tokio::test]
    async fn test_decision_count() {
        let logger = DecisionLogger::new();

        assert_eq!(logger.decision_count().await, 0);

        let decision1 =
            create_test_decision("code_gen", "agent1", "/project", "/project/src/main.rs");
        logger.log_decision(decision1).await.unwrap();

        assert_eq!(logger.decision_count().await, 1);

        let decision2 =
            create_test_decision("refactor", "agent2", "/project", "/project/src/lib.rs");
        logger.log_decision(decision2).await.unwrap();

        assert_eq!(logger.decision_count().await, 2);
    }

    #[tokio::test]
    async fn test_clear() {
        let logger = DecisionLogger::new();

        let decision =
            create_test_decision("code_gen", "agent1", "/project", "/project/src/main.rs");
        logger.log_decision(decision).await.unwrap();

        assert_eq!(logger.decision_count().await, 1);

        logger.clear().await;

        assert_eq!(logger.decision_count().await, 0);
    }

    #[tokio::test]
    async fn test_get_statistics() {
        let logger = DecisionLogger::new();

        let decision1 =
            create_test_decision("code_gen", "agent1", "/project", "/project/src/main.rs");
        let decision2 =
            create_test_decision("refactor", "agent2", "/project", "/project/src/lib.rs");
        let decision3 =
            create_test_decision("code_gen", "agent1", "/project", "/project/src/utils.rs");

        logger.log_decision(decision1).await.unwrap();
        logger.log_decision(decision2).await.unwrap();
        logger.log_decision(decision3).await.unwrap();

        let stats = logger.get_statistics().await;

        assert_eq!(stats.total_decisions, 3);
        assert_eq!(stats.decision_types.get("code_gen"), Some(&2));
        assert_eq!(stats.decision_types.get("refactor"), Some(&1));
        assert_eq!(stats.agent_types.get("agent1"), Some(&2));
        assert_eq!(stats.agent_types.get("agent2"), Some(&1));
    }

    #[tokio::test]
    async fn test_multiple_decisions_same_context() {
        let logger = DecisionLogger::new();

        let context = DecisionContext {
            project_path: PathBuf::from("/project"),
            file_path: PathBuf::from("/project/src/main.rs"),
            line_number: 10,
            agent_type: "agent1".to_string(),
        };

        for i in 0..5 {
            let decision = Decision::new(
                context.clone(),
                format!("type_{}", i),
                serde_json::json!({}),
                serde_json::json!({}),
            );
            logger.log_decision(decision).await.unwrap();
        }

        let context_decisions = logger.get_history_by_context(&context).await;
        assert_eq!(context_decisions.len(), 5);
    }
}
