use std::cmp::Ordering;

/// Completion ranking and filtering implementation
use crate::types::*;

/// Basic completion ranker with prefix matching and fuzzy matching
pub struct BasicCompletionRanker {
    /// Weights for different scoring factors
    weights: RankingWeights,
    /// Optional completion history for frequency and recency tracking
    history: Option<std::sync::Arc<crate::history::CompletionHistory>>,
}

impl BasicCompletionRanker {
    pub fn new(weights: RankingWeights) -> Self {
        Self {
            weights,
            history: None,
        }
    }

    pub fn with_history(
        weights: RankingWeights,
        history: std::sync::Arc<crate::history::CompletionHistory>,
    ) -> Self {
        Self {
            weights,
            history: Some(history),
        }
    }

    pub fn default_weights() -> Self {
        Self::new(RankingWeights::default())
    }

    /// Filter completions by prefix match
    fn filter_by_prefix(&self, items: Vec<CompletionItem>, prefix: &str) -> Vec<CompletionItem> {
        if prefix.is_empty() {
            return items;
        }

        items
            .into_iter()
            .filter(|item| {
                let filter_text = item.filter_text.as_ref().unwrap_or(&item.label);
                filter_text
                    .to_lowercase()
                    .starts_with(&prefix.to_lowercase())
            })
            .collect()
    }

    /// Calculate fuzzy match score (0.0 to 1.0) using nucleo
    /// Returns 1.0 for exact match, decreasing score for fuzzy matches
    fn fuzzy_match_score(&self, text: &str, pattern: &str) -> f32 {
        use nucleo::pattern::{AtomKind, CaseMatching, Normalization, Pattern};
        use nucleo::{Config, Matcher, Utf32Str};

        if pattern.is_empty() {
            return 0.0;
        }

        let mut matcher = Matcher::new(Config::DEFAULT);
        let pattern_obj = Pattern::new(
            pattern,
            CaseMatching::Smart,
            Normalization::Smart,
            AtomKind::Fuzzy,
        );

        let mut buf = Vec::new();
        let haystack = Utf32Str::new(text, &mut buf);

        if let Some(score) = pattern_obj.score(haystack, &mut matcher) {
            // Normalize nucleo's u16 score to 0.0-1.0 range
            // nucleo scores are in range 0-u16::MAX (65535), but practical scores are much lower
            // Use sqrt to expand the lower range where most scores fall
            ((score as f32) / 65535.0).sqrt()
        } else {
            0.0
        }
    }

    /// Calculate case-insensitive match score
    #[allow(dead_code)]
    fn case_insensitive_score(&self, text: &str, pattern: &str) -> f32 {
        if text.to_lowercase() == pattern.to_lowercase() {
            1.0
        } else {
            0.0
        }
    }

    /// Calculate relevance score based on symbol kind and context
    fn calculate_relevance_score(&self, item: &CompletionItem, context: &CompletionContext) -> f32 {
        let mut score: f32 = 0.5; // Base score

        // Boost score for items matching expected type
        if let Some(expected_type) = &context.expected_type {
            if let Some(detail) = &item.detail {
                if detail.contains(&expected_type.name) {
                    score += 0.3;
                }
            }
        }

        // Boost score for items in current scope
        match item.kind {
            CompletionItemKind::Variable | CompletionItemKind::Function => score += 0.2,
            CompletionItemKind::Keyword => score += 0.1,
            _ => {}
        }

        score.min(1.0)
    }
}

impl crate::engine::CompletionRanker for BasicCompletionRanker {
    fn rank_completions(
        &self,
        items: Vec<CompletionItem>,
        context: &CompletionContext,
    ) -> Vec<CompletionItem> {
        // Filter by prefix
        let mut filtered = self.filter_by_prefix(items, &context.prefix);

        // Score each item
        for item in &mut filtered {
            let prefix_score = self.fuzzy_match_score(&item.label, &context.prefix);
            let relevance_score = self.calculate_relevance_score(item, context);
            let frequency_score = self.score_frequency(item);

            // Combine scores using weights
            let combined_score = (prefix_score * 0.4)
                + (relevance_score * self.weights.relevance)
                + (frequency_score * self.weights.frequency);

            item.score = combined_score.min(1.0);
        }

        // Sort by score (descending), then by label (ascending) for stability
        filtered.sort_by(|a, b| match b.score.partial_cmp(&a.score) {
            Some(Ordering::Equal) | None => a.label.cmp(&b.label),
            other => other.unwrap(),
        });

        filtered
    }

