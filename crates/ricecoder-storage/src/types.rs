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
}

impl ConfigFormat {
    /// Get the file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            ConfigFormat::Yaml => "yaml",
            ConfigFormat::Toml => "toml",
            ConfigFormat::Json => "json",
        }
    }

    /// Detect format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "yaml" | "yml" => Some(ConfigFormat::Yaml),
            "toml" => Some(ConfigFormat::Toml),
            "json" => Some(ConfigFormat::Json),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_type_dir_names() {
        assert_eq!(ResourceType::Template.dir_name(), "templates");
        assert_eq!(ResourceType::Standard.dir_name(), "standards");
        assert_eq!(ResourceType::Spec.dir_name(), "specs");
        assert_eq!(ResourceType::Steering.dir_name(), "steering");
        assert_eq!(ResourceType::Boilerplate.dir_name(), "boilerplates");
        assert_eq!(ResourceType::Rule.dir_name(), "rules");
    }

    #[test]
    fn test_config_format_extensions() {
        assert_eq!(ConfigFormat::Yaml.extension(), "yaml");
        assert_eq!(ConfigFormat::Toml.extension(), "toml");
        assert_eq!(ConfigFormat::Json.extension(), "json");
    }

    #[test]
    fn test_config_format_detection() {
        assert_eq!(ConfigFormat::from_extension("yaml"), Some(ConfigFormat::Yaml));
        assert_eq!(ConfigFormat::from_extension("yml"), Some(ConfigFormat::Yaml));
        assert_eq!(ConfigFormat::from_extension("toml"), Some(ConfigFormat::Toml));
        assert_eq!(ConfigFormat::from_extension("json"), Some(ConfigFormat::Json));
        assert_eq!(ConfigFormat::from_extension("txt"), None);
    }

    #[test]
    fn test_document_format_detection() {
        assert_eq!(
            DocumentFormat::from_extension("yaml"),
            Some(DocumentFormat::Yaml)
        );
        assert_eq!(
            DocumentFormat::from_extension("md"),
            Some(DocumentFormat::Markdown)
        );
        assert_eq!(
            DocumentFormat::from_extension("markdown"),
            Some(DocumentFormat::Markdown)
        );
        assert_eq!(DocumentFormat::from_extension("txt"), None);
    }
}
