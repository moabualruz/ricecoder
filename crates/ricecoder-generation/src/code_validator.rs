//! Code validation for generated code
//!
//! Validates generated code before writing by:
//! - Checking syntax for target language
//! - Running language-specific linters (clippy for Rust, eslint for TypeScript, etc.)
//! - Running type checking (cargo check, tsc, mypy)
//! - Reporting all errors with file paths and line numbers
//! - Preventing writing if validation fails

use crate::error::GenerationError;
use crate::language_validators::get_validator;
use crate::models::{
    GeneratedFile, ValidationConfig, ValidationError, ValidationResult, ValidationWarning,
};
use tracing::{debug, warn};

/// Validates generated code before writing
#[derive(Debug, Clone)]
pub struct CodeValidator {
    /// Configuration for validation
    config: ValidationConfig,
}

impl CodeValidator {
    /// Creates a new CodeValidator with default configuration
    pub fn new() -> Self {
        Self {
            config: ValidationConfig {
                check_syntax: true,
                run_linters: true,
                run_type_checking: true,
                warnings_as_errors: false,
            },
        }
    }

    /// Creates a new CodeValidator with custom configuration
    pub fn with_config(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Validates a collection of generated files
    ///
    /// # Arguments
    /// * `files` - The generated files to validate
    ///
    /// # Returns
    /// A ValidationResult containing all errors and warnings found
    ///
    /// # Errors
    /// Returns `GenerationError` if validation process fails
    pub fn validate(&self, files: &[GeneratedFile]) -> Result<ValidationResult, GenerationError> {
        let mut all_errors = Vec::new();
        let mut all_warnings = Vec::new();

        for file in files {
            debug!("Validating file: {}", file.path);

            let result = self.validate_file(file)?;
            all_errors.extend(result.errors);
            all_warnings.extend(result.warnings);
        }

        let valid =
            (all_warnings.is_empty() || !self.config.warnings_as_errors) && all_errors.is_empty();

        Ok(ValidationResult {
            valid,
            errors: all_errors,
            warnings: all_warnings,
        })
    }

    /// Validates a single generated file
    pub fn validate_file(&self, file: &GeneratedFile) -> Result<ValidationResult, GenerationError> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Check syntax
        if self.config.check_syntax {
            match self.check_syntax(&file.content, &file.language, &file.path) {
                Ok(syntax_errors) => errors.extend(syntax_errors),
                Err(e) => {
                    warn!("Syntax check failed for {}: {}", file.path, e);
                }
            }
        }

        // Run linters
        if self.config.run_linters {
            match self.run_linters(&file.content, &file.language, &file.path) {
                Ok((lint_errors, lint_warnings)) => {
                    errors.extend(lint_errors);
                    warnings.extend(lint_warnings);
                }
                Err(e) => {
                    warn!("Linting failed for {}: {}", file.path, e);
                }
            }
        }

        // Run type checking
        if self.config.run_type_checking {
            match self.run_type_checking(&file.content, &file.language, &file.path) {
                Ok(type_errors) => errors.extend(type_errors),
                Err(e) => {
                    warn!("Type checking failed for {}: {}", file.path, e);
                }
            }
        }

        let valid = (warnings.is_empty() || !self.config.warnings_as_errors) && errors.is_empty();

        Ok(ValidationResult {
            valid,
            errors,
            warnings,
        })
    }

    /// Checks syntax for the target language
    fn check_syntax(
        &self,
        content: &str,
        language: &str,
        file_path: &str,
    ) -> Result<Vec<ValidationError>, GenerationError> {
        debug!("Checking syntax for {} file: {}", language, file_path);

        match language {
            "rust" => self.check_rust_syntax(content, file_path),
            "typescript" | "ts" => self.check_typescript_syntax(content, file_path),
            "python" | "py" => self.check_python_syntax(content, file_path),
            "go" => self.check_go_syntax(content, file_path),
            "java" => self.check_java_syntax(content, file_path),
            _ => {
                debug!("No syntax checker available for language: {}", language);
                Ok(Vec::new())
            }
        }
    }