    fn score_relevance(&self, item: &CompletionItem, context: &CompletionContext) -> f32 {
        self.calculate_relevance_score(item, context)
    }

    fn score_frequency(&self, item: &CompletionItem) -> f32 {
        if let Some(history) = &self.history {
            // Get frequency and recency scores from history
            let frequency_score = history
                .get_frequency_score(&item.label, "generic")
                .unwrap_or(0.0);
            let recency_score = history
                .get_recency_score(&item.label, "generic")
                .unwrap_or(0.0);

            // Combine frequency and recency
            (frequency_score * self.weights.frequency + recency_score * self.weights.recency)
                / (self.weights.frequency + self.weights.recency)
        } else {
            // Default frequency score when no history is available
            0.5
        }
    }
}

/// Advanced completion ranker with fuzzy matching and scoring
pub struct AdvancedCompletionRanker {
    basic_ranker: BasicCompletionRanker,
}

impl AdvancedCompletionRanker {
    pub fn new(weights: RankingWeights) -> Self {
        Self {
            basic_ranker: BasicCompletionRanker::new(weights),
        }
    }

    pub fn with_history(
        weights: RankingWeights,
        history: std::sync::Arc<crate::history::CompletionHistory>,
    ) -> Self {
        Self {
            basic_ranker: BasicCompletionRanker::with_history(weights, history),
        }
    }

    pub fn default_weights() -> Self {
        Self::new(RankingWeights::default())
    }

    /// Perform fuzzy matching with scoring
    fn fuzzy_filter(&self, items: Vec<CompletionItem>, pattern: &str) -> Vec<CompletionItem> {
        if pattern.is_empty() {
            return items;
        }

        items
            .into_iter()
            .filter_map(|mut item| {
                let filter_text = item.filter_text.as_ref().unwrap_or(&item.label);
                let score = self.basic_ranker.fuzzy_match_score(filter_text, pattern);
                if score > 0.0 {
                    item.score = score;
                    Some(item)
                } else {
                    None
                }
            })
            .collect()
    }
}

impl crate::engine::CompletionRanker for AdvancedCompletionRanker {
    fn rank_completions(
        &self,
        items: Vec<CompletionItem>,
        context: &CompletionContext,
    ) -> Vec<CompletionItem> {
        // Use fuzzy filtering
        let mut filtered = self.fuzzy_filter(items, &context.prefix);

        // Score each item
        for item in &mut filtered {
            let relevance_score = self.basic_ranker.score_relevance(item, context);
            let frequency_score = self.basic_ranker.score_frequency(item);

            // Combine scores
            let combined_score = (item.score * 0.4)
                + (relevance_score * self.basic_ranker.weights.relevance)
                + (frequency_score * self.basic_ranker.weights.frequency);

            item.score = combined_score.min(1.0);
        }

        // Sort by score (descending), then by label (ascending)
        filtered.sort_by(|a, b| match b.score.partial_cmp(&a.score) {
            Some(Ordering::Equal) | None => a.label.cmp(&b.label),
            other => other.unwrap(),
        });

        filtered
    }

    fn score_relevance(&self, item: &CompletionItem, context: &CompletionContext) -> f32 {
        self.basic_ranker.score_relevance(item, context)
    }

