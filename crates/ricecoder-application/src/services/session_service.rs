//! Session Application Service
//!
//! Orchestrates session-related use cases using domain aggregates.
//!
//! REQ-APP-020: SessionService
//! AC-3.2.1: Stateless and reusable across requests
//! AC-3.2.2: Depends only on repository abstractions
//! AC-3.2.3: Manages transaction boundaries for multi-step operations

use std::sync::Arc;

use chrono::Utc;

use crate::dto::{
    CreateSessionCommand, MessageDto, SessionDetailDto, SessionSummaryDto,
};
use crate::errors::{ApplicationError, ApplicationResult};
use crate::events::{ApplicationEvent, EventPublisher};
use crate::ports::UnitOfWork;

use ricecoder_domain::session::{Session, MessageRole};
use ricecoder_domain::value_objects::{ProjectId, SessionId};
use ricecoder_domain::repositories::{SessionRepository, ProjectRepository};

/// Session Application Service
///
/// Orchestrates session-related use cases. Stateless: all mutable state
/// is persisted via repositories.
pub struct SessionService<SR, PR, U, E>
where
    SR: SessionRepository + Send + Sync,
    PR: ProjectRepository + Send + Sync,
    U: UnitOfWork + Send + Sync,
    E: EventPublisher + Send + Sync,
{
    session_repository: Arc<SR>,
    project_repository: Arc<PR>,
    uow: Arc<U>,
    events: Arc<E>,
}

