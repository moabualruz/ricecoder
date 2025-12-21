//! Tests for ricecoder-agents models
//!
//! Tests for agent configuration, output, findings, suggestions, and related data structures.

use std::collections::HashMap;
use std::path::PathBuf;

use ricecoder_agents::models::{
    AgentConfig, AgentMetadata, AgentMetrics, AgentOutput, CodeLocation, ConfigSchema, Finding,
    GeneratedContent, ProjectContext, Severity, Suggestion, TaskScope, TaskTarget, TaskType,
};

#[test]
fn test_agent_config_default() {
    let config = AgentConfig::default();
    assert!(config.enabled);
    assert!(config.settings.is_empty());
}

#[test]
fn test_agent_config_serialization() {
    let config = AgentConfig {
        enabled: true,
        settings: {
            let mut map = HashMap::new();
            map.insert("key".to_string(), serde_json::json!("value"));
            map
        },
    };

    let json = serde_json::to_string(&config).expect("serialization failed");
    let deserialized: AgentConfig = serde_json::from_str(&json).expect("deserialization failed");

    assert_eq!(deserialized.enabled, config.enabled);
    assert_eq!(deserialized.settings, config.settings);
}

#[test]
fn test_agent_output_default() {
    let output = AgentOutput::default();
    assert!(output.findings.is_empty());
    assert!(output.suggestions.is_empty());
    assert!(output.generated.is_empty());
}

#[test]
fn test_agent_output_serialization() {
    let output = AgentOutput {
        findings: vec![Finding {
            id: "finding-1".to_string(),
            severity: Severity::Warning,
            category: "quality".to_string(),
            message: "Test finding".to_string(),
            location: None,
            suggestion: None,
        }],
        suggestions: vec![],
        generated: vec![],
        metadata: AgentMetadata::default(),
    };

    let json = serde_json::to_string(&output).expect("serialization failed");
    let deserialized: AgentOutput = serde_json::from_str(&json).expect("deserialization failed");

    assert_eq!(deserialized.findings.len(), 1);
    assert_eq!(deserialized.findings[0].id, "finding-1");
    assert_eq!(deserialized.findings[0].severity, Severity::Warning);
}

#[test]
fn test_severity_ordering() {
    assert!(Severity::Info < Severity::Warning);
    assert!(Severity::Warning < Severity::Critical);
    assert!(Severity::Info < Severity::Critical);
}

#[test]
fn test_task_type_serialization() {
    let task_type = TaskType::CodeReview;
    let json = serde_json::to_string(&task_type).expect("serialization failed");
    let deserialized: TaskType = serde_json::from_str(&json).expect("deserialization failed");

    assert_eq!(deserialized, task_type);
}

#[test]
fn test_task_scope_serialization() {
    let scope = TaskScope::Module;
    let json = serde_json::to_string(&scope).expect("serialization failed");
    let deserialized: TaskScope = serde_json::from_str(&json).expect("deserialization failed");

    assert_eq!(deserialized, scope);
}

#[test]
fn test_finding_with_location() {
    let finding = Finding {
        id: "finding-1".to_string(),
        severity: Severity::Critical,
        category: "security".to_string(),
        message: "Security vulnerability".to_string(),
        location: Some(CodeLocation {
            file: PathBuf::from("src/main.rs"),
            line: 42,
            column: 10,
        }),
        suggestion: Some("Use safe alternative".to_string()),
    };

    assert_eq!(finding.id, "finding-1");
    assert_eq!(finding.severity, Severity::Critical);
    assert!(finding.location.is_some());
    assert!(finding.suggestion.is_some());

    let location = finding.location.unwrap();
    assert_eq!(location.line, 42);
    assert_eq!(location.column, 10);
}

#[test]
fn test_suggestion_auto_fixable() {
    let suggestion = Suggestion {
        id: "suggestion-1".to_string(),
        description: "Fix naming".to_string(),
        diff: None,
        auto_fixable: true,
    };

    assert!(suggestion.auto_fixable);
}

#[test]
fn test_agent_metadata_serialization() {
    let metadata = AgentMetadata {
        agent_id: "test-agent".to_string(),
        execution_time_ms: 1000,
        tokens_used: 500,
    };

    let json = serde_json::to_string(&metadata).expect("serialization failed");
    let deserialized: AgentMetadata = serde_json::from_str(&json).expect("deserialization failed");

    assert_eq!(deserialized.agent_id, "test-agent");
    assert_eq!(deserialized.execution_time_ms, 1000);
    assert_eq!(deserialized.tokens_used, 500);
}

#[test]
fn test_agent_metrics_default() {
    let metrics = AgentMetrics::default();
    assert_eq!(metrics.execution_count, 0);
    assert_eq!(metrics.success_count, 0);
    assert_eq!(metrics.error_count, 0);
    assert_eq!(metrics.avg_duration_ms, 0.0);
}

#[test]
fn test_config_schema_default() {
    let schema = ConfigSchema::default();
    assert!(schema.properties.is_empty());
}

#[test]
fn test_task_target_multiple_files() {
    let target = TaskTarget {
        files: vec![
            PathBuf::from("src/main.rs"),
            PathBuf::from("src/lib.rs"),
            PathBuf::from("tests/test.rs"),
        ],
        scope: TaskScope::Project,
    };

    assert_eq!(target.files.len(), 3);
    assert_eq!(target.scope, TaskScope::Project);
}

#[test]
fn test_project_context() {
    let context = ProjectContext {
        name: "my-project".to_string(),
        root: PathBuf::from("/home/user/projects/my-project"),
    };

    assert_eq!(context.name, "my-project");
    assert_eq!(
        context.root,
        PathBuf::from("/home/user/projects/my-project")
    );
}

#[test]
fn test_generated_content() {
    let content = GeneratedContent {
        file: PathBuf::from("generated/test.rs"),
        content: "// Generated code".to_string(),
    };

    assert_eq!(content.file, PathBuf::from("generated/test.rs"));
    assert_eq!(content.content, "// Generated code");
}
