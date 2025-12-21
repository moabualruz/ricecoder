//! Core types for RiceCoder storage

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Path to global storage directory
    pub global_path: PathBuf,
    /// Path to project storage directory (if in a project)
    pub project_path: Option<PathBuf>,
    /// Storage mode (how to combine global and project storage)
    pub mode: StorageMode,
    /// Whether this is the first initialization
    pub first_run: bool,
}

/// Storage mode determines how global and project storage are combined
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageMode {
    /// Only use global storage
    GlobalOnly,
    /// Only use project storage
    ProjectOnly,
    /// Merge both, with project overriding global
    Merged,
}

/// Resource types that can be stored
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    /// Templates for code generation
    Template,
    /// Coding standards and guidelines
    Standard,
    /// Specification documents
    Spec,
    /// Steering documents (project rules)
    Steering,
    /// Boilerplate projects
    Boilerplate,
    /// Learned rules from the learning system
    Rule,
    /// Custom command definitions
    CustomCommand,
    /// LSP language configuration files
    LspLanguageConfig,
    /// Code completion language configuration files
    CompletionLanguageConfig,
    /// Hooks configuration files
    HooksConfig,
    /// Refactoring language configuration files
    RefactoringLanguageConfig,
}

impl ResourceType {
    /// Get the directory name for this resource type
    pub fn dir_name(&self) -> &'static str {
        match self {
            ResourceType::Template => "templates",
            ResourceType::Standard => "standards",
            ResourceType::Spec => "specs",
            ResourceType::Steering => "steering",
            ResourceType::Boilerplate => "boilerplates",
            ResourceType::Rule => "rules",
            ResourceType::CustomCommand => "commands",
            ResourceType::LspLanguageConfig => "lsp/languages",
            ResourceType::CompletionLanguageConfig => "completion/languages",
            ResourceType::HooksConfig => "hooks",
            ResourceType::RefactoringLanguageConfig => "refactoring/languages",
        }
    }
}

/// Configuration file format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigFormat {
    /// YAML format (.yaml, .yml)
    Yaml,
    /// TOML format (.toml)
    Toml,
    /// JSON format (.json)
    Json,
    /// JSONC format (.jsonc) - JSON with comments
    Jsonc,
}

impl ConfigFormat {
    /// Get the file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            ConfigFormat::Yaml => "yaml",
            ConfigFormat::Toml => "toml",
            ConfigFormat::Json => "json",
            ConfigFormat::Jsonc => "jsonc",
        }
    }

    /// Detect format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "yaml" | "yml" => Some(ConfigFormat::Yaml),
            "toml" => Some(ConfigFormat::Toml),
            "json" => Some(ConfigFormat::Json),
            "jsonc" => Some(ConfigFormat::Jsonc),
            _ => None,
        }
    }
}

/// Document format for steering and specs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentFormat {
    /// YAML format
    Yaml,
    /// Markdown format
    Markdown,
}

impl DocumentFormat {
    /// Get the file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            DocumentFormat::Yaml => "yaml",
            DocumentFormat::Markdown => "md",
        }
    }

    /// Detect format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "yaml" | "yml" => Some(DocumentFormat::Yaml),
            "md" | "markdown" => Some(DocumentFormat::Markdown),
            _ => None,
        }
    }
}

/// Storage availability state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageState {
    /// Storage is available and writable
    Available,
    /// Storage is unavailable (e.g., external drive disconnected)
    Unavailable { reason: String },
    /// Storage is available but read-only (e.g., offline mode)
    ReadOnly { cached_at: String },
}
