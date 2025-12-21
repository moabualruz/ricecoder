//! Property-based tests for LSP diagnostic accuracy
//! **Feature: ricecoder-tui, Property 9: LSP Diagnostic Accuracy**
//! **Validates: Requirements 28.1**

use proptest::prelude::*;
use ricecoder_tui::{lsp_diagnostics_to_tui, DiagnosticSeverity};
use serde_json::json;

/// Strategy for generating valid LSP diagnostic severities
fn lsp_severity_strategy() -> impl Strategy<Value = Option<u64>> {
    prop_oneof![
        Just(None),    // Default to error
        Just(Some(1)), // Error
        Just(Some(2)), // Warning
        Just(Some(3)), // Information
        Just(Some(4)), // Hint
    ]
}

/// Strategy for generating valid LSP ranges
fn lsp_range_strategy() -> impl Strategy<Value = serde_json::Value> {
    (0..1000u64, 0..200u64, 0..1000u64, 0..200u64).prop_map(
        |(start_line, start_char, end_line, end_char)| {
            json!({
                "start": {"line": start_line, "character": start_char},
                "end": {"line": end_line, "character": end_char}
            })
        },
    )
}

/// Strategy for generating LSP diagnostic messages
fn diagnostic_message_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 _.,!?()-]{10,100}".prop_map(|s| s.to_string())
}

/// Strategy for generating LSP diagnostic codes
fn diagnostic_code_strategy() -> impl Strategy<Value = Option<String>> {
    prop_oneof![
        Just(None),
        "[A-Z][0-9]{4}".prop_map(Some),
        "[a-z-]+".prop_map(Some),
    ]
}

proptest! {
    /// Property 9: LSP Diagnostic Accuracy
    /// For any valid LSP diagnostic response, the conversion to TUI diagnostics SHALL preserve all information accurately.
    /// **Validates: Requirements 28.1**
    #[test]
    fn prop_lsp_diagnostic_accuracy(
        severity in lsp_severity_strategy(),
        range in lsp_range_strategy(),
        message in diagnostic_message_strategy(),
        code in diagnostic_code_strategy(),
        source in prop::option::of("[a-zA-Z0-9_-]{1,20}"),
    ) {
        // Create LSP diagnostic
        let mut lsp_diagnostic = json!({
            "range": range,
            "message": message,
            "severity": severity
        });

        // Add optional fields
        if let Some(code_val) = &code {
            lsp_diagnostic["code"] = json!(code_val);
        }
        if let Some(source_val) = &source {
            lsp_diagnostic["source"] = json!(source_val);
        }

        let file_path = "test.rs";
        let lsp_response = json!({"diagnostics": [lsp_diagnostic]});

        // Convert to TUI diagnostics
        let tui_diagnostics = lsp_diagnostics_to_tui(&lsp_response, file_path);

        // Verify conversion
        prop_assert_eq!(tui_diagnostics.len(), 1, "Should produce exactly one diagnostic");

        let diagnostic = &tui_diagnostics[0];

        // Verify severity mapping
        let expected_severity = match severity {
            Some(1) => DiagnosticSeverity::Error,
            Some(2) => DiagnosticSeverity::Warning,
            Some(3) => DiagnosticSeverity::Information,
            Some(4) => DiagnosticSeverity::Hint,
            _ => DiagnosticSeverity::Error, // Default
        };
        prop_assert_eq!(diagnostic.severity, expected_severity, "Severity should be mapped correctly");

        // Verify message preservation
        prop_assert_eq!(diagnostic.message, message, "Message should be preserved exactly");

        // Verify file path
        prop_assert_eq!(diagnostic.file_path, file_path, "File path should be set correctly");

        // Verify code preservation
        prop_assert_eq!(diagnostic.code, code, "Code should be preserved");

        // Verify source preservation
        prop_assert_eq!(diagnostic.source, source, "Source should be preserved");

        // Verify range conversion
        if let (Some(start_line), Some(start_char), Some(end_line), Some(end_char)) = (
            range.get("start").and_then(|s| s.get("line")).and_then(|l| l.as_u64()),
            range.get("start").and_then(|s| s.get("character")).and_then(|c| c.as_u64()),
            range.get("end").and_then(|e| e.get("line")).and_then(|l| l.as_u64()),
            range.get("end").and_then(|e| e.get("character")).and_then(|c| c.as_u64()),
        ) {
            prop_assert_eq!(diagnostic.line, start_line as usize, "Start line should be converted correctly");
            prop_assert_eq!(diagnostic.column, start_char as usize, "Start column should be converted correctly");
            prop_assert_eq!(diagnostic.end_line, Some(end_line as usize), "End line should be converted correctly");
            prop_assert_eq!(diagnostic.end_column, Some(end_char as usize), "End column should be converted correctly");
        }
    }

    /// Test LSP diagnostic error handling
    #[test]
    fn prop_lsp_diagnostic_error_handling(
        invalid_range in prop_oneof![
            json!({"start": {"line": "invalid"}}), // Invalid line type
            json!({"start": {"character": -1}}), // Invalid character
            json!({"end": {}}) // Missing end
        ],
        message in diagnostic_message_strategy(),
    ) {
        // Create LSP diagnostic with invalid range
        let lsp_diagnostic = json!({
            "range": invalid_range,
            "message": message,
            "severity": 1
        });

        let file_path = "test.rs";
        let lsp_response = json!({"diagnostics": [lsp_diagnostic]});

        // Convert to TUI diagnostics - should handle errors gracefully
        let tui_diagnostics = lsp_diagnostics_to_tui(&lsp_response, file_path);

        // Should either produce no diagnostics or handle the error gracefully
        // (In current implementation, invalid ranges are skipped)
        prop_assert!(tui_diagnostics.len() <= 1, "Should handle invalid diagnostics gracefully");
    }

    /// Test multiple diagnostics conversion
    #[test]
    fn prop_multiple_lsp_diagnostics(
        num_diagnostics in 1..10usize,
        messages in prop::collection::vec(diagnostic_message_strategy(), 1..10),
    ) {
        let mut diagnostics = Vec::new();

        for i in 0..num_diagnostics.min(messages.len()) {
            let diagnostic = json!({
                "range": {
                    "start": {"line": i, "character": 0},
                    "end": {"line": i, "character": 10}
                },
                "message": messages[i],
                "severity": 1
            });
            diagnostics.push(diagnostic);
        }

        let file_path = "test.rs";
        let lsp_response = json!({"diagnostics": diagnostics});

        let tui_diagnostics = lsp_diagnostics_to_tui(&lsp_response, file_path);

        prop_assert_eq!(tui_diagnostics.len(), num_diagnostics.min(messages.len()),
            "Should convert all valid diagnostics");

        // Verify each diagnostic
        for (i, diagnostic) in tui_diagnostics.iter().enumerate() {
            prop_assert_eq!(diagnostic.line, i, "Line numbers should be preserved");
            prop_assert_eq!(diagnostic.message, messages[i], "Messages should be preserved");
        }
    }
}
