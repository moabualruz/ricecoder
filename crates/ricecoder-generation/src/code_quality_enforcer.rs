//! Code quality enforcement for generated code
//!
//! Ensures generated code follows project standards including:
//! - Doc comments for all public types and functions
//! - Language-specific naming conventions
//! - Error handling for fallible operations
//! - Optional unit test generation

use crate::error::GenerationError;
use crate::models::GeneratedFile;

/// Configuration for code quality enforcement
#[derive(Debug, Clone)]
pub struct CodeQualityConfig {
    /// Whether to require doc comments for public items
    pub require_doc_comments: bool,
    /// Whether to generate unit tests
    pub generate_tests: bool,
    /// Whether to enforce naming conventions
    pub enforce_naming: bool,
    /// Whether to enforce error handling
    pub enforce_error_handling: bool,
}

impl Default for CodeQualityConfig {
    fn default() -> Self {
        Self {
            require_doc_comments: true,
            generate_tests: false,
            enforce_naming: true,
            enforce_error_handling: true,
        }
    }
}

/// Enforces code quality standards on generated code
#[derive(Debug, Clone)]
pub struct CodeQualityEnforcer {
    /// Configuration for quality enforcement
    config: CodeQualityConfig,
}

impl CodeQualityEnforcer {
    /// Creates a new CodeQualityEnforcer with default configuration
    pub fn new() -> Self {
        Self {
            config: CodeQualityConfig::default(),
        }
    }

    /// Creates a new CodeQualityEnforcer with custom configuration
    pub fn with_config(config: CodeQualityConfig) -> Self {
        Self { config }
    }

    /// Enforces code quality standards on generated files
    ///
    /// # Arguments
    /// * `files` - The generated files to enforce quality on
    ///
    /// # Returns
    /// A vector of enhanced files with quality improvements
    ///
    /// # Errors
    /// Returns `GenerationError` if quality enforcement fails
    pub fn enforce(
        &self,
        files: Vec<GeneratedFile>,
    ) -> Result<Vec<GeneratedFile>, GenerationError> {
        let mut enhanced_files = Vec::new();

        for file in files {
            let enhanced = self.enforce_file(&file)?;
            enhanced_files.push(enhanced);
        }

        Ok(enhanced_files)
    }

    /// Enforces quality standards on a single file
    pub fn enforce_file(&self, file: &GeneratedFile) -> Result<GeneratedFile, GenerationError> {
        let mut content = file.content.clone();

        // Apply quality standards based on language
        match file.language.as_str() {
            "rust" => {
                content = self.enforce_rust_quality(&content)?;
            }
            "typescript" | "ts" => {
                content = self.enforce_typescript_quality(&content)?;
            }
            "python" | "py" => {
                content = self.enforce_python_quality(&content)?;
            }
            _ => {
                // For unknown languages, apply generic quality standards
                content = self.enforce_generic_quality(&content)?;
            }
        }

        Ok(GeneratedFile {
            path: file.path.clone(),
            content,
            language: file.language.clone(),
        })
    }

    /// Enforces Rust-specific quality standards
    fn enforce_rust_quality(&self, content: &str) -> Result<String, GenerationError> {
        let mut enhanced = content.to_string();

        if self.config.require_doc_comments {
            enhanced = self.add_rust_doc_comments(&enhanced);
        }

        if self.config.enforce_naming {
            enhanced = self.enforce_rust_naming(&enhanced);
        }

        if self.config.enforce_error_handling {
            enhanced = self.enforce_rust_error_handling(&enhanced);
        }

        Ok(enhanced)
    }

    /// Adds doc comments to Rust public items
    fn add_rust_doc_comments(&self, content: &str) -> String {
        let mut result = String::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i];

            // Check if this is a public item without doc comments
            if (line.contains("pub fn ")
                || line.contains("pub struct ")
                || line.contains("pub enum "))
                && !line.trim().starts_with("///")
            {
                // Check if previous line has doc comment
                let has_doc = if i > 0 {
                    lines[i - 1].trim().starts_with("///")
                } else {
                    false
                };

                if !has_doc {
                    // Extract the item name
                    let item_name = if let Some(start) = line.find("pub ") {
                        let after_pub = &line[start + 4..];
                        if let Some(space_idx) = after_pub.find(' ') {
                            after_pub[..space_idx].to_string()
                        } else if let Some(paren_idx) = after_pub.find('(') {
                            after_pub[..paren_idx].to_string()
                        } else if let Some(brace_idx) = after_pub.find('{') {
                            after_pub[..brace_idx].to_string()
                        } else {
                            "item".to_string()
                        }
                    } else {
                        "item".to_string()
                    };

                    // Add doc comment
                    let indent = line.len() - line.trim_start().len();
                    result.push_str(&" ".repeat(indent));
                    result.push_str(&format!("/// {}\n", item_name));
                }
            }

