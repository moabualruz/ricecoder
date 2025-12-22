//! Core data models for code generation

use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

/// Represents a code template with placeholders
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    /// Unique identifier for the template
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Programming language
    pub language: String,
    /// Template content with placeholders
    pub content: String,
    /// List of placeholders in the template
    pub placeholders: Vec<Placeholder>,
    /// Template metadata
    pub metadata: TemplateMetadata,
}

/// Metadata about a template
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TemplateMetadata {
    /// Template description
    pub description: Option<String>,
    /// Template version
    pub version: Option<String>,
    /// Template author
    pub author: Option<String>,
}

/// Represents a placeholder in a template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Placeholder {
    /// Placeholder name
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Default value if not provided
    pub default: Option<String>,
    /// Whether this placeholder is required
    pub required: bool,
}

/// Case transformation options for placeholders
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaseTransform {
    /// PascalCase (e.g., MyProject)
    PascalCase,
    /// camelCase (e.g., myProject)
    CamelCase,
    /// snake_case (e.g., my_project)
    SnakeCase,
    /// kebab-case (e.g., my-project)
    KebabCase,
    /// UPPERCASE (e.g., MY_PROJECT)
    UpperCase,
    /// lowercase (e.g., myproject)
    LowerCase,
}

/// Context for template rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateContext {
    /// Variable values for substitution
    pub values: HashMap<String, String>,
    /// Rendering options
    pub options: RenderOptions,
}

/// Options for template rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderOptions {
    /// Whether to format output
    pub format_output: bool,
    /// Whether to validate syntax
    pub validate_syntax: bool,
    /// Whether to preserve whitespace
    pub preserve_whitespace: bool,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            format_output: true,
            validate_syntax: true,
            preserve_whitespace: false,
        }
    }
}

/// Result of template rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderResult {
    /// Rendered content
    pub content: String,
    /// Warnings during rendering
    pub warnings: Vec<String>,
    /// Placeholders that were used
    pub placeholders_used: Vec<String>,
}

/// Represents a boilerplate project scaffold
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Boilerplate {
    /// Unique identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description
    pub description: String,
    /// Programming language
    pub language: String,
    /// Files in the boilerplate
    pub files: Vec<BoilerplateFile>,
    /// Dependencies to install
    pub dependencies: Vec<Dependency>,
    /// Setup scripts to run
    pub scripts: Vec<Script>,
}

/// A file in a boilerplate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoilerplateFile {
    /// File path relative to project root
    pub path: String,
    /// Template content or file reference
    pub template: String,
    /// Optional condition for including this file
    pub condition: Option<String>,
}

/// A dependency for a boilerplate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    /// Dependency name
    pub name: String,
    /// Dependency version
    pub version: String,
}

/// A setup script for a boilerplate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Script {
    /// Script name
    pub name: String,
    /// Script command
    pub command: String,
}

/// Metadata about a boilerplate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoilerplateMetadata {
    /// Unique identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description
    pub description: String,
    /// Programming language
    pub language: String,
    /// Source location of the boilerplate
    pub source: BoilerplateSource,
}

/// Source location of a boilerplate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BoilerplateSource {
    /// Global boilerplate location
    Global(PathBuf),
    /// Project-specific boilerplate location
    Project(PathBuf),
}

/// Conflict resolution strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictResolution {
    /// Skip the conflicting file
    Skip,
    /// Overwrite the existing file
    Overwrite,
    /// Merge with existing file
    Merge,
}

/// Result of boilerplate discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoilerplateDiscoveryResult {
    /// Found boilerplates
    pub boilerplates: Vec<BoilerplateMetadata>,
    /// Paths that were searched
    pub search_paths: Vec<PathBuf>,
}

/// A generated code file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedFile {
    /// File path relative to project root
    pub path: String,
    /// File content
    pub content: String,
    /// Programming language
    pub language: String,
}

/// Result of code validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether validation passed
    pub valid: bool,
    /// Validation errors found
    pub errors: Vec<ValidationError>,
    /// Validation warnings found
    pub warnings: Vec<ValidationWarning>,
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

/// A validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// File path where error occurred
    pub file: String,
    /// Line number where error occurred
    pub line: usize,
    /// Column number where error occurred
    pub column: usize,
    /// Error message
    pub message: String,
    /// Error code (e.g., "E0001")
    pub code: Option<String>,
}

/// A validation warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    /// File path where warning occurred
    pub file: String,
    /// Line number where warning occurred
    pub line: usize,
    /// Column number where warning occurred
    pub column: usize,
    /// Warning message
    pub message: String,
    /// Warning code (e.g., "W0001")
    pub code: Option<String>,
}

/// Configuration for code validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// Whether to check syntax
    pub check_syntax: bool,
    /// Whether to run linters
    pub run_linters: bool,
    /// Whether to run type checking
    pub run_type_checking: bool,
    /// Whether to treat warnings as errors
    pub warnings_as_errors: bool,
}
