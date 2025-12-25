//! Repository Contract Tests - LSP Validation
//!
//! LSP compliance verification
//!
//! These tests define the behavioral contract that ALL repository implementations
//! MUST satisfy. Any implementation that passes these tests is guaranteed to be
//! substitutable for any other implementation (Liskov Substitution Principle).
//!
//! ## Contract Properties
//!
//! 1. **Save-Find Roundtrip**: save(entity) → find_by_id(id) returns Some(entity)
//! 2. **Not Found Returns None**: find_by_id(non_existent_id) returns None (not Err)
//! 3. **Save-Exists Consistency**: save(entity) → exists(id) returns true
//! 4. **Delete-Exists Consistency**: delete(id) → exists(id) returns false
//! 5. **Idempotent Delete**: delete(non_existent_id) succeeds (no error)
//! 6. **Find All Contains Saved**: save(entity) → find_all() contains entity

use async_trait::async_trait;
use ricecoder_domain::{
    errors::DomainResult,
    project::Project,
    repositories::{ProjectRepository, SessionRepository, SpecificationRepository, SpecificationReader, SpecificationWriter},
    session::Session,
    specification::Specification,
    value_objects::{ProgrammingLanguage, ProjectId, SessionId, SpecificationId},
};

// ============================================================================
// Test Entity Factories
// ============================================================================

/// Create a test project with a unique name
fn create_test_project(suffix: &str) -> Project {
    let (project, _events) = Project::create(
        format!("test-project-{}", suffix),
        ProgrammingLanguage::Rust,
        format!("/test/path/{}", suffix),
        None,
    )
    .expect("Failed to create test project");
    project
}

/// Create a test session for a project
fn create_test_session(project_id: &ProjectId) -> Session {
    let (session, _events) = Session::create(project_id.clone(), 100)
        .expect("Failed to create test session");
    session
}

/// Create a test specification for a project
fn create_test_specification(project_id: &ProjectId, suffix: &str) -> Specification {
    let (spec, _events) = Specification::create(
        project_id.clone(),
        format!("Test Spec {}", suffix),
        format!("Description for spec {}", suffix),
        "1.0.0".to_string(),
    )
    .expect("Failed to create test specification");
    spec
}

// ============================================================================
// ProjectRepository Contract Tests
// ============================================================================

/// Contract test trait for ProjectRepository
///
/// Any implementation that passes these tests satisfies the LSP contract.
#[async_trait]
pub trait ProjectRepositoryContract: Sized + Send + Sync {
    /// Get the repository under test
    fn repo(&self) -> &dyn ProjectRepository;
    /// Setup/cleanup between tests
    async fn reset(&self);
}

/// Run all contract tests against a ProjectRepository implementation
pub async fn run_project_repository_contracts<T: ProjectRepositoryContract>(harness: &T) {
    println!("Running ProjectRepository contract tests...");
    
    // Contract 1: Save-Find Roundtrip
    harness.reset().await;
    project_contract_save_find_roundtrip(harness).await;
    
    // Contract 2: Not Found Returns None
    harness.reset().await;
    project_contract_not_found_returns_none(harness).await;
    
    // Contract 3: Save-Exists Consistency
    harness.reset().await;
    project_contract_save_exists_consistency(harness).await;
    
    // Contract 4: Delete-Exists Consistency
    harness.reset().await;
    project_contract_delete_exists_consistency(harness).await;
    
    // Contract 5: Idempotent Delete
    harness.reset().await;
    project_contract_idempotent_delete(harness).await;
    
    // Contract 6: Find All Contains Saved
    harness.reset().await;
    project_contract_find_all_contains_saved(harness).await;
    
    // Contract 7: Update Preserves Identity
    harness.reset().await;
    project_contract_update_preserves_identity(harness).await;
    
    println!("All ProjectRepository contracts passed!");
}

async fn project_contract_save_find_roundtrip<T: ProjectRepositoryContract>(harness: &T) {
    let project = create_test_project("roundtrip");
    let id = project.id().clone();
    
    harness.repo().save(&project).await
        .expect("Contract violation: save() should succeed");
    
    let found = harness.repo().find_by_id(&id).await
        .expect("Contract violation: find_by_id() should not return Err");
    
    assert!(found.is_some(), "Contract violation: find_by_id() must return Some after save()");
    assert_eq!(found.unwrap().name(), project.name(), 
        "Contract violation: found entity must equal saved entity");
}