            result.push_str(line);
            result.push('\n');
            i += 1;
        }

        result
    }

    /// Enforces Rust naming conventions
    fn enforce_rust_naming(&self, content: &str) -> String {
        // This is a simplified implementation
        // In a real implementation, we would use a proper parser
        content.to_string()
    }

    /// Enforces Rust error handling patterns
    fn enforce_rust_error_handling(&self, content: &str) -> String {
        let result = content.to_string();

        // Check for functions that should return Result
        if result.contains("fn ") && result.contains("io::") {
            // This is a simplified check - in reality we'd need proper parsing
            if !result.contains("Result<") && !result.contains("-> ") {
                // Function likely needs error handling
            }
        }

        result
    }

    /// Enforces TypeScript-specific quality standards
    fn enforce_typescript_quality(&self, content: &str) -> Result<String, GenerationError> {
        let mut enhanced = content.to_string();

        if self.config.require_doc_comments {
            enhanced = self.add_typescript_doc_comments(&enhanced);
        }

        if self.config.enforce_naming {
            enhanced = self.enforce_typescript_naming(&enhanced);
        }

        if self.config.enforce_error_handling {
            enhanced = self.enforce_typescript_error_handling(&enhanced);
        }

        Ok(enhanced)
    }

    /// Adds JSDoc comments to TypeScript public items
    fn add_typescript_doc_comments(&self, content: &str) -> String {
        let mut result = String::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i];

            // Check if this is a public export without JSDoc
            if (line.contains("export function ")
                || line.contains("export class ")
                || line.contains("export interface "))
                && !line.trim().starts_with("/**")
            {
                // Check if previous line has JSDoc
                let has_doc = if i > 0 {
                    lines[i - 1].trim().starts_with("/**") || lines[i - 1].trim().starts_with("*")
                } else {
                    false
                };

                if !has_doc {
                    // Extract the item name
                    let item_name = if let Some(start) = line.find("export ") {
                        let after_export = &line[start + 7..];
                        if let Some(space_idx) = after_export.find(' ') {
                            after_export[space_idx + 1..]
                                .split('(')
                                .next()
                                .unwrap_or("item")
                        } else {
                            "item"
                        }
                    } else {
                        "item"
                    };

                    // Add JSDoc comment
                    let indent = line.len() - line.trim_start().len();
                    result.push_str(&" ".repeat(indent));
                    result.push_str("/**\n");
                    result.push_str(&" ".repeat(indent));
                    result.push_str(&format!(" * {}\n", item_name));
                    result.push_str(&" ".repeat(indent));
                    result.push_str(" */\n");
                }
            }

            result.push_str(line);
            result.push('\n');
            i += 1;
        }

        result
    }

    /// Enforces TypeScript naming conventions
    fn enforce_typescript_naming(&self, content: &str) -> String {
        // This is a simplified implementation
        content.to_string()
    }

    /// Enforces TypeScript error handling patterns
    fn enforce_typescript_error_handling(&self, content: &str) -> String {
        // This is a simplified implementation
        content.to_string()
    }

    /// Enforces Python-specific quality standards
    fn enforce_python_quality(&self, content: &str) -> Result<String, GenerationError> {
        let mut enhanced = content.to_string();

        if self.config.require_doc_comments {
            enhanced = self.add_python_doc_comments(&enhanced);
        }

        if self.config.enforce_naming {
            enhanced = self.enforce_python_naming(&enhanced);
        }

        if self.config.enforce_error_handling {
            enhanced = self.enforce_python_error_handling(&enhanced);
        }

        Ok(enhanced)
    }

    /// Adds docstrings to Python public items
    fn add_python_doc_comments(&self, content: &str) -> String {
        let mut result = String::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i];

            // Check if this is a function or class definition
            if (line.trim().starts_with("def ") || line.trim().starts_with("class "))
                && !line.trim().starts_with("def _")
                && !line.trim().starts_with("class _")
            {
                // Check if next line has docstring
                let has_docstring = if i + 1 < lines.len() {
                    let next_line = lines[i + 1].trim();
                    next_line.starts_with("\"\"\"") || next_line.starts_with("'''")
                } else {
                    false
                };

                result.push_str(line);
                result.push('\n');

                if !has_docstring {
                    // Extract the item name
                    let item_name = if let Some(start) = line.find("def ") {
                        let after_def = &line[start + 4..];
                        after_def.split('(').next().unwrap_or("item")
                    } else if let Some(start) = line.find("class ") {
                        let after_class = &line[start + 6..];
                        after_class.split('(').next().unwrap_or("item")
                    } else {
                        "item"
                    };

                    // Add docstring
                    let indent = line.len() - line.trim_start().len();
                    result.push_str(&" ".repeat(indent + 4));
                    result.push_str(&format!("\"\"\"{}.\"\"\"\n", item_name));
                }

                i += 1;
                continue;
            }

            result.push_str(line);
            result.push('\n');
            i += 1;
        }

        result
    }

    /// Enforces Python naming conventions
    fn enforce_python_naming(&self, content: &str) -> String {
        // This is a simplified implementation
        content.to_string()
    }

    /// Enforces Python error handling patterns
    fn enforce_python_error_handling(&self, content: &str) -> String {
        // This is a simplified implementation
        content.to_string()
    }

    /// Enforces generic quality standards for unknown languages
    fn enforce_generic_quality(&self, content: &str) -> Result<String, GenerationError> {
        // For unknown languages, return content as-is
        Ok(content.to_string())
    }

    /// Checks if code has doc comments for public items
    pub fn check_doc_comments(&self, content: &str, language: &str) -> Vec<String> {
        let mut issues = Vec::new();

        match language {
            "rust" => {
                for line in content.lines() {
                    if (line.contains("pub fn ")
                        || line.contains("pub struct ")
                        || line.contains("pub enum "))
                        && !line.trim().starts_with("///")
                    {
                        issues.push(format!("Missing doc comment: {}", line.trim()));
                    }
                }
            }
            "typescript" | "ts" => {
                for line in content.lines() {
                    if (line.contains("export function ") || line.contains("export class "))
                        && !line.trim().starts_with("/**")
                    {
                        issues.push(format!("Missing JSDoc comment: {}", line.trim()));
                    }
                }
            }
            "python" | "py" => {
                for line in content.lines() {
                    if (line.trim().starts_with("def ") || line.trim().starts_with("class "))
                        && !line.trim().starts_with("def _")
                        && !line.trim().starts_with("class _")
                    {
                        issues.push(format!("Missing docstring: {}", line.trim()));
                    }
                }
            }
            _ => {}
        }

        issues
    }

    /// Checks if code has proper error handling
    pub fn check_error_handling(&self, content: &str, language: &str) -> Vec<String> {
        let mut issues = Vec::new();

        match language {
            "rust" => {
                // Check for unwrap() calls
                for (idx, line) in content.lines().enumerate() {
                    if line.contains(".unwrap()") {
                        issues.push(format!("Line {}: Unsafe unwrap() call", idx + 1));
                    }
                }
            }
            "typescript" | "ts" => {
                // Check for missing error handling
                for (idx, line) in content.lines().enumerate() {
                    if line.contains("throw ") && !line.contains("Error") {
                        issues.push(format!(
                            "Line {}: Generic throw without Error type",
                            idx + 1
                        ));
                    }
                }
            }
            "python" | "py" => {
                // Check for bare except
                for (idx, line) in content.lines().enumerate() {
                    if line.trim() == "except:" {
                        issues.push(format!("Line {}: Bare except clause", idx + 1));
                    }
                }
            }
            _ => {}
        }

        issues
    }
}

