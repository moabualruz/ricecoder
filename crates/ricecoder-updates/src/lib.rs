//! Update management system for RiceCoder

pub mod analytics;
pub mod checker;
pub mod error;
pub mod models;
pub mod policy;
pub mod rollback;
pub mod updater;

// Re-exports for convenience
pub use analytics::AnalyticsCollector;
pub use checker::UpdateChecker;
pub use error::UpdateError;
pub use models::*;
pub use policy::UpdatePolicy;
pub use rollback::RollbackManager;
pub use updater::BinaryUpdater;

#[cfg(test)]
mod tests {
    #[test]
    fn placeholder() {
        // Placeholder test - integration tests are in tests/ directory
        assert!(true);
    }
}
