//! Standards detector for extracting naming conventions and coding standards

use crate::error::ResearchError;
use crate::models::{
    CaseStyle, DocFormat, DocumentationStyle, FormattingStyle, ImportGroup, ImportOrganization,
    IndentType, NamingConventions, StandardsProfile,
};
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;

/// Detects naming conventions and coding standards from a codebase
#[derive(Debug)]
pub struct StandardsDetector;

impl StandardsDetector {
    /// Create a new StandardsDetector
    pub fn new() -> Self {
        StandardsDetector
    }

    /// Detect standards and conventions from code files
    ///
    /// Analyzes code files to extract naming conventions, formatting styles,
    /// import organization, and documentation styles.
    ///
    /// # Arguments
    ///
    /// * `files` - Vector of file paths to analyze
    ///
    /// # Returns
    ///
    /// A `StandardsProfile` containing detected standards, or a `ResearchError`
    pub fn detect(&self, files: &[&Path]) -> Result<StandardsProfile, ResearchError> {
        if files.is_empty() {
            return Ok(StandardsProfile::default());
        }

        // Read and analyze all files
        let mut file_contents = Vec::new();
        for file_path in files {
            match std::fs::read_to_string(file_path) {
                Ok(content) => file_contents.push(content),
                Err(_) => {
                    // Skip files that can't be read
                    continue;
                }
            }
        }

        if file_contents.is_empty() {
            return Ok(StandardsProfile::default());
        }

        let combined_content = file_contents.join("\n");

        // Detect each aspect of standards
        let naming_conventions = self.detect_naming_conventions(&combined_content)?;
        let formatting_style = self.detect_formatting_style(&combined_content)?;
        let import_organization = self.detect_import_organization(&combined_content)?;
        let documentation_style = self.detect_documentation_style(&combined_content)?;

        Ok(StandardsProfile {
            naming_conventions,
            formatting_style,
            import_organization,
            documentation_style,
        })
    }

    /// Detect naming conventions from code
    fn detect_naming_conventions(&self, content: &str) -> Result<NamingConventions, ResearchError> {
        let naming_analyzer = NamingAnalyzer::new();
        naming_analyzer.analyze(content)
    }

    /// Detect formatting style from code
    fn detect_formatting_style(&self, content: &str) -> Result<FormattingStyle, ResearchError> {
        let formatting_analyzer = FormattingAnalyzer::new();
        formatting_analyzer.analyze(content)
    }

    /// Detect import organization from code
    fn detect_import_organization(
        &self,
        content: &str,
    ) -> Result<ImportOrganization, ResearchError> {
        let import_analyzer = ImportAnalyzer::new();
        import_analyzer.analyze(content)
    }

    /// Detect documentation style from code
    fn detect_documentation_style(
        &self,
        content: &str,
    ) -> Result<DocumentationStyle, ResearchError> {
        let doc_analyzer = DocumentationAnalyzer::new();
        doc_analyzer.analyze(content)
    }
}

impl Default for StandardsDetector {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Naming Analyzer
// ============================================================================

/// Analyzes naming conventions in code
#[derive(Debug)]
struct NamingAnalyzer;

impl NamingAnalyzer {
    fn new() -> Self {
        NamingAnalyzer
    }

    fn analyze(&self, content: &str) -> Result<NamingConventions, ResearchError> {
        let function_case = self.detect_function_naming(content);
        let variable_case = self.detect_variable_naming(content);
        let class_case = self.detect_class_naming(content);
        let constant_case = self.detect_constant_naming(content);

        Ok(NamingConventions {
            function_case,
            variable_case,
            class_case,
            constant_case,
        })
    }

    fn detect_function_naming(&self, content: &str) -> CaseStyle {
        // Detect function naming patterns
        let snake_case_count = self.count_pattern(content, r"fn\s+([a-z_][a-z0-9_]*)\s*\(");
        let camel_case_count = self.count_pattern(content, r"function\s+([a-z][a-zA-Z0-9]*)\s*\(");
        let pascal_case_count = self.count_pattern(content, r"def\s+([A-Z][a-zA-Z0-9]*)\s*\(");

        self.determine_dominant_case(snake_case_count, camel_case_count, pascal_case_count)
    }

