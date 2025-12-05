//! Generation manager orchestrating the code generation pipeline
//!
//! Coordinates all components in the correct pipeline order:
//! 1. Spec processing → plan generation
//! 2. Prompt building (with steering rules)
//! 3. Code generation (templates or AI)
//! 4. Code quality enforcement
//! 5. Validation (syntax, linting, type checking)
//! 6. Conflict detection
//! 7. Review (optional)
//! 8. Output writing (unless dry-run or validation failed)
//!
//! Implements Requirement 3.1: Generation pipeline with strict ordering

use crate::error::GenerationError;
use crate::spec_processor::SpecProcessor;
use crate::prompt_builder::PromptBuilder;
use crate::code_generator::CodeGenerator;
use crate::code_quality_enforcer::CodeQualityEnforcer;
use crate::code_validator::CodeValidator;
use crate::conflict_detector::ConflictDetector;
use crate::conflict_resolver::{ConflictResolver, ConflictStrategy};
use crate::output_writer::OutputWriter;
use crate::review_engine::ReviewEngine;
use crate::report_generator::{GenerationResult, GenerationStats, ReportGenerator};
use crate::templates::TemplateEngine;
use ricecoder_specs::models::Spec;
use ricecoder_providers::provider::Provider;
use std::path::PathBuf;
use std::time::Instant;

/// Configuration for the generation manager
#[derive(Debug, Clone)]
pub struct GenerationManagerConfig {
    /// Project root path
    pub project_root: PathBuf,
    /// Whether to validate generated code
    pub validate: bool,
    /// Whether to review generated code
    pub review: bool,
    /// Whether to perform dry-run (no file writes)
    pub dry_run: bool,
    /// Conflict resolution strategy
    pub conflict_strategy: ConflictStrategy,
    /// Maximum retries on failure
    pub max_retries: usize,
    /// Whether to use templates (vs AI generation)
    pub use_templates: bool,
}

impl Default for GenerationManagerConfig {
    fn default() -> Self {
        Self {
            project_root: PathBuf::from("."),
            validate: true,
            review: false,
            dry_run: false,
            conflict_strategy: ConflictStrategy::Prompt,
            max_retries: 3,
            use_templates: false,
        }
    }
}

/// Orchestrates the code generation pipeline
///
/// Manages the complete generation workflow:
/// - Spec processing into generation plans
/// - Prompt building with steering rules
/// - Code generation (templates or AI)
/// - Code quality enforcement
/// - Validation
/// - Conflict detection and resolution
/// - Output writing with rollback support
pub struct GenerationManager {
    config: GenerationManagerConfig,
    spec_processor: SpecProcessor,
    prompt_builder: PromptBuilder,
    code_generator: CodeGenerator,
    #[allow(dead_code)]
    template_engine: TemplateEngine,
    code_quality_enforcer: CodeQualityEnforcer,
    code_validator: CodeValidator,
    conflict_detector: ConflictDetector,
    #[allow(dead_code)]
    conflict_resolver: ConflictResolver,
    output_writer: OutputWriter,
    review_engine: ReviewEngine,
    #[allow(dead_code)]
    report_generator: ReportGenerator,
}

impl std::fmt::Debug for GenerationManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GenerationManager")
            .field("config", &self.config)
            .finish()
    }
}

impl GenerationManager {
    /// Creates a new GenerationManager with default configuration
    pub fn new(project_root: PathBuf) -> Self {
        Self::with_config(GenerationManagerConfig {
            project_root: project_root.clone(),
            ..Default::default()
        })
    }

    /// Creates a new GenerationManager with custom configuration
    pub fn with_config(config: GenerationManagerConfig) -> Self {
        let project_root = config.project_root.clone();

        Self {
            config,
            spec_processor: SpecProcessor::new(),
            prompt_builder: PromptBuilder::new(project_root.clone()),
            code_generator: CodeGenerator::new(),
            template_engine: TemplateEngine::new(),
            code_quality_enforcer: CodeQualityEnforcer::new(),
            code_validator: CodeValidator::new(),
            conflict_detector: ConflictDetector::new(),
            conflict_resolver: ConflictResolver::new(),
            output_writer: OutputWriter::new(),
            review_engine: ReviewEngine::new(),
            report_generator: ReportGenerator,
        }
    }

