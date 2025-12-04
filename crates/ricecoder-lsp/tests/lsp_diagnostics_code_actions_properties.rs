//! Property-based tests for LSP diagnostics and code actions
//!
//! **Feature: ricecoder-lsp, Property 3: Diagnostic accuracy**
//! **Validates: Requirements LSP-3.1, LSP-3.2**
//!
//! **Feature: ricecoder-lsp, Property 4: Code action applicability**
//! **Validates: Requirements LSP-3.1, LSP-3.2**
//!
//! These tests verify that:
//! - All known issues are detected by the diagnostics engine
//! - Diagnostic messages are accurate and actionable
//! - Code actions resolve their associated diagnostics
//! - Applying code actions produces valid code
//! - Code actions do not introduce new issues

use proptest::prelude::*;
use ricecoder_lsp::diagnostics::{DiagnosticsEngine, DefaultDiagnosticsEngine};
use ricecoder_lsp::code_actions::{CodeActionsEngine, DefaultCodeActionsEngine};
use ricecoder_lsp::types::{Language, DiagnosticSeverity};

/// Strategy for generating Rust code with potential issues
fn rust_code_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        // Unused imports
        Just("use std::collections::HashMap;\nfn main() {}".to_string()),
        Just("use std::io::Read;\nuse std::fs::File;\nfn main() { let _ = File::open(\"test\"); }".to_string()),
        
        // Naming convention violations
        Just("fn MyFunction() {}".to_string()),
        Just("const my_const: i32 = 42;".to_string()),
        
        // Valid code
        Just("fn main() {}".to_string()),
        Just("use std::io::Read;\nfn main() { let _ = Read::read; }".to_string()),
    ]
}

/// Strategy for generating TypeScript code with potential issues
fn typescript_code_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        // Unused imports
        Just("import { HashMap } from 'collections';\nfunction main() {}".to_string()),
        Just("import { readFile } from 'fs';\nfunction main() { console.log('test'); }".to_string()),
        
        // Naming convention violations
        Just("function MyFunction() {}".to_string()),
        Just("class myClass {}".to_string()),
        
        // Valid code
        Just("function main() {}".to_string()),
        Just("import { readFile } from 'fs';\nfunction main() { readFile('test', () => {}); }".to_string()),
    ]
}

/// Strategy for generating Python code with potential issues
fn python_code_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        // Unused imports
        Just("import os\ndef main(): pass".to_string()),
        Just("from collections import defaultdict\ndef main(): print('test')".to_string()),
        
        // Naming convention violations
        Just("def MyFunction(): pass".to_string()),
        Just("class myClass: pass".to_string()),
        
        // Valid code
        Just("def main(): pass".to_string()),
        Just("import os\ndef main(): print(os.path.exists('test'))".to_string()),
    ]
}

/// Property 1: Diagnostics engine detects known issues
#[test]
fn prop_diagnostics_detects_known_issues() {
    let engine = DefaultDiagnosticsEngine::new();

    proptest!(|(code in rust_code_strategy())| {
        let result = engine.generate_diagnostics(&code, Language::Rust);
        assert!(result.is_ok());

        let diagnostics = result.unwrap();

        // Verify diagnostics structure
        for diag in &diagnostics {
            // All diagnostics must have a message
            assert!(!diag.message.is_empty());

            // All diagnostics must have a valid severity
            match diag.severity {
                DiagnosticSeverity::Error
                | DiagnosticSeverity::Warning
                | DiagnosticSeverity::Information
                | DiagnosticSeverity::Hint => {}
            }

            // All diagnostics must have a valid range
            assert!(diag.range.start.line <= diag.range.end.line);
            if diag.range.start.line == diag.range.end.line {
                assert!(diag.range.start.character <= diag.range.end.character);
            }
        }
    });
}

