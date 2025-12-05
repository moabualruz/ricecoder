//! Property-based tests for the orchestrator
//!
//! **Feature: ricecoder-agents, Property 4: Result Aggregation Correctness**
//! **Validates: Requirements 3.6, 4.1, 4.2**

#[cfg(test)]
mod tests {
    use crate::models::{AgentMetadata, AgentOutput, Finding, Severity};
    use crate::orchestrator::AgentOrchestrator;
    use crate::registry::AgentRegistry;
    use std::sync::Arc;

    fn create_finding(id: &str, severity: Severity, category: &str, message: &str) -> Finding {
        Finding {
            id: id.to_string(),
            severity,
            category: category.to_string(),
            message: message.to_string(),
            location: None,
            suggestion: None,
        }
    }

    fn create_output_with_findings(agent_id: &str, findings: Vec<Finding>) -> AgentOutput {
        let mut output = AgentOutput::default();
        output.findings = findings;
        output.metadata = AgentMetadata {
            agent_id: agent_id.to_string(),
            execution_time_ms: 100,
            tokens_used: 50,
        };
        output
    }

    /// Property 4: Result Aggregation Correctness
    /// For any multi-agent execution, aggregated results SHALL include all findings
    /// from all agents without loss or duplication.
    ///
    /// This property tests that:
    /// 1. All findings from all agents are included in the aggregated result
    /// 2. No findings are lost during aggregation
    /// 3. The aggregation is deterministic (same inputs produce same outputs)
    #[test]
    fn property_result_aggregation_correctness_all_findings_included() {
        let registry = Arc::new(AgentRegistry::new());
        let orchestrator = AgentOrchestrator::new(registry);

        // Create findings from multiple agents
        let agent1_findings = vec![
            create_finding("f1", Severity::Critical, "security", "issue1"),
            create_finding("f2", Severity::Warning, "quality", "issue2"),
        ];

        let agent2_findings = vec![
            create_finding("f3", Severity::Info, "style", "issue3"),
            create_finding("f4", Severity::Critical, "performance", "issue4"),
        ];

        let agent3_findings = vec![create_finding("f5", Severity::Warning, "quality", "issue5")];

        let output1 = create_output_with_findings("agent1", agent1_findings);
        let output2 = create_output_with_findings("agent2", agent2_findings);
        let output3 = create_output_with_findings("agent3", agent3_findings);

        // Aggregate multiple times to verify determinism
        let mut all_results = Vec::new();
        for _ in 0..3 {
            let result = orchestrator
                .coordinator()
                .aggregate(vec![output1.clone(), output2.clone(), output3.clone()])
                .unwrap();
            all_results.push(result);
        }

        // Property: All aggregations should produce the same number of findings
        for result in &all_results {
            assert_eq!(
                result.findings.len(),
                5,
                "All 5 unique findings should be included in aggregation"
            );
        }

        // Property: All finding IDs should be present in all aggregations
        for result in &all_results {
            let finding_ids: std::collections::HashSet<_> =
                result.findings.iter().map(|f| f.id.clone()).collect();
            assert!(finding_ids.contains("f1"), "Finding f1 should be included");
            assert!(finding_ids.contains("f2"), "Finding f2 should be included");
            assert!(finding_ids.contains("f3"), "Finding f3 should be included");
            assert!(finding_ids.contains("f4"), "Finding f4 should be included");
            assert!(finding_ids.contains("f5"), "Finding f5 should be included");
        }

        // Property: Aggregations should be deterministic (same order)
        let first_result = &all_results[0];
        for result in &all_results[1..] {
            for (i, finding) in result.findings.iter().enumerate() {
                assert_eq!(
                    finding.id, first_result.findings[i].id,
                    "Finding {} should have same ID in all aggregations",
                    i
                );
                assert_eq!(
                    finding.severity, first_result.findings[i].severity,
                    "Finding {} should have same severity in all aggregations",
                    i
                );
            }
        }
    }

