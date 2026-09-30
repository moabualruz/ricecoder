//! Integration tests for public multi-agent orchestration workflows.

use std::{collections::BTreeMap, path::PathBuf, sync::Arc};

use ricecoder_agents::models::TaskOptions;
use ricecoder_agents::{
    Agent, AgentError, AgentInput, AgentMetadata, AgentOrchestrator, AgentOutput, AgentRegistry,
    AgentTask, Finding, Severity, TaskScope, TaskTarget, TaskType,
};

struct TestAgent {
    id: String,
    task_type: TaskType,
    finding_count: usize,
}

#[async_trait::async_trait]
impl Agent for TestAgent {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        "Integration Test Agent"
    }

    fn description(&self) -> &str {
        "Produces deterministic integration-test output"
    }

    fn supports(&self, task_type: TaskType) -> bool {
        task_type == self.task_type
    }

    async fn execute(&self, input: AgentInput) -> Result<AgentOutput, AgentError> {
        assert_eq!(input.task.task_type, self.task_type);
        let findings = (0..self.finding_count)
            .map(|index| Finding {
                id: format!("{}-{index}", self.id),
                severity: Severity::Warning,
                category: "integration-test".to_string(),
                message: format!("Finding {index} from {}", self.id),
                location: None,
                suggestion: Some(format!("Resolve finding {index} from {}", self.id)),
            })
            .collect();

        Ok(AgentOutput {
            findings,
            metadata: AgentMetadata {
                agent_id: self.id.clone(),
                execution_time_ms: 10,
                tokens_used: 0,
            },
            ..AgentOutput::default()
        })
    }
}

fn test_agent(id: &str, task_type: TaskType, finding_count: usize) -> TestAgent {
    TestAgent {
        id: id.to_string(),
        task_type,
        finding_count,
    }
}

fn orchestrator(agents: impl IntoIterator<Item = TestAgent>) -> AgentOrchestrator {
    let mut registry = AgentRegistry::new();
    for agent in agents {
        registry.register(Arc::new(agent));
    }
    AgentOrchestrator::with_defaults(Arc::new(registry))
}

fn task(id: &str, task_type: TaskType) -> AgentTask {
    AgentTask {
        id: id.to_string(),
        task_type,
        target: TaskTarget {
            files: vec![PathBuf::from("test.rs")],
            scope: TaskScope::File,
        },
        options: TaskOptions::default(),
    }
}

fn finding_counts_by_agent(results: &[AgentOutput]) -> BTreeMap<String, usize> {
    results
        .iter()
        .map(|result| (result.metadata.agent_id.clone(), result.findings.len()))
        .collect()
}

#[tokio::test]
async fn task_results_match_each_registered_agent() {
    let orchestrator = orchestrator([
        test_agent("agent-1", TaskType::CodeReview, 0),
        test_agent("agent-2", TaskType::SecurityAnalysis, 0),
    ]);
    let results = orchestrator
        .execute(vec![
            task("task-1", TaskType::CodeReview),
            task("task-2", TaskType::SecurityAnalysis),
        ])
        .await
        .unwrap();

    assert_eq!(results.len(), 2);
    assert_eq!(
        finding_counts_by_agent(&results),
        BTreeMap::from([("agent-1".to_string(), 0), ("agent-2".to_string(), 0)])
    );
}

#[tokio::test]
async fn findings_remain_attached_to_their_task_result() {
    let orchestrator = orchestrator([
        test_agent("agent-1", TaskType::CodeReview, 3),
        test_agent("agent-2", TaskType::SecurityAnalysis, 2),
    ]);
    let results = orchestrator
        .execute(vec![
            task("task-1", TaskType::CodeReview),
            task("task-2", TaskType::SecurityAnalysis),
        ])
        .await
        .unwrap();

    assert_eq!(
        finding_counts_by_agent(&results),
        BTreeMap::from([("agent-1".to_string(), 3), ("agent-2".to_string(), 2)])
    );
}

#[tokio::test]
async fn parallel_execution_runs_every_registered_agent() {
    let orchestrator = orchestrator([
        test_agent("agent-1", TaskType::CodeReview, 0),
        test_agent("agent-2", TaskType::SecurityAnalysis, 0),
    ]);
    let results = orchestrator
        .execute(vec![
            task("task-1", TaskType::CodeReview),
            task("task-2", TaskType::SecurityAnalysis),
        ])
        .await
        .unwrap();

    assert_eq!(results.len(), 2);
}

