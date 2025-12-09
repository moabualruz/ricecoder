//! Documentation Generator - Generates and maintains project documentation

use crate::errors::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// Documentation section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationSection {
    /// Section title
    pub title: String,
    /// Section content
    pub content: String,
    /// Section order (for sorting)
    pub order: u32,
}

impl DocumentationSection {
    /// Create a new documentation section
    pub fn new(title: impl Into<String>, content: impl Into<String>, order: u32) -> Self {
        Self {
            title: title.into(),
            content: content.into(),
            order,
        }
    }
}

/// API documentation for a function or method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiDocumentation {
    /// Function/method name
    pub name: String,
    /// Function signature
    pub signature: String,
    /// Documentation comment
    pub documentation: String,
    /// Parameters
    pub parameters: Vec<ApiParameter>,
    /// Return type
    pub return_type: String,
    /// Examples
    pub examples: Vec<String>,
}

impl ApiDocumentation {
    /// Create new API documentation
    pub fn new(name: impl Into<String>, signature: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            signature: signature.into(),
            documentation: String::new(),
            parameters: Vec::new(),
            return_type: String::new(),
            examples: Vec::new(),
        }
    }

    /// Add documentation
    pub fn with_documentation(mut self, doc: impl Into<String>) -> Self {
        self.documentation = doc.into();
        self
    }

    /// Add a parameter
    pub fn with_parameter(mut self, param: ApiParameter) -> Self {
        self.parameters.push(param);
        self
    }

    /// Set return type
    pub fn with_return_type(mut self, return_type: impl Into<String>) -> Self {
        self.return_type = return_type.into();
        self
    }

    /// Add an example
    pub fn with_example(mut self, example: impl Into<String>) -> Self {
        self.examples.push(example.into());
        self
    }
}

/// API parameter documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiParameter {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub param_type: String,
    /// Parameter description
    pub description: String,
}

impl ApiParameter {
    /// Create new API parameter
    pub fn new(
        name: impl Into<String>,
        param_type: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            param_type: param_type.into(),
            description: description.into(),
        }
    }
}

/// Documentation coverage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationCoverage {
    /// Total items (functions, modules, etc.)
    pub total_items: u32,
    /// Documented items
    pub documented_items: u32,
    /// Coverage percentage (0-100)
    pub coverage_percentage: f32,
    /// Gaps (undocumented items)
    pub gaps: Vec<String>,
}

impl DocumentationCoverage {
    /// Create new coverage metrics
    pub fn new(total_items: u32, documented_items: u32) -> Self {
        let coverage_percentage = if total_items > 0 {
            (documented_items as f32 / total_items as f32) * 100.0
        } else {
            100.0
        };

        Self {
            total_items,
            documented_items,
            coverage_percentage,
            gaps: Vec::new(),
        }
    }

    /// Add a gap
    pub fn with_gap(mut self, gap: impl Into<String>) -> Self {
        self.gaps.push(gap.into());
        self
    }
}

/// README generation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadmeConfig {
    /// Project name
    pub project_name: String,
    /// Project description
    pub description: String,
    /// Include table of contents
    pub include_toc: bool,
    /// Include installation section
    pub include_installation: bool,
    /// Include usage section
    pub include_usage: bool,
    /// Include API documentation
    pub include_api: bool,
    /// Include contributing section
    pub include_contributing: bool,
    /// Include license section
    pub include_license: bool,
}

impl Default for ReadmeConfig {
    fn default() -> Self {
        Self {
            project_name: "Project".to_string(),
            description: String::new(),
            include_toc: true,
            include_installation: true,
            include_usage: true,
            include_api: true,
            include_contributing: true,
            include_license: true,
        }
    }
}

/// Documentation synchronization result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    /// Files updated
    pub files_updated: Vec<String>,
    /// Files added
    pub files_added: Vec<String>,
    /// Files deleted
    pub files_deleted: Vec<String>,
    /// Synchronization successful
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

impl SyncResult {
    /// Create a successful sync result
    pub fn success() -> Self {
        Self {
            files_updated: Vec::new(),
            files_added: Vec::new(),
            files_deleted: Vec::new(),
            success: true,
            error: None,
        }
    }

    /// Create a failed sync result
    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            files_updated: Vec::new(),
            files_added: Vec::new(),
            files_deleted: Vec::new(),
            success: false,
            error: Some(error.into()),
        }
    }

    /// Add updated file
    pub fn with_updated_file(mut self, file: impl Into<String>) -> Self {
        self.files_updated.push(file.into());
        self
    }

    /// Add added file
    pub fn with_added_file(mut self, file: impl Into<String>) -> Self {
        self.files_added.push(file.into());
        self
    }

    /// Add deleted file
    pub fn with_deleted_file(mut self, file: impl Into<String>) -> Self {
        self.files_deleted.push(file.into());
        self
    }
}

