//! Agent coordinator for aggregating and prioritizing results

use crate::error::Result;
use crate::models::{AgentOutput, Finding, Suggestion};
use std::collections::HashMap;

/// Represents a finding with its source agent information
#[derive(Debug, Clone)]
struct FindingWithSource {
    /// The finding itself
    finding: Finding,
    /// ID of the agent that produced this finding
    agent_id: String,
}

/// Represents a suggestion with its source agent information
#[derive(Debug, Clone)]
struct SuggestionWithSource {
    /// The suggestion itself
    suggestion: Suggestion,
    /// ID of the agent that produced this suggestion
    agent_id: String,
}

/// Agent coordinator for aggregating and prioritizing results
pub struct AgentCoordinator {
    /// Tracks findings by their source agents for traceability
    findings_by_source: HashMap<String, Vec<Finding>>,
    /// Tracks suggestions by their source agents for traceability
    suggestions_by_source: HashMap<String, Vec<Suggestion>>,
}

impl AgentCoordinator {
    /// Create a new agent coordinator
    pub fn new() -> Self {
        Self {
            findings_by_source: HashMap::new(),
            suggestions_by_source: HashMap::new(),
        }
    }

    /// Aggregate results from multiple agents
    pub fn aggregate(&self, outputs: Vec<AgentOutput>) -> Result<AgentOutput> {
        let mut aggregated = AgentOutput::default();
        let mut findings_with_source = Vec::new();
        let mut suggestions_with_source = Vec::new();

        // Collect findings and suggestions with source tracking
        for output in outputs {
            let agent_id = output.metadata.agent_id.clone();

            for finding in output.findings {
                findings_with_source.push(FindingWithSource {
                    finding,
                    agent_id: agent_id.clone(),
                });
            }

            for suggestion in output.suggestions {
                suggestions_with_source.push(SuggestionWithSource {
                    suggestion,
                    agent_id: agent_id.clone(),
                });
            }

            // Aggregate generated content
            aggregated.generated.extend(output.generated);
        }

        // Deduplicate findings while maintaining source information
        let deduplicated_findings = self.deduplicate_findings_with_source(findings_with_source);
        aggregated.findings = deduplicated_findings;

        // Deduplicate suggestions while maintaining source information
        let deduplicated_suggestions =
            self.deduplicate_suggestions_with_source(suggestions_with_source);
        aggregated.suggestions = deduplicated_suggestions;

        // Sort by severity
        aggregated
            .findings
            .sort_by(|a, b| b.severity.cmp(&a.severity));

        Ok(aggregated)
    }

    /// Deduplicate findings while maintaining source information
    fn deduplicate_findings_with_source(
        &self,
        findings_with_source: Vec<FindingWithSource>,
    ) -> Vec<Finding> {
        let mut seen = HashMap::new();
        let mut deduplicated = Vec::new();

        for item in findings_with_source {
            let key = (item.finding.category.clone(), item.finding.message.clone());

            match seen.entry(key) {
                std::collections::hash_map::Entry::Occupied(mut entry) => {
                    let sources: &mut Vec<String> = entry.get_mut();
                    if !sources.contains(&item.agent_id) {
                        sources.push(item.agent_id);
                    }
                }
                std::collections::hash_map::Entry::Vacant(entry) => {
                    entry.insert(vec![item.agent_id]);
                    deduplicated.push(item.finding);
                }
            }
        }

        deduplicated
    }

    /// Deduplicate suggestions while maintaining source information
    fn deduplicate_suggestions_with_source(
        &self,
        suggestions_with_source: Vec<SuggestionWithSource>,
    ) -> Vec<Suggestion> {
        let mut seen = HashMap::new();
        let mut deduplicated = Vec::new();

        for item in suggestions_with_source {
            let key = item.suggestion.description.clone();

            match seen.entry(key) {
                std::collections::hash_map::Entry::Occupied(mut entry) => {
                    let sources: &mut Vec<String> = entry.get_mut();
                    if !sources.contains(&item.agent_id) {
                        sources.push(item.agent_id);
                    }
                }
                std::collections::hash_map::Entry::Vacant(entry) => {
                    entry.insert(vec![item.agent_id]);
                    deduplicated.push(item.suggestion);
                }
            }
        }

        deduplicated
    }

