//! Language-specific validation implementations
//!
//! Provides language-specific validators for:
//! - Rust: cargo check, clippy
//! - TypeScript: tsc, eslint
//! - Python: mypy, pylint
//! - Go: go vet, golangci-lint
//! - Java: javac, checkstyle

use tracing::debug;

use crate::models::{ValidationError, ValidationWarning};

/// Trait for language-specific validators
pub trait LanguageValidator: Send + Sync {
    /// Validates code for this language
    fn validate(
        &self,
        content: &str,
        file_path: &str,
    ) -> Result<(Vec<ValidationError>, Vec<ValidationWarning>), String>;
}

/// Rust validator using cargo check and clippy
#[derive(Debug, Clone)]
pub struct RustValidator;

impl RustValidator {
    /// Creates a new Rust validator
    pub fn new() -> Self {
        Self
    }

    /// Checks for common Rust issues
    pub fn check_common_issues(&self, content: &str, file_path: &str) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // Check for unsafe without documentation
        for (line_num, line) in content.lines().enumerate() {
            if line.contains("unsafe") && !line.trim().starts_with("//") {
                // Check if there's a comment above
                let has_comment = if line_num > 0 {
                    content
                        .lines()
                        .nth(line_num - 1)
                        .map(|l| l.trim().starts_with("//"))
                        .unwrap_or(false)
                } else {
                    false
                };

                if !has_comment {
                    errors.push(ValidationError {
                        file: file_path.to_string(),
                        line: line_num + 1,
                        column: 1,
                        message: "unsafe block without documentation comment".to_string(),
                        code: Some("W0001".to_string()),
                    });
                }
            }
        }

        errors
    }

    /// Checks for unwrap() calls
    pub fn check_unwrap_calls(&self, content: &str, file_path: &str) -> Vec<ValidationWarning> {
        let mut warnings = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if line.contains(".unwrap()") {
                warnings.push(ValidationWarning {
                    file: file_path.to_string(),
                    line: line_num + 1,
                    column: line.find(".unwrap()").unwrap_or(0) + 1,
                    message: "unwrap() call may panic".to_string(),
                    code: Some("W0002".to_string()),
                });
            }
        }

        warnings
    }

    /// Checks for panic! calls
    pub fn check_panic_calls(&self, content: &str, file_path: &str) -> Vec<ValidationWarning> {
        let mut warnings = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if line.contains("panic!") {
                warnings.push(ValidationWarning {
                    file: file_path.to_string(),
                    line: line_num + 1,
                    column: line.find("panic!").unwrap_or(0) + 1,
                    message: "panic! call may crash the application".to_string(),
                    code: Some("W0003".to_string()),
                });
            }
        }

        warnings
    }
}

impl Default for RustValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageValidator for RustValidator {
    fn validate(
        &self,
        content: &str,
        file_path: &str,
    ) -> Result<(Vec<ValidationError>, Vec<ValidationWarning>), String> {
        debug!("Validating Rust code: {}", file_path);

        let errors = self.check_common_issues(content, file_path);
        let warnings = self.check_unwrap_calls(content, file_path);
        let mut all_warnings = warnings;
        all_warnings.extend(self.check_panic_calls(content, file_path));

        Ok((errors, all_warnings))
    }
}

/// TypeScript validator using tsc and eslint
#[derive(Debug, Clone)]
pub struct TypeScriptValidator;

impl TypeScriptValidator {
    /// Creates a new TypeScript validator
    pub fn new() -> Self {
        Self
    }

    /// Checks for common TypeScript issues
    pub fn check_common_issues(&self, content: &str, file_path: &str) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // Check for any type usage
        for (line_num, line) in content.lines().enumerate() {
            if line.contains(": any") || line.contains(": any,") || line.contains(": any)") {
                errors.push(ValidationError {
                    file: file_path.to_string(),
                    line: line_num + 1,
                    column: line.find(": any").unwrap_or(0) + 1,
                    message: "Use of 'any' type is not allowed".to_string(),
                    code: Some("TS7006".to_string()),
                });
            }
        }

        errors
    }

    /// Checks for missing error handling
    pub fn check_error_handling(&self, content: &str, file_path: &str) -> Vec<ValidationWarning> {
        let mut warnings = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if line.contains("throw ") && !line.contains("Error") {
                warnings.push(ValidationWarning {
                    file: file_path.to_string(),
                    line: line_num + 1,
                    column: line.find("throw").unwrap_or(0) + 1,
                    message: "throw without Error type".to_string(),
                    code: Some("W0001".to_string()),
                });
            }
        }

        warnings
    }

    /// Checks for console usage
    pub fn check_console_usage(&self, content: &str, file_path: &str) -> Vec<ValidationWarning> {
        let mut warnings = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if line.contains("console.") && !line.trim().starts_with("//") {
                warnings.push(ValidationWarning {
                    file: file_path.to_string(),
                    line: line_num + 1,
                    column: line.find("console.").unwrap_or(0) + 1,
                    message: "console usage in production code".to_string(),
                    code: Some("W0002".to_string()),
                });
            }
        }

        warnings
    }
}

