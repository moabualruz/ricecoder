//! Code Application Service
//!
//! Orchestrates code-related use cases including code generation,
//! analysis, and refactoring operations.
//!
//! This service coordinates between domain services and infrastructure
//! to provide code operations.

use std::sync::Arc;

use crate::errors::{ApplicationError, ApplicationResult};
use crate::ports::UnitOfWork;

use ricecoder_domain::{ProjectId, ProjectRepository};

/// Code generation request
#[derive(Debug, Clone)]
pub struct CodeGenerationRequest {
    /// Project ID
    pub project_id: String,
    /// File path to generate code for
    pub file_path: String,
    /// Description or prompt for code generation
    pub prompt: String,
    /// Optional context from existing code
    pub context: Option<String>,
}

/// Code generation result
#[derive(Debug, Clone)]
pub struct CodeGenerationResult {
    /// Generated code content
    pub content: String,
    /// File path
    pub file_path: String,
    /// Language detected/used
    pub language: String,
    /// Confidence score (0.0-1.0)
    pub confidence: f32,
}

/// Code analysis request
#[derive(Debug, Clone)]
pub struct CodeAnalysisRequest {
    /// Project ID
    pub project_id: String,
    /// File paths to analyze
    pub file_paths: Vec<String>,
    /// Type of analysis to perform
    pub analysis_type: AnalysisType,
}

/// Type of code analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalysisType {
    /// Complexity analysis
    Complexity,
    /// Quality analysis
    Quality,
    /// Security analysis
    Security,
    /// Performance analysis
    Performance,
    /// Full analysis (all types)
    Full,
}

/// Code analysis result
#[derive(Debug, Clone)]
pub struct CodeAnalysisResult {
    /// Overall score (0-100)
    pub score: u32,
    /// Issues found
    pub issues: Vec<CodeIssue>,
    /// Recommendations
    pub recommendations: Vec<String>,
}

/// Code issue found during analysis
#[derive(Debug, Clone)]
pub struct CodeIssue {
    /// Issue severity
    pub severity: IssueSeverity,
    /// Issue message
    pub message: String,
    /// File path
    pub file_path: String,
    /// Line number (if applicable)
    pub line: Option<u32>,
}

/// Issue severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueSeverity {
    /// Informational
    Info,
    /// Warning
    Warning,
    /// Error
    Error,
    /// Critical
    Critical,
}

/// Code Application Service
///
/// Orchestrates code-related use cases. Stateless: all mutable state
/// is persisted via repositories.
pub struct CodeService<PR, U>
where
    PR: ProjectRepository + Send + Sync,
    U: UnitOfWork + Send + Sync,
{
    project_repository: Arc<PR>,
    uow: Arc<U>,
}