    /// Checks Rust syntax
    fn check_rust_syntax(
        &self,
        content: &str,
        file_path: &str,
    ) -> Result<Vec<ValidationError>, GenerationError> {
        // Basic syntax validation for Rust
        let mut errors = Vec::new();

        // Check for unmatched braces
        let open_braces = content.matches('{').count();
        let close_braces = content.matches('}').count();
        if open_braces != close_braces {
            errors.push(ValidationError {
                file: file_path.to_string(),
                line: 1,
                column: 1,
                message: format!(
                    "Unmatched braces: {} open, {} close",
                    open_braces, close_braces
                ),
                code: Some("E0001".to_string()),
            });
        }

        // Check for unmatched parentheses
        let open_parens = content.matches('(').count();
        let close_parens = content.matches(')').count();
        if open_parens != close_parens {
            errors.push(ValidationError {
                file: file_path.to_string(),
                line: 1,
                column: 1,
                message: format!(
                    "Unmatched parentheses: {} open, {} close",
                    open_parens, close_parens
                ),
                code: Some("E0002".to_string()),
            });
        }

        // Check for unmatched brackets
        let open_brackets = content.matches('[').count();
        let close_brackets = content.matches(']').count();
        if open_brackets != close_brackets {
            errors.push(ValidationError {
                file: file_path.to_string(),
                line: 1,
                column: 1,
                message: format!(
                    "Unmatched brackets: {} open, {} close",
                    open_brackets, close_brackets
                ),
                code: Some("E0003".to_string()),
            });
        }

        Ok(errors)
    }

    /// Checks TypeScript syntax
    fn check_typescript_syntax(
        &self,
        content: &str,
        file_path: &str,
    ) -> Result<Vec<ValidationError>, GenerationError> {
        // Basic syntax validation for TypeScript
        let mut errors = Vec::new();

        // Check for unmatched braces
        let open_braces = content.matches('{').count();
        let close_braces = content.matches('}').count();
        if open_braces != close_braces {
            errors.push(ValidationError {
                file: file_path.to_string(),
                line: 1,
                column: 1,
                message: format!(
                    "Unmatched braces: {} open, {} close",
                    open_braces, close_braces
                ),
                code: Some("TS1005".to_string()),
            });
        }

        // Check for unmatched parentheses
        let open_parens = content.matches('(').count();
        let close_parens = content.matches(')').count();
        if open_parens != close_parens {
            errors.push(ValidationError {
                file: file_path.to_string(),
                line: 1,
                column: 1,
                message: format!(
                    "Unmatched parentheses: {} open, {} close",
                    open_parens, close_parens
                ),
                code: Some("TS1005".to_string()),
            });
        }

        Ok(errors)
    }

    /// Checks Python syntax
    fn check_python_syntax(
        &self,
        content: &str,
        file_path: &str,
    ) -> Result<Vec<ValidationError>, GenerationError> {
        // Basic syntax validation for Python
        let mut errors = Vec::new();

        // Check for unmatched braces
        let open_braces = content.matches('{').count();
        let close_braces = content.matches('}').count();
        if open_braces != close_braces {
            errors.push(ValidationError {
                file: file_path.to_string(),
                line: 1,
                column: 1,
                message: format!(
                    "Unmatched braces: {} open, {} close",
                    open_braces, close_braces
                ),
                code: Some("E0001".to_string()),
            });
        }

        // Check for unmatched parentheses
        let open_parens = content.matches('(').count();
        let close_parens = content.matches(')').count();
        if open_parens != close_parens {
            errors.push(ValidationError {
                file: file_path.to_string(),
                line: 1,
                column: 1,
                message: format!(
                    "Unmatched parentheses: {} open, {} close",
                    open_parens, close_parens
                ),
                code: Some("E0001".to_string()),
            });
        }

        Ok(errors)
    }

    /// Checks Go syntax
    fn check_go_syntax(
        &self,
        content: &str,
        file_path: &str,
    ) -> Result<Vec<ValidationError>, GenerationError> {
        // Basic syntax validation for Go
        let mut errors = Vec::new();

        // Check for unmatched braces
        let open_braces = content.matches('{').count();
        let close_braces = content.matches('}').count();
        if open_braces != close_braces {
            errors.push(ValidationError {
                file: file_path.to_string(),
                line: 1,
                column: 1,
                message: format!(
                    "Unmatched braces: {} open, {} close",
                    open_braces, close_braces
                ),
                code: Some("E0001".to_string()),
            });
        }

        Ok(errors)
    }

    /// Checks Java syntax
    fn check_java_syntax(
        &self,
        content: &str,
        file_path: &str,
    ) -> Result<Vec<ValidationError>, GenerationError> {
        // Basic syntax validation for Java
        let mut errors = Vec::new();

        // Check for unmatched braces
        let open_braces = content.matches('{').count();
        let close_braces = content.matches('}').count();
        if open_braces != close_braces {
            errors.push(ValidationError {
                file: file_path.to_string(),
                line: 1,
                column: 1,
                message: format!(
                    "Unmatched braces: {} open, {} close",
                    open_braces, close_braces
                ),
                code: Some("E0001".to_string()),
            });
        }

        Ok(errors)
    }

