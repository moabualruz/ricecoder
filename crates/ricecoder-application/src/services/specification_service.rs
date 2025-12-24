//! Specification Application Service
//!
//! Orchestrates specification-related use cases using domain aggregates.
//!
//! REQ-APP-030: SpecificationService
//! AC-3.3.1: Stateless and reusable across requests
//! AC-3.3.2: Depends only on repository abstractions
//! AC-3.3.3: Validates requirement traceability before accepting tasks

use std::sync::Arc;

use chrono::Utc;

use crate::dto::{
    CreateSpecificationCommand, RequirementDto, SpecificationSummaryDto, TaskDto,
};
use crate::errors::{ApplicationError, ApplicationResult};
use crate::events::{ApplicationEvent, EventPublisher};
use crate::ports::UnitOfWork;

use ricecoder_domain::specification::{Specification, SpecStatus};
use ricecoder_domain::value_objects::{ProjectId, SpecificationId, RequirementId, TaskId};
use ricecoder_domain::repositories::{SpecificationRepository, ProjectRepository};

/// Specification Application Service
///
/// Orchestrates specification-related use cases. Stateless: all mutable state
/// is persisted via repositories.
pub struct SpecificationService<SR, PR, U, E>
where
    SR: SpecificationRepository + Send + Sync,
    PR: ProjectRepository + Send + Sync,
    U: UnitOfWork + Send + Sync,
    E: EventPublisher + Send + Sync,
{
    spec_repository: Arc<SR>,
    project_repository: Arc<PR>,
    uow: Arc<U>,
    events: Arc<E>,
}

