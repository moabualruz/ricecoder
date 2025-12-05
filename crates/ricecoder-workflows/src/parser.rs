//! Workflow definition parser

use crate::error::{WorkflowError, WorkflowResult};
use crate::models::Workflow;
use std::collections::{HashMap, HashSet};

/// Parses and validates workflow definitions
pub struct WorkflowParser;

impl WorkflowParser {
    /// Parse a workflow from YAML string
    ///
    /// Parses YAML into a Workflow struct, handling nested step configurations
    /// and resolving step references and dependencies.
    pub fn parse_yaml(yaml_content: &str) -> WorkflowResult<Workflow> {
        let workflow: Workflow = serde_yaml::from_str(yaml_content)
            .map_err(|e| WorkflowError::Invalid(format!("Failed to parse YAML: {}", e)))?;

        Self::validate(&workflow)?;
        Ok(workflow)
    }

    /// Parse a workflow from JSON string
    ///
    /// Parses JSON into a Workflow struct, handling nested step configurations
    /// and resolving step references and dependencies.
    pub fn parse_json(json_content: &str) -> WorkflowResult<Workflow> {
        let workflow: Workflow = serde_json::from_str(json_content)
            .map_err(|e| WorkflowError::Invalid(format!("Failed to parse JSON: {}", e)))?;

        Self::validate(&workflow)?;
        Ok(workflow)
    }

    /// Serialize a workflow to YAML string
    ///
    /// Converts a Workflow struct back to YAML format for round-trip testing.
    pub fn to_yaml(workflow: &Workflow) -> WorkflowResult<String> {
        serde_yaml::to_string(workflow)
            .map_err(|e| WorkflowError::Invalid(format!("Failed to serialize to YAML: {}", e)))
    }

    /// Serialize a workflow to JSON string
    ///
    /// Converts a Workflow struct back to JSON format for round-trip testing.
    pub fn to_json(workflow: &Workflow) -> WorkflowResult<String> {
        serde_json::to_string_pretty(workflow)
            .map_err(|e| WorkflowError::Invalid(format!("Failed to serialize to JSON: {}", e)))
    }

    /// Validate a workflow definition
    ///
    /// Validates:
    /// - Required fields (id, name, steps)
    /// - Step dependencies (no missing references)
    /// - Circular dependencies in step execution order
    /// - Parameter references
    pub fn validate(workflow: &Workflow) -> WorkflowResult<()> {
        // Validate required fields
        if workflow.id.is_empty() {
            return Err(WorkflowError::Invalid("Workflow id is required".to_string()));
        }

        if workflow.name.is_empty() {
            return Err(WorkflowError::Invalid("Workflow name is required".to_string()));
        }

        if workflow.steps.is_empty() {
            return Err(WorkflowError::Invalid(
                "Workflow must have at least one step".to_string(),
            ));
        }

        // Validate parameters
        Self::validate_parameters(workflow)?;

        // Validate step IDs are unique
        let mut step_ids = HashSet::new();
        for step in &workflow.steps {
            if step.id.is_empty() {
                return Err(WorkflowError::Invalid(
                    "Step id cannot be empty".to_string(),
                ));
            }
            if !step_ids.insert(&step.id) {
                return Err(WorkflowError::Invalid(format!(
                    "Duplicate step id: {}",
                    step.id
                )));
            }
            if step.name.is_empty() {
                return Err(WorkflowError::Invalid(format!(
                    "Step {} name cannot be empty",
                    step.id
                )));
            }
        }

        // Validate step dependencies
        for step in &workflow.steps {
            for dep in &step.dependencies {
                if !step_ids.contains(dep) {
                    return Err(WorkflowError::Invalid(format!(
                        "Step {} depends on non-existent step {}",
                        step.id, dep
                    )));
                }
            }
        }

        // Detect circular dependencies
        Self::detect_circular_dependencies(workflow)?;

        Ok(())
    }

    /// Detect circular dependencies in workflow steps
    ///
    /// Uses depth-first search to detect cycles in the dependency graph.
    /// Returns an error if any circular dependency is found.
    fn detect_circular_dependencies(workflow: &Workflow) -> WorkflowResult<()> {
        let step_map: HashMap<&String, &crate::models::WorkflowStep> =
            workflow.steps.iter().map(|s| (&s.id, s)).collect();

        // For each step, perform DFS to detect cycles
        for start_step in &workflow.steps {
            let mut visited = HashSet::new();
            let mut rec_stack = HashSet::new();

            Self::dfs_detect_cycle(&step_map, &start_step.id, &mut visited, &mut rec_stack)?;
        }

        Ok(())
    }

