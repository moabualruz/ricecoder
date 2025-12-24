//! Search Files Use Case
//!
//! Orchestrates search operations across indexed files.

use crate::application::{AppResult, IndexRepository, EventPublisher};
use crate::domain::{SearchQuery, SearchResult, DomainEvent};

/// Request for searching files
#[derive(Debug, Clone)]
pub struct SearchFilesRequest {
    /// Search pattern (literal or regex)
    pub pattern: String,
    /// Whether pattern is regex
    pub is_regex: bool,
    /// Whether search is case-sensitive
    pub case_sensitive: bool,
    /// Whether to match whole words only
    pub whole_word: bool,
    /// Optional path filter
    pub path_filter: Option<String>,
    /// Maximum results to return
    pub max_results: Option<usize>,
}

impl SearchFilesRequest {
    /// Create a simple literal search request
    pub fn literal(pattern: impl Into<String>) -> Self {
        SearchFilesRequest {
            pattern: pattern.into(),
            is_regex: false,
            case_sensitive: true,
            whole_word: false,
            path_filter: None,
            max_results: None,
        }
    }
    
    /// Create a regex search request
    pub fn regex(pattern: impl Into<String>) -> Self {
        SearchFilesRequest {
            pattern: pattern.into(),
            is_regex: true,
            case_sensitive: true,
            whole_word: false,
            path_filter: None,
            max_results: None,
        }
    }
}

/// Response from search operation
#[derive(Debug, Clone)]
pub struct SearchFilesResponse {
    /// Search results
    pub results: Vec<SearchResult>,
    /// Total matches found
    pub total_matches: usize,
    /// Whether results were truncated
    pub truncated: bool,
}

/// Use case for searching files
///
/// # Example
/// ```ignore
/// let use_case = SearchFilesUseCase::new(index_repo, event_publisher);
/// let request = SearchFilesRequest::literal("TODO");
/// let response = use_case.execute(request)?;
/// ```
pub struct SearchFilesUseCase<I: IndexRepository, E: EventPublisher> {
    index_repo: I,
    event_publisher: E,
}

impl<I: IndexRepository, E: EventPublisher> SearchFilesUseCase<I, E> {
    /// Create a new search files use case
    pub fn new(index_repo: I, event_publisher: E) -> Self {
        SearchFilesUseCase {
            index_repo,
            event_publisher,
        }
    }

    /// Execute the search operation
    pub fn execute(&self, request: SearchFilesRequest) -> AppResult<SearchFilesResponse> {
        // 1. Create domain search query
        // SearchQuery::new(query, case_sensitive, whole_words, is_regex)
        let query = SearchQuery::new(
            &request.pattern,
            request.case_sensitive,
            request.whole_word,
            request.is_regex,
        ).map_err(|e| crate::application::AppError::Validation { 
            message: e.to_string() 
        })?;
        
        // 2. Execute search via repository
        let mut results = self.index_repo.search(&query)?;
        
        // 3. Apply path filter if specified
        if let Some(ref filter) = request.path_filter {
            results.retain(|r| {
                let path_str = r.file_path().as_path().to_string_lossy();
                path_str.contains(filter)
            });
        }
        
        // 4. Apply max results limit
        let truncated = if let Some(max) = request.max_results {
            if results.len() > max {
                results.truncate(max);
                true
            } else {
                false
            }
        } else {
            false
        };
        
        // 5. Calculate total matches
        let total_matches: usize = results.iter()
            .map(|r| r.matches().len())
            .sum();
        
        // 6. Publish search events
        for result in &results {
            self.event_publisher.publish(&DomainEvent::SearchExecuted {
                file_path: result.file_path().as_path().to_string_lossy().to_string(),
                matches_found: result.matches().len(),
            });
        }
        
        Ok(SearchFilesResponse {
            results,
            total_matches,
            truncated,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::FileIndexEntry;
    use crate::domain::FilePath;
    use std::cell::RefCell;
    use std::collections::HashMap;

    // Test doubles
    struct TestIndexRepo {
        entries: RefCell<HashMap<String, FileIndexEntry>>,
    }

    impl TestIndexRepo {
        fn new() -> Self {
            TestIndexRepo {
                entries: RefCell::new(HashMap::new()),
            }
        }
    }

    impl IndexRepository for TestIndexRepo {
        fn get_metadata(&self, path: &FilePath) -> Option<FileIndexEntry> {
            let path_str = path.as_path().to_string_lossy().to_string();
            self.entries.borrow().get(&path_str).cloned()
        }
        
        fn update_metadata(&self, entry: FileIndexEntry) -> AppResult<()> {
            self.entries.borrow_mut().insert(entry.path.clone(), entry);
            Ok(())
        }
        
        fn remove_metadata(&self, path: &FilePath) -> AppResult<()> {
            let path_str = path.as_path().to_string_lossy().to_string();
            self.entries.borrow_mut().remove(&path_str);
            Ok(())
        }
        
        fn search(&self, _query: &SearchQuery) -> AppResult<Vec<SearchResult>> {
            // Mock returns empty results for now
            Ok(vec![])
        }
    }

    struct TestEventPublisher {
        events: RefCell<Vec<DomainEvent>>,
    }

    impl TestEventPublisher {
        fn new() -> Self {
            TestEventPublisher { events: RefCell::new(Vec::new()) }
        }
        
        fn event_count(&self) -> usize {
            self.events.borrow().len()
        }
    }

    impl EventPublisher for TestEventPublisher {
        fn publish(&self, event: &DomainEvent) {
            self.events.borrow_mut().push(event.clone());
        }
    }

    #[test]
    fn test_search_literal_request() {
        let request = SearchFilesRequest::literal("TODO");
        
        assert_eq!(request.pattern, "TODO");
        assert!(!request.is_regex);
        assert!(request.case_sensitive);
    }

    #[test]
    fn test_search_regex_request() {
        let request = SearchFilesRequest::regex(r"fn\s+\w+");
        
        assert_eq!(request.pattern, r"fn\s+\w+");
        assert!(request.is_regex);
    }

    #[test]
    fn test_search_empty_results() {
        let index_repo = TestIndexRepo::new();
        let event_pub = TestEventPublisher::new();
        
        let use_case = SearchFilesUseCase::new(index_repo, event_pub);
        let request = SearchFilesRequest::literal("nonexistent");
        
        let response = use_case.execute(request).unwrap();
        
        assert!(response.results.is_empty());
        assert_eq!(response.total_matches, 0);
        assert!(!response.truncated);
    }

    #[test]
    fn test_search_invalid_regex() {
        let index_repo = TestIndexRepo::new();
        let event_pub = TestEventPublisher::new();
        
        let use_case = SearchFilesUseCase::new(index_repo, event_pub);
        let request = SearchFilesRequest::regex("[invalid");
        
        let result = use_case.execute(request);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_search_no_events_for_empty_results() {
        let index_repo = TestIndexRepo::new();
        let event_pub = TestEventPublisher::new();
        
        let use_case = SearchFilesUseCase::new(index_repo, event_pub);
        let request = SearchFilesRequest::literal("test");
        
        use_case.execute(request).unwrap();
        
        // No events for empty results
        assert_eq!(use_case.event_publisher.event_count(), 0);
    }
}