impl<PR, U> CodeService<PR, U>
where
    PR: ProjectRepository + Send + Sync,
    U: UnitOfWork + Send + Sync,
{
    /// Create a new CodeService with injected dependencies
    pub fn new(project_repository: Arc<PR>, uow: Arc<U>) -> Self {
        Self {
            project_repository,
            uow,
        }
    }

    /// Generate code based on a prompt
    ///
    /// This is a placeholder implementation. In a real system, this would
    /// integrate with AI providers to generate code.
    pub async fn generate_code(
        &self,
        request: CodeGenerationRequest,
    ) -> ApplicationResult<CodeGenerationResult> {
        // Validate project exists
        let project_id = ProjectId::from_string(&request.project_id)
            .map_err(|_| ApplicationError::ValidationFailed("Invalid project ID".into()))?;

        let project_exists = self
            .project_repository
            .exists(&project_id)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?;

        if !project_exists {
            return Err(ApplicationError::ProjectNotFound(request.project_id));
        }

        // In a real implementation, this would:
        // 1. Load project context
        // 2. Build prompt with context
        // 3. Call AI provider
        // 4. Validate generated code
        // 5. Return result

        // Placeholder: Real implementation requires AI provider integration
        // which belongs in Infrastructure Layer. This stub validates the
        // orchestration pattern and ensures project existence checks work.
        // See: ricecoder-providers crate for actual AI integration.
        Ok(CodeGenerationResult {
            content: format!("// Generated code for: {}\n// Awaiting AI provider integration", request.prompt),
            file_path: request.file_path,
            language: "rust".to_string(),
            confidence: 0.0, // Zero confidence = placeholder
        })
    }

    /// Analyze code for issues
    ///
    /// This is a placeholder implementation. In a real system, this would
    /// run static analysis tools and AI-powered analysis.
    pub async fn analyze_code(
        &self,
        request: CodeAnalysisRequest,
    ) -> ApplicationResult<CodeAnalysisResult> {
        // Validate project exists
        let project_id = ProjectId::from_string(&request.project_id)
            .map_err(|_| ApplicationError::ValidationFailed("Invalid project ID".into()))?;

        let project_exists = self
            .project_repository
            .exists(&project_id)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?;

        if !project_exists {
            return Err(ApplicationError::ProjectNotFound(request.project_id));
        }

        // Validate at least one file path
        if request.file_paths.is_empty() {
            return Err(ApplicationError::ValidationFailed(
                "At least one file path is required".into(),
            ));
        }

        // In a real implementation, this would:
        // 1. Load files
        // 2. Run analysis based on analysis_type
        // 3. Aggregate results
        // 4. Return analysis result

        // For now, return a placeholder result
        Ok(CodeAnalysisResult {
            score: 80,
            issues: vec![],
            recommendations: vec![
                "Consider adding more tests".to_string(),
                "Review error handling".to_string(),
            ],
        })
    }

    /// Validate code syntax
    pub async fn validate_syntax(
        &self,
        project_id: &str,
        file_path: &str,
        content: &str,
    ) -> ApplicationResult<Vec<CodeIssue>> {
        // Validate project exists
        let pid = ProjectId::from_string(project_id)
            .map_err(|_| ApplicationError::ValidationFailed("Invalid project ID".into()))?;

        let project_exists = self
            .project_repository
            .exists(&pid)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?;

        if !project_exists {
            return Err(ApplicationError::ProjectNotFound(project_id.to_string()));
        }

        // In a real implementation, this would:
        // 1. Detect language from file extension
        // 2. Parse content with language-specific parser
        // 3. Return syntax errors

        // For now, return empty issues (no syntax errors)
        let _ = (file_path, content); // Silence unused warnings
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::NoOpUnitOfWork;
    use async_trait::async_trait;
    use ricecoder_domain::DomainResult;
    use ricecoder_domain::project::Project;
    use ricecoder_domain::value_objects::ProgrammingLanguage;
    use std::collections::HashMap;
    use std::sync::Mutex;

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
    async fn test_generate_code_success() {
        let project_repo = Arc::new(InMemoryProjectRepository::new());
        let uow = Arc::new(NoOpUnitOfWork);

        // Create a project first
        let (project, _events) = Project::create(
            "test-project".to_string(),
            ProgrammingLanguage::Rust,
            "/path".to_string(),
            None,
        ).unwrap();
        let project_id = project.id().to_string();
        project_repo.add_project(project);

        let service = CodeService::new(project_repo, uow);

        let request = CodeGenerationRequest {
            project_id,
            file_path: "src/main.rs".to_string(),
            prompt: "Create a hello world function".to_string(),
            context: None,
        };

        let result = service.generate_code(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_generate_code_project_not_found() {
        let project_repo = Arc::new(InMemoryProjectRepository::new());
        let uow = Arc::new(NoOpUnitOfWork);

        let service = CodeService::new(project_repo, uow);

        let request = CodeGenerationRequest {
            project_id: "00000000-0000-0000-0000-000000000000".to_string(),
            file_path: "src/main.rs".to_string(),
            prompt: "Create a hello world function".to_string(),
            context: None,
        };

        let result = service.generate_code(request).await;
        assert!(matches!(result, Err(ApplicationError::ProjectNotFound(_))));
    }

    #[tokio::test]
    async fn test_analyze_code_success() {
        let project_repo = Arc::new(InMemoryProjectRepository::new());
        let uow = Arc::new(NoOpUnitOfWork);

        // Create a project first
        let (project, _events) = Project::create(
            "test-project".to_string(),
            ProgrammingLanguage::Rust,
            "/path".to_string(),
            None,
        ).unwrap();
        let project_id = project.id().to_string();
        project_repo.add_project(project);

        let service = CodeService::new(project_repo, uow);

        let request = CodeAnalysisRequest {
            project_id,
            file_paths: vec!["src/main.rs".to_string()],
            analysis_type: AnalysisType::Quality,
        };

        let result = service.analyze_code(request).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().score, 80);
    }

    #[tokio::test]
    async fn test_analyze_code_empty_files() {
        let project_repo = Arc::new(InMemoryProjectRepository::new());
        let uow = Arc::new(NoOpUnitOfWork);

        // Create a project first
        let (project, _events) = Project::create(
            "test-project".to_string(),
            ProgrammingLanguage::Rust,
            "/path".to_string(),
            None,
        ).unwrap();
        let project_id = project.id().to_string();
        project_repo.add_project(project);

        let service = CodeService::new(project_repo, uow);

        let request = CodeAnalysisRequest {
            project_id,
            file_paths: vec![],
            analysis_type: AnalysisType::Quality,
        };

        let result = service.analyze_code(request).await;
        assert!(matches!(result, Err(ApplicationError::ValidationFailed(_))));
    }
}