    /// Depth-first search to detect cycles in dependency graph
    fn dfs_detect_cycle(
        step_map: &HashMap<&String, &crate::models::WorkflowStep>,
        step_id: &String,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> WorkflowResult<()> {
        visited.insert(step_id.clone());
        rec_stack.insert(step_id.clone());

        if let Some(step) = step_map.get(step_id) {
            for dep in &step.dependencies {
                if !visited.contains(dep) {
                    Self::dfs_detect_cycle(step_map, dep, visited, rec_stack)?;
                } else if rec_stack.contains(dep) {
                    return Err(WorkflowError::Invalid(format!(
                        "Circular dependency detected: {} -> {}",
                        step_id, dep
                    )));
                }
            }
        }

        rec_stack.remove(step_id);
        Ok(())
    }

    /// Validate workflow parameters
    fn validate_parameters(workflow: &Workflow) -> WorkflowResult<()> {
        let mut seen_names = HashSet::new();

        for param in &workflow.parameters {
            // Check for duplicate names
            if !seen_names.insert(&param.name) {
                return Err(WorkflowError::Invalid(format!(
                    "Duplicate parameter name: {}",
                    param.name
                )));
            }

            // Validate parameter name
            if param.name.is_empty() {
                return Err(WorkflowError::Invalid(
                    "Parameter name cannot be empty".to_string(),
                ));
            }

            // Check that parameter name is valid (alphanumeric, underscore, hyphen)
            if !param.name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
                return Err(WorkflowError::Invalid(format!(
                    "Invalid parameter name: {}. Must contain only alphanumeric characters, underscores, and hyphens",
                    param.name
                )));
            }

            // Validate parameter type
            match param.param_type.as_str() {
                "string" | "number" | "boolean" | "object" | "array" => {}
                _ => {
                    return Err(WorkflowError::Invalid(format!(
                        "Invalid parameter type for '{}': {}. Must be one of: string, number, boolean, object, array",
                        param.name, param.param_type
                    )));
                }
            }

            // Validate default value matches type if provided
            if let Some(default) = &param.default {
                Self::validate_parameter_type(&param.name, &param.param_type, default)?;
            }

            // Required parameters should not have defaults
            if param.required && param.default.is_some() {
                return Err(WorkflowError::Invalid(format!(
                    "Required parameter '{}' cannot have a default value",
                    param.name
                )));
            }
        }

        Ok(())
    }

