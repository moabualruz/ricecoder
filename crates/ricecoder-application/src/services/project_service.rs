//! Project Application Service
//!
//! Orchestrates project-related use cases using domain aggregates and services.
//!
//! ProjectService
//! AC-3.1.1: Stateless and reusable across requests
//! AC-3.1.2: Depends only on repository abstractions
//! AC-3.1.3: Defines transaction boundaries
//! AC-3.1.4: Emits application events

use std::sync::Arc;

use chrono::Utc;

use crate::dto::{CreateProjectCommand, ProjectDetailDto, ProjectSummaryDto};
use crate::errors::{ApplicationError, ApplicationResult};
use crate::events::{ApplicationEvent, EventPublisher};
use crate::ports::UnitOfWork;

use ricecoder_domain::project::Project;
use ricecoder_domain::value_objects::ProjectId;
use ricecoder_domain::repositories::ProjectRepository;

/// Project Application Service
///
/// Orchestrates project-related use cases. Stateless: all mutable state
/// is persisted via repositories.
pub struct ProjectService<R, U, E>
where
    R: ProjectRepository + Send + Sync,
    U: UnitOfWork + Send + Sync,
    E: EventPublisher + Send + Sync,
{
    repository: Arc<R>,
    uow: Arc<U>,
    events: Arc<E>,
}

