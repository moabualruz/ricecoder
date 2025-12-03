#![warn(missing_docs)]
#![forbid(unsafe_code)]

//! RiceCoder Research System
//!
//! Provides project analysis and context gathering capabilities, enabling ricecoder to understand
//! project structure, dependencies, patterns, and conventions before generating code.

pub mod error;
pub mod models;
pub mod manager;
pub mod project_analyzer;
pub mod codebase_scanner;
pub mod symbol_extractor;
pub mod reference_tracker;
pub mod semantic_index;
pub mod search_engine;
pub mod pattern_detector;
pub mod architectural_patterns;
pub mod coding_patterns;
pub mod standards_detector;
pub mod architectural_intent;
pub mod context_builder;
pub mod relevance_scorer;
pub mod context_optimizer;
pub mod context_provider;
pub mod dependency_analyzer;
pub mod cache_manager;
pub mod change_detector;
pub mod cache_stats;

pub use error::ResearchError;
pub use models::{
    ProjectContext, ProjectType, ProjectStructure, DetectedPattern, PatternCategory,
    ArchitecturalIntent, ArchitecturalStyle, ArchitecturalDecision, CodeContext, FileContext,
    Symbol, SymbolKind, SymbolReference, ReferenceKind, SearchResult,
    StandardsProfile, NamingConventions, FormattingStyle, CaseStyle, IndentType,
    ImportOrganization, ImportGroup, DocumentationStyle, DocFormat,
};
pub use manager::ResearchManager;
pub use project_analyzer::ProjectAnalyzer;
pub use codebase_scanner::{CodebaseScanner, FileMetadata, ScanResult};
pub use symbol_extractor::SymbolExtractor;
pub use reference_tracker::{ReferenceTracker, ReferenceTrackingResult};
pub use semantic_index::SemanticIndex;
pub use search_engine::{SearchEngine, SearchOptions, SearchStatistics};
pub use pattern_detector::PatternDetector;
pub use architectural_patterns::ArchitecturalPatternDetector;
pub use coding_patterns::CodingPatternDetector;
pub use standards_detector::StandardsDetector;
pub use architectural_intent::ArchitecturalIntentTracker;
pub use context_builder::ContextBuilder;
pub use relevance_scorer::{RelevanceScorer, ScoringWeights};
pub use context_optimizer::ContextOptimizer;
pub use context_provider::ContextProvider;
pub use dependency_analyzer::{
    DependencyAnalyzer, DependencyParser, VersionConflict, VersionUpdate,
    RustParser, NodeJsParser, PythonParser, GoParser, JavaParser, KotlinParser,
    DotNetParser, PhpParser, RubyParser, SwiftParser, DartParser,
};
pub use cache_manager::{CacheManager, CacheStatistics};
pub use change_detector::{ChangeDetector, ChangeDetection};
pub use cache_stats::{DetailedCacheStats, CacheOperationTimer, CacheStatsTracker};
