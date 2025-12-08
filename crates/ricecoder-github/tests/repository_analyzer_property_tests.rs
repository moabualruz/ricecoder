//! Property-based tests for RepositoryAnalyzer
//!
//! **Feature: ricecoder-github, Property 16: Repository Metadata Fetching**
//! **Feature: ricecoder-github, Property 17: Dependency Identification**
//! **Feature: ricecoder-github, Property 18: Code Pattern Extraction**
//! **Feature: ricecoder-github, Property 19: Codebase Summary Generation**
//! **Feature: ricecoder-github, Property 20: Analysis Result Caching**

use proptest::prelude::*;
use ricecoder_github::RepositoryAnalyzer;

// Strategy for generating valid repository owners
fn valid_owner_strategy() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9][a-zA-Z0-9\-]{0,38}"
        .prop_map(|s| s.to_string())
}

// Strategy for generating valid repository names
fn valid_repo_strategy() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9][a-zA-Z0-9\-_.]{0,38}"
        .prop_map(|s| s.to_string())
}

// Property 16: Repository Metadata Fetching
// *For any* repository, the system SHALL fetch and return metadata including name, owner, description, and language.
// **Validates: Requirements 4.1**
proptest! {
    #[test]
    fn prop_repository_metadata_fetching_returns_valid_metadata(
        owner in valid_owner_strategy(),
        repo in valid_repo_strategy()
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let analyzer = RepositoryAnalyzer::new();
            let result = analyzer.fetch_repository_metadata(&owner, &repo).await;

            // Should succeed for valid inputs
            prop_assert!(result.is_ok());

            let metadata = result.unwrap();

            // Metadata should contain the repository name
            prop_assert_eq!(&metadata.name, &repo);

            // Metadata should contain the owner
            prop_assert_eq!(&metadata.owner, &owner);

            // Metadata should have a description
            prop_assert!(!metadata.description.is_empty());

            // Metadata should have a URL
            prop_assert!(!metadata.url.is_empty());
            prop_assert!(metadata.url.contains(owner.as_str()));
            prop_assert!(metadata.url.contains(repo.as_str()));

            // Metadata should have a structure
            prop_assert!(
                !metadata.structure.directories.is_empty()
                    || !metadata.structure.files.is_empty()
            );
            
            Ok(())
        })?
    }
}

// Property 20: Analysis Result Caching
// *For any* repository analysis, subsequent calls with the same repository SHALL return cached results.
// **Validates: Requirements 4.5**
proptest! {
    #[test]
    fn prop_analysis_result_caching_returns_same_result(
        owner in valid_owner_strategy(),
        repo in valid_repo_strategy()
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let mut analyzer = RepositoryAnalyzer::new();

            // First analysis
            let result1 = analyzer.analyze_repository(&owner, &repo).await;
            prop_assert!(result1.is_ok());

            let analysis1 = result1.unwrap();

            // Second analysis (should be cached)
            let result2 = analyzer.analyze_repository(&owner, &repo).await;
            prop_assert!(result2.is_ok());

            let analysis2 = result2.unwrap();

            // Both analyses should have the same repository info
            prop_assert_eq!(analysis1.repository.name, analysis2.repository.name);
            prop_assert_eq!(analysis1.repository.owner, analysis2.repository.owner);

            // Both analyses should have the same summary
            prop_assert_eq!(
                analysis1.summary.primary_language,
                analysis2.summary.primary_language
            );
            prop_assert_eq!(analysis1.summary.languages, analysis2.summary.languages);

            // Verify cache contains the result
            let cached = analyzer.get_cached_analysis(&owner, &repo);
            prop_assert!(cached.is_some());
            
            Ok(())
        })?
    }
}

// Additional property: Cache invalidation
// *For any* cached analysis, clearing the cache SHALL remove the cached result
proptest! {
    #[test]
    fn prop_cache_invalidation_removes_cached_result(
        owner in valid_owner_strategy(),
        repo in valid_repo_strategy()
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let mut analyzer = RepositoryAnalyzer::new();

            // Perform analysis
            let _ = analyzer.analyze_repository(&owner, &repo).await;

            // Verify cache contains result
            let cached_before = analyzer.get_cached_analysis(&owner, &repo);
            prop_assert!(cached_before.is_some());

            // Clear cache
            analyzer.clear_cache_entry(&owner, &repo);

            // Verify cache no longer contains result
            let cached_after = analyzer.get_cached_analysis(&owner, &repo);
            prop_assert!(cached_after.is_none());
            
            Ok(())
        })?
    }
}

// Additional property: Invalid input handling
// *For any* invalid input (empty owner or repo), the system SHALL return an error
proptest! {
    #[test]
    fn prop_invalid_input_returns_error(
        repo in valid_repo_strategy()
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let analyzer = RepositoryAnalyzer::new();

            // Empty owner should fail
            let result = analyzer.fetch_repository_metadata("", &repo).await;
            prop_assert!(result.is_err());

            // Empty repo should fail
            let result = analyzer.fetch_repository_metadata("owner", "").await;
            prop_assert!(result.is_err());
            
            Ok(())
        })?
    }
}
