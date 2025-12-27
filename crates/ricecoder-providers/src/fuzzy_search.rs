//! Fuzzy search for model names and provider suggestions

use crate::models::ModelInfo;

/// Fuzzy match score (0.0 to 1.0)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct MatchScore(f64);

impl MatchScore {
    pub fn new(score: f64) -> Self {
        Self(score.clamp(0.0, 1.0))
    }

    pub fn value(&self) -> f64 {
        self.0
    }
}

/// Fuzzy match result
#[derive(Debug, Clone)]
pub struct FuzzyMatch<T> {
    pub item: T,
    pub score: MatchScore,
}

/// Find top N fuzzy matches for a query
pub fn fuzzy_search_models(
    query: &str,
    models: &[ModelInfo],
    limit: usize,
) -> Vec<FuzzyMatch<ModelInfo>> {
    let mut matches: Vec<FuzzyMatch<ModelInfo>> = models
        .iter()
        .map(|model| FuzzyMatch {
            item: model.clone(),
            score: fuzzy_match(&model.id, query),
        })
        .filter(|m| m.score.value() > 0.0) // Any non-zero match is valid
        .collect();

    matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    matches.into_iter().take(limit).collect()
}

/// Find top N fuzzy matches for provider names
pub fn fuzzy_search_providers(
    query: &str,
    providers: &[String],
    limit: usize,
) -> Vec<FuzzyMatch<String>> {
    let mut matches: Vec<FuzzyMatch<String>> = providers
        .iter()
        .map(|provider| FuzzyMatch {
            item: provider.clone(),
            score: fuzzy_match(provider, query),
        })
        .filter(|m| m.score.value() > 0.0) // Any non-zero match is valid
        .collect();

    matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    matches.into_iter().take(limit).collect()
}

/// Calculate fuzzy match score using nucleo (Helix editor's fuzzy matcher)
fn fuzzy_match(text: &str, pattern: &str) -> MatchScore {
    use nucleo::pattern::{AtomKind, CaseMatching, Normalization, Pattern};
    use nucleo::{Config, Matcher, Utf32Str};

    if pattern.is_empty() {
        return MatchScore::new(0.0);
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
        let normalized = ((score as f64) / 65535.0).sqrt();
        MatchScore::new(normalized)
    } else {
        MatchScore::new(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Capability;

    #[test]
    fn test_fuzzy_match_exact() {
        let score = fuzzy_match("gpt-4", "gpt-4");
        // nucleo scores are normalized with sqrt, so exact match won't be 1.0
        assert!(score.value() > 0.0, "Exact match should have non-zero score");
        assert!(score.value() <= 1.0, "Score should be normalized");
    }

    #[test]
    fn test_fuzzy_match_contains() {
        let score = fuzzy_match("gpt-4-turbo", "gpt-4");
        // nucleo should match prefix patterns
        assert!(score.value() > 0.0, "Prefix match should have non-zero score");
    }

    #[test]
    fn test_fuzzy_search_models() {
        let models = vec![
            ModelInfo {
                id: "gpt-4".to_string(),
                name: "GPT-4".to_string(),
                provider: "openai".to_string(),
                context_window: 8192,
                capabilities: vec![Capability::Chat],
                pricing: None,
                is_free: false,
            },
            ModelInfo {
                id: "claude-3".to_string(),
                name: "Claude 3".to_string(),
                provider: "anthropic".to_string(),
                context_window: 200000,
                capabilities: vec![Capability::Chat],
                pricing: None,
                is_free: false,
            },
        ];

        let matches = fuzzy_search_models("gpt", &models, 3);
        assert!(!matches.is_empty());
        assert_eq!(matches[0].item.id, "gpt-4");
    }
}
