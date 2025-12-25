//! Integration tests for application services
//!
//! These tests verify service behavior with mock dependencies.

use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;

use ricecoder_application::dto::{CreateProjectCommand, CreateSessionCommand};
use ricecoder_application::errors::{ApplicationError, ApplicationResult};
use ricecoder_application::events::{ApplicationEvent, EventPublisher};
use ricecoder_application::ports::UnitOfWork;
use ricecoder_application::services::{ProjectService, SessionService};

use ricecoder_domain::errors::DomainResult;
use ricecoder_domain::events::DomainEvent;
use ricecoder_domain::project::Project;
use ricecoder_domain::session::Session;
use ricecoder_domain::repositories::{ProjectRepository, SessionRepository};
use ricecoder_domain::value_objects::{ProgrammingLanguage, ProjectId, SessionId};

// ============================================================================
// Mock Implementations
// ============================================================================

/// In-memory project repository for testing
#[derive(Default)]
struct MockProjectRepository {
    projects: RwLock<HashMap<String, Project>>,
}

#[async_trait]
impl ProjectRepository for MockProjectRepository {
    async fn find_by_id(&self, id: &ProjectId) -> DomainResult<Option<Project>> {
        let projects = self.projects.read().unwrap();
        Ok(projects.get(&id.to_string()).cloned())
    }

    async fn find_all(&self) -> DomainResult<Vec<Project>> {
        let projects = self.projects.read().unwrap();
        Ok(projects.values().cloned().collect())
    }

    async fn save(&self, project: &Project) -> DomainResult<()> {
        let mut projects = self.projects.write().unwrap();
        projects.insert(project.id().to_string(), project.clone());
        Ok(())
    }

    async fn delete(&self, id: &ProjectId) -> DomainResult<()> {
        let mut projects = self.projects.write().unwrap();
        projects.remove(&id.to_string());
        Ok(())
    }

    async fn exists(&self, id: &ProjectId) -> DomainResult<bool> {
        let projects = self.projects.read().unwrap();
        Ok(projects.contains_key(&id.to_string()))
    }
}

/// In-memory session repository for testing
#[derive(Default)]
struct MockSessionRepository {
    sessions: RwLock<HashMap<String, Session>>,
}

#[async_trait]
impl SessionRepository for MockSessionRepository {
    async fn find_by_id(&self, id: &SessionId) -> DomainResult<Option<Session>> {
        let sessions = self.sessions.read().unwrap();
        Ok(sessions.get(&id.to_string()).cloned())
    }

    async fn find_by_project(&self, project_id: &ProjectId) -> DomainResult<Vec<Session>> {
        let sessions = self.sessions.read().unwrap();
        Ok(sessions
            .values()
            .filter(|s| s.project_id() == *project_id)
            .cloned()
            .collect())
    }

    async fn find_active(&self) -> DomainResult<Vec<Session>> {
        let sessions = self.sessions.read().unwrap();
        Ok(sessions.values().filter(|s| s.is_active()).cloned().collect())
    }

    async fn save(&self, session: &Session) -> DomainResult<()> {
        let mut sessions = self.sessions.write().unwrap();
        sessions.insert(session.id().to_string(), session.clone());
        Ok(())
    }

    async fn delete(&self, id: &SessionId) -> DomainResult<()> {
        let mut sessions = self.sessions.write().unwrap();
        sessions.remove(&id.to_string());
        Ok(())
    }
}

/// Mock unit of work using NoOpUnitOfWork pattern
struct MockUnitOfWork;

#[async_trait]
impl UnitOfWork for MockUnitOfWork {
    async fn execute<T, F>(&self, f: F) -> ApplicationResult<T>
    where
        T: Send + 'static,
        F: Future<Output = ApplicationResult<T>> + Send + 'static,
    {
        f.await
    }
}

/// Mock event publisher that collects events
#[derive(Default)]
struct MockEventPublisher {
    events: RwLock<Vec<ApplicationEvent>>,
}

#[async_trait]
impl EventPublisher for MockEventPublisher {
    async fn publish(&self, event: ApplicationEvent) {
        let mut events = self.events.write().unwrap();
        events.push(event);
    }

    async fn publish_domain_event(&self, _event: Box<dyn DomainEvent>) {
        // No-op for testing
    }
}

impl MockEventPublisher {
    fn event_count(&self) -> usize {
        self.events.read().unwrap().len()
    }

    #[allow(dead_code)]
    fn last_event(&self) -> Option<ApplicationEvent> {
        self.events.read().unwrap().last().cloned()
    }
}

// ============================================================================
// ProjectService Tests
// ============================================================================

#[cfg(test)]
mod project_service_tests {
    use super::*;

    fn create_project_service() -> (
        ProjectService<MockProjectRepository, MockUnitOfWork, MockEventPublisher>,
        Arc<MockProjectRepository>,
        Arc<MockEventPublisher>,
    ) {
        let repo = Arc::new(MockProjectRepository::default());
        let uow = Arc::new(MockUnitOfWork);
        let events = Arc::new(MockEventPublisher::default());

        let service = ProjectService::new(Arc::clone(&repo), uow, Arc::clone(&events));

        (service, repo, events)
    }

