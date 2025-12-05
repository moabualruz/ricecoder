//! Property-based tests for the coordinator
//!
//! **Feature: ricecoder-agents, Property 2: Finding Consistency**
//! **Validates: Requirements 2.1, 2.2, 2.3, 2.4**

#[cfg(test)]
mod tests {
    use crate::coordinator::AgentCoordinator;
    use crate::models::{AgentMetadata, AgentOutput, Finding, Severity};

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

    fn create_output_with_agent(agent_id: &str, findings: Vec<Finding>) -> AgentOutput {
        let mut output = AgentOutput::default();
        output.findings = findings;
        output.metadata = AgentMetadata {
            agent_id: agent_id.to_string(),
            execution_time_ms: 100,
            tokens_used: 50,
        };
        output
    }

    /// Property 2: Finding Consistency
    /// For any code input, the same agent SHALL produce consistent findings for unchanged code
    /// across multiple executions.
    ///
    /// This property tests that:
    /// 1. Multiple aggregations of the same findings produce identical results
    /// 2. Finding order is deterministic (sorted by severity)
    /// 3. Deduplication is consistent across runs
    #[test]
    fn property_finding_consistency_deterministic_aggregation() {
        let coordinator = AgentCoordinator::new();

        // Create consistent findings
        let findings = vec![
            create_finding("f1", Severity::Warning, "quality", "naming issue"),
            create_finding("f2", Severity::Critical, "security", "hardcoded secret"),
            create_finding("f3", Severity::Info, "style", "spacing"),
        ];

        let output = create_output_with_agent("agent1", findings);

        // Aggregate multiple times
        let mut all_results = Vec::new();
        for _ in 0..5 {
            let result = coordinator.aggregate(vec![output.clone()]).unwrap();
            all_results.push(result);
        }

        // Property: All aggregations should produce the same number of findings
        for result in &all_results {
            assert_eq!(
                result.findings.len(),
                3,
                "Each aggregation should produce 3 findings"
            );
        }

        // Property: All aggregations should have findings in the same order (by severity)
        let first_result = &all_results[0];
        for result in &all_results[1..] {
            for (i, finding) in result.findings.iter().enumerate() {
                assert_eq!(
                    finding.severity, first_result.findings[i].severity,
                    "Finding {} should have same severity in all aggregations",
                    i
                );
                assert_eq!(
                    finding.id, first_result.findings[i].id,
                    "Finding {} should have same ID in all aggregations",
                    i
                );
            }
        }

        // Property: Severity order should be Critical > Warning > Info
        assert_eq!(all_results[0].findings[0].severity, Severity::Critical);
        assert_eq!(all_results[0].findings[1].severity, Severity::Warning);
        assert_eq!(all_results[0].findings[2].severity, Severity::Info);
    }