    /// Property 4: Result Aggregation Correctness (No Duplication)
    /// For any duplicate findings from multiple agents, only unique findings
    /// should be kept in the aggregated result.
    ///
    /// This property tests that:
    /// 1. Duplicate findings are removed
    /// 2. Unique findings are preserved
    /// 3. Deduplication is consistent across multiple runs
    #[test]
    fn property_result_aggregation_correctness_no_duplication() {
        let registry = Arc::new(AgentRegistry::new());
        let orchestrator = AgentOrchestrator::new(registry);

        // Create duplicate findings from multiple agents
        let agent1_findings = vec![
            create_finding("f1", Severity::Critical, "security", "sql injection"),
            create_finding("f2", Severity::Warning, "quality", "long function"),
            create_finding("f3", Severity::Info, "style", "spacing"),
        ];

        let agent2_findings = vec![
            create_finding("f4", Severity::Critical, "security", "sql injection"), // Duplicate
            create_finding("f5", Severity::Warning, "quality", "long function"),   // Duplicate
            create_finding("f6", Severity::Critical, "performance", "n+1 query"),
        ];

        let agent3_findings = vec![
            create_finding("f7", Severity::Info, "style", "spacing"), // Duplicate
            create_finding("f8", Severity::Warning, "quality", "naming"),
        ];

        let output1 = create_output_with_findings("agent1", agent1_findings);
        let output2 = create_output_with_findings("agent2", agent2_findings);
        let output3 = create_output_with_findings("agent3", agent3_findings);

        // Aggregate multiple times
        let mut all_results = Vec::new();
        for _ in 0..3 {
            let result = orchestrator
                .coordinator()
                .aggregate(vec![output1.clone(), output2.clone(), output3.clone()])
                .unwrap();
            all_results.push(result);
        }

        // Property: All aggregations should have the same number of unique findings
        for result in &all_results {
            assert_eq!(
                result.findings.len(),
                5,
                "Duplicate findings should be removed, leaving 5 unique findings"
            );
        }

        // Property: Deduplication should be consistent
        let first_result = &all_results[0];
        for result in &all_results[1..] {
            for (i, finding) in result.findings.iter().enumerate() {
                assert_eq!(
                    finding.category, first_result.findings[i].category,
                    "Finding {} category should be consistent across aggregations",
                    i
                );
                assert_eq!(
                    finding.message, first_result.findings[i].message,
                    "Finding {} message should be consistent across aggregations",
                    i
                );
            }
        }

        // Property: Unique findings should be preserved
        let first_result = &all_results[0];
        let categories: Vec<_> = first_result.findings.iter().map(|f| &f.category).collect();
        assert!(categories.contains(&&"security".to_string()));
        assert!(categories.contains(&&"quality".to_string()));
        assert!(categories.contains(&&"style".to_string()));
        assert!(categories.contains(&&"performance".to_string()));
    }

    /// Property 4: Result Aggregation Correctness (Severity Preservation)
    /// For any findings with different severity levels, all severity levels
    /// should be preserved in the aggregated result.
    ///
    /// This property tests that:
    /// 1. All severity levels are preserved
    /// 2. Findings are sorted by severity (Critical > Warning > Info)
    /// 3. Severity preservation is consistent across multiple runs
    #[test]
    fn property_result_aggregation_correctness_severity_preservation() {
        let registry = Arc::new(AgentRegistry::new());
        let orchestrator = AgentOrchestrator::new(registry);

        // Create findings with all severity levels
        let agent1_findings = vec![
            create_finding("f1", Severity::Critical, "cat1", "msg1"),
            create_finding("f2", Severity::Warning, "cat2", "msg2"),
            create_finding("f3", Severity::Info, "cat3", "msg3"),
        ];

        let agent2_findings = vec![
            create_finding("f4", Severity::Warning, "cat4", "msg4"),
            create_finding("f5", Severity::Critical, "cat5", "msg5"),
            create_finding("f6", Severity::Info, "cat6", "msg6"),
        ];

        let output1 = create_output_with_findings("agent1", agent1_findings);
        let output2 = create_output_with_findings("agent2", agent2_findings);

        // Aggregate multiple times
        for _ in 0..3 {
            let result = orchestrator
                .coordinator()
                .aggregate(vec![output1.clone(), output2.clone()])
                .unwrap();

            // Property: Each severity level should be preserved
            assert!(
                result
                    .findings
                    .iter()
                    .any(|f| f.severity == Severity::Critical),
                "Critical findings should be preserved"
            );
            assert!(
                result
                    .findings
                    .iter()
                    .any(|f| f.severity == Severity::Warning),
                "Warning findings should be preserved"
            );
            assert!(
                result.findings.iter().any(|f| f.severity == Severity::Info),
                "Info findings should be preserved"
            );

            // Property: Findings should be sorted by severity (descending)
            for i in 0..result.findings.len() - 1 {
                assert!(
                    result.findings[i].severity >= result.findings[i + 1].severity,
                    "Findings should be sorted by severity (Critical > Warning > Info)"
                );
            }

            // Property: Critical findings should come first
            let critical_count = result
                .findings
                .iter()
                .filter(|f| f.severity == Severity::Critical)
                .count();
            assert_eq!(critical_count, 2, "Should have 2 critical findings");
            assert_eq!(
                result.findings[0].severity,
                Severity::Critical,
                "First finding should be critical"
            );
            assert_eq!(
                result.findings[1].severity,
                Severity::Critical,
                "Second finding should be critical"
            );
        }
    }