    /// Validate that a value matches a parameter type
    fn validate_parameter_type(
        param_name: &str,
        param_type: &str,
        value: &serde_json::Value,
    ) -> WorkflowResult<()> {
        let matches = match param_type {
            "string" => value.is_string(),
            "number" => value.is_number(),
            "boolean" => value.is_boolean(),
            "object" => value.is_object(),
            "array" => value.is_array(),
            _ => false,
        };

        if !matches {
            return Err(WorkflowError::Invalid(format!(
                "Default value for parameter '{}' does not match type '{}'",
                param_name, param_type
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_yaml() {
        let yaml = r#"
id: test-workflow
name: Test Workflow
description: A test workflow
parameters: []
steps:
  - id: step1
    name: Step 1
    step_type:
      type: agent
      agent_id: test-agent
      task: test-task
    dependencies: []
    approval_required: false
    on_error:
      action: fail
    config: {}
config:
  timeout_ms: 5000
"#;

        let result = WorkflowParser::parse_yaml(yaml);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_invalid_yaml_missing_id() {
        let yaml = r#"
name: Test Workflow
description: A test workflow
parameters: []
steps: []
config: {}
"#;

        let result = WorkflowParser::parse_yaml(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_missing_dependency() {
        let yaml = r#"
id: test-workflow
name: Test Workflow
description: A test workflow
parameters: []
steps:
  - id: step1
    name: Step 1
    type: agent
    agent_id: test-agent
    task: test-task
    dependencies: [non-existent]
    approval_required: false
    on_error:
      action: fail
    config: {}
config: {}
"#;

        let result = WorkflowParser::parse_yaml(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_circular_dependency() {
        let yaml = r#"
id: test-workflow
name: Test Workflow
description: A test workflow
parameters: []
steps:
  - id: step1
    name: Step 1
    type: agent
    agent_id: test-agent
    task: test-task
    dependencies: [step2]
    approval_required: false
    on_error:
      action: fail
    config: {}
  - id: step2
    name: Step 2
    type: agent
    agent_id: test-agent
    task: test-task
    dependencies: [step1]
    approval_required: false
    on_error:
      action: fail
    config: {}
config: {}
"#;

        let result = WorkflowParser::parse_yaml(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_empty_step_id() {
        let yaml = r#"
id: test-workflow
name: Test Workflow
description: A test workflow
parameters: []
steps:
  - id: ""
    name: Step 1
    type: agent
    agent_id: test-agent
    task: test-task
    dependencies: []
    approval_required: false
    on_error:
      action: fail
    config: {}
config: {}
"#;

        let result = WorkflowParser::parse_yaml(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_empty_step_name() {
        let yaml = r#"
id: test-workflow
name: Test Workflow
description: A test workflow
parameters: []
steps:
  - id: step1
    name: ""
    type: agent
    agent_id: test-agent
    task: test-task
    dependencies: []
    approval_required: false
    on_error:
      action: fail
    config: {}
config: {}
"#;

        let result = WorkflowParser::parse_yaml(yaml);
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    /// **Feature: ricecoder-workflows, Property 1: Workflow Parsing Round Trip**
    /// **Validates: Requirements 1.1**
    ///
    /// For any valid YAML workflow definition, parsing the definition and then
    /// serializing the resulting Workflow object SHALL produce an equivalent definition.
    #[test]
    fn prop_workflow_parsing_round_trip() {
        proptest!(|(
            id in "[a-z0-9_-]{1,20}",
            name in "[a-zA-Z0-9]{1,30}",
        )| {
            // Create a workflow programmatically to ensure it's valid
            let workflow = crate::models::Workflow {
                id: id.clone(),
                name: name.clone(),
                description: "Test workflow".to_string(),
                parameters: vec![],
                steps: vec![
                    crate::models::WorkflowStep {
                        id: "step1".to_string(),
                        name: "Step 1".to_string(),
                        step_type: crate::models::StepType::Agent(crate::models::AgentStep {
                            agent_id: "test-agent".to_string(),
                            task: "test-task".to_string(),
                        }),
                        config: crate::models::StepConfig {
                            config: serde_json::json!({}),
                        },
                        dependencies: vec![],
                        approval_required: false,
                        on_error: crate::models::ErrorAction::Fail,
                        risk_score: None,
                        risk_factors: crate::models::RiskFactors::default(),
                    },
                ],
                config: crate::models::WorkflowConfig {
                    timeout_ms: Some(5000),
                    max_parallel: None,
                },
            };

            // Serialize to YAML
            let serialized = WorkflowParser::to_yaml(&workflow);
            prop_assert!(serialized.is_ok(), "Failed to serialize workflow to YAML");

            let serialized = serialized.unwrap();

            // Parse the serialized YAML
            let reparsed = WorkflowParser::parse_yaml(&serialized);
            prop_assert!(reparsed.is_ok(), "Failed to reparse serialized YAML");

            let reparsed = reparsed.unwrap();

            // Verify the reparsed workflow matches the original
            prop_assert_eq!(&workflow.id, &reparsed.id, "Workflow ID mismatch");
            prop_assert_eq!(&workflow.name, &reparsed.name, "Workflow name mismatch");
            prop_assert_eq!(workflow.steps.len(), reparsed.steps.len(), "Step count mismatch");

            for (original_step, reparsed_step) in workflow.steps.iter().zip(reparsed.steps.iter()) {
                prop_assert_eq!(&original_step.id, &reparsed_step.id, "Step ID mismatch");
                prop_assert_eq!(&original_step.name, &reparsed_step.name, "Step name mismatch");
                prop_assert_eq!(&original_step.dependencies, &reparsed_step.dependencies, "Dependencies mismatch");
            }
        });
    }

    /// **Feature: ricecoder-workflows, Property 6: Workflow Validation**
    /// **Validates: Requirements 1.6**
    ///
    /// For any invalid workflow definition, the validation system SHALL identify
    /// and report all syntax errors before execution begins.
    #[test]
    fn prop_workflow_validation_rejects_invalid() {
        proptest!(|(
            missing_field in 0..3u32,
        )| {
            let yaml = match missing_field {
                0 => r#"
name: Test Workflow
description: A test workflow
steps:
  - id: step1
    name: Step 1
    type: agent
    agent_id: test-agent
    task: test-task
    dependencies: []
    approval_required: false
    on_error:
      action: fail
    config: {}
config: {}
"#,
                1 => r#"
id: test-workflow
description: A test workflow
steps:
  - id: step1
    name: Step 1
    type: agent
    agent_id: test-agent
    task: test-task
    dependencies: []
    approval_required: false
    on_error:
      action: fail
    config: {}
config: {}
"#,
                _ => r#"
id: test-workflow
name: Test Workflow
description: A test workflow
steps: []
config: {}
"#,
            };

            let result = WorkflowParser::parse_yaml(yaml);
            prop_assert!(result.is_err(), "Should reject invalid workflow");
        });
    }
}


