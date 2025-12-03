//! Relevance scoring for files based on semantic similarity to queries

use crate::models::{FileContext, Symbol};

/// Scores files by semantic similarity to a query
#[derive(Debug, Clone)]
pub struct RelevanceScorer {
    /// Weights for different scoring factors
    weights: ScoringWeights,
}

/// Weights for different scoring factors
#[derive(Debug, Clone)]
pub struct ScoringWeights {
    /// Weight for exact path match
    pub path_exact_match: f32,
    /// Weight for path substring match
    pub path_substring_match: f32,
    /// Weight for filename match
    pub filename_match: f32,
    /// Weight for content match
    pub content_match: f32,
    /// Weight for summary match
    pub summary_match: f32,
    /// Weight for symbol name match
    pub symbol_match: f32,
    /// Weight for recency (newer files score higher)
    pub recency_weight: f32,
}

impl Default for ScoringWeights {
    fn default() -> Self {
        ScoringWeights {
            path_exact_match: 0.8,
            path_substring_match: 0.5,
            filename_match: 0.6,
            content_match: 0.3,
            summary_match: 0.7,
            symbol_match: 0.4,
            recency_weight: 0.1,
        }
    }
}

impl RelevanceScorer {
    /// Create a new relevance scorer with default weights
    pub fn new() -> Self {
        RelevanceScorer {
            weights: ScoringWeights::default(),
        }
    }

    /// Create a new relevance scorer with custom weights
    pub fn with_weights(weights: ScoringWeights) -> Self {
        RelevanceScorer { weights }
    }

    /// Score a file based on relevance to a query
    pub fn score_file(&self, file: &FileContext, query: &str) -> f32 {
        let query_lower = query.to_lowercase();
        let mut score = 0.0;

        // Score path match
        score += self.score_path_match(&file.path, &query_lower);

        // Score content match
        if let Some(content) = &file.content {
            score += self.score_content_match(content, &query_lower);
        }

        // Score summary match
        if let Some(summary) = &file.summary {
            score += self.score_summary_match(summary, &query_lower);
        }

        // Normalize to 0.0-1.0 range
        score.min(1.0)
    }

    /// Score files by relevance to a query
    pub fn score_files(&self, files: &[FileContext], query: &str) -> Vec<(usize, f32)> {
        files
            .iter()
            .enumerate()
            .map(|(idx, file)| (idx, self.score_file(file, query)))
            .collect()
    }

    /// Score path match
    fn score_path_match(&self, path: &std::path::PathBuf, query: &str) -> f32 {
        let path_str = path.to_string_lossy().to_lowercase();

        // Exact match
        if path_str == query {
            return self.weights.path_exact_match;
        }

        // Substring match
        if path_str.contains(query) {
            return self.weights.path_substring_match;
        }

        // Filename match
        if let Some(filename) = path.file_name() {
            let filename_str = filename.to_string_lossy().to_lowercase();
            if filename_str.contains(query) {
                return self.weights.filename_match;
            }
        }

        0.0
    }

    /// Score content match
    fn score_content_match(&self, content: &str, query: &str) -> f32 {
        let content_lower = content.to_lowercase();
        let query_words: Vec<&str> = query.split_whitespace().collect();

        if query_words.is_empty() {
            return 0.0;
        }

        let mut matches = 0;
        for word in &query_words {
            if content_lower.contains(word) {
                matches += 1;
            }
        }

        let match_ratio = matches as f32 / query_words.len() as f32;
        match_ratio * self.weights.content_match
    }

    /// Score summary match
    fn score_summary_match(&self, summary: &str, query: &str) -> f32 {
        let summary_lower = summary.to_lowercase();

        if summary_lower.contains(query) {
            return self.weights.summary_match;
        }

        // Partial match in summary
        let query_words: Vec<&str> = query.split_whitespace().collect();
        if query_words.is_empty() {
            return 0.0;
        }

        let mut matches = 0;
        for word in &query_words {
            if summary_lower.contains(word) {
                matches += 1;
            }
        }

        let match_ratio = matches as f32 / query_words.len() as f32;
        match_ratio * self.weights.summary_match * 0.5 // Reduce weight for partial matches
    }

    /// Score symbols by relevance to a query
    pub fn score_symbols(&self, symbols: &[Symbol], query: &str) -> Vec<(usize, f32)> {
        let query_lower = query.to_lowercase();
        symbols
            .iter()
            .enumerate()
            .map(|(idx, symbol)| {
                let score = self.score_symbol(symbol, &query_lower);
                (idx, score)
            })
            .collect()
    }

    /// Score a single symbol
    fn score_symbol(&self, symbol: &Symbol, query: &str) -> f32 {
        let symbol_name_lower = symbol.name.to_lowercase();

        // Exact match
        if symbol_name_lower == query {
            return 1.0;
        }

        // Substring match
        if symbol_name_lower.contains(query) {
            return 0.8;
        }

        // Partial word match
        let query_words: Vec<&str> = query.split_whitespace().collect();
        let mut matches = 0;
        for word in &query_words {
            if symbol_name_lower.contains(word) {
                matches += 1;
            }
        }

        if matches > 0 {
            (matches as f32 / query_words.len() as f32) * self.weights.symbol_match
        } else {
            0.0
        }
    }