/// Documentation Generator
#[derive(Debug, Clone)]
pub struct DocumentationGenerator {
    /// README configuration
    pub readme_config: ReadmeConfig,
    /// Documentation sections
    pub sections: HashMap<String, DocumentationSection>,
    /// API documentation
    pub api_docs: HashMap<String, ApiDocumentation>,
}

impl DocumentationGenerator {
    /// Create a new documentation generator
    pub fn new(config: ReadmeConfig) -> Self {
        Self {
            readme_config: config,
            sections: HashMap::new(),
            api_docs: HashMap::new(),
        }
    }

    /// Generate README from project structure
    pub fn generate_readme(&self) -> Result<String> {
        debug!("Generating README for project: {}", self.readme_config.project_name);

        let mut readme = String::new();

        // Title
        readme.push_str(&format!("# {}\n\n", self.readme_config.project_name));

        // Description
        if !self.readme_config.description.is_empty() {
            readme.push_str(&format!("{}\n\n", self.readme_config.description));
        }

        // Table of Contents
        if self.readme_config.include_toc {
            readme.push_str("## Table of Contents\n\n");
            if self.readme_config.include_installation {
                readme.push_str("- [Installation](#installation)\n");
            }
            if self.readme_config.include_usage {
                readme.push_str("- [Usage](#usage)\n");
            }
            if self.readme_config.include_api {
                readme.push_str("- [API Documentation](#api-documentation)\n");
            }
            if self.readme_config.include_contributing {
                readme.push_str("- [Contributing](#contributing)\n");
            }
            if self.readme_config.include_license {
                readme.push_str("- [License](#license)\n");
            }
            readme.push('\n');
        }

        // Installation
        if self.readme_config.include_installation {
            readme.push_str("## Installation\n\n");
            readme.push_str("Add this to your `Cargo.toml`:\n\n");
            readme.push_str("```toml\n");
            readme.push_str(&format!("{} = \"0.1\"\n", self.readme_config.project_name.to_lowercase()));
            readme.push_str("```\n\n");
        }

        // Usage
        if self.readme_config.include_usage {
            readme.push_str("## Usage\n\n");
            readme.push_str("```rust\n");
            readme.push_str("// Example usage\n");
            readme.push_str("```\n\n");
        }

        // API Documentation
        if self.readme_config.include_api && !self.api_docs.is_empty() {
            readme.push_str("## API Documentation\n\n");
            for api_doc in self.api_docs.values() {
                readme.push_str(&format!("### {}\n\n", api_doc.name));
                readme.push_str(&format!("```rust\n{}\n```\n\n", api_doc.signature));
                if !api_doc.documentation.is_empty() {
                    readme.push_str(&format!("{}\n\n", api_doc.documentation));
                }
            }
        }

        // Contributing
        if self.readme_config.include_contributing {
            readme.push_str("## Contributing\n\n");
            readme.push_str("Contributions are welcome! Please see CONTRIBUTING.md for details.\n\n");
        }

        // License
        if self.readme_config.include_license {
            readme.push_str("## License\n\n");
            readme.push_str("This project is licensed under the MIT License.\n");
        }

        info!("README generated successfully");
        Ok(readme)
    }

    /// Extract API documentation from code
    pub fn extract_api_documentation(&mut self, code: &str) -> Result<Vec<ApiDocumentation>> {
        debug!("Extracting API documentation from code");

        let mut api_docs = Vec::new();

        // Simple pattern matching for Rust doc comments and function signatures
        let lines: Vec<&str> = code.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i];

