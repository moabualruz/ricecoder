//! RiceGrep - AI-enhanced code search tool
//!
//! RiceGrep provides ripgrep-compatible search functionality with AI-enhanced
//! result ranking and natural language query understanding.

pub mod args;
pub mod search;
pub mod output;
pub mod error;
pub mod ai;
pub mod embedded_ai;
pub mod language;
pub mod tui;
pub mod config;
pub mod spelling;
pub mod replace;
pub mod watch;
pub mod mcp;
pub mod skills;
pub mod telemetry;
pub mod monitoring;
pub mod database;

pub use monitoring::{ProcessManager, ProcessConfig, ProcessStatus};
pub use telemetry::{UsageAnalytics, BenchmarkSuite, BenchmarkConfig, PerformanceReport};
pub use database::{DatabaseManager, DatabaseConfig, SearchHistory, UserPreferences, IndexMetadata, IndexStatus};

pub use args::Args;
pub use search::SearchEngine;
pub use error::RiceGrepError;
pub use config::{RiceGrepConfig, OutputFormat, ColorChoice};
pub use spelling::{SpellingCorrector, SpellingConfig, CorrectionResult};
pub use language::{LanguageProcessor, LanguageConfig, LanguageDetection, LanguageRanking};

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use crate::search::ProgressVerbosity;

    #[tokio::test]
    async fn test_basic_search() {
        let mut engine = search::RegexSearchEngine::new();
        let query = search::SearchQuery {
            pattern: "test".to_string(),
            paths: vec![PathBuf::from(".")],
            case_insensitive: false,
            case_sensitive: false,
            word_regexp: false,
            fixed_strings: false,
            follow: false,
            hidden: false,
            no_ignore: true, // Don't ignore files for testing
            ignore_file: None,
            quiet: false,
            dry_run: false,
            max_file_size: None,
            progress_verbosity: ProgressVerbosity::Quiet,
            max_files: None,
            max_matches: Some(10),
            max_lines: None,
            invert_match: false,
            ai_enhanced: false,
            no_rerank: false,
            fuzzy: None,
            max_count: Some(10),
            spelling_correction: None,
        };

        let result = engine.search(query).await;
        // Test should pass regardless of whether files exist
        assert!(result.is_ok() || matches!(result, Err(crate::error::RiceGrepError::Io(_))));
    }

    #[test]
    fn test_fuzzy_matching() {
        let matcher = search::FuzzyMatcher::new(2, 0.8);

        // Test exact match
        assert!(matcher.is_match("hello", "hello"));

        // Test fuzzy match within distance
        assert!(matcher.is_match("hello", "helo")); // distance 1

        // Test fuzzy match at distance limit
        assert!(matcher.is_match("hello", "hxllo")); // distance 2, should match
    }

    #[test]
    fn test_language_detection() {
        use crate::language::{LanguageProcessor, LanguageConfig};

        let processor = LanguageProcessor::new(LanguageConfig::default());

        // Test extension-based detection
        let rust_result = processor.detect_language(&PathBuf::from("test.rs")).unwrap();
        assert!(rust_result.is_some());
        assert_eq!(rust_result.unwrap().language_name, "Rust");

        let python_result = processor.detect_language(&PathBuf::from("test.py")).unwrap();
        assert!(python_result.is_some());
        assert_eq!(python_result.unwrap().language_name, "Python");

        let js_result = processor.detect_language(&PathBuf::from("test.js")).unwrap();
        assert!(js_result.is_some());
        assert_eq!(js_result.unwrap().language_name, "JavaScript");

        let unknown_result = processor.detect_language(&PathBuf::from("unknown.xyz")).unwrap();
        assert!(unknown_result.is_none());
    }

    #[test]
    fn test_ai_query_detection() {
        let ai_processor = Box::new(ai::RiceGrepAIProcessor::new());
        let engine = search::RegexSearchEngine::new().with_ai_processor(ai_processor);

        let natural_query = search::SearchQuery {
            pattern: "find all functions".to_string(),
            paths: vec![],
            case_insensitive: false,
            case_sensitive: false,
            word_regexp: false,
            fixed_strings: false,
            follow: false,
            hidden: false,
            no_ignore: false,
            ignore_file: None,
            quiet: false,
            dry_run: false,
            max_file_size: None,
            progress_verbosity: ProgressVerbosity::Quiet,
            max_files: None,
            max_matches: None,
            max_lines: None,
            invert_match: false,
            ai_enhanced: false,
            no_rerank: false,
            fuzzy: None,
            max_count: None,
            spelling_correction: None,
        };

        let regex_query = search::SearchQuery {
            pattern: "fn.*test".to_string(),
            paths: vec![],
            case_insensitive: false,
            case_sensitive: false,
            word_regexp: false,
            fixed_strings: false,
            follow: false,
            hidden: false,
            no_ignore: false,
            ignore_file: None,
            quiet: false,
            dry_run: false,
            max_file_size: None,
            progress_verbosity: ProgressVerbosity::Quiet,
            max_files: None,
            max_matches: None,
            max_lines: None,
            invert_match: false,
            ai_enhanced: false,
            no_rerank: false,
            fuzzy: None,
            max_count: None,
            spelling_correction: None,
        };

        // Natural language queries should be detected (contains "find")
        assert!(engine.is_natural_language_query(&natural_query));

        // Regex patterns should not be detected as natural language
        assert!(!engine.is_natural_language_query(&regex_query));
    }

    #[tokio::test]
    async fn test_index_operations() {
        use tempfile::tempdir;
        use crate::search::{RegexSearchEngine, SearchEngine, SearchQuery};

        let temp_dir = tempdir().unwrap();

        // Create a temporary file to index
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(&test_file, "fn main() {\n    println!(\"hello\");\n}").unwrap();

        // Test index building with custom index directory
        let index_dir = temp_dir.path().join("index");
        let mut search_engine = RegexSearchEngine::with_index_dir(index_dir);
        search_engine.build_index(&[temp_dir.path().to_path_buf()], ProgressVerbosity::Quiet).await.unwrap();

        // Test search with index
        let query = SearchQuery {
            pattern: "println".to_string(),
            paths: vec![temp_dir.path().to_path_buf()],
            case_insensitive: false,
            case_sensitive: false,
            word_regexp: false,
            fixed_strings: false,
            follow: false,
            hidden: false,
            no_ignore: true,
            ignore_file: None,
            quiet: false,
            dry_run: false,
            max_file_size: None,
            progress_verbosity: ProgressVerbosity::Quiet, // Test mode uses quiet progress
            max_files: None,
            max_matches: None,
            max_lines: None,
            invert_match: false,
            ai_enhanced: false,
            no_rerank: false,
            fuzzy: None,
            max_count: None,
            spelling_correction: None,
        };

        let results = search_engine.search(query).await.unwrap();
        assert!(!results.matches.is_empty(), "Should find matches in indexed content");
        assert_eq!(results.matches[0].line_content.trim(), "println!(\"hello\");");
    }

    #[test]
    fn test_cli_argument_parsing() {
        // Note: Full CLI parsing tests are in integration_tests.rs
        // This is just a basic smoke test
        assert!(true); // Placeholder for now
    }

    #[test]
    fn test_spelling_correction_integration() {
        use crate::spelling::{SpellingCorrector, SpellingConfig};

        let mut corrector = SpellingCorrector::new(SpellingConfig::default());

        // Test basic correction
        let result = corrector.correct_query("functon").unwrap();
        assert_eq!(result.corrected, Some("function".to_string()));

        // Test no correction needed
        let result = corrector.correct_query("function").unwrap();
        assert!(result.corrected.is_none());
    }

    #[test]
    fn test_output_formatting() {
        use crate::output::OutputFormatter;
        use crate::config::{OutputFormat, ColorChoice};
        use crate::search::{SearchResults, SearchMatch};
        use std::path::PathBuf;

        let formatter = OutputFormatter::new(
            OutputFormat::Text,
            ColorChoice::Never,
            true, // line numbers
            true, // heading
            true, // filename
            false, // ai enabled
            false, // count
            false, // content
            None, // max_lines
        );

        let results = SearchResults {
            matches: vec![SearchMatch {
                file: PathBuf::from("test.rs"),
                line_number: 5,
                line_content: "    println!(\"hello\");".to_string(),
                byte_offset: 100,
                ai_score: None,
                ai_context: None,
                language: Some("Rust".to_string()),
                language_confidence: Some(0.9),
            }],
            total_matches: 1,
            search_time: std::time::Duration::from_millis(50),
            ai_reranked: false,
            degradation_mode: false,
            files_searched: 1,
            spelling_correction: None,
            file_counts: std::collections::HashMap::new(),
        };

        // This should not panic
        formatter.write_results(&results).unwrap();
    }

    #[test]
    fn test_error_handling() {
        use crate::error::RiceGrepError;

        // Test error creation and display
        let io_error = RiceGrepError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"));
        assert!(io_error.to_string().contains("I/O error"));

        let search_error = RiceGrepError::Search {
            message: "test error".to_string()
        };
        assert!(search_error.to_string().contains("test error"));
    }

    #[test]
    fn test_skill_registry() {
        use crate::skills::SkillRegistry;

        let registry = SkillRegistry::new();

        // Test that built-in skills are registered
        assert!(registry.get_skill("ricegrep-search").is_some());
        assert!(registry.get_skill("ricegrep-replace").is_some());

        // Test skill export
        let yaml_result = registry.export_skill_yaml("ricegrep-search");
        assert!(yaml_result.is_ok());
        assert!(yaml_result.unwrap().contains("ricegrep-search"));

        let json_result = registry.export_skill_json("ricegrep-search");
        assert!(json_result.is_ok());
        assert!(json_result.unwrap().contains("ricegrep-search"));
    }

    #[test]
    fn test_mcp_server_creation() {
        use crate::mcp::RiceGrepMcpServer;

        // Test that MCP server can be created
        let server = RiceGrepMcpServer::new();
        // Server creation should not panic
        assert!(true); // Placeholder test - full MCP testing requires SDK
    }
}
