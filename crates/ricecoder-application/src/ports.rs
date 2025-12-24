//! Application layer ports (abstractions for infrastructure)
//!
//! Ports define the contracts that the Infrastructure Layer must implement.
//! This follows the Ports and Adapters (Hexagonal) architecture pattern.

use async_trait::async_trait;
use std::future::Future;
use std::pin::Pin;

use crate::errors::ApplicationResult;

/// Unit of Work pattern for transaction management
///
/// Provides atomic transaction boundaries for multi-step operations.
/// Infrastructure Layer provides the concrete implementation.
///
/// # Example
///
/// ```ignore
/// async fn create_project(&self, cmd: CreateProjectCommand) -> ApplicationResult<String> {
///     self.uow.execute(async move {
///         // All operations here are atomic
///         let project = Project::create(...)?;
///         self.repository.save(&project).await?;
///         Ok(project.id().to_string())
///     }).await
/// }
/// ```
#[async_trait]
pub trait UnitOfWork: Send + Sync {
    /// Execute a function within a transaction boundary
    ///
    /// If the function returns Ok, the transaction is committed.
    /// If the function returns Err, the transaction is rolled back.
    async fn execute<T, F>(&self, f: F) -> ApplicationResult<T>
    where
        T: Send + 'static,
        F: Future<Output = ApplicationResult<T>> + Send + 'static;
}

/// Simple implementation that executes without transaction management
///
/// Used for testing or when transactions are not needed
pub struct NoOpUnitOfWork;

#[async_trait]
impl UnitOfWork for NoOpUnitOfWork {
    async fn execute<T, F>(&self, f: F) -> ApplicationResult<T>
    where
        T: Send + 'static,
        F: Future<Output = ApplicationResult<T>> + Send + 'static,
    {
        f.await
    }
}

/// Box wrapper for async closures in UnitOfWork
pub type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_no_op_unit_of_work_success() {
        let uow = NoOpUnitOfWork;
        let result = uow.execute(async { Ok::<_, crate::errors::ApplicationError>(42) }).await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_no_op_unit_of_work_error() {
        let uow = NoOpUnitOfWork;
        let result = uow
            .execute(async {
                Err::<i32, _>(crate::errors::ApplicationError::ValidationFailed(
                    "test".into(),
                ))
            })
            .await;
        assert!(result.is_err());
    }
}
