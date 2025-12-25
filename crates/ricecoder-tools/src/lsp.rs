// projects/ricecoder/crates/ricecoder-tools/src/lsp.rs
// WHAT: OpenCode-compatible LSP tool with 9 operations + Helix patterns
// WHY: Faithful translation of OpenCode LSP capabilities
// HOW: Single tool with operation enum, external LSP routing, internal fallback

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// LSP tool error types (OpenCode + Helix patterns)
#[derive(Error, Debug)]
pub enum LspError {
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    #[error("No LSP server available for this file type.")]
    NoServerAvailable,

    #[error("Operation {operation} failed: {message}")]
    OperationFailed {
        operation: String,
        message: String,
    },

    #[error("Request timeout after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    #[error("Invalid position: line {line}, character {character}")]
    InvalidPosition { line: u32, character: u32 },
}

/// LSP operations (OpenCode - 9 operations)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LspOperation {
    /// Jump to definition (textDocument/definition)
    GoToDefinition,

    /// Find all references (textDocument/references)
    FindReferences,

    /// Get hover information (textDocument/hover)
    Hover,

    /// Get document symbols (textDocument/documentSymbol)
    DocumentSymbol,

    /// Search workspace symbols (workspace/symbol)
    WorkspaceSymbol,

    /// Jump to implementation (textDocument/implementation)
    GoToImplementation,

    /// Prepare call hierarchy (textDocument/prepareCallHierarchy)
    PrepareCallHierarchy,

    /// Get incoming calls (callHierarchy/incomingCalls)
    IncomingCalls,

    /// Get outgoing calls (callHierarchy/outgoingCalls)
    OutgoingCalls,
}

impl LspOperation {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::GoToDefinition => "goToDefinition",
            Self::FindReferences => "findReferences",
            Self::Hover => "hover",
            Self::DocumentSymbol => "documentSymbol",
            Self::WorkspaceSymbol => "workspaceSymbol",
            Self::GoToImplementation => "goToImplementation",
            Self::PrepareCallHierarchy => "prepareCallHierarchy",
            Self::IncomingCalls => "incomingCalls",
            Self::OutgoingCalls => "outgoingCalls",
        }
    }
}

/// LSP position (1-based for editor, 0-based for protocol)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LspPosition {
    /// Line number (1-based, as shown in editors)
    pub line: u32,

    /// Character offset (1-based, as shown in editors)
    pub character: u32,
}

impl LspPosition {
    /// Convert to 0-based LSP position (OpenCode behavior)
    pub fn to_lsp(&self) -> (u32, u32) {
        (
            self.line.saturating_sub(1),
            self.character.saturating_sub(1),
        )
    }

    /// Validate and clamp position to line bounds (Helix pattern)
    pub fn clamp_to_line(&self, line_length: u32) -> LspPosition {
        LspPosition {
            line: self.line,
            character: self.character.min(line_length),
        }
    }
}

/// LSP tool input (OpenCode-compatible parameters)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspToolInput {
    /// The LSP operation to perform
    pub operation: LspOperation,

    /// The absolute or relative path to the file
    pub file_path: PathBuf,

    /// The line number (1-based, as shown in editors)
    pub line: u32,

    /// The character offset (1-based, as shown in editors)
    pub character: u32,

    /// Optional timeout in milliseconds (Helix pattern)
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
}

fn default_timeout() -> u64 {
    5000 // 5 seconds default timeout
}

/// LSP tool output (OpenCode-compatible format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspToolOutput {
    /// Title: "{operation} {relPath}:{line}:{character}"
    pub title: String,

    /// Metadata with results
    pub metadata: LspMetadata,

    /// Human-readable output (JSON or "No results found …")
    pub output: String,
}

/// LSP metadata (OpenCode format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspMetadata {
    /// Raw LSP results (array of locations, symbols, etc.)
    pub result: serde_json::Value,
}

/// LSP tool implementation (OpenCode + Helix + RiceCoder)
pub struct LspTool {
    /// Working directory for relative path resolution
    pub working_directory: PathBuf,

    /// External LSP client (optional)
    #[allow(dead_code)]
    external_lsp_client: Option<Box<dyn ExternalLspClient>>,
}

/// External LSP client trait (for dependency injection)
pub trait ExternalLspClient: Send + Sync {
    /// Check if LSP server is available for file
    fn has_server(&self, file_path: &Path) -> bool;

