//! Grep Content Search Tool
//!
//! Fast content search with regex support and safety limits.
//! Matches OpenCode's grep tool behavior.

use async_trait::async_trait;
use ::glob::Pattern as GlobPattern;
use ignore::WalkBuilder;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

use crate::context::ToolContext;
use crate::error::ToolError;
use crate::tool::{ParameterSchema, Tool, ToolDefinition, ToolExecutionResult, ToolParameters};

/// Maximum number of matches to return
const MAX_MATCHES: usize = 100;

/// Maximum line length before truncation
const MAX_LINE_LENGTH: usize = 2000;

/// Maximum output size (10MB)
const MAX_OUTPUT_SIZE: usize = 10 * 1024 * 1024;

/// Grep tool input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrepInput {
    /// Regex pattern to search for
    pub pattern: String,

    /// Directory to search in (defaults to workspace root)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// File pattern to include (e.g., "*.rs", "*.{ts,tsx}")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<String>,
}

/// A single match result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrepMatch {
    /// File path
    pub file: String,

    /// Line number (1-indexed)
    pub line: usize,

    /// Line content
    pub content: String,

    /// File modification time
    pub mtime: SystemTime,
}

/// Grep tool output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrepOutput {
    /// List of matches
    pub matches: Vec<GrepMatch>,

    /// Number of matches found
    pub count: usize,

    /// Whether results were truncated
    pub truncated: bool,
}

/// Grep content search tool
pub struct GrepTool {
    /// Default workspace root
    workspace_root: PathBuf,
}

impl GrepTool {
    /// Create a new GrepTool with a workspace root
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }

    /// Create a new GrepTool using current directory as workspace
    pub fn with_current_dir() -> Result<Self, ToolError> {
        let workspace_root = std::env::current_dir().map_err(|e| {
            ToolError::new(
                "INIT_ERROR",
                format!("Failed to get current directory: {}", e),
            )
        })?;
        Ok(Self { workspace_root })
    }

    /// Search file contents for pattern
    pub async fn search(
        &self,
        input: &GrepInput,
        _ctx: &ToolContext,
    ) -> Result<GrepOutput, ToolError> {
        // Validate pattern
        if input.pattern.trim().is_empty() {
            return Err(ToolError::new(
                "INVALID_PATTERN",
                "Search pattern cannot be empty",
            ));
        }

        // Compile regex
        let regex = Regex::new(&input.pattern).map_err(|e| {
            ToolError::new(
                "INVALID_PATTERN",
                format!("Invalid regex pattern '{}': {}", input.pattern, e),
            )
        })?;

        // Parse include pattern if provided
        let include_pattern = if let Some(ref include) = input.include {
            Some(GlobPattern::new(include).map_err(|e| {
                ToolError::new(
                    "INVALID_PATTERN",
                    format!("Invalid include pattern '{}': {}", include, e),
                )
            })?)
        } else {
            None
        };

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

        // Collect matches
        let mut matches: Vec<GrepMatch> = Vec::new();
        let mut truncated = false;

        'outer: for entry in walker.flatten() {
            let path = entry.path();

            // Skip directories
            if path.is_dir() {
                continue;
            }

            // Check include pattern
            if let Some(ref pattern) = include_pattern {
                let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                if !pattern.matches(&file_name) {
                    // Also try matching against relative path
                    let rel_path = path
                        .strip_prefix(&search_root)
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|_| file_name.to_string());
                    if !pattern.matches(&rel_path) {
                        continue;
                    }
                }
            }

            // Get file modification time
            let mtime = path
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH);

            // Read file content
            let content = match std::fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue, // Skip binary/unreadable files
            };

            // Search for matches
            for (line_num, line) in content.lines().enumerate() {
                if regex.is_match(line) {
                    // Truncate long lines
                    let truncated_line = if line.len() > MAX_LINE_LENGTH {
                        format!("{}...", &line[..MAX_LINE_LENGTH])
                    } else {
                        line.to_string()
                    };

                    matches.push(GrepMatch {
                        file: path.display().to_string(),
                        line: line_num + 1, // 1-indexed
                        content: truncated_line,
                        mtime,
                    });

                    // Check limit
                    if matches.len() >= MAX_MATCHES {
                        truncated = true;
                        break 'outer;
                    }
                }
            }
        }

        // Sort by mtime descending (newest first)
        matches.sort_by(|a, b| b.mtime.cmp(&a.mtime));

        let count = matches.len();

        Ok(GrepOutput {
            matches,
            count,
            truncated,
        })
    }
}

impl Default for GrepTool {
    fn default() -> Self {
        Self::with_current_dir().unwrap_or_else(|_| Self {
            workspace_root: PathBuf::from("."),
        })
    }
}

#[async_trait]
impl Tool for GrepTool {
    fn id(&self) -> &str {
        "grep"
    }

