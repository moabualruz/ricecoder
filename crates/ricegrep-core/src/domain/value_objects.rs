//! Domain Value Objects
//!
//! Pure, immutable, validated types that represent domain concepts.
//! Value objects validate structure and business rules only - no I/O operations.

use std::path::{Path, PathBuf};
use crate::domain::errors::{DomainError, DomainResult};

/// A validated file path for domain operations
///
/// # Design Principles
/// - Pure: validates structure only, no filesystem I/O
/// - Immutable once constructed
/// - Zero external dependencies
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct FilePath {
    path: PathBuf,
}

impl FilePath {
    /// Create a new FilePath with structural validation
    pub fn new<P: AsRef<Path>>(path: P) -> DomainResult<Self> {
        let path_buf = path.as_ref().to_path_buf();
        
        if path_buf.as_os_str().is_empty() {
            return Err(DomainError::InvalidFilePath("Path cannot be empty".to_string()));
        }
        
        if path_buf.to_string_lossy().contains('\0') {
            return Err(DomainError::InvalidFilePath("Path cannot contain null bytes".to_string()));
        }
        
        Ok(FilePath { path: path_buf })
    }
    
    pub fn as_path(&self) -> &Path { &self.path }
    pub fn file_name(&self) -> Option<&str> { self.path.file_name()?.to_str() }
    pub fn extension(&self) -> Option<&str> { self.path.extension()?.to_str() }
    
    pub fn is_likely_binary(&self) -> bool {
        let binary_extensions = ["exe", "dll", "so", "dylib", "a", "o", "obj",
            "png", "jpg", "jpeg", "gif", "bmp", "ico", "pdf", "zip", "tar", "gz", "bz2", "7z"];
        self.extension().map(|ext| binary_extensions.iter().any(|&b| ext.eq_ignore_ascii_case(b))).unwrap_or(false)
    }
    
    pub fn into_path_buf(self) -> PathBuf { self.path }
}

/// A validated edit pattern (regex or literal string)
#[derive(Debug, Clone, PartialEq)]
pub struct EditPattern {
    pattern: String,
    is_regex: bool,
}

impl EditPattern {
    pub fn new(pattern: &str, is_regex: bool) -> DomainResult<Self> {
        if pattern.is_empty() {
            return Err(DomainError::InvalidEditPattern("Pattern cannot be empty".to_string()));
        }
        if is_regex && Self::is_valid_regex_basic(pattern).is_err() {
            return Err(DomainError::InvalidEditPattern(format!("Invalid regex syntax: {}", pattern)));
        }
        Ok(EditPattern { pattern: pattern.to_string(), is_regex })
    }
    
    pub fn pattern(&self) -> &str { &self.pattern }
    pub fn is_regex(&self) -> bool { self.is_regex }
    pub fn literal(pattern: &str) -> DomainResult<Self> { Self::new(pattern, false) }
    pub fn regex(pattern: &str) -> DomainResult<Self> { Self::new(pattern, true) }
    
    pub(crate) fn is_valid_regex_basic(pattern: &str) -> Result<(), ()> {
        let mut paren_count = 0i32;
        let mut bracket_count = 0i32;
        let mut escape_next = false;
        
        for ch in pattern.chars() {
            if escape_next { escape_next = false; continue; }
            match ch {
                '\\' => escape_next = true,
                '(' => paren_count += 1,
                ')' => { paren_count -= 1; if paren_count < 0 { return Err(()); } },
                '[' => bracket_count += 1,
                ']' => { bracket_count -= 1; if bracket_count < 0 { return Err(()); } },
                _ => {}
            }
        }
        if paren_count != 0 || bracket_count != 0 { return Err(()); }
        Ok(())
    }
}

/// A validated search query with options
#[derive(Debug, Clone, PartialEq)]
pub struct SearchQuery {
    query: String,
    case_sensitive: bool,
    whole_words: bool,
    is_regex: bool,
}

impl SearchQuery {
    pub fn new(query: &str, case_sensitive: bool, whole_words: bool, is_regex: bool) -> DomainResult<Self> {
        if query.is_empty() {
            return Err(DomainError::InvalidSearchQuery("Query cannot be empty".to_string()));
        }
        if is_regex && EditPattern::is_valid_regex_basic(query).is_err() {
            return Err(DomainError::InvalidSearchQuery(format!("Invalid regex syntax: {}", query)));
        }
        Ok(SearchQuery { query: query.to_string(), case_sensitive, whole_words, is_regex })
    }
    
    pub fn query(&self) -> &str { &self.query }
    pub fn is_case_sensitive(&self) -> bool { self.case_sensitive }
    pub fn is_whole_words(&self) -> bool { self.whole_words }
    pub fn is_regex(&self) -> bool { self.is_regex }
    pub fn simple(query: &str) -> DomainResult<Self> { Self::new(query, false, false, false) }
    pub fn case_sensitive(query: &str) -> DomainResult<Self> { Self::new(query, true, false, false) }
    pub fn regex(query: &str) -> DomainResult<Self> { Self::new(query, false, false, true) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_path_validation() {
        let file_path = FilePath::new("src/main.rs").unwrap();
        assert_eq!(file_path.file_name(), Some("main.rs"));
    }
    
    #[test]
    fn test_file_path_empty_fails() {
        assert!(FilePath::new("").is_err());
    }
    
    #[test]
    fn test_edit_pattern_literal() {
        let pattern = EditPattern::literal("hello").unwrap();
        assert!(!pattern.is_regex());
    }
    
    #[test]
    fn test_search_query_simple() {
        let query = SearchQuery::simple("test").unwrap();
        assert!(!query.is_case_sensitive());
    }
}
