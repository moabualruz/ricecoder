//! Generation manager for orchestrating code generation workflows
//!
//! Provides high-level orchestration of the code generation pipeline,
//! including prompt building, AI generation, validation, and output writing.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{
    error::GenerationError,
    models::GeneratedFile,
    prompt_builder::{GeneratedPrompt, GovernanceRules, PromptBuilder, PromptContext},
    CodeGenerator,
};

/// Configuration for the generation manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationManagerConfig {
    /// Model to use for generation
    pub model: String,
    /// Temperature for sampling (0.0 to 2.0)
    pub temperature: f32,
    /// Maximum tokens to generate
    pub max_tokens: usize,
    /// Output directory for generated files
    pub output_dir: PathBuf,
    /// Whether to validate generated code
    pub validate: bool,
    /// Whether to run formatting on generated code
    pub format: bool,
}

impl Default for GenerationManagerConfig {
    fn default() -> Self {
        Self {
            model: "gpt-4".to_string(),
            temperature: 0.7,
            max_tokens: 4096,
            output_dir: PathBuf::from("."),
            validate: true,
            format: true,
        }
    }
}

/// Orchestrates code generation workflows
pub struct GenerationManager {
    /// Configuration for the manager
    config: GenerationManagerConfig,
    /// Code generator instance
    code_generator: CodeGenerator,
    /// Prompt builder instance
    prompt_builder: PromptBuilder,
}

impl GenerationManager {
    /// Creates a new generation manager with default configuration
    pub fn new() -> Self {
        Self {
            config: GenerationManagerConfig::default(),
            code_generator: CodeGenerator::new(),
            prompt_builder: PromptBuilder::new(),
        }
    }

    /// Creates a new generation manager with custom configuration
    pub fn with_config(config: GenerationManagerConfig) -> Self {
        Self {
            config,
            code_generator: CodeGenerator::new(),
            prompt_builder: PromptBuilder::new(),
        }
    }

    /// Sets the prompt context
    pub fn with_context(mut self, context: PromptContext) -> Self {
        self.prompt_builder = self.prompt_builder.with_context(context);
        self
    }

    /// Sets governance rules
    pub fn with_governance(mut self, governance: GovernanceRules) -> Self {
        self.prompt_builder = self.prompt_builder.with_governance(governance);
        self
    }

    /// Builds a prompt for the given request
    pub fn build_prompt(&self, request: &str) -> GeneratedPrompt {
        self.prompt_builder.build(request)
    }

    /// Gets the current configuration
    pub fn config(&self) -> &GenerationManagerConfig {
        &self.config
    }

    /// Gets the code generator
    pub fn code_generator(&self) -> &CodeGenerator {
        &self.code_generator
    }
}

impl Default for GenerationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generation_manager_new() {
        let manager = GenerationManager::new();
        assert_eq!(manager.config.model, "gpt-4");
        assert_eq!(manager.config.temperature, 0.7);
    }

    #[test]
    fn test_generation_manager_with_config() {
        let config = GenerationManagerConfig {
            model: "gpt-3.5-turbo".to_string(),
            temperature: 0.5,
            ..Default::default()
        };
        let manager = GenerationManager::with_config(config);
        assert_eq!(manager.config.model, "gpt-3.5-turbo");
        assert_eq!(manager.config.temperature, 0.5);
    }

    #[test]
    fn test_build_prompt() {
        let manager = GenerationManager::new();
        let prompt = manager.build_prompt("Generate a hello world function");
        assert!(prompt.user_prompt.contains("hello world"));
    }
}
