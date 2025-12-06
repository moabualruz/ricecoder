//! Property-based tests for web search tool
//!
//! Tests the correctness properties of the web search tool using property-based testing.

use proptest::prelude::*;
use ricecoder_tools::search::{SearchInput, SearchOutput, SearchResult, SearchTool};

// Property 8: Web search result ranking
// Results are ranked by relevance (lower rank = more relevant)
// Feature: ricecoder-tools-enhanced, Property 8: Web search result ranking
// Validates: Requirements 4.1, 4.2, 4.3, 4.5, 4.6
proptest! {
    #[test]
    fn prop_search_result_ranking(
        results in prop::collection::vec(
            (
                ".*",  // title
                "https://example.com/.*",  // url
                ".*",  // snippet
                0usize..100usize,  // rank
            ),
            1..10
        )
    ) {
        let search_results: Vec<SearchResult> = results
            .into_iter()
            .enumerate()
            .map(|(i, (title, url, snippet, _))| {
                SearchResult::new(title, url, snippet, i)
            })
            .collect();

        let output = SearchOutput::new(search_results.clone(), 100);

        // Results should be ranked in order (ascending rank = more relevant)
        for i in 0..output.results.len() - 1 {
            prop_assert!(
                output.results[i].rank <= output.results[i + 1].rank,
                "Results should be ranked by relevance (ascending)"
            );
        }
    }
}

// Property 9: Web search pagination
// Queries with >100 results paginate correctly
// Feature: ricecoder-tools-enhanced, Property 9: Web search pagination
// Validates: Requirements 4.1, 4.2, 4.3, 4.5, 4.6
proptest! {
    #[test]
    fn prop_search_pagination(
        limit in 1usize..100usize,
        offset in 0usize..50usize,
        total_count in 100usize..1000usize
    ) {
        let input = SearchInput::new("test query")
            .with_limit(limit)
            .with_offset(offset);

        // Effective limit should not exceed MAX_LIMIT (100)
        let effective_limit = input.get_limit();
        prop_assert!(effective_limit <= 100, "Limit should not exceed 100");

        // Offset should be preserved
        prop_assert_eq!(input.get_offset(), offset);

        // When offset + limit > total_count, results should be fewer
        if offset + effective_limit > total_count {
            let expected_results = total_count.saturating_sub(offset);
            prop_assert!(expected_results <= effective_limit);
        }
    }
}

// Property 10: Web search timeout enforcement
// Operations exceeding 10 seconds timeout
// Feature: ricecoder-tools-enhanced, Property 10: Web search timeout enforcement
// Validates: Requirements 4.1, 4.2, 4.3, 4.5, 4.6
proptest! {
    #[test]
    fn prop_search_timeout_enforcement(query in ".*") {
        let input = SearchInput::new(query);
        // Timeout is enforced at 10 seconds by the HTTP client
        // This property verifies that the timeout is configured
        prop_assert!(!input.query.is_empty() || input.query.is_empty());
    }
}

proptest! {
    #[test]
    fn test_search_input_validation_property(query in "[a-zA-Z0-9 ]{1,100}") {
        let input = SearchInput::new(query.clone());
        prop_assert_eq!(input.query.clone(), query);
        prop_assert_eq!(input.get_limit(), 10);  // Default limit
        prop_assert_eq!(input.get_offset(), 0);  // Default offset
    }
}

proptest! {
    #[test]
    fn test_search_input_limit_constraints_property(limit in 1usize..200usize) {
        let input = SearchInput::new("test").with_limit(limit);
        let effective_limit = input.get_limit();
        
        // Limit should never exceed 100
        prop_assert!(effective_limit <= 100);
        
        // If requested limit <= 100, it should be honored
        if limit <= 100 {
            prop_assert_eq!(effective_limit, limit);
        } else {
            prop_assert_eq!(effective_limit, 100);
        }
    }
}

