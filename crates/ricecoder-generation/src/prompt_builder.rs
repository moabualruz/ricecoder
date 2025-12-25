//! Prompt building for code generation
//!
//! Provides structures for building prompts sent to AI providers for code generation.

use serde::{Deserialize, Serialize};

/// Generated prompt ready to send to an AI provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedPrompt {
    /// System prompt with context and instructions
    pub system_prompt: String,
    /// User prompt with the generation request
    pub user_prompt: String,
}

impl GeneratedPrompt {
    /// Creates a new generated prompt
    pub fn new(system_prompt: impl Into<String>, user_prompt: impl Into<String>) -> Self {
        Self {
            system_prompt: system_prompt.into(),
            user_prompt: user_prompt.into(),
        }
    }
}

/// Context for building prompts
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromptContext {
    /// Project name
    pub project_name: Option<String>,
    /// Project description
    pub project_description: Option<String>,
    /// Language/framework hints
    pub language: Option<String>,
    /// Additional context files
    pub context_files: Vec<ContextFile>,
    /// User-provided instructions
    pub instructions: Option<String>,
}

/// A file included for context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextFile {
    /// File path
    pub path: String,
    /// File content
    pub content: String,
}

/// Governance rules for code generation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GovernanceRules {
    /// Allowed file patterns
    pub allowed_patterns: Vec<String>,
    /// Denied file patterns
    pub denied_patterns: Vec<String>,
    /// Maximum file size in bytes
    pub max_file_size: Option<usize>,
    /// Required code conventions
    pub conventions: Vec<String>,
}

/// Builds prompts for code generation
#[derive(Debug, Clone, Default)]
pub struct PromptBuilder {
    /// Context for building prompts
    context: PromptContext,
    /// Governance rules to apply
    governance: GovernanceRules,
}

impl PromptBuilder {
    /// Creates a new prompt builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the prompt context
    pub fn with_context(mut self, context: PromptContext) -> Self {
        self.context = context;
        self
    }

    /// Sets governance rules
    pub fn with_governance(mut self, governance: GovernanceRules) -> Self {
        self.governance = governance;
        self
    }

    /// Builds a prompt for the given request
    pub fn build(&self, request: &str) -> GeneratedPrompt {
        let mut system_parts = vec![
            "You are an expert code generator.".to_string(),
            "Generate high-quality, well-documented code.".to_string(),
        ];

        if let Some(lang) = &self.context.language {
            system_parts.push(format!("Primary language: {}", lang));
        }

        if let Some(name) = &self.context.project_name {
            system_parts.push(format!("Project: {}", name));
        }

        if !self.governance.conventions.is_empty() {
            system_parts.push(format!(
                "Follow these conventions: {}",
                self.governance.conventions.join(", ")
            ));
        }

        let mut user_parts = vec![request.to_string()];

        if let Some(instructions) = &self.context.instructions {
            user_parts.push(format!("\nAdditional instructions: {}", instructions));
        }

        GeneratedPrompt {
            system_prompt: system_parts.join("\n"),
            user_prompt: user_parts.join("\n"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generated_prompt_new() {
        let prompt = GeneratedPrompt::new("system", "user");
        assert_eq!(prompt.system_prompt, "system");
        assert_eq!(prompt.user_prompt, "user");
    }

    #[test]
    fn test_prompt_builder_basic() {
        let builder = PromptBuilder::new();
        let prompt = builder.build("Generate a hello world function");
        assert!(prompt.system_prompt.contains("code generator"));
        assert!(prompt.user_prompt.contains("hello world"));
    }

    #[test]
    fn test_prompt_builder_with_context() {
        let context = PromptContext {
            project_name: Some("test-project".to_string()),
            language: Some("rust".to_string()),
            ..Default::default()
        };
        let builder = PromptBuilder::new().with_context(context);
        let prompt = builder.build("Generate code");
        assert!(prompt.system_prompt.contains("rust"));
        assert!(prompt.system_prompt.contains("test-project"));
    }
}