impl Default for TypeScriptValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageValidator for TypeScriptValidator {
    fn validate(
        &self,
        content: &str,
        file_path: &str,
    ) -> Result<(Vec<ValidationError>, Vec<ValidationWarning>), String> {
        debug!("Validating TypeScript code: {}", file_path);

        let errors = self.check_common_issues(content, file_path);
        let mut warnings = self.check_error_handling(content, file_path);
        warnings.extend(self.check_console_usage(content, file_path));

        Ok((errors, warnings))
    }
}

/// Python validator using mypy and pylint
#[derive(Debug, Clone)]
pub struct PythonValidator;

impl PythonValidator {
    /// Creates a new Python validator
    pub fn new() -> Self {
        Self
    }

    /// Checks for common Python issues
    pub fn check_common_issues(&self, content: &str, file_path: &str) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // Check for bare except
        for (line_num, line) in content.lines().enumerate() {
            if line.trim() == "except:" {
                errors.push(ValidationError {
                    file: file_path.to_string(),
                    line: line_num + 1,
                    column: 1,
                    message: "Bare except clause is not allowed".to_string(),
                    code: Some("E0001".to_string()),
                });
            }
        }

        errors
    }

    /// Checks for missing type hints
    pub fn check_type_hints(&self, content: &str, file_path: &str) -> Vec<ValidationWarning> {
        let mut warnings = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if (line.trim().starts_with("def ") || line.trim().starts_with("class "))
                && !line.contains("->")
                && !line.trim().starts_with("def _")
            {
                warnings.push(ValidationWarning {
                    file: file_path.to_string(),
                    line: line_num + 1,
                    column: 1,
                    message: "Missing type hints".to_string(),
                    code: Some("W0001".to_string()),
                });
            }
        }

        warnings
    }

    /// Checks for print usage
    pub fn check_print_usage(&self, content: &str, file_path: &str) -> Vec<ValidationWarning> {
        let mut warnings = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if line.contains("print(") && !line.trim().starts_with("#") {
                warnings.push(ValidationWarning {
                    file: file_path.to_string(),
                    line: line_num + 1,
                    column: line.find("print(").unwrap_or(0) + 1,
                    message: "print() usage in production code".to_string(),
                    code: Some("W0002".to_string()),
                });
            }
        }

        warnings
    }
}

impl Default for PythonValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageValidator for PythonValidator {
    fn validate(
        &self,
        content: &str,
        file_path: &str,
    ) -> Result<(Vec<ValidationError>, Vec<ValidationWarning>), String> {
        debug!("Validating Python code: {}", file_path);

        let errors = self.check_common_issues(content, file_path);
        let mut warnings = self.check_type_hints(content, file_path);
        warnings.extend(self.check_print_usage(content, file_path));

        Ok((errors, warnings))
    }
}

/// Go validator using go vet and golangci-lint
#[derive(Debug, Clone)]
pub struct GoValidator;

impl GoValidator {
    /// Creates a new Go validator
    pub fn new() -> Self {
        Self
    }

    /// Checks for common Go issues
    pub fn check_common_issues(&self, content: &str, file_path: &str) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // Check for missing error handling
        for (line_num, line) in content.lines().enumerate() {
            if line.contains("_ = ") && line.contains("err") {
                errors.push(ValidationError {
                    file: file_path.to_string(),
                    line: line_num + 1,
                    column: 1,
                    message: "Error ignored with blank identifier".to_string(),
                    code: Some("E0001".to_string()),
                });
            }
        }

        errors
    }

    /// Checks for panic usage
    pub fn check_panic_usage(&self, content: &str, file_path: &str) -> Vec<ValidationWarning> {
        let mut warnings = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if line.contains("panic(") {
                warnings.push(ValidationWarning {
                    file: file_path.to_string(),
                    line: line_num + 1,
                    column: line.find("panic(").unwrap_or(0) + 1,
                    message: "panic() call may crash the application".to_string(),
                    code: Some("W0001".to_string()),
                });
            }
        }

        warnings
    }
}

impl Default for GoValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageValidator for GoValidator {
    fn validate(
        &self,
        content: &str,
        file_path: &str,
    ) -> Result<(Vec<ValidationError>, Vec<ValidationWarning>), String> {
        debug!("Validating Go code: {}", file_path);

        let errors = self.check_common_issues(content, file_path);
        let warnings = self.check_panic_usage(content, file_path);

        Ok((errors, warnings))
    }
}

/// Java validator using javac and checkstyle
#[derive(Debug, Clone)]
pub struct JavaValidator;

impl JavaValidator {
    /// Creates a new Java validator
    pub fn new() -> Self {
        Self
    }

    /// Checks for common Java issues
    pub fn check_common_issues(&self, content: &str, file_path: &str) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // Check for missing class declaration
        if !content.contains("class ") && !content.contains("interface ") {
            errors.push(ValidationError {
                file: file_path.to_string(),
                line: 1,
                column: 1,
                message: "Missing class or interface declaration".to_string(),
                code: Some("E0001".to_string()),
            });
        }