    /// Deduplicate findings (legacy method for backward compatibility)
    /// Reserved for future use when deduplication strategy needs to be applied
    #[allow(dead_code)]
    fn deduplicate_findings(&self, findings: &mut Vec<Finding>) {
        let mut seen = HashMap::new();
        findings.retain(|finding| {
            let key = (finding.category.clone(), finding.message.clone());
            seen.insert(key, true).is_none()
        });
    }

    /// Resolve conflicts between findings
    ///
    /// Conflicts occur when multiple agents report different findings for the same code location.
    /// This method applies severity-based prioritization to resolve conflicts:
    /// - Critical findings take precedence over Warning and Info
    /// - Warning findings take precedence over Info
    /// - When findings have the same severity, all are kept
    pub fn resolve_conflicts(&self, findings: &[Finding]) -> Vec<Finding> {
        if findings.is_empty() {
            return Vec::new();
        }

        // Group findings by location
        let mut findings_by_location: HashMap<String, Vec<Finding>> = HashMap::new();

        for finding in findings {
            let location_key = if let Some(loc) = &finding.location {
                format!("{}:{}:{}", loc.file.display(), loc.line, loc.column)
            } else {
                format!("{}:{}", finding.category, finding.message)
            };

            findings_by_location
                .entry(location_key)
                .or_default()
                .push(finding.clone());
        }

        // Resolve conflicts at each location by keeping highest severity
        let mut resolved = Vec::new();

        for (_, location_findings) in findings_by_location {
            if location_findings.is_empty() {
                continue;
            }

            // Find the highest severity at this location
            let max_severity = location_findings
                .iter()
                .map(|f| f.severity)
                .max()
                .unwrap_or(crate::models::Severity::Info);

            // Keep all findings with the highest severity
            for finding in location_findings {
                if finding.severity == max_severity {
                    resolved.push(finding);
                }
            }
        }

        resolved
    }

    /// Prioritize findings by severity
    pub fn prioritize(&self, findings: &mut [Finding]) {
        findings.sort_by(|a, b| b.severity.cmp(&a.severity));
    }

    /// Get findings grouped by source agent
    pub fn findings_by_source(&self) -> &HashMap<String, Vec<Finding>> {
        &self.findings_by_source
    }

    /// Get suggestions grouped by source agent
    pub fn suggestions_by_source(&self) -> &HashMap<String, Vec<Suggestion>> {
        &self.suggestions_by_source
    }
}

