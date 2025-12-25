#![warn(missing_docs)]

//! Code generation module for ricecoder
//!
//! Provides template engine for code generation with variable substitution,
//! conditional logic, and boilerplate scaffolding from global and project-specific locations.
//! Also provides spec processing for converting specifications into generation plans.

pub mod code_generator;
pub mod code_quality_enforcer;
pub mod code_validator;
pub mod conflict_detector;
pub mod conflict_prompter;
pub mod conflict_resolver;
pub mod error;
pub mod generation_manager;
pub mod generation_plan_builder;
pub mod language_validators;
pub mod models;
pub mod output_writer;
pub mod prompt_builder;
pub mod report_generator;
pub mod review_engine;
pub mod scoring_system;
pub mod spec_processor;
pub mod templates;

// Re-export public API
pub use code_generator::{CodeGenerator, CodeGeneratorConfig};
pub use code_quality_enforcer::{CodeQualityConfig, CodeQualityEnforcer};
pub use code_validator::CodeValidator;
pub use conflict_detector::{ConflictDetector, DiffLine, FileConflictInfo, FileDiff};
pub use conflict_prompter::{ConflictPrompter, PromptResult};
pub use conflict_resolver::{ConflictResolutionResult, ConflictResolver, ConflictStrategy};
pub use error::GenerationError;
pub use generation_manager::{GenerationManager, GenerationManagerConfig};
pub use generation_plan_builder::{GenerationPlanBuilder, PlanValidation};
pub use language_validators::{
    get_validator, GoValidator, JavaValidator, LanguageValidator, PythonValidator, RustValidator,
    TypeScriptValidator,
};
pub use models::{
    Boilerplate, BoilerplateFile, BoilerplateMetadata, BoilerplateSource, CaseTransform,
    ConflictResolution, GeneratedFile, Placeholder, RenderOptions, RenderResult, Template,
    TemplateContext, ValidationConfig, ValidationError, ValidationResult, ValidationWarning,
};
pub use output_writer::{
    FileWriteResult, OutputWriter, OutputWriterConfig, RollbackInfo, WriteResult,
};
pub use prompt_builder::{GeneratedPrompt, PromptBuilder, PromptContext, GovernanceRules};
pub use report_generator::{
    ConflictReport, FileStatistics, GenerationReport, GenerationResult, GenerationStats,
    PerformanceMetrics, ReportGenerator, ReportSummary, ReviewReport, ValidationReport,
};
pub use review_engine::{
    CodeQualityMetrics, ComplianceDetails, IssueSeverity, ReviewConfig, ReviewEngine, ReviewIssue,
    ReviewResult, Suggestion, SuggestionCategory,
};
pub use scoring_system::{
    ComplianceScore, ScoreBreakdown, ScoreComponent, ScoringConfig, ScoringFeedback, ScoringSystem,
};
pub use spec_processor::{
    Constraint, ConstraintType, GenerationPlan, GenerationStep, SpecProcessor,
};
pub use templates::{
    BoilerplateError, BoilerplateManager, CacheStats, FileConflict, ParsedTemplate,
    PlaceholderResolver, ScaffoldingResult, TemplateCache, TemplateElement, TemplateEngine,
    TemplateError, TemplateParser, ValidationEngine,
};