    #[tokio::test]
    async fn test_create_project_success() {
        let (service, repo, events) = create_project_service();

        let cmd = CreateProjectCommand {
            name: "test-project".to_string(),
            language: ProgrammingLanguage::Rust,
            root_path: "/path/to/project".to_string(),
            description: Some("Test description".to_string()),
        };

        let result = service.create_project(cmd).await;

        assert!(result.is_ok());
        let project_id = result.unwrap();
        assert!(!project_id.is_empty());

        // Verify project was saved
        let saved = repo.projects.read().unwrap();
        assert_eq!(saved.len(), 1);

        // Verify event was published
        assert_eq!(events.event_count(), 1);
    }

    #[tokio::test]
    async fn test_create_project_empty_name_fails() {
        let (service, _, _) = create_project_service();

        let cmd = CreateProjectCommand {
            name: "   ".to_string(), // Empty after trim
            language: ProgrammingLanguage::Rust,
            root_path: "/path/to/project".to_string(),
            description: None,
        };

        let result = service.create_project(cmd).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ApplicationError::ValidationFailed(_)
        ));
    }

    #[tokio::test]
    async fn test_create_project_empty_path_fails() {
        let (service, _, _) = create_project_service();

        let cmd = CreateProjectCommand {
            name: "test-project".to_string(),
            language: ProgrammingLanguage::Rust,
            root_path: "".to_string(),
            description: None,
        };

        let result = service.create_project(cmd).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ApplicationError::RequiredFieldMissing(_)
        ));
    }

    #[tokio::test]
    async fn test_get_project_not_found() {
        let (service, _, _) = create_project_service();

        let result = service
            .get_project("550e8400-e29b-41d4-a716-446655440000")
            .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ApplicationError::ProjectNotFound(_)
        ));
    }

    #[tokio::test]
    async fn test_get_project_invalid_id() {
        let (service, _, _) = create_project_service();

        let result = service.get_project("not-a-uuid").await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ApplicationError::ValidationFailed(_)
        ));
    }
}

// ============================================================================
// SessionService Tests
// ============================================================================

#[cfg(test)]
mod session_service_tests {
    use super::*;

    fn create_session_service() -> (
        SessionService<MockSessionRepository, MockProjectRepository, MockUnitOfWork, MockEventPublisher>,
        Arc<MockSessionRepository>,
        Arc<MockProjectRepository>,
        Arc<MockEventPublisher>,
    ) {
        let session_repo = Arc::new(MockSessionRepository::default());
        let project_repo = Arc::new(MockProjectRepository::default());
        let uow = Arc::new(MockUnitOfWork);
        let events = Arc::new(MockEventPublisher::default());

        let service = SessionService::new(
            Arc::clone(&session_repo),
            Arc::clone(&project_repo),
            uow,
            Arc::clone(&events),
        );

        (service, session_repo, project_repo, events)
    }

    async fn create_test_project(repo: &MockProjectRepository) -> String {
        let (project, _) = Project::create(
            "test-project".to_string(),
            ProgrammingLanguage::Rust,
            "/test/path".to_string(),
            None,
        )
        .unwrap();
        let id = project.id().to_string();
        repo.save(&project).await.unwrap();
        id
    }

    #[tokio::test]
    async fn test_create_session_success() {
        let (service, session_repo, project_repo, events) = create_session_service();

        // Create a project first
        let project_id = create_test_project(&project_repo).await;

        let cmd = CreateSessionCommand {
            project_id: project_id.clone(),
            max_messages: Some(50),
        };

        let result = service.create_session(cmd).await;

        assert!(result.is_ok());
        let session_id = result.unwrap();
        assert!(!session_id.is_empty());

        // Verify session was saved
        let saved = session_repo.sessions.read().unwrap();
        assert_eq!(saved.len(), 1);

        // Verify event was published
        assert_eq!(events.event_count(), 1);
    }

    #[tokio::test]
    async fn test_create_session_project_not_found() {
        let (service, _, _, _) = create_session_service();

        let cmd = CreateSessionCommand {
            project_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            max_messages: None,
        };

        let result = service.create_session(cmd).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ApplicationError::ProjectNotFound(_)
        ));
    }

    #[tokio::test]
    async fn test_create_session_invalid_project_id() {
        let (service, _, _, _) = create_session_service();

        let cmd = CreateSessionCommand {
            project_id: "not-a-uuid".to_string(),
            max_messages: None,
        };

        let result = service.create_session(cmd).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ApplicationError::ValidationFailed(_)
        ));
    }

    #[tokio::test]
    async fn test_get_session_not_found() {
        let (service, _, _, _) = create_session_service();

        let result = service
            .get_session("550e8400-e29b-41d4-a716-446655440000")
            .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ApplicationError::SessionNotFound(_)
        ));
    }
}
