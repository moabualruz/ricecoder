//! List Directory Tool
//!
//! Lists directory contents with filtering and ignore pattern support.
//! Matches OpenCode's list tool behavior.

use async_trait::async_trait;
use ::glob::Pattern;
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

use crate::context::ToolContext;
use crate::descriptions::get_description;
use crate::error::ToolError;
use crate::tool::{ParameterSchema, Tool, ToolDefinition, ToolExecutionResult, ToolParameters};

/// Maximum number of entries before truncation
const MAX_ENTRIES: usize = 100;

/// Default patterns to ignore
const DEFAULT_IGNORE_PATTERNS: &[&str] = &[
    "node_modules/",
    "__pycache__/",
    ".git/",
    "dist/",
    "build/",
    "target/",
    "vendor/",
    "bin/",
    "obj/",
    ".idea/",
    ".vscode/",
    ".zig-cache/",
    "zig-out/",
    ".coverage/",
    "coverage/",
    "tmp/",
    "temp/",
    ".cache/",
    "cache/",
    "logs/",
    ".venv/",
    "venv/",
    "env/",
];

/// List tool input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListInput {
    /// Directory path to list (defaults to workspace root)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// Additional glob patterns to ignore
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore: Option<Vec<String>>,
}

/// List tool output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListOutput {
    /// Tree-structured directory listing
    pub output: String,

    /// Number of entries found
    pub count: usize,

    /// Whether results were truncated
    pub truncated: bool,

    /// Directory path listed
    pub path: String,
}

/// Directory entry for tree building
#[derive(Debug, Default)]
struct DirEntry {
    files: Vec<String>,
    dirs: BTreeMap<String, DirEntry>,
}

impl DirEntry {
    fn render(&self, prefix: &str) -> String {
        let mut output = String::new();
        
        // Render directories first (sorted)
        for (name, entry) in &self.dirs {
            output.push_str(&format!("{}{}/\n", prefix, name));
            output.push_str(&entry.render(&format!("{}  ", prefix)));
        }
        
        // Render files (sorted)
        let mut files: Vec<_> = self.files.iter().collect();
        files.sort();
        for file in files {
            output.push_str(&format!("{}{}\n", prefix, file));
        }
        
        output
    }
}

/// List directory contents tool
pub struct ListTool {
    /// Default workspace root
    workspace_root: PathBuf,
}

impl ListTool {
    /// Create a new ListTool with a workspace root
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }

    /// Create a new ListTool using current directory as workspace
    pub fn with_current_dir() -> Result<Self, ToolError> {
        let workspace_root = std::env::current_dir().map_err(|e| {
            ToolError::new(
                "INIT_ERROR",
                format!("Failed to get current directory: {}", e),
            )
        })?;
        Ok(Self { workspace_root })
    }

    /// List directory contents
    pub async fn list_directory(
        &self,
        input: &ListInput,
        _ctx: &ToolContext,
    ) -> Result<ListOutput, ToolError> {
        // Determine target directory
        let target_dir = if let Some(ref path) = input.path {
            let path = PathBuf::from(path);
            if path.is_absolute() {
                path
            } else {
                self.workspace_root.join(path)
            }
        } else {
            self.workspace_root.clone()
        };

        // Verify directory exists
        if !target_dir.exists() {
            return Err(ToolError::new(
                "NOT_FOUND",
                format!("Directory does not exist: {}", target_dir.display()),
            ));
        }

        if !target_dir.is_dir() {
            return Err(ToolError::new(
                "NOT_A_DIRECTORY",
                format!("Path is not a directory: {}", target_dir.display()),
            ));
        }

        // Build walker
        let mut builder = WalkBuilder::new(&target_dir);
        builder.hidden(false); // Show hidden files
        builder.git_ignore(true); // Respect .gitignore
        builder.git_global(true);
        builder.git_exclude(true);

        // Compile default ignore patterns
        let default_patterns: Vec<Pattern> = DEFAULT_IGNORE_PATTERNS
            .iter()
            .filter_map(|p| Pattern::new(p).ok())
            .collect();

        // Collect entries
        let mut root = DirEntry::default();
        let mut count = 0;
        let mut truncated = false;

        for entry in builder.build().flatten() {
            let path = entry.path();
            
            // Skip the root directory itself
            if path == target_dir {
                continue;
            }

            // Get relative path
            let rel_path = path
                .strip_prefix(&target_dir)
                .unwrap_or(path);
            let rel_path_str = rel_path.to_string_lossy();

            // Check default ignore patterns
            if default_patterns.iter().any(|pat| pat.matches(&rel_path_str)) {
                continue;
            }

            // Check user ignore patterns
            if let Some(ref patterns) = input.ignore {
                if patterns.iter().any(|p| {
                    Pattern::new(p)
                        .map(|pat| pat.matches(&rel_path_str))
                        .unwrap_or(false)
                }) {
                    continue;
                }
            }

            // Build tree structure
            let components: Vec<_> = rel_path.components().collect();
            let mut current = &mut root;

            for (i, component) in components.iter().enumerate() {
                let name = component.as_os_str().to_string_lossy().to_string();
                
                if i == components.len() - 1 {
                    // Last component - file or directory
                    if path.is_dir() {
                        current.dirs.entry(name).or_default();
                    } else {
                        current.files.push(name);
                    }
                } else {
                    // Intermediate directory
                    current = current.dirs.entry(name).or_default();
                }
            }

            count += 1;
            if count >= MAX_ENTRIES {
                truncated = true;
                break;
            }
        }

        // Render tree
        let mut output = format!("{}/\n", target_dir.display());
        output.push_str(&root.render("  "));

        if truncated {
            output.push_str("\n(Results truncated. Consider using a more specific path.)\n");
        }

        Ok(ListOutput {
            output,
            count,
            truncated,
            path: target_dir.display().to_string(),
        })
    }
}

