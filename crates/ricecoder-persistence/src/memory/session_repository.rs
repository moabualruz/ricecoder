//! In-Memory Session Repository Implementation
//!
//! Memory backend for tests

use async_trait::async_trait;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

use ricecoder_domain::{
    errors::DomainResult,
    repositories::SessionRepository,
    session::{Session, SessionState},
    value_objects::{ProjectId, SessionId},
};

/// Thread-safe in-memory implementation of SessionRepository
///
/// Uses RwLock for concurrent read access with exclusive write access.
/// Stores cloned Session instances to maintain isolation.
#[derive(Debug, Default)]
pub struct InMemorySessionRepository {
    sessions: RwLock<HashMap<SessionId, Session>>,
}

impl InMemorySessionRepository {
    /// Create a new empty in-memory session repository
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
        }
    }

    /// Create with initial sessions (useful for testing)
    pub fn with_sessions(sessions: Vec<Session>) -> Self {
        let map: HashMap<SessionId, Session> = sessions
            .into_iter()
            .map(|s| (s.id().clone(), s))
            .collect();
        Self {
            sessions: RwLock::new(map),
        }
    }

    /// Get the current count of sessions (for testing)
    pub fn count(&self) -> usize {
        self.sessions.read().len()
    }

    /// Clear all sessions (for testing)
    pub fn clear(&self) {
        self.sessions.write().clear();
    }
}

#[async_trait]
impl SessionRepository for InMemorySessionRepository {
    async fn save(&self, session: &Session) -> DomainResult<()> {
        let mut sessions = self.sessions.write();
        sessions.insert(session.id().clone(), session.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: &SessionId) -> DomainResult<Option<Session>> {
        let sessions = self.sessions.read();
        Ok(sessions.get(id).cloned())
    }

    async fn find_by_project(&self, project_id: &ProjectId) -> DomainResult<Vec<Session>> {
        let sessions = self.sessions.read();
        Ok(sessions
            .values()
            .filter(|s| s.project_id() == *project_id)
            .cloned()
            .collect())
    }

    async fn find_active(&self) -> DomainResult<Vec<Session>> {
        let sessions = self.sessions.read();
        Ok(sessions
            .values()
            .filter(|s| matches!(s.state(), SessionState::Active))
            .cloned()
            .collect())
    }

    async fn delete(&self, id: &SessionId) -> DomainResult<()> {
        let mut sessions = self.sessions.write();
        sessions.remove(id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_domain::session::Session;
    use ricecoder_domain::value_objects::ProjectId;

    fn create_test_session(project_id: &ProjectId) -> Session {
        let (session, _events) = Session::create(project_id.clone(), 100).unwrap();
        session
    }

    #[tokio::test]
    async fn test_save_and_find_by_id() {
        let repo = InMemorySessionRepository::new();
        let project_id = ProjectId::new();
        let session = create_test_session(&project_id);
        let id = session.id().clone();

        repo.save(&session).await.unwrap();

        let found = repo.find_by_id(&id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id(), session.id());
    }

    #[tokio::test]
    async fn test_find_by_id_not_found() {
        let repo = InMemorySessionRepository::new();
        let id = SessionId::new();

        let found = repo.find_by_id(&id).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_find_by_project() {
        let repo = InMemorySessionRepository::new();
        let project1 = ProjectId::new();
        let project2 = ProjectId::new();

        let s1 = create_test_session(&project1);
        let s2 = create_test_session(&project1);
        let s3 = create_test_session(&project2);

        repo.save(&s1).await.unwrap();
        repo.save(&s2).await.unwrap();
        repo.save(&s3).await.unwrap();

        let project1_sessions = repo.find_by_project(&project1).await.unwrap();
        assert_eq!(project1_sessions.len(), 2);

        let project2_sessions = repo.find_by_project(&project2).await.unwrap();
        assert_eq!(project2_sessions.len(), 1);
    }

    #[tokio::test]
    async fn test_find_active() {
        let repo = InMemorySessionRepository::new();
        let project_id = ProjectId::new();

        let s1 = create_test_session(&project_id);
        let mut s2 = create_test_session(&project_id);

        // Complete s2 to make it non-active
        s2.complete().unwrap();

        repo.save(&s1).await.unwrap();
        repo.save(&s2).await.unwrap();

        let active = repo.find_active().await.unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id(), s1.id());
    }

    #[tokio::test]
    async fn test_delete() {
        let repo = InMemorySessionRepository::new();
        let project_id = ProjectId::new();
        let session = create_test_session(&project_id);
        let id = session.id().clone();

        repo.save(&session).await.unwrap();
        assert!(repo.find_by_id(&id).await.unwrap().is_some());

        repo.delete(&id).await.unwrap();
        assert!(repo.find_by_id(&id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_count_and_clear() {
        let repo = InMemorySessionRepository::new();
        let project_id = ProjectId::new();

        assert_eq!(repo.count(), 0);

        repo.save(&create_test_session(&project_id)).await.unwrap();
        repo.save(&create_test_session(&project_id)).await.unwrap();
        assert_eq!(repo.count(), 2);

        repo.clear();
        assert_eq!(repo.count(), 0);
    }
}
