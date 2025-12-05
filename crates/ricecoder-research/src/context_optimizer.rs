//! Context optimization for fitting large files within token budgets

use crate::models::FileContext;
use crate::ResearchError;

/// Optimizes context by summarizing large files to fit token budgets
#[derive(Debug, Clone)]
pub struct ContextOptimizer {
    /// Maximum tokens per file
    max_tokens_per_file: usize,
    /// Minimum tokens to preserve for important sections
    min_important_tokens: usize,
}

impl ContextOptimizer {
    /// Create a new context optimizer
    pub fn new(max_tokens_per_file: usize) -> Self {
        ContextOptimizer {
            max_tokens_per_file,
            min_important_tokens: 100,
        }
    }

    /// Optimize a file to fit within token budget
    pub fn optimize_file(&self, file: &FileContext) -> Result<FileContext, ResearchError> {
        let mut optimized = file.clone();

        if let Some(content) = &file.content {
            let tokens = self.estimate_tokens(content);

            if tokens > self.max_tokens_per_file {
                // Summarize the file
                optimized.content = Some(self.summarize_content(content)?);
            }
        }

        Ok(optimized)
    }

    /// Optimize multiple files
    pub fn optimize_files(
        &self,
        files: Vec<FileContext>,
    ) -> Result<Vec<FileContext>, ResearchError> {
        files
            .into_iter()
            .map(|file| self.optimize_file(&file))
            .collect()
    }

    /// Estimate token count for content (rough approximation: 1 token per 4 characters)
    pub fn estimate_tokens(&self, content: &str) -> usize {
        (content.len() / 4).max(1)
    }

    /// Summarize content to fit within token budget
    fn summarize_content(&self, content: &str) -> Result<String, ResearchError> {
        let lines: Vec<&str> = content.lines().collect();

        if lines.is_empty() {
            return Ok(String::new());
        }

        let mut summary = String::new();

        // Include important sections: imports, type definitions, function signatures
        let mut included_lines = Vec::new();

        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Always include imports
            if trimmed.starts_with("use ") || trimmed.starts_with("import ") {
                included_lines.push((idx, line));
                continue;
            }

            // Include type definitions
            if trimmed.starts_with("pub struct ")
                || trimmed.starts_with("pub enum ")
                || trimmed.starts_with("pub trait ")
                || trimmed.starts_with("pub type ")
                || trimmed.starts_with("struct ")
                || trimmed.starts_with("enum ")
                || trimmed.starts_with("trait ")
                || trimmed.starts_with("type ")
            {
                included_lines.push((idx, line));
                continue;
            }

            // Include function signatures
            if trimmed.starts_with("pub fn ")
                || trimmed.starts_with("pub async fn ")
                || trimmed.starts_with("fn ")
                || trimmed.starts_with("async fn ")
            {
                included_lines.push((idx, line));
                continue;
            }

            // Include comments
            if trimmed.starts_with("//") || trimmed.starts_with("/*") {
                included_lines.push((idx, line));
                continue;
            }
        }

        // If we have important lines, use them
        if !included_lines.is_empty() {
            for (_, line) in included_lines {
                summary.push_str(line);
                summary.push('\n');
            }

            // Add ellipsis to indicate truncation
            summary.push_str("\n// ... (content truncated for context window) ...\n");

            // Check if summary fits within budget
            if self.estimate_tokens(&summary) <= self.max_tokens_per_file {
                return Ok(summary);
            }
        }

        // Fallback: just take first N lines
        let max_lines = (self.max_tokens_per_file * 4) / 50; // Rough estimate
        let mut result = String::new();

        for line in lines.iter().take(max_lines) {
            result.push_str(line);
            result.push('\n');
        }

        if lines.len() > max_lines {
            result.push_str("\n// ... (content truncated for context window) ...\n");
        }

        Ok(result)
    }

    /// Extract key sections from content
    pub fn extract_key_sections(&self, content: &str) -> Vec<String> {
        let mut sections = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        let mut current_section = String::new();
        let mut in_function = false;

        for line in lines {
            let trimmed = line.trim();

            // Start of function
            if trimmed.starts_with("pub fn ")
                || trimmed.starts_with("pub async fn ")
                || trimmed.starts_with("fn ")
                || trimmed.starts_with("async fn ")
            {
                if !current_section.is_empty() {
                    sections.push(current_section.clone());
                    current_section.clear();
                }
                in_function = true;
                current_section.push_str(line);
                current_section.push('\n');
            } else if in_function {
                current_section.push_str(line);
                current_section.push('\n');

                // Simple heuristic: end of function (closing brace at start of line)
                if trimmed == "}" {
                    in_function = false;
                    sections.push(current_section.clone());
                    current_section.clear();
                }
            }
        }

        if !current_section.is_empty() {
            sections.push(current_section);
        }

        sections
    }

    /// Get maximum tokens per file
    pub fn max_tokens_per_file(&self) -> usize {
        self.max_tokens_per_file
    }

    /// Set maximum tokens per file
    pub fn set_max_tokens_per_file(&mut self, max_tokens: usize) {
        self.max_tokens_per_file = max_tokens;
    }

    /// Get minimum important tokens
    pub fn min_important_tokens(&self) -> usize {
        self.min_important_tokens
    }

    /// Set minimum important tokens
    pub fn set_min_important_tokens(&mut self, min_tokens: usize) {
        self.min_important_tokens = min_tokens;
    }
}

