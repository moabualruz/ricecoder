//! Core types for RiceCoder storage
//!
//! Unified folder structure:
//! ```text
//! ~/Documents/.ricecoder/          # User config folder (OS-appropriate Documents)
//! ├── config/                      # User-editable config files
//! │   ├── config.yaml              # Main config (yaml/json/toml supported)
//! │   ├── agents/                  # User agent overrides
//! │   ├── commands/                # User slash commands
//! │   ├── themes/                  # User custom themes
//! │   ├── prompts/                 # User prompts
//! │   └── tips.txt                 # User tips
//! ├── auth/                        # Credentials (separate for security)
//! │   └── providers.yaml           # API keys per provider
//! ├── storage/                     # Runtime data
//! │   ├── sessions/                # Session data
//! │   ├── messages/                # Message history
//! │   ├── parts/                   # Message parts
//! │   ├── projects/                # Project data
//! │   ├── todo/                    # Todo items
//! │   └── migration                # Migration marker
//! ├── logs/                        # Log files
//! ├── cache/                       # Cached data
//! └── templates/                   # User templates
//!
//! .rice/                           # Project-specific config
//! ├── config.yaml                  # Project config (overrides user)
//! ├── agents/                      # Project-specific agents
//! ├── commands/                    # Project-specific commands
//! └── ...
//! ```

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Top-level directory categories in the unified storage structure
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StorageDirectory {
    /// User-editable config files (config/)
    Config,
    /// Authentication credentials (auth/)
    Auth,
    /// Runtime data storage (storage/)
    Storage,
    /// Log files (logs/)
    Logs,
    /// Cached data (cache/)
    Cache,
    /// User templates (templates/)
    Templates,
}

impl StorageDirectory {
    /// Get the directory name for this storage category
    pub fn dir_name(&self) -> &'static str {
        match self {
            StorageDirectory::Config => "config",
            StorageDirectory::Auth => "auth",
            StorageDirectory::Storage => "storage",
            StorageDirectory::Logs => "logs",
            StorageDirectory::Cache => "cache",
            StorageDirectory::Templates => "templates",
        }
    }

    /// Get all directories that should be created on initialization
    pub fn all() -> &'static [StorageDirectory] {
        &[
            StorageDirectory::Config,
            StorageDirectory::Auth,
            StorageDirectory::Storage,
            StorageDirectory::Logs,
            StorageDirectory::Cache,
            StorageDirectory::Templates,
        ]
    }
}

/// Subdirectories within the storage/ directory
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RuntimeStorageType {
    /// Session data
    Sessions,
    /// Message history
    Messages,
    /// Message parts (attachments, etc.)
    Parts,
    /// Project-specific data
    Projects,
    /// Todo items
    Todo,
    /// Session diffs
    SessionDiff,
    /// Agent usage reminders
    AgentUsageReminder,
    /// Directory agents
    DirectoryAgents,
    /// Directory readme
    DirectoryReadme,
}

impl RuntimeStorageType {
    /// Get the directory name
    pub fn dir_name(&self) -> &'static str {
        match self {
            RuntimeStorageType::Sessions => "sessions",
            RuntimeStorageType::Messages => "messages",
            RuntimeStorageType::Parts => "parts",
            RuntimeStorageType::Projects => "projects",
            RuntimeStorageType::Todo => "todo",
            RuntimeStorageType::SessionDiff => "session_diff",
            RuntimeStorageType::AgentUsageReminder => "agent-usage-reminder",
            RuntimeStorageType::DirectoryAgents => "directory-agents",
            RuntimeStorageType::DirectoryReadme => "directory-readme",
        }
    }

    /// Get all storage types that should be created on initialization
    pub fn all() -> &'static [RuntimeStorageType] {
        &[
            RuntimeStorageType::Sessions,
            RuntimeStorageType::Messages,
            RuntimeStorageType::Parts,
            RuntimeStorageType::Projects,
            RuntimeStorageType::Todo,
            RuntimeStorageType::SessionDiff,
            RuntimeStorageType::AgentUsageReminder,
            RuntimeStorageType::DirectoryAgents,
            RuntimeStorageType::DirectoryReadme,
        ]
    }
}

/// Subdirectories within the config/ directory
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConfigSubdirectory {
    /// Agent definitions
    Agents,
    /// Slash commands
    Commands,
    /// Theme files
    Themes,
    /// Prompt templates
    Prompts,
}

impl ConfigSubdirectory {
    /// Get the directory name
    pub fn dir_name(&self) -> &'static str {
        match self {
            ConfigSubdirectory::Agents => "agents",
            ConfigSubdirectory::Commands => "commands",
            ConfigSubdirectory::Themes => "themes",
            ConfigSubdirectory::Prompts => "prompts",
        }
    }

    /// Get all config subdirectories
    pub fn all() -> &'static [ConfigSubdirectory] {
        &[
            ConfigSubdirectory::Agents,
            ConfigSubdirectory::Commands,
            ConfigSubdirectory::Themes,
            ConfigSubdirectory::Prompts,
        ]
    }
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Path to global storage directory (~/Documents/.ricecoder/)
    pub global_path: PathBuf,
    /// Path to project storage directory (.rice/)
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
    /// Governance documents (project rules)
    Governance,
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
            ResourceType::Governance => "Governance",
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

/// Document format for Governance and specs
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