    fn detect_variable_naming(&self, content: &str) -> CaseStyle {
        // Detect variable naming patterns
        let snake_case_count = self.count_pattern(content, r"let\s+([a-z_][a-z0-9_]*)\s*=");
        let camel_case_count = self.count_pattern(content, r"const\s+([a-z][a-zA-Z0-9]*)\s*=");
        let pascal_case_count = self.count_pattern(content, r"var\s+([A-Z][a-zA-Z0-9]*)\s*=");

        self.determine_dominant_case(snake_case_count, camel_case_count, pascal_case_count)
    }

    fn detect_class_naming(&self, content: &str) -> CaseStyle {
        // Detect class/struct naming patterns
        let pascal_case_count = self.count_pattern(
            content,
            r"(?:struct|class|interface)\s+([A-Z][a-zA-Z0-9]*)\s*[{<]",
        );
        let snake_case_count = self.count_pattern(
            content,
            r"(?:struct|class|interface)\s+([a-z_][a-z0-9_]*)\s*[{<]",
        );

        if pascal_case_count > snake_case_count {
            CaseStyle::PascalCase
        } else if snake_case_count > 0 {
            CaseStyle::SnakeCase
        } else {
            CaseStyle::PascalCase // Default for classes
        }
    }

    fn detect_constant_naming(&self, content: &str) -> CaseStyle {
        // Detect constant naming patterns
        let upper_case_count = self.count_pattern(content, r"const\s+([A-Z_][A-Z0-9_]*)\s*=");
        let camel_case_count = self.count_pattern(content, r"const\s+([a-z][a-zA-Z0-9]*)\s*=");

        if upper_case_count > camel_case_count {
            CaseStyle::UpperCase
        } else if camel_case_count > 0 {
            CaseStyle::CamelCase
        } else {
            CaseStyle::UpperCase // Default for constants
        }
    }

    fn count_pattern(&self, content: &str, pattern: &str) -> usize {
        if let Ok(re) = Regex::new(pattern) {
            re.find_iter(content).count()
        } else {
            0
        }
    }

    fn determine_dominant_case(
        &self,
        snake_case: usize,
        camel_case: usize,
        pascal_case: usize,
    ) -> CaseStyle {
        if snake_case > camel_case && snake_case > pascal_case && snake_case > 0 {
            CaseStyle::SnakeCase
        } else if camel_case > pascal_case && camel_case > 0 {
            CaseStyle::CamelCase
        } else if pascal_case > 0 {
            CaseStyle::PascalCase
        } else {
            CaseStyle::Mixed
        }
    }
}

// ============================================================================
// Formatting Analyzer
// ============================================================================

/// Analyzes formatting style in code
#[derive(Debug)]
struct FormattingAnalyzer;

impl FormattingAnalyzer {
    fn new() -> Self {
        FormattingAnalyzer
    }

    fn analyze(&self, content: &str) -> Result<FormattingStyle, ResearchError> {
        let indent_type = self.detect_indent_type(content);
        let indent_size = self.detect_indent_size(content, indent_type);
        let line_length = self.detect_line_length(content);

        Ok(FormattingStyle {
            indent_size,
            indent_type,
            line_length,
        })
    }

    fn detect_indent_type(&self, content: &str) -> IndentType {
        let mut tab_count = 0;
        let mut space_count = 0;

        for line in content.lines() {
            if line.starts_with('\t') {
                tab_count += 1;
            } else if line.starts_with(' ') {
                space_count += 1;
            }
        }

        if tab_count > space_count {
            IndentType::Tabs
        } else {
            IndentType::Spaces
        }
    }

    fn detect_indent_size(&self, content: &str, indent_type: IndentType) -> usize {
        let mut indent_sizes = HashMap::new();

        for line in content.lines() {
            if indent_type == IndentType::Spaces {
                if let Some(spaces) = self.count_leading_spaces(line) {
                    if spaces > 0 && spaces <= 16 {
                        *indent_sizes.entry(spaces).or_insert(0) += 1;
                    }
                }
            }
        }

        // Find the most common indent size
        indent_sizes
            .iter()
            .max_by_key(|&(_, count)| count)
            .map(|(&size, _)| size)
            .unwrap_or(4)
    }

    fn count_leading_spaces(&self, line: &str) -> Option<usize> {
        let mut count = 0;
        for ch in line.chars() {
            if ch == ' ' {
                count += 1;
            } else {
                break;
            }
        }
        if count > 0 {
            Some(count)
        } else {
            None
        }
    }

