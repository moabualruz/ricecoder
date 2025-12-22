//! Code Actions Module
//!
//! This module provides code action suggestions for fixing issues and refactoring code.
//!
//! # Architecture
//!
//! The code actions module is organized into:
//! - `CodeActionsEngine`: Main trait for generating code actions
//! - `applier`: Module for applying code actions to code
//! - Language-specific action generators
//!
//! # Example
//!
//! ```ignore
//! use ricecoder_lsp::code_actions::CodeActionsEngine;
//! use ricecoder_lsp::types::{Diagnostic, Language};
//!
//! let engine = DefaultCodeActionsEngine::new();
//! let actions = engine.suggest_code_actions(&diagnostic, code)?;
//! ```

pub mod adapters;
pub mod applier;
pub mod generic_engine;

use std::{error::Error, fmt};

pub use adapters::{PythonCodeActionAdapter, RustCodeActionAdapter, TypeScriptCodeActionAdapter};
pub use generic_engine::GenericCodeActionsEngine;

use crate::types::{CodeAction, CodeActionKind, Diagnostic, TextEdit, WorkspaceEdit};

/// Error type for code actions operations
#[derive(Debug, Clone)]
pub enum CodeActionsError {
    /// Action generation failed
    GenerationFailed(String),
    /// Invalid diagnostic
    InvalidDiagnostic(String),
    /// Application failed
    ApplicationFailed(String),
}

impl fmt::Display for CodeActionsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CodeActionsError::GenerationFailed(msg) => {
                write!(f, "Action generation failed: {}", msg)
            }
            CodeActionsError::InvalidDiagnostic(msg) => write!(f, "Invalid diagnostic: {}", msg),
            CodeActionsError::ApplicationFailed(msg) => write!(f, "Application failed: {}", msg),
        }
    }
}

impl Error for CodeActionsError {}

/// Result type for code actions operations
pub type CodeActionsResult<T> = Result<T, CodeActionsError>;

/// Trait for generating code actions
pub trait CodeActionsEngine: Send + Sync {
    /// Suggest code actions for a diagnostic
    fn suggest_code_actions(
        &self,
        diagnostic: &Diagnostic,
        code: &str,
    ) -> CodeActionsResult<Vec<CodeAction>>;

    /// Apply a code action to code
    fn apply_code_action(&self, code: &str, action: &CodeAction) -> CodeActionsResult<String>;
}

/// Default code actions engine implementation
pub struct DefaultCodeActionsEngine;

