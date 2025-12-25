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
        .filter(|m| m.score.value() > 0.3) // Minimum threshold
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
        .filter(|m| m.score.value() > 0.3)
        .collect();

    matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    matches.into_iter().take(limit).collect()
}

/// Calculate fuzzy match score using Levenshtein-like algorithm
fn fuzzy_match(text: &str, pattern: &str) -> MatchScore {
    let text_lower = text.to_lowercase();
    let pattern_lower = pattern.to_lowercase();

    // Exact match
    if text_lower == pattern_lower {
        return MatchScore::new(1.0);
    }

    // Contains match
    if text_lower.contains(&pattern_lower) {
        let ratio = pattern_lower.len() as f64 / text_lower.len() as f64;
        return MatchScore::new(0.8 * ratio);
    }

    // Character-by-character matching
    let text_chars: Vec<char> = text_lower.chars().collect();
    let pattern_chars: Vec<char> = pattern_lower.chars().collect();

    let mut pattern_idx = 0;
    let mut matched = 0;

    for &ch in &text_chars {
        if pattern_idx < pattern_chars.len() && ch == pattern_chars[pattern_idx] {
            matched += 1;
            pattern_idx += 1;
        }
    }

    let score = if pattern_chars.is_empty() {
        0.0
    } else {
        (matched as f64 / pattern_chars.len() as f64) * 0.6
    };

    MatchScore::new(score)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Capability;

    #[test]
    fn test_fuzzy_match_exact() {
        let score = fuzzy_match("gpt-4", "gpt-4");
        assert_eq!(score.value(), 1.0);
    }

    #[test]
    fn test_fuzzy_match_contains() {
        let score = fuzzy_match("gpt-4-turbo", "gpt-4");
        assert!(score.value() > 0.4);
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
