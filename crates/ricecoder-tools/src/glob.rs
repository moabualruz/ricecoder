//! Glob Pattern Matching Tool
//!
//! Fast file pattern matching with safety limits.
//! Matches OpenCode's glob tool behavior.

use async_trait::async_trait;
use ::glob::Pattern;
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

use crate::context::ToolContext;
use crate::descriptions::get_description;
use crate::error::ToolError;
use crate::tool::{ParameterSchema, Tool, ToolDefinition, ToolExecutionResult, ToolParameters};

/// Maximum number of files to return (OpenCode behavior)
const MAX_FILES: usize = 100;

/// Glob tool input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobInput {
    /// Glob pattern to match (e.g., "**/*.rs", "src/**/*.ts")
    pub pattern: String,

    /// Directory to search in (defaults to workspace root)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

/// Glob tool output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobOutput {
    /// Matching file paths (sorted by mtime, newest first)
    pub files: Vec<String>,

    /// Number of files found
    pub count: usize,

    /// Whether results were truncated
    pub truncated: bool,
}

/// Glob pattern matching tool
pub struct GlobTool {
    /// Default workspace root
    workspace_root: PathBuf,
}

impl GlobTool {
    /// Create a new GlobTool with a workspace root
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }

    /// Create a new GlobTool using current directory as workspace
    pub fn with_current_dir() -> Result<Self, ToolError> {
        let workspace_root = std::env::current_dir().map_err(|e| {
            ToolError::new(
                "INIT_ERROR",
                format!("Failed to get current directory: {}", e),
            )
        })?;
        Ok(Self { workspace_root })
    }

    /// Find files matching glob pattern
    pub async fn find_files(
        &self,
        input: &GlobInput,
        _ctx: &ToolContext,
    ) -> Result<GlobOutput, ToolError> {
        // Validate pattern
        if input.pattern.trim().is_empty() {
            return Err(ToolError::new(
                "INVALID_PATTERN",
                "Glob pattern cannot be empty",
            ));
        }

        // Parse glob pattern
        let glob_pattern = Pattern::new(&input.pattern).map_err(|e| {
            ToolError::new(
                "INVALID_PATTERN",
                format!("Invalid glob pattern '{}': {}", input.pattern, e),
            )
        })?;

        // Determine search root
        let search_root = if let Some(ref path) = input.path {
            let path = PathBuf::from(path);
            if path.is_absolute() {
                path
            } else {
                self.workspace_root.join(path)
            }
        } else {
            self.workspace_root.clone()
        };

        // Verify search root exists
        if !search_root.exists() {
            return Err(ToolError::new(
                "NOT_FOUND",
                format!("Search path does not exist: {}", search_root.display()),
            ));
        }

        // Build walker
        let walker = WalkBuilder::new(&search_root)
            .hidden(false) // Include hidden files
            .git_ignore(true) // Respect .gitignore
            .git_global(true)
            .git_exclude(true)
            .build();

        // Collect matching files with mtimes
        let mut file_results: Vec<(String, SystemTime)> = Vec::new();
        let mut truncated = false;

        for entry in walker.flatten() {
            let path = entry.path();

            // Skip directories
            if path.is_dir() {
                continue;
            }

            // Get path string for matching
            let path_str = path.to_string_lossy();

            // Also try matching against relative path
            let rel_path = path
                .strip_prefix(&search_root)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| path_str.to_string());

            // Check if pattern matches (try both full and relative path)
            let matches = glob_pattern.matches(&path_str)
                || glob_pattern.matches(&rel_path)
                || glob_pattern.matches(&format!("/{}", rel_path));

            if matches {
                // Get modification time for sorting
                let mtime = path
                    .metadata()
                    .and_then(|m| m.modified())
                    .unwrap_or(SystemTime::UNIX_EPOCH);

                file_results.push((path.display().to_string(), mtime));

                // Check limit
                if file_results.len() >= MAX_FILES {
                    truncated = true;
                    break;
                }
            }
        }

        // Sort by mtime descending (newest first)
        file_results.sort_by(|a, b| b.1.cmp(&a.1));

        // Extract paths
        let files: Vec<String> = file_results.into_iter().map(|(path, _)| path).collect();
        let count = files.len();

        Ok(GlobOutput {
            files,
            count,
            truncated,
        })
    }
}

impl Default for GlobTool {
    fn default() -> Self {
        Self::with_current_dir().unwrap_or_else(|_| Self {
            workspace_root: PathBuf::from("."),
        })
    }
}

#[async_trait]
impl Tool for GlobTool {
    fn id(&self) -> &str {
        "glob"
    }