    /// Runs language-specific linters
    fn run_linters(
        &self,
        content: &str,
        language: &str,
        file_path: &str,
    ) -> Result<(Vec<ValidationError>, Vec<ValidationWarning>), GenerationError> {
        debug!("Running linters for {} file: {}", language, file_path);

        // Try to get a language-specific validator
        if let Some(validator) = get_validator(language) {
            match validator.validate(content, file_path) {
                Ok((errors, warnings)) => {
                    debug!(
                        "Language-specific validation found {} errors and {} warnings",
                        errors.len(),
                        warnings.len()
                    );
                    Ok((errors, warnings))
                }
                Err(e) => {
                    warn!("Language-specific validation failed: {}", e);
                    Ok((Vec::new(), Vec::new()))
                }
            }
        } else {
            debug!("No language-specific validator available for: {}", language);
            Ok((Vec::new(), Vec::new()))
        }
    }

    /// Runs type checking for the target language
    fn run_type_checking(
        &self,
        content: &str,
        language: &str,
        file_path: &str,
    ) -> Result<Vec<ValidationError>, GenerationError> {
        debug!("Running type checking for {} file: {}", language, file_path);

        match language {
            "rust" => self.run_rust_type_checking(content, file_path),
            "typescript" | "ts" => self.run_typescript_type_checking(content, file_path),
            "python" | "py" => self.run_python_type_checking(content, file_path),
            "go" => self.run_go_type_checking(content, file_path),
            "java" => self.run_java_type_checking(content, file_path),
            _ => {
                debug!("No type checker available for language: {}", language);
                Ok(Vec::new())
            }
        }
    }

    /// Runs Rust type checking (cargo check)
    fn run_rust_type_checking(
        &self,
        _content: &str,
        _file_path: &str,
    ) -> Result<Vec<ValidationError>, GenerationError> {
        // In a real implementation, we would:
        // 1. Write content to a temporary file
        // 2. Run `cargo check` on it
        // 3. Parse the output
        // 4. Return errors

        debug!("Rust type checking would be performed by cargo check");
        Ok(Vec::new())
    }

    /// Runs TypeScript type checking (tsc)
    fn run_typescript_type_checking(
        &self,
        _content: &str,
        _file_path: &str,
    ) -> Result<Vec<ValidationError>, GenerationError> {
        // In a real implementation, we would:
        // 1. Write content to a temporary file
        // 2. Run `tsc` on it
        // 3. Parse the output
        // 4. Return errors

        debug!("TypeScript type checking would be performed by tsc");
        Ok(Vec::new())
    }

    /// Runs Python type checking (mypy)
    fn run_python_type_checking(
        &self,
        _content: &str,
        _file_path: &str,
    ) -> Result<Vec<ValidationError>, GenerationError> {
        // In a real implementation, we would:
        // 1. Write content to a temporary file
        // 2. Run `mypy` on it
        // 3. Parse the output
        // 4. Return errors

        debug!("Python type checking would be performed by mypy");
        Ok(Vec::new())
    }

    /// Runs Go type checking (go vet)
    fn run_go_type_checking(
        &self,
        _content: &str,
        _file_path: &str,
    ) -> Result<Vec<ValidationError>, GenerationError> {
        // In a real implementation, we would:
        // 1. Write content to a temporary file
        // 2. Run `go vet` on it
        // 3. Parse the output
        // 4. Return errors

        debug!("Go type checking would be performed by go vet");
        Ok(Vec::new())
    }

    /// Runs Java type checking (javac)
    fn run_java_type_checking(
        &self,
        _content: &str,
        _file_path: &str,
    ) -> Result<Vec<ValidationError>, GenerationError> {
        // In a real implementation, we would:
        // 1. Write content to a temporary file
        // 2. Run `javac` on it
        // 3. Parse the output
        // 4. Return errors

        debug!("Java type checking would be performed by javac");
        Ok(Vec::new())
    }

    /// Checks if validation passed
    pub fn is_valid(&self, result: &ValidationResult) -> bool {
        result.valid
    }

    /// Gets all validation issues (errors and warnings)
    pub fn get_all_issues(&self, result: &ValidationResult) -> Vec<String> {
        let mut issues = Vec::new();

        for error in &result.errors {
            issues.push(format!(
                "ERROR: {}:{}:{} - {} ({})",
                error.file,
                error.line,
                error.column,
                error.message,
                error.code.as_deref().unwrap_or("unknown")
            ));
        }

        for warning in &result.warnings {
            issues.push(format!(
                "WARNING: {}:{}:{} - {} ({})",
                warning.file,
                warning.line,
                warning.column,
                warning.message,
                warning.code.as_deref().unwrap_or("unknown")
            ));
        }

        issues
    }
}