    /// Property 2: Finding Consistency (Deduplication)
    /// For any set of duplicate findings, deduplication should be consistent
    #[test]
    fn property_finding_consistency_deduplication() {
        let coordinator = AgentCoordinator::new();

        // Create duplicate findings from multiple agents
        let findings1 = vec![
            create_finding("f1", Severity::Warning, "quality", "naming issue"),
            create_finding("f2", Severity::Critical, "security", "hardcoded secret"),
        ];

        let findings2 = vec![
            create_finding("f3", Severity::Warning, "quality", "naming issue"), // Duplicate
            create_finding("f4", Severity::Info, "style", "spacing"),
        ];

        let output1 = create_output_with_agent("agent1", findings1);
        let output2 = create_output_with_agent("agent2", findings2);

        // Aggregate multiple times with same inputs
        let mut all_results = Vec::new();
        for _ in 0..3 {
            let result = coordinator.aggregate(vec![output1.clone(), output2.clone()]).unwrap();
            all_results.push(result);
        }

        // Property: All aggregations should deduplicate to the same number of findings
        for result in &all_results {
            assert_eq!(
                result.findings.len(),
                3,
                "Deduplication should consistently produce 3 findings"
            );
        }

        // Property: Duplicate findings should be removed consistently
        let first_result = &all_results[0];
        for result in &all_results[1..] {
            for (i, finding) in result.findings.iter().enumerate() {
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

    /// Property 2: Finding Consistency (Multiple Agents)
    /// For any set of agents analyzing the same code, findings should be consistent
    #[test]
    fn property_finding_consistency_multiple_agents() {
        let coordinator = AgentCoordinator::new();

        // Simulate multiple agents analyzing the same code
        let agent1_findings = vec![
            create_finding("f1", Severity::Critical, "security", "sql injection"),
            create_finding("f2", Severity::Warning, "quality", "long function"),
        ];

        let agent2_findings = vec![
            create_finding("f3", Severity::Critical, "security", "sql injection"), // Same issue
            create_finding("f4", Severity::Info, "style", "naming"),
        ];

        let agent3_findings = vec![
            create_finding("f5", Severity::Warning, "quality", "long function"), // Same issue
            create_finding("f6", Severity::Critical, "performance", "n+1 query"),
        ];

        let output1 = create_output_with_agent("agent1", agent1_findings);
        let output2 = create_output_with_agent("agent2", agent2_findings);
        let output3 = create_output_with_agent("agent3", agent3_findings);

        // Aggregate multiple times
        let mut all_results = Vec::new();
        for _ in 0..3 {
            let result = coordinator
                .aggregate(vec![output1.clone(), output2.clone(), output3.clone()])
                .unwrap();
            all_results.push(result);
        }

        // Property: All aggregations should produce the same number of deduplicated findings
        for result in &all_results {
            assert_eq!(
                result.findings.len(),
                4,
                "Should consistently deduplicate to 4 unique findings"
            );
        }

        // Property: Critical findings should come first
        for result in &all_results {
            let critical_count = result
                .findings
                .iter()
                .filter(|f| f.severity == Severity::Critical)
                .count();
            assert_eq!(critical_count, 2, "Should have 2 critical findings");
            assert_eq!(
                result.findings[0].severity, Severity::Critical,
                "First finding should be critical"
            );
            assert_eq!(
                result.findings[1].severity, Severity::Critical,
                "Second finding should be critical"
            );
        }

        // Property: Findings should be sorted by severity
        for result in &all_results {
            for i in 0..result.findings.len() - 1 {
                assert!(
                    result.findings[i].severity >= result.findings[i + 1].severity,
                    "Findings should be sorted by severity (descending)"
                );
            }
        }
    }

    /// Property 2: Finding Consistency (Empty Inputs)
    /// For any empty input, aggregation should consistently produce empty output
    #[test]
    fn property_finding_consistency_empty_inputs() {
        let coordinator = AgentCoordinator::new();

        // Aggregate empty outputs multiple times
        let mut all_results = Vec::new();
        for _ in 0..5 {
            let result = coordinator.aggregate(vec![]).unwrap();
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

    /// Property 2: Finding Consistency (Single Agent)
    /// For any single agent output, aggregation should be consistent
    #[test]
    fn property_finding_consistency_single_agent() {
        let coordinator = AgentCoordinator::new();

        let findings = vec![
            create_finding("f1", Severity::Warning, "quality", "issue1"),
            create_finding("f2", Severity::Critical, "security", "issue2"),
            create_finding("f3", Severity::Info, "style", "issue3"),
        ];

        let output = create_output_with_agent("agent1", findings);

        // Aggregate multiple times
        let mut all_results = Vec::new();
        for _ in 0..5 {
            let result = coordinator.aggregate(vec![output.clone()]).unwrap();
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

    /// Property 2: Finding Consistency (Severity Preservation)
    /// For any findings, severity levels should be preserved during aggregation
    #[test]
    fn property_finding_consistency_severity_preservation() {
        let coordinator = AgentCoordinator::new();

        // Create findings with all severity levels
        let findings = vec![
            create_finding("f1", Severity::Critical, "cat1", "msg1"),
            create_finding("f2", Severity::Warning, "cat2", "msg2"),
            create_finding("f3", Severity::Info, "cat3", "msg3"),
        ];

        let output = create_output_with_agent("agent1", findings.clone());

        // Aggregate multiple times
        for _ in 0..5 {
            let result = coordinator.aggregate(vec![output.clone()]).unwrap();

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
                result
                    .findings
                    .iter()
                    .any(|f| f.severity == Severity::Info),
                "Info findings should be preserved"
            );
        }
    }

    /// Property 4: Result Aggregation Correctness
    /// For any multi-agent execution, aggregated results SHALL include all findings
    /// from all agents without loss or duplication.
    ///
    /// This property tests that:
    /// 1. All unique findings are included
    /// 2. Duplicate findings are removed
    /// 3. No findings are lost during aggregation
    #[test]
    fn property_result_aggregation_correctness_no_loss() {
        let coordinator = AgentCoordinator::new();

        // Create findings from multiple agents
        let agent1_findings = vec![
            create_finding("f1", Severity::Critical, "security", "issue1"),
            create_finding("f2", Severity::Warning, "quality", "issue2"),
        ];

        let agent2_findings = vec![
            create_finding("f3", Severity::Info, "style", "issue3"),
            create_finding("f4", Severity::Critical, "performance", "issue4"),
        ];

        let agent3_findings = vec![
            create_finding("f5", Severity::Warning, "quality", "issue5"),
        ];

        let output1 = create_output_with_agent("agent1", agent1_findings);
        let output2 = create_output_with_agent("agent2", agent2_findings);
        let output3 = create_output_with_agent("agent3", agent3_findings);

        let result = coordinator
            .aggregate(vec![output1, output2, output3])
            .unwrap();

        // Property: All unique findings should be included
        assert_eq!(
            result.findings.len(),
            5,
            "All 5 unique findings should be included"
        );

        // Property: All finding IDs should be present
        let finding_ids: std::collections::HashSet<_> =
            result.findings.iter().map(|f| f.id.clone()).collect();
        assert!(finding_ids.contains("f1"));
        assert!(finding_ids.contains("f2"));
        assert!(finding_ids.contains("f3"));
        assert!(finding_ids.contains("f4"));
        assert!(finding_ids.contains("f5"));
    }

    /// Property 4: Result Aggregation Correctness (Deduplication)
    /// For any duplicate findings, only one should be kept
    #[test]
    fn property_result_aggregation_correctness_deduplication() {
        let coordinator = AgentCoordinator::new();

        // Create duplicate findings from multiple agents
        let agent1_findings = vec![
            create_finding("f1", Severity::Critical, "security", "sql injection"),
            create_finding("f2", Severity::Warning, "quality", "long function"),
        ];

        let agent2_findings = vec![
            create_finding("f3", Severity::Critical, "security", "sql injection"), // Duplicate
            create_finding("f4", Severity::Warning, "quality", "long function"), // Duplicate
        ];

        let output1 = create_output_with_agent("agent1", agent1_findings);
        let output2 = create_output_with_agent("agent2", agent2_findings);

        let result = coordinator.aggregate(vec![output1, output2]).unwrap();

        // Property: Duplicates should be removed
        assert_eq!(
            result.findings.len(),
            2,
            "Duplicate findings should be removed"
        );

        // Property: Unique findings should be preserved
        let categories: Vec<_> = result.findings.iter().map(|f| &f.category).collect();
        assert!(categories.contains(&&"security".to_string()));
        assert!(categories.contains(&&"quality".to_string()));
    }

    /// Property 4: Result Aggregation Correctness (Completeness)
    /// For any number of agents, all findings should be aggregated
    #[test]
    fn property_result_aggregation_correctness_completeness() {
        let coordinator = AgentCoordinator::new();

        // Test with varying numbers of agents
        for num_agents in &[1, 2, 3, 5, 10] {
            let mut outputs = Vec::new();

            for agent_idx in 0..*num_agents {
                let findings = vec![
                    create_finding(
                        &format!("f{}_1", agent_idx),
                        Severity::Critical,
                        &format!("cat{}", agent_idx),
                        &format!("msg{}_1", agent_idx),
                    ),
                    create_finding(
                        &format!("f{}_2", agent_idx),
                        Severity::Warning,
                        &format!("cat{}", agent_idx),
                        &format!("msg{}_2", agent_idx),
                    ),
                ];

                outputs.push(create_output_with_agent(&format!("agent{}", agent_idx), findings));
            }

            let result = coordinator.aggregate(outputs).unwrap();

            // Property: All findings should be aggregated
            assert_eq!(
                result.findings.len(),
                num_agents * 2,
                "All {} agents' findings should be aggregated",
                num_agents
            );
        }
    }
}
