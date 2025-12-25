// Coordinator requires local-embeddings feature for HybridQueryEngine
#[cfg(feature = "local-embeddings")]
pub mod coordinator;

#[cfg(feature = "local-embeddings")]
pub use coordinator::SearchCoordinator;