#[tokio::test]
async fn parallel_execution_keeps_each_agents_findings() {
    let orchestrator = orchestrator([
        test_agent("agent-1", TaskType::CodeReview, 2),
        test_agent("agent-2", TaskType::SecurityAnalysis, 3),
        test_agent("agent-3", TaskType::Refactoring, 1),
    ]);
    let results = orchestrator
        .execute(vec![
            task("task-1", TaskType::CodeReview),
            task("task-2", TaskType::SecurityAnalysis),
            task("task-3", TaskType::Refactoring),
        ])
        .await
        .unwrap();

    assert_eq!(results.len(), 3);
    assert_eq!(
        finding_counts_by_agent(&results),
        BTreeMap::from([
            ("agent-1".to_string(), 2),
            ("agent-2".to_string(), 3),
            ("agent-3".to_string(), 1),
        ])
    );
}

#[tokio::test]
async fn repeated_parallel_execution_returns_deterministic_results() {
    let orchestrator = orchestrator([
        test_agent("agent-1", TaskType::CodeReview, 5),
        test_agent("agent-2", TaskType::SecurityAnalysis, 3),
    ]);
    let tasks = vec![
        task("task-1", TaskType::CodeReview),
        task("task-2", TaskType::SecurityAnalysis),
    ];

    let first = orchestrator.execute(tasks.clone()).await.unwrap();
    let second = orchestrator.execute(tasks).await.unwrap();

    let expected = BTreeMap::from([("agent-1".to_string(), 5), ("agent-2".to_string(), 3)]);
    assert_eq!(first.len(), second.len());
    assert_eq!(finding_counts_by_agent(&first), expected);
    assert_eq!(finding_counts_by_agent(&second), expected);
}

#[tokio::test]
async fn conditional_workflow_continues_when_predicate_allows_it() {
    let orchestrator = orchestrator([
        test_agent("agent-1", TaskType::CodeReview, 0),
        test_agent("agent-2", TaskType::SecurityAnalysis, 0),
    ]);
    let results = orchestrator
        .execute_conditional(
            vec![
                task("task-1", TaskType::CodeReview),
                task("task-2", TaskType::SecurityAnalysis),
            ],
            |outputs| outputs.len() < 2,
        )
        .await
        .unwrap();

    assert_eq!(results.len(), 2);
}

#[tokio::test]
async fn conditional_workflow_stops_when_predicate_rejects_the_first_phase() {
    let orchestrator = orchestrator([test_agent("agent-1", TaskType::CodeReview, 0)]);
    let results = orchestrator
        .execute_conditional(vec![task("task-1", TaskType::CodeReview)], |_| false)
        .await
        .unwrap();

    assert_eq!(results.len(), 0);
}

#[tokio::test]
async fn conditional_workflow_can_continue_based_on_findings() {
    let orchestrator = orchestrator([
        test_agent("agent-1", TaskType::CodeReview, 2),
        test_agent("agent-2", TaskType::SecurityAnalysis, 0),
    ]);
    let results = orchestrator
        .execute_conditional(
            vec![
                task("task-1", TaskType::CodeReview),
                task("task-2", TaskType::SecurityAnalysis),
            ],
            |outputs| outputs.is_empty() || !outputs[0].findings.is_empty(),
        )
        .await
        .unwrap();

    assert_eq!(results.len(), 2);
}

#[tokio::test]
async fn empty_task_list_returns_no_results() {
    let orchestrator = orchestrator([]);
    let results = orchestrator.execute(vec![]).await.unwrap();

    assert_eq!(results.len(), 0);
}

#[tokio::test]
async fn missing_agent_returns_an_error() {
    let orchestrator = orchestrator([]);
    let result = orchestrator
        .execute(vec![task("task-1", TaskType::CodeReview)])
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn result_aggregation_preserves_all_unique_findings() {
    let orchestrator = orchestrator([
        test_agent("agent-1", TaskType::CodeReview, 3),
        test_agent("agent-2", TaskType::SecurityAnalysis, 2),
    ]);
    let results = orchestrator
        .execute(vec![
            task("task-1", TaskType::CodeReview),
            task("task-2", TaskType::SecurityAnalysis),
        ])
        .await
        .unwrap();

    let total_findings: usize = results.iter().map(|result| result.findings.len()).sum();
    assert_eq!(total_findings, 5);
}

#[tokio::test]
async fn execute_and_aggregate_returns_all_unique_findings() {
    let orchestrator = orchestrator([
        test_agent("agent-1", TaskType::CodeReview, 2),
        test_agent("agent-2", TaskType::SecurityAnalysis, 3),
    ]);
    let aggregated = orchestrator
        .execute_and_aggregate(vec![
            task("task-1", TaskType::CodeReview),
            task("task-2", TaskType::SecurityAnalysis),
        ])
        .await
        .unwrap();

    assert_eq!(aggregated.findings.len(), 5);
}