impl<SR, PR, U, E> SpecificationService<SR, PR, U, E>
where
    SR: SpecificationRepository + Send + Sync + 'static,
    PR: ProjectRepository + Send + Sync + 'static,
    U: UnitOfWork + Send + Sync,
    E: EventPublisher + Send + Sync,
{
    /// Create a new SpecificationService with injected dependencies
    pub fn new(
        spec_repository: Arc<SR>,
        project_repository: Arc<PR>,
        uow: Arc<U>,
        events: Arc<E>,
    ) -> Self {
        Self {
            spec_repository,
            project_repository,
            uow,
            events,
        }
    }

    /// Create a new specification for a project
    pub async fn create_specification(
        &self,
        cmd: CreateSpecificationCommand,
    ) -> ApplicationResult<String> {
        // Validate input
        let name = cmd.name.trim();
        if name.is_empty() {
            return Err(ApplicationError::ValidationFailed(
                "Specification name is required".into(),
            ));
        }

        // Validate project exists
        let project_id = ProjectId::from_string(&cmd.project_id)
            .map_err(|_| ApplicationError::ValidationFailed("Invalid project ID".into()))?;

        let project_exists = self
            .project_repository
            .exists(&project_id)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?;

        if !project_exists {
            return Err(ApplicationError::ProjectNotFound(cmd.project_id.clone()));
        }

        // Create specification
        let version = cmd.version.unwrap_or_else(|| "1.0.0".to_string());
        let (spec, _event) = Specification::create(
            project_id.clone(),
            name.to_string(),
            cmd.description.clone(),
            version,
        )
        .map_err(|e| ApplicationError::DomainError(e.to_string()))?;

        let spec_id = spec.id().to_string();

        // Persist within transaction
        let repo = Arc::clone(&self.spec_repository);
        let s = spec.clone();
        self.uow
            .execute(async move {
                repo.save(&s)
                    .await
                    .map_err(|e| ApplicationError::RepositoryError(e.to_string()))
            })
            .await?;

        // Emit event
        self.events
            .publish(ApplicationEvent::SpecificationCreated {
                specification_id: spec_id.clone(),
                project_id: project_id.to_string(),
                name: name.to_string(),
                timestamp: Utc::now(),
            })
            .await;

        Ok(spec_id)
    }

    /// Get specification by ID
    pub async fn get_specification(
        &self,
        id: &str,
    ) -> ApplicationResult<SpecificationSummaryDto> {
        let spec_id = SpecificationId::from_string(id)
            .map_err(|_| ApplicationError::ValidationFailed("Invalid specification ID".into()))?;

        let spec = self
            .spec_repository
            .find_by_id(&spec_id)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?
            .ok_or_else(|| ApplicationError::SpecificationNotFound(id.to_string()))?;

        Ok(SpecificationSummaryDto::from_domain(&spec))
    }

    /// List specifications for a project
    pub async fn list_specifications_for_project(
        &self,
        project_id: &str,
    ) -> ApplicationResult<Vec<SpecificationSummaryDto>> {
        let pid = ProjectId::from_string(project_id)
            .map_err(|_| ApplicationError::ValidationFailed("Invalid project ID".into()))?;

        let specs = self
            .spec_repository
            .find_by_project(&pid)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?;

        Ok(specs
            .iter()
            .map(SpecificationSummaryDto::from_domain)
            .collect())
    }

    /// List all specifications
    pub async fn list_all_specifications(&self) -> ApplicationResult<Vec<SpecificationSummaryDto>> {
        let specs = self
            .spec_repository
            .find_all()
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?;

        Ok(specs
            .iter()
            .map(SpecificationSummaryDto::from_domain)
            .collect())
    }

    /// Add a requirement to a specification
    pub async fn add_requirement(
        &self,
        spec_id: &str,
        requirement: RequirementDto,
    ) -> ApplicationResult<String> {
        let sid = SpecificationId::from_string(spec_id)
            .map_err(|_| ApplicationError::ValidationFailed("Invalid specification ID".into()))?;

        let mut spec = self
            .spec_repository
            .find_by_id(&sid)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?
            .ok_or_else(|| ApplicationError::SpecificationNotFound(spec_id.to_string()))?;

        // Add requirement - domain enforces that AC list is not empty
        let (req_id, _event) = spec
            .add_requirement(
                requirement.title.clone(),
                requirement.description.clone(),
                requirement.acceptance_criteria.clone(),
            )
            .map_err(|e| ApplicationError::DomainError(e.to_string()))?;

        // Persist
        let repo = Arc::clone(&self.spec_repository);
        let s = spec.clone();
        self.uow
            .execute(async move {
                repo.save(&s)
                    .await
                    .map_err(|e| ApplicationError::RepositoryError(e.to_string()))
            })
            .await?;

        Ok(req_id.to_string())
    }

    /// Add a task to a specification
    ///
    /// AC-3.3.3: Validates requirement traceability before accepting tasks
    pub async fn add_task(&self, spec_id: &str, task: TaskDto) -> ApplicationResult<String> {
        let sid = SpecificationId::from_string(spec_id)
            .map_err(|_| ApplicationError::ValidationFailed("Invalid specification ID".into()))?;

        let mut spec = self
            .spec_repository
            .find_by_id(&sid)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?
            .ok_or_else(|| ApplicationError::SpecificationNotFound(spec_id.to_string()))?;

        // Parse requirement IDs
        let requirement_refs: Vec<RequirementId> = task
            .requirement_ids
            .iter()
            .map(|id| RequirementId::from_string(id))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| ApplicationError::ValidationFailed("Invalid requirement ID".into()))?;

        // Add task - domain enforces requirement traceability
        let (task_id, _event) = spec
            .add_task(
                task.title.clone(),
                task.description.clone(),
                requirement_refs,
            )
            .map_err(|e| ApplicationError::DomainError(e.to_string()))?;

        // Persist
        let repo = Arc::clone(&self.spec_repository);
        let s = spec.clone();
        self.uow
            .execute(async move {
                repo.save(&s)
                    .await
                    .map_err(|e| ApplicationError::RepositoryError(e.to_string()))
            })
            .await?;

        Ok(task_id.to_string())
    }

    /// Complete a task
    pub async fn complete_task(&self, spec_id: &str, task_id: &str) -> ApplicationResult<()> {
        let sid = SpecificationId::from_string(spec_id)
            .map_err(|_| ApplicationError::ValidationFailed("Invalid specification ID".into()))?;

        let mut spec = self
            .spec_repository
            .find_by_id(&sid)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?
            .ok_or_else(|| ApplicationError::SpecificationNotFound(spec_id.to_string()))?;

        // Parse task_id and complete task
        let tid = TaskId::from_string(task_id)
            .map_err(|_| ApplicationError::ValidationFailed("Invalid task ID".into()))?;
        spec.complete_task(tid)
            .map_err(|e| ApplicationError::DomainError(e.to_string()))?;

        // Persist
        let repo = Arc::clone(&self.spec_repository);
        let s = spec.clone();
        self.uow
            .execute(async move {
                repo.save(&s)
                    .await
                    .map_err(|e| ApplicationError::RepositoryError(e.to_string()))
            })
            .await?;

        // Check if specification is complete (all tasks done)
        if (spec.completion_percentage() - 100.0).abs() < f32::EPSILON {
            self.events
                .publish(ApplicationEvent::SpecificationCompleted {
                    specification_id: spec_id.to_string(),
                    completion_percentage: spec.completion_percentage(),
                    timestamp: Utc::now(),
                })
                .await;
        }

        Ok(())
    }

    /// Get specifications by status
    pub async fn get_specifications_by_status(
        &self,
        status: SpecStatus,
    ) -> ApplicationResult<Vec<SpecificationSummaryDto>> {
        let specs = self
            .spec_repository
            .find_by_status(status)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?;

        Ok(specs
            .iter()
            .map(SpecificationSummaryDto::from_domain)
            .collect())
    }

    /// Delete a specification
    pub async fn delete_specification(&self, spec_id: &str) -> ApplicationResult<()> {
        let sid = SpecificationId::from_string(spec_id)
            .map_err(|_| ApplicationError::ValidationFailed("Invalid specification ID".into()))?;

        // Verify exists
        let exists = self
            .spec_repository
            .exists(&sid)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?;

        if !exists {
            return Err(ApplicationError::SpecificationNotFound(spec_id.to_string()));
        }

        // Delete within transaction
        let repo = Arc::clone(&self.spec_repository);
        let id = sid.clone();
        self.uow
            .execute(async move {
                repo.delete(&id)
                    .await
                    .map_err(|e| ApplicationError::RepositoryError(e.to_string()))
            })
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::NoOpEventPublisher;
    use crate::ports::NoOpUnitOfWork;
    use async_trait::async_trait;
    use ricecoder_domain::DomainResult;
    use ricecoder_domain::project::Project;
    use ricecoder_domain::value_objects::ProgrammingLanguage;
    use std::collections::HashMap;
    use std::sync::Mutex;

    /// In-memory specification repository for testing
    struct InMemorySpecificationRepository {
        specs: Mutex<HashMap<String, Specification>>,
    }

    impl InMemorySpecificationRepository {
        fn new() -> Self {
            Self {
                specs: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl SpecificationRepository for InMemorySpecificationRepository {
        async fn save(&self, spec: &Specification) -> DomainResult<()> {
            self.specs
                .lock()
                .unwrap()
                .insert(spec.id().to_string(), spec.clone());
            Ok(())
        }

        async fn find_by_id(&self, id: &SpecificationId) -> DomainResult<Option<Specification>> {
            Ok(self.specs.lock().unwrap().get(&id.to_string()).cloned())
        }

        async fn find_by_project(&self, project_id: &ProjectId) -> DomainResult<Vec<Specification>> {
            Ok(self
                .specs
                .lock()
                .unwrap()
                .values()
                .filter(|s| s.project_id() == *project_id)
                .cloned()
                .collect())
        }

        async fn find_all(&self) -> DomainResult<Vec<Specification>> {
            Ok(self.specs.lock().unwrap().values().cloned().collect())
        }

        async fn delete(&self, id: &SpecificationId) -> DomainResult<()> {
            self.specs.lock().unwrap().remove(&id.to_string());
            Ok(())
        }

        async fn exists(&self, id: &SpecificationId) -> DomainResult<bool> {
            Ok(self.specs.lock().unwrap().contains_key(&id.to_string()))
        }

        async fn find_by_status(&self, status: SpecStatus) -> DomainResult<Vec<Specification>> {
            Ok(self
                .specs
                .lock()
                .unwrap()
                .values()
                .filter(|s| s.status() == status)
                .cloned()
                .collect())
        }
    }

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

        fn add_project(&self, project: Project) {
            self.projects
                .lock()
                .unwrap()
                .insert(project.id().to_string(), project);
        }
    }

    #[async_trait]
    impl ProjectRepository for InMemoryProjectRepository {
        async fn save(&self, project: &Project) -> DomainResult<()> {
            self.projects
                .lock()
                .unwrap()
                .insert(project.id().to_string(), project.clone());
            Ok(())
        }

        async fn find_by_id(&self, id: &ProjectId) -> DomainResult<Option<Project>> {
            Ok(self.projects.lock().unwrap().get(&id.to_string()).cloned())
        }

        async fn find_all(&self) -> DomainResult<Vec<Project>> {
            Ok(self.projects.lock().unwrap().values().cloned().collect())
        }

        async fn delete(&self, id: &ProjectId) -> DomainResult<()> {
            self.projects.lock().unwrap().remove(&id.to_string());
            Ok(())
        }

        async fn exists(&self, id: &ProjectId) -> DomainResult<bool> {
            Ok(self
                .projects
                .lock()
                .unwrap()
                .contains_key(&id.to_string()))
        }
    }

    #[tokio::test]
    async fn test_create_specification_success() {
        let spec_repo = Arc::new(InMemorySpecificationRepository::new());
        let project_repo = Arc::new(InMemoryProjectRepository::new());
        let uow = Arc::new(NoOpUnitOfWork);
        let events = Arc::new(NoOpEventPublisher);

        // Create a project first
        let (project, _events) = Project::create(
            "test-project".to_string(),
            ProgrammingLanguage::Rust,
            "/path".to_string(),
            None,
        ).unwrap();
        let project_id = project.id().to_string();
        project_repo.add_project(project);

        let service = SpecificationService::new(spec_repo, project_repo, uow, events);

        let cmd = CreateSpecificationCommand {
            name: "test-spec".to_string(),
            project_id: project_id.clone(),
            description: "A test specification".to_string(),
            version: Some("1.0.0".to_string()),
        };

        let result = service.create_specification(cmd).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_specification_empty_name() {
        let spec_repo = Arc::new(InMemorySpecificationRepository::new());
        let project_repo = Arc::new(InMemoryProjectRepository::new());
        let uow = Arc::new(NoOpUnitOfWork);
        let events = Arc::new(NoOpEventPublisher);

        let service = SpecificationService::new(spec_repo, project_repo, uow, events);

        let cmd = CreateSpecificationCommand {
            name: "".to_string(),
            project_id: "00000000-0000-0000-0000-000000000000".to_string(),
            description: "Test description".to_string(),
            version: None,
        };

        let result = service.create_specification(cmd).await;
        assert!(matches!(result, Err(ApplicationError::ValidationFailed(_))));
    }

    #[tokio::test]
    async fn test_create_specification_project_not_found() {
        let spec_repo = Arc::new(InMemorySpecificationRepository::new());
        let project_repo = Arc::new(InMemoryProjectRepository::new());
        let uow = Arc::new(NoOpUnitOfWork);
        let events = Arc::new(NoOpEventPublisher);

        let service = SpecificationService::new(spec_repo, project_repo, uow, events);

        let cmd = CreateSpecificationCommand {
            name: "test-spec".to_string(),
            project_id: "00000000-0000-0000-0000-000000000000".to_string(),
            description: "Test description".to_string(),
            version: None,
        };

        let result = service.create_specification(cmd).await;
        assert!(matches!(result, Err(ApplicationError::ProjectNotFound(_))));
    }
}
