//! Repository interfaces for data persistence
//!
//! REQ-ARCH-001.4: Infrastructure implements domain interfaces
//!
//! These interfaces define the contracts for data access.
//! Implementations will be provided by infrastructure crates.
//! The domain layer defines only interfaces (traits), no concrete implementations.

use async_trait::async_trait;

use crate::{
    errors::*,
    project::Project,
    session::Session,
    specification::{Specification, SpecStatus},
    value_objects::*,
};

/// Repository for project entities
#[async_trait]
pub trait ProjectRepository {
    /// Save a project
    async fn save(&self, project: &Project) -> DomainResult<()>;

    /// Find project by ID
    async fn find_by_id(&self, id: &ProjectId) -> DomainResult<Option<Project>>;

    /// Find all projects
    async fn find_all(&self) -> DomainResult<Vec<Project>>;

    /// Delete project by ID
    async fn delete(&self, id: &ProjectId) -> DomainResult<()>;

    /// Check if project exists
    async fn exists(&self, id: &ProjectId) -> DomainResult<bool>;
}

/// Repository for session entities
#[async_trait]
pub trait SessionRepository {
    /// Save a session
    async fn save(&self, session: &Session) -> DomainResult<()>;

    /// Find session by ID
    async fn find_by_id(&self, id: &SessionId) -> DomainResult<Option<Session>>;

    /// Find sessions by project ID
    async fn find_by_project(&self, project_id: &ProjectId) -> DomainResult<Vec<Session>>;

    /// Find active sessions
    async fn find_active(&self) -> DomainResult<Vec<Session>>;

    /// Delete session by ID
    async fn delete(&self, id: &SessionId) -> DomainResult<()>;
}

/// Repository for specification entities
///
/// REQ-ARCH-001.4: Infrastructure implements domain interfaces
#[async_trait]
pub trait SpecificationRepository {
    /// Save a specification
    async fn save(&self, specification: &Specification) -> DomainResult<()>;

    /// Find specification by ID
    async fn find_by_id(&self, id: &SpecificationId) -> DomainResult<Option<Specification>>;

    /// Find specifications by project ID
    async fn find_by_project(&self, project_id: &ProjectId) -> DomainResult<Vec<Specification>>;

    /// Find all specifications
    async fn find_all(&self) -> DomainResult<Vec<Specification>>;

    /// Delete specification by ID
    async fn delete(&self, id: &SpecificationId) -> DomainResult<()>;

    /// Check if specification exists
    async fn exists(&self, id: &SpecificationId) -> DomainResult<bool>;

    /// List specifications by status
    async fn find_by_status(
        &self,
        status: crate::specification::SpecStatus,
    ) -> DomainResult<Vec<Specification>>;
}



/// Generic repository trait for common operations
#[async_trait]
pub trait Repository<T, ID>: Send + Sync {
    /// Save entity
    async fn save(&self, entity: &T) -> DomainResult<()>;

    /// Find by ID
    async fn find_by_id(&self, id: &ID) -> DomainResult<Option<T>>;

    /// Find all entities
    async fn find_all(&self) -> DomainResult<Vec<T>>;

    /// Delete by ID
    async fn delete(&self, id: &ID) -> DomainResult<()>;

    /// Check existence
    async fn exists(&self, id: &ID) -> DomainResult<bool>;
}

/// Unit of work pattern for transactional operations
#[async_trait]
pub trait UnitOfWork: Send + Sync {
    /// Begin transaction
    async fn begin(&self) -> DomainResult<Box<dyn TransactionDelegate + Send + Sync>>;

    /// Execute within transaction
    async fn execute<F, R>(&self, f: F) -> DomainResult<R>
    where
        F: FnOnce() -> DomainResult<R> + Send,
        R: Send;
}

/// Transaction delegate to work around dyn async trait limitations
pub trait TransactionDelegate: Send + Sync {
    /// Commit transaction
    fn commit(
        self: Box<Self>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = DomainResult<()>> + Send>>;

    /// Rollback transaction
    fn rollback(
        self: Box<Self>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = DomainResult<()>> + Send>>;
}