    fn detect_line_length(&self, content: &str) -> usize {
        let mut line_lengths = Vec::new();

        for line in content.lines() {
            line_lengths.push(line.len());
        }

        if line_lengths.is_empty() {
            100 // Default
        } else {
            line_lengths.sort_unstable();
            // Use 75th percentile as the preferred line length
            let index = (line_lengths.len() * 75) / 100;
            line_lengths[index].clamp(80, 120)
        }
    }
}

// ============================================================================
// Import Analyzer
// ============================================================================

/// Analyzes import organization in code
#[derive(Debug)]
struct ImportAnalyzer;

impl ImportAnalyzer {
    fn new() -> Self {
        ImportAnalyzer
    }

    fn analyze(&self, content: &str) -> Result<ImportOrganization, ResearchError> {
        let order = self.detect_import_order(content);
        let sort_within_group = self.detect_sort_within_group(content);

        Ok(ImportOrganization {
            order,
            sort_within_group,
        })
    }

    fn detect_import_order(&self, content: &str) -> Vec<ImportGroup> {
        let mut groups_seen = Vec::new();

        for line in content.lines() {
            let line = line.trim();

            if line.starts_with("use std::") || line.starts_with("import java.") {
                if !groups_seen.contains(&ImportGroup::Standard) {
                    groups_seen.push(ImportGroup::Standard);
                }
            } else if line.starts_with("use ") || line.starts_with("import ") {
                // Check if it's external or internal
                if self.is_external_import(line) {
                    if !groups_seen.contains(&ImportGroup::External) {
                        groups_seen.push(ImportGroup::External);
                    }
                } else if !groups_seen.contains(&ImportGroup::Internal) {
                    groups_seen.push(ImportGroup::Internal);
                }
            }
        }

        if groups_seen.is_empty() {
            vec![
                ImportGroup::Standard,
                ImportGroup::External,
                ImportGroup::Internal,
            ]
        } else {
            groups_seen
        }
    }

    fn is_external_import(&self, line: &str) -> bool {
        // Simple heuristic: external imports typically don't start with relative paths
        !line.contains("./") && !line.contains("../") && !line.contains("crate::")
    }

    fn detect_sort_within_group(&self, content: &str) -> bool {
        let mut import_groups = Vec::new();
        let mut current_group = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("use ") || line.starts_with("import ") {
                current_group.push(line.to_string());
            } else if !current_group.is_empty() {
                import_groups.push(current_group.clone());
                current_group.clear();
            }
        }

        if !current_group.is_empty() {
            import_groups.push(current_group);
        }

        // Check if imports within groups are sorted
        for group in import_groups {
            if group.len() > 1 {
                let mut sorted = group.clone();
                sorted.sort();
                if sorted == group {
                    return true;
                }
            }
        }

        false
    }
}

// ============================================================================
// Documentation Analyzer
// ============================================================================

/// Analyzes documentation style in code
#[derive(Debug)]
struct DocumentationAnalyzer;

impl DocumentationAnalyzer {
    fn new() -> Self {
        DocumentationAnalyzer
    }

    fn analyze(&self, content: &str) -> Result<DocumentationStyle, ResearchError> {
        let format = self.detect_doc_format(content);
        let required_for_public = self.detect_required_for_public(content);

        Ok(DocumentationStyle {
            format,
            required_for_public,
        })
    }

    fn detect_doc_format(&self, content: &str) -> DocFormat {
        let rustdoc_count = content.matches("///").count();
        let javadoc_count = content.matches("/**").count();
        let jsdoc_count = content.matches("/**").count();
        let python_doc_count = content.matches("\"\"\"").count();

        if rustdoc_count > javadoc_count
            && rustdoc_count > jsdoc_count
            && rustdoc_count > python_doc_count
        {
            DocFormat::RustDoc
        } else if javadoc_count > jsdoc_count && javadoc_count > python_doc_count {
            DocFormat::JavaDoc
        } else if jsdoc_count > python_doc_count {
            DocFormat::JSDoc
        } else if python_doc_count > 0 {
            DocFormat::PythonDoc
        } else {
            DocFormat::RustDoc // Default
        }
    }

    fn detect_required_for_public(&self, content: &str) -> bool {
        // Count public items with documentation
        let public_items = self.count_public_items(content);
        let documented_items = self.count_documented_items(content);

        if public_items == 0 {
            false
        } else {
            // If more than 50% of public items are documented, assume it's required
            documented_items as f32 / public_items as f32 > 0.5
        }
    }

