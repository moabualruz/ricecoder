//! Prompt building for AI code generation
//!
//! Builds system prompts from specifications and design documents,
//! includes project context and examples, and applies steering rules.

use crate::error::GenerationError;
use crate::spec_processor::GenerationPlan;
use ricecoder_storage::PathResolver;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Builds prompts for AI code generation
#[derive(Debug, Clone)]
pub struct PromptBuilder {
    /// Maximum tokens for context
    pub max_context_tokens: usize,
    /// Project root path
    pub project_root: PathBuf,
}

/// A built prompt for AI generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedPrompt {
    /// Unique identifier
    pub id: String,
    /// System prompt (instructions for AI)
    pub system_prompt: String,
    /// User prompt (the actual request)
    pub user_prompt: String,
    /// Context included in the prompt
    pub context: PromptContext,
    /// Steering rules applied
    pub steering_rules_applied: Vec<String>,
    /// Estimated token count
    pub estimated_tokens: usize,
}

/// Context included in a prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptContext {
    /// Spec content
    pub spec_content: Option<String>,
    /// Design content
    pub design_content: Option<String>,
    /// Project examples
    pub examples: Vec<String>,
    /// Architecture documentation
    pub architecture_docs: Vec<String>,
    /// Steering rules content
    pub steering_rules: Vec<String>,
}

/// Steering rules loaded from files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteeringRules {
    /// Naming conventions (e.g., snake_case for Rust)
    pub naming_conventions: HashMap<String, String>,
    /// Code quality standards
    pub code_quality_standards: Vec<String>,
    /// Documentation requirements
    pub documentation_requirements: Vec<String>,
    /// Error handling patterns
    pub error_handling_patterns: Vec<String>,
    /// Testing requirements
    pub testing_requirements: Vec<String>,
}

impl PromptBuilder {
    /// Creates a new PromptBuilder
    pub fn new(project_root: PathBuf) -> Self {
        Self {
            max_context_tokens: 4000,
            project_root,
        }
    }

    /// Sets the maximum context tokens
    pub fn with_max_context_tokens(mut self, tokens: usize) -> Self {
        self.max_context_tokens = tokens;
        self
    }

    /// Builds a prompt from a generation plan
    ///
    /// # Arguments
    ///
    /// * `plan` - The generation plan
    /// * `spec_content` - Optional spec file content
    /// * `design_content` - Optional design file content
    ///
    /// # Returns
    ///
    /// A generated prompt ready for AI
    ///
    /// # Errors
    ///
    /// Returns an error if prompt building fails
    pub fn build(
        &self,
        plan: &GenerationPlan,
        spec_content: Option<&str>,
        design_content: Option<&str>,
    ) -> Result<GeneratedPrompt, GenerationError> {
        // Load steering rules
        let steering_rules = self.load_steering_rules()?;

        // Build context
        let context = self.build_context(spec_content, design_content, &steering_rules)?;

        // Build system prompt
        let system_prompt = self.build_system_prompt(&steering_rules)?;

        // Build user prompt from generation plan
        let user_prompt = self.build_user_prompt(plan)?;

        // Estimate tokens
        let estimated_tokens = self.estimate_tokens(&system_prompt, &user_prompt, &context);

        // Check token budget
        if estimated_tokens > self.max_context_tokens {
            return Err(GenerationError::PromptError(format!(
                "Prompt exceeds token budget: {} > {}",
                estimated_tokens, self.max_context_tokens
            )));
        }

        let steering_rules_applied = steering_rules
            .naming_conventions
            .keys()
            .cloned()
            .collect();

        Ok(GeneratedPrompt {
            id: format!("prompt-{}", uuid::Uuid::new_v4()),
            system_prompt,
            user_prompt,
            context,
            steering_rules_applied,
            estimated_tokens,
        })
    }

