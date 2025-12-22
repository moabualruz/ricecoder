use crate::nlq::{
    error::ProcessingError,
    models::{ParsedQuery, QueryIntent},
};

pub struct IntentClassifier {
    rules: Vec<IntentRule>,
}

struct IntentRule {
    intent: QueryIntent,
    keywords: Vec<String>,
    confidence: f32,
}

impl IntentRule {
    fn new(intent: QueryIntent, keywords: &[&str], confidence: f32) -> Self {
        Self {
            intent,
            keywords: keywords.iter().map(|kw| kw.to_string()).collect(),
            confidence,
        }
    }

    fn match_count(&self, query: &ParsedQuery) -> usize {
        let text = query.original.to_lowercase();
        self.keywords
            .iter()
            .filter(|keyword| {
                let keyword_str = keyword.as_str();
                text.contains(keyword_str) || query.tokens.iter().any(|token| token == keyword_str)
            })
            .count()
    }
}

impl IntentClassifier {
    pub fn new(rules: Vec<IntentRule>) -> Self {
        Self { rules }
    }

    pub fn with_defaults() -> Self {
        Self::new(Self::default_rules())
    }

    pub async fn classify(&self, query: &ParsedQuery) -> Result<QueryIntent, ProcessingError> {
        let mut best = (QueryIntent::GeneralSearch, 0.55, 0);
        let normalized = query.original.to_lowercase();
        if normalized.contains("documentation") || normalized.contains("docs") {
            return Ok(QueryIntent::DocumentationSearch);
        }
        for rule in &self.rules {
            let matches = rule.match_count(query);
            if matches == 0 {
                continue;
            }
            if rule.confidence > best.1 || (rule.confidence == best.1 && matches > best.2) {
                best = (rule.intent.clone(), rule.confidence, matches);
            }
        }
        Ok(best.0)
    }

    pub async fn confidence_score(&self, intent: &QueryIntent) -> f32 {
        self.rules
            .iter()
            .find(|rule| &rule.intent == intent)
            .map(|rule| rule.confidence)
            .unwrap_or(0.6)
    }

    fn default_rules() -> Vec<IntentRule> {
        vec![
            IntentRule::new(
                QueryIntent::CodeSearch,
                &["function", "method", "implementation", "snippet", "code"],
                0.92,
            ),
            IntentRule::new(
                QueryIntent::APISearch,
                &["api", "endpoint", "rest", "graphql", "http"],
                0.88,
            ),
            IntentRule::new(
                QueryIntent::DocumentationSearch,
                &["doc", "documentation", "guide", "reference", "tutorial"],
                0.85,
            ),
            IntentRule::new(
                QueryIntent::BugReportSearch,
                &["error", "bug", "issue", "exception", "stacktrace"],
                0.9,
            ),
        ]
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
    async fn classifies_documentation_queries() {
        let classifier = IntentClassifier::with_defaults();
        let parsed = stub_query("Show me the API documentation for the search endpoint");
        let intent = classifier.classify(&parsed).await.unwrap();
        assert_eq!(intent, QueryIntent::DocumentationSearch);
    }

    #[tokio::test]
    async fn falls_back_to_general_search() {
        let classifier = IntentClassifier::with_defaults();
        let parsed = stub_query("pizza recipes");
        let intent = classifier.classify(&parsed).await.unwrap();
        assert_eq!(intent, QueryIntent::GeneralSearch);
    }
}
