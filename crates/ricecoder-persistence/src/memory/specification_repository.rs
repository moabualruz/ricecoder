//! In-Memory Specification Repository Implementation
//!
//! Memory backend for tests

use async_trait::async_trait;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

use ricecoder_domain::{
    errors::DomainResult,
    repositories::{SpecificationReader, SpecificationWriter},
    specification::{SpecStatus, Specification},
    value_objects::{ProjectId, SpecificationId},
};

/// Thread-safe in-memory implementation of SpecificationRepository
///
/// Uses RwLock for concurrent read access with exclusive write access.
/// Stores cloned Specification instances to maintain isolation.
#[derive(Debug, Default)]
pub struct InMemorySpecificationRepository {
    specifications: RwLock<HashMap<SpecificationId, Specification>>,
}

impl InMemorySpecificationRepository {
    /// Create a new empty in-memory specification repository
    pub fn new() -> Self {
        Self {
            specifications: RwLock::new(HashMap::new()),
        }
    }

    /// Create with initial specifications (useful for testing)
    pub fn with_specifications(specs: Vec<Specification>) -> Self {
        let map: HashMap<SpecificationId, Specification> = specs
            .into_iter()
            .map(|s| (s.id().clone(), s))
            .collect();
        Self {
            specifications: RwLock::new(map),
        }
    }

    /// Get the current count of specifications (for testing)
    pub fn count(&self) -> usize {
        self.specifications.read().len()
    }

    /// Clear all specifications (for testing)
    pub fn clear(&self) {
        self.specifications.write().clear();
    }
}

/// Implementation of SpecificationReader for read operations
#[async_trait]
impl SpecificationReader for InMemorySpecificationRepository {
    async fn find_by_id(&self, id: &SpecificationId) -> DomainResult<Option<Specification>> {
        let specs = self.specifications.read();
        Ok(specs.get(id).cloned())
    }

    async fn find_by_project(&self, project_id: &ProjectId) -> DomainResult<Vec<Specification>> {
        let specs = self.specifications.read();
        Ok(specs
            .values()
            .filter(|s| s.project_id() == *project_id)
            .cloned()
            .collect())
    }

    async fn find_all(&self) -> DomainResult<Vec<Specification>> {
        let specs = self.specifications.read();
        Ok(specs.values().cloned().collect())
    }

    async fn exists(&self, id: &SpecificationId) -> DomainResult<bool> {
        let specs = self.specifications.read();
        Ok(specs.contains_key(id))
    }

    async fn find_by_status(&self, status: SpecStatus) -> DomainResult<Vec<Specification>> {
        let specs = self.specifications.read();
        Ok(specs
            .values()
            .filter(|s| s.status() == status)
            .cloned()
            .collect())
    }
}

/// Implementation of SpecificationWriter for write operations
#[async_trait]
impl SpecificationWriter for InMemorySpecificationRepository {
    async fn save(&self, specification: &Specification) -> DomainResult<()> {
        let mut specs = self.specifications.write();
        specs.insert(specification.id().clone(), specification.clone());
        Ok(())
    }

    async fn delete(&self, id: &SpecificationId) -> DomainResult<()> {
        let mut specs = self.specifications.write();
        specs.remove(id);
        Ok(())
    }
}