impl DefaultCodeActionsEngine {
    /// Create a new code actions engine
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultCodeActionsEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeActionsEngine for DefaultCodeActionsEngine {
    fn suggest_code_actions(
        &self,
        diagnostic: &Diagnostic,
        code: &str,
    ) -> CodeActionsResult<Vec<CodeAction>> {
        let mut actions = Vec::new();

        // Generate actions based on diagnostic code
        if let Some(code_str) = &diagnostic.code {
            match code_str.as_str() {
                "unused-import" => {
                    actions.extend(suggest_remove_import_action(diagnostic, code)?);
                }
                "unreachable-code" => {
                    actions.extend(suggest_remove_unreachable_action(diagnostic, code)?);
                }
                "naming-convention" => {
                    actions.extend(suggest_rename_action(diagnostic, code)?);
                }
                "syntax-error" => {
                    actions.extend(suggest_fix_syntax_action(diagnostic, code)?);
                }
                _ => {}
            }
        }

        Ok(actions)
    }

    fn apply_code_action(&self, code: &str, action: &CodeAction) -> CodeActionsResult<String> {
        applier::apply_code_action(code, action)
    }
}

/// Suggest removing an unused import
fn suggest_remove_import_action(
    diagnostic: &Diagnostic,
    code: &str,
) -> CodeActionsResult<Vec<CodeAction>> {
    let line_num = diagnostic.range.start.line as usize;

    // Get the line to remove
    if let Some(_line) = code.lines().nth(line_num) {
        let mut edit = WorkspaceEdit::new();
        let text_edit = TextEdit {
            range: diagnostic.range,
            new_text: String::new(),
        };

        edit.add_edit("file://unknown".to_string(), text_edit);

        let action = CodeAction::new(
            "Remove unused import".to_string(),
            CodeActionKind::QuickFix,
            edit,
        );

        Ok(vec![action])
    } else {
        Ok(Vec::new())
    }
}

/// Suggest removing unreachable code
fn suggest_remove_unreachable_action(
    diagnostic: &Diagnostic,
    code: &str,
) -> CodeActionsResult<Vec<CodeAction>> {
    let line_num = diagnostic.range.start.line as usize;

    // Get the line to remove
    if let Some(_line) = code.lines().nth(line_num) {
        let mut edit = WorkspaceEdit::new();
        let text_edit = TextEdit {
            range: diagnostic.range,
            new_text: String::new(),
        };

        edit.add_edit("file://unknown".to_string(), text_edit);

        let action = CodeAction::new(
            "Remove unreachable code".to_string(),
            CodeActionKind::QuickFix,
            edit,
        );

        Ok(vec![action])
    } else {
        Ok(Vec::new())
    }
}

/// Suggest renaming to follow naming conventions
fn suggest_rename_action(
    diagnostic: &Diagnostic,
    _code: &str,
) -> CodeActionsResult<Vec<CodeAction>> {
    // Extract the current name from the diagnostic message
    if let Some(start) = diagnostic.message.find('`') {
        if let Some(end) = diagnostic.message[start + 1..].find('`') {
            let current_name = &diagnostic.message[start + 1..start + 1 + end];

            // Suggest a corrected name based on the convention
            let suggested_name = if diagnostic.message.contains("snake_case") {
                to_snake_case(current_name)
            } else if diagnostic.message.contains("PascalCase") {
                to_pascal_case(current_name)
            } else if diagnostic.message.contains("camelCase") {
                to_camel_case(current_name)
            } else {
                return Ok(Vec::new());
            };

            if suggested_name != current_name {
                let mut edit = WorkspaceEdit::new();
                let text_edit = TextEdit {
                    range: diagnostic.range,
                    new_text: suggested_name.clone(),
                };

                edit.add_edit("file://unknown".to_string(), text_edit);

                let action = CodeAction::new(
                    format!("Rename to `{}`", suggested_name),
                    CodeActionKind::QuickFix,
                    edit,
                );

                return Ok(vec![action]);
            }
        }
    }

    Ok(Vec::new())
}

/// Suggest fixing syntax errors
fn suggest_fix_syntax_action(
    diagnostic: &Diagnostic,
    _code: &str,
) -> CodeActionsResult<Vec<CodeAction>> {
    if diagnostic.message.contains("mismatched brackets") {
        // This is a complex fix that would require parsing
        // For now, we'll just suggest a generic fix
        let action = CodeAction::new(
            "Check bracket matching".to_string(),
            CodeActionKind::QuickFix,
            WorkspaceEdit::new(),
        );

        Ok(vec![action])
    } else {
        Ok(Vec::new())
    }
}

/// Convert a name to snake_case
fn to_snake_case(name: &str) -> String {
    let mut result = String::new();

    for (i, ch) in name.chars().enumerate() {
        if i > 0 && ch.is_uppercase() {
            result.push('_');
        }
        result.push(ch.to_lowercase().next().unwrap_or(ch));
    }

    result
}

/// Convert a name to PascalCase
fn to_pascal_case(name: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for ch in name.chars() {
        if ch == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(ch.to_uppercase().next().unwrap_or(ch));
            capitalize_next = false;
        } else {
            result.push(ch);
        }
    }

    result
}

/// Convert a name to camelCase
fn to_camel_case(name: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;

    for ch in name.chars() {
        if ch == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(ch.to_uppercase().next().unwrap_or(ch));
            capitalize_next = false;
        } else {
            result.push(ch.to_lowercase().next().unwrap_or(ch));
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("MyFunction"), "my_function");
        assert_eq!(to_snake_case("myFunction"), "my_function");
        assert_eq!(to_snake_case("CONSTANT"), "c_o_n_s_t_a_n_t");
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("my_function"), "MyFunction");
        assert_eq!(to_pascal_case("myFunction"), "MyFunction");
        assert_eq!(to_pascal_case("my_const"), "MyConst");
    }

    #[test]
    fn test_to_camel_case() {
        assert_eq!(to_camel_case("my_function"), "myFunction");
        assert_eq!(to_camel_case("MyFunction"), "myfunction");
        assert_eq!(to_camel_case("my_const"), "myConst");
    }

    #[test]
    fn test_code_actions_engine_creation() {
        let engine = DefaultCodeActionsEngine::new();
        let _ = engine;
    }
}
