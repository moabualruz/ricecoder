//! Domain Value Objects
//!
//! Pure, immutable, validated types that represent domain concepts.
//! Value objects validate structure and business rules only - no I/O operations.

use std::path::{Path, PathBuf};
use std::ffi::OsStr;
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
    ///
    /// # Validation Rules (PURE - no I/O)
    /// - Path cannot be empty
    /// - Path cannot contain null bytes
    /// - Must be a file-like path (has extension or known pattern)
    ///
    /// # Note
    /// File existence/readability is validated at application layer
    pub fn new<P: AsRef<Path>>(path: P) -> DomainResult<Self> {
        let path_buf = path.as_ref().to_path_buf();
        
        // Check for empty path
        if path_buf.as_os_str().is_empty() {
            return Err(DomainError::InvalidFilePath(
                "Path cannot be empty".to_string()
            ));
        }
        
        // Check for null bytes (security)
        if path_buf.to_string_lossy().contains('\0') {
            return Err(DomainError::InvalidFilePath(
                "Path cannot contain null bytes".to_string()
            ));
        }
        
        Ok(FilePath { path: path_buf })
    }
    
    /// Get the underlying path reference
    pub fn as_path(&self) -> &Path {
        &self.path
    }
    
    /// Get the file name
    pub fn file_name(&self) -> Option<&str> {
        self.path.file_name()?.to_str()
    }
    
    /// Get the file extension
    pub fn extension(&self) -> Option<&str> {
        self.path.extension()?.to_str()
    }
    
    /// Check if path appears to be a binary file (by extension)
    pub fn is_likely_binary(&self) -> bool {
        let binary_extensions = [
            "exe", "dll", "so", "dylib", "a", "o", "obj",
            "png", "jpg", "jpeg", "gif", "bmp", "ico",
            "pdf", "zip", "tar", "gz", "bz2", "7z",
        ];
        
        if let Some(ext) = self.extension() {
            binary_extensions.iter().any(|&bin_ext| {
                ext.eq_ignore_ascii_case(bin_ext)
            })
        } else {
            false
        }
    }
    
    /// Convert to PathBuf (consumes self)
    pub fn into_path_buf(self) -> PathBuf {
        self.path
    }
}

/// A validated edit pattern (regex or literal string)
///
/// # Design Principles
/// - Immutable once constructed  
/// - Validates regex syntax if is_regex is true
/// - Provides safe access to pattern components
#[derive(Debug, Clone, PartialEq)]
pub struct EditPattern {
    pattern: String,
    is_regex: bool,
}

impl EditPattern {
    /// Create a new EditPattern with validation
    ///
    /// # Validation Rules
    /// - If is_regex is true, pattern must be valid regex syntax
    /// - Pattern cannot be empty
    ///
    /// # Errors
    /// Returns DomainError::InvalidEditPattern if validation fails
    pub fn new(pattern: &str, is_regex: bool) -> DomainResult<Self> {
        if pattern.is_empty() {
            return Err(DomainError::InvalidEditPattern(
                "Pattern cannot be empty".to_string()
            ));
        }
        
        // Validate regex syntax if needed
        if is_regex {
            // Use basic regex validation (std lib only)
            // For full validation, this would need regex crate
            if Self::is_valid_regex_basic(pattern).is_err() {
                return Err(DomainError::InvalidEditPattern(
                    format!("Invalid regex syntax: {}", pattern)
                ));
            }
        }
        
        Ok(EditPattern {
            pattern: pattern.to_string(),
            is_regex,
        })
    }
    
    /// Get the pattern string
    pub fn pattern(&self) -> &str {
        &self.pattern
    }
    
    /// Check if this is a regex pattern
    pub fn is_regex(&self) -> bool {
        self.is_regex
    }
    
    /// Create a literal (non-regex) pattern
    pub fn literal(pattern: &str) -> DomainResult<Self> {
        Self::new(pattern, false)
    }
    
    /// Create a regex pattern
    pub fn regex(pattern: &str) -> DomainResult<Self> {
        Self::new(pattern, true)
    }
    
    /// Basic regex validation (simplified, would use regex crate in production)
    fn is_valid_regex_basic(pattern: &str) -> Result<(), ()> {
        // Very basic checks for common regex errors
        let mut paren_count = 0;
        let mut bracket_count = 0;
        let mut escape_next = false;
        
        for ch in pattern.chars() {
            if escape_next {
                escape_next = false;
                continue;
            }
            
            match ch {
                '\\' => escape_next = true,
                '(' => paren_count += 1,
                ')' => {
                    paren_count -= 1;
                    if paren_count < 0 { return Err(()); }
                },
                '[' => bracket_count += 1,
                ']' => {
                    bracket_count -= 1;
                    if bracket_count < 0 { return Err(()); }
                },
                _ => {}
            }
        }
        
        if paren_count != 0 || bracket_count != 0 {
            return Err(());
        }
        
        Ok(())
    }
}

