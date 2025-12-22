//! LSP integration for TUI diagnostics display
//!
//! This module provides integration between LSP servers and the TUI diagnostics system,
//! converting LSP diagnostic responses into TUI-compatible diagnostic items.

use std::collections::HashMap;

use serde_json::Value;

use super::diagnostics_widget::{
    DiagnosticItem, DiagnosticLocation, DiagnosticRelatedInformation, DiagnosticSeverity,
};

/// LSP diagnostic severity levels (match LSP specification)
#[derive(Debug, Clone, Copy)]
enum LspSeverity {
    Error = 1,
    Warning = 2,
    Information = 3,
    Hint = 4,
}

impl LspSeverity {
    fn from_number(n: u64) -> Option<Self> {
        match n {
            1 => Some(LspSeverity::Error),
            2 => Some(LspSeverity::Warning),
            3 => Some(LspSeverity::Information),
            4 => Some(LspSeverity::Hint),
            _ => None,
        }
    }
}

/// Convert LSP diagnostic severity to TUI diagnostic severity
fn lsp_severity_to_tui(severity: Option<u64>) -> DiagnosticSeverity {
    severity
        .and_then(LspSeverity::from_number)
        .map(|s| match s {
            LspSeverity::Error => DiagnosticSeverity::Error,
            LspSeverity::Warning => DiagnosticSeverity::Warning,
            LspSeverity::Information => DiagnosticSeverity::Information,
            LspSeverity::Hint => DiagnosticSeverity::Hint,
        })
        .unwrap_or(DiagnosticSeverity::Error)
}

/// LSP range structure
#[derive(Debug, Clone)]
struct LspRange {
    start_line: usize,
    start_character: usize,
    end_line: usize,
    end_character: usize,
}

impl LspRange {
    fn from_value(value: &Value) -> Option<Self> {
        let start = value.get("start")?;
        let end = value.get("end")?;

        Some(LspRange {
            start_line: start.get("line")?.as_u64()? as usize,
            start_character: start.get("character")?.as_u64()? as usize,
            end_line: end.get("line")?.as_u64()? as usize,
            end_character: end.get("character")?.as_u64()? as usize,
        })
    }
}

/// Convert LSP diagnostics response to TUI diagnostic items
pub fn lsp_diagnostics_to_tui(lsp_response: &Value, file_path: &str) -> Vec<DiagnosticItem> {
    let mut diagnostics = Vec::new();

    // LSP publishDiagnostics response structure:
    // {
    //   "uri": "file:///path/to/file",
    //   "diagnostics": [
    //     {
    //       "range": {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 5}},
    //       "severity": 1,
    //       "code": "E0425",
    //       "source": "rustc",
    //       "message": "cannot find value `x` in this scope",
    //       "relatedInformation": [...]
    //     }
    //   ]
    // }

    if let Some(diagnostics_array) = lsp_response.get("diagnostics").and_then(|d| d.as_array()) {
        for diagnostic_value in diagnostics_array {
            if let Some(diagnostic) = lsp_diagnostic_to_tui(diagnostic_value, file_path) {
                diagnostics.push(diagnostic);
            }
        }
    }

    diagnostics
}

/// Convert a single LSP diagnostic to TUI diagnostic item
fn lsp_diagnostic_to_tui(diagnostic: &Value, file_path: &str) -> Option<DiagnosticItem> {
    let range = diagnostic.get("range").and_then(LspRange::from_value)?;
    let severity = diagnostic.get("severity").and_then(|s| s.as_u64());
    let message = diagnostic.get("message")?.as_str()?;
    let code = diagnostic
        .get("code")
        .and_then(|c| c.as_str())
        .map(|s| s.to_string());
    let source = diagnostic
        .get("source")
        .and_then(|s| s.as_str())
        .map(|s| s.to_string());

    let mut diagnostic_item = DiagnosticItem::new(
        lsp_severity_to_tui(severity),
        message,
        file_path,
        range.start_line,
        range.start_character,
    )
    .with_end_position(range.end_line, range.end_character);

    if let Some(code) = code {
        diagnostic_item = diagnostic_item.with_code(code);
    }

    if let Some(source) = source {
        diagnostic_item = diagnostic_item.with_source(source);
    }

    // Handle related information
    if let Some(related_array) = diagnostic
        .get("relatedInformation")
        .and_then(|r| r.as_array())
    {
        for related in related_array {
            if let Some(related_info) = parse_related_information(related) {
                diagnostic_item = diagnostic_item.with_related_info(related_info);
            }
        }
    }

    Some(diagnostic_item)
}