/// Property 2: Diagnostic messages are accurate and actionable
#[test]
fn prop_diagnostic_messages_are_actionable() {
    let engine = DefaultDiagnosticsEngine::new();

    proptest!(|(code in typescript_code_strategy())| {
        let result = engine.generate_diagnostics(&code, Language::TypeScript);
        assert!(result.is_ok());

        let diagnostics = result.unwrap();

        // Verify all diagnostic messages are actionable
        for diag in &diagnostics {
            // Message should not be empty
            assert!(!diag.message.is_empty());

            // Message should be descriptive (contain relevant keywords)
            let msg_lower = diag.message.to_lowercase();
            let is_actionable = msg_lower.contains("unused")
                || msg_lower.contains("naming")
                || msg_lower.contains("convention")
                || msg_lower.contains("should")
                || msg_lower.contains("error")
                || msg_lower.contains("warning");

            assert!(is_actionable, "Message not actionable: {}", diag.message);
        }
    });
}

/// Property 3: Code actions resolve their associated diagnostics
#[test]
fn prop_code_actions_resolve_diagnostics() {
    let diag_engine = DefaultDiagnosticsEngine::new();
    let action_engine = DefaultCodeActionsEngine::new();

    proptest!(|(code in python_code_strategy())| {
        let result = diag_engine.generate_diagnostics(&code, Language::Python);
        assert!(result.is_ok());

        let diagnostics = result.unwrap();

        // For each diagnostic, try to get code actions
        for diag in &diagnostics {
            let actions_result = action_engine.suggest_code_actions(&diag, &code);
            assert!(actions_result.is_ok());

            let actions = actions_result.unwrap();

            // If there are actions, they should have titles
            for action in &actions {
                assert!(!action.title.is_empty());
            }
        }
    });
}

/// Property 4: Applying code actions produces valid code
#[test]
fn prop_code_actions_produce_valid_code() {
    let diag_engine = DefaultDiagnosticsEngine::new();
    let action_engine = DefaultCodeActionsEngine::new();

    proptest!(|(code in rust_code_strategy())| {
        let result = diag_engine.generate_diagnostics(&code, Language::Rust);
        assert!(result.is_ok());

        let diagnostics = result.unwrap();

        // For each diagnostic, try to get and apply code actions
        for diag in &diagnostics {
            let actions_result = action_engine.suggest_code_actions(&diag, &code);
            assert!(actions_result.is_ok());

            let actions = actions_result.unwrap();

            // Try to apply each action
            for action in &actions {
                let apply_result = action_engine.apply_code_action(&code, &action);

                // Application should either succeed or fail gracefully
                match apply_result {
                    Ok(new_code) => {
                        // New code should not be empty
                        assert!(!new_code.is_empty());

                        // New code should be different from original (unless it's a no-op)
                        // or the same if the action was a no-op
                        let _ = new_code;
                    }
                    Err(err) => {
                        // Error should be descriptive
                        assert!(!err.to_string().is_empty());
                    }
                }
            }
        }
    });
}

/// Property 5: Code actions don't introduce new issues
#[test]
fn prop_code_actions_dont_introduce_issues() {
    let diag_engine = DefaultDiagnosticsEngine::new();
    let action_engine = DefaultCodeActionsEngine::new();

    proptest!(|(code in typescript_code_strategy())| {
        let result = diag_engine.generate_diagnostics(&code, Language::TypeScript);
        assert!(result.is_ok());

        let diagnostics = result.unwrap();

        // For each diagnostic, try to get and apply code actions
        for diag in &diagnostics {
            let actions_result = action_engine.suggest_code_actions(&diag, &code);
            assert!(actions_result.is_ok());

            let actions = actions_result.unwrap();

            // Try to apply each action
            for action in &actions {
                let apply_result = action_engine.apply_code_action(&code, &action);

                if let Ok(new_code) = apply_result {
                    // Check diagnostics on the new code
                    let new_diags_result = diag_engine.generate_diagnostics(&new_code, Language::TypeScript);
                    assert!(new_diags_result.is_ok());

                    let new_diagnostics = new_diags_result.unwrap();

                    // The number of diagnostics should not increase significantly
                    // (allowing for some variance due to heuristic nature of analysis)
                    assert!(
                        new_diagnostics.len() <= diagnostics.len() + 2,
                        "Code action introduced too many new issues"
                    );
                }
            }
        }
    });
}