impl Default for CodeQualityEnforcer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_rust_doc_comments() {
        let enforcer = CodeQualityEnforcer::new();
        let content = "pub fn hello() {\n    println!(\"Hello\");\n}";
        let enhanced = enforcer.add_rust_doc_comments(content);

        assert!(enhanced.contains("///"));
        assert!(enhanced.contains("hello"));
    }

    #[test]
    fn test_add_typescript_doc_comments() {
        let enforcer = CodeQualityEnforcer::new();
        let content = "export function hello() {\n    console.log(\"Hello\");\n}";
        let enhanced = enforcer.add_typescript_doc_comments(content);

        assert!(enhanced.contains("/**"));
        assert!(enhanced.contains("hello"));
    }

    #[test]
    fn test_add_python_doc_comments() {
        let enforcer = CodeQualityEnforcer::new();
        let content = "def hello():\n    print(\"Hello\")";
        let enhanced = enforcer.add_python_doc_comments(content);

        assert!(enhanced.contains("\"\"\""));
        assert!(enhanced.contains("hello"));
    }

    #[test]
    fn test_check_doc_comments_rust() {
        let enforcer = CodeQualityEnforcer::new();
        let content = "pub fn hello() {}";
        let issues = enforcer.check_doc_comments(content, "rust");

        assert!(!issues.is_empty());
        assert!(issues[0].contains("Missing doc comment"));
    }

    #[test]
    fn test_check_error_handling_rust() {
        let enforcer = CodeQualityEnforcer::new();
        let content = "let x = result.unwrap();";
        let issues = enforcer.check_error_handling(content, "rust");

        assert!(!issues.is_empty());
        assert!(issues[0].contains("unwrap"));
    }

    #[test]
    fn test_enforce_file_rust() {
        let enforcer = CodeQualityEnforcer::new();
        let file = GeneratedFile {
            path: "src/main.rs".to_string(),
            content: "pub fn hello() {}".to_string(),
            language: "rust".to_string(),
        };

        let enhanced = enforcer.enforce_file(&file).unwrap();
        assert!(enhanced.content.contains("///"));
    }
}
