//! SurrealDB Specification Repository Implementation

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::debug;

use ricecoder_domain::{
    errors::{DomainError, DomainResult},
    repositories::{SpecificationReader, SpecificationWriter},
    specification::{Requirement, SpecStatus, Specification, Task, TaskStatus},
    value_objects::{ProjectId, RequirementId, SpecificationId, TaskId},
};

use super::connection::{DatabaseClient, SharedConnection};

/// SurrealDB table name for specifications
const TABLE_NAME: &str = "specifications";

/// Serializable requirement record for SurrealDB
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RequirementRecord {
    id: String,
    title: String,
    description: String,
    acceptance_criteria: Vec<String>,
    approved: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<&Requirement> for RequirementRecord {
    fn from(req: &Requirement) -> Self {
        Self {
            id: req.id().to_string(),
            title: req.title().to_string(),
            description: req.description().to_string(),
            acceptance_criteria: req.acceptance_criteria().to_vec(),
            approved: req.is_approved(),
            created_at: req.created_at(),
            updated_at: req.updated_at(),
        }
    }
}

/// Serializable task record for SurrealDB
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TaskRecord {
    id: String,
    title: String,
    description: String,
    requirement_refs: Vec<String>,
    status: String,
    parent_task_id: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<&Task> for TaskRecord {
    fn from(task: &Task) -> Self {
        let status = match task.status() {
            TaskStatus::Pending => "Pending",
            TaskStatus::InProgress => "InProgress",
            TaskStatus::Completed => "Completed",
            TaskStatus::Cancelled => "Cancelled",
        };
        Self {
            id: task.id().to_string(),
            title: task.title().to_string(),
            description: task.description().to_string(),
            requirement_refs: task.requirement_refs().iter().map(|r| r.to_string()).collect(),
            status: status.to_string(),
            parent_task_id: task.parent_task_id().map(|id| id.to_string()),
            created_at: task.created_at(),
            updated_at: task.updated_at(),
        }
    }
}

/// Serializable specification record for SurrealDB
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SpecificationRecord {
    id: String,
    project_id: String,
    name: String,
    description: String,
    version: String,
    requirements: Vec<RequirementRecord>,
    tasks: Vec<TaskRecord>,
    status: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    concurrency_version: u64,
}

impl From<&Specification> for SpecificationRecord {
    fn from(spec: &Specification) -> Self {
        let status = match spec.status() {
            SpecStatus::Draft => "Draft",
            SpecStatus::RequirementsComplete => "RequirementsComplete",
            SpecStatus::DesignComplete => "DesignComplete",
            SpecStatus::TasksPlanned => "TasksPlanned",
            SpecStatus::InProgress => "InProgress",
            SpecStatus::Complete => "Complete",
            SpecStatus::Archived => "Archived",
        };
        Self {
            id: spec.id().to_string(),
            project_id: spec.project_id().to_string(),
            name: spec.name().to_string(),
            description: spec.description().to_string(),
            version: spec.version().to_string(),
            requirements: spec.requirements().iter().map(RequirementRecord::from).collect(),
            tasks: spec.tasks().iter().map(TaskRecord::from).collect(),
            status: status.to_string(),
            created_at: spec.created_at(),
            updated_at: spec.updated_at(),
            concurrency_version: spec.concurrency_version(),
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

/// SurrealDB implementation of SpecificationRepository
pub struct SurrealSpecificationRepository {
    connection: SharedConnection,
}

impl SurrealSpecificationRepository {
    /// Create a new SurrealDB specification repository
    pub fn new(connection: SharedConnection) -> Self {
        Self { connection }
    }
}

#[async_trait]
impl SpecificationWriter for SurrealSpecificationRepository {
    async fn save(&self, spec: &Specification) -> DomainResult<()> {
        let record = SpecificationRecord::from(spec);
        let id = &record.id;
        
        debug!("Saving specification {} to SurrealDB", id);

        match self.connection.client() {
            DatabaseClient::Local(db) => {
                let _: Option<SpecificationRecord> = db
                    .upsert((TABLE_NAME, id.as_str()))
                    .content(record.clone())
                    .await
                    .map_err(to_domain_error)?;
            }
            DatabaseClient::Remote(db) => {
                let _: Option<SpecificationRecord> = db
                    .upsert((TABLE_NAME, id.as_str()))
                    .content(record)
                    .await
                    .map_err(to_domain_error)?;
            }
        }

        Ok(())
    }

    async fn delete(&self, id: &SpecificationId) -> DomainResult<()> {
        debug!("Deleting specification: {}", id);
        
        match self.connection.client() {
            DatabaseClient::Local(db) => {
                let _: Option<SpecificationRecord> = db
                    .delete((TABLE_NAME, id.to_string().as_str()))
                    .await
                    .map_err(to_domain_error)?;
            }
            DatabaseClient::Remote(db) => {
                let _: Option<SpecificationRecord> = db
                    .delete((TABLE_NAME, id.to_string().as_str()))
                    .await
                    .map_err(to_domain_error)?;
            }
        }

        Ok(())
    }
}

#[async_trait]
impl SpecificationReader for SurrealSpecificationRepository {
    async fn find_by_id(&self, id: &SpecificationId) -> DomainResult<Option<Specification>> {
        debug!("Finding specification by id: {}", id);
        
        let record: Option<SpecificationRecord> = match self.connection.client() {
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
                let spec = reconstitute_specification(r)?;
                Ok(Some(spec))
            }
            None => Ok(None),
        }
    }

    async fn find_by_project(&self, project_id: &ProjectId) -> DomainResult<Vec<Specification>> {
        debug!("Finding specifications for project: {}", project_id);
        
        let query = format!(
            "SELECT * FROM {} WHERE project_id = '{}'",
            TABLE_NAME,
            project_id
        );

        let records: Vec<SpecificationRecord> = match self.connection.client() {
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

        let mut specs = Vec::with_capacity(records.len());
        for r in records {
            let spec = reconstitute_specification(r)?;
            specs.push(spec);
        }

        Ok(specs)
    }

    async fn find_all(&self) -> DomainResult<Vec<Specification>> {
        debug!("Finding all specifications");
        
        let records: Vec<SpecificationRecord> = match self.connection.client() {
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

        let mut specs = Vec::with_capacity(records.len());
        for r in records {
            let spec = reconstitute_specification(r)?;
            specs.push(spec);
        }

        Ok(specs)
    }

    async fn exists(&self, id: &SpecificationId) -> DomainResult<bool> {
        let result = self.find_by_id(id).await?;
        Ok(result.is_some())
    }

    async fn find_by_status(&self, status: SpecStatus) -> DomainResult<Vec<Specification>> {
        debug!("Finding specifications by status: {:?}", status);
        
        let status_str = match status {
            SpecStatus::Draft => "Draft",
            SpecStatus::RequirementsComplete => "RequirementsComplete",
            SpecStatus::DesignComplete => "DesignComplete",
            SpecStatus::TasksPlanned => "TasksPlanned",
            SpecStatus::InProgress => "InProgress",
            SpecStatus::Complete => "Complete",
            SpecStatus::Archived => "Archived",
        };
        
        let query = format!(
            "SELECT * FROM {} WHERE status = '{}'",
            TABLE_NAME,
            status_str
        );

        let records: Vec<SpecificationRecord> = match self.connection.client() {
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

        let mut specs = Vec::with_capacity(records.len());
        for r in records {
            let spec = reconstitute_specification(r)?;
            specs.push(spec);
        }

        Ok(specs)
    }
}

/// Reconstitute a Specification from a database record
fn reconstitute_specification(r: SpecificationRecord) -> DomainResult<Specification> {
    let id = SpecificationId::from_string(&r.id)
        .map_err(|e| DomainError::ExternalServiceError {
            service: "SurrealDB".to_string(),
            reason: format!("Invalid specification ID: {}", e),
        })?;
    
    let project_id = ProjectId::from_string(&r.project_id)
        .map_err(|e| DomainError::ExternalServiceError {
            service: "SurrealDB".to_string(),
            reason: format!("Invalid project ID: {}", e),
        })?;
    
    let status = parse_spec_status(&r.status)?;
    
    // Reconstitute requirements
    let mut requirements = Vec::with_capacity(r.requirements.len());
    for req_record in r.requirements {
        let req_id = RequirementId::from_string(&req_record.id)
            .map_err(|e| DomainError::ExternalServiceError {
                service: "SurrealDB".to_string(),
                reason: format!("Invalid requirement ID: {}", e),
            })?;
        let requirement = Requirement::reconstitute(
            req_id,
            req_record.title,
            req_record.description,
            req_record.acceptance_criteria,
            req_record.approved,
            req_record.created_at,
            req_record.updated_at,
        );
        requirements.push(requirement);
    }
    
    // Reconstitute tasks
    let mut tasks = Vec::with_capacity(r.tasks.len());
    for task_record in r.tasks {
        let task_id = TaskId::from_string(&task_record.id)
            .map_err(|e| DomainError::ExternalServiceError {
                service: "SurrealDB".to_string(),
                reason: format!("Invalid task ID: {}", e),
            })?;
        
        let mut requirement_refs = Vec::with_capacity(task_record.requirement_refs.len());
        for ref_id in task_record.requirement_refs {
            let req_id = RequirementId::from_string(&ref_id)
                .map_err(|e| DomainError::ExternalServiceError {
                    service: "SurrealDB".to_string(),
                    reason: format!("Invalid requirement ref ID: {}", e),
                })?;
            requirement_refs.push(req_id);
        }
        
        let task_status = parse_task_status(&task_record.status)?;
        
        let parent_task_id = match task_record.parent_task_id {
            Some(pid) => Some(TaskId::from_string(&pid)
                .map_err(|e| DomainError::ExternalServiceError {
                    service: "SurrealDB".to_string(),
                    reason: format!("Invalid parent task ID: {}", e),
                })?),
            None => None,
        };
        
        let task = Task::reconstitute(
            task_id,
            task_record.title,
            task_record.description,
            requirement_refs,
            task_status,
            parent_task_id,
            task_record.created_at,
            task_record.updated_at,
        );
        tasks.push(task);
    }
    
    Ok(Specification::reconstitute(
        id,
        project_id,
        r.name,
        r.description,
        r.version,
        requirements,
        tasks,
        status,
        r.created_at,
        r.updated_at,
        r.concurrency_version,
    ))
}

/// Parse status string back to SpecStatus
fn parse_spec_status(s: &str) -> DomainResult<SpecStatus> {
    match s {
        "Draft" => Ok(SpecStatus::Draft),
        "RequirementsComplete" => Ok(SpecStatus::RequirementsComplete),
        "DesignComplete" => Ok(SpecStatus::DesignComplete),
        "TasksPlanned" => Ok(SpecStatus::TasksPlanned),
        "InProgress" => Ok(SpecStatus::InProgress),
        "Complete" => Ok(SpecStatus::Complete),
        "Archived" => Ok(SpecStatus::Archived),
        _ => Err(DomainError::ExternalServiceError {
            service: "SurrealDB".to_string(),
            reason: format!("Invalid specification status: {}", s),
        }),
    }
}

/// Parse status string back to TaskStatus
fn parse_task_status(s: &str) -> DomainResult<TaskStatus> {
    match s {
        "Pending" => Ok(TaskStatus::Pending),
        "InProgress" => Ok(TaskStatus::InProgress),
        "Completed" => Ok(TaskStatus::Completed),
        "Cancelled" => Ok(TaskStatus::Cancelled),
        _ => Err(DomainError::ExternalServiceError {
            service: "SurrealDB".to_string(),
            reason: format!("Invalid task status: {}", s),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::surreal::connection::{ConnectionMode, SurrealConnection};

    async fn create_test_repo() -> SurrealSpecificationRepository {
        let conn = SurrealConnection::new(ConnectionMode::Memory)
            .await
            .expect("Failed to create connection");
        SurrealSpecificationRepository::new(Arc::new(conn))
    }

    fn create_test_spec(name: &str) -> Specification {
        let project_id = ProjectId::new();
        let (spec, _) = Specification::create(
            project_id,
            name.to_string(),
            "Test description".to_string(),
            "1.0.0".to_string(),
        ).unwrap();
        spec
    }

    #[tokio::test]
    async fn test_save_and_find() {
        let repo = create_test_repo().await;
        let spec = create_test_spec("test-spec");
        let id = spec.id();

        repo.save(&spec).await.unwrap();
        
        let found = repo.find_by_id(&id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name(), "test-spec");
    }

    #[tokio::test]
    async fn test_find_all() {
        let repo = create_test_repo().await;
        
        repo.save(&create_test_spec("s1")).await.unwrap();
        repo.save(&create_test_spec("s2")).await.unwrap();
        
        let all = repo.find_all().await.unwrap();
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn test_find_by_status() {
        let repo = create_test_repo().await;
        
        let spec = create_test_spec("draft-spec");
        repo.save(&spec).await.unwrap();
        
        let found = repo.find_by_status(SpecStatus::Draft).await.unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].name(), "draft-spec");
    }

    #[tokio::test]
    async fn test_delete() {
        let repo = create_test_repo().await;
        let spec = create_test_spec("to-delete");
        let id = spec.id();

        repo.save(&spec).await.unwrap();
        assert!(repo.exists(&id).await.unwrap());

        repo.delete(&id).await.unwrap();
        assert!(!repo.exists(&id).await.unwrap());
    }
}