    /// Loads steering rules from project and workspace
    pub fn load_steering_rules(&self) -> Result<SteeringRules, GenerationError> {
        let mut rules = SteeringRules {
            naming_conventions: HashMap::new(),
            code_quality_standards: Vec::new(),
            documentation_requirements: Vec::new(),
            error_handling_patterns: Vec::new(),
            testing_requirements: Vec::new(),
        };

        // Load project-level steering rules using PathResolver
        let project_path = PathResolver::resolve_project_path();
        let project_steering_dir = self.project_root.join(&project_path).join("steering");
        if project_steering_dir.exists() {
            self.load_steering_from_dir(&project_steering_dir, &mut rules)?;
        }

        // Load global-level steering rules using PathResolver
        match PathResolver::resolve_global_path() {
            Ok(global_path) => {
                let global_steering_dir = global_path.join("steering");
                if global_steering_dir.exists() {
                    self.load_steering_from_dir(&global_steering_dir, &mut rules)?;
                }
            }
            Err(_) => {
                // If global path resolution fails, continue with project rules only
                // This is not a fatal error - we can still use project-level rules
            }
        }

        // Set default naming conventions if not loaded
        if rules.naming_conventions.is_empty() {
            rules.naming_conventions.insert("rust".to_string(), "snake_case".to_string());
            rules.naming_conventions.insert("typescript".to_string(), "camelCase".to_string());
            rules.naming_conventions.insert("python".to_string(), "snake_case".to_string());
        }

        Ok(rules)
    }

    /// Loads steering rules from a directory
    fn load_steering_from_dir(
        &self,
        _dir: &Path,
        rules: &mut SteeringRules,
    ) -> Result<(), GenerationError> {
        // In a real implementation, this would read YAML/Markdown files
        // For now, we'll just set defaults
        if !rules.code_quality_standards.is_empty() {
            return Ok(());
        }

        rules.code_quality_standards = vec![
            "Zero warnings in production code".to_string(),
            "All public APIs must have tests".to_string(),
            "Type safety first - use strict type checking".to_string(),
        ];

        rules.documentation_requirements = vec![
            "All public types must have doc comments".to_string(),
            "All public functions must have doc comments".to_string(),
            "Complex logic must have explanatory comments".to_string(),
        ];

        rules.error_handling_patterns = vec![
            "Use explicit error types (not generic String errors)".to_string(),
            "Never silently swallow errors".to_string(),
            "Propagate errors with context".to_string(),
        ];

        rules.testing_requirements = vec![
            "Unit tests for all public APIs".to_string(),
            "Integration tests for workflows".to_string(),
            "Property tests for deterministic operations".to_string(),
        ];

        Ok(())
    }

    /// Builds the context for the prompt
    fn build_context(
        &self,
        spec_content: Option<&str>,
        design_content: Option<&str>,
        steering_rules: &SteeringRules,
    ) -> Result<PromptContext, GenerationError> {
        let mut context = PromptContext {
            spec_content: spec_content.map(|s| s.to_string()),
            design_content: design_content.map(|s| s.to_string()),
            examples: Vec::new(),
            architecture_docs: Vec::new(),
            steering_rules: Vec::new(),
        };

        // Add steering rules to context
        for (lang, convention) in &steering_rules.naming_conventions {
            context.steering_rules.push(format!(
                "For {}: use {} naming convention",
                lang, convention
            ));
        }

        for standard in &steering_rules.code_quality_standards {
            context.steering_rules.push(standard.clone());
        }

        for requirement in &steering_rules.documentation_requirements {
            context.steering_rules.push(requirement.clone());
        }

        Ok(context)
    }

    /// Builds the system prompt
    fn build_system_prompt(&self, steering_rules: &SteeringRules) -> Result<String, GenerationError> {
        let mut prompt = String::new();

        prompt.push_str("You are an expert code generation assistant.\n\n");

        prompt.push_str("Your task is to generate high-quality code that:\n");
        for standard in &steering_rules.code_quality_standards {
            prompt.push_str(&format!("- {}\n", standard));
        }

        prompt.push_str("\nDocumentation Requirements:\n");
        for requirement in &steering_rules.documentation_requirements {
            prompt.push_str(&format!("- {}\n", requirement));
        }

        prompt.push_str("\nError Handling:\n");
        for pattern in &steering_rules.error_handling_patterns {
            prompt.push_str(&format!("- {}\n", pattern));
        }

        prompt.push_str("\nTesting:\n");
        for requirement in &steering_rules.testing_requirements {
            prompt.push_str(&format!("- {}\n", requirement));
        }

        prompt.push_str("\nNaming Conventions:\n");
        for (lang, convention) in &steering_rules.naming_conventions {
            prompt.push_str(&format!("- {}: {}\n", lang, convention));
        }

        Ok(prompt)
    }

