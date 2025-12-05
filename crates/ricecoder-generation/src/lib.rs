#![warn(missing_docs)]

//! Code generation module for ricecoder
//!
//! Provides template engine for code generation with variable substitution,
//! conditional logic, and boilerplate scaffolding from global and project-specific locations.
//! Also provides spec processing for converting specifications into generation plans.

pub mod error;
pub mod models;
pub mod templates;
pub mod spec_processor;
pub mod generation_plan_builder;
pub mod prompt_builder;
pub mod code_generator;
pub mod code_quality_enforcer;
pub mod code_validator;
pub mod language_validators;
pub mod conflict_detector;
pub mod conflict_resolver;
pub mod conflict_prompter;
pub mod output_writer;
pub mod review_engine;
pub mod scoring_system;
pub mod report_generator;
pub mod generation_manager;

// Re-export public API
pub use error::GenerationError;
pub use models::{
    Template, Placeholder, TemplateContext, RenderOptions, RenderResult,
    Boilerplate, BoilerplateFile, BoilerplateMetadata, BoilerplateSource,
    ConflictResolution, CaseTransform, GeneratedFile,
    ValidationResult, ValidationError, ValidationWarning, ValidationConfig,
};
pub use templates::{
    TemplateEngine, TemplateCache, CacheStats,
    TemplateError, BoilerplateError,
    TemplateParser, ParsedTemplate, TemplateElement,
    PlaceholderResolver, ValidationEngine,
    BoilerplateManager, ScaffoldingResult, FileConflict,
};
pub use spec_processor::{
    SpecProcessor, GenerationPlan, GenerationStep, Constraint, ConstraintType,
};
pub use generation_plan_builder::{
    GenerationPlanBuilder, PlanValidation,
};
pub use prompt_builder::{
    PromptBuilder, GeneratedPrompt, PromptContext, SteeringRules,
};
pub use code_generator::{
    CodeGenerator, CodeGeneratorConfig,
};
pub use code_quality_enforcer::{
    CodeQualityEnforcer, CodeQualityConfig,
};
pub use code_validator::{
    CodeValidator,
};
pub use language_validators::{
    LanguageValidator, RustValidator, TypeScriptValidator, PythonValidator,
    GoValidator, JavaValidator, get_validator,
};
pub use conflict_detector::{
    ConflictDetector, FileConflictInfo, FileDiff, DiffLine,
};
pub use conflict_resolver::{
    ConflictResolver, ConflictStrategy, ConflictResolutionResult,
};
pub use conflict_prompter::{
    ConflictPrompter, PromptResult,
};
pub use output_writer::{
    OutputWriter, OutputWriterConfig, FileWriteResult, WriteResult, RollbackInfo,
};
pub use review_engine::{
    ReviewEngine, ReviewResult, CodeQualityMetrics, ComplianceDetails,
    Suggestion, SuggestionCategory, ReviewIssue, IssueSeverity, ReviewConfig,
};
pub use scoring_system::{
    ScoringSystem, ScoreBreakdown, ScoreComponent, ComplianceScore,
    ScoringFeedback, ScoringConfig,
};
pub use report_generator::{
    GenerationStats, GenerationResult, GenerationReport, ReportSummary,
    FileStatistics, ValidationReport, ConflictReport, ReviewReport,
    PerformanceMetrics, ReportGenerator,
};
pub use generation_manager::{
    GenerationManager, GenerationManagerConfig,
};