async fn project_contract_not_found_returns_none<T: ProjectRepositoryContract>(harness: &T) {
    let non_existent_id = ProjectId::new();
    
    let result = harness.repo().find_by_id(&non_existent_id).await;
    
    assert!(result.is_ok(), "Contract violation: find_by_id() for non-existent entity must return Ok, not Err");
    assert!(result.unwrap().is_none(), "Contract violation: find_by_id() for non-existent entity must return None");
}

async fn project_contract_save_exists_consistency<T: ProjectRepositoryContract>(harness: &T) {
    let project = create_test_project("exists");
    let id = project.id().clone();
    
    // Before save, exists should be false
    let exists_before = harness.repo().exists(&id).await
        .expect("Contract violation: exists() should not return Err");
    assert!(!exists_before, "Contract violation: exists() must return false for unsaved entity");
    
    // After save, exists should be true
    harness.repo().save(&project).await.expect("save failed");
    let exists_after = harness.repo().exists(&id).await
        .expect("Contract violation: exists() should not return Err");
    assert!(exists_after, "Contract violation: exists() must return true after save()");
}

async fn project_contract_delete_exists_consistency<T: ProjectRepositoryContract>(harness: &T) {
    let project = create_test_project("delete");
    let id = project.id().clone();
    
    harness.repo().save(&project).await.expect("save failed");
    assert!(harness.repo().exists(&id).await.unwrap(), "precondition: entity should exist");
    
    harness.repo().delete(&id).await
        .expect("Contract violation: delete() should succeed");
    
    let exists_after = harness.repo().exists(&id).await
        .expect("Contract violation: exists() should not return Err");
    assert!(!exists_after, "Contract violation: exists() must return false after delete()");
}

async fn project_contract_idempotent_delete<T: ProjectRepositoryContract>(harness: &T) {
    let non_existent_id = ProjectId::new();
    
    // Deleting non-existent entity should NOT return an error
    let result = harness.repo().delete(&non_existent_id).await;
    assert!(result.is_ok(), "Contract violation: delete() for non-existent entity must succeed (idempotent)");
}

async fn project_contract_find_all_contains_saved<T: ProjectRepositoryContract>(harness: &T) {
    let p1 = create_test_project("all-1");
    let p2 = create_test_project("all-2");
    let id1 = p1.id().clone();
    let id2 = p2.id().clone();
    
    harness.repo().save(&p1).await.expect("save p1 failed");
    harness.repo().save(&p2).await.expect("save p2 failed");
    
    let all = harness.repo().find_all().await
        .expect("Contract violation: find_all() should not return Err");
    
    assert!(all.iter().any(|p| p.id() == id1), 
        "Contract violation: find_all() must contain all saved entities");
    assert!(all.iter().any(|p| p.id() == id2), 
        "Contract violation: find_all() must contain all saved entities");
}

async fn project_contract_update_preserves_identity<T: ProjectRepositoryContract>(harness: &T) {
    let mut project = create_test_project("update");
    let id = project.id().clone();
    
    harness.repo().save(&project).await.expect("save failed");
    
    // Update the project
    project.add_metadata("key".to_string(), "value".to_string());
    harness.repo().save(&project).await.expect("update save failed");
    
    let found = harness.repo().find_by_id(&id).await
        .expect("find failed")
        .expect("should exist");
    
    assert_eq!(found.id(), id, "Contract violation: update must preserve entity identity");
    assert_eq!(found.get_metadata("key"), Some(&"value".to_string()), 
        "Contract violation: update must persist changes");
}

// ============================================================================
// SessionRepository Contract Tests
// ============================================================================

#[async_trait]
pub trait SessionRepositoryContract: Sized + Send + Sync {
    fn repo(&self) -> &dyn SessionRepository;
    async fn reset(&self);
}

pub async fn run_session_repository_contracts<T: SessionRepositoryContract>(harness: &T) {
    println!("Running SessionRepository contract tests...");
    
    harness.reset().await;
    session_contract_save_find_roundtrip(harness).await;
    
    harness.reset().await;
    session_contract_not_found_returns_none(harness).await;
    
    harness.reset().await;
    session_contract_find_by_project(harness).await;
    
    harness.reset().await;
    session_contract_find_active(harness).await;
    
    harness.reset().await;
    session_contract_delete(harness).await;
    
    println!("All SessionRepository contracts passed!");
}

async fn session_contract_save_find_roundtrip<T: SessionRepositoryContract>(harness: &T) {
    let project_id = ProjectId::new();
    let session = create_test_session(&project_id);
    let id = session.id().clone();
    
    harness.repo().save(&session).await
        .expect("Contract violation: save() should succeed");
    
    let found = harness.repo().find_by_id(&id).await
        .expect("Contract violation: find_by_id() should not return Err");
    
    assert!(found.is_some(), "Contract violation: find_by_id() must return Some after save()");
    assert_eq!(found.unwrap().id(), session.id());
}

