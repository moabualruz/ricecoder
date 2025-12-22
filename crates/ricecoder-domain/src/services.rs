//! Domain services - business logic that doesn't belong to entities

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{entities::*, errors::*, value_objects::*};

/// Service for managing projects
#[async_trait]
pub trait ProjectService {
    /// Create a new project
    async fn create_project(
        &self,
        name: String,
        language: ProgrammingLanguage,
        root_path: String,
    ) -> DomainResult<Project>;

    /// Get project by ID
    async fn get_project(&self, id: &ProjectId) -> DomainResult<Option<Project>>;

    /// Update project
    async fn update_project(&self, project: Project) -> DomainResult<Project>;

    /// Delete project
    async fn delete_project(&self, id: &ProjectId) -> DomainResult<()>;

    /// List all projects
    async fn list_projects(&self) -> DomainResult<Vec<Project>>;
}

/// Service for managing sessions
#[async_trait]
pub trait SessionService {
    /// Create a new session
    async fn create_session(&self, provider_id: String, model_id: String) -> DomainResult<Session>;

    /// Get session by ID
    async fn get_session(&self, id: &SessionId) -> DomainResult<Option<Session>>;

    /// Update session
    async fn update_session(&self, session: Session) -> DomainResult<Session>;

    /// End session
    async fn end_session(&self, id: &SessionId) -> DomainResult<()>;

    /// List active sessions
    async fn list_active_sessions(&self) -> DomainResult<Vec<Session>>;

    /// Generate shareable session link
    async fn generate_share_link(&self, session_id: &SessionId) -> DomainResult<String>;

    /// Load session from shareable link
    async fn load_from_share_link(&self, link: &str) -> DomainResult<Session>;
}

/// Service for managing providers
#[async_trait]
pub trait ProviderService {
    /// Register a new provider
    async fn register_provider(&self, provider: Provider) -> DomainResult<()>;

    /// Get provider by ID
    async fn get_provider(&self, id: &str) -> DomainResult<Option<Provider>>;

    /// Update provider
    async fn update_provider(&self, provider: Provider) -> DomainResult<()>;

    /// List all providers
    async fn list_providers(&self) -> DomainResult<Vec<Provider>>;

    /// Get available models for a provider
    async fn get_provider_models(&self, provider_id: &str) -> DomainResult<Vec<ModelInfo>>;

    /// Validate provider configuration
    async fn validate_provider_config(&self, provider: &Provider)
        -> DomainResult<ValidationResult>;
}

/// Service for code analysis
#[async_trait]
pub trait AnalysisService {
    /// Analyze a single file
    async fn analyze_file(&self, file: &CodeFile) -> DomainResult<AnalysisResult>;

    /// Analyze an entire project
    async fn analyze_project(&self, project: &Project) -> DomainResult<Vec<AnalysisResult>>;

    /// Get analysis history for a project
    async fn get_analysis_history(
        &self,
        project_id: &ProjectId,
    ) -> DomainResult<Vec<AnalysisResult>>;

    /// Get analysis result by ID
    async fn get_analysis_result(&self, id: &str) -> DomainResult<Option<AnalysisResult>>;
}

/// Service for file operations
#[async_trait]
pub trait FileService {
    /// Load file from path
    async fn load_file(
        &self,
        project_id: &ProjectId,
        relative_path: &str,
    ) -> DomainResult<CodeFile>;

    /// Save file content
    async fn save_file(&self, file: &CodeFile) -> DomainResult<()>;

    /// List files in project
    async fn list_project_files(&self, project_id: &ProjectId) -> DomainResult<Vec<CodeFile>>;

    /// Search files by pattern
    async fn search_files(
        &self,
        project_id: &ProjectId,
        pattern: &str,
    ) -> DomainResult<Vec<CodeFile>>;
}

/// Validation result for domain operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn invalid(errors: Vec<String>) -> Self {
        Self {
            is_valid: false,
            errors,
            warnings: Vec::new(),
        }
    }

    pub fn with_warnings(mut self, warnings: Vec<String>) -> Self {
        self.warnings = warnings;
        self
    }

    pub fn add_error(&mut self, error: String) {
        self.is_valid = false;
        self.errors.push(error);
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
}

/// Business rules validation service
pub struct BusinessRulesValidator;

impl BusinessRulesValidator {
    /// Validate project creation rules
    pub fn validate_project_creation(
        name: &str,
        language: ProgrammingLanguage,
    ) -> ValidationResult {
        let mut result = ValidationResult::valid();

        // Rule: Project names should be meaningful
        if name.len() < 3 {
            result.add_warning(
                "Project name is very short, consider a more descriptive name".to_string(),
            );
        }

        // Rule: Certain languages have specific requirements
        match language {
            ProgrammingLanguage::Rust => {
                // Rust projects should have proper naming
                if !name.chars().any(|c| c.is_lowercase()) {
                    result.add_warning("Rust project names are typically lowercase".to_string());
                }
            }
            ProgrammingLanguage::Python => {
                // Python projects should follow PEP 8
                if name.contains('_') {
                    result.add_warning(
                        "Python project names typically use hyphens instead of underscores"
                            .to_string(),
                    );
                }
            }
            _ => {}
        }

        result
    }

    /// Validate session operations
    pub fn validate_session_operation(
        session: &Session,
        operation: SessionOperation,
    ) -> ValidationResult {
        let mut result = ValidationResult::valid();

        match operation {
            SessionOperation::End if !session.is_active() => {
                result.add_error("Cannot end a session that is not active".to_string());
            }
            SessionOperation::Resume if session.status != SessionStatus::Paused => {
                result.add_error("Can only resume paused sessions".to_string());
            }
            SessionOperation::Pause if !session.is_active() => {
                result.add_error("Can only pause active sessions".to_string());
            }
            _ => {}
        }

        result
    }

    /// Validate analysis operations
    pub fn validate_analysis_operation(
        file: &CodeFile,
        analysis_type: AnalysisType,
    ) -> ValidationResult {
        let mut result = ValidationResult::valid();

        // Rule: Don't analyze empty files
        if file.is_empty() {
            result.add_warning(
                "Analyzing empty files may not provide meaningful results".to_string(),
            );
        }

        // Rule: Certain analyses require minimum file size
        match analysis_type {
            AnalysisType::Complexity if file.size_bytes < 100 => {
                result.add_warning(
                    "Complexity analysis may not be meaningful for very small files".to_string(),
                );
            }
            AnalysisType::Dependencies
                if !matches!(
                    file.language,
                    ProgrammingLanguage::Rust | ProgrammingLanguage::Go
                ) =>
            {
                result.add_warning(format!(
                    "Dependency analysis is most useful for languages like {}, not {}",
                    ProgrammingLanguage::Rust,
                    file.language
                ));
            }
            _ => {}
        }

        result
    }
}

/// Session operation types for validation
#[derive(Debug, Clone, Copy)]
pub enum SessionOperation {
    End,
    Pause,
    Resume,
    Update,
}
