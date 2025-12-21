//! Core domain entities with business logic and validation

use crate::errors::*;
use crate::value_objects::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Core project entity representing a code project being analyzed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: ProjectId,
    pub name: String,
    pub description: Option<String>,
    pub language: ProgrammingLanguage,
    pub root_path: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

impl Project {
    /// Create a new project with validation
    pub fn new(
        name: String,
        language: ProgrammingLanguage,
        root_path: String,
    ) -> DomainResult<Self> {
        Self::validate_name(&name)?;
        Self::validate_path(&root_path)?;

        let now = Utc::now();
        Ok(Self {
            id: ProjectId::new(),
            name,
            description: None,
            language,
            root_path,
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        })
    }

    /// Update project name with validation
    pub fn update_name(&mut self, name: String) -> DomainResult<()> {
        Self::validate_name(&name)?;
        self.name = name;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Update project description
    pub fn update_description(&mut self, description: Option<String>) {
        self.description = description;
        self.updated_at = Utc::now();
    }

    /// Add metadata
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
        self.updated_at = Utc::now();
    }

    /// Validate project name
    fn validate_name(name: &str) -> DomainResult<()> {
        if name.trim().is_empty() {
            return Err(DomainError::InvalidProjectName {
                reason: "Project name cannot be empty".to_string(),
            });
        }

        if name.len() > 100 {
            return Err(DomainError::InvalidProjectName {
                reason: "Project name cannot exceed 100 characters".to_string(),
            });
        }

        // Check for valid characters (alphanumeric, dash, underscore)
        if !regex::Regex::new(r"^[a-zA-Z0-9_-]+$")
            .unwrap()
            .is_match(name)
        {
            return Err(DomainError::InvalidProjectName {
                reason: "Project name can only contain letters, numbers, dashes, and underscores"
                    .to_string(),
            });
        }

        Ok(())
    }

    /// Validate project path
    fn validate_path(path: &str) -> DomainResult<()> {
        if path.trim().is_empty() {
            return Err(DomainError::InvalidFilePath {
                reason: "Project path cannot be empty".to_string(),
            });
        }

        // Basic path validation - could be enhanced
        if path.contains("..") {
            return Err(DomainError::InvalidFilePath {
                reason: "Project path cannot contain '..'".to_string(),
            });
        }

        Ok(())
    }
}

/// File entity representing a source code file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeFile {
    pub id: FileId,
    pub project_id: ProjectId,
    pub relative_path: String,
    pub language: ProgrammingLanguage,
    pub content: String,
    pub size_bytes: usize,
    pub mime_type: MimeType,
    pub last_modified: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

impl CodeFile {
    /// Create a new code file
    pub fn new(
        project_id: ProjectId,
        relative_path: String,
        content: String,
        language: ProgrammingLanguage,
    ) -> DomainResult<Self> {
        let id = FileId::from_path(&relative_path);

        Ok(Self {
            id,
            project_id,
            relative_path: relative_path.clone(),
            language,
            content: content.clone(),
            size_bytes: content.len(),
            mime_type: MimeType::from_path(&relative_path),
            last_modified: Utc::now(),
            metadata: HashMap::new(),
        })
    }

    /// Update file content
    pub fn update_content(&mut self, content: String) {
        self.content = content.clone();
        self.size_bytes = content.len();
        self.last_modified = Utc::now();
    }

    /// Check if file is empty
    pub fn is_empty(&self) -> bool {
        self.content.trim().is_empty()
    }

    /// Get file extension
    pub fn extension(&self) -> Option<&str> {
        std::path::Path::new(&self.relative_path)
            .extension()
            .and_then(|ext| ext.to_str())
    }
}

/// Analysis result entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub id: String,
    pub project_id: ProjectId,
    pub file_id: Option<FileId>,
    pub analysis_type: AnalysisType,
    pub status: AnalysisStatus,
    pub results: serde_json::Value,
    pub metrics: AnalysisMetrics,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl AnalysisResult {
    /// Create a new analysis result
    pub fn new(
        project_id: ProjectId,
        file_id: Option<FileId>,
        analysis_type: AnalysisType,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            project_id,
            file_id,
            analysis_type,
            status: AnalysisStatus::Pending,
            results: serde_json::Value::Null,
            metrics: AnalysisMetrics::default(),
            created_at: Utc::now(),
            completed_at: None,
        }
    }

    /// Mark analysis as completed
    pub fn complete(&mut self, results: serde_json::Value, metrics: AnalysisMetrics) {
        self.status = AnalysisStatus::Completed;
        self.results = results;
        self.metrics = metrics;
        self.completed_at = Some(Utc::now());
    }

    /// Mark analysis as failed
    pub fn fail(&mut self, error: String) {
        self.status = AnalysisStatus::Failed;
        self.results = serde_json::Value::String(error);
        self.completed_at = Some(Utc::now());
    }

    /// Check if analysis is complete
    pub fn is_complete(&self) -> bool {
        matches!(
            self.status,
            AnalysisStatus::Completed | AnalysisStatus::Failed
        )
    }
}