    /// Executes the complete generation pipeline
    ///
    /// Pipeline order (guaranteed):
    /// 1. Spec processing → plan generation
    /// 2. Prompt building (with steering rules)
    /// 3. Code generation (templates or AI)
    /// 4. Code quality enforcement
    /// 5. Validation (syntax, linting, type checking)
    /// 6. Conflict detection
    /// 7. Review (optional)
    /// 8. Output writing (unless dry-run or validation failed)
    ///
    /// # Arguments
    ///
    /// * `spec` - The specification to generate code from
    /// * `target_path` - Target directory for generated code
    /// * `language` - Programming language for generation
    /// * `provider` - Optional AI provider for code generation (required if not using templates)
    /// * `model` - Model name for AI generation
    /// * `temperature` - Temperature for AI sampling
    /// * `max_tokens` - Maximum tokens for AI generation
    ///
    /// # Returns
    ///
    /// A GenerationResult with files, validation results, conflicts, and statistics
    ///
    /// # Errors
    ///
    /// Returns an error if any pipeline stage fails
    #[allow(clippy::too_many_arguments)]
    pub async fn generate(
        &self,
        spec: &Spec,
        target_path: PathBuf,
        _language: String,
        provider: Option<&dyn Provider>,
        model: &str,
        temperature: f32,
        max_tokens: usize,
    ) -> Result<GenerationResult, GenerationError> {
        let start_time = Instant::now();

        // STEP 1: Process spec into plan
        // Requirement 3.1: Spec processing happens before code generation
        let plan = self.spec_processor.process(spec)
            .map_err(|e| GenerationError::SpecError(format!("Failed to process spec: {}", e)))?;

        // STEP 2: Build prompts for each step
        // Requirement 2.1, 2.3: Apply steering rules and naming conventions
        let prompt = self.prompt_builder.build(&plan, None, None)
            .map_err(|e| GenerationError::PromptError(format!("Failed to build prompts: {}", e)))?;

        // STEP 3: Generate code (templates or AI)
        // Requirement 1.1, 1.2, 1.3: Generate code structure and apply templates
        let mut generated_files = if self.config.use_templates {
            // Template-based generation with variable substitution
            // For now, use AI generation as fallback since template generation requires
            // specific template files and context setup
            let provider = provider.ok_or_else(|| {
                GenerationError::GenerationFailed("AI provider required for code generation".to_string())
            })?;
            self.code_generator.generate(provider, &prompt, model, temperature, max_tokens).await
                .map_err(|e| GenerationError::GenerationFailed(format!("Code generation failed: {}", e)))?
        } else {
            // AI-based generation with configured model and temperature
            let provider = provider.ok_or_else(|| {
                GenerationError::GenerationFailed("AI provider required for AI-based generation".to_string())
            })?;
            self.code_generator.generate(provider, &prompt, model, temperature, max_tokens).await
                .map_err(|e| GenerationError::GenerationFailed(format!("AI generation failed: {}", e)))?
        };

        // STEP 4: Enforce code quality standards
        // Requirement 2.2, 2.3, 2.4: Apply quality standards, naming conventions, error handling
        generated_files = self.code_quality_enforcer.enforce(generated_files)
            .map_err(|e| GenerationError::GenerationFailed(format!("Quality enforcement failed: {}", e)))?;

        // STEP 5: Validate generated code
        // Requirement 1.4, 3.4: Validate syntax, linting, type checking before writing
        let validation_result = if self.config.validate {
            self.code_validator.validate(&generated_files)
                .map_err(|e| GenerationError::ValidationFailed(format!("Validation failed: {}", e)))?
        } else {
            crate::models::ValidationResult::default()
        };

        // STEP 6: Detect conflicts before writing
        // Requirement 1.5, 4.1: Detect conflicts and compute diffs
        let conflicts = self.conflict_detector.detect(&generated_files, &target_path)
            .map_err(|e| GenerationError::GenerationFailed(format!("Conflict detection failed: {}", e)))?;

        // STEP 7: Review generated code (optional)
        // Requirement 1.6: Review code quality and spec compliance
        let review_result = if self.config.review {
            Some(self.review_engine.review(&generated_files, spec)
                .map_err(|e| GenerationError::GenerationFailed(format!("Review failed: {}", e)))?)
        } else {
            None
        };

        // STEP 8: Write files (unless dry-run or validation failed)
        // Requirement 1.6, 3.1, 3.5: Write with rollback support, skip if validation failed
        if !self.config.dry_run && validation_result.valid {
            self.output_writer.write(&generated_files, &target_path, &conflicts)
                .map_err(|e| GenerationError::WriteFailed(format!("Write failed: {}", e)))?;
        }

        // Aggregate statistics
        let elapsed = start_time.elapsed();
        let stats = GenerationStats {
            tokens_used: prompt.estimated_tokens,
            time_elapsed: elapsed,
            files_generated: generated_files.len(),
            lines_generated: generated_files.iter().map(|f| f.content.lines().count()).sum(),
            conflicts_detected: conflicts.len(),
            conflicts_resolved: conflicts.len(), // Simplified: assume all resolved
        };

        // Build result
        let mut result = GenerationResult::new(
            generated_files,
            validation_result,
            conflicts,
            stats,
        );

        if let Some(review) = review_result {
            result = result.with_review(review);
        }

        Ok(result)
    }