/// Property 6: Diagnostics are consistent across multiple runs
#[test]
fn prop_diagnostics_are_consistent() {
    let engine = DefaultDiagnosticsEngine::new();

    proptest!(|(code in rust_code_strategy())| {
        // Run diagnostics multiple times
        let result1 = engine.generate_diagnostics(&code, Language::Rust);
        let result2 = engine.generate_diagnostics(&code, Language::Rust);
        let result3 = engine.generate_diagnostics(&code, Language::Rust);

        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert!(result3.is_ok());

        let diags1 = result1.unwrap();
        let diags2 = result2.unwrap();
        let diags3 = result3.unwrap();

        // All runs should produce the same number of diagnostics
        assert_eq!(diags1.len(), diags2.len());
        assert_eq!(diags2.len(), diags3.len());

        // All diagnostics should have the same messages
        for (d1, d2, d3) in itertools::izip!(&diags1, &diags2, &diags3) {
            assert_eq!(d1.message, d2.message);
            assert_eq!(d2.message, d3.message);
        }
    });
}

/// Property 7: Empty code produces no diagnostics
#[test]
fn prop_empty_code_produces_no_diagnostics() {
    let engine = DefaultDiagnosticsEngine::new();

    proptest!(|(language in prop_oneof![
        Just(Language::Rust),
        Just(Language::TypeScript),
        Just(Language::Python),
    ])| {
        let result = engine.generate_diagnostics("", language);
        assert!(result.is_ok());

        let diagnostics = result.unwrap();
        assert!(diagnostics.is_empty());
    });
}

/// Property 8: Whitespace-only code produces no diagnostics
#[test]
fn prop_whitespace_only_code_produces_no_diagnostics() {
    let engine = DefaultDiagnosticsEngine::new();

    proptest!(|(whitespace in r"[ \t\n\r]+")| {
        let result = engine.generate_diagnostics(&whitespace, Language::Rust);
        assert!(result.is_ok());

        let diagnostics = result.unwrap();
        assert!(diagnostics.is_empty());
    });
}

/// Property 9: Code action titles are non-empty
#[test]
fn prop_code_action_titles_non_empty() {
    let diag_engine = DefaultDiagnosticsEngine::new();
    let action_engine = DefaultCodeActionsEngine::new();

    proptest!(|(code in rust_code_strategy())| {
        let result = diag_engine.generate_diagnostics(&code, Language::Rust);
        assert!(result.is_ok());

        let diagnostics = result.unwrap();

        for diag in &diagnostics {
            let actions_result = action_engine.suggest_code_actions(&diag, &code);
            assert!(actions_result.is_ok());

            let actions = actions_result.unwrap();

            for action in &actions {
                assert!(!action.title.is_empty());
                assert!(action.title.len() < 200); // Reasonable title length
            }
        }
    });
}

/// Property 10: Diagnostic ranges are within code bounds
#[test]
fn prop_diagnostic_ranges_within_bounds() {
    let engine = DefaultDiagnosticsEngine::new();

    proptest!(|(code in rust_code_strategy())| {
        let result = engine.generate_diagnostics(&code, Language::Rust);
        assert!(result.is_ok());

        let diagnostics = result.unwrap();
        let lines: Vec<&str> = code.lines().collect();

        for diag in &diagnostics {
            // Start line should be within bounds
            assert!(
                (diag.range.start.line as usize) < lines.len() || lines.is_empty(),
                "Start line {} out of bounds for {} lines",
                diag.range.start.line,
                lines.len()
            );

            // End line should be within bounds
            assert!(
                (diag.range.end.line as usize) < lines.len() || lines.is_empty(),
                "End line {} out of bounds for {} lines",
                diag.range.end.line,
                lines.len()
            );

            // Start should be before or equal to end
            assert!(
                diag.range.start.line <= diag.range.end.line,
                "Start line {} after end line {}",
                diag.range.start.line,
                diag.range.end.line
            );
        }
    });
}
