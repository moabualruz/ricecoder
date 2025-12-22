//! Vector embedding + ANN indexing (Qdrant) with fallback semantic storage.

pub mod alerting;
pub mod batch;
pub mod embeddings;
pub mod fallback;
pub mod metrics_storage;
pub mod observability;
pub mod pipeline;
pub mod qdrant;
pub mod search;

pub use alerting::{AlertManager, AlertSeverity, AlertState, AlertSummary};
pub use metrics_storage::{MetricsHistoryEntry, MetricsStorage};
pub use observability::{
    VectorError, VectorHealth, VectorHealthStatus, VectorTelemetry, VectorTelemetrySnapshot,
};