impl Default for ContextOptimizer {
    fn default() -> Self {
        ContextOptimizer::new(2048) // Default to 2K tokens per file
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_context_optimizer_creation() {
        let optimizer = ContextOptimizer::new(2048);
        assert_eq!(optimizer.max_tokens_per_file(), 2048);
    }

    #[test]
    fn test_context_optimizer_default() {
        let optimizer = ContextOptimizer::default();
        assert_eq!(optimizer.max_tokens_per_file(), 2048);
    }

    #[test]
    fn test_estimate_tokens() {
        let optimizer = ContextOptimizer::new(2048);
        let content = "x".repeat(400); // 400 chars = ~100 tokens
        let tokens = optimizer.estimate_tokens(&content);
        assert_eq!(tokens, 100);
    }

    #[test]
    fn test_optimize_file_small_content() {
        let optimizer = ContextOptimizer::new(2048);
        let file = FileContext {
            path: PathBuf::from("src/main.rs"),
            relevance: 0.9,
            summary: None,
            content: Some("fn main() {}".to_string()),
        };

        let result = optimizer.optimize_file(&file);
        assert!(result.is_ok());

        let optimized = result.unwrap();
        assert_eq!(optimized.content, file.content);
    }

    #[test]
    fn test_optimize_file_large_content() {
        let optimizer = ContextOptimizer::new(100); // Very small budget

        let large_content = "fn main() {}\n".repeat(100); // Large content
        let file = FileContext {
            path: PathBuf::from("src/main.rs"),
            relevance: 0.9,
            summary: None,
            content: Some(large_content),
        };

        let result = optimizer.optimize_file(&file);
        assert!(result.is_ok());

        let optimized = result.unwrap();
        assert!(optimized.content.is_some());
        // Optimized content should be smaller
        let optimized_tokens = optimizer.estimate_tokens(optimized.content.as_ref().unwrap());
        assert!(optimized_tokens <= 100);
    }

    #[test]
    fn test_summarize_content_with_imports() {
        let optimizer = ContextOptimizer::new(2048);
        let content = "use std::path::PathBuf;\nuse std::collections::HashMap;\n\nfn main() {}\n";

        let result = optimizer.summarize_content(content);
        assert!(result.is_ok());

        let summary = result.unwrap();
        assert!(summary.contains("use std::path::PathBuf"));
        assert!(summary.contains("use std::collections::HashMap"));
    }

    #[test]
    fn test_summarize_content_with_types() {
        let optimizer = ContextOptimizer::new(2048);
        let content = "pub struct MyStruct {\n    field: String,\n}\n\nfn main() {}\n";

        let result = optimizer.summarize_content(content);
        assert!(result.is_ok());

        let summary = result.unwrap();
        assert!(summary.contains("pub struct MyStruct"));
    }

    #[test]
    fn test_extract_key_sections() {
        let optimizer = ContextOptimizer::new(2048);
        let content =
            "fn helper() {\n    println!(\"hello\");\n}\n\nfn main() {\n    helper();\n}\n";

        let sections = optimizer.extract_key_sections(content);
        assert!(!sections.is_empty());
    }

    #[test]
    fn test_optimize_files() {
        let optimizer = ContextOptimizer::new(2048);
        let files = vec![
            FileContext {
                path: PathBuf::from("src/main.rs"),
                relevance: 0.9,
                summary: None,
                content: Some("fn main() {}".to_string()),
            },
            FileContext {
                path: PathBuf::from("src/lib.rs"),
                relevance: 0.8,
                summary: None,
                content: Some("pub fn helper() {}".to_string()),
            },
        ];

        let result = optimizer.optimize_files(files);
        assert!(result.is_ok());

        let optimized = result.unwrap();
        assert_eq!(optimized.len(), 2);
    }

    #[test]
    fn test_set_max_tokens_per_file() {
        let mut optimizer = ContextOptimizer::new(2048);
        optimizer.set_max_tokens_per_file(4096);
        assert_eq!(optimizer.max_tokens_per_file(), 4096);
    }

    #[test]
    fn test_set_min_important_tokens() {
        let mut optimizer = ContextOptimizer::new(2048);
        optimizer.set_min_important_tokens(200);
        assert_eq!(optimizer.min_important_tokens(), 200);
    }
}