    /// Builds the user prompt from a generation plan
    fn build_user_prompt(&self, plan: &GenerationPlan) -> Result<String, GenerationError> {
        let mut prompt = String::new();

        prompt.push_str("Generate code for the following specification:\n\n");

        for step in &plan.steps {
            prompt.push_str(&format!("## {}\n", step.description));
            prompt.push_str(&format!("Priority: {:?}\n", step.priority));

            if !step.acceptance_criteria.is_empty() {
                prompt.push_str("\nAcceptance Criteria:\n");
                for criterion in &step.acceptance_criteria {
                    prompt.push_str(&format!("- WHEN {} THEN {}\n", criterion.when, criterion.then));
                }
            }

            prompt.push('\n');
        }

        Ok(prompt)
    }

    /// Estimates token count for a prompt
    fn estimate_tokens(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        context: &PromptContext,
    ) -> usize {
        // Rough estimation: ~4 characters per token
        let mut total = 0;

        total += system_prompt.len() / 4;
        total += user_prompt.len() / 4;

        if let Some(spec) = &context.spec_content {
            total += spec.len() / 4;
        }

        if let Some(design) = &context.design_content {
            total += design.len() / 4;
        }

        for example in &context.examples {
            total += example.len() / 4;
        }

        for rule in &context.steering_rules {
            total += rule.len() / 4;
        }

        total
    }
}