impl<SR, PR, U, E> SessionService<SR, PR, U, E>
where
    SR: SessionRepository + Send + Sync + 'static,
    PR: ProjectRepository + Send + Sync + 'static,
    U: UnitOfWork + Send + Sync,
    E: EventPublisher + Send + Sync,
{
    /// Create a new SessionService with injected dependencies
    pub fn new(
        session_repository: Arc<SR>,
        project_repository: Arc<PR>,
        uow: Arc<U>,
        events: Arc<E>,
    ) -> Self {
        Self {
            session_repository,
            project_repository,
            uow,
            events,
        }
    }

    /// Create a new session for a project
    pub async fn create_session(&self, cmd: CreateSessionCommand) -> ApplicationResult<String> {
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

        // Create session (default max_messages = 100)
        let max_messages = cmd.max_messages.unwrap_or(100);
        let (session, _events) = Session::create(project_id.clone(), max_messages)
            .map_err(|e| ApplicationError::DomainError(e.to_string()))?;

        let session_id = session.id().to_string();

        // Persist within transaction
        let repo = Arc::clone(&self.session_repository);
        let s = session.clone();
        self.uow
            .execute(async move {
                repo.save(&s)
                    .await
                    .map_err(|e| ApplicationError::RepositoryError(e.to_string()))
            })
            .await?;

        // Emit event
        self.events
            .publish(ApplicationEvent::SessionStarted {
                session_id: session_id.clone(),
                project_id: project_id.to_string(),
                timestamp: Utc::now(),
            })
            .await;

        Ok(session_id)
    }

    /// Get session by ID
    pub async fn get_session(&self, id: &str) -> ApplicationResult<SessionDetailDto> {
        let session_id = SessionId::from_string(id)
            .map_err(|_| ApplicationError::ValidationFailed("Invalid session ID".into()))?;

        let session = self
            .session_repository
            .find_by_id(&session_id)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?
            .ok_or_else(|| ApplicationError::SessionNotFound(id.to_string()))?;

        Ok(SessionDetailDto::from_domain(&session))
    }

    /// List sessions for a project
    pub async fn list_sessions_for_project(
        &self,
        project_id: &str,
    ) -> ApplicationResult<Vec<SessionSummaryDto>> {
        let pid = ProjectId::from_string(project_id)
            .map_err(|_| ApplicationError::ValidationFailed("Invalid project ID".into()))?;

        let sessions = self
            .session_repository
            .find_by_project(&pid)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?;

        Ok(sessions
            .iter()
            .map(SessionSummaryDto::from_domain)
            .collect())
    }

    /// List active sessions
    pub async fn list_active_sessions(&self) -> ApplicationResult<Vec<SessionSummaryDto>> {
        let sessions = self
            .session_repository
            .find_active()
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?;

        Ok(sessions
            .iter()
            .map(SessionSummaryDto::from_domain)
            .collect())
    }

    /// Add a message to a session
    pub async fn add_message(
        &self,
        session_id: &str,
        role: MessageRole,
        content: String,
    ) -> ApplicationResult<MessageDto> {
        let sid = SessionId::from_string(session_id)
            .map_err(|_| ApplicationError::ValidationFailed("Invalid session ID".into()))?;

        let mut session = self
            .session_repository
            .find_by_id(&sid)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?
            .ok_or_else(|| ApplicationError::SessionNotFound(session_id.to_string()))?;

        // Add message to session (domain method: content, role)
        session
            .add_message(content.clone(), role)
            .map_err(|e| ApplicationError::DomainError(e.to_string()))?;

        // Persist
        let repo = Arc::clone(&self.session_repository);
        let s = session.clone();
        self.uow
            .execute(async move {
                repo.save(&s)
                    .await
                    .map_err(|e| ApplicationError::RepositoryError(e.to_string()))
            })
            .await?;

        // Return the message DTO (get the last added message)
        let last_message = session.messages().last().expect("Message just added");
        Ok(MessageDto {
            id: last_message.id().to_string(),
            role: format!("{:?}", last_message.role()),
            content: last_message.content().to_string(),
            created_at: last_message.created_at(),
        })
    }

    /// Complete a session
    pub async fn complete_session(&self, session_id: &str) -> ApplicationResult<()> {
        let sid = SessionId::from_string(session_id)
            .map_err(|_| ApplicationError::ValidationFailed("Invalid session ID".into()))?;

        let mut session = self
            .session_repository
            .find_by_id(&sid)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?
            .ok_or_else(|| ApplicationError::SessionNotFound(session_id.to_string()))?;

        let message_count = session.message_count();

        // Complete the session
        session
            .complete()
            .map_err(|e| ApplicationError::DomainError(e.to_string()))?;

        // Persist
        let repo = Arc::clone(&self.session_repository);
        let s = session.clone();
        self.uow
            .execute(async move {
                repo.save(&s)
                    .await
                    .map_err(|e| ApplicationError::RepositoryError(e.to_string()))
            })
            .await?;

        // Emit event
        self.events
            .publish(ApplicationEvent::SessionCompleted {
                session_id: session_id.to_string(),
                message_count,
                timestamp: Utc::now(),
            })
            .await;

        Ok(())
    }

    /// Pause a session
    pub async fn pause_session(&self, session_id: &str) -> ApplicationResult<()> {
        let sid = SessionId::from_string(session_id)
            .map_err(|_| ApplicationError::ValidationFailed("Invalid session ID".into()))?;

        let mut session = self
            .session_repository
            .find_by_id(&sid)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?
            .ok_or_else(|| ApplicationError::SessionNotFound(session_id.to_string()))?;

        // Pause the session
        session
            .pause()
            .map_err(|e| ApplicationError::DomainError(e.to_string()))?;

        // Persist
        let repo = Arc::clone(&self.session_repository);
        let s = session.clone();
        self.uow
            .execute(async move {
                repo.save(&s)
                    .await
                    .map_err(|e| ApplicationError::RepositoryError(e.to_string()))
            })
            .await?;

        Ok(())
    }

    /// Resume a paused session
    pub async fn resume_session(&self, session_id: &str) -> ApplicationResult<()> {
        let sid = SessionId::from_string(session_id)
            .map_err(|_| ApplicationError::ValidationFailed("Invalid session ID".into()))?;

        let mut session = self
            .session_repository
            .find_by_id(&sid)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?
            .ok_or_else(|| ApplicationError::SessionNotFound(session_id.to_string()))?;

        // Resume the session
        session
            .resume()
            .map_err(|e| ApplicationError::DomainError(e.to_string()))?;

        // Persist
        let repo = Arc::clone(&self.session_repository);
        let s = session.clone();
        self.uow
            .execute(async move {
                repo.save(&s)
                    .await
                    .map_err(|e| ApplicationError::RepositoryError(e.to_string()))
            })
            .await?;

        Ok(())
    }

    /// Delete a session
    pub async fn delete_session(&self, session_id: &str) -> ApplicationResult<()> {
        let sid = SessionId::from_string(session_id)
            .map_err(|_| ApplicationError::ValidationFailed("Invalid session ID".into()))?;

        // Delete within transaction
        let repo = Arc::clone(&self.session_repository);
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

    /// In-memory session repository for testing
    struct InMemorySessionRepository {
        sessions: Mutex<HashMap<String, Session>>,
    }

    impl InMemorySessionRepository {
        fn new() -> Self {
            Self {
                sessions: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl SessionRepository for InMemorySessionRepository {
        async fn save(&self, session: &Session) -> DomainResult<()> {
            self.sessions
                .lock()
                .unwrap()
                .insert(session.id().to_string(), session.clone());
            Ok(())
        }

        async fn find_by_id(&self, id: &SessionId) -> DomainResult<Option<Session>> {
            Ok(self.sessions.lock().unwrap().get(&id.to_string()).cloned())
        }

        async fn find_by_project(&self, project_id: &ProjectId) -> DomainResult<Vec<Session>> {
            Ok(self
                .sessions
                .lock()
                .unwrap()
                .values()
                .filter(|s| s.project_id() == *project_id)
                .cloned()
                .collect())
        }

        async fn find_active(&self) -> DomainResult<Vec<Session>> {
            Ok(self
                .sessions
                .lock()
                .unwrap()
                .values()
                .filter(|s| s.is_active())
                .cloned()
                .collect())
        }

        async fn delete(&self, id: &SessionId) -> DomainResult<()> {
            self.sessions.lock().unwrap().remove(&id.to_string());
            Ok(())
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
    async fn test_create_session_success() {
        let session_repo = Arc::new(InMemorySessionRepository::new());
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

        let service = SessionService::new(session_repo, project_repo, uow, events);

        let cmd = CreateSessionCommand {
            project_id: project_id.clone(),
            max_messages: Some(100),
        };

        let result = service.create_session(cmd).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_session_project_not_found() {
        let session_repo = Arc::new(InMemorySessionRepository::new());
        let project_repo = Arc::new(InMemoryProjectRepository::new());
        let uow = Arc::new(NoOpUnitOfWork);
        let events = Arc::new(NoOpEventPublisher);

        let service = SessionService::new(session_repo, project_repo, uow, events);

        let cmd = CreateSessionCommand {
            project_id: "00000000-0000-0000-0000-000000000000".to_string(),
            max_messages: Some(100),
        };

        let result = service.create_session(cmd).await;
        assert!(matches!(result, Err(ApplicationError::ProjectNotFound(_))));
    }
}