// SpecificationRepository is automatically implemented via blanket impl in domain

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_domain::repositories::{SpecificationReader, SpecificationWriter};
    use ricecoder_domain::specification::Specification;
    use ricecoder_domain::value_objects::ProjectId;

    fn create_test_spec(project_id: &ProjectId, name: &str) -> Specification {
        let (spec, _event) = Specification::create(
            project_id.clone(),
            name.to_string(),
            format!("Test description for {}", name),
            "1.0.0".to_string(),
        ).unwrap();
        spec
    }

    #[tokio::test]
    async fn test_save_and_find_by_id() {
        let repo = InMemorySpecificationRepository::new();
        let project_id = ProjectId::new();
        let spec = create_test_spec(&project_id, "Test Spec");
        let id = spec.id().clone();

        repo.save(&spec).await.unwrap();

        let found = repo.find_by_id(&id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name(), spec.name());
    }

    #[tokio::test]
    async fn test_find_by_id_not_found() {
        let repo = InMemorySpecificationRepository::new();
        let id = SpecificationId::new();

        let found = repo.find_by_id(&id).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_find_by_project() {
        let repo = InMemorySpecificationRepository::new();
        let project1 = ProjectId::new();
        let project2 = ProjectId::new();

        let s1 = create_test_spec(&project1, "Spec 1");
        let s2 = create_test_spec(&project1, "Spec 2");
        let s3 = create_test_spec(&project2, "Spec 3");

        repo.save(&s1).await.unwrap();
        repo.save(&s2).await.unwrap();
        repo.save(&s3).await.unwrap();

        let project1_specs = repo.find_by_project(&project1).await.unwrap();
        assert_eq!(project1_specs.len(), 2);

        let project2_specs = repo.find_by_project(&project2).await.unwrap();
        assert_eq!(project2_specs.len(), 1);
    }

    #[tokio::test]
    async fn test_find_all() {
        let repo = InMemorySpecificationRepository::new();
        let project_id = ProjectId::new();

        repo.save(&create_test_spec(&project_id, "Spec 1")).await.unwrap();
        repo.save(&create_test_spec(&project_id, "Spec 2")).await.unwrap();

        let all = repo.find_all().await.unwrap();
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn test_delete() {
        let repo = InMemorySpecificationRepository::new();
        let project_id = ProjectId::new();
        let spec = create_test_spec(&project_id, "To Delete");
        let id = spec.id().clone();

        repo.save(&spec).await.unwrap();
        assert!(repo.exists(&id).await.unwrap());

        repo.delete(&id).await.unwrap();
        assert!(!repo.exists(&id).await.unwrap());
    }

    #[tokio::test]
    async fn test_exists() {
        let repo = InMemorySpecificationRepository::new();
        let project_id = ProjectId::new();
        let spec = create_test_spec(&project_id, "Exists Test");
        let id = spec.id().clone();

        assert!(!repo.exists(&id).await.unwrap());
        repo.save(&spec).await.unwrap();
        assert!(repo.exists(&id).await.unwrap());
    }

    #[tokio::test]
    async fn test_find_by_status() {
        let repo = InMemorySpecificationRepository::new();
        let project_id = ProjectId::new();

        let s1 = create_test_spec(&project_id, "Draft Spec");
        let mut s2 = create_test_spec(&project_id, "In Progress Spec");

        // Add requirement to move to RequirementsComplete
        s2.add_requirement("REQ-001".to_string(), "Test req".to_string(), vec!["AC1".to_string()]).unwrap();
        s2.mark_requirements_complete().unwrap();

        repo.save(&s1).await.unwrap();
        repo.save(&s2).await.unwrap();

        let drafts = repo.find_by_status(SpecStatus::Draft).await.unwrap();
        assert_eq!(drafts.len(), 1);
        assert_eq!(drafts[0].name(), "Draft Spec");

        let req_complete = repo.find_by_status(SpecStatus::RequirementsComplete).await.unwrap();
        assert_eq!(req_complete.len(), 1);
        assert_eq!(req_complete[0].name(), "In Progress Spec");
    }

    #[tokio::test]
    async fn test_count_and_clear() {
        let repo = InMemorySpecificationRepository::new();
        let project_id = ProjectId::new();

        assert_eq!(repo.count(), 0);

        repo.save(&create_test_spec(&project_id, "S1")).await.unwrap();
        repo.save(&create_test_spec(&project_id, "S2")).await.unwrap();
        assert_eq!(repo.count(), 2);

        repo.clear();
        assert_eq!(repo.count(), 0);
    }
}
