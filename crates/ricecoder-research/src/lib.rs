#![warn(missing_docs)]
#![forbid(unsafe_code)]

//! RiceCoder Research System
//!
//! Provides project analysis and context gathering capabilities, enabling ricecoder to understand
//! project structure, dependencies, patterns, and conventions before generating code.

pub mod architectural_intent;
pub mod architectural_patterns;
pub mod cache_manager;
pub mod cache_stats;
pub mod change_detector;
pub mod codebase_scanner;
pub mod coding_patterns;
pub mod context_builder;
pub mod context_optimizer;
pub mod context_provider;
pub mod dependency_analyzer;
pub mod error;
pub mod manager;
pub mod models;
pub mod pattern_detector;
pub mod project_analyzer;
pub mod reference_tracker;
pub mod relevance_scorer;
pub mod search_engine;
pub mod semantic_index;
pub mod standards_detector;
pub mod symbol_extractor;

pub use architectural_intent::ArchitecturalIntentTracker;
pub use architectural_patterns::ArchitecturalPatternDetector;
pub use cache_manager::{CacheManager, CacheStatistics};
pub use cache_stats::{CacheOperationTimer, CacheStatsTracker, DetailedCacheStats};
pub use change_detector::{ChangeDetection, ChangeDetector};
pub use codebase_scanner::{CodebaseScanner, FileMetadata, ScanResult};
pub use coding_patterns::CodingPatternDetector;
pub use context_builder::ContextBuilder;
pub use context_optimizer::ContextOptimizer;
pub use context_provider::ContextProvider;
pub use dependency_analyzer::{
    DartParser, DependencyAnalyzer, DependencyParser, DotNetParser, GoParser, JavaParser,
    KotlinParser, NodeJsParser, PhpParser, PythonParser, RubyParser, RustParser, SwiftParser,
    VersionConflict, VersionUpdate,
};
pub use error::ResearchError;
pub use manager::ResearchManager;
pub use models::{
    ArchitecturalDecision, ArchitecturalIntent, ArchitecturalStyle, CaseStyle, CodeContext,
    DetectedPattern, DocFormat, DocumentationStyle, FileContext, FormattingStyle, ImportGroup,
    ImportOrganization, IndentType, NamingConventions, PatternCategory, ProjectContext,
    ProjectStructure, ProjectType, ReferenceKind, SearchResult, StandardsProfile, Symbol,
    SymbolKind, SymbolReference,
};
pub use pattern_detector::PatternDetector;
pub use project_analyzer::ProjectAnalyzer;
pub use reference_tracker::{ReferenceTracker, ReferenceTrackingResult};
pub use relevance_scorer::{RelevanceScorer, ScoringWeights};
pub use search_engine::{SearchEngine, SearchOptions, SearchStatistics};
pub use semantic_index::SemanticIndex;
pub use standards_detector::StandardsDetector;
pub use symbol_extractor::SymbolExtractor;