    async fn init(&self, _ctx: Option<&ToolContext>) -> Result<ToolDefinition, ToolError> {
        let mut parameters = ToolParameters::new();

        parameters.insert(
            "pattern".to_string(),
            ParameterSchema {
                type_: "string".to_string(),
                description: "The regex pattern to search for in file contents. Supports full regex syntax (e.g., \"log.*Error\", \"function\\s+\\w+\").".to_string(),
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

        parameters.insert(
            "include".to_string(),
            ParameterSchema {
                type_: "string".to_string(),
                description: "File pattern to include in the search (e.g., \"*.js\", \"*.{ts,tsx}\").".to_string(),
                required: false,
                default: None,
                properties: None,
                items: None,
            },
        );

        Ok(ToolDefinition {
            description: "Fast content search tool with safety limits (60s timeout, 10MB output). Searches file contents using regular expressions. Supports full regex syntax. Filter files by pattern with the include parameter. Returns file paths with matches sorted by modification time.".to_string(),
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
        let include = args.get("include").and_then(|v| v.as_str()).map(String::from);

        let input = GrepInput {
            pattern: pattern.clone(),
            path,
            include,
        };

        // Execute
        let result = self.search(&input, ctx).await?;

        // Build metadata
        let mut metadata = HashMap::new();
        metadata.insert("matches".to_string(), Value::Number(result.count.into()));
        metadata.insert("truncated".to_string(), Value::Bool(result.truncated));

        // Build title
        let title = pattern;

        // Build output
        let mut output = String::new();
        let mut current_file = String::new();

        for m in &result.matches {
            if m.file != current_file {
                if !current_file.is_empty() {
                    output.push('\n');
                }
                output.push_str(&format!("{}:\n", m.file));
                current_file = m.file.clone();
            }
            output.push_str(&format!("  {}| {}\n", m.line, m.content));

            // Check output size limit
            if output.len() > MAX_OUTPUT_SIZE {
                output.push_str("\n(Output truncated due to size limit)\n");
                break;
            }
        }

        if result.count == 0 {
            output = "No matches found".to_string();
        } else if result.truncated {
            output.push_str(&format!(
                "\n(Results truncated. Showing {} of {} matches. Consider a more specific pattern or path.)",
                MAX_MATCHES, result.count
            ));
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

        // Create some files with content
        std::fs::create_dir(dir.path().join("src")).unwrap();
        std::fs::write(
            dir.path().join("src/main.rs"),
            "fn main() {\n    println!(\"Hello, world!\");\n}\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("src/lib.rs"),
            "pub fn greet(name: &str) {\n    println!(\"Hello, {}!\", name);\n}\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("README.md"),
            "# Hello\n\nThis is a test project.\n",
        )
        .unwrap();

        dir
    }

    #[tokio::test]
    async fn test_grep_tool_init() {
        let tool = GrepTool::default();
        let def = tool.init(None).await.unwrap();

        assert!(def.description.contains("search"));
        assert!(def.parameters.contains_key("pattern"));
        assert!(def.parameters.get("pattern").unwrap().required);
    }

    #[tokio::test]
    async fn test_grep_find_println() {
        let dir = setup_test_dir().await;
        let tool = GrepTool::new(dir.path().to_path_buf());
        let ctx = ToolContext::default();

        let input = GrepInput {
            pattern: "println!".to_string(),
            path: None,
            include: None,
        };

        let result = tool.search(&input, &ctx).await.unwrap();

        assert_eq!(result.count, 2); // 2 println! calls (main.rs and lib.rs)
        assert!(!result.truncated);
    }

    #[tokio::test]
    async fn test_grep_with_include_filter() {
        let dir = setup_test_dir().await;
        let tool = GrepTool::new(dir.path().to_path_buf());
        let ctx = ToolContext::default();

        let input = GrepInput {
            pattern: "Hello".to_string(),
            path: None,
            include: Some("*.md".to_string()),
        };

        let result = tool.search(&input, &ctx).await.unwrap();

        assert_eq!(result.count, 1); // Only README.md
        assert!(result.matches[0].file.contains("README.md"));
    }

    #[tokio::test]
    async fn test_grep_empty_pattern() {
        let tool = GrepTool::default();
        let ctx = ToolContext::default();

        let input = GrepInput {
            pattern: "".to_string(),
            path: None,
            include: None,
        };

        let result = tool.search(&input, &ctx).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_grep_invalid_regex() {
        let tool = GrepTool::default();
        let ctx = ToolContext::default();

        let input = GrepInput {
            pattern: "[invalid".to_string(),
            path: None,
            include: None,
        };

        let result = tool.search(&input, &ctx).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_grep_no_matches() {
        let dir = setup_test_dir().await;
        let tool = GrepTool::new(dir.path().to_path_buf());
        let ctx = ToolContext::default();

        let input = GrepInput {
            pattern: "NONEXISTENT_PATTERN_12345".to_string(),
            path: None,
            include: None,
        };

        let result = tool.search(&input, &ctx).await.unwrap();

        assert_eq!(result.count, 0);
        assert!(!result.truncated);
    }
}