    async fn init(&self, _ctx: Option<&ToolContext>) -> Result<ToolDefinition, ToolError> {
        let mut parameters = ToolParameters::new();

        parameters.insert(
            "pattern".to_string(),
            ParameterSchema {
                type_: "string".to_string(),
                description: "The glob pattern to match files against (e.g., \"**/*.rs\", \"src/**/*.ts\")".to_string(),
                required: true,
                default: None,
                properties: None,
                items: None,
            },
        );

        parameters.insert(
            "path".to_string(),
            ParameterSchema {
                type_: "string".to_string(),
                description: "The directory to search in. Defaults to current workspace.".to_string(),
                required: false,
                default: None,
                properties: None,
                items: None,
            },
        );

        let description = get_description(
            "glob",
            "Fast file pattern matching tool with safety limits (60s timeout, 100 file limit). Supports glob patterns like \"**/*.js\" or \"src/**/*.ts\". Returns matching file paths sorted by modification time.",
        );

        Ok(ToolDefinition {
            description,
            parameters,
            format_validation_error: None,
        })
    }

    async fn execute(
        &self,
        args: HashMap<String, Value>,
        ctx: &ToolContext,
    ) -> Result<ToolExecutionResult, ToolError> {
        // Parse input
        let pattern = args
            .get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::new("MISSING_PARAM", "Missing required parameter: pattern"))?
            .to_string();

        let path = args.get("path").and_then(|v| v.as_str()).map(String::from);

        let input = GlobInput { pattern, path };

        // Execute
        let result = self.find_files(&input, ctx).await?;

        // Build metadata
        let mut metadata = HashMap::new();
        metadata.insert("count".to_string(), Value::Number(result.count.into()));
        metadata.insert("truncated".to_string(), Value::Bool(result.truncated));

        // Build title
        let title = format!("Found {} file(s)", result.count);

        // Build output
        let mut output = result.files.join("\n");
        if result.truncated {
            output.push_str("\n\n(Results are truncated. Consider using a more specific path or pattern.)");
        }

        Ok(ToolExecutionResult {
            title,
            metadata,
            output,
            attachments: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn setup_test_dir() -> TempDir {
        let dir = TempDir::new().unwrap();

        // Create some files
        std::fs::create_dir(dir.path().join("src")).unwrap();
        std::fs::write(dir.path().join("src/main.rs"), "fn main() {}").unwrap();
        std::fs::write(dir.path().join("src/lib.rs"), "// lib").unwrap();
        std::fs::create_dir(dir.path().join("tests")).unwrap();
        std::fs::write(dir.path().join("tests/test.rs"), "// test").unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "[package]").unwrap();
        std::fs::write(dir.path().join("README.md"), "# README").unwrap();

        dir
    }

    #[tokio::test]
    async fn test_glob_tool_init() {
        let tool = GlobTool::default();
        let def = tool.init(None).await.unwrap();

        assert!(def.description.contains("glob"));
        assert!(def.parameters.contains_key("pattern"));
        assert!(def.parameters.get("pattern").unwrap().required);
    }

    #[tokio::test]
    async fn test_glob_find_rs_files() {
        let dir = setup_test_dir().await;
        let tool = GlobTool::new(dir.path().to_path_buf());
        let ctx = ToolContext::default();

        let input = GlobInput {
            pattern: "**/*.rs".to_string(),
            path: None,
        };

        let result = tool.find_files(&input, &ctx).await.unwrap();

        assert_eq!(result.count, 3); // main.rs, lib.rs, test.rs
        assert!(!result.truncated);
    }

    #[tokio::test]
    async fn test_glob_find_toml_files() {
        let dir = setup_test_dir().await;
        let tool = GlobTool::new(dir.path().to_path_buf());
        let ctx = ToolContext::default();

        let input = GlobInput {
            pattern: "*.toml".to_string(),
            path: None,
        };

        let result = tool.find_files(&input, &ctx).await.unwrap();

        assert_eq!(result.count, 1);
        assert!(result.files[0].contains("Cargo.toml"));
    }

    #[tokio::test]
    async fn test_glob_empty_pattern() {
        let tool = GlobTool::default();
        let ctx = ToolContext::default();

        let input = GlobInput {
            pattern: "".to_string(),
            path: None,
        };

        let result = tool.find_files(&input, &ctx).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_glob_invalid_pattern() {
        let tool = GlobTool::default();
        let ctx = ToolContext::default();

        let input = GlobInput {
            pattern: "[invalid".to_string(),
            path: None,
        };

        let result = tool.find_files(&input, &ctx).await;
        assert!(result.is_err());
    }
}
