//! Rust-specific diagnostic rules

use super::DiagnosticsResult;
use crate::types::{Diagnostic, DiagnosticSeverity, Position, Range};

/// Generate diagnostics for Rust code
pub fn generate_rust_diagnostics(code: &str) -> DiagnosticsResult<Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();

    // Check for unused imports
    diagnostics.extend(check_unused_imports(code));

    // Check for unreachable code patterns
    diagnostics.extend(check_unreachable_code(code));

    // Check for naming conventions
    diagnostics.extend(check_naming_conventions(code));

    Ok(diagnostics)
}

/// Check for unused imports
fn check_unused_imports(code: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for (line_num, line) in code.lines().enumerate() {
        // Simple heuristic: look for use statements that might be unused
        if line.trim().starts_with("use ") && line.contains("::") {
            // Check if the imported name appears elsewhere in the code
            let parts: Vec<&str> = line.split("::").collect();
            if let Some(last_part) = parts.last() {
                let imported_name = last_part.trim().trim_end_matches(';').trim_end_matches(',');

                // Count occurrences of the imported name (excluding the import line itself)
                let occurrences = code
                    .lines()
                    .enumerate()
                    .filter(|(i, l)| *i != line_num && l.contains(imported_name))
                    .count();

                // If not used elsewhere, it might be unused
                if occurrences == 0 && !imported_name.is_empty() {
                    let range = Range::new(
                        Position::new(line_num as u32, 0),
                        Position::new(line_num as u32, line.len() as u32),
                    );

                    let diagnostic = Diagnostic::new(
                        range,
                        DiagnosticSeverity::Warning,
                        format!("Unused import: `{}`", imported_name),
                    );

                    diagnostics.push(diagnostic);
                }
            }
        }
    }

    diagnostics
}

/// Check for unreachable code patterns
fn check_unreachable_code(code: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for (line_num, line) in code.lines().enumerate() {
        let trimmed = line.trim();

        // Check for code after return/panic/unreachable
        if trimmed.starts_with("return")
            || trimmed.starts_with("panic!")
            || trimmed.starts_with("unreachable!")
        {
            // Check if there's code on the same line after the statement
            if let Some(pos) = line.find("return") {
                let after_return = &line[pos + 6..].trim();
                if !after_return.is_empty()
                    && !after_return.starts_with(';')
                    && !after_return.starts_with("(")
                {
                    let range = Range::new(
                        Position::new(line_num as u32, (pos + 6) as u32),
                        Position::new(line_num as u32, line.len() as u32),
                    );

                    let diagnostic = Diagnostic::new(
                        range,
                        DiagnosticSeverity::Warning,
                        "Unreachable code after return statement".to_string(),
                    );

                    diagnostics.push(diagnostic);
                }
            }
        }
    }

    diagnostics
}

/// Check for naming convention violations
fn check_naming_conventions(code: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for (line_num, line) in code.lines().enumerate() {
        // Check for function definitions with non-snake_case names
        if line.contains("fn ") {
            if let Some(fn_pos) = line.find("fn ") {
                let after_fn = &line[fn_pos + 3..];
                if let Some(paren_pos) = after_fn.find('(') {
                    let fn_name = after_fn[..paren_pos].trim();

                    // Check if name contains uppercase letters (not snake_case)
                    if fn_name.contains(|c: char| c.is_uppercase()) {
                        let range = Range::new(
                            Position::new(line_num as u32, (fn_pos + 3) as u32),
                            Position::new(line_num as u32, (fn_pos + 3 + fn_name.len()) as u32),
                        );

                        let diagnostic = Diagnostic::new(
                            range,
                            DiagnosticSeverity::Hint,
                            format!("Function name `{}` should be in snake_case", fn_name),
                        );

                        diagnostics.push(diagnostic);
                    }
                }
            }
        }

        // Check for const definitions with non-UPPER_CASE names
        if line.contains("const ") {
            if let Some(const_pos) = line.find("const ") {
                let after_const = &line[const_pos + 6..];
                if let Some(colon_pos) = after_const.find(':') {
                    let const_name = after_const[..colon_pos].trim();

                    // Check if name is not all uppercase
                    if !const_name
                        .chars()
                        .all(|c| c.is_uppercase() || c == '_' || c.is_numeric())
                    {
                        let range = Range::new(
                            Position::new(line_num as u32, (const_pos + 6) as u32),
                            Position::new(
                                line_num as u32,
                                (const_pos + 6 + const_name.len()) as u32,
                            ),
                        );

                        let diagnostic = Diagnostic::new(
                            range,
                            DiagnosticSeverity::Hint,
                            format!("Constant name `{}` should be in UPPER_CASE", const_name),
                        );

                        diagnostics.push(diagnostic);
                    }
                }
            }
        }
    }

    diagnostics
}
