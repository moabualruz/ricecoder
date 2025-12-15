#![warn(missing_docs)]
#![forbid(unsafe_code)]

//! RiceCoder Core Research System
//!
//! Provides core project analysis and context gathering capabilities with MCP integration.
//! Focuses on semantic understanding, search, and standards detection for AI-assisted development.

pub mod error;
pub mod manager;
pub mod models;
pub mod project_analyzer;
pub mod reference_tracker;
pub mod relevance_scorer;
pub mod search_engine;
pub mod semantic_index;
pub mod standards_detector;

// Re-export from dependencies for convenience (when features are enabled)
#[cfg(feature = "parsers")]
pub use ricecoder_parsers as parsers;
#[cfg(feature = "patterns")]
pub use ricecoder_patterns as patterns;

// Re-export core types
pub use error::ResearchError;
pub use manager::ResearchManager;
pub use models::*;
pub use project_analyzer::ProjectAnalyzer;
pub use reference_tracker::{ReferenceTracker, ReferenceTrackingResult};
pub use relevance_scorer::{RelevanceScorer, ScoringWeights};
pub use search_engine::{SearchEngine, SearchOptions, SearchStatistics};
pub use semantic_index::SemanticIndex;
pub use standards_detector::StandardsDetector;

// Re-export pattern types for convenience
#[cfg(feature = "patterns")]
pub use ricecoder_patterns::DetectedPattern;
