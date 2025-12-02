#![warn(missing_docs)]

//! Code generation module for ricecoder
//!
//! Provides template engine for code generation with variable substitution,
//! conditional logic, and boilerplate scaffolding from global and project-specific locations.

pub mod error;
pub mod models;
pub mod templates;

// Re-export public API
pub use error::GenerationError;
pub use models::{
    Template, Placeholder, TemplateContext, RenderOptions, RenderResult,
    Boilerplate, BoilerplateFile, BoilerplateMetadata, BoilerplateSource,
    ConflictResolution, CaseTransform,
};
pub use templates::{
    TemplateEngine, TemplateCache, CacheStats,
    TemplateError, BoilerplateError,
    TemplateParser, ParsedTemplate, TemplateElement,
    PlaceholderResolver, ValidationEngine,
    BoilerplateManager, ScaffoldingResult, FileConflict,
};