impl Default for ListTool {
    fn default() -> Self {
        Self::with_current_dir().unwrap_or_else(|_| Self {
            workspace_root: PathBuf::from("."),
        })
    }
}

#[async_trait]
impl Tool for ListTool {
    fn id(&self) -> &str {
        "list"
    }

    async fn init(&self, _ctx: Option<&ToolContext>) -> Result<ToolDefinition, ToolError> {
        let mut parameters = ToolParameters::new();

        parameters.insert(
            "path".to_string(),
            ParameterSchema {
                type_: "string".to_string(),
                description: "The absolute path to the directory to list (must be absolute, not relative). Omit to use current workspace.".to_string(),
                required: false,
                default: None,
                properties: None,
                items: None,
            },
        );

        parameters.insert(
            "ignore".to_string(),
            ParameterSchema {
                type_: "array".to_string(),
                description: "List of glob patterns to ignore".to_string(),
                required: false,
                default: None,
                properties: None,
                items: Some(Box::new(ParameterSchema {
                    type_: "string".to_string(),
                    description: "Glob pattern".to_string(),
                    required: false,
                    default: None,
                    properties: None,
                    items: None,
                })),
            },
        );

        let description = get_description(
            "list",
            "List files and directories in a given path. The path parameter must be absolute; omit it to use the current workspace directory. You can optionally provide an array of glob patterns to ignore.",
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
        let path = args.get("path").and_then(|v| v.as_str()).map(String::from);
        
        let ignore = args.get("ignore").and_then(|v| {
            v.as_array().map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
        });

        let input = ListInput { path, ignore };

        // Execute
        let result = self.list_directory(&input, ctx).await?;

        // Build metadata
        let mut metadata = HashMap::new();
        metadata.insert("count".to_string(), Value::Number(result.count.into()));
        metadata.insert("truncated".to_string(), Value::Bool(result.truncated));
        metadata.insert("path".to_string(), Value::String(result.path.clone()));

        // Build title
        let title = format!("Listed {} entries", result.count);

        Ok(ToolExecutionResult {
            title,
            metadata,
            output: result.output,
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
        
        // Create some files and directories
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
    async fn test_list_tool_init() {
        let tool = ListTool::default();
        let def = tool.init(None).await.unwrap();

        assert!(def.description.to_lowercase().contains("list"));
        assert!(def.parameters.contains_key("path"));
        assert!(def.parameters.contains_key("ignore"));
    }

    #[tokio::test]
    async fn test_list_directory() {
        let dir = setup_test_dir().await;
        let tool = ListTool::new(dir.path().to_path_buf());
        let ctx = ToolContext::default();

        let input = ListInput {
            path: None,
            ignore: None,
        };

        let result = tool.list_directory(&input, &ctx).await.unwrap();
        
        assert!(result.output.contains("src/"));
        assert!(result.output.contains("tests/"));
        assert!(result.output.contains("Cargo.toml"));
        assert!(result.output.contains("README.md"));
        assert!(!result.truncated);
    }

    #[tokio::test]
    async fn test_list_with_ignore() {
        let dir = setup_test_dir().await;
        let tool = ListTool::new(dir.path().to_path_buf());
        let ctx = ToolContext::default();

        let input = ListInput {
            path: None,
            ignore: Some(vec!["*.md".to_string()]),
        };

        let result = tool.list_directory(&input, &ctx).await.unwrap();
        
        assert!(!result.output.contains("README.md"));
        assert!(result.output.contains("Cargo.toml"));
    }

    #[tokio::test]
    async fn test_list_nonexistent_directory() {
        let tool = ListTool::default();
        let ctx = ToolContext::default();

        let input = ListInput {
            path: Some("/nonexistent/path/12345".to_string()),
            ignore: None,
        };

        let result = tool.list_directory(&input, &ctx).await;
        assert!(result.is_err());
    }
}