/// Parse LSP related information
fn parse_related_information(related: &Value) -> Option<DiagnosticRelatedInformation> {
    let location = related.get("location")?;
    let uri = location.get("uri")?.as_str()?;
    let range = location.get("range").and_then(LspRange::from_value)?;
    let message = related.get("message")?.as_str()?;

    // Convert URI to file path (simplified - assumes file:// scheme)
    let file_path = if uri.starts_with("file://") {
        uri[7..].to_string()
    } else {
        uri.to_string()
    };

    Some(DiagnosticRelatedInformation {
        location: DiagnosticLocation {
            file_path,
            line: range.start_line,
            column: range.start_character,
        },
        message: message.to_string(),
    })
}

/// Convert LSP hover response to plain text for TUI display
pub fn lsp_hover_to_text(hover_response: &Value) -> Option<String> {
    // LSP hover response structure:
    // {
    //   "contents": {
    //     "kind": "markdown",
    //     "value": "# Function Name\n\nDescription..."
    //   } | "string content" | ["multiple", "contents"]
    // }

    let contents = hover_response.get("contents")?;

    if let Some(value) = contents.as_str() {
        return Some(value.to_string());
    }

    if let Some(obj) = contents.as_object() {
        if let Some(value) = obj.get("value").and_then(|v| v.as_str()) {
            return Some(value.to_string());
        }
    }

    if let Some(array) = contents.as_array() {
        let mut result = String::new();
        for item in array {
            if let Some(text) = item.as_str() {
                if !result.is_empty() {
                    result.push('\n');
                }
                result.push_str(text);
            }
        }
        if !result.is_empty() {
            return Some(result);
        }
    }

    None
}

/// Extract language from file path for syntax highlighting hints
pub fn language_from_file_path(file_path: &str) -> Option<&'static str> {
    if file_path.ends_with(".rs") {
        Some("rust")
    } else if file_path.ends_with(".js") || file_path.ends_with(".mjs") {
        Some("javascript")
    } else if file_path.ends_with(".ts") {
        Some("typescript")
    } else if file_path.ends_with(".py") {
        Some("python")
    } else if file_path.ends_with(".java") {
        Some("java")
    } else if file_path.ends_with(".cpp")
        || file_path.ends_with(".cc")
        || file_path.ends_with(".cxx")
    {
        Some("cpp")
    } else if file_path.ends_with(".c") {
        Some("c")
    } else if file_path.ends_with(".go") {
        Some("go")
    } else if file_path.ends_with(".php") {
        Some("php")
    } else if file_path.ends_with(".rb") {
        Some("ruby")
    } else if file_path.ends_with(".swift") {
        Some("swift")
    } else if file_path.ends_with(".kt") {
        Some("kotlin")
    } else if file_path.ends_with(".scala") {
        Some("scala")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_lsp_diagnostic_conversion() {
        let lsp_diagnostic = json!({
            "range": {
                "start": {"line": 5, "character": 10},
                "end": {"line": 5, "character": 15}
            },
            "severity": 1,
            "code": "E0425",
            "source": "rustc",
            "message": "cannot find value `x` in this scope"
        });

        let diagnostics =
            lsp_diagnostics_to_tui(&json!({"diagnostics": [lsp_diagnostic]}), "main.rs");

        assert_eq!(diagnostics.len(), 1);
        let diagnostic = &diagnostics[0];

        assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
        assert_eq!(diagnostic.message, "cannot find value `x` in this scope");
        assert_eq!(diagnostic.file_path, "main.rs");
        assert_eq!(diagnostic.line, 5);
        assert_eq!(diagnostic.column, 10);
        assert_eq!(diagnostic.code, Some("E0425".to_string()));
        assert_eq!(diagnostic.source, Some("rustc".to_string()));
    }

    #[test]
    fn test_lsp_hover_conversion() {
        let hover_response = json!({
            "contents": {
                "kind": "markdown",
                "value": "# println!\n\nPrints to stdout"
            }
        });

        let text = lsp_hover_to_text(&hover_response);
        assert_eq!(text, Some("# println!\n\nPrints to stdout".to_string()));
    }

    #[test]
    fn test_language_detection() {
        assert_eq!(language_from_file_path("main.rs"), Some("rust"));
        assert_eq!(language_from_file_path("script.js"), Some("javascript"));
        assert_eq!(language_from_file_path("app.py"), Some("python"));
        assert_eq!(language_from_file_path("unknown.xyz"), None);
    }

    #[test]
    fn test_empty_diagnostics() {
        let diagnostics = lsp_diagnostics_to_tui(&json!({"diagnostics": []}), "test.rs");
        assert_eq!(diagnostics.len(), 0);
    }
}
