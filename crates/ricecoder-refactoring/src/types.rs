//! Core data types for the refactoring engine

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// A refactoring operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Refactoring {
    /// Unique identifier for the refactoring
    pub id: String,
    /// Type of refactoring to perform
    pub refactoring_type: RefactoringType,
    /// Target of the refactoring
    pub target: RefactoringTarget,
    /// Options for the refactoring
    pub options: RefactoringOptions,
}

/// Types of refactoring operations
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RefactoringType {
    /// Rename a symbol
    Rename,
    /// Extract code into a new function/method
    Extract,
    /// Inline a function/method
    Inline,
    /// Move a symbol to another location
    Move,
    /// Change function signature
    ChangeSignature,
    /// Remove unused code
    RemoveUnused,
    /// Simplify code
    Simplify,
}

/// Target of a refactoring operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringTarget {
    /// File containing the target
    pub file: PathBuf,
    /// Symbol name to refactor
    pub symbol: String,
    /// Range in the file (line:col - line:col)
    pub range: Option<String>,
}

/// Options for refactoring operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringOptions {
    /// Perform a dry run without applying changes
    pub dry_run: bool,
    /// Automatically rollback on failure
    pub auto_rollback_on_failure: bool,
    /// Run tests after refactoring
    pub run_tests_after: bool,
    /// Additional options as key-value pairs
    pub extra: HashMap<String, String>,
}

impl Default for RefactoringOptions {
    fn default() -> Self {
        Self {
            dry_run: false,
            auto_rollback_on_failure: true,
            run_tests_after: false,
            extra: HashMap::new(),
        }
    }
}

/// Result of a refactoring operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringResult {
    /// Changes made by the refactoring
    pub changes: Vec<FileChange>,
    /// Impact analysis of the changes
    pub impact: Option<String>,
    /// Validation result
    pub validation: Option<String>,
    /// Whether the refactoring succeeded
    pub success: bool,
}

/// A change to a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    /// Path to the file
    pub file: PathBuf,
    /// Original content
    pub original: String,
    /// New content
    pub new: String,
    /// Type of change
    pub change_type: ChangeType,
}

/// Type of change to a file
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChangeType {
    /// File was modified
    Modified,
    /// File was created
    Created,
    /// File was deleted
    Deleted,
    /// File was renamed
    Renamed,
}

impl std::fmt::Display for RefactoringType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RefactoringType::Rename => write!(f, "rename"),
            RefactoringType::Extract => write!(f, "extract"),
            RefactoringType::Inline => write!(f, "inline"),
            RefactoringType::Move => write!(f, "move"),
            RefactoringType::ChangeSignature => write!(f, "change_signature"),
            RefactoringType::RemoveUnused => write!(f, "remove_unused"),
            RefactoringType::Simplify => write!(f, "simplify"),
        }
    }
}

/// Result of validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether validation passed
    pub passed: bool,
    /// Validation errors
    pub errors: Vec<String>,
    /// Validation warnings
    pub warnings: Vec<String>,
}

/// Configuration for a language's refactoring rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringConfig {
    /// Language name
    pub language: String,
    /// File extensions for this language
    pub extensions: Vec<String>,
    /// Refactoring rules
    pub rules: Vec<RefactoringRule>,
    /// Refactoring transformations
    pub transformations: Vec<RefactoringTransformation>,
    /// Optional provider reference (e.g., LSP server name)
    pub provider: Option<String>,
}

impl RefactoringConfig {
    /// Validate the configuration
    pub fn validate(&self) -> crate::error::Result<()> {
        if self.language.is_empty() {
            return Err(crate::error::RefactoringError::ConfigError(
                "Language name cannot be empty".to_string(),
            ));
        }
        if self.extensions.is_empty() {
            return Err(crate::error::RefactoringError::ConfigError(
                "At least one file extension must be specified".to_string(),
            ));
        }
        Ok(())
    }

    /// Create a generic fallback configuration for a language
    pub fn generic_fallback(language: &str) -> Self {
        Self {
            language: language.to_string(),
            extensions: vec![],
            rules: vec![],
            transformations: vec![],
            provider: None,
        }
    }
}

/// A refactoring rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringRule {
    /// Rule name
    pub name: String,
    /// Pattern to match
    pub pattern: String,
    /// Type of refactoring
    pub refactoring_type: RefactoringType,
    /// Whether the rule is enabled
    pub enabled: bool,
}

/// A refactoring transformation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringTransformation {
    /// Transformation name
    pub name: String,
    /// Pattern to match
    pub from_pattern: String,
    /// Replacement pattern
    pub to_pattern: String,
    /// Description
    pub description: String,
}

/// Impact analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAnalysis {
    /// Affected files
    pub affected_files: Vec<PathBuf>,
    /// Affected symbols
    pub affected_symbols: Vec<String>,
    /// Risk level
    pub risk_level: RiskLevel,
    /// Estimated effort (1-10)
    pub estimated_effort: u8,
}

/// Risk level for a refactoring
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RiskLevel {
    /// Low risk
    Low,
    /// Medium risk
    Medium,
    /// High risk
    High,
}

/// Preview of refactoring changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringPreview {
    /// Changes to be made
    pub changes: Vec<FileChange>,
    /// Impact analysis
    pub impact: ImpactAnalysis,
    /// Estimated time in seconds
    pub estimated_time_seconds: u32,
}

/// Backup information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    /// Backup ID
    pub id: String,
    /// Timestamp of backup
    pub timestamp: String,
    /// Backed up files (path -> content)
    pub files: HashMap<PathBuf, String>,
}

impl std::str::FromStr for RefactoringType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "rename" => Ok(RefactoringType::Rename),
            "extract" => Ok(RefactoringType::Extract),
            "inline" => Ok(RefactoringType::Inline),
            "move" => Ok(RefactoringType::Move),
            "change_signature" => Ok(RefactoringType::ChangeSignature),
            "remove_unused" => Ok(RefactoringType::RemoveUnused),
            "simplify" => Ok(RefactoringType::Simplify),
            _ => Err(format!("Unknown refactoring type: {}", s)),
        }
    }
}
