//! Glob pattern matching for tool names

use crate::error::{Error, Result};

/// Glob pattern matcher for matching tool names against patterns
#[derive(Debug, Clone)]
pub struct GlobMatcher;

impl GlobMatcher {
    /// Create a new glob matcher
    pub fn new() -> Self {
        Self
    }

    /// Match a tool name against a glob pattern
    ///
    /// Supports:
    /// - `*` to match any sequence of characters
    /// - `?` to match a single character
    /// - Exact matches for literal strings
    ///
    /// # Arguments
    /// * `pattern` - The glob pattern to match against
    /// * `tool_name` - The tool name to match
    ///
    /// # Returns
    /// `true` if the tool name matches the pattern, `false` otherwise
    pub fn match_pattern(&self, pattern: &str, tool_name: &str) -> bool {
        self.match_recursive(pattern.as_bytes(), tool_name.as_bytes())
    }

    /// Recursively match pattern against tool name
    #[allow(clippy::only_used_in_recursion)]
    fn match_recursive(&self, pattern: &[u8], tool_name: &[u8]) -> bool {
        match (pattern.first(), tool_name.first()) {
            // Both empty - match
            (None, None) => true,
            // Pattern empty but tool name not - no match
            (None, Some(_)) => false,
            // Tool name empty but pattern not - check if remaining pattern is all wildcards
            (Some(&b'*'), None) => self.match_recursive(&pattern[1..], tool_name),
            (Some(_), None) => false,
            // Wildcard in pattern
            (Some(&b'*'), Some(_)) => {
                // Try matching rest of pattern with rest of tool name
                // OR try matching wildcard with rest of tool name
                self.match_recursive(&pattern[1..], tool_name)
                    || self.match_recursive(pattern, &tool_name[1..])
            }
            // Question mark matches single character
            (Some(&b'?'), Some(_)) => self.match_recursive(&pattern[1..], &tool_name[1..]),
            // Literal character must match
            (Some(&p), Some(&t)) if p == t => self.match_recursive(&pattern[1..], &tool_name[1..]),
            // Characters don't match
            _ => false,
        }
    }

    /// Validate a glob pattern for syntax errors
    ///
    /// # Arguments
    /// * `pattern` - The pattern to validate
    ///
    /// # Returns
    /// `Ok(())` if the pattern is valid, `Err` with description if invalid
    pub fn validate_pattern(&self, pattern: &str) -> Result<()> {
        // Empty pattern is invalid
        if pattern.is_empty() {
            return Err(Error::InvalidGlobPattern(
                "Pattern cannot be empty".to_string(),
            ));
        }

        // Check for unmatched brackets (common glob syntax)
        let mut bracket_count = 0;
        for ch in pattern.chars() {
            match ch {
                '[' => bracket_count += 1,
                ']' => {
                    bracket_count -= 1;
                    if bracket_count < 0 {
                        return Err(Error::InvalidGlobPattern(
                            "Unmatched closing bracket ']'".to_string(),
                        ));
                    }
                }
                _ => {}
            }
        }

        if bracket_count > 0 {
            return Err(Error::InvalidGlobPattern(
                "Unmatched opening bracket '['".to_string(),
            ));
        }

        Ok(())
    }

    /// Resolve conflicts between multiple matching patterns
    ///
    /// When multiple patterns match a tool name, returns the most specific match.
    /// Priority ordering: exact match > specific pattern > wildcard
    ///
    /// # Arguments
    /// * `patterns` - List of patterns to check
    /// * `tool_name` - The tool name to match
    ///
    /// # Returns
    /// The index of the most specific matching pattern, or `None` if no patterns match
    pub fn resolve_conflicts(&self, patterns: &[&str], tool_name: &str) -> Option<usize> {
        let mut best_match: Option<(usize, PatternSpecificity)> = None;

        for (idx, pattern) in patterns.iter().enumerate() {
            if self.match_pattern(pattern, tool_name) {
                let specificity = self.calculate_specificity(pattern);

                // Update best match if this pattern is more specific
                if let Some((_, best_specificity)) = best_match {
                    if specificity > best_specificity {
                        best_match = Some((idx, specificity));
                    }
                } else {
                    best_match = Some((idx, specificity));
                }
            }
        }

        best_match.map(|(idx, _)| idx)
    }

    /// Calculate the specificity of a pattern
    ///
    /// Specificity is determined by:
    /// 1. Exact matches (no wildcards) are most specific
    /// 2. Patterns with fewer wildcards are more specific
    /// 3. Patterns with `?` are more specific than `*`
    fn calculate_specificity(&self, pattern: &str) -> PatternSpecificity {
        let has_star = pattern.contains('*');
        let has_question = pattern.contains('?');
        let star_count = pattern.matches('*').count();
        let question_count = pattern.matches('?').count();
        let pattern_len = pattern.len();

        PatternSpecificity {
            is_exact: !has_star && !has_question,
            star_count,
            question_count,
            pattern_len,
        }
    }
}

