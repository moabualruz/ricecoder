//! SurrealDB Project Repository Implementation

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::debug;

use ricecoder_domain::{
    errors::{DomainError, DomainResult},
    project::{Project, ProjectStatus},
    repositories::ProjectRepository,
    value_objects::{ProjectId, ProgrammingLanguage},
};

use super::connection::{DatabaseClient, SharedConnection};

/// SurrealDB table name for projects
const TABLE_NAME: &str = "projects";

/// Serializable project record for SurrealDB
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProjectRecord {
    id: String,
    name: String,
    language: String,
    root_path: String,
    description: Option<String>,
    metadata: HashMap<String, String>,
    status: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    version: u64,
}

impl From<&Project> for ProjectRecord {
    fn from(project: &Project) -> Self {
        Self {
            id: project.id().to_string(),
            name: project.name().to_string(),
            language: format!("{:?}", project.language()),
            root_path: project.root_path().to_string(),
            description: project.description().map(|s| s.to_string()),
            metadata: project.metadata().clone(),
            status: format!("{:?}", project.status()),
            created_at: project.created_at(),
            updated_at: project.updated_at(),
            version: project.version(),
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

/// SurrealDB implementation of ProjectRepository
pub struct SurrealProjectRepository {
    connection: SharedConnection,
}

impl SurrealProjectRepository {
    /// Create a new SurrealDB project repository
    pub fn new(connection: SharedConnection) -> Self {
        Self { connection }
    }
}

#[async_trait]
impl ProjectRepository for SurrealProjectRepository {
    async fn save(&self, project: &Project) -> DomainResult<()> {
        let record = ProjectRecord::from(project);
        let id = &record.id;
        
        debug!("Saving project {} to SurrealDB", id);

        match self.connection.client() {
            DatabaseClient::Local(db) => {
                let _: Option<ProjectRecord> = db
                    .upsert((TABLE_NAME, id.as_str()))
                    .content(record.clone())
                    .await
                    .map_err(to_domain_error)?;
            }
            DatabaseClient::Remote(db) => {
                let _: Option<ProjectRecord> = db
                    .upsert((TABLE_NAME, id.as_str()))
                    .content(record)
                    .await
                    .map_err(to_domain_error)?;
            }
        }

        Ok(())
    }

    async fn find_by_id(&self, id: &ProjectId) -> DomainResult<Option<Project>> {
        debug!("Finding project by id: {}", id);
        
        let record: Option<ProjectRecord> = match self.connection.client() {
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
                let project = reconstitute_project(r)?;
                Ok(Some(project))
            }
            None => Ok(None),
        }
    }

    async fn find_all(&self) -> DomainResult<Vec<Project>> {
        debug!("Finding all projects");
        
        let records: Vec<ProjectRecord> = match self.connection.client() {
            DatabaseClient::Local(db) => {
                db.select(TABLE_NAME)
                    .await
                    .map_err(to_domain_error)?
            }
            DatabaseClient::Remote(db) => {
                db.select(TABLE_NAME)
                    .await
                    .map_err(to_domain_error)?
            }
        };

        let mut projects = Vec::with_capacity(records.len());
        for r in records {
            let project = reconstitute_project(r)?;
            projects.push(project);
        }

        Ok(projects)
    }

    async fn delete(&self, id: &ProjectId) -> DomainResult<()> {
        debug!("Deleting project: {}", id);
        
        match self.connection.client() {
            DatabaseClient::Local(db) => {
                let _: Option<ProjectRecord> = db
                    .delete((TABLE_NAME, id.to_string().as_str()))
                    .await
                    .map_err(to_domain_error)?;
            }
            DatabaseClient::Remote(db) => {
                let _: Option<ProjectRecord> = db
                    .delete((TABLE_NAME, id.to_string().as_str()))
                    .await
                    .map_err(to_domain_error)?;
            }
        }

        Ok(())
    }

    async fn exists(&self, id: &ProjectId) -> DomainResult<bool> {
        let result = self.find_by_id(id).await?;
        Ok(result.is_some())
    }
}

/// Reconstitute a Project from a database record
fn reconstitute_project(r: ProjectRecord) -> DomainResult<Project> {
    let id = ProjectId::from_string(&r.id)
        .map_err(|e| DomainError::ExternalServiceError {
            service: "SurrealDB".to_string(),
            reason: format!("Invalid project ID: {}", e),
        })?;
    
    let language = parse_language(&r.language)?;
    let status = parse_status(&r.status)?;
    
    Ok(Project::reconstitute(
        id,
        r.name,
        r.description,
        language,
        r.root_path,
        r.created_at,
        r.updated_at,
        r.metadata,
        status,
        r.version,
    ))
}

/// Parse language string back to ProgrammingLanguage
fn parse_language(s: &str) -> DomainResult<ProgrammingLanguage> {
    match s {
        "Rust" => Ok(ProgrammingLanguage::Rust),
        "Python" => Ok(ProgrammingLanguage::Python),
        "TypeScript" => Ok(ProgrammingLanguage::TypeScript),
        "JavaScript" => Ok(ProgrammingLanguage::JavaScript),
        "Go" => Ok(ProgrammingLanguage::Go),
        "Java" => Ok(ProgrammingLanguage::Java),
        "CSharp" => Ok(ProgrammingLanguage::CSharp),
        "Cpp" => Ok(ProgrammingLanguage::Cpp),
        "C" => Ok(ProgrammingLanguage::C),
        "Ruby" => Ok(ProgrammingLanguage::Ruby),
        "Php" => Ok(ProgrammingLanguage::Php),
        "Swift" => Ok(ProgrammingLanguage::Swift),
        "Kotlin" => Ok(ProgrammingLanguage::Kotlin),
        "Scala" => Ok(ProgrammingLanguage::Scala),
        "Haskell" => Ok(ProgrammingLanguage::Haskell),
        _ => Ok(ProgrammingLanguage::Other),
    }
}

/// Parse status string back to ProjectStatus
fn parse_status(s: &str) -> DomainResult<ProjectStatus> {
    match s {
        "Active" => Ok(ProjectStatus::Active),
        "Archived" => Ok(ProjectStatus::Archived),
        "Deleted" => Ok(ProjectStatus::Deleted),
        _ => Err(DomainError::ExternalServiceError {
            service: "SurrealDB".to_string(),
            reason: format!("Invalid project status: {}", s),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::surreal::connection::{ConnectionMode, SurrealConnection};

    async fn create_test_repo() -> SurrealProjectRepository {
        let conn = SurrealConnection::new(ConnectionMode::Memory)
            .await
            .expect("Failed to create connection");
        SurrealProjectRepository::new(Arc::new(conn))
    }

    fn create_test_project(name: &str) -> Project {
        let (project, _) = Project::create(
            name.to_string(),
            ProgrammingLanguage::Rust,
            format!("/test/{}", name),
            None,
        ).unwrap();
        project
    }

    #[tokio::test]
    async fn test_save_and_find() {
        let repo = create_test_repo().await;
        let project = create_test_project("test-project");
        let id = project.id();

        repo.save(&project).await.unwrap();
        
        let found = repo.find_by_id(&id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name(), "test-project");
    }

    #[tokio::test]
    async fn test_find_all() {
        let repo = create_test_repo().await;
        
        repo.save(&create_test_project("p1")).await.unwrap();
        repo.save(&create_test_project("p2")).await.unwrap();
        
        let all = repo.find_all().await.unwrap();
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn test_delete() {
        let repo = create_test_repo().await;
        let project = create_test_project("to-delete");
        let id = project.id();

        repo.save(&project).await.unwrap();
        assert!(repo.exists(&id).await.unwrap());

        repo.delete(&id).await.unwrap();
        assert!(!repo.exists(&id).await.unwrap());
    }
}
