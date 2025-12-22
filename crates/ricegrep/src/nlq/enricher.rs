use std::collections::{HashMap, HashSet};

use crate::nlq::{
    error::ProcessingError,
    filter::FilterExtractor,
    models::{EnrichedQuery, ParsedQuery, QueryIntent},
};

pub struct QueryEnricher {
    synonyms: HashMap<String, Vec<String>>,
    filter_extractor: FilterExtractor,
}

impl QueryEnricher {
    pub fn new() -> Self {
        Self {
            synonyms: Self::default_synonyms(),
            filter_extractor: FilterExtractor::new(),
        }
    }

    pub async fn enrich(
        &self,
        query: &ParsedQuery,
        intent: &QueryIntent,
    ) -> Result<EnrichedQuery, ProcessingError> {
        let expanded_terms = self.expand_terms(query, intent);
        let filters = self.filter_extractor.extract(&query.tokens);
        let confidence = self.estimate_confidence(intent, &expanded_terms, filters.len());

        Ok(EnrichedQuery {
            parsed: query.clone(),
            intent: intent.clone(),
            expanded_terms,
            filters,
            confidence,
        })
    }

    fn expand_terms(&self, query: &ParsedQuery, intent: &QueryIntent) -> Vec<String> {
        let mut seen = HashSet::new();
        let mut terms = Vec::with_capacity(query.tokens.len() * 2);

        for token in &query.tokens {
            self.push_unique(&mut terms, &mut seen, token.clone());
            if let Some(synonyms) = self.synonyms.get(token) {
                for synonym in synonyms {
                    self.push_unique(&mut terms, &mut seen, synonym.clone());
                }
            }
        }

        if matches!(intent, QueryIntent::DocumentationSearch) {
            for extra in ["docs", "guide", "tutorial", "reference"] {
                self.push_unique(&mut terms, &mut seen, extra.to_string());
            }
        }

        terms
    }

    fn push_unique(&self, terms: &mut Vec<String>, seen: &mut HashSet<String>, candidate: String) {
        if seen.insert(candidate.clone()) {
            terms.push(candidate);
        }
    }

    fn estimate_confidence(
        &self,
        intent: &QueryIntent,
        expanded_terms: &[String],
        filter_count: usize,
    ) -> f32 {
        let base = match intent {
            QueryIntent::GeneralSearch => 0.55,
            _ => 0.75,
        };
        let synonym_bonus = ((expanded_terms.len().saturating_sub(1)) as f32 * 0.015).min(0.18);
        let filter_bonus = (filter_count as f32 * 0.02).min(0.1);
        (base + synonym_bonus + filter_bonus).min(0.97)
    }

    fn default_synonyms() -> HashMap<String, Vec<String>> {
        let mut synonyms = HashMap::new();
        synonyms.insert(
            "api".into(),
            vec!["endpoint".into(), "rest".into(), "graphql".into()],
        );
        synonyms.insert("search".into(), vec!["find".into(), "query".into()]);
        synonyms.insert(
            "bug".into(),
            vec!["issue".into(), "error".into(), "failure".into()],
        );
        synonyms.insert("function".into(), vec!["method".into(), "fn".into()]);
        synonyms.insert(
            "documentation".into(),
            vec!["docs".into(), "guide".into(), "manual".into()],
        );
        synonyms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nlq::models::{ParsedQuery, QueryComplexity, QueryIntent};

    fn stub_query(text: &str) -> ParsedQuery {
        ParsedQuery {
            original: text.to_string(),
            tokens: text
                .split_whitespace()
                .map(|token| token.to_lowercase())
                .collect(),
            language: "en".into(),
            complexity: QueryComplexity::Moderate,
        }
    }

    #[tokio::test]
    async fn enrich_adds_synonyms_and_confidence() {
        let enricher = QueryEnricher::new();
        let parsed = stub_query("documentation api health check");
        let enriched = enricher
            .enrich(&parsed, &QueryIntent::DocumentationSearch)
            .await
            .unwrap();
        assert!(enriched.expanded_terms.contains(&"docs".into()));
        assert!(enriched.expanded_terms.contains(&"endpoint".into()));
        assert!(enriched.confidence > 0.7);
    }

    #[tokio::test]
    async fn enrich_extracts_filters() {
        let enricher = QueryEnricher::new();
        let parsed = ParsedQuery {
            original: "search language:rust repo=core".into(),
            tokens: vec!["search".into(), "language:rust".into(), "repo=core".into()],
            language: "en".into(),
            complexity: QueryComplexity::Moderate,
        };
        let enriched = enricher
            .enrich(&parsed, &QueryIntent::CodeSearch)
            .await
            .unwrap();
        assert_eq!(enriched.filters.len(), 2);
        assert!(enriched
            .filters
            .iter()
            .any(|filter| filter.field == "language" && filter.value == "rust"));
        assert!(enriched
            .filters
            .iter()
            .any(|filter| filter.field == "repo" && filter.value == "core"));
    }
}
