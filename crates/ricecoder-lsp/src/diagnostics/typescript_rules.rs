//! TypeScript-specific diagnostic rules

use crate::types::{Diagnostic, DiagnosticSeverity, Position, Range};
use super::DiagnosticsResult;

/// Generate diagnostics for TypeScript code
pub fn generate_typescript_diagnostics(code: &str) -> DiagnosticsResult<Vec<Diagnostic>> {
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
        // Check for import statements
        if line.trim().starts_with("import ") {
            // Extract imported names
            let imported_names = extract_import_names(line);

            for (name, start_pos) in imported_names {
                // Count occurrences of the imported name (excluding the import line itself)
                let occurrences = code
                    .lines()
                    .enumerate()
                    .filter(|(i, l)| *i != line_num && l.contains(&name))
                    .count();

                // If not used elsewhere, it might be unused
                if occurrences == 0 {
                    let range = Range::new(
                        Position::new(line_num as u32, start_pos as u32),
                        Position::new(line_num as u32, (start_pos + name.len()) as u32),
                    );

                    let diagnostic = Diagnostic::new(
                        range,
                        DiagnosticSeverity::Warning,
                        format!("Unused import: `{}`", name),
                    );

                    diagnostics.push(diagnostic);
                }
            }
        }
    }

    diagnostics
}

/// Extract imported names from an import statement
fn extract_import_names(line: &str) -> Vec<(String, usize)> {
    let mut names = Vec::new();

    // Handle: import { name1, name2 } from "module"
    if let Some(start) = line.find('{') {
        if let Some(end) = line.find('}') {
            let imports_str = &line[start + 1..end];
            let mut current_pos = start + 1;

            for part in imports_str.split(',') {
                let trimmed = part.trim();
                if !trimmed.is_empty() {
                    // Handle "name as alias"
                    let name = if let Some(as_pos) = trimmed.find(" as ") {
                        trimmed[as_pos + 4..].trim().to_string()
                    } else {
                        trimmed.to_string()
                    };

                    names.push((name, current_pos));
                    current_pos += part.len() + 1;
                }
            }
        }
    }

    // Handle: import name from "module"
    if let Some(from_pos) = line.find(" from ") {
        let before_from = &line[7..from_pos]; // Skip "import "
        let name = before_from.trim().to_string();
        if !name.is_empty() && !name.contains('{') {
            names.push((name, 7));
        }
    }

    names
}

/// Check for unreachable code patterns
fn check_unreachable_code(code: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for (line_num, line) in code.lines().enumerate() {
        let trimmed = line.trim();

        // Check for code after return/throw
        if trimmed.starts_with("return") || trimmed.starts_with("throw ") {
            // Check if there's code on the same line after the statement
            if let Some(pos) = line.find("return") {
                let after_return = &line[pos + 6..].trim();
                if !after_return.is_empty() && !after_return.starts_with(';') {
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
        // Check for function definitions with non-camelCase names
        if line.contains("function ") {
            if let Some(fn_pos) = line.find("function ") {
                let after_fn = &line[fn_pos + 9..];
                if let Some(paren_pos) = after_fn.find('(') {
                    let fn_name = after_fn[..paren_pos].trim();

                    // Check if name starts with uppercase (should be camelCase)
                    if fn_name.chars().next().is_some_and(|c| c.is_uppercase()) {
                        let range = Range::new(
                            Position::new(line_num as u32, (fn_pos + 9) as u32),
                            Position::new(line_num as u32, (fn_pos + 9 + fn_name.len()) as u32),
                        );

                        let diagnostic = Diagnostic::new(
                            range,
                            DiagnosticSeverity::Hint,
                            format!("Function name `{}` should be in camelCase", fn_name),
                        );

                        diagnostics.push(diagnostic);
                    }
                }
            }
        }

        // Check for class definitions with non-PascalCase names
        if line.contains("class ") {
            if let Some(class_pos) = line.find("class ") {
                let after_class = &line[class_pos + 6..];
                if let Some(space_or_brace) = after_class.find(|c: char| c.is_whitespace() || c == '{') {
                    let class_name = after_class[..space_or_brace].trim();

                    // Check if name starts with lowercase (should be PascalCase)
                    if class_name.chars().next().is_some_and(|c| c.is_lowercase()) {
                        let range = Range::new(
                            Position::new(line_num as u32, (class_pos + 6) as u32),
                            Position::new(line_num as u32, (class_pos + 6 + class_name.len()) as u32),
                        );

                        let diagnostic = Diagnostic::new(
                            range,
                            DiagnosticSeverity::Hint,
                            format!("Class name `{}` should be in PascalCase", class_name),
                        );

                        diagnostics.push(diagnostic);
                    }
                }
            }
        }
    }

    diagnostics
}