async fn session_contract_not_found_returns_none<T: SessionRepositoryContract>(harness: &T) {
    let non_existent_id = SessionId::new();
    
    let result = harness.repo().find_by_id(&non_existent_id).await;
    
    assert!(result.is_ok(), "Contract violation: find_by_id() must return Ok for non-existent entity");
    assert!(result.unwrap().is_none(), "Contract violation: find_by_id() must return None for non-existent entity");
}

async fn session_contract_find_by_project<T: SessionRepositoryContract>(harness: &T) {
    let project1 = ProjectId::new();
    let project2 = ProjectId::new();
    
    let s1 = create_test_session(&project1);
    let s2 = create_test_session(&project1);
    let s3 = create_test_session(&project2);
    
    harness.repo().save(&s1).await.expect("save s1 failed");
    harness.repo().save(&s2).await.expect("save s2 failed");
    harness.repo().save(&s3).await.expect("save s3 failed");
    
    let project1_sessions = harness.repo().find_by_project(&project1).await
        .expect("Contract violation: find_by_project() should not return Err");
    
    assert_eq!(project1_sessions.len(), 2, 
        "Contract violation: find_by_project() must return only sessions for that project");
    
    let project2_sessions = harness.repo().find_by_project(&project2).await
        .expect("find_by_project failed");
    assert_eq!(project2_sessions.len(), 1);
}

async fn session_contract_find_active<T: SessionRepositoryContract>(harness: &T) {
    let project_id = ProjectId::new();
    
    let s1 = create_test_session(&project_id);
    let mut s2 = create_test_session(&project_id);
    s2.complete().expect("complete failed");
    
    harness.repo().save(&s1).await.expect("save s1 failed");
    harness.repo().save(&s2).await.expect("save s2 failed");
    
    let active = harness.repo().find_active().await
        .expect("Contract violation: find_active() should not return Err");
    
    assert_eq!(active.len(), 1, "Contract violation: find_active() must only return active sessions");
    assert_eq!(active[0].id(), s1.id());
}

async fn session_contract_delete<T: SessionRepositoryContract>(harness: &T) {
    let project_id = ProjectId::new();
    let session = create_test_session(&project_id);
    let id = session.id().clone();
    
    harness.repo().save(&session).await.expect("save failed");
    harness.repo().delete(&id).await
        .expect("Contract violation: delete() should succeed");
    
    let found = harness.repo().find_by_id(&id).await.expect("find failed");
    assert!(found.is_none(), "Contract violation: find_by_id() must return None after delete()");
}

// ============================================================================
// SpecificationRepository Contract Tests
// ============================================================================

#[async_trait]
pub trait SpecificationRepositoryContract: Sized + Send + Sync {
    fn repo(&self) -> &dyn SpecificationRepository;
    async fn reset(&self);
}

pub async fn run_specification_repository_contracts<T: SpecificationRepositoryContract>(harness: &T) {
    println!("Running SpecificationRepository contract tests...");
    
    harness.reset().await;
    spec_contract_save_find_roundtrip(harness).await;
    
    harness.reset().await;
    spec_contract_not_found_returns_none(harness).await;
    
    harness.reset().await;
    spec_contract_exists(harness).await;
    
    harness.reset().await;
    spec_contract_find_by_project(harness).await;
    
    harness.reset().await;
    spec_contract_find_all(harness).await;
    
    harness.reset().await;
    spec_contract_delete(harness).await;
    
    println!("All SpecificationRepository contracts passed!");
}

async fn spec_contract_save_find_roundtrip<T: SpecificationRepositoryContract>(harness: &T) {
    let project_id = ProjectId::new();
    let spec = create_test_specification(&project_id, "roundtrip");
    let id = spec.id().clone();
    
    harness.repo().save(&spec).await
        .expect("Contract violation: save() should succeed");
    
    let found = harness.repo().find_by_id(&id).await
        .expect("Contract violation: find_by_id() should not return Err");
    
    assert!(found.is_some(), "Contract violation: find_by_id() must return Some after save()");
    assert_eq!(found.unwrap().name(), spec.name());
}

async fn spec_contract_not_found_returns_none<T: SpecificationRepositoryContract>(harness: &T) {
    let non_existent_id = SpecificationId::new();
    
    let result = harness.repo().find_by_id(&non_existent_id).await;
    
    assert!(result.is_ok(), "Contract violation: find_by_id() must return Ok for non-existent entity");
    assert!(result.unwrap().is_none(), "Contract violation: find_by_id() must return None for non-existent entity");
}

