//! IDE response formatting
//!
//! This module provides formatting utilities for IDE responses to ensure they are
//! properly formatted for IDE consumption. It handles formatting for different IDE types
//! and ensures responses are in the correct format for each IDE.

use serde_json::{json, Value};
use tracing::debug;

use crate::types::*;

/// Response formatter for IDE responses
pub struct ResponseFormatter;

impl ResponseFormatter {
    /// Format completion items for IDE consumption
    pub fn format_completions(items: &[CompletionItem]) -> Value {
        debug!("Formatting {} completion items for IDE", items.len());

        let formatted_items: Vec<Value> = items
            .iter()
            .map(|item| {
                json!({
                    "label": item.label,
                    "kind": Self::format_completion_kind(item.kind),
                    "detail": item.detail,
                    "documentation": item.documentation,
                    "insertText": item.insert_text,
                })
            })
            .collect();

        json!({
            "items": formatted_items,
            "isIncomplete": false,
        })
    }

    /// Format diagnostics for IDE display
    pub fn format_diagnostics(diagnostics: &[Diagnostic]) -> Value {
        debug!("Formatting {} diagnostics for IDE", diagnostics.len());

        let formatted_diagnostics: Vec<Value> = diagnostics
            .iter()
            .map(|diag| {
                json!({
                    "range": {
                        "start": {
                            "line": diag.range.start.line,
                            "character": diag.range.start.character,
                        },
                        "end": {
                            "line": diag.range.end.line,
                            "character": diag.range.end.character,
                        },
                    },
                    "severity": Self::format_diagnostic_severity(diag.severity),
                    "message": diag.message,
                    "source": diag.source,
                })
            })
            .collect();

        json!({
            "diagnostics": formatted_diagnostics,
        })
    }

    /// Format hover information for IDE display
    pub fn format_hover(hover: &Option<Hover>) -> Value {
        debug!("Formatting hover information for IDE");

        match hover {
            Some(h) => {
                let range = h.range.map(|r| {
                    json!({
                        "start": {
                            "line": r.start.line,
                            "character": r.start.character,
                        },
                        "end": {
                            "line": r.end.line,
                            "character": r.end.character,
                        },
                    })
                });

                json!({
                    "contents": h.contents,
                    "range": range,
                })
            }
            None => json!(null),
        }
    }

    /// Format definition location for IDE navigation
    pub fn format_definition(location: &Option<Location>) -> Value {
        debug!("Formatting definition location for IDE");

        match location {
            Some(loc) => {
                json!({
                    "uri": loc.file_path,
                    "range": {
                        "start": {
                            "line": loc.range.start.line,
                            "character": loc.range.start.character,
                        },
                        "end": {
                            "line": loc.range.end.line,
                            "character": loc.range.end.character,
                        },
                    },
                })
            }
            None => json!(null),
        }
    }

    /// Format completion item kind for IDE
    fn format_completion_kind(kind: CompletionItemKind) -> u32 {
        match kind {
            CompletionItemKind::Text => 1,
            CompletionItemKind::Method => 2,
            CompletionItemKind::Function => 3,
            CompletionItemKind::Constructor => 4,
            CompletionItemKind::Field => 5,
            CompletionItemKind::Variable => 6,
            CompletionItemKind::Class => 7,
            CompletionItemKind::Interface => 8,
            CompletionItemKind::Module => 9,
            CompletionItemKind::Property => 10,
            CompletionItemKind::Unit => 11,
            CompletionItemKind::Value => 12,
            CompletionItemKind::Enum => 13,
            CompletionItemKind::Keyword => 14,
            CompletionItemKind::Snippet => 15,
            CompletionItemKind::Color => 16,
            CompletionItemKind::File => 17,
            CompletionItemKind::Reference => 18,
            CompletionItemKind::Folder => 19,
            CompletionItemKind::EnumMember => 20,
            CompletionItemKind::Constant => 21,
            CompletionItemKind::Struct => 22,
            CompletionItemKind::Event => 23,
            CompletionItemKind::Operator => 24,
            CompletionItemKind::TypeParameter => 25,
        }
    }

    /// Format diagnostic severity for IDE
    fn format_diagnostic_severity(severity: DiagnosticSeverity) -> u32 {
        match severity {
            DiagnosticSeverity::Error => 1,
            DiagnosticSeverity::Warning => 2,
            DiagnosticSeverity::Information => 3,
            DiagnosticSeverity::Hint => 4,
        }
    }

    /// Format response for VS Code
    pub fn format_for_vscode(response_type: &str, data: Value) -> Value {
        debug!("Formatting response for VS Code: {}", response_type);

        json!({
            "jsonrpc": "2.0",
            "result": data,
        })
    }

    /// Format response for terminal editors
    pub fn format_for_terminal(response_type: &str, data: Value) -> Value {
        debug!("Formatting response for terminal editor: {}", response_type);

        json!({
            "type": response_type,
            "data": data,
        })
    }

