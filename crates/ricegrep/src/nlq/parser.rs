use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;

use crate::nlq::{
    error::{ProcessingError, ValidationError},
    models::{ParsedQuery, QueryComplexity},
};

pub struct QueryParser {
    pub max_length: usize,
}

impl QueryParser {
    pub fn new(max_length: usize) -> Self {
        Self { max_length }
    }

    pub fn parse(&self, raw_query: &str) -> Result<ParsedQuery, ProcessingError> {
        let trimmed = raw_query.trim();
        if trimmed.is_empty() {
            return Err(ProcessingError::Parse("query is empty".into()));
        }
        if trimmed.len() > self.max_length {
            return Err(ProcessingError::Parse(format!(
                "query exceeds max length {}",
                self.max_length
            )));
        }

        let normalized = Self::normalize(trimmed);
        let tokens = Self::tokenize(&normalized);
        let complexity = Self::determine_complexity(trimmed, tokens.len());
        let language = Self::detect_language(trimmed);

        Ok(ParsedQuery {
            original: trimmed.to_string(),
            tokens,
            language,
            complexity,
        })
    }

    pub fn validate(&self, query: &ParsedQuery) -> Result<(), ValidationError> {
        if query.original.len() > self.max_length {
            return Err(ValidationError::TooLong(
                query.original.len(),
                self.max_length,
            ));
        }
        if query.tokens.is_empty() {
            return Err(ValidationError::InvalidCharacters);
        }
        Ok(())
    }

    fn normalize(text: &str) -> String {
        text.nfkc()
            .collect::<String>()
            .chars()
            .filter(|c| !c.is_control())
            .collect::<String>()
            .to_lowercase()
    }

    fn tokenize(text: &str) -> Vec<String> {
        text.unicode_words()
            .map(|word| word.trim_matches(|c: char| !c.is_alphanumeric()))
            .filter(|word| !word.is_empty())
            .map(|word| word.to_string())
            .collect()
    }

    fn detect_language(text: &str) -> String {
        if text.chars().any(|c| matches!(c, '\u{4e00}'..='\u{9fff}' | '\u{3040}'..='\u{30ff}' | '\u{ac00}'..='\u{d7ff}' | '\u{1100}'..='\u{11ff}')) {
            return "zh".to_string();
        }
        if text
            .chars()
            .any(|c| matches!(c, '\u{00c0}'..='\u{017f}' | '\u{0100}'..='\u{024f}'))
        {
            return "es".to_string();
        }
        "en".to_string()
    }

    fn determine_complexity(original: &str, token_count: usize) -> QueryComplexity {
        let uses_punctuation =
            original.contains(',') || original.contains('?') || original.contains(" how ");
        match token_count {
            0..=4 => QueryComplexity::Simple,
            5..=9 => {
                if uses_punctuation {
                    QueryComplexity::Complex
                } else {
                    QueryComplexity::Moderate
                }
            }
            10..=20 => QueryComplexity::Complex,
            _ => QueryComplexity::VeryComplex,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nlq::models::{ParsedQuery, QueryComplexity};

    #[test]
    fn parses_tokens_and_detects_language() {
        let parser = QueryParser::new(1024);
        let parsed = parser.parse("Show me the API docs for repo:core").unwrap();
        assert_eq!(parsed.language, "en");
        assert!(parsed.tokens.contains(&"api".to_string()));
        assert!(parsed.tokens.contains(&"docs".to_string()));
    }

    #[test]
    fn classifies_complex_query() {
        let parser = QueryParser::new(1024);
        let parsed = parser
            .parse("function error handler in Rust, how to catch exceptions?")
            .unwrap();
        assert_eq!(parsed.complexity, QueryComplexity::Complex);
    }

    #[test]
    fn detects_non_english_language() {
        let parser = QueryParser::new(1024);
        let parsed = parser.parse("如何在Rust中捕获错误").unwrap();
        assert_eq!(parsed.language, "zh");
        assert!(!parsed.tokens.is_empty());
    }
}
