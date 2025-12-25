//! In-Memory Project Repository Implementation
//!
//! Memory backend for tests

use async_trait::async_trait;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

use ricecoder_domain::{
    errors::DomainResult,
    project::Project,
    repositories::ProjectRepository,
    value_objects::{ProjectId, ProgrammingLanguage},
};

/// Thread-safe in-memory implementation of ProjectRepository
///
/// Uses RwLock for concurrent read access with exclusive write access.
/// Stores cloned Project instances to maintain isolation.
///
/// # Example
///
/// ```ignore
/// use ricecoder_persistence::memory::InMemoryProjectRepository;
/// use ricecoder_domain::repositories::ProjectRepository;
/// use std::sync::Arc;
///
/// let repo = Arc::new(InMemoryProjectRepository::new());
/// ```
#[derive(Debug, Default)]
pub struct InMemoryProjectRepository {
    projects: RwLock<HashMap<ProjectId, Project>>,
}

impl InMemoryProjectRepository {
    /// Create a new empty in-memory project repository
    pub fn new() -> Self {
        Self {
            projects: RwLock::new(HashMap::new()),
        }
    }

    /// Create with initial projects (useful for testing)
    pub fn with_projects(projects: Vec<Project>) -> Self {
        let map: HashMap<ProjectId, Project> = projects
            .into_iter()
            .map(|p| (p.id().clone(), p))
            .collect();
        Self {
            projects: RwLock::new(map),
        }
    }

    /// Get the current count of projects (for testing)
    pub fn count(&self) -> usize {
        self.projects.read().len()
    }

    /// Clear all projects (for testing)
    pub fn clear(&self) {
        self.projects.write().clear();
    }
}

#[async_trait]
impl ProjectRepository for InMemoryProjectRepository {
    async fn save(&self, project: &Project) -> DomainResult<()> {
        let mut projects = self.projects.write();
        projects.insert(project.id().clone(), project.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: &ProjectId) -> DomainResult<Option<Project>> {
        let projects = self.projects.read();
        Ok(projects.get(id).cloned())
    }

    async fn find_all(&self) -> DomainResult<Vec<Project>> {
        let projects = self.projects.read();
        Ok(projects.values().cloned().collect())
    }

    async fn delete(&self, id: &ProjectId) -> DomainResult<()> {
        let mut projects = self.projects.write();
        projects.remove(id);
        Ok(())
    }

    async fn exists(&self, id: &ProjectId) -> DomainResult<bool> {
        let projects = self.projects.read();
        Ok(projects.contains_key(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_domain::project::Project;

    fn create_test_project(name: &str) -> Project {
        let (project, _events) = Project::create(
            name.to_string(),
            ProgrammingLanguage::Rust,
            format!("/test/{}", name),
            None,
        ).unwrap();
        project
    }

    #[tokio::test]
    async fn test_save_and_find_by_id() {
        let repo = InMemoryProjectRepository::new();
        let project = create_test_project("test-project");
        let id = project.id().clone();

        repo.save(&project).await.unwrap();

        let found = repo.find_by_id(&id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name(), project.name());
    }

    #[tokio::test]
    async fn test_find_by_id_not_found() {
        let repo = InMemoryProjectRepository::new();
        let id = ProjectId::new();

        let found = repo.find_by_id(&id).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_find_all() {
        let repo = InMemoryProjectRepository::new();
        let p1 = create_test_project("project-1");
        let p2 = create_test_project("project-2");

        repo.save(&p1).await.unwrap();
        repo.save(&p2).await.unwrap();

        let all = repo.find_all().await.unwrap();
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn test_delete() {
        let repo = InMemoryProjectRepository::new();
        let project = create_test_project("to-delete");
        let id = project.id().clone();

        repo.save(&project).await.unwrap();
        assert!(repo.exists(&id).await.unwrap());

        repo.delete(&id).await.unwrap();
        assert!(!repo.exists(&id).await.unwrap());
    }

    #[tokio::test]
    async fn test_exists() {
        let repo = InMemoryProjectRepository::new();
        let project = create_test_project("exists-test");
        let id = project.id().clone();

        assert!(!repo.exists(&id).await.unwrap());
        repo.save(&project).await.unwrap();
        assert!(repo.exists(&id).await.unwrap());
    }

    #[tokio::test]
    async fn test_update_existing() {
        let repo = InMemoryProjectRepository::new();
        let mut project = create_test_project("update-test");
        let id = project.id().clone();

        repo.save(&project).await.unwrap();

        // Update project
        project.add_metadata("key".to_string(), "value".to_string());
        repo.save(&project).await.unwrap();

        let found = repo.find_by_id(&id).await.unwrap().unwrap();
        assert_eq!(found.get_metadata("key"), Some(&"value".to_string()));
    }

    #[tokio::test]
    async fn test_count_and_clear() {
        let repo = InMemoryProjectRepository::new();
        assert_eq!(repo.count(), 0);

        repo.save(&create_test_project("p1")).await.unwrap();
        repo.save(&create_test_project("p2")).await.unwrap();
        assert_eq!(repo.count(), 2);

        repo.clear();
        assert_eq!(repo.count(), 0);
    }

    #[tokio::test]
    async fn test_with_initial_projects() {
        let projects = vec![
            create_test_project("init-1"),
            create_test_project("init-2"),
        ];
        let repo = InMemoryProjectRepository::with_projects(projects);

        assert_eq!(repo.count(), 2);
    }
}
