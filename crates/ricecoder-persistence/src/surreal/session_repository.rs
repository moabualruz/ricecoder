//! SurrealDB Session Repository Implementation

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::debug;

use ricecoder_domain::{
    errors::{DomainError, DomainResult},
    repositories::SessionRepository,
    session::{Message, MessageRole, Session, SessionState},
    value_objects::{ProjectId, SessionId},
};

use super::connection::{DatabaseClient, SharedConnection};

/// SurrealDB table name for sessions
const TABLE_NAME: &str = "sessions";

/// Serializable message record for SurrealDB
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MessageRecord {
    id: String,
    content: String,
    role: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<&Message> for MessageRecord {
    fn from(msg: &Message) -> Self {
        let role = match msg.role() {
            MessageRole::User => "User",
            MessageRole::Assistant => "Assistant",
            MessageRole::System => "System",
        };
        Self {
            id: msg.id().to_string(),
            content: msg.content().to_string(),
            role: role.to_string(),
            created_at: msg.created_at(),
        }
    }
}

/// Serializable session record for SurrealDB
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SessionRecord {
    id: String,
    project_id: String,
    messages: Vec<MessageRecord>,
    state: String,
    max_messages: usize,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    version: u64,
}

impl From<&Session> for SessionRecord {
    fn from(session: &Session) -> Self {
        let state = match session.state() {
            SessionState::Active => "Active",
            SessionState::Paused => "Paused",
            SessionState::Completed => "Completed",
            SessionState::Archived => "Archived",
        };
        Self {
            id: session.id().to_string(),
            project_id: session.project_id().to_string(),
            messages: session.messages().iter().map(MessageRecord::from).collect(),
            state: state.to_string(),
            max_messages: session.max_messages(),
            created_at: session.created_at(),
            updated_at: session.updated_at(),
            version: session.version(),
        }
    }
}

/// Helper to convert SurrealDB errors to DomainError
fn to_domain_error(e: surrealdb::Error) -> DomainError {
    DomainError::ExternalServiceError {
        service: "SurrealDB".to_string(),
        reason: e.to_string(),
    }
}

/// SurrealDB implementation of SessionRepository
pub struct SurrealSessionRepository {
    connection: SharedConnection,
}

impl SurrealSessionRepository {
    /// Create a new SurrealDB session repository
    pub fn new(connection: SharedConnection) -> Self {
        Self { connection }
    }
}

#[async_trait]
impl SessionRepository for SurrealSessionRepository {
    async fn save(&self, session: &Session) -> DomainResult<()> {
        let record = SessionRecord::from(session);
        let id = &record.id;
        
        debug!("Saving session {} to SurrealDB", id);

        match self.connection.client() {
            DatabaseClient::Local(db) => {
                let _: Option<SessionRecord> = db
                    .upsert((TABLE_NAME, id.as_str()))
                    .content(record.clone())
                    .await
                    .map_err(to_domain_error)?;
            }
            DatabaseClient::Remote(db) => {
                let _: Option<SessionRecord> = db
                    .upsert((TABLE_NAME, id.as_str()))
                    .content(record)
                    .await
                    .map_err(to_domain_error)?;
            }
        }

        Ok(())
    }

    async fn find_by_id(&self, id: &SessionId) -> DomainResult<Option<Session>> {
        debug!("Finding session by id: {}", id);
        
        let record: Option<SessionRecord> = match self.connection.client() {
            DatabaseClient::Local(db) => {
                db.select((TABLE_NAME, id.to_string().as_str()))
                    .await
                    .map_err(to_domain_error)?
            }
            DatabaseClient::Remote(db) => {
                db.select((TABLE_NAME, id.to_string().as_str()))
                    .await
                    .map_err(to_domain_error)?
            }
        };

        match record {
            Some(r) => {
                let session = reconstitute_session(r)?;
                Ok(Some(session))
            }
            None => Ok(None),
        }
    }

    async fn find_by_project(&self, project_id: &ProjectId) -> DomainResult<Vec<Session>> {
        debug!("Finding sessions for project: {}", project_id);
        
        let query = format!(
            "SELECT * FROM {} WHERE project_id = '{}'",
            TABLE_NAME,
            project_id
        );

        let records: Vec<SessionRecord> = match self.connection.client() {
            DatabaseClient::Local(db) => {
                let mut response = db.query(&query).await
                    .map_err(to_domain_error)?;
                response.take(0)
                    .map_err(to_domain_error)?
            }
            DatabaseClient::Remote(db) => {
                let mut response = db.query(&query).await
                    .map_err(to_domain_error)?;
                response.take(0)
                    .map_err(to_domain_error)?
            }
        };

        let mut sessions = Vec::with_capacity(records.len());
        for r in records {
            let session = reconstitute_session(r)?;
            sessions.push(session);
        }

        Ok(sessions)
    }

    async fn find_active(&self) -> DomainResult<Vec<Session>> {
        debug!("Finding active sessions");
        
        let query = format!(
            "SELECT * FROM {} WHERE state = 'Active'",
            TABLE_NAME
        );

        let records: Vec<SessionRecord> = match self.connection.client() {
            DatabaseClient::Local(db) => {
                let mut response = db.query(&query).await
                    .map_err(to_domain_error)?;
                response.take(0)
                    .map_err(to_domain_error)?
            }
            DatabaseClient::Remote(db) => {
                let mut response = db.query(&query).await
                    .map_err(to_domain_error)?;
                response.take(0)
                    .map_err(to_domain_error)?
            }
        };

        let mut sessions = Vec::with_capacity(records.len());
        for r in records {
            let session = reconstitute_session(r)?;
            sessions.push(session);
        }

        Ok(sessions)
    }

    async fn delete(&self, id: &SessionId) -> DomainResult<()> {
        debug!("Deleting session: {}", id);
        
        match self.connection.client() {
            DatabaseClient::Local(db) => {
                let _: Option<SessionRecord> = db
                    .delete((TABLE_NAME, id.to_string().as_str()))
                    .await
                    .map_err(to_domain_error)?;
            }
            DatabaseClient::Remote(db) => {
                let _: Option<SessionRecord> = db
                    .delete((TABLE_NAME, id.to_string().as_str()))
                    .await
                    .map_err(to_domain_error)?;
            }
        }

        Ok(())
    }
}

/// Reconstitute a Session from a database record
fn reconstitute_session(r: SessionRecord) -> DomainResult<Session> {
    let id = SessionId::from_string(&r.id)
        .map_err(|e| DomainError::ExternalServiceError {
            service: "SurrealDB".to_string(),
            reason: format!("Invalid session ID: {}", e),
        })?;
    
    let project_id = ProjectId::from_string(&r.project_id)
        .map_err(|e| DomainError::ExternalServiceError {
            service: "SurrealDB".to_string(),
            reason: format!("Invalid project ID: {}", e),
        })?;
    
    let state = parse_session_state(&r.state)?;
    
    // Reconstitute messages
    let mut messages = Vec::with_capacity(r.messages.len());
    for msg_record in r.messages {
        let role = parse_message_role(&msg_record.role)?;
        let message = Message::reconstitute(
            msg_record.id,
            msg_record.content,
            role,
            msg_record.created_at,
        );
        messages.push(message);
    }
    
    Ok(Session::reconstitute(
        id,
        project_id,
        messages,
        state,
        r.max_messages,
        r.created_at,
        r.updated_at,
        r.version,
    ))
}

/// Parse state string back to SessionState
fn parse_session_state(s: &str) -> DomainResult<SessionState> {
    match s {
        "Active" => Ok(SessionState::Active),
        "Paused" => Ok(SessionState::Paused),
        "Completed" => Ok(SessionState::Completed),
        "Archived" => Ok(SessionState::Archived),
        _ => Err(DomainError::ExternalServiceError {
            service: "SurrealDB".to_string(),
            reason: format!("Invalid session state: {}", s),
        }),
    }
}

/// Parse role string back to MessageRole
fn parse_message_role(s: &str) -> DomainResult<MessageRole> {
    match s {
        "User" => Ok(MessageRole::User),
        "Assistant" => Ok(MessageRole::Assistant),
        "System" => Ok(MessageRole::System),
        _ => Err(DomainError::ExternalServiceError {
            service: "SurrealDB".to_string(),
            reason: format!("Invalid message role: {}", s),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::surreal::connection::{ConnectionMode, SurrealConnection};

    async fn create_test_repo() -> SurrealSessionRepository {
        let conn = SurrealConnection::new(ConnectionMode::Memory)
            .await
            .expect("Failed to create connection");
        SurrealSessionRepository::new(Arc::new(conn))
    }

    fn create_test_session() -> Session {
        let project_id = ProjectId::new();
        let (session, _) = Session::create(project_id, 100).unwrap();
        session
    }

    #[tokio::test]
    async fn test_save_and_find() {
        let repo = create_test_repo().await;
        let session = create_test_session();
        let id = session.id();

        repo.save(&session).await.unwrap();
        
        let found = repo.find_by_id(&id).await.unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.max_messages(), 100);
        assert!(found.is_active());
    }

    #[tokio::test]
    async fn test_find_active() {
        let repo = create_test_repo().await;
        
        let s1 = create_test_session();
        let mut s2 = create_test_session();
        s2.pause().unwrap();
        
        repo.save(&s1).await.unwrap();
        repo.save(&s2).await.unwrap();
        
        let active = repo.find_active().await.unwrap();
        assert_eq!(active.len(), 1);
        assert!(active[0].is_active());
    }

    #[tokio::test]
    async fn test_find_by_project() {
        let repo = create_test_repo().await;
        
        let session = create_test_session();
        let project_id = session.project_id();
        
        repo.save(&session).await.unwrap();
        
        let found = repo.find_by_project(&project_id).await.unwrap();
        assert_eq!(found.len(), 1);
    }

    #[tokio::test]
    async fn test_delete() {
        let repo = create_test_repo().await;
        let session = create_test_session();
        let id = session.id();

        repo.save(&session).await.unwrap();
        
        let found = repo.find_by_id(&id).await.unwrap();
        assert!(found.is_some());

        repo.delete(&id).await.unwrap();
        
        let found = repo.find_by_id(&id).await.unwrap();
        assert!(found.is_none());
    }
}
