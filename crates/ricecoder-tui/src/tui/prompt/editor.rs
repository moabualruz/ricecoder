//! External editor integration for prompt
//!
//! Opens the user's preferred editor for composing longer prompts.
//! Supports $EDITOR, $VISUAL, and falls back to common editors.
//!
//! # DDD Layer: Infrastructure
//! External editor integration for the prompt system.

use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;
use tempfile::NamedTempFile;

/// Editor configuration
#[derive(Debug, Clone)]
pub struct EditorConfig {
    /// Custom editor command (overrides environment)
    pub editor: Option<String>,
    /// File extension for temp file
    pub extension: String,
    /// Initial content
    pub initial_content: String,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            editor: None,
            extension: "md".to_string(),
            initial_content: String::new(),
        }
    }
}

impl EditorConfig {
    /// Create with initial content
    pub fn with_content(content: impl Into<String>) -> Self {
        Self {
            initial_content: content.into(),
            ..Default::default()
        }
    }
}

/// Result of editor operation
#[derive(Debug)]
pub enum EditorResult {
    /// Content was modified
    Modified(String),
    /// Editor was cancelled (content unchanged or empty)
    Cancelled,
    /// Error occurred
    Error(EditorError),
}

/// Editor errors
#[derive(Debug, thiserror::Error)]
pub enum EditorError {
    #[error("No editor found. Set $EDITOR or $VISUAL environment variable.")]
    NoEditor,
    
    #[error("Failed to create temp file: {0}")]
    TempFile(#[from] io::Error),
    
    #[error("Editor command failed: {0}")]
    CommandFailed(String),
    
    #[error("Failed to read edited content: {0}")]
    ReadFailed(io::Error),
}

/// External editor handler
pub struct ExternalEditor;

impl ExternalEditor {
    /// Find the user's preferred editor
    pub fn find_editor() -> Option<String> {
        // Check environment variables in order of preference
        for var in ["VISUAL", "EDITOR"] {
            if let Ok(editor) = env::var(var) {
                if !editor.is_empty() {
                    return Some(editor);
                }
            }
        }
        
        // Fall back to common editors
        let fallbacks = if cfg!(windows) {
            vec!["code", "notepad", "notepad++"]
        } else {
            vec!["nvim", "vim", "vi", "nano", "emacs", "code"]
        };
        
        for editor in fallbacks {
            if Self::editor_exists(editor) {
                return Some(editor.to_string());
            }
        }
        
        None
    }
    
    /// Check if an editor command exists
    fn editor_exists(editor: &str) -> bool {
        let cmd = if cfg!(windows) { "where" } else { "which" };
        Command::new(cmd)
            .arg(editor)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
    
    /// Open editor with content
    pub fn open(config: &EditorConfig) -> EditorResult {
        let editor = config.editor.clone()
            .or_else(Self::find_editor)
            .ok_or(EditorError::NoEditor);
        
        let editor = match editor {
            Ok(e) => e,
            Err(e) => return EditorResult::Error(e),
        };
        
        // Create temp file with extension
        let temp_file = match Self::create_temp_file(&config.extension, &config.initial_content) {
            Ok(f) => f,
            Err(e) => return EditorResult::Error(e),
        };
        
        let temp_path = temp_file.path().to_path_buf();
        
        // Keep the temp file open to prevent deletion on Windows
        let _guard = temp_file;
        
        // Open editor
        match Self::run_editor(&editor, &temp_path) {
            Ok(()) => {}
            Err(e) => return EditorResult::Error(e),
        }
        
        // Read result
        match fs::read_to_string(&temp_path) {
            Ok(content) => {
                let trimmed = content.trim().to_string();
                if trimmed.is_empty() || trimmed == config.initial_content.trim() {
                    EditorResult::Cancelled
                } else {
                    EditorResult::Modified(trimmed)
                }
            }
            Err(e) => EditorResult::Error(EditorError::ReadFailed(e)),
        }
    }
    
    /// Create temp file with content
    fn create_temp_file(extension: &str, content: &str) -> Result<NamedTempFile, EditorError> {
        let suffix = format!(".{}", extension);
        let mut file = tempfile::Builder::new()
            .prefix("ricecoder-prompt-")
            .suffix(&suffix)
            .tempfile()?;
        
        file.write_all(content.as_bytes())?;
        file.flush()?;
        
        Ok(file)
    }
    
    /// Run the editor command
    fn run_editor(editor: &str, path: &PathBuf) -> Result<(), EditorError> {
        // Parse editor command (may include arguments like "code --wait")
        let parts: Vec<&str> = editor.split_whitespace().collect();
        let (cmd, args) = parts.split_first()
            .map(|(c, a)| (*c, a.to_vec()))
            .unwrap_or((editor, vec![]));
        
        let mut command = Command::new(cmd);
        for arg in &args {
            command.arg(arg);
        }
        command.arg(path);
        
        // For VS Code, add --wait flag if not present
        if cmd.contains("code") && !args.iter().any(|a| a.contains("wait")) {
            command.arg("--wait");
        }
        
        let status = command
            .status()
            .map_err(|e| EditorError::CommandFailed(e.to_string()))?;
        
        if status.success() {
            Ok(())
        } else {
            Err(EditorError::CommandFailed(format!(
                "Editor exited with status: {}",
                status
            )))
        }
    }
    
    /// Open editor for a new prompt (empty)
    pub fn open_new() -> EditorResult {
        Self::open(&EditorConfig::default())
    }
    
    /// Open editor to edit existing prompt
    pub fn open_edit(content: &str) -> EditorResult {
        Self::open(&EditorConfig::with_content(content))
    }
}

/// Quick function to get editor name for display
pub fn editor_name() -> Option<String> {
    ExternalEditor::find_editor().map(|e| {
        // Extract just the command name
        e.split_whitespace()
            .next()
            .map(|s| {
                PathBuf::from(s)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| s.to_string())
            })
            .unwrap_or(e)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_editor_config_default() {
        let config = EditorConfig::default();
        assert!(config.editor.is_none());
        assert_eq!(config.extension, "md");
        assert!(config.initial_content.is_empty());
    }
    
    #[test]
    fn test_editor_config_with_content() {
        let config = EditorConfig::with_content("hello world");
        assert_eq!(config.initial_content, "hello world");
    }
    
    #[test]
    fn test_find_editor() {
        // This test is environment-dependent
        // Just verify it doesn't panic
        let _editor = ExternalEditor::find_editor();
    }
    
    #[test]
    fn test_editor_name() {
        let name = editor_name();
        // If an editor is found, it should be a simple name
        if let Some(name) = name {
            assert!(!name.contains('/'));
            assert!(!name.contains('\\'));
        }
    }
    
    #[test]
    fn test_create_temp_file() {
        let content = "test content";
        let file = ExternalEditor::create_temp_file("md", content).unwrap();
        
        let path = file.path();
        assert!(path.exists());
        assert!(path.to_string_lossy().ends_with(".md"));
        
        let read_content = fs::read_to_string(path).unwrap();
        assert_eq!(read_content, content);
    }
}
