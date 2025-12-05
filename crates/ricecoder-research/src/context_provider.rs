//! Automatic context provision for AI providers

use crate::context_builder::ContextBuilder;
use crate::context_optimizer::ContextOptimizer;
use crate::models::{CodeContext, FileContext};
use crate::relevance_scorer::RelevanceScorer;
use crate::ResearchError;

/// Provides context automatically to AI providers
#[derive(Debug, Clone)]
pub struct ContextProvider {
    /// Context builder for selecting files
    context_builder: ContextBuilder,
    /// Context optimizer for fitting within token budgets
    context_optimizer: ContextOptimizer,
    /// Relevance scorer for ranking files
    relevance_scorer: RelevanceScorer,
}

impl ContextProvider {
    /// Create a new context provider
    pub fn new(max_tokens: usize, max_tokens_per_file: usize) -> Self {
        ContextProvider {
            context_builder: ContextBuilder::new(max_tokens),
            context_optimizer: ContextOptimizer::new(max_tokens_per_file),
            relevance_scorer: RelevanceScorer::new(),
        }
    }

    /// Provide context for a code generation task
    pub fn provide_context_for_generation(
        &self,
        query: &str,
        available_files: Vec<FileContext>,
    ) -> Result<CodeContext, ResearchError> {
        // Select relevant files
        let relevant_files = self
            .context_builder
            .select_relevant_files(query, available_files)?;

        // Optimize files to fit within token budget
        let optimized_files = self.context_optimizer.optimize_files(relevant_files)?;

        // Build final context
        self.context_builder.build_context(optimized_files)
    }

    /// Provide context for a code review task
    pub fn provide_context_for_review(
        &self,
        query: &str,
        available_files: Vec<FileContext>,
    ) -> Result<CodeContext, ResearchError> {
        // For review, we want to include more context
        let mut builder = self.context_builder.clone();
        builder.set_max_tokens(builder.max_tokens() * 2); // Double the budget for review

        // Select relevant files
        let relevant_files = builder.select_relevant_files(query, available_files)?;

        // Optimize files
        let optimized_files = self.context_optimizer.optimize_files(relevant_files)?;

        // Build final context
        builder.build_context(optimized_files)
    }

    /// Provide context for a refactoring task
    pub fn provide_context_for_refactoring(
        &self,
        query: &str,
        available_files: Vec<FileContext>,
    ) -> Result<CodeContext, ResearchError> {
        // For refactoring, include related files
        let relevant_files = self
            .context_builder
            .select_relevant_files(query, available_files)?;

        // Optimize files
        let optimized_files = self.context_optimizer.optimize_files(relevant_files)?;

        // Build final context
        self.context_builder.build_context(optimized_files)
    }

    /// Provide context for a documentation task
    pub fn provide_context_for_documentation(
        &self,
        query: &str,
        available_files: Vec<FileContext>,
    ) -> Result<CodeContext, ResearchError> {
        // For documentation, prioritize files with summaries
        let mut scored_files: Vec<(FileContext, f32)> = available_files
            .into_iter()
            .map(|file| {
                let mut score = self.relevance_scorer.score_file(&file, query);
                // Boost score if file has a summary
                if file.summary.is_some() {
                    score += 0.2;
                }
                (file, score)
            })
            .collect();

        // Sort by score
        scored_files.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Extract files
        let mut files: Vec<FileContext> = scored_files
            .into_iter()
            .map(|(mut file, score)| {
                file.relevance = score;
                file
            })
            .collect();

        // Filter out files with zero relevance
        files.retain(|f| f.relevance > 0.0);

        // Optimize files
        let optimized_files = self.context_optimizer.optimize_files(files)?;

        // Build final context
        self.context_builder.build_context(optimized_files)
    }

    /// Get the context builder
    pub fn context_builder(&self) -> &ContextBuilder {
        &self.context_builder
    }

    /// Get the context optimizer
    pub fn context_optimizer(&self) -> &ContextOptimizer {
        &self.context_optimizer
    }

    /// Get the relevance scorer
    pub fn relevance_scorer(&self) -> &RelevanceScorer {
        &self.relevance_scorer
    }