/// Represents the specificity of a pattern for conflict resolution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PatternSpecificity {
    is_exact: bool,
    star_count: usize,
    question_count: usize,
    pattern_len: usize,
}

impl PartialOrd for PatternSpecificity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PatternSpecificity {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering;

        // Exact matches are most specific
        match (self.is_exact, other.is_exact) {
            (true, false) => return Ordering::Greater,
            (false, true) => return Ordering::Less,
            _ => {}
        }

        // Fewer wildcards is more specific
        match self.star_count.cmp(&other.star_count) {
            Ordering::Equal => {}
            other_ord => return other_ord.reverse(),
        }

        // More question marks is more specific (they're more restrictive than stars)
        match self.question_count.cmp(&other.question_count) {
            Ordering::Equal => {}
            other_ord => return other_ord,
        }

        // Longer patterns are more specific
        self.pattern_len.cmp(&other.pattern_len)
    }
}

impl Default for GlobMatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_matcher_creation() {
        let _matcher = GlobMatcher::new();
        let _matcher2 = GlobMatcher::default();
    }

    #[test]
    fn test_exact_match() {
        let matcher = GlobMatcher::new();
        assert!(matcher.match_pattern("test_tool", "test_tool"));
        assert!(!matcher.match_pattern("test_tool", "test_tool2"));
        assert!(!matcher.match_pattern("test_tool", "test"));
    }

    #[test]
    fn test_wildcard_star() {
        let matcher = GlobMatcher::new();
        // Star matches any sequence
        assert!(matcher.match_pattern("test_*", "test_tool"));
        assert!(matcher.match_pattern("test_*", "test_"));
        assert!(matcher.match_pattern("test_*", "test_a_b_c"));
        assert!(!matcher.match_pattern("test_*", "other_tool"));

        // Star at beginning
        assert!(matcher.match_pattern("*_tool", "test_tool"));
        assert!(matcher.match_pattern("*_tool", "my_tool"));
        assert!(!matcher.match_pattern("*_tool", "tool"));

        // Star in middle
        assert!(matcher.match_pattern("test_*_tool", "test_my_tool"));
        assert!(matcher.match_pattern("test_*_tool", "test_a_b_tool"));
        assert!(!matcher.match_pattern("test_*_tool", "test_tool"));

        // Multiple stars
        assert!(matcher.match_pattern("*test*", "my_test_tool"));
        assert!(matcher.match_pattern("*test*", "test"));
        assert!(!matcher.match_pattern("*test*", "my_tool"));
    }

    #[test]
    fn test_wildcard_question_mark() {
        let matcher = GlobMatcher::new();
        // Question mark matches single character
        assert!(matcher.match_pattern("test_?", "test_a"));
        assert!(matcher.match_pattern("test_?", "test_1"));
        assert!(!matcher.match_pattern("test_?", "test_ab"));
        assert!(!matcher.match_pattern("test_?", "test_"));

        // Multiple question marks
        assert!(matcher.match_pattern("test_???", "test_abc"));
        assert!(!matcher.match_pattern("test_???", "test_ab"));
        assert!(!matcher.match_pattern("test_???", "test_abcd"));
    }

    #[test]
    fn test_combined_wildcards() {
        let matcher = GlobMatcher::new();
        assert!(matcher.match_pattern("test_*_?", "test_my_a"));
        assert!(matcher.match_pattern("test_*_?", "test_a_b"));
        assert!(!matcher.match_pattern("test_*_?", "test_my"));
        assert!(!matcher.match_pattern("test_*_?", "test_my_ab"));
    }

    #[test]
    fn test_universal_wildcard() {
        let matcher = GlobMatcher::new();
        assert!(matcher.match_pattern("*", "anything"));
        assert!(matcher.match_pattern("*", "test_tool"));
        assert!(matcher.match_pattern("*", "a"));
        assert!(matcher.match_pattern("*", ""));
    }

    #[test]
    fn test_empty_pattern_validation() {
        let matcher = GlobMatcher::new();
        assert!(matcher.validate_pattern("").is_err());
    }

    #[test]
    fn test_valid_pattern_validation() {
        let matcher = GlobMatcher::new();
        assert!(matcher.validate_pattern("test_*").is_ok());
        assert!(matcher.validate_pattern("*").is_ok());
        assert!(matcher.validate_pattern("test_?").is_ok());
        assert!(matcher.validate_pattern("test_tool").is_ok());
    }

    #[test]
    fn test_bracket_validation() {
        let matcher = GlobMatcher::new();
        assert!(matcher.validate_pattern("[abc]").is_ok());
        assert!(matcher.validate_pattern("test_[abc]").is_ok());
        assert!(matcher.validate_pattern("[abc").is_err());
        assert!(matcher.validate_pattern("abc]").is_err());
        assert!(matcher.validate_pattern("[abc][def]").is_ok());
    }

    #[test]
    fn test_pattern_validation_with_wildcards() {
        let matcher = GlobMatcher::new();
        assert!(matcher.validate_pattern("*").is_ok());
        assert!(matcher.validate_pattern("?").is_ok());
        assert!(matcher.validate_pattern("*?*").is_ok());
        assert!(matcher.validate_pattern("test_*_?").is_ok());
    }

    #[test]
    fn test_pattern_validation_special_chars() {
        let matcher = GlobMatcher::new();
        assert!(matcher.validate_pattern("test-tool").is_ok());
        assert!(matcher.validate_pattern("test.tool").is_ok());
        assert!(matcher.validate_pattern("test_tool").is_ok());
        assert!(matcher.validate_pattern("test:tool").is_ok());
    }

    #[test]
    fn test_pattern_validation_nested_brackets() {
        let matcher = GlobMatcher::new();
        // Nested brackets are technically valid for our simple validator
        assert!(matcher.validate_pattern("[[abc]]").is_ok());
    }

    #[test]
    fn test_pattern_validation_multiple_unmatched() {
        let matcher = GlobMatcher::new();
        assert!(matcher.validate_pattern("[[[").is_err());
        assert!(matcher.validate_pattern("]]]").is_err());
    }

    #[test]
    fn test_edge_cases() {
        let matcher = GlobMatcher::new();
        // Empty tool name
        assert!(matcher.match_pattern("*", ""));
        assert!(!matcher.match_pattern("test", ""));
        assert!(matcher.match_pattern("", ""));

        // Special characters
        assert!(matcher.match_pattern("test-*", "test-tool"));
        assert!(matcher.match_pattern("test.*", "test.py"));
        assert!(matcher.match_pattern("test_*_v?", "test_my_v1"));
    }

    #[test]
    fn test_conflict_resolution_exact_vs_pattern() {
        let matcher = GlobMatcher::new();
        let patterns = vec!["test_tool", "test_*"];
        // Exact match should win
        assert_eq!(matcher.resolve_conflicts(&patterns, "test_tool"), Some(0));
    }

    #[test]
    fn test_conflict_resolution_specific_vs_wildcard() {
        let matcher = GlobMatcher::new();
        let patterns = vec!["test_*", "*"];
        // More specific pattern should win
        assert_eq!(matcher.resolve_conflicts(&patterns, "test_tool"), Some(0));
    }

    #[test]
    fn test_conflict_resolution_question_vs_star() {
        let matcher = GlobMatcher::new();
        let patterns = vec!["test_*", "test_?"];
        // Question mark is more specific than star
        assert_eq!(matcher.resolve_conflicts(&patterns, "test_a"), Some(1));
    }

    #[test]
    fn test_conflict_resolution_no_match() {
        let matcher = GlobMatcher::new();
        let patterns = vec!["test_*", "other_*"];
        // No pattern matches
        assert_eq!(matcher.resolve_conflicts(&patterns, "unrelated"), None);
    }

    #[test]
    fn test_conflict_resolution_multiple_exact() {
        let matcher = GlobMatcher::new();
        let patterns = vec!["test_tool", "test_tool"];
        // First exact match wins
        assert_eq!(matcher.resolve_conflicts(&patterns, "test_tool"), Some(0));
    }

    #[test]
    fn test_conflict_resolution_complex() {
        let matcher = GlobMatcher::new();
        let patterns = vec!["*", "test_*", "test_tool_*", "test_tool"];
        // Most specific should win: exact match
        assert_eq!(matcher.resolve_conflicts(&patterns, "test_tool"), Some(3));
        // Next most specific: longer pattern
        assert_eq!(
            matcher.resolve_conflicts(&patterns, "test_tool_extra"),
            Some(2)
        );
        // Less specific: shorter pattern
        assert_eq!(matcher.resolve_conflicts(&patterns, "test_other"), Some(1));
        // Least specific: wildcard
        assert_eq!(matcher.resolve_conflicts(&patterns, "other"), Some(0));
    }

    #[test]
    fn test_conflict_resolution_empty_patterns() {
        let matcher = GlobMatcher::new();
        let patterns: Vec<&str> = vec![];
        assert_eq!(matcher.resolve_conflicts(&patterns, "test_tool"), None);
    }
}