    /// Format error response
    pub fn format_error(code: i32, message: &str) -> Value {
        debug!(
            "Formatting error response: code={}, message={}",
            code, message
        );

        json!({
            "jsonrpc": "2.0",
            "error": {
                "code": code,
                "message": message,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_completions() {
        let items = vec![
            CompletionItem {
                label: "test".to_string(),
                kind: CompletionItemKind::Function,
                detail: Some("test function".to_string()),
                documentation: None,
                insert_text: "test()".to_string(),
            },
            CompletionItem {
                label: "hello".to_string(),
                kind: CompletionItemKind::Variable,
                detail: None,
                documentation: Some("hello variable".to_string()),
                insert_text: "hello".to_string(),
            },
        ];

        let result = ResponseFormatter::format_completions(&items);
        assert!(result.get("items").is_some());
        assert_eq!(result["items"].as_array().unwrap().len(), 2);
        assert_eq!(result["items"][0]["label"], "test");
        assert_eq!(result["items"][1]["label"], "hello");
    }

    #[test]
    fn test_format_diagnostics() {
        let diagnostics = vec![Diagnostic {
            range: Range {
                start: Position {
                    line: 1,
                    character: 0,
                },
                end: Position {
                    line: 1,
                    character: 10,
                },
            },
            severity: DiagnosticSeverity::Error,
            message: "Test error".to_string(),
            source: "test".to_string(),
        }];

        let result = ResponseFormatter::format_diagnostics(&diagnostics);
        assert!(result.get("diagnostics").is_some());
        assert_eq!(result["diagnostics"].as_array().unwrap().len(), 1);
        assert_eq!(result["diagnostics"][0]["message"], "Test error");
        assert_eq!(result["diagnostics"][0]["severity"], 1); // Error
    }

    #[test]
    fn test_format_hover_with_content() {
        let hover = Some(Hover {
            contents: "test hover".to_string(),
            range: Some(Range {
                start: Position {
                    line: 1,
                    character: 0,
                },
                end: Position {
                    line: 1,
                    character: 4,
                },
            }),
        });

        let result = ResponseFormatter::format_hover(&hover);
        assert_eq!(result["contents"], "test hover");
        assert!(result.get("range").is_some());
    }

    #[test]
    fn test_format_hover_empty() {
        let hover: Option<Hover> = None;

        let result = ResponseFormatter::format_hover(&hover);
        assert!(result.is_null());
    }

    #[test]
    fn test_format_definition_with_location() {
        let location = Some(Location {
            file_path: "src/main.rs".to_string(),
            range: Range {
                start: Position {
                    line: 10,
                    character: 0,
                },
                end: Position {
                    line: 10,
                    character: 5,
                },
            },
        });

        let result = ResponseFormatter::format_definition(&location);
        assert_eq!(result["uri"], "src/main.rs");
        assert_eq!(result["range"]["start"]["line"], 10);
    }

    #[test]
    fn test_format_definition_empty() {
        let location: Option<Location> = None;

        let result = ResponseFormatter::format_definition(&location);
        assert!(result.is_null());
    }

    #[test]
    fn test_format_completion_kind() {
        assert_eq!(
            ResponseFormatter::format_completion_kind(CompletionItemKind::Text),
            1
        );
        assert_eq!(
            ResponseFormatter::format_completion_kind(CompletionItemKind::Function),
            3
        );
        assert_eq!(
            ResponseFormatter::format_completion_kind(CompletionItemKind::Variable),
            6
        );
    }

    #[test]
    fn test_format_diagnostic_severity() {
        assert_eq!(
            ResponseFormatter::format_diagnostic_severity(DiagnosticSeverity::Error),
            1
        );
        assert_eq!(
            ResponseFormatter::format_diagnostic_severity(DiagnosticSeverity::Warning),
            2
        );
        assert_eq!(
            ResponseFormatter::format_diagnostic_severity(DiagnosticSeverity::Information),
            3
        );
        assert_eq!(
            ResponseFormatter::format_diagnostic_severity(DiagnosticSeverity::Hint),
            4
        );
    }

    #[test]
    fn test_format_for_vscode() {
        let data = json!({"test": "data"});
        let result = ResponseFormatter::format_for_vscode("completion", data);
        assert_eq!(result["jsonrpc"], "2.0");
        assert!(result.get("result").is_some());
    }

    #[test]
    fn test_format_for_terminal() {
        let data = json!({"test": "data"});
        let result = ResponseFormatter::format_for_terminal("completion", data);
        assert_eq!(result["type"], "completion");
        assert!(result.get("data").is_some());
    }

    #[test]
    fn test_format_error() {
        let result = ResponseFormatter::format_error(-32600, "Invalid Request");
        assert_eq!(result["jsonrpc"], "2.0");
        assert_eq!(result["error"]["code"], -32600);
        assert_eq!(result["error"]["message"], "Invalid Request");
    }
}
