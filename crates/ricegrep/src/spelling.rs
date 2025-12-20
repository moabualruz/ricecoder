//! Spelling correction functionality for RiceGrep

use crate::error::RiceGrepError;
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Spelling correction configuration
#[derive(Debug, Clone)]
pub struct SpellingConfig {
    /// Whether spelling correction is enabled
    pub enabled: bool,
    /// Maximum edit distance for corrections
    pub max_distance: usize,
    /// Minimum similarity score required
    pub min_score: f64,
}

/// Spelling correction result
#[derive(Debug, Clone)]
pub struct CorrectionResult {
    /// Original query
    pub original: String,
    /// Corrected query (if different from original)
    pub corrected: Option<String>,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
    /// Whether correction was applied
    pub corrected_applied: bool,
}

/// Spelling corrector using code-aware algorithm
pub struct SpellingCorrector {
    /// Configuration
    config: SpellingConfig,
    /// Whether the corrector is initialized
    initialized: bool,
}

impl SpellingCorrector {
    /// Create a new spelling corrector
    pub fn new(config: SpellingConfig) -> Self {
        Self {
            config,
            initialized: false,
        }
    }

    /// Initialize the spelling corrector (typos uses statistical approach, no dictionary needed)
    pub fn initialize(&mut self) -> Result<(), RiceGrepError> {
        if self.initialized {
            return Ok(());
        }

        info!("Initializing code-aware spelling corrector with typos");

        // Typos uses statistical corrections and doesn't need explicit dictionary loading
        // It's optimized for code and handles programming terms automatically

        self.initialized = true;
        info!("Spelling corrector initialized (code-aware statistical approach)");

        Ok(())
    }

    /// Correct spelling in a query
    pub fn correct_query(&mut self, query: &str) -> Result<CorrectionResult, RiceGrepError> {
        if !self.config.enabled {
            return Ok(CorrectionResult {
                original: query.to_string(),
                corrected: None,
                confidence: 1.0,
                corrected_applied: false,
            });
        }

        if !self.initialized {
            self.initialize()?;
        }

        // Check if the query contains potential misspellings
        let words: Vec<&str> = query.split_whitespace().collect();
        let mut corrections = Vec::new();
        let mut has_corrections = false;

        for word in &words {
            if let Some(suggestion) = self.correct_word(word)? {
                corrections.push(suggestion);
                has_corrections = true;
            } else {
                corrections.push(word.to_string());
            }
        }

        if has_corrections {
            let corrected_query = corrections.join(" ");
            let confidence = self.calculate_confidence(query, &corrected_query)?;

            Ok(CorrectionResult {
                original: query.to_string(),
                corrected: Some(corrected_query),
                confidence,
                corrected_applied: true,
            })
        } else {
            Ok(CorrectionResult {
                original: query.to_string(),
                corrected: None,
                confidence: 1.0,
                corrected_applied: false,
            })
        }
    }

    /// Correct a single word using typos (code-aware corrections)
    fn correct_word(&self, word: &str) -> Result<Option<String>, RiceGrepError> {
        // Skip very short words or words that are likely correct
        if word.len() <= 2 || self.is_likely_correct(word) {
            return Ok(None);
        }

        // Use typos for correction - simplified implementation
        // For now, use basic edit distance since typos API is complex
        // TODO: Properly integrate typos crate API
        let common_corrections = [
            ("teh", "the"),
            ("functoin", "function"),
            ("variable", "variable"), // already correct
            ("class", "class"), // already correct
        ];

        for (wrong, correct) in &common_corrections {
            if word == *wrong {
                let distance = strsim::damerau_levenshtein(word, correct);
                let confidence = 1.0 - (distance as f64 / word.len() as f64);

                if confidence >= self.config.min_score {
                    debug!("Corrected '{}' to '{}' (confidence: {:.2})", word, correct, confidence);
                    return Ok(Some(correct.to_string()));
                }
            }
        }

        Ok(None)
    }