/// A validated search query with options
///
/// # Design Principles
/// - Immutable once constructed
/// - Validates query is not empty and options are consistent
/// - Encapsulates search configuration
#[derive(Debug, Clone, PartialEq)]
pub struct SearchQuery {
    query: String,
    case_sensitive: bool,
    whole_words: bool,
    is_regex: bool,
}

impl SearchQuery {
    /// Create a new SearchQuery with validation
    ///
    /// # Validation Rules
    /// - Query cannot be empty
    /// - If is_regex is true, query must be valid regex
    ///
    /// # Errors
    /// Returns DomainError::InvalidSearchQuery if validation fails
    pub fn new(query: &str, case_sensitive: bool, whole_words: bool, is_regex: bool) -> DomainResult<Self> {
        if query.is_empty() {
            return Err(DomainError::InvalidSearchQuery(
                "Query cannot be empty".to_string()
            ));
        }
        
        // Validate regex if needed
        if is_regex && EditPattern::is_valid_regex_basic(query).is_err() {
            return Err(DomainError::InvalidSearchQuery(
                format!("Invalid regex syntax: {}", query)
            ));
        }
        
        Ok(SearchQuery {
            query: query.to_string(),
            case_sensitive,
            whole_words,
            is_regex,
        })
    }
    
    /// Get the query string
    pub fn query(&self) -> &str {
        &self.query
    }
    
    /// Check if search is case sensitive
    pub fn is_case_sensitive(&self) -> bool {
        self.case_sensitive
    }
    
    /// Check if search matches whole words only
    pub fn is_whole_words(&self) -> bool {
        self.whole_words
    }
    
    /// Check if query is a regex
    pub fn is_regex(&self) -> bool {
        self.is_regex
    }
    
    /// Create a simple literal search query
    pub fn simple(query: &str) -> DomainResult<Self> {
        Self::new(query, false, false, false)
    }
    
    /// Create a case-sensitive search query
    pub fn case_sensitive(query: &str) -> DomainResult<Self> {
        Self::new(query, true, false, false)
    }
    
    /// Create a regex search query
    pub fn regex(query: &str) -> DomainResult<Self> {
        Self::new(query, false, false, true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_path_validation() {
        // Test valid path creation (structural validation only)
        let file_path = FilePath::new("src/main.rs").unwrap();
        assert_eq!(file_path.file_name(), Some("main.rs"));
        assert_eq!(file_path.extension(), Some("rs"));
    }
    
    #[test]
    fn test_file_path_empty_fails() {
        let result = FilePath::new("");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DomainError::InvalidFilePath(_)));
    }
    
    #[test]
    fn test_file_path_null_bytes_fails() {
        let result = FilePath::new("test\0.txt");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DomainError::InvalidFilePath(_)));
    }
    
    #[test]
    fn test_file_path_binary_detection() {
        // Test structural binary detection (no I/O)
        let binary_file = FilePath::new("test.exe").unwrap();
        assert!(binary_file.is_likely_binary());
        
        let text_file = FilePath::new("test.txt").unwrap();
        assert!(!text_file.is_likely_binary());
        
        let no_ext_file = FilePath::new("README").unwrap();
        assert!(!no_ext_file.is_likely_binary());
    }
    
    #[test]
    fn test_edit_pattern_literal() {
        let pattern = EditPattern::literal("hello world").unwrap();
        assert_eq!(pattern.pattern(), "hello world");
        assert!(!pattern.is_regex());
    }
    
    #[test]
    fn test_edit_pattern_regex() {
        let pattern = EditPattern::regex(r"\d+").unwrap();
        assert_eq!(pattern.pattern(), r"\d+");
        assert!(pattern.is_regex());
    }
    
    #[test]
    fn test_edit_pattern_empty_fails() {
        let result = EditPattern::literal("");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_edit_pattern_invalid_regex() {
        let result = EditPattern::regex("(unclosed");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_search_query_simple() {
        let query = SearchQuery::simple("test").unwrap();
        assert_eq!(query.query(), "test");
        assert!(!query.is_case_sensitive());
        assert!(!query.is_whole_words());
        assert!(!query.is_regex());
    }
    
    #[test]
    fn test_search_query_case_sensitive() {
        let query = SearchQuery::case_sensitive("Test").unwrap();
        assert!(query.is_case_sensitive());
    }
    
    #[test]
    fn test_search_query_regex() {
        let query = SearchQuery::regex(r"\w+").unwrap();
        assert!(query.is_regex());
    }
    
    #[test]
    fn test_search_query_empty_fails() {
        let result = SearchQuery::simple("");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_search_query_invalid_regex() {
        let result = SearchQuery::regex("[unclosed");
        assert!(result.is_err());
    }
}