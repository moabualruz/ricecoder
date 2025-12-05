//! Pattern matching for refactoring rules

use crate::error::Result;
use crate::types::{RefactoringRule, RefactoringType};
use regex::Regex;
use std::collections::HashMap;

/// Matches refactoring patterns in code
pub struct PatternMatcher;

/// Result of pattern matching
#[derive(Debug, Clone)]
pub struct MatchResult {
    /// Whether the pattern matched
    pub matched: bool,
    /// Matched text
    pub matched_text: Option<String>,
    /// Match position (line, column)
    pub position: Option<(usize, usize)>,
    /// Captured groups
    pub captures: HashMap<String, String>,
}

impl PatternMatcher {
    /// Match a rule pattern against code
    pub fn match_rule(code: &str, rule: &RefactoringRule) -> Result<Vec<MatchResult>> {
        let mut results = vec![];

        // Try regex matching first
        if let Ok(re) = Regex::new(&rule.pattern) {
            for (line_num, line) in code.lines().enumerate() {
                for mat in re.find_iter(line) {
                    results.push(MatchResult {
                        matched: true,
                        matched_text: Some(mat.as_str().to_string()),
                        position: Some((line_num + 1, mat.start())),
                        captures: HashMap::new(),
                    });
                }
            }
        } else {
            // Fall back to simple string matching
            for (line_num, line) in code.lines().enumerate() {
                if line.contains(&rule.pattern) {
                    results.push(MatchResult {
                        matched: true,
                        matched_text: Some(rule.pattern.clone()),
                        position: Some((line_num + 1, 0)),
                        captures: HashMap::new(),
                    });
                }
            }
        }

        Ok(results)
    }

    /// Match multiple rules against code
    pub fn match_rules(code: &str, rules: &[RefactoringRule]) -> Result<HashMap<String, Vec<MatchResult>>> {
        let mut all_matches = HashMap::new();

        for rule in rules {
            if rule.enabled {
                let matches = Self::match_rule(code, rule)?;
                if !matches.is_empty() {
                    all_matches.insert(rule.name.clone(), matches);
                }
            }
        }

        Ok(all_matches)
    }

    /// Check if a refactoring type is applicable based on matched patterns
    pub fn is_applicable(
        refactoring_type: RefactoringType,
        matched_patterns: &[&str],
    ) -> bool {
        match refactoring_type {
            RefactoringType::Rename => {
                // Rename is applicable if we found a symbol
                !matched_patterns.is_empty()
            }
            RefactoringType::RemoveUnused => {
                // RemoveUnused is applicable if we found unused code patterns
                matched_patterns.iter().any(|p| p.contains("unused"))
            }
            RefactoringType::Extract => {
                // Extract is applicable if we found extractable code
                !matched_patterns.is_empty()
            }
            RefactoringType::Inline => {
                // Inline is applicable if we found inlinable code
                !matched_patterns.is_empty()
            }
            RefactoringType::Move => {
                // Move is applicable if we found movable code
                !matched_patterns.is_empty()
            }
            RefactoringType::ChangeSignature => {
                // ChangeSignature is applicable if we found function signatures
                matched_patterns.iter().any(|p| p.contains("fn") || p.contains("function"))
            }
            RefactoringType::Simplify => {
                // Simplify is applicable if we found simplifiable patterns
                !matched_patterns.is_empty()
            }
        }
    }

    /// Extract captures from a regex match
    pub fn extract_captures(pattern: &str, text: &str) -> Result<HashMap<String, String>> {
        let mut captures = HashMap::new();

        if let Ok(re) = Regex::new(pattern) {
            if let Some(caps) = re.captures(text) {
                for (i, cap) in caps.iter().enumerate() {
                    if let Some(m) = cap {
                        captures.insert(format!("group_{}", i), m.as_str().to_string());
                    }
                }
            }
        }

        Ok(captures)
    }

    /// Count matches of a pattern in code
    pub fn count_matches(code: &str, pattern: &str) -> Result<usize> {
        if let Ok(re) = Regex::new(pattern) {
            Ok(re.find_iter(code).count())
        } else {
            Ok(code.matches(pattern).count())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_rule_regex() -> Result<()> {
        let rule = RefactoringRule {
            name: "test".to_string(),
            pattern: r"\bfn\s+\w+".to_string(),
            refactoring_type: RefactoringType::Rename,
            enabled: true,
        };

        let code = "fn main() {}\nfn helper() {}";
        let results = PatternMatcher::match_rule(code, &rule)?;

        assert_eq!(results.len(), 2);
        assert!(results[0].matched);

        Ok(())
    }

    #[test]
    fn test_match_rule_string() -> Result<()> {
        let rule = RefactoringRule {
            name: "test".to_string(),
            pattern: "unused".to_string(),
            refactoring_type: RefactoringType::RemoveUnused,
            enabled: true,
        };

        let code = "let unused = 5;\nlet used = 10;";
        let results = PatternMatcher::match_rule(code, &rule)?;

        assert_eq!(results.len(), 1);
        assert!(results[0].matched);

        Ok(())
    }

    #[test]
    fn test_match_rules() -> Result<()> {
        let rules = vec![
            RefactoringRule {
                name: "rule1".to_string(),
                pattern: "fn".to_string(),
                refactoring_type: RefactoringType::Rename,
                enabled: true,
            },
            RefactoringRule {
                name: "rule2".to_string(),
                pattern: "unused".to_string(),
                refactoring_type: RefactoringType::RemoveUnused,
                enabled: true,
            },
        ];

        let code = "fn main() {}\nlet unused = 5;";
        let results = PatternMatcher::match_rules(code, &rules)?;

        assert_eq!(results.len(), 2);
        assert!(results.contains_key("rule1"));
        assert!(results.contains_key("rule2"));

        Ok(())
    }

    #[test]
    fn test_is_applicable() {
        assert!(PatternMatcher::is_applicable(
            RefactoringType::Rename,
            &["symbol"]
        ));
        assert!(PatternMatcher::is_applicable(
            RefactoringType::RemoveUnused,
            &["unused_var"]
        ));
        assert!(!PatternMatcher::is_applicable(
            RefactoringType::RemoveUnused,
            &["used_var"]
        ));
    }

    #[test]
    fn test_count_matches() -> Result<()> {
        let count = PatternMatcher::count_matches("fn main() {}\nfn helper() {}", r"\bfn\b")?;
        assert_eq!(count, 2);

        Ok(())
    }
}