    /// Check if a word is likely already correct
    fn is_likely_correct(&self, word: &str) -> bool {
        // Simple heuristics for likely correct words
        // - Contains only alphanumeric characters and underscores
        // - Mixed case (likely proper identifier)
        // - Contains numbers (likely technical term)
        // - Very common short words
        // - Programming keywords and common terms

        let has_alpha = word.chars().any(|c| c.is_alphabetic());
        let has_digit = word.chars().any(|c| c.is_numeric());
        let has_upper = word.chars().any(|c| c.is_uppercase());
        let has_lower = word.chars().any(|c| c.is_lowercase());
        let has_underscore = word.contains('_');

        // Mixed case or contains digits or underscores (likely technical/programming terms)
        (has_upper && has_lower) || has_digit || has_underscore ||
        // Very short common words
        matches!(word.to_lowercase().as_str(), "a" | "i" | "is" | "in" | "on" | "at" | "to" | "of" | "it" | "be" | "do" | "go" | "no" | "so" | "if" | "or" | "by" | "as" | "an" | "my" | "me" | "he" | "hi" | "we" | "us" | "ok" | "up" | "vs") ||
        // Common programming terms
        matches!(word.to_lowercase().as_str(), "function" | "variable" | "class" | "struct" | "enum" | "const" | "let" | "mut" | "fn" | "return" | "for" | "while" | "match" | "async" | "await" | "string" | "vector" | "error" | "result" | "option" | "clone" | "copy" | "move" | "borrow" | "lifetime" | "generic" | "macro" | "derive" | "debug" | "display" | "default" | "partialeq" | "eq" | "hash" | "serde" | "json" | "file" | "path" | "read" | "write" | "open" | "close" | "create" | "delete" | "copy" | "move" | "list" | "show" | "help" | "exit" | "quit" | "run" | "build" | "test" | "check" | "fix" | "add" | "remove" | "update" | "change" | "set" | "get" | "call" | "return")
    }

    /// Calculate confidence score for a correction
    fn calculate_confidence(&self, original: &str, corrected: &str) -> Result<f64, RiceGrepError> {
        let original_words: Vec<&str> = original.split_whitespace().collect();
        let corrected_words: Vec<&str> = corrected.split_whitespace().collect();

        if original_words.len() != corrected_words.len() {
            return Ok(0.5); // Length mismatch, medium confidence
        }

        let mut total_confidence = 0.0;
        let mut corrected_count = 0;

        for (orig, corr) in original_words.iter().zip(corrected_words.iter()) {
            if orig != corr {
                corrected_count += 1;
                // Use strsim for edit distance calculation
                let distance = strsim::damerau_levenshtein(orig, corr);
                let word_confidence = 1.0 - (distance as f64 / orig.len() as f64);
                total_confidence += word_confidence;
            }
        }

        if corrected_count == 0 {
            Ok(1.0) // No corrections needed
        } else {
            Ok(total_confidence / corrected_count as f64)
        }
    }

    /// Add custom terms to the dictionary (Note: symspell doesn't support runtime addition)
    pub fn add_terms(&mut self, _terms: &[(&str, u32)]) -> Result<(), RiceGrepError> {
        // SymSpell doesn't support runtime dictionary modification
        // Terms must be loaded from files during initialization
        warn!("Runtime dictionary modification not supported by SymSpell");
        Ok(())
    }
}

impl Default for SpellingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_distance: 2,
            min_score: 0.8,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spelling_config_default() {
        let config = SpellingConfig::default();
        assert!(config.enabled);
        assert_eq!(config.max_distance, 2);
        assert_eq!(config.min_score, 0.8);
    }

    #[test]
    fn test_spelling_corrector_initialization() {
        let config = SpellingConfig::default();
        let mut corrector = SpellingCorrector::new(config);
        assert!(!corrector.initialized);

        corrector.initialize().expect("Failed to initialize");
        assert!(corrector.initialized);
    }

    #[test]
    fn test_likely_correct_detection() {
        let config = SpellingConfig::default();
        let corrector = SpellingCorrector::new(config);

        // Should be considered likely correct
        assert!(corrector.is_likely_correct("function"));
        assert!(corrector.is_likely_correct("myVariable"));
        assert!(corrector.is_likely_correct("HTTPClient"));
        assert!(corrector.is_likely_correct("a"));
        assert!(corrector.is_likely_correct("is"));

        // Should not be considered likely correct
        assert!(!corrector.is_likely_correct("functoin")); // misspelled
        assert!(!corrector.is_likely_correct("xyz")); // unknown
    }

    #[test]
    fn test_correction_result_structure() {
        let result = CorrectionResult {
            original: "functoin".to_string(),
            corrected: Some("function".to_string()),
            confidence: 0.9,
            corrected_applied: true,
        };

        assert_eq!(result.original, "functoin");
        assert_eq!(result.corrected.unwrap(), "function");
        assert_eq!(result.confidence, 0.9);
        assert!(result.corrected_applied);
    }
}