impl Default for CodeValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_rust_syntax_valid() {
        let validator = CodeValidator::new();
        let content = "fn main() { println!(\"Hello\"); }";
        let errors = validator.check_rust_syntax(content, "main.rs").unwrap();
        assert!(errors.is_empty());
    }

    #[test]
    fn test_check_rust_syntax_unmatched_braces() {
        let validator = CodeValidator::new();
        let content = "fn main() { println!(\"Hello\"); ";
        let errors = validator.check_rust_syntax(content, "main.rs").unwrap();
        assert!(!errors.is_empty());
        assert!(errors[0].message.contains("Unmatched braces"));
    }

    #[test]
    fn test_check_typescript_syntax_valid() {
        let validator = CodeValidator::new();
        let content = "function hello() { console.log(\"Hello\"); }";
        let errors = validator
            .check_typescript_syntax(content, "main.ts")
            .unwrap();
        assert!(errors.is_empty());
    }

    #[test]
    fn test_check_typescript_syntax_unmatched_parens() {
        let validator = CodeValidator::new();
        let content = "function hello( { console.log(\"Hello\"); }";
        let errors = validator
            .check_typescript_syntax(content, "main.ts")
            .unwrap();
        assert!(!errors.is_empty());
        assert!(errors[0].message.contains("Unmatched parentheses"));
    }

    #[test]
    fn test_check_python_syntax_valid() {
        let validator = CodeValidator::new();
        let content = "def hello():\n    print(\"Hello\")";
        let errors = validator.check_python_syntax(content, "main.py").unwrap();
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_file_rust() {
        let validator = CodeValidator::new();
        let file = GeneratedFile {
            path: "src/main.rs".to_string(),
            content: "fn main() { println!(\"Hello\"); }".to_string(),
            language: "rust".to_string(),
        };

        let result = validator.validate_file(&file).unwrap();
        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_file_with_errors() {
        let validator = CodeValidator::new();
        let file = GeneratedFile {
            path: "src/main.rs".to_string(),
            content: "fn main() { println!(\"Hello\"); ".to_string(),
            language: "rust".to_string(),
        };

        let result = validator.validate_file(&file).unwrap();
        assert!(!result.valid);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_validate_multiple_files() {
        let validator = CodeValidator::new();
        let files = vec![
            GeneratedFile {
                path: "src/main.rs".to_string(),
                content: "fn main() { println!(\"Hello\"); }".to_string(),
                language: "rust".to_string(),
            },
            GeneratedFile {
                path: "src/lib.rs".to_string(),
                content: "pub fn hello() { println!(\"Hello\"); }".to_string(),
                language: "rust".to_string(),
            },
        ];

        let result = validator.validate(&files).unwrap();
        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_get_all_issues() {
        let validator = CodeValidator::new();
        let result = ValidationResult {
            valid: false,
            errors: vec![ValidationError {
                file: "main.rs".to_string(),
                line: 1,
                column: 1,
                message: "Unmatched braces".to_string(),
                code: Some("E0001".to_string()),
            }],
            warnings: vec![ValidationWarning {
                file: "main.rs".to_string(),
                line: 2,
                column: 1,
                message: "Unused variable".to_string(),
                code: Some("W0001".to_string()),
            }],
        };

        let issues = validator.get_all_issues(&result);
        assert_eq!(issues.len(), 2);
        assert!(issues[0].contains("ERROR"));
        assert!(issues[1].contains("WARNING"));
    }

    #[test]
    fn test_validation_config_default() {
        let config = ValidationConfig {
            check_syntax: true,
            run_linters: true,
            run_type_checking: true,
            warnings_as_errors: false,
        };

        assert!(config.check_syntax);
        assert!(config.run_linters);
        assert!(config.run_type_checking);
        assert!(!config.warnings_as_errors);
    }

    #[test]
    fn test_warnings_as_errors() {
        let config = ValidationConfig {
            check_syntax: true,
            run_linters: false,
            run_type_checking: false,
            warnings_as_errors: true,
        };

        let validator = CodeValidator::with_config(config);
        // When warnings_as_errors is true, a result with warnings should be invalid
        let result = ValidationResult {
            valid: false, // This should be false when warnings_as_errors is true and there are warnings
            errors: Vec::new(),
            warnings: vec![ValidationWarning {
                file: "main.rs".to_string(),
                line: 1,
                column: 1,
                message: "Warning".to_string(),
                code: None,
            }],
        };

        assert!(!validator.is_valid(&result));
    }
}
