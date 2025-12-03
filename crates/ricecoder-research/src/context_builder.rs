//! Context builder for selecting and prioritizing relevant files for AI providers

use crate::models::{CodeContext, FileContext};
use crate::semantic_index::SemanticIndex;
use crate::ResearchError;

/// Builds context for AI providers by selecting and prioritizing relevant files
#[derive(Debug, Clone)]
pub struct ContextBuilder {
    /// Maximum tokens allowed in context
    max_tokens: usize,
    /// Semantic index for searching
    semantic_index: Option<SemanticIndex>,
}

impl ContextBuilder {
    /// Create a new context builder
    pub fn new(max_tokens: usize) -> Self {
        ContextBuilder {
            max_tokens,
            semantic_index: None,
        }
    }

    /// Set the semantic index for searching
    pub fn with_semantic_index(mut self, index: SemanticIndex) -> Self {
        self.semantic_index = Some(index);
        self
    }

    /// Select relevant files based on a query
    pub fn select_relevant_files(
        &self,
        query: &str,
        all_files: Vec<FileContext>,
    ) -> Result<Vec<FileContext>, ResearchError> {
        if all_files.is_empty() {
            return Ok(Vec::new());
        }

        // Score files based on relevance to query
        let mut scored_files: Vec<(FileContext, f32)> = all_files
            .into_iter()
            .map(|file| {
                let relevance = self.calculate_file_relevance(&file, query);
                (file, relevance)
            })
            .collect();

        // Sort by relevance (highest first)
        scored_files.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Update relevance scores in FileContext
        let mut result: Vec<FileContext> = scored_files
            .into_iter()
            .map(|(mut file, relevance)| {
                file.relevance = relevance;
                file
            })
            .collect();

        // Filter out files with zero relevance
        result.retain(|f| f.relevance > 0.0);

        Ok(result)
    }

    /// Build context from selected files
    pub fn build_context(&self, files: Vec<FileContext>) -> Result<CodeContext, ResearchError> {
        let mut total_tokens = 0;
        let mut included_files = Vec::new();
        let mut all_symbols = Vec::new();
        let mut all_references = Vec::new();

        for file in files {
            // Estimate tokens (rough approximation: 1 token per 4 characters)
            let file_tokens = file
                .content
                .as_ref()
                .map(|c| (c.len() / 4).max(1))
                .unwrap_or(0);

            // Check if adding this file would exceed token budget
            if total_tokens + file_tokens > self.max_tokens && !included_files.is_empty() {
                break;
            }

            total_tokens += file_tokens;
            included_files.push(file);
        }

        // Extract symbols and references from included files
        if let Some(index) = &self.semantic_index {
            for file in &included_files {
                let symbols = index.get_symbols_in_file(&file.path);
                for symbol in symbols {
                    all_symbols.push(symbol.clone());
                    let refs = index.get_references_to_symbol(&symbol.id);
                    for reference in refs {
                        all_references.push(reference.clone());
                    }
                }
            }
        }

        Ok(CodeContext {
            files: included_files,
            symbols: all_symbols,
            references: all_references,
            total_tokens,
        })
    }

    /// Calculate relevance score for a file based on query
    fn calculate_file_relevance(&self, file: &FileContext, query: &str) -> f32 {
        let mut score: f32 = 0.0;

        // Check file path for query terms
        let path_str = file.path.to_string_lossy().to_lowercase();
        let query_lower = query.to_lowercase();

        if path_str.contains(&query_lower) {
            score += 0.5;
        }

        // Check file content for query terms
        if let Some(content) = &file.content {
            let content_lower = content.to_lowercase();
            let query_words: Vec<&str> = query_lower.split_whitespace().collect();

            for word in query_words {
                if content_lower.contains(word) {
                    score += 0.1;
                }
            }
        }

        // Check file summary for query terms
        if let Some(summary) = &file.summary {
            let summary_lower = summary.to_lowercase();
            if summary_lower.contains(&query_lower) {
                score += 0.3;
            }
        }

        // Normalize score to 0.0-1.0 range
        score.min(1.0)
    }