    /// Get scoring weights
    pub fn weights(&self) -> &ScoringWeights {
        &self.weights
    }

    /// Set scoring weights
    pub fn set_weights(&mut self, weights: ScoringWeights) {
        self.weights = weights;
    }
}

impl Default for RelevanceScorer {
    fn default() -> Self {
        RelevanceScorer::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::SymbolKind;
    use std::path::PathBuf;

    #[test]
    fn test_relevance_scorer_creation() {
        let scorer = RelevanceScorer::new();
        assert_eq!(scorer.weights().path_exact_match, 0.8);
    }

    #[test]
    fn test_score_file_with_path_match() {
        let scorer = RelevanceScorer::new();
        let file = FileContext {
            path: PathBuf::from("src/utils.rs"),
            relevance: 0.0,
            summary: None,
            content: None,
        };

        let score = scorer.score_file(&file, "utils");
        assert!(score > 0.0);
    }

    #[test]
    fn test_score_file_with_content_match() {
        let scorer = RelevanceScorer::new();
        let file = FileContext {
            path: PathBuf::from("src/main.rs"),
            relevance: 0.0,
            summary: None,
            content: Some("fn helper_function() {}".to_string()),
        };

        let score = scorer.score_file(&file, "helper");
        assert!(score > 0.0);
    }

    #[test]
    fn test_score_file_with_summary_match() {
        let scorer = RelevanceScorer::new();
        let file = FileContext {
            path: PathBuf::from("src/main.rs"),
            relevance: 0.0,
            summary: Some("Utility functions for parsing".to_string()),
            content: None,
        };

        let score = scorer.score_file(&file, "parsing");
        assert!(score > 0.0);
    }

    #[test]
    fn test_score_file_no_match() {
        let scorer = RelevanceScorer::new();
        let file = FileContext {
            path: PathBuf::from("src/main.rs"),
            relevance: 0.0,
            summary: None,
            content: Some("fn main() {}".to_string()),
        };

        let score = scorer.score_file(&file, "nonexistent");
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_score_files() {
        let scorer = RelevanceScorer::new();
        let files = vec![
            FileContext {
                path: PathBuf::from("src/utils.rs"),
                relevance: 0.0,
                summary: None,
                content: None,
            },
            FileContext {
                path: PathBuf::from("src/main.rs"),
                relevance: 0.0,
                summary: None,
                content: None,
            },
        ];

        let scores = scorer.score_files(&files, "utils");
        assert_eq!(scores.len(), 2);
        // First file should have higher score
        assert!(scores[0].1 > scores[1].1);
    }

    #[test]
    fn test_score_symbols() {
        let scorer = RelevanceScorer::new();
        let symbols = vec![
            Symbol {
                id: "1".to_string(),
                name: "helper_function".to_string(),
                kind: SymbolKind::Function,
                file: PathBuf::from("src/main.rs"),
                line: 1,
                column: 1,
                references: vec![],
            },
            Symbol {
                id: "2".to_string(),
                name: "main".to_string(),
                kind: SymbolKind::Function,
                file: PathBuf::from("src/main.rs"),
                line: 10,
                column: 1,
                references: vec![],
            },
        ];

        let scores = scorer.score_symbols(&symbols, "helper");
        assert_eq!(scores.len(), 2);
        // First symbol should have higher score
        assert!(scores[0].1 > scores[1].1);
    }

    #[test]
    fn test_score_symbol_exact_match() {
        let scorer = RelevanceScorer::new();
        let symbol = Symbol {
            id: "1".to_string(),
            name: "helper_function".to_string(),
            kind: SymbolKind::Function,
            file: PathBuf::from("src/main.rs"),
            line: 1,
            column: 1,
            references: vec![],
        };

        let score = scorer.score_symbol(&symbol, "helper_function");
        assert_eq!(score, 1.0);
    }

    #[test]
    fn test_score_symbol_substring_match() {
        let scorer = RelevanceScorer::new();
        let symbol = Symbol {
            id: "1".to_string(),
            name: "helper_function".to_string(),
            kind: SymbolKind::Function,
            file: PathBuf::from("src/main.rs"),
            line: 1,
            column: 1,
            references: vec![],
        };

        let score = scorer.score_symbol(&symbol, "helper");
        assert_eq!(score, 0.8);
    }

    #[test]
    fn test_custom_weights() {
        let weights = ScoringWeights {
            path_exact_match: 1.0,
            path_substring_match: 0.9,
            filename_match: 0.8,
            content_match: 0.7,
            summary_match: 0.6,
            symbol_match: 0.5,
            recency_weight: 0.1,
        };

        let scorer = RelevanceScorer::with_weights(weights);
        assert_eq!(scorer.weights().path_exact_match, 1.0);
    }
}
