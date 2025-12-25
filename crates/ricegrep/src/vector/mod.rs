//! Vector embedding + ANN indexing (Qdrant) with fallback semantic storage.

pub mod alerting;
pub mod fallback;
pub mod metrics_storage;
pub mod observability;
pub mod qdrant;

// Embedding-dependent modules (require local-embeddings feature)
#[cfg(feature = "local-embeddings")]
pub mod batch;
#[cfg(feature = "local-embeddings")]
pub mod embeddings;
#[cfg(feature = "local-embeddings")]
pub mod pipeline;
#[cfg(feature = "local-embeddings")]
pub mod search;

pub use alerting::{AlertManager, AlertSeverity, AlertState, AlertSummary};
pub use metrics_storage::{MetricsHistoryEntry, MetricsStorage};
pub use observability::{
    VectorError, VectorHealth, VectorHealthStatus, VectorTelemetry, VectorTelemetrySnapshot,
};

#[cfg(feature = "local-embeddings")]
pub use embeddings::{EmbeddingConfig, EmbeddingGenerator, EmbeddingModelKind, ModelManager};
#[cfg(feature = "local-embeddings")]
pub use batch::BatchProcessor;
#[cfg(feature = "local-embeddings")]
pub use pipeline::VectorPipeline;
#[cfg(feature = "local-embeddings")]
pub use search::{EmbeddingProvider, HybridQueryEngine};