    /// Execute LSP operation
    fn execute(
        &self,
        operation: LspOperation,
        file_path: &Path,
        line: u32,
        character: u32,
        timeout_ms: u64,
    ) -> Result<serde_json::Value, LspError>;
}

impl LspTool {
    /// Create new LSP tool
    pub fn new(working_directory: PathBuf) -> Self {
        Self {
            working_directory,
            external_lsp_client: None,
        }
    }

    /// Create LSP tool with external LSP client
    pub fn with_external_lsp(
        working_directory: PathBuf,
        client: Box<dyn ExternalLspClient>,
    ) -> Self {
        Self {
            working_directory,
            external_lsp_client: Some(client),
        }
    }

    /// Execute LSP tool (OpenCode behavior)
    pub fn execute(&self, input: LspToolInput) -> Result<LspToolOutput, LspError> {
        // Resolve file path (OpenCode: absolute or join with working directory)
        let file_path = if input.file_path.is_absolute() {
            input.file_path.clone()
        } else {
            self.working_directory.join(&input.file_path)
        };

        // Check if file exists (OpenCode: throws "File not found: …")
        if !file_path.exists() {
            return Err(LspError::FileNotFound(file_path));
        }

        // Check if LSP server is available (OpenCode: throws "No LSP server available for this file type.")
        let has_server = self
            .external_lsp_client
            .as_ref()
            .map(|client| client.has_server(&file_path))
            .unwrap_or(false);

        if !has_server {
            return Err(LspError::NoServerAvailable);
        }

        // Convert position to LSP (1-based → 0-based)
        let position = LspPosition {
            line: input.line,
            character: input.character,
        };
        let (lsp_line, lsp_character) = position.to_lsp();

        // Execute operation via external LSP
        let result = self
            .external_lsp_client
            .as_ref()
            .ok_or(LspError::NoServerAvailable)?
            .execute(
                input.operation,
                &file_path,
                lsp_line,
                lsp_character,
                input.timeout_ms,
            )?;

        // Build output (OpenCode format)
        let rel_path = file_path
            .strip_prefix(&self.working_directory)
            .unwrap_or(&file_path)
            .display()
            .to_string();

        let title = format!(
            "{} {}:{}:{}",
            input.operation.as_str(),
            rel_path,
            input.line,
            input.character
        );

        let output = if result.is_null() || (result.is_array() && result.as_array().unwrap().is_empty()) {
            format!("No results found for {}", input.operation.as_str())
        } else {
            serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string())
        };

        Ok(LspToolOutput {
            title,
            metadata: LspMetadata { result },
            output,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lsp_position_conversion() {
        let pos = LspPosition {
            line: 10,
            character: 5,
        };
        let (line, char) = pos.to_lsp();
        assert_eq!(line, 9); // 1-based → 0-based
        assert_eq!(char, 4); // 1-based → 0-based
    }

    #[test]
    fn test_lsp_position_clamp() {
        let pos = LspPosition {
            line: 10,
            character: 100,
        };
        let clamped = pos.clamp_to_line(50);
        assert_eq!(clamped.character, 50);
    }

    #[test]
    fn test_lsp_operation_display() {
        assert_eq!(LspOperation::GoToDefinition.as_str(), "goToDefinition");
        assert_eq!(LspOperation::FindReferences.as_str(), "findReferences");
        assert_eq!(LspOperation::Hover.as_str(), "hover");
    }

    #[test]
    fn test_file_not_found_error() {
        let tool = LspTool::new(PathBuf::from("/tmp"));
        let input = LspToolInput {
            operation: LspOperation::GoToDefinition,
            file_path: PathBuf::from("/nonexistent/file.rs"),
            line: 1,
            character: 1,
            timeout_ms: 5000,
        };

        let result = tool.execute(input);
        assert!(matches!(result, Err(LspError::FileNotFound(_))));
    }

    #[test]
    fn test_no_server_available_error() {
        let tool = LspTool::new(std::env::current_dir().unwrap());
        let input = LspToolInput {
            operation: LspOperation::GoToDefinition,
            file_path: PathBuf::from("Cargo.toml"), // Exists but no LSP configured
            line: 1,
            character: 1,
            timeout_ms: 5000,
        };

        let result = tool.execute(input);
        assert!(matches!(result, Err(LspError::NoServerAvailable)));
    }
}
