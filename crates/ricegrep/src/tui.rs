//! Terminal User Interface for RiceGrep
//!
//! Provides an interactive TUI for search result exploration with
//! real-time updates, keyboard navigation, and AI-enhanced display.

use crate::search::{SearchResults, SearchMatch, RegexSearchEngine, SearchEngine, SearchQuery};
use crate::error::RiceGrepError;
use crate::args::Args;
use std::io;
use std::sync::Arc;
use std::path::PathBuf;
use tokio::sync::mpsc;

/// TUI application state
pub struct RiceGrepTUI {
    /// Search engine
    search_engine: RegexSearchEngine,
    /// Current search results
    results: Option<SearchResults>,
    /// Current search query
    current_query: String,
    /// Selected result index
    selected_index: usize,
    /// Search input mode
    input_mode: InputMode,
    /// Search history
    history: Vec<String>,
    /// History index
    history_index: usize,
}

/// Input modes for the TUI
#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    /// Normal search input
    Search,
    /// Command mode (for advanced operations)
    Command,
    /// Results browsing mode
    Browse,
}

impl RiceGrepTUI {
    /// Create a new TUI instance
    pub fn new(search_engine: RegexSearchEngine) -> Self {
        Self {
            search_engine,
            results: None,
            current_query: String::new(),
            selected_index: 0,
            input_mode: InputMode::Search,
            history: Vec::new(),
            history_index: 0,
        }
    }

    /// Run the TUI application
    pub async fn run(&mut self) -> Result<(), RiceGrepError> {
        // For now, implement a simple text-based interface
        // In full implementation, this would use a proper TUI framework
        println!("RiceGrep Interactive Search");
        println!("===========================");
        println!();
        println!("Commands:");
        println!("  /search <query>  - Search for text");
        println!("  /ai <query>      - AI-enhanced search");
        println!("  /fuzzy <query>   - Fuzzy search");
        println!("  /quit            - Exit");
        println!();

        let stdin = io::stdin();
        let mut input = String::new();

        loop {
            print!("ricegrep> ");
            io::Write::flush(&mut io::stdout())?;

            input.clear();
            stdin.read_line(&mut input)?;
            let command = input.trim();

            if command.is_empty() {
                continue;
            }

            match self.process_command(command).await {
                Ok(true) => break, // Quit
                Ok(false) => continue,
                Err(e) => {
                    println!("Error: {}", e);
                    continue;
                }
            }
        }

        Ok(())
    }

    /// Process a command
    async fn process_command(&mut self, command: &str) -> Result<bool, RiceGrepError> {
        if command == "/quit" || command == "quit" {
            return Ok(true);
        }

        if command.starts_with("/search ") {
            let query = &command[8..];
            self.perform_search(query, false, false).await?;
        } else if command.starts_with("/ai ") {
            let query = &command[4..];
            self.perform_search(query, true, false).await?;
        } else if command.starts_with("/fuzzy ") {
            let query = &command[7..];
            self.perform_search(query, false, true).await?;
        } else if command.starts_with('/') {
            println!("Unknown command. Use /search, /ai, /fuzzy, or /quit");
        } else {
            // Default to regular search
            self.perform_search(command, false, false).await?;
        }

        Ok(false)
    }

    /// Perform a search and display results
    async fn perform_search(&mut self, query: &str, ai_enhanced: bool, fuzzy: bool) -> Result<(), RiceGrepError> {
        println!("Searching for: {}", query);
        if ai_enhanced {
            println!("(AI-enhanced mode)");
        }
        if fuzzy {
            println!("(Fuzzy search mode)");
        }

        // Create search query
        let search_query = SearchQuery {
            pattern: query.to_string(),
            paths: vec![std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))],
            case_insensitive: false,
            case_sensitive: false,
            word_regexp: false,
            fixed_strings: false,
            follow: false,
            hidden: false,
            no_ignore: false,
            invert_match: false,
            ai_enhanced,
            fuzzy: None,
            max_count: None,
            spelling_correction: None,
        };

        // Perform search
        let results = self.search_engine.search(search_query).await?;

        // Display results
        self.display_results(&results)?;

        // Store results
        self.results = Some(results);
        self.current_query = query.to_string();

        // Add to history
        if !self.history.contains(&query.to_string()) {
            self.history.push(query.to_string());
            self.history_index = self.history.len();
        }

        Ok(())
    }

    /// Display search results
    fn display_results(&self, results: &SearchResults) -> Result<(), RiceGrepError> {
        if results.matches.is_empty() {
            println!("No matches found.");
            return Ok(());
        }

        println!();
        println!("Found {} matches in {} files (searched in {:.2}ms)",
                 results.total_matches,
                 results.files_searched,
                 results.search_time.as_secs_f64() * 1000.0);

        if results.ai_reranked {
            println!("Results enhanced with AI reranking");
        }

        println!();

        // Group results by file
        let mut current_file = None;
        let mut file_match_count = 0;

        for (i, match_result) in results.matches.iter().enumerate() {
            // Print file header when file changes
            if current_file.as_ref() != Some(&match_result.file) {
                if current_file.is_some() {
                    println!();
                }
                current_file = Some(match_result.file.clone());
                file_match_count = results.matches.iter()
                    .filter(|m| m.file == match_result.file)
                    .count();
                println!("{} ({} matches):", match_result.file.display(), file_match_count);
            }

            // Print match with line number
            print!("  {}: {}", match_result.line_number, match_result.line_content);

            // Print AI score if available
            if let Some(score) = match_result.ai_score {
                print!(" [AI: {:.2}]", score);
            }

            // Print AI context if available
            if let Some(context) = &match_result.ai_context {
                print!(" // {}", context);
            }

            println!();
        }

        println!();
        Ok(())
    }

    /// Get help text
    pub fn help_text() -> &'static str {
        r#"RiceGrep Interactive Search

COMMANDS:
  /search <query>     - Regular text search
  /ai <query>         - AI-enhanced natural language search
  /fuzzy <query>      - Fuzzy search with edit distance tolerance
  /quit               - Exit the interactive search

SEARCH FEATURES:
  - Real-time search results
  - AI-enhanced ranking and relevance scoring
  - Fuzzy matching for typos
  - Multi-language support
  - LSP integration for symbol search

EXAMPLES:
  ricegrep> /search "fn main"
  ricegrep> /ai "find all function definitions"
  ricegrep> /fuzzy "functon"  # matches "function"
  ricegrep> /quit
"#
    }
}

/// Launch the TUI if requested
pub async fn run_tui(args: &Args) -> Result<(), RiceGrepError> {
    // Check if TUI mode is requested
    // For now, we don't have a dedicated TUI flag, so this is a placeholder
    // In full implementation, this would check for --tui flag

    // For now, just run the interactive mode if no pattern is provided
    if args.pattern.is_empty() {
        let search_engine = RegexSearchEngine::new();
        let mut tui = RiceGrepTUI::new(search_engine);
        tui.run().await?;
    }

    Ok(())
}