//! Python-specific diagnostic rules

use super::DiagnosticsResult;
use crate::types::{Diagnostic, DiagnosticSeverity, Position, Range};

/// Generate diagnostics for Python code
pub fn generate_python_diagnostics(code: &str) -> DiagnosticsResult<Vec<Diagnostic>> {
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
        if line.trim().starts_with("import ") || line.trim().starts_with("from ") {
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

    let trimmed = line.trim();

    // Handle: from module import name1, name2
    if trimmed.starts_with("from ") {
        if let Some(import_pos) = trimmed.find(" import ") {
            let imports_str = &trimmed[import_pos + 8..];
            let mut current_pos = import_pos + 8;

            for part in imports_str.split(',') {
                let trimmed_part = part.trim();
                if !trimmed_part.is_empty() && trimmed_part != "*" {
                    // Handle "name as alias"
                    let name = if let Some(as_pos) = trimmed_part.find(" as ") {
                        trimmed_part[as_pos + 4..].trim().to_string()
                    } else {
                        trimmed_part.to_string()
                    };

                    names.push((name, current_pos));
                    current_pos += part.len() + 1;
                }
            }
        }
    }

    // Handle: import name1, name2
    if let Some(imports_str) = trimmed.strip_prefix("import ") {
        let mut current_pos = 7;

        for part in imports_str.split(',') {
            let trimmed_part = part.trim();
            if !trimmed_part.is_empty() {
                // Handle "name as alias"
                let name = if let Some(as_pos) = trimmed_part.find(" as ") {
                    trimmed_part[as_pos + 4..].trim().to_string()
                } else {
                    trimmed_part.to_string()
                };

                names.push((name, current_pos));
                current_pos += part.len() + 1;
            }
        }
    }

    names
}

/// Check for unreachable code patterns
fn check_unreachable_code(code: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for (line_num, line) in code.lines().enumerate() {
        let trimmed = line.trim();

        // Check for code after return/raise
        if trimmed.starts_with("return") || trimmed.starts_with("raise ") {
            // Check if there's code on the same line after the statement
            if let Some(pos) = line.find("return") {
                let after_return = &line[pos + 6..].trim();
                if !after_return.is_empty() && !after_return.starts_with('#') {
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
        if line.contains("def ") {
            if let Some(def_pos) = line.find("def ") {
                let after_def = &line[def_pos + 4..];
                if let Some(paren_pos) = after_def.find('(') {
                    let fn_name = after_def[..paren_pos].trim();

                    // Check if name contains uppercase letters (not snake_case)
                    if fn_name.contains(|c: char| c.is_uppercase()) {
                        let range = Range::new(
                            Position::new(line_num as u32, (def_pos + 4) as u32),
                            Position::new(line_num as u32, (def_pos + 4 + fn_name.len()) as u32),
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

        // Check for class definitions with non-PascalCase names
        if line.contains("class ") {
            if let Some(class_pos) = line.find("class ") {
                let after_class = &line[class_pos + 6..];
                if let Some(paren_or_colon) = after_class.find(['(', ':']) {
                    let class_name = after_class[..paren_or_colon].trim();

                    // Check if name starts with lowercase (should be PascalCase)
                    if class_name.chars().next().is_some_and(|c| c.is_lowercase()) {
                        let range = Range::new(
                            Position::new(line_num as u32, (class_pos + 6) as u32),
                            Position::new(
                                line_num as u32,
                                (class_pos + 6 + class_name.len()) as u32,
                            ),
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