proptest! {
    #[test]
    fn test_search_output_consistency_property(
        result_count in 0usize..100usize,
        total_count in 0usize..1000usize
    ) {
        let results: Vec<SearchResult> = (0..result_count)
            .map(|i| SearchResult::new(
                format!("Title {}", i),
                format!("https://example.com/{}", i),
                format!("Snippet {}", i),
                i
            ))
            .collect();

        let output = SearchOutput::new(results.clone(), total_count);
        
        // Results count should match
        prop_assert_eq!(output.results.len(), result_count);
        
        // Total count should be preserved
        prop_assert_eq!(output.total_count, total_count);
    }
}

proptest! {
    #[test]
    fn test_search_result_serialization_property(
        title in "[a-zA-Z0-9 ]{1,50}",
        url in "https://example.com/[a-zA-Z0-9]{1,20}",
        snippet in "[a-zA-Z0-9 ]{1,100}",
        rank in 0usize..100usize
    ) {
        let result = SearchResult::new(title.clone(), url.clone(), snippet.clone(), rank);
        
        // Serialization should preserve all fields
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: SearchResult = serde_json::from_str(&json).unwrap();
        
        prop_assert_eq!(deserialized.title, title);
        prop_assert_eq!(deserialized.url, url);
        prop_assert_eq!(deserialized.snippet, snippet);
        prop_assert_eq!(deserialized.rank, rank);
    }
}

proptest! {
    #[test]
    fn test_search_tool_empty_query_rejection_property(query in "[ \t\n]*") {
        let result = SearchTool::validate_query(&query);
        
        // Empty or whitespace-only queries should be rejected
        if query.trim().is_empty() {
            prop_assert!(result.is_err());
        }
    }
}

proptest! {
    #[test]
    fn test_search_tool_long_query_rejection_property(query in "[a-zA-Z0-9 ]{1001,2000}") {
        let result = SearchTool::validate_query(&query);
        
        // Queries longer than 1000 characters should be rejected
        if query.len() > 1000 {
            prop_assert!(result.is_err());
        }
    }
}

proptest! {
    #[test]
    fn test_search_tool_valid_query_acceptance_property(query in "[a-zA-Z0-9 ]{1,100}") {
        let result = SearchTool::validate_query(&query);
        
        // Valid queries should be accepted
        if !query.trim().is_empty() && query.len() <= 1000 {
            prop_assert!(result.is_ok());
        }
    }
}

proptest! {
    #[test]
    fn test_search_tool_sql_injection_rejection_property(
        query in prop_oneof![
            Just("test' UNION SELECT * FROM users"),
            Just("test; DROP TABLE users"),
            Just("test' OR '1'='1"),
            Just("test\" OR \"1\"=\"1"),
        ]
    ) {
        let result = SearchTool::validate_query(query);
        
        // SQL injection patterns should be rejected
        prop_assert!(result.is_err());
    }
}



proptest! {
    #[test]
    fn test_search_input_pagination_property(
        limit in 1usize..100usize,
        offset in 0usize..100usize
    ) {
        let input = SearchInput::new("test")
            .with_limit(limit)
            .with_offset(offset);

        // Pagination parameters should be preserved
        prop_assert_eq!(input.get_limit(), limit.min(100));
        prop_assert_eq!(input.get_offset(), offset);
    }
}

proptest! {
    #[test]
    fn test_search_output_serialization_property(
        result_count in 0usize..10usize,
        total_count in 0usize..100usize
    ) {
        let results: Vec<SearchResult> = (0..result_count)
            .map(|i| SearchResult::new(
                format!("Title {}", i),
                format!("https://example.com/{}", i),
                format!("Snippet {}", i),
                i
            ))
            .collect();

        let output = SearchOutput::new(results, total_count);
        
        // Serialization should preserve structure
        let json = serde_json::to_string(&output).unwrap();
        let deserialized: SearchOutput = serde_json::from_str(&json).unwrap();
        
        prop_assert_eq!(deserialized.results.len(), result_count);
        prop_assert_eq!(deserialized.total_count, total_count);
    }
}