/// Analysis type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnalysisType {
    Syntax,
    Semantic,
    Complexity,
    Dependencies,
    Patterns,
    Security,
    Performance,
}

/// Analysis status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnalysisStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

/// Analysis metrics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisMetrics {
    pub lines_of_code: usize,
    pub cyclomatic_complexity: f64,
    pub maintainability_index: f64,
    pub technical_debt_ratio: f64,
    pub execution_time_ms: u64,
}

impl Default for AnalysisMetrics {
    fn default() -> Self {
        Self {
            lines_of_code: 0,
            cyclomatic_complexity: 0.0,
            maintainability_index: 100.0,
            technical_debt_ratio: 0.0,
            execution_time_ms: 0,
        }
    }
}

/// Session entity representing an AI interaction session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub project_id: Option<ProjectId>,
    pub name: Option<String>,
    pub provider_id: String,
    pub model_id: String,
    pub status: SessionStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Session {
    /// Create a new session
    pub fn new(provider_id: String, model_id: String) -> Self {
        let now = Utc::now();
        Self {
            id: SessionId::new(),
            project_id: None,
            name: None,
            provider_id,
            model_id,
            status: SessionStatus::Active,
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        }
    }

    /// Associate session with a project
    pub fn set_project(&mut self, project_id: ProjectId) {
        self.project_id = Some(project_id);
        self.updated_at = Utc::now();
    }

    /// Update session name
    pub fn set_name(&mut self, name: String) {
        self.name = Some(name);
        self.updated_at = Utc::now();
    }

    /// End the session
    pub fn end(&mut self) {
        self.status = SessionStatus::Ended;
        self.updated_at = Utc::now();
    }

    /// Pause the session
    pub fn pause(&mut self) {
        self.status = SessionStatus::Paused;
        self.updated_at = Utc::now();
    }

    /// Resume the session
    pub fn resume(&mut self) {
        if self.status == SessionStatus::Paused {
            self.status = SessionStatus::Active;
            self.updated_at = Utc::now();
        }
    }

    /// Check if session is active
    pub fn is_active(&self) -> bool {
        self.status == SessionStatus::Active
    }
}

/// Session status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionStatus {
    Active,
    Paused,
    Ended,
}

/// User entity representing a system user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

impl User {
    /// Create a new user
    pub fn new(id: String, username: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            username,
            email: None,
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        }
    }

    /// Update username
    pub fn update_username(&mut self, username: String) {
        self.username = username;
        self.updated_at = Utc::now();
    }

    /// Set email
    pub fn set_email(&mut self, email: Option<String>) {
        self.email = email;
        self.updated_at = Utc::now();
    }
}

/// Provider configuration entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    pub id: String,
    pub name: String,
    pub provider_type: ProviderType,
    pub base_url: Option<String>,
    pub models: Vec<ModelInfo>,
    pub config: HashMap<String, serde_json::Value>,
    pub is_active: bool,
}

impl Provider {
    /// Create a new provider
    pub fn new(id: String, name: String, provider_type: ProviderType) -> Self {
        Self {
            id,
            name,
            provider_type,
            base_url: None,
            models: Vec::new(),
            config: HashMap::new(),
            is_active: true,
        }
    }

    /// Add a model to the provider
    pub fn add_model(&mut self, model: ModelInfo) {
        self.models.push(model);
    }

    /// Get a model by ID
    pub fn get_model(&self, model_id: &str) -> Option<&ModelInfo> {
        self.models.iter().find(|m| m.id == model_id)
    }

    /// Check if provider supports a model
    pub fn supports_model(&self, model_id: &str) -> bool {
        self.models.iter().any(|m| m.id == model_id)
    }
}

/// Provider type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderType {
    OpenAI,
    Anthropic,
    Google,
    Local,
    Custom,
}

/// Model information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub context_window: usize,
    pub supports_function_calling: bool,
    pub supports_vision: bool,
    pub cost_per_1m_input: Option<f64>,
    pub cost_per_1m_output: Option<f64>,
}

impl ModelInfo {
    /// Create a new model info
    pub fn new(id: String, name: String, context_window: usize) -> Self {
        Self {
            id,
            name,
            context_window,
            supports_function_calling: false,
            supports_vision: false,
            cost_per_1m_input: None,
            cost_per_1m_output: None,
        }
    }

    /// Enable function calling
    pub fn with_function_calling(mut self) -> Self {
        self.supports_function_calling = true;
        self
    }

    /// Enable vision
    pub fn with_vision(mut self) -> Self {
        self.supports_vision = true;
        self
    }

    /// Set pricing
    pub fn with_pricing(mut self, input_cost: f64, output_cost: f64) -> Self {
        self.cost_per_1m_input = Some(input_cost);
        self.cost_per_1m_output = Some(output_cost);
        self
    }
}