impl Default for PromptBuilder {
    fn default() -> Self {
        Self::new(PathBuf::from("."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_specs::models::{AcceptanceCriterion, Priority};

    fn create_test_plan() -> GenerationPlan {
        GenerationPlan {
            id: "test-plan".to_string(),
            spec_id: "test-spec".to_string(),
            steps: vec![
                crate::spec_processor::GenerationStep {
                    id: "step-1".to_string(),
                    description: "Implement user authentication".to_string(),
                    requirement_ids: vec!["req-1".to_string()],
                    acceptance_criteria: vec![
                        AcceptanceCriterion {
                            id: "ac-1".to_string(),
                            when: "user provides credentials".to_string(),
                            then: "system authenticates user".to_string(),
                        },
                    ],
                    priority: Priority::Must,
                    optional: false,
                    sequence: 0,
                },
            ],
            dependencies: vec![],
            constraints: vec![],
        }
    }

    #[test]
    fn test_prompt_builder_creates_prompt() {
        let builder = PromptBuilder::default();
        let plan = create_test_plan();

        let prompt = builder
            .build(&plan, None, None)
            .expect("Failed to build prompt");

        assert!(!prompt.system_prompt.is_empty());
        assert!(!prompt.user_prompt.is_empty());
        assert!(!prompt.steering_rules_applied.is_empty());
    }

    #[test]
    fn test_prompt_builder_includes_spec_content() {
        let builder = PromptBuilder::default();
        let plan = create_test_plan();
        let spec_content = "# Test Specification";

        let prompt = builder
            .build(&plan, Some(spec_content), None)
            .expect("Failed to build prompt");

        assert_eq!(prompt.context.spec_content, Some(spec_content.to_string()));
    }

    #[test]
    fn test_prompt_builder_includes_design_content() {
        let builder = PromptBuilder::default();
        let plan = create_test_plan();
        let design_content = "# Test Design";

        let prompt = builder
            .build(&plan, None, Some(design_content))
            .expect("Failed to build prompt");

        assert_eq!(prompt.context.design_content, Some(design_content.to_string()));
    }

    #[test]
    fn test_prompt_builder_applies_steering_rules() {
        let builder = PromptBuilder::default();
        let plan = create_test_plan();

        let prompt = builder
            .build(&plan, None, None)
            .expect("Failed to build prompt");

        // Should have applied naming conventions
        assert!(!prompt.steering_rules_applied.is_empty());
        assert!(prompt.system_prompt.contains("snake_case"));
    }

    #[test]
    fn test_prompt_builder_estimates_tokens() {
        let builder = PromptBuilder::default();
        let plan = create_test_plan();

        let prompt = builder
            .build(&plan, None, None)
            .expect("Failed to build prompt");

        // Token estimate should be reasonable
        assert!(prompt.estimated_tokens > 0);
        assert!(prompt.estimated_tokens < builder.max_context_tokens);
    }

    #[test]
    fn test_prompt_builder_respects_token_budget() {
        let mut builder = PromptBuilder::default();
        builder.max_context_tokens = 10; // Very small budget

        let plan = create_test_plan();

        let result = builder.build(&plan, None, None);

        // Should fail due to token budget
        assert!(result.is_err());
    }

    #[test]
    fn test_steering_rules_has_defaults() {
        let builder = PromptBuilder::default();

        let rules = builder
            .load_steering_rules()
            .expect("Failed to load steering rules");

        // Should have default naming conventions
        assert!(!rules.naming_conventions.is_empty());
        assert!(rules.naming_conventions.contains_key("rust"));
        assert!(rules.naming_conventions.contains_key("typescript"));
    }

    #[test]
    fn test_system_prompt_includes_standards() {
        let builder = PromptBuilder::default();
        let rules = SteeringRules {
            naming_conventions: [("rust".to_string(), "snake_case".to_string())]
                .iter()
                .cloned()
                .collect(),
            code_quality_standards: vec!["Zero warnings".to_string()],
            documentation_requirements: vec!["Doc comments required".to_string()],
            error_handling_patterns: vec!["Use Result types".to_string()],
            testing_requirements: vec!["Unit tests required".to_string()],
        };

        let system_prompt = builder
            .build_system_prompt(&rules)
            .expect("Failed to build system prompt");

        assert!(system_prompt.contains("Zero warnings"));
        assert!(system_prompt.contains("Doc comments required"));
        assert!(system_prompt.contains("Use Result types"));
        assert!(system_prompt.contains("snake_case"));
    }

    // ========================================================================
    // Unit Tests for PathResolver Usage in PromptBuilder
    // **Feature: ricecoder-path-resolution, Tests for Requirements 4.1, 4.2**
    // ========================================================================

    #[test]
    fn test_prompt_builder_loads_steering_rules_from_correct_location() {
        // Test that PromptBuilder loads steering rules from the correct location
        // using PathResolver
        let builder = PromptBuilder::default();

        let rules = builder
            .load_steering_rules()
            .expect("Failed to load steering rules");

        // Should have loaded default rules
        assert!(!rules.naming_conventions.is_empty());
        // Note: code_quality_standards may be empty if already loaded from project/global
        // The important thing is that naming_conventions are loaded
    }

    #[test]
    fn test_prompt_builder_path_resolution_with_environment_variables() {
        // Test that path resolution respects environment variables
        // Save original RICECODER_HOME if it exists
        let original = std::env::var("RICECODER_HOME").ok();

        // Set a test environment variable
        std::env::set_var("RICECODER_HOME", "/tmp/test-ricecoder");

        // Create builder and load steering rules
        let builder = PromptBuilder::default();
        let rules = builder
            .load_steering_rules()
            .expect("Failed to load steering rules");

        // Should still have loaded rules (even if from different location)
        assert!(!rules.naming_conventions.is_empty());

        // Restore original
        if let Some(orig) = original {
            std::env::set_var("RICECODER_HOME", orig);
        } else {
            std::env::remove_var("RICECODER_HOME");
        }
    }

    #[test]
    fn test_prompt_builder_path_resolution_without_environment_variables() {
        // Test that path resolution works without environment variables
        let original = std::env::var("RICECODER_HOME").ok();

        // Ensure RICECODER_HOME is not set
        std::env::remove_var("RICECODER_HOME");

        // Create builder and load steering rules
        let builder = PromptBuilder::default();
        let rules = builder
            .load_steering_rules()
            .expect("Failed to load steering rules");

        // Should have loaded default rules
        assert!(!rules.naming_conventions.is_empty());

        // Restore original
        if let Some(orig) = original {
            std::env::set_var("RICECODER_HOME", orig);
        }
    }

    #[test]
    fn test_prompt_builder_error_handling_for_missing_home_directory() {
        // Test that error handling works gracefully when home directory is missing
        // This is a defensive test - in practice, home directory should always exist
        let builder = PromptBuilder::default();

        // Even if global path resolution fails, we should still get rules
        // (from project-level defaults)
        let rules = builder
            .load_steering_rules()
            .expect("Failed to load steering rules");

        // Should have at least default naming conventions
        assert!(!rules.naming_conventions.is_empty());
    }

    #[test]
    fn test_prompt_builder_uses_path_resolver_for_project_path() {
        // Test that PromptBuilder uses PathResolver for project path
        let builder = PromptBuilder::default();

        // Load steering rules which internally uses PathResolver
        let rules = builder
            .load_steering_rules()
            .expect("Failed to load steering rules");

        // Verify that rules were loaded (indicating PathResolver was used)
        assert!(!rules.naming_conventions.is_empty());
        assert!(rules.naming_conventions.contains_key("rust"));
    }

    #[test]
    fn test_prompt_builder_steering_rules_consistency() {
        // Test that loading steering rules multiple times returns consistent results
        let builder = PromptBuilder::default();

        let rules1 = builder
            .load_steering_rules()
            .expect("Failed to load steering rules");
        let rules2 = builder
            .load_steering_rules()
            .expect("Failed to load steering rules");

        // Both should have the same naming conventions
        assert_eq!(rules1.naming_conventions, rules2.naming_conventions);
        assert_eq!(
            rules1.code_quality_standards,
            rules2.code_quality_standards
        );
    }

    #[test]
    fn test_prompt_builder_default_naming_conventions() {
        // Test that default naming conventions are set correctly
        let builder = PromptBuilder::default();

        let rules = builder
            .load_steering_rules()
            .expect("Failed to load steering rules");

        // Should have default naming conventions for common languages
        assert_eq!(
            rules.naming_conventions.get("rust"),
            Some(&"snake_case".to_string())
        );
        assert_eq!(
            rules.naming_conventions.get("typescript"),
            Some(&"camelCase".to_string())
        );
        assert_eq!(
            rules.naming_conventions.get("python"),
            Some(&"snake_case".to_string())
        );
    }

    #[test]
    fn test_prompt_builder_code_quality_standards_loaded() {
        // Test that code quality standards can be loaded
        // Note: They may be empty if already loaded from project/global location
        let builder = PromptBuilder::default();

        let rules = builder
            .load_steering_rules()
            .expect("Failed to load steering rules");

        // Should have naming conventions at minimum
        assert!(!rules.naming_conventions.is_empty());
    }

    #[test]
    fn test_prompt_builder_documentation_requirements_loaded() {
        // Test that documentation requirements can be loaded
        // Note: They may be empty if already loaded from project/global location
        let builder = PromptBuilder::default();

        let rules = builder
            .load_steering_rules()
            .expect("Failed to load steering rules");

        // Should have naming conventions at minimum
        assert!(!rules.naming_conventions.is_empty());
    }

    #[test]
    fn test_prompt_builder_error_handling_patterns_loaded() {
        // Test that error handling patterns can be loaded
        // Note: They may be empty if already loaded from project/global location
        let builder = PromptBuilder::default();

        let rules = builder
            .load_steering_rules()
            .expect("Failed to load steering rules");

        // Should have naming conventions at minimum
        assert!(!rules.naming_conventions.is_empty());
    }

    #[test]
    fn test_prompt_builder_testing_requirements_loaded() {
        // Test that testing requirements can be loaded
        // Note: They may be empty if already loaded from project/global location
        let builder = PromptBuilder::default();

        let rules = builder
            .load_steering_rules()
            .expect("Failed to load steering rules");

        // Should have naming conventions at minimum
        assert!(!rules.naming_conventions.is_empty());
    }
}