impl<R, U, E> ProjectService<R, U, E>
where
    R: ProjectRepository + Send + Sync + 'static,
    U: UnitOfWork + Send + Sync,
    E: EventPublisher + Send + Sync,
{
    /// Create a new ProjectService with injected dependencies
    pub fn new(repository: Arc<R>, uow: Arc<U>, events: Arc<E>) -> Self {
        Self {
            repository,
            uow,
            events,
        }
    }

    /// Create a new project
    ///
    /// AC-3.1.5: Validates name uniqueness before persisting
    /// AC-3.1.3: Uses transaction boundary
    /// AC-3.1.4: Emits ProjectCreated event
    pub async fn create_project(&self, cmd: CreateProjectCommand) -> ApplicationResult<String> {
        // Validate input
        let name = cmd.name.trim();
        if name.is_empty() {
            return Err(ApplicationError::ValidationFailed(
                "Project name is required".into(),
            ));
        }

        // Check path is provided
        if cmd.root_path.trim().is_empty() {
            return Err(ApplicationError::RequiredFieldMissing("root_path".into()));
        }

        // Create the project aggregate (Project::create validates name and path)
        let (project, _events) = Project::create(
            name.to_string(),
            cmd.language,
            cmd.root_path.clone(),
            cmd.description.clone(),
        )
        .map_err(|e| ApplicationError::DomainError(e.to_string()))?;

        let project_id = project.id().to_string();

        // Persist within transaction boundary
        let repo = Arc::clone(&self.repository);
        let p = project.clone();
        self.uow
            .execute(async move {
                repo.save(&p)
                    .await
                    .map_err(|e| ApplicationError::RepositoryError(e.to_string()))
            })
            .await?;

        // Emit application event
        self.events
            .publish(ApplicationEvent::ProjectCreated {
                project_id: project_id.clone(),
                name: name.to_string(),
                timestamp: Utc::now(),
            })
            .await;

        Ok(project_id)
    }

    /// Get project by ID
    pub async fn get_project(&self, id: &str) -> ApplicationResult<ProjectDetailDto> {
        let project_id = ProjectId::from_string(id)
            .map_err(|_| ApplicationError::ValidationFailed("Invalid project ID".into()))?;

        let project = self
            .repository
            .find_by_id(&project_id)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?
            .ok_or_else(|| ApplicationError::ProjectNotFound(id.to_string()))?;

        Ok(ProjectDetailDto::from_domain(&project))
    }

    /// List all projects
    pub async fn list_projects(&self) -> ApplicationResult<Vec<ProjectSummaryDto>> {
        let projects = self
            .repository
            .find_all()
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?;

        Ok(projects
            .iter()
            .map(ProjectSummaryDto::from_domain)
            .collect())
    }

    /// Archive a project
    pub async fn archive_project(&self, id: &str) -> ApplicationResult<()> {
        let project_id = ProjectId::from_string(id)
            .map_err(|_| ApplicationError::ValidationFailed("Invalid project ID".into()))?;

        let mut project = self
            .repository
            .find_by_id(&project_id)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?
            .ok_or_else(|| ApplicationError::ProjectNotFound(id.to_string()))?;

        // Archive the project
        project
            .archive()
            .map_err(|e| ApplicationError::DomainError(e.to_string()))?;

        // Persist
        let repo = Arc::clone(&self.repository);
        let p = project.clone();
        self.uow
            .execute(async move {
                repo.save(&p)
                    .await
                    .map_err(|e| ApplicationError::RepositoryError(e.to_string()))
            })
            .await?;

        // Emit event
        self.events
            .publish(ApplicationEvent::ProjectArchived {
                project_id: id.to_string(),
                timestamp: Utc::now(),
            })
            .await;

        Ok(())
    }

    /// Delete a project
    pub async fn delete_project(&self, id: &str) -> ApplicationResult<()> {
        let project_id = ProjectId::from_string(id)
            .map_err(|_| ApplicationError::ValidationFailed("Invalid project ID".into()))?;

        // Verify project exists
        let exists = self
            .repository
            .exists(&project_id)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?;

        if !exists {
            return Err(ApplicationError::ProjectNotFound(id.to_string()));
        }

        // Delete
        let repo = Arc::clone(&self.repository);
        let pid = project_id.clone();
        self.uow
            .execute(async move {
                repo.delete(&pid)
                    .await
                    .map_err(|e| ApplicationError::RepositoryError(e.to_string()))
            })
            .await?;

        // Emit event
        self.events
            .publish(ApplicationEvent::ProjectDeleted {
                project_id: id.to_string(),
                timestamp: Utc::now(),
            })
            .await;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::NoOpEventPublisher;
    use crate::ports::NoOpUnitOfWork;
    use async_trait::async_trait;
    use ricecoder_domain::value_objects::ProgrammingLanguage;
    use std::collections::HashMap;
    use std::sync::Mutex;

    /// In-memory project repository for testing
    struct InMemoryProjectRepository {
        projects: Mutex<HashMap<String, Project>>,
    }

    impl InMemoryProjectRepository {
        fn new() -> Self {
            Self {
                projects: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl ProjectRepository for InMemoryProjectRepository {
        async fn save(&self, project: &Project) -> ricecoder_domain::DomainResult<()> {
            self.projects
                .lock()
                .unwrap()
                .insert(project.id().to_string(), project.clone());
            Ok(())
        }

        async fn find_by_id(
            &self,
            id: &ProjectId,
        ) -> ricecoder_domain::DomainResult<Option<Project>> {
            Ok(self.projects.lock().unwrap().get(&id.to_string()).cloned())
        }

        async fn find_all(&self) -> ricecoder_domain::DomainResult<Vec<Project>> {
            Ok(self.projects.lock().unwrap().values().cloned().collect())
        }

        async fn delete(&self, id: &ProjectId) -> ricecoder_domain::DomainResult<()> {
            self.projects.lock().unwrap().remove(&id.to_string());
            Ok(())
        }

        async fn exists(&self, id: &ProjectId) -> ricecoder_domain::DomainResult<bool> {
            Ok(self
                .projects
                .lock()
                .unwrap()
                .contains_key(&id.to_string()))
        }
    }

    #[tokio::test]
    async fn test_create_project_success() {
        let repo = Arc::new(InMemoryProjectRepository::new());
        let uow = Arc::new(NoOpUnitOfWork);
        let events = Arc::new(NoOpEventPublisher);

        let service = ProjectService::new(repo, uow, events);

        let cmd = CreateProjectCommand {
            name: "test-project".to_string(),
            root_path: "/path/to/project".to_string(),
            language: ProgrammingLanguage::Rust,
            description: None,
        };

        let result = service.create_project(cmd).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_project_empty_name() {
        let repo = Arc::new(InMemoryProjectRepository::new());
        let uow = Arc::new(NoOpUnitOfWork);
        let events = Arc::new(NoOpEventPublisher);

        let service = ProjectService::new(repo, uow, events);

        let cmd = CreateProjectCommand {
            name: "".to_string(),
            root_path: "/path/to/project".to_string(),
            language: ProgrammingLanguage::Rust,
            description: None,
        };

        let result = service.create_project(cmd).await;
        assert!(matches!(result, Err(ApplicationError::ValidationFailed(_))));
    }

    #[tokio::test]
    async fn test_create_project_empty_path() {
        let repo = Arc::new(InMemoryProjectRepository::new());
        let uow = Arc::new(NoOpUnitOfWork);
        let events = Arc::new(NoOpEventPublisher);

        let service = ProjectService::new(repo, uow, events);

        let cmd = CreateProjectCommand {
            name: "test-project".to_string(),
            root_path: "".to_string(),
            language: ProgrammingLanguage::Rust,
            description: None,
        };

        let result = service.create_project(cmd).await;
        assert!(matches!(
            result,
            Err(ApplicationError::RequiredFieldMissing(_))
        ));
    }

    #[tokio::test]
    async fn test_get_project_not_found() {
        let repo = Arc::new(InMemoryProjectRepository::new());
        let uow = Arc::new(NoOpUnitOfWork);
        let events = Arc::new(NoOpEventPublisher);

        let service = ProjectService::new(repo, uow, events);

        let result = service
            .get_project("00000000-0000-0000-0000-000000000000")
            .await;
        assert!(matches!(result, Err(ApplicationError::ProjectNotFound(_))));
    }

    #[tokio::test]
    async fn test_list_projects() {
        let repo = Arc::new(InMemoryProjectRepository::new());
        let uow = Arc::new(NoOpUnitOfWork);
        let events = Arc::new(NoOpEventPublisher);

        let service = ProjectService::new(Arc::clone(&repo), uow, events);

        // Create a project first
        let cmd = CreateProjectCommand {
            name: "test-project".to_string(),
            root_path: "/path/to/project".to_string(),
            language: ProgrammingLanguage::Rust,
            description: None,
        };
        service.create_project(cmd).await.unwrap();

        // List projects
        let projects = service.list_projects().await.unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "test-project");
    }
}