async fn spec_contract_exists<T: SpecificationRepositoryContract>(harness: &T) {
    let project_id = ProjectId::new();
    let spec = create_test_specification(&project_id, "exists");
    let id = spec.id().clone();
    
    let exists_before = harness.repo().exists(&id).await
        .expect("exists should not error");
    assert!(!exists_before, "Contract violation: exists() must return false for unsaved entity");
    
    harness.repo().save(&spec).await.expect("save failed");
    
    let exists_after = harness.repo().exists(&id).await
        .expect("exists should not error");
    assert!(exists_after, "Contract violation: exists() must return true after save()");
}

async fn spec_contract_find_by_project<T: SpecificationRepositoryContract>(harness: &T) {
    let project1 = ProjectId::new();
    let project2 = ProjectId::new();
    
    let s1 = create_test_specification(&project1, "proj1-1");
    let s2 = create_test_specification(&project1, "proj1-2");
    let s3 = create_test_specification(&project2, "proj2-1");
    
    harness.repo().save(&s1).await.expect("save s1 failed");
    harness.repo().save(&s2).await.expect("save s2 failed");
    harness.repo().save(&s3).await.expect("save s3 failed");
    
    let project1_specs = harness.repo().find_by_project(&project1).await
        .expect("find_by_project should not error");
    
    assert_eq!(project1_specs.len(), 2, 
        "Contract violation: find_by_project() must return only specs for that project");
}

async fn spec_contract_find_all<T: SpecificationRepositoryContract>(harness: &T) {
    let project_id = ProjectId::new();
    let s1 = create_test_specification(&project_id, "all-1");
    let s2 = create_test_specification(&project_id, "all-2");
    let id1 = s1.id().clone();
    let id2 = s2.id().clone();
    
    harness.repo().save(&s1).await.expect("save s1 failed");
    harness.repo().save(&s2).await.expect("save s2 failed");
    
    let all = harness.repo().find_all().await
        .expect("Contract violation: find_all() should not return Err");
    
    assert!(all.iter().any(|s| s.id() == id1), "Contract violation: find_all() must contain all saved entities");
    assert!(all.iter().any(|s| s.id() == id2), "Contract violation: find_all() must contain all saved entities");
}

async fn spec_contract_delete<T: SpecificationRepositoryContract>(harness: &T) {
    let project_id = ProjectId::new();
    let spec = create_test_specification(&project_id, "delete");
    let id = spec.id().clone();
    
    harness.repo().save(&spec).await.expect("save failed");
    harness.repo().delete(&id).await
        .expect("Contract violation: delete() should succeed");
    
    let exists = harness.repo().exists(&id).await.expect("exists failed");
    assert!(!exists, "Contract violation: exists() must return false after delete()");
}