impl Default for AgentCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AgentMetadata, Severity};

    fn create_test_finding(id: &str, severity: Severity) -> Finding {
        Finding {
            id: id.to_string(),
            severity,
            category: "test".to_string(),
            message: "test message".to_string(),
            location: None,
            suggestion: None,
        }
    }

    fn create_test_output_with_agent(agent_id: &str, findings: Vec<Finding>) -> AgentOutput {
        let mut output = AgentOutput::default();
        output.findings = findings;
        output.metadata = AgentMetadata {
            agent_id: agent_id.to_string(),
            execution_time_ms: 100,
            tokens_used: 50,
        };
        output
    }

    #[test]
    fn test_aggregate_empty() {
        let coordinator = AgentCoordinator::new();
        let result = coordinator.aggregate(vec![]).unwrap();
        assert_eq!(result.findings.len(), 0);
    }

    #[test]
    fn test_aggregate_single_output() {
        let coordinator = AgentCoordinator::new();
        let output = create_test_output_with_agent(
            "agent1",
            vec![create_test_finding("f1", Severity::Warning)],
        );

        let result = coordinator.aggregate(vec![output]).unwrap();
        assert_eq!(result.findings.len(), 1);
    }

    #[test]
    fn test_aggregate_multiple_outputs() {
        let coordinator = AgentCoordinator::new();
        let output1 = create_test_output_with_agent(
            "agent1",
            vec![Finding {
                id: "f1".to_string(),
                severity: Severity::Warning,
                category: "category1".to_string(),
                message: "message1".to_string(),
                location: None,
                suggestion: None,
            }],
        );

        let output2 = create_test_output_with_agent(
            "agent2",
            vec![Finding {
                id: "f2".to_string(),
                severity: Severity::Critical,
                category: "category2".to_string(),
                message: "message2".to_string(),
                location: None,
                suggestion: None,
            }],
        );

        let result = coordinator.aggregate(vec![output1, output2]).unwrap();
        assert_eq!(result.findings.len(), 2);
    }

    #[test]
    fn test_deduplicate_findings() {
        let coordinator = AgentCoordinator::new();
        let mut findings = vec![
            create_test_finding("f1", Severity::Warning),
            create_test_finding("f2", Severity::Warning),
        ];

        coordinator.deduplicate_findings(&mut findings);
        // Both have same category and message, so should be deduplicated to 1
        assert_eq!(findings.len(), 1);
    }

    #[test]
    fn test_prioritize_findings() {
        let coordinator = AgentCoordinator::new();
        let mut findings = vec![
            create_test_finding("f1", Severity::Info),
            create_test_finding("f2", Severity::Critical),
            create_test_finding("f3", Severity::Warning),
        ];

        coordinator.prioritize(&mut findings);
        assert_eq!(findings[0].severity, Severity::Critical);
        assert_eq!(findings[1].severity, Severity::Warning);
        assert_eq!(findings[2].severity, Severity::Info);
    }

    #[test]
    fn test_deduplicate_findings_with_source() {
        let coordinator = AgentCoordinator::new();
        let output1 = create_test_output_with_agent(
            "agent1",
            vec![Finding {
                id: "f1".to_string(),
                severity: Severity::Warning,
                category: "quality".to_string(),
                message: "naming issue".to_string(),
                location: None,
                suggestion: None,
            }],
        );

        let output2 = create_test_output_with_agent(
            "agent2",
            vec![Finding {
                id: "f2".to_string(),
                severity: Severity::Warning,
                category: "quality".to_string(),
                message: "naming issue".to_string(),
                location: None,
                suggestion: None,
            }],
        );

        let result = coordinator.aggregate(vec![output1, output2]).unwrap();
        // Should deduplicate to 1 finding (same category and message)
        assert_eq!(result.findings.len(), 1);
    }

    #[test]
    fn test_aggregate_with_suggestions() {
        let coordinator = AgentCoordinator::new();
        let mut output1 = create_test_output_with_agent("agent1", vec![]);
        output1.suggestions.push(Suggestion {
            id: "s1".to_string(),
            description: "Use better naming".to_string(),
            diff: None,
            auto_fixable: true,
        });

        let mut output2 = create_test_output_with_agent("agent2", vec![]);
        output2.suggestions.push(Suggestion {
            id: "s2".to_string(),
            description: "Use better naming".to_string(),
            diff: None,
            auto_fixable: true,
        });

        let result = coordinator.aggregate(vec![output1, output2]).unwrap();
        // Should deduplicate suggestions with same description
        assert_eq!(result.suggestions.len(), 1);
    }

    #[test]
    fn test_aggregate_preserves_severity_order() {
        let coordinator = AgentCoordinator::new();
        let mut f1 = create_test_finding("f1", Severity::Info);
        f1.category = "cat1".to_string();
        f1.message = "msg1".to_string();

        let mut f2 = create_test_finding("f2", Severity::Critical);
        f2.category = "cat2".to_string();
        f2.message = "msg2".to_string();

        let mut f3 = create_test_finding("f3", Severity::Warning);
        f3.category = "cat3".to_string();
        f3.message = "msg3".to_string();

        let output = create_test_output_with_agent("agent1", vec![f1, f2, f3]);

        let result = coordinator.aggregate(vec![output]).unwrap();
        assert_eq!(result.findings[0].severity, Severity::Critical);
        assert_eq!(result.findings[1].severity, Severity::Warning);
        assert_eq!(result.findings[2].severity, Severity::Info);
    }

    #[test]
    fn test_resolve_conflicts() {
        let coordinator = AgentCoordinator::new();
        let mut f1 = create_test_finding("f1", Severity::Warning);
        f1.category = "category1".to_string();
        f1.message = "message1".to_string();

        let mut f2 = create_test_finding("f2", Severity::Critical);
        f2.category = "category2".to_string();
        f2.message = "message2".to_string();

        let findings = vec![f1, f2];
        let resolved = coordinator.resolve_conflicts(&findings);
        // Different categories/messages, so both should be kept
        assert_eq!(resolved.len(), 2);
    }

    #[test]
    fn test_resolve_conflicts_prioritizes_severity() {
        let coordinator = AgentCoordinator::new();
        let mut f1 = create_test_finding("f1", Severity::Info);
        f1.location = Some(crate::models::CodeLocation {
            file: std::path::PathBuf::from("test.rs"),
            line: 10,
            column: 5,
        });

        let mut f2 = create_test_finding("f2", Severity::Critical);
        f2.location = Some(crate::models::CodeLocation {
            file: std::path::PathBuf::from("test.rs"),
            line: 10,
            column: 5,
        });

        let findings = vec![f1, f2];
        let resolved = coordinator.resolve_conflicts(&findings);

        // Should keep only the Critical finding
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].severity, Severity::Critical);
    }

    #[test]
    fn test_resolve_conflicts_keeps_same_severity() {
        let coordinator = AgentCoordinator::new();
        let mut f1 = create_test_finding("f1", Severity::Warning);
        f1.location = Some(crate::models::CodeLocation {
            file: std::path::PathBuf::from("test.rs"),
            line: 10,
            column: 5,
        });

        let mut f2 = create_test_finding("f2", Severity::Warning);
        f2.location = Some(crate::models::CodeLocation {
            file: std::path::PathBuf::from("test.rs"),
            line: 10,
            column: 5,
        });

        let findings = vec![f1, f2];
        let resolved = coordinator.resolve_conflicts(&findings);

        // Should keep both findings with same severity
        assert_eq!(resolved.len(), 2);
    }

    #[test]
    fn test_resolve_conflicts_different_locations() {
        let coordinator = AgentCoordinator::new();
        let mut f1 = create_test_finding("f1", Severity::Warning);
        f1.location = Some(crate::models::CodeLocation {
            file: std::path::PathBuf::from("test.rs"),
            line: 10,
            column: 5,
        });

        let mut f2 = create_test_finding("f2", Severity::Critical);
        f2.location = Some(crate::models::CodeLocation {
            file: std::path::PathBuf::from("test.rs"),
            line: 20,
            column: 5,
        });

        let findings = vec![f1, f2];
        let resolved = coordinator.resolve_conflicts(&findings);

        // Should keep both findings at different locations
        assert_eq!(resolved.len(), 2);
    }

    #[test]
    fn test_aggregate_with_generated_content() {
        let coordinator = AgentCoordinator::new();
        let mut output = create_test_output_with_agent("agent1", vec![]);
        output.generated.push(crate::models::GeneratedContent {
            file: std::path::PathBuf::from("test.rs"),
            content: "// generated".to_string(),
        });

        let result = coordinator.aggregate(vec![output]).unwrap();
        assert_eq!(result.generated.len(), 1);
    }
}