    /// Get the maximum tokens allowed
    pub fn max_tokens(&self) -> usize {
        self.max_tokens
    }

    /// Set the maximum tokens allowed
    pub fn set_max_tokens(&mut self, max_tokens: usize) {
        self.max_tokens = max_tokens;
    }
}

impl Default for ContextBuilder {
    fn default() -> Self {
        ContextBuilder::new(4096) // Default to 4K tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_context_builder_creation() {
        let builder = ContextBuilder::new(8192);
        assert_eq!(builder.max_tokens(), 8192);
    }

    #[test]
    fn test_context_builder_default() {
        let builder = ContextBuilder::default();
        assert_eq!(builder.max_tokens(), 4096);
    }

    #[test]
    fn test_select_relevant_files_empty() {
        let builder = ContextBuilder::new(4096);
        let result = builder.select_relevant_files("test", vec![]);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_select_relevant_files_with_query() {
        let builder = ContextBuilder::new(4096);

        let file1 = FileContext {
            path: PathBuf::from("src/main.rs"),
            relevance: 0.0,
            summary: Some("Main entry point".to_string()),
            content: Some("fn main() {}".to_string()),
        };

        let file2 = FileContext {
            path: PathBuf::from("src/lib.rs"),
            relevance: 0.0,
            summary: Some("Library module".to_string()),
            content: Some("pub fn helper() {}".to_string()),
        };

        let result = builder.select_relevant_files("main", vec![file1, file2]);
        assert!(result.is_ok());

        let files = result.unwrap();
        assert!(!files.is_empty());
        // First file should have higher relevance (contains "main" in path and content)
        assert!(files[0].relevance > 0.0);
    }

    #[test]
    fn test_build_context_respects_token_budget() {
        let builder = ContextBuilder::new(100); // Very small budget

        let file1 = FileContext {
            path: PathBuf::from("src/file1.rs"),
            relevance: 0.9,
            summary: None,
            content: Some("x".repeat(200)), // 200 chars = ~50 tokens
        };

        let file2 = FileContext {
            path: PathBuf::from("src/file2.rs"),
            relevance: 0.8,
            summary: None,
            content: Some("y".repeat(200)), // 200 chars = ~50 tokens
        };

        let result = builder.build_context(vec![file1, file2]);
        assert!(result.is_ok());

        let context = result.unwrap();
        // Should include at least one file but respect token budget
        assert!(context.total_tokens <= 100);
    }

    #[test]
    fn test_calculate_file_relevance_path_match() {
        let builder = ContextBuilder::new(4096);
        let file = FileContext {
            path: PathBuf::from("src/utils.rs"),
            relevance: 0.0,
            summary: None,
            content: None,
        };

        let relevance = builder.calculate_file_relevance(&file, "utils");
        assert!(relevance > 0.0);
    }

    #[test]
    fn test_calculate_file_relevance_content_match() {
        let builder = ContextBuilder::new(4096);
        let file = FileContext {
            path: PathBuf::from("src/main.rs"),
            relevance: 0.0,
            summary: None,
            content: Some("fn helper_function() {}".to_string()),
        };

        let relevance = builder.calculate_file_relevance(&file, "helper");
        assert!(relevance > 0.0);
    }

    #[test]
    fn test_calculate_file_relevance_summary_match() {
        let builder = ContextBuilder::new(4096);
        let file = FileContext {
            path: PathBuf::from("src/main.rs"),
            relevance: 0.0,
            summary: Some("Utility functions for parsing".to_string()),
            content: None,
        };

        let relevance = builder.calculate_file_relevance(&file, "parsing");
        assert!(relevance > 0.0);
    }

    #[test]
    fn test_calculate_file_relevance_no_match() {
        let builder = ContextBuilder::new(4096);
        let file = FileContext {
            path: PathBuf::from("src/main.rs"),
            relevance: 0.0,
            summary: None,
            content: Some("fn main() {}".to_string()),
        };

        let relevance = builder.calculate_file_relevance(&file, "nonexistent");
        assert_eq!(relevance, 0.0);
    }
}