// ============================================================================
// Integration Tests with In-Memory Implementations
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    // In-memory implementations for testing (inline stubs for domain-only tests)
    use std::collections::HashMap;
    use std::sync::RwLock;

    /// In-memory project repository for contract testing
    pub struct InMemoryProjectRepoStub {
        projects: RwLock<HashMap<ProjectId, Project>>,
    }

    impl InMemoryProjectRepoStub {
        pub fn new() -> Self {
            Self {
                projects: RwLock::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl ProjectRepository for InMemoryProjectRepoStub {
        async fn save(&self, project: &Project) -> DomainResult<()> {
            self.projects.write().unwrap().insert(project.id().clone(), project.clone());
            Ok(())
        }

        async fn find_by_id(&self, id: &ProjectId) -> DomainResult<Option<Project>> {
            Ok(self.projects.read().unwrap().get(id).cloned())
        }

        async fn find_all(&self) -> DomainResult<Vec<Project>> {
            Ok(self.projects.read().unwrap().values().cloned().collect())
        }

        async fn delete(&self, id: &ProjectId) -> DomainResult<()> {
            self.projects.write().unwrap().remove(id);
            Ok(())
        }

        async fn exists(&self, id: &ProjectId) -> DomainResult<bool> {
            Ok(self.projects.read().unwrap().contains_key(id))
        }
    }

    struct ProjectRepoHarness {
        repo: InMemoryProjectRepoStub,
    }

    #[async_trait]
    impl ProjectRepositoryContract for ProjectRepoHarness {
        fn repo(&self) -> &dyn ProjectRepository {
            &self.repo
        }

        async fn reset(&self) {
            self.repo.projects.write().unwrap().clear();
        }
    }

    #[tokio::test]
    async fn test_project_repository_contracts() {
        let harness = ProjectRepoHarness {
            repo: InMemoryProjectRepoStub::new(),
        };
        run_project_repository_contracts(&harness).await;
    }

    /// In-memory session repository for contract testing
    pub struct InMemorySessionRepoStub {
        sessions: RwLock<HashMap<SessionId, Session>>,
    }

    impl InMemorySessionRepoStub {
        pub fn new() -> Self {
            Self {
                sessions: RwLock::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl SessionRepository for InMemorySessionRepoStub {
        async fn save(&self, session: &Session) -> DomainResult<()> {
            self.sessions.write().unwrap().insert(session.id().clone(), session.clone());
            Ok(())
        }

        async fn find_by_id(&self, id: &SessionId) -> DomainResult<Option<Session>> {
            Ok(self.sessions.read().unwrap().get(id).cloned())
        }

        async fn find_by_project(&self, project_id: &ProjectId) -> DomainResult<Vec<Session>> {
            Ok(self.sessions.read().unwrap()
                .values()
                .filter(|s| s.project_id() == *project_id)
                .cloned()
                .collect())
        }

        async fn find_active(&self) -> DomainResult<Vec<Session>> {
            use ricecoder_domain::session::SessionState;
            Ok(self.sessions.read().unwrap()
                .values()
                .filter(|s| matches!(s.state(), SessionState::Active))
                .cloned()
                .collect())
        }

        async fn delete(&self, id: &SessionId) -> DomainResult<()> {
            self.sessions.write().unwrap().remove(id);
            Ok(())
        }
    }

    struct SessionRepoHarness {
        repo: InMemorySessionRepoStub,
    }

    #[async_trait]
    impl SessionRepositoryContract for SessionRepoHarness {
        fn repo(&self) -> &dyn SessionRepository {
            &self.repo
        }

        async fn reset(&self) {
            self.repo.sessions.write().unwrap().clear();
        }
    }

    #[tokio::test]
    async fn test_session_repository_contracts() {
        let harness = SessionRepoHarness {
            repo: InMemorySessionRepoStub::new(),
        };
        run_session_repository_contracts(&harness).await;
    }

    /// In-memory specification repository for contract testing
    pub struct InMemorySpecRepoStub {
        specs: RwLock<HashMap<SpecificationId, Specification>>,
    }

    impl InMemorySpecRepoStub {
        pub fn new() -> Self {
            Self {
                specs: RwLock::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl SpecificationReader for InMemorySpecRepoStub {
        async fn find_by_id(&self, id: &SpecificationId) -> DomainResult<Option<Specification>> {
            Ok(self.specs.read().unwrap().get(id).cloned())
        }

        async fn find_by_project(&self, project_id: &ProjectId) -> DomainResult<Vec<Specification>> {
            Ok(self.specs.read().unwrap()
                .values()
                .filter(|s| s.project_id() == *project_id)
                .cloned()
                .collect())
        }

        async fn find_all(&self) -> DomainResult<Vec<Specification>> {
            Ok(self.specs.read().unwrap().values().cloned().collect())
        }

        async fn exists(&self, id: &SpecificationId) -> DomainResult<bool> {
            Ok(self.specs.read().unwrap().contains_key(id))
        }

        async fn find_by_status(
            &self,
            status: ricecoder_domain::specification::SpecStatus,
        ) -> DomainResult<Vec<Specification>> {
            Ok(self.specs.read().unwrap()
                .values()
                .filter(|s| s.status() == status)
                .cloned()
                .collect())
        }
    }

    #[async_trait]
    impl SpecificationWriter for InMemorySpecRepoStub {
        async fn save(&self, spec: &Specification) -> DomainResult<()> {
            self.specs.write().unwrap().insert(spec.id().clone(), spec.clone());
            Ok(())
        }

        async fn delete(&self, id: &SpecificationId) -> DomainResult<()> {
            self.specs.write().unwrap().remove(id);
            Ok(())
        }
    }

    struct SpecRepoHarness {
        repo: InMemorySpecRepoStub,
    }

    #[async_trait]
    impl SpecificationRepositoryContract for SpecRepoHarness {
        fn repo(&self) -> &dyn SpecificationRepository {
            &self.repo
        }

        async fn reset(&self) {
            self.repo.specs.write().unwrap().clear();
        }
    }

    #[tokio::test]
    async fn test_specification_repository_contracts() {
        let harness = SpecRepoHarness {
            repo: InMemorySpecRepoStub::new(),
        };
        run_specification_repository_contracts(&harness).await;
    }
}
