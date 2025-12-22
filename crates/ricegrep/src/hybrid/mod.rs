//! Hybrid retrieval and ranking scaffolding.

pub mod error;
pub mod fusion;
pub mod models;
pub mod selector;

pub use error::{FusionError, RankingError};
pub use fusion::{FusionConfig, FusionEngine, FusionMethod, HybridWeights};
pub use models::{CandidateChunk, FusedResults, HybridConfig, QualityMetrics, RetrievalSource};
pub use selector::CandidateSelector;