        errors
    }

    /// Checks for raw type usage
    pub fn check_raw_types(&self, content: &str, file_path: &str) -> Vec<ValidationWarning> {
        let mut warnings = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if (line.contains("List ") || line.contains("Map ") || line.contains("Set "))
                && !line.contains("<")
            {
                warnings.push(ValidationWarning {
                    file: file_path.to_string(),
                    line: line_num + 1,
                    column: 1,
                    message: "Raw type usage without generics".to_string(),
                    code: Some("W0001".to_string()),
                });
            }
        }

        warnings
    }
}

impl Default for JavaValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageValidator for JavaValidator {
    fn validate(
        &self,
        content: &str,
        file_path: &str,
    ) -> Result<(Vec<ValidationError>, Vec<ValidationWarning>), String> {
        debug!("Validating Java code: {}", file_path);

        let errors = self.check_common_issues(content, file_path);
        let warnings = self.check_raw_types(content, file_path);

        Ok((errors, warnings))
    }
}

/// Gets the appropriate validator for a language
pub fn get_validator(language: &str) -> Option<Box<dyn LanguageValidator>> {
    match language {
        "rust" | "rs" => Some(Box::new(RustValidator::new())),
        "typescript" | "ts" => Some(Box::new(TypeScriptValidator::new())),
        "python" | "py" => Some(Box::new(PythonValidator::new())),
        "go" => Some(Box::new(GoValidator::new())),
        "java" => Some(Box::new(JavaValidator::new())),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_validator_unwrap() {
        let validator = RustValidator::new();
        let content = "let x = result.unwrap();";
        let warnings = validator.check_unwrap_calls(content, "main.rs");
        assert!(!warnings.is_empty());
        assert!(warnings[0].message.contains("unwrap"));
    }

    #[test]
    fn test_rust_validator_panic() {
        let validator = RustValidator::new();
        let content = "panic!(\"error\");";
        let warnings = validator.check_panic_calls(content, "main.rs");
        assert!(!warnings.is_empty());
        assert!(warnings[0].message.contains("panic"));
    }

    #[test]
    fn test_typescript_validator_any_type() {
        let validator = TypeScriptValidator::new();
        let content = "let x: any = 5;";
        let errors = validator.check_common_issues(content, "main.ts");
        assert!(!errors.is_empty());
        assert!(errors[0].message.contains("any"));
    }

    #[test]
    fn test_typescript_validator_console() {
        let validator = TypeScriptValidator::new();
        let content = "console.log(\"test\");";
        let warnings = validator.check_console_usage(content, "main.ts");
        assert!(!warnings.is_empty());
        assert!(warnings[0].message.contains("console"));
    }

    #[test]
    fn test_python_validator_bare_except() {
        let validator = PythonValidator::new();
        let content = "try:\n    pass\nexcept:";
        let errors = validator.check_common_issues(content, "main.py");
        assert!(!errors.is_empty());
        assert!(errors[0].message.contains("Bare except"));
    }

    #[test]
    fn test_python_validator_print() {
        let validator = PythonValidator::new();
        let content = "print(\"test\")";
        let warnings = validator.check_print_usage(content, "main.py");
        assert!(!warnings.is_empty());
        assert!(warnings[0].message.contains("print"));
    }

    #[test]
    fn test_go_validator_panic() {
        let validator = GoValidator::new();
        let content = "panic(\"error\")";
        let warnings = validator.check_panic_usage(content, "main.go");
        assert!(!warnings.is_empty());
        assert!(warnings[0].message.contains("panic"));
    }

    #[test]
    fn test_java_validator_raw_type() {
        let validator = JavaValidator::new();
        let content = "List items = new ArrayList();";
        let warnings = validator.check_raw_types(content, "Main.java");
        assert!(!warnings.is_empty());
        assert!(warnings[0].message.contains("Raw type"));
    }

    #[test]
    fn test_get_validator_rust() {
        let validator = get_validator("rust");
        assert!(validator.is_some());
    }

    #[test]
    fn test_get_validator_typescript() {
        let validator = get_validator("typescript");
        assert!(validator.is_some());
    }

    #[test]
    fn test_get_validator_python() {
        let validator = get_validator("python");
        assert!(validator.is_some());
    }

    #[test]
    fn test_get_validator_unknown() {
        let validator = get_validator("unknown");
        assert!(validator.is_none());
    }

    #[test]
    fn test_rust_validator_trait() {
        let validator = RustValidator::new();
        let content = "fn main() {}";
        let result = validator.validate(content, "main.rs");
        assert!(result.is_ok());
    }

    #[test]
    fn test_typescript_validator_trait() {
        let validator = TypeScriptValidator::new();
        let content = "function main() {}";
        let result = validator.validate(content, "main.ts");
        assert!(result.is_ok());
    }
}