    /// Property 4: Result Aggregation Correctness (Completeness)
    /// For any number of agents with varying numbers of findings,
    /// all unique findings should be aggregated.
    ///
    /// This property tests that:
    /// 1. Aggregation works with varying numbers of agents
    /// 2. Aggregation works with varying numbers of findings per agent
    /// 3. Completeness is maintained across different configurations
    #[test]
    fn property_result_aggregation_correctness_completeness() {
        let registry = Arc::new(AgentRegistry::new());
        let orchestrator = AgentOrchestrator::new(registry);

        // Test with varying numbers of agents and findings
        for num_agents in &[1, 2, 3, 5] {
            for findings_per_agent in &[1, 2, 3] {
                let mut outputs = Vec::new();
                let mut total_unique_findings = 0;

                for agent_idx in 0..*num_agents {
                    let mut findings = Vec::new();
                    for finding_idx in 0..*findings_per_agent {
                        findings.push(create_finding(
                            &format!("f{}_{}", agent_idx, finding_idx),
                            Severity::Warning,
                            &format!("cat{}", agent_idx),
                            &format!("msg{}_{}", agent_idx, finding_idx),
                        ));
                        total_unique_findings += 1;
                    }
                    outputs.push(create_output_with_findings(
                        &format!("agent{}", agent_idx),
                        findings,
                    ));
                }

                let result = orchestrator.coordinator().aggregate(outputs).unwrap();

                // Property: All findings should be aggregated
                assert_eq!(
                    result.findings.len(),
                    total_unique_findings,
                    "All {} findings from {} agents should be aggregated",
                    total_unique_findings,
                    num_agents
                );
            }
        }
    }

    /// Property 4: Result Aggregation Correctness (Empty Inputs)
    /// For any empty input, aggregation should produce empty output.
    ///
    /// This property tests that:
    /// 1. Empty input produces empty output
    /// 2. Empty aggregation is consistent
    #[test]
    fn property_result_aggregation_correctness_empty_inputs() {
        let registry = Arc::new(AgentRegistry::new());
        let orchestrator = AgentOrchestrator::new(registry);

        // Aggregate empty outputs multiple times
        let mut all_results = Vec::new();
        for _ in 0..5 {
            let result = orchestrator.coordinator().aggregate(vec![]).unwrap();
            all_results.push(result);
        }

        // Property: All aggregations should produce empty findings
        for result in &all_results {
            assert_eq!(
                result.findings.len(),
                0,
                "Empty input should consistently produce empty findings"
            );
        }
    }

    /// Property 4: Result Aggregation Correctness (Single Agent)
    /// For any single agent output, aggregation should be consistent.
    ///
    /// This property tests that:
    /// 1. Single agent aggregation is deterministic
    /// 2. All findings are preserved
    /// 3. Aggregation is idempotent for single agent
    #[test]
    fn property_result_aggregation_correctness_single_agent() {
        let registry = Arc::new(AgentRegistry::new());
        let orchestrator = AgentOrchestrator::new(registry);

        let findings = vec![
            create_finding("f1", Severity::Warning, "quality", "issue1"),
            create_finding("f2", Severity::Critical, "security", "issue2"),
            create_finding("f3", Severity::Info, "style", "issue3"),
        ];

        let output = create_output_with_findings("agent1", findings);

        // Aggregate multiple times
        let mut all_results = Vec::new();
        for _ in 0..5 {
            let result = orchestrator
                .coordinator()
                .aggregate(vec![output.clone()])
                .unwrap();
            all_results.push(result);
        }

        // Property: All aggregations should be identical
        let first_result = &all_results[0];
        for result in &all_results[1..] {
            assert_eq!(
                result.findings.len(),
                first_result.findings.len(),
                "All aggregations should have same number of findings"
            );

            for (i, finding) in result.findings.iter().enumerate() {
                assert_eq!(
                    finding.id, first_result.findings[i].id,
                    "Finding {} ID should be consistent",
                    i
                );
                assert_eq!(
                    finding.severity, first_result.findings[i].severity,
                    "Finding {} severity should be consistent",
                    i
                );
                assert_eq!(
                    finding.category, first_result.findings[i].category,
                    "Finding {} category should be consistent",
                    i
                );
                assert_eq!(
                    finding.message, first_result.findings[i].message,
                    "Finding {} message should be consistent",
                    i
                );
            }
        }
    }
}