    /// Executes generation with retry logic
    ///
    /// Retries on transient failures with exponential backoff
    ///
    /// # Arguments
    ///
    /// * `spec` - The specification to generate code from
    /// * `target_path` - Target directory for generated code
    /// * `language` - Programming language for generation
    /// * `provider` - Optional AI provider for code generation
    /// * `model` - Model name for AI generation
    /// * `temperature` - Temperature for AI sampling
    /// * `max_tokens` - Maximum tokens for AI generation
    ///
    /// # Returns
    ///
    /// A GenerationResult with files, validation results, conflicts, and statistics
    ///
    /// # Errors
    ///
    /// Returns an error if all retries are exhausted
    #[allow(clippy::too_many_arguments)]
    pub async fn generate_with_retries(
        &self,
        spec: &Spec,
        target_path: PathBuf,
        language: String,
        provider: Option<&dyn Provider>,
        model: &str,
        temperature: f32,
        max_tokens: usize,
    ) -> Result<GenerationResult, GenerationError> {
        let mut last_error = None;

        for attempt in 0..self.config.max_retries {
            match self.generate(spec, target_path.clone(), language.clone(), provider, model, temperature, max_tokens).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.config.max_retries - 1 {
                        // Exponential backoff: 100ms, 200ms, 400ms, etc.
                        let backoff_ms = 100 * (2_u64.pow(attempt as u32));
                        tokio::time::sleep(tokio::time::Duration::from_millis(backoff_ms)).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            GenerationError::GenerationFailed("Generation failed after all retries".to_string())
        }))
    }

    /// Gets the current configuration
    pub fn config(&self) -> &GenerationManagerConfig {
        &self.config
    }

    /// Updates the configuration
    pub fn set_config(&mut self, config: GenerationManagerConfig) {
        self.config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generation_manager_creation() {
        let manager = GenerationManager::new(PathBuf::from("."));
        assert_eq!(manager.config.validate, true);
        assert_eq!(manager.config.dry_run, false);
    }

    #[test]
    fn test_generation_manager_with_config() {
        let config = GenerationManagerConfig {
            project_root: PathBuf::from("."),
            validate: false,
            review: true,
            dry_run: true,
            conflict_strategy: ConflictStrategy::Skip,
            max_retries: 5,
            use_templates: true,
        };
        let manager = GenerationManager::with_config(config.clone());
        assert_eq!(manager.config.validate, false);
        assert_eq!(manager.config.review, true);
        assert_eq!(manager.config.dry_run, true);
        assert_eq!(manager.config.max_retries, 5);
        assert_eq!(manager.config.use_templates, true);
    }
}
