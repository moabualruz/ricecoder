//! Repository interfaces for data persistence
//!
//! These interfaces define the contracts for data access.
//! Implementations will be provided by infrastructure crates.

use crate::entities::*;
use crate::errors::*;
use crate::value_objects::*;
use async_trait::async_trait;

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

    /// Update session status
    async fn update_status(&self, id: &SessionId, status: SessionStatus) -> DomainResult<()>;
}

/// Repository for analysis results
#[async_trait]
pub trait AnalysisRepository {
    /// Save analysis result
    async fn save(&self, result: &AnalysisResult) -> DomainResult<()>;

    /// Find by ID
    async fn find_by_id(&self, id: &str) -> DomainResult<Option<AnalysisResult>>;

    /// Find by project ID
    async fn find_by_project(&self, project_id: &ProjectId) -> DomainResult<Vec<AnalysisResult>>;

    /// Find by file ID
    async fn find_by_file(&self, file_id: &FileId) -> DomainResult<Vec<AnalysisResult>>;

    /// Find by analysis type
    async fn find_by_type(&self, analysis_type: AnalysisType) -> DomainResult<Vec<AnalysisResult>>;

    /// Delete old results (cleanup)
    async fn delete_older_than(&self, cutoff: chrono::DateTime<chrono::Utc>)
        -> DomainResult<usize>;
}

/// Repository for file entities
#[async_trait]
pub trait FileRepository {
    /// Save file
    async fn save(&self, file: &CodeFile) -> DomainResult<()>;

    /// Find by ID
    async fn find_by_id(&self, id: &FileId) -> DomainResult<Option<CodeFile>>;

    /// Find files by project
    async fn find_by_project(&self, project_id: &ProjectId) -> DomainResult<Vec<CodeFile>>;

    /// Find by path
    async fn find_by_path(
        &self,
        project_id: &ProjectId,
        relative_path: &str,
    ) -> DomainResult<Option<CodeFile>>;

    /// Search files by content pattern
    async fn search_content(
        &self,
        project_id: &ProjectId,
        pattern: &str,
    ) -> DomainResult<Vec<CodeFile>>;

    /// Delete file
    async fn delete(&self, id: &FileId) -> DomainResult<()>;
}

/// Repository for provider configurations
#[async_trait]
pub trait ProviderRepository {
    /// Save provider
    async fn save(&self, provider: &Provider) -> DomainResult<()>;

    /// Find by ID
    async fn find_by_id(&self, id: &str) -> DomainResult<Option<Provider>>;

    /// Find all providers
    async fn find_all(&self) -> DomainResult<Vec<Provider>>;

    /// Find active providers
    async fn find_active(&self) -> DomainResult<Vec<Provider>>;

    /// Delete provider
    async fn delete(&self, id: &str) -> DomainResult<()>;

    /// Update provider status
    async fn update_status(&self, id: &str, is_active: bool) -> DomainResult<()>;
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