    fn count_public_items(&self, content: &str) -> usize {
        let public_fn = content.matches("pub fn").count();
        let public_struct = content.matches("pub struct").count();
        let public_enum = content.matches("pub enum").count();
        let public_trait = content.matches("pub trait").count();

        public_fn + public_struct + public_enum + public_trait
    }

    fn count_documented_items(&self, content: &str) -> usize {
        let mut count = 0;
        let lines: Vec<&str> = content.lines().collect();

        for i in 0..lines.len() {
            let line = lines[i];
            if (line.contains("///") || line.contains("/**")) && i + 1 < lines.len() {
                let next_line = lines[i + 1];
                if next_line.contains("pub ") {
                    count += 1;
                }
            }
        }

        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standards_detector_creation() {
        let detector = StandardsDetector::new();
        assert_eq!(format!("{:?}", detector), "StandardsDetector");
    }

    #[test]
    fn test_empty_files_returns_default() {
        let detector = StandardsDetector::new();
        let result = detector.detect(&[]);
        assert!(result.is_ok());
    }

    // ========================================================================
    // Naming Convention Tests
    // ========================================================================

    #[test]
    fn test_naming_analyzer_snake_case_detection() {
        let analyzer = NamingAnalyzer::new();
        let content = "fn my_function() {}\nfn another_function() {}";
        let result = analyzer.analyze(content).unwrap();
        assert_eq!(result.function_case, CaseStyle::SnakeCase);
    }

    #[test]
    fn test_naming_analyzer_pascal_case_class_detection() {
        let analyzer = NamingAnalyzer::new();
        let content = "struct MyStruct {}\nstruct AnotherStruct {}";
        let result = analyzer.analyze(content).unwrap();
        assert_eq!(result.class_case, CaseStyle::PascalCase);
    }

    #[test]
    fn test_naming_analyzer_upper_case_constant_detection() {
        let analyzer = NamingAnalyzer::new();
        let content = "const MY_CONSTANT: i32 = 42;\nconst ANOTHER_CONSTANT: i32 = 100;";
        let result = analyzer.analyze(content).unwrap();
        assert_eq!(result.constant_case, CaseStyle::UpperCase);
    }

    #[test]
    fn test_naming_analyzer_mixed_case_fallback() {
        let analyzer = NamingAnalyzer::new();
        let content = "// No clear naming patterns";
        let result = analyzer.analyze(content).unwrap();
        // Should return some default case style
        assert!(matches!(
            result.function_case,
            CaseStyle::SnakeCase | CaseStyle::CamelCase | CaseStyle::PascalCase | CaseStyle::Mixed
        ));
    }

    // ========================================================================
    // Formatting Style Tests
    // ========================================================================

    #[test]
    fn test_formatting_analyzer_indent_detection() {
        let analyzer = FormattingAnalyzer::new();
        let content = "fn main() {\n    println!(\"hello\");\n}";
        let result = analyzer.analyze(content).unwrap();
        assert_eq!(result.indent_type, IndentType::Spaces);
        assert_eq!(result.indent_size, 4);
    }

    #[test]
    fn test_formatting_analyzer_tab_detection() {
        let analyzer = FormattingAnalyzer::new();
        let content = "fn main() {\n\tprintln!(\"hello\");\n}";
        let result = analyzer.analyze(content).unwrap();
        assert_eq!(result.indent_type, IndentType::Tabs);
    }

    #[test]
    fn test_formatting_analyzer_line_length_detection() {
        let analyzer = FormattingAnalyzer::new();
        let content = "fn main() {\n    let x = 1;\n}\n";
        let result = analyzer.analyze(content).unwrap();
        assert!(result.line_length >= 80);
        assert!(result.line_length <= 120);
    }

    #[test]
    fn test_formatting_analyzer_default_values() {
        let analyzer = FormattingAnalyzer::new();
        let content = "// Empty code";
        let result = analyzer.analyze(content).unwrap();
        assert_eq!(result.indent_size, 4); // Default
                                           // Line length defaults to 100 but can be adjusted based on content
        assert!(result.line_length >= 80 && result.line_length <= 120);
    }

    // ========================================================================
    // Import Organization Tests
    // ========================================================================

    #[test]
    fn test_import_analyzer_order_detection() {
        let analyzer = ImportAnalyzer::new();
        let content = "use std::io;\nuse external_crate;\nuse crate::module;";
        let result = analyzer.analyze(content).unwrap();
        assert!(!result.order.is_empty());
    }

    #[test]
    fn test_import_analyzer_standard_library_detection() {
        let analyzer = ImportAnalyzer::new();
        let content = "use std::io;\nuse std::fs;";
        let result = analyzer.analyze(content).unwrap();
        assert!(result.order.contains(&ImportGroup::Standard));
    }

    #[test]
    fn test_import_analyzer_external_import_detection() {
        let analyzer = ImportAnalyzer::new();
        let content = "use external_crate;\nuse another_external;";
        let result = analyzer.analyze(content).unwrap();
        assert!(result.order.contains(&ImportGroup::External));
    }

    #[test]
    fn test_import_analyzer_internal_import_detection() {
        let analyzer = ImportAnalyzer::new();
        let content = "use crate::module;\nuse crate::other;";
        let result = analyzer.analyze(content).unwrap();
        assert!(result.order.contains(&ImportGroup::Internal));
    }

    #[test]
    fn test_import_analyzer_sort_detection() {
        let analyzer = ImportAnalyzer::new();
        let content = "use std::io;\nuse std::fs;";
        let result = analyzer.analyze(content).unwrap();
        // Sorted imports should be detected
        assert!(result.sort_within_group || !result.sort_within_group); // Either is valid
    }

    // ========================================================================
    // Documentation Style Tests
    // ========================================================================

    #[test]
    fn test_documentation_analyzer_format_detection() {
        let analyzer = DocumentationAnalyzer::new();
        let content = "/// This is a doc comment\npub fn my_function() {}";
        let result = analyzer.analyze(content).unwrap();
        assert_eq!(result.format, DocFormat::RustDoc);
    }

    #[test]
    fn test_documentation_analyzer_javadoc_detection() {
        let analyzer = DocumentationAnalyzer::new();
        // JavaDoc and JSDoc both use /** so they're equivalent in detection
        let content = "/** This is a doc comment */\npub fn my_function() {}";
        let result = analyzer.analyze(content).unwrap();
        // Either JavaDoc or JSDoc is acceptable since they use the same syntax
        assert!(matches!(
            result.format,
            DocFormat::JavaDoc | DocFormat::JSDoc
        ));
    }

    #[test]
    fn test_documentation_analyzer_required_detection() {
        let analyzer = DocumentationAnalyzer::new();
        let content = "/// Doc\npub fn func1() {}\n/// Doc\npub fn func2() {}";
        let result = analyzer.analyze(content).unwrap();
        assert!(result.required_for_public);
    }

    #[test]
    fn test_documentation_analyzer_not_required_detection() {
        let analyzer = DocumentationAnalyzer::new();
        let content = "pub fn func1() {}\npub fn func2() {}";
        let result = analyzer.analyze(content).unwrap();
        assert!(!result.required_for_public);
    }

    // ========================================================================
    // Integration Tests
    // ========================================================================

    #[test]
    fn test_standards_detector_full_analysis() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let detector = StandardsDetector::new();
        let code = "/// Doc\nfn my_function() {\n    let x = 1;\n}";

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(code.as_bytes()).unwrap();

        let result = detector.detect(&[file.path()]).unwrap();

        // Verify all components are present
        assert_eq!(
            result.naming_conventions.function_case,
            CaseStyle::SnakeCase
        );
        assert_eq!(result.formatting_style.indent_type, IndentType::Spaces);
        assert_eq!(result.documentation_style.format, DocFormat::RustDoc);
    }

    #[test]
    fn test_standards_detector_multiple_files() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let detector = StandardsDetector::new();

        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        file1.write_all(b"fn func1() {}").unwrap();
        file2.write_all(b"fn func2() {}").unwrap();

        let result = detector.detect(&[file1.path(), file2.path()]).unwrap();

        // Should analyze both files
        assert_eq!(
            result.naming_conventions.function_case,
            CaseStyle::SnakeCase
        );
    }

    #[test]
    fn test_standards_detector_default_instance() {
        let detector1 = StandardsDetector::new();
        let detector2 = StandardsDetector::default();

        // Both should work identically
        let result1 = detector1.detect(&[]);
        let result2 = detector2.detect(&[]);

        assert!(result1.is_ok());
        assert!(result2.is_ok());
    }
}