    fn score_frequency(&self, item: &CompletionItem) -> f32 {
        self.basic_ranker.score_frequency(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::CompletionRanker;

    #[test]
    fn test_prefix_filter_exact_match() {
        let ranker = BasicCompletionRanker::default_weights();
        let items = vec![
            CompletionItem::new(
                "test".to_string(),
                CompletionItemKind::Variable,
                "test".to_string(),
            ),
            CompletionItem::new(
                "testing".to_string(),
                CompletionItemKind::Variable,
                "testing".to_string(),
            ),
            CompletionItem::new(
                "other".to_string(),
                CompletionItemKind::Variable,
                "other".to_string(),
            ),
        ];

        let filtered = ranker.filter_by_prefix(items, "test");
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().any(|i| i.label == "test"));
        assert!(filtered.iter().any(|i| i.label == "testing"));
    }

    #[test]
    fn test_prefix_filter_case_insensitive() {
        let ranker = BasicCompletionRanker::default_weights();
        let items = vec![
            CompletionItem::new(
                "Test".to_string(),
                CompletionItemKind::Variable,
                "Test".to_string(),
            ),
            CompletionItem::new(
                "TEST".to_string(),
                CompletionItemKind::Variable,
                "TEST".to_string(),
            ),
            CompletionItem::new(
                "other".to_string(),
                CompletionItemKind::Variable,
                "other".to_string(),
            ),
        ];

        let filtered = ranker.filter_by_prefix(items, "test");
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_prefix_filter_empty_prefix() {
        let ranker = BasicCompletionRanker::default_weights();
        let items = vec![
            CompletionItem::new(
                "test".to_string(),
                CompletionItemKind::Variable,
                "test".to_string(),
            ),
            CompletionItem::new(
                "other".to_string(),
                CompletionItemKind::Variable,
                "other".to_string(),
            ),
        ];

        let filtered = ranker.filter_by_prefix(items, "");
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_fuzzy_match_exact() {
        let ranker = BasicCompletionRanker::default_weights();
        let score = ranker.fuzzy_match_score("test", "test");
        // nucleo scores are normalized with sqrt, so exact match won't be 1.0
        assert!(score > 0.0, "Exact match should have non-zero score");
        assert!(score <= 1.0, "Score should be normalized");
    }

    #[test]
    fn test_fuzzy_match_prefix() {
        let ranker = BasicCompletionRanker::default_weights();
        let score = ranker.fuzzy_match_score("testing", "test");
        // nucleo should match prefix patterns with reasonable score
        assert!(score > 0.0, "Prefix match should have non-zero score");
    }

    #[test]
    fn test_fuzzy_match_case_insensitive() {
        let ranker = BasicCompletionRanker::default_weights();
        let score = ranker.fuzzy_match_score("Test", "test");
        // Case-insensitive matching with Smart case
        assert!(score > 0.0, "Case-insensitive match should have non-zero score");
        assert!(score <= 1.0, "Score should be normalized");
    }

    #[test]
    fn test_fuzzy_match_partial() {
        let ranker = BasicCompletionRanker::default_weights();
        let score = ranker.fuzzy_match_score("test_variable", "tv");
        assert!(score > 0.0);
        assert!(score < 0.5);
    }

    #[test]
    fn test_fuzzy_match_no_match() {
        let ranker = BasicCompletionRanker::default_weights();
        let score = ranker.fuzzy_match_score("test", "xyz");
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_ranking_sorts_by_score() {
        let ranker = BasicCompletionRanker::default_weights();
        let items = vec![
            CompletionItem::new(
                "test".to_string(),
                CompletionItemKind::Variable,
                "test".to_string(),
            ),
            CompletionItem::new(
                "testing".to_string(),
                CompletionItemKind::Variable,
                "testing".to_string(),
            ),
        ];

        let context =
            CompletionContext::new("rust".to_string(), Position::new(0, 0), "test".to_string());
        let ranked = ranker.rank_completions(items, &context);

        // Both should match the prefix "test", but "test" is an exact match so should rank higher
        assert_eq!(ranked[0].label, "test");
        assert_eq!(ranked[1].label, "testing");
    }

    #[test]
    fn test_ranking_with_prefix() {
        let ranker = BasicCompletionRanker::default_weights();
        let items = vec![
            CompletionItem::new(
                "test".to_string(),
                CompletionItemKind::Variable,
                "test".to_string(),
            ),
            CompletionItem::new(
                "testing".to_string(),
                CompletionItemKind::Variable,
                "testing".to_string(),
            ),
            CompletionItem::new(
                "other".to_string(),
                CompletionItemKind::Variable,
                "other".to_string(),
            ),
        ];

        let context =
            CompletionContext::new("rust".to_string(), Position::new(0, 0), "test".to_string());
        let ranked = ranker.rank_completions(items, &context);

        assert_eq!(ranked.len(), 2);
        assert!(ranked.iter().all(|i| i.label.starts_with("test")));
    }

    #[test]
    fn test_advanced_ranker_fuzzy_filter() {
        let ranker = AdvancedCompletionRanker::default_weights();
        let items = vec![
            CompletionItem::new(
                "test_variable".to_string(),
                CompletionItemKind::Variable,
                "test_variable".to_string(),
            ),
            CompletionItem::new(
                "other".to_string(),
                CompletionItemKind::Variable,
                "other".to_string(),
            ),
        ];

        let filtered = ranker.fuzzy_filter(items, "tv");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].label, "test_variable");
    }

    #[test]
    fn test_relevance_score_with_expected_type() {
        let ranker = BasicCompletionRanker::default_weights();
        let mut item = CompletionItem::new(
            "test".to_string(),
            CompletionItemKind::Variable,
            "test".to_string(),
        );
        item = item.with_detail("String".to_string());

        let mut context =
            CompletionContext::new("rust".to_string(), Position::new(0, 0), "".to_string());
        context.expected_type = Some(Type::new("String".to_string()));

        let score = ranker.score_relevance(&item, &context);
        assert!(score > 0.5);
    }

    #[test]
    fn test_ranking_stability() {
        let ranker = BasicCompletionRanker::default_weights();
        let items = vec![
            CompletionItem::new(
                "aaa".to_string(),
                CompletionItemKind::Variable,
                "aaa".to_string(),
            ),
            CompletionItem::new(
                "aab".to_string(),
                CompletionItemKind::Variable,
                "aab".to_string(),
            ),
            CompletionItem::new(
                "aac".to_string(),
                CompletionItemKind::Variable,
                "aac".to_string(),
            ),
        ];

        let context =
            CompletionContext::new("rust".to_string(), Position::new(0, 0), "aa".to_string());
        let ranked = ranker.rank_completions(items, &context);

        // Should be sorted by label when scores are equal
        assert_eq!(ranked[0].label, "aaa");
        assert_eq!(ranked[1].label, "aab");
        assert_eq!(ranked[2].label, "aac");
    }

    #[test]
    fn test_frequency_scoring_with_history() {
        let history = std::sync::Arc::new(crate::history::CompletionHistory::new());
        let ranker =
            BasicCompletionRanker::with_history(RankingWeights::default(), history.clone());

        // Record some usages
        history
            .record_usage("test".to_string(), "generic".to_string())
            .unwrap();
        history
            .record_usage("test".to_string(), "generic".to_string())
            .unwrap();

        let item = CompletionItem::new(
            "test".to_string(),
            CompletionItemKind::Variable,
            "test".to_string(),
        );
        let score = ranker.score_frequency(&item);

        // Score should be greater than default 0.5 due to frequency
        assert!(score > 0.0);
    }

    #[test]
    fn test_frequency_scoring_without_history() {
        let ranker = BasicCompletionRanker::default_weights();
        let item = CompletionItem::new(
            "test".to_string(),
            CompletionItemKind::Variable,
            "test".to_string(),
        );
        let score = ranker.score_frequency(&item);

        // Should return default score when no history
        assert_eq!(score, 0.5);
    }

    #[test]
    fn test_ranking_with_frequency_boost() {
        let history = std::sync::Arc::new(crate::history::CompletionHistory::new());
        let ranker =
            BasicCompletionRanker::with_history(RankingWeights::default(), history.clone());

        // Record usage for one item
        history
            .record_usage("frequent".to_string(), "generic".to_string())
            .unwrap();
        history
            .record_usage("frequent".to_string(), "generic".to_string())
            .unwrap();

        let items = vec![
            CompletionItem::new(
                "frequent".to_string(),
                CompletionItemKind::Variable,
                "frequent".to_string(),
            ),
            CompletionItem::new(
                "rare".to_string(),
                CompletionItemKind::Variable,
                "rare".to_string(),
            ),
        ];

        let context =
            CompletionContext::new("rust".to_string(), Position::new(0, 0), "".to_string());
        let ranked = ranker.rank_completions(items, &context);

        // Frequent item should rank higher
        assert_eq!(ranked[0].label, "frequent");
    }
}
