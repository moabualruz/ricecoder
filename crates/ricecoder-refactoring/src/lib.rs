//! Ricecoder Refactoring Engine
//!
//! A safe, language-agnostic refactoring engine with impact analysis, preview, and rollback capabilities.
//!
//! # Architecture
//!
//! The refactoring engine follows a language-agnostic, configuration-driven architecture:
//!
//! - **Language-Agnostic Core**: Generic refactoring engine that works with any language
//! - **Configuration-Driven**: Language-specific behavior defined in YAML/JSON configuration files
//! - **Pluggable Providers**: Language-specific adapters for Rust, TypeScript, Python, and generic fallback
//! - **Storage Integration**: Uses `ricecoder-storage` for configuration management
//! - **Safe by Default**: Automatic backups, validation, and rollback capabilities
//!
//! # Provider Priority Chain
//!
//! The engine implements a provider priority chain for language support:
//!
//! 1. External LSP Servers (language-specific, maintained by language communities)
//! 2. Configured Refactoring Rules (YAML/JSON configuration files)
//! 3. Built-in Language-Specific Providers (Rust, TypeScript, Python)
//! 4. Generic Text-Based Refactoring (fallback for any language)
//!
//! # Example
//!
//! ```ignore
//! use ricecoder_refactoring::RefactoringEngine;
//! use ricecoder_refactoring::config::ConfigManager;
//!
//! // Create configuration manager
//! let config_manager = ConfigManager::new(storage);
//!
//! // Create refactoring engine
//! let engine = RefactoringEngine::new(config_manager);
//!
//! // Perform refactoring
//! let result = engine.refactor(code, language, refactoring_type)?;
//! ```

pub mod adapters;
pub mod config;
pub mod error;
pub mod impact;
pub mod patterns;
pub mod preview;
pub mod providers;
pub mod safety;
pub mod types;
pub mod validation;

// Re-export commonly used types
pub use adapters::{
    GenericRefactoringProvider, PythonRefactoringProvider, RustRefactoringProvider,
    TypeScriptRefactoringProvider,
};
pub use config::{ConfigLoader, ConfigManager, LanguageConfig, StorageConfigLoader};
pub use error::{RefactoringError, Result};
pub use impact::{Dependency, DependencyGraph, DependencyType, ImpactAnalyzer, Symbol, SymbolType};
pub use patterns::{
    PatternApplication, PatternExporter, PatternMatcher, PatternParameter, PatternScope,
    PatternStore, PatternValidator, RefactoringPattern,
};
pub use preview::{DiffHunk, PreviewGenerator, UnifiedDiff};
pub use providers::{LspProvider, LspProviderRegistry, ProviderRegistry, RefactoringProvider};
pub use safety::{RollbackHandler, SafetyChecker};
pub use types::{
    ChangeType, FileChange, Refactoring, RefactoringOptions, RefactoringResult, RefactoringTarget,
    RefactoringType, ValidationResult,
};
pub use validation::ValidationEngine;

/// The main refactoring engine
pub struct RefactoringEngine {
    config_manager: ConfigManager,
    provider_registry: ProviderRegistry,
    impact_analyzer: ImpactAnalyzer,
    preview_generator: PreviewGenerator,
    safety_checker: SafetyChecker,
    validation_engine: ValidationEngine,
}

impl RefactoringEngine {
    /// Create a new refactoring engine
    pub fn new(config_manager: ConfigManager, provider_registry: ProviderRegistry) -> Self {
        Self {
            config_manager,
            provider_registry,
            impact_analyzer: ImpactAnalyzer::new(),
            preview_generator: PreviewGenerator::new(),
            safety_checker: SafetyChecker::new(),
            validation_engine: ValidationEngine::new(),
        }
    }

    /// Get the configuration manager
    pub fn config_manager(&self) -> &ConfigManager {
        &self.config_manager
    }

    /// Get the provider registry
    pub fn provider_registry(&self) -> &ProviderRegistry {
        &self.provider_registry
    }

    /// Get the impact analyzer
    pub fn impact_analyzer(&self) -> &ImpactAnalyzer {
        &self.impact_analyzer
    }

    /// Get the preview generator
    pub fn preview_generator(&self) -> &PreviewGenerator {
        &self.preview_generator
    }

    /// Get the safety checker
    pub fn safety_checker(&self) -> &SafetyChecker {
        &self.safety_checker
    }

    /// Get the validation engine
    pub fn validation_engine(&self) -> &ValidationEngine {
        &self.validation_engine
    }
}