    /// Set maximum tokens
    pub fn set_max_tokens(&mut self, max_tokens: usize) {
        self.context_builder.set_max_tokens(max_tokens);
    }

    /// Set maximum tokens per file
    pub fn set_max_tokens_per_file(&mut self, max_tokens: usize) {
        self.context_optimizer.set_max_tokens_per_file(max_tokens);
    }
}

impl Default for ContextProvider {
    fn default() -> Self {
        ContextProvider::new(4096, 2048) // Default: 4K total, 2K per file
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_context_provider_creation() {
        let provider = ContextProvider::new(4096, 2048);
        assert_eq!(provider.context_builder().max_tokens(), 4096);
        assert_eq!(provider.context_optimizer().max_tokens_per_file(), 2048);
    }

    #[test]
    fn test_context_provider_default() {
        let provider = ContextProvider::default();
        assert_eq!(provider.context_builder().max_tokens(), 4096);
        assert_eq!(provider.context_optimizer().max_tokens_per_file(), 2048);
    }

    #[test]
    fn test_provide_context_for_generation() {
        let provider = ContextProvider::new(4096, 2048);

        let files = vec![
            FileContext {
                path: PathBuf::from("src/main.rs"),
                relevance: 0.0,
                summary: Some("Main entry point".to_string()),
                content: Some("fn main() {}".to_string()),
            },
            FileContext {
                path: PathBuf::from("src/lib.rs"),
                relevance: 0.0,
                summary: Some("Library module".to_string()),
                content: Some("pub fn helper() {}".to_string()),
            },
        ];

        let result = provider.provide_context_for_generation("main", files);
        assert!(result.is_ok());

        let context = result.unwrap();
        assert!(!context.files.is_empty());
    }

    #[test]
    fn test_provide_context_for_review() {
        let provider = ContextProvider::new(4096, 2048);

        let files = vec![FileContext {
            path: PathBuf::from("src/main.rs"),
            relevance: 0.0,
            summary: Some("Main entry point".to_string()),
            content: Some("fn main() {}".to_string()),
        }];

        let result = provider.provide_context_for_review("main", files);
        assert!(result.is_ok());

        let context = result.unwrap();
        assert!(!context.files.is_empty());
    }

    #[test]
    fn test_provide_context_for_refactoring() {
        let provider = ContextProvider::new(4096, 2048);

        let files = vec![FileContext {
            path: PathBuf::from("src/main.rs"),
            relevance: 0.0,
            summary: Some("Main entry point".to_string()),
            content: Some("fn main() {}".to_string()),
        }];

        let result = provider.provide_context_for_refactoring("main", files);
        assert!(result.is_ok());

        let context = result.unwrap();
        assert!(!context.files.is_empty());
    }

    #[test]
    fn test_provide_context_for_documentation() {
        let provider = ContextProvider::new(4096, 2048);

        let files = vec![
            FileContext {
                path: PathBuf::from("src/main.rs"),
                relevance: 0.0,
                summary: Some("Main entry point".to_string()),
                content: Some("fn main() {}".to_string()),
            },
            FileContext {
                path: PathBuf::from("src/lib.rs"),
                relevance: 0.0,
                summary: None,
                content: Some("pub fn helper() {}".to_string()),
            },
        ];

        let result = provider.provide_context_for_documentation("main", files);
        assert!(result.is_ok());

        let context = result.unwrap();
        assert!(!context.files.is_empty());
    }

    #[test]
    fn test_set_max_tokens() {
        let mut provider = ContextProvider::new(4096, 2048);
        provider.set_max_tokens(8192);
        assert_eq!(provider.context_builder().max_tokens(), 8192);
    }

    #[test]
    fn test_set_max_tokens_per_file() {
        let mut provider = ContextProvider::new(4096, 2048);
        provider.set_max_tokens_per_file(4096);
        assert_eq!(provider.context_optimizer().max_tokens_per_file(), 4096);
    }

    #[test]
    fn test_provide_context_empty_files() {
        let provider = ContextProvider::new(4096, 2048);
        let result = provider.provide_context_for_generation("test", vec![]);
        assert!(result.is_ok());

        let context = result.unwrap();
        assert!(context.files.is_empty());
    }
}