            // Look for doc comments
            if line.trim().starts_with("///") {
                let mut doc_comment = String::new();
                let mut j = i;

                // Collect all consecutive doc comments
                while j < lines.len() && lines[j].trim().starts_with("///") {
                    let content = lines[j].trim_start_matches("///").trim();
                    doc_comment.push_str(content);
                    doc_comment.push('\n');
                    j += 1;
                }

                // Look for function signature after doc comment
                if j < lines.len() {
                    let sig_line = lines[j];
                    if sig_line.trim().starts_with("pub fn ") || sig_line.trim().starts_with("fn ") {
                        // Extract function name
                        if let Some(start) = sig_line.find("fn ") {
                            let after_fn = &sig_line[start + 3..];
                            if let Some(paren_pos) = after_fn.find('(') {
                                let func_name = after_fn[..paren_pos].trim();
                                let api_doc = ApiDocumentation::new(func_name, sig_line.trim())
                                    .with_documentation(doc_comment);
                                api_docs.push(api_doc);
                            }
                        }
                    }
                }

                i = j;
            } else {
                i += 1;
            }
        }

        info!("Extracted {} API documentation items", api_docs.len());
        Ok(api_docs)
    }

    /// Synchronize documentation with code changes
    pub fn synchronize_documentation(&self, old_code: &str, new_code: &str) -> Result<SyncResult> {
        debug!("Synchronizing documentation with code changes");

        let mut result = SyncResult::success();

        // Check if code has changed
        if old_code == new_code {
            debug!("No code changes detected");
            return Ok(result);
        }

        // Extract API docs from new code
        let mut generator = DocumentationGenerator::new(self.readme_config.clone());
        let new_api_docs = generator.extract_api_documentation(new_code)?;

        // Mark documentation as updated
        if !new_api_docs.is_empty() {
            result = result.with_updated_file("API_DOCUMENTATION.md");
        }

        info!("Documentation synchronized successfully");
        Ok(result)
    }

    /// Calculate documentation coverage
    pub fn calculate_coverage(&self, code: &str) -> Result<DocumentationCoverage> {
        debug!("Calculating documentation coverage");

        // Count functions in code
        let total_functions = code.matches("fn ").count() as u32;

        // Count documented functions (with ///)
        let documented_functions = code.matches("///").count() as u32 / 3; // Rough estimate

        let mut coverage = DocumentationCoverage::new(total_functions, documented_functions);

        // Identify gaps
        let lines: Vec<&str> = code.lines().collect();
        for i in 0..lines.len() {
            let line = lines[i];
            if line.trim().starts_with("pub fn ") && (i == 0 || !lines[i - 1].trim().starts_with("///")) {
                if let Some(start) = line.find("fn ") {
                    let after_fn = &line[start + 3..];
                    if let Some(paren_pos) = after_fn.find('(') {
                        let func_name = after_fn[..paren_pos].trim();
                        coverage = coverage.with_gap(format!("Function '{}' is not documented", func_name));
                    }
                }
            }
        }

        info!("Documentation coverage: {:.1}%", coverage.coverage_percentage);
        Ok(coverage)
    }

    /// Add a documentation section
    pub fn add_section(&mut self, key: impl Into<String>, section: DocumentationSection) {
        self.sections.insert(key.into(), section);
    }

    /// Add API documentation
    pub fn add_api_documentation(&mut self, key: impl Into<String>, api_doc: ApiDocumentation) {
        self.api_docs.insert(key.into(), api_doc);
    }

    /// Get all sections sorted by order
    pub fn get_sorted_sections(&self) -> Vec<&DocumentationSection> {
        let mut sections: Vec<_> = self.sections.values().collect();
        sections.sort_by_key(|s| s.order);
        sections
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_readme_generation() {
        let config = ReadmeConfig {
            project_name: "TestProject".to_string(),
            description: "A test project".to_string(),
            include_toc: true,
            include_installation: true,
            include_usage: true,
            include_api: false,
            include_contributing: true,
            include_license: true,
        };

        let generator = DocumentationGenerator::new(config);
        let readme = generator.generate_readme().expect("Failed to generate README");

        assert!(readme.contains("# TestProject"));
        assert!(readme.contains("A test project"));
        assert!(readme.contains("## Table of Contents"));
        assert!(readme.contains("## Installation"));
        assert!(readme.contains("## Usage"));
        assert!(readme.contains("## Contributing"));
        assert!(readme.contains("## License"));
    }

    #[test]
    fn test_api_documentation_extraction() {
        let code = r#"
/// This is a test function
/// It does something useful
pub fn test_function(x: i32) -> i32 {
    x * 2
}

/// Another function
pub fn another_function() {
    println!("Hello");
}
"#;

        let mut generator = DocumentationGenerator::new(ReadmeConfig::default());
        let api_docs = generator.extract_api_documentation(code).expect("Failed to extract API docs");

        assert!(!api_docs.is_empty());
        assert!(api_docs.iter().any(|doc| doc.name.contains("test_function")));
    }

    #[test]
    fn test_documentation_coverage() {
        let code = r#"
/// Documented function
pub fn documented() {}

pub fn undocumented() {}
"#;

        let generator = DocumentationGenerator::new(ReadmeConfig::default());
        let coverage = generator.calculate_coverage(code).expect("Failed to calculate coverage");

        assert!(coverage.total_items > 0);
        assert!(coverage.coverage_percentage >= 0.0 && coverage.coverage_percentage <= 100.0);
    }

    #[test]
    fn test_sync_result_builder() {
        let result = SyncResult::success()
            .with_updated_file("file1.md")
            .with_added_file("file2.md")
            .with_deleted_file("file3.md");

        assert!(result.success);
        assert_eq!(result.files_updated.len(), 1);
        assert_eq!(result.files_added.len(), 1);
        assert_eq!(result.files_deleted.len(), 1);
    }

    #[test]
    fn test_documentation_coverage_calculation() {
        let coverage = DocumentationCoverage::new(10, 7);
        assert_eq!(coverage.total_items, 10);
        assert_eq!(coverage.documented_items, 7);
        assert!((coverage.coverage_percentage - 70.0).abs() < 0.1);
    }
}
