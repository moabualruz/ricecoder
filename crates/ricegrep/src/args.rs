//! Command-line argument parsing for RiceGrep
//!
//! This module provides ripgrep-compatible command line argument parsing
//! with additional AI-specific options.

use clap::{Arg, ArgMatches, Command, ArgAction, value_parser};
use std::path::PathBuf;
use crate::config::{RiceGrepConfig, OutputFormat, ColorChoice};

/// Parsed command-line arguments for RiceGrep
#[derive(Debug, Clone)]
pub struct Args {
    /// The search pattern (regex or literal)
    pub pattern: String,
    /// Paths to search in
    pub paths: Vec<PathBuf>,
    /// Case insensitive search
    pub case_insensitive: bool,
    /// Case sensitive search (overrides case_insensitive)
    pub case_sensitive: bool,
    /// Match whole words only
    pub word_regexp: bool,
    /// Literal string search (no regex)
    pub fixed_strings: bool,
    /// Line number display
    pub line_number: bool,
    /// Explicitly request line numbers
    pub line_number_flag: bool,
    /// Suppress line numbers
    pub no_line_number: bool,
    /// Only show filenames
    pub files: bool,
    /// Only show filenames with matches
    pub files_with_matches: bool,
    /// Only show filenames without matches
    pub files_without_match: bool,
    /// Count matches per file
    pub count: bool,
    /// Invert match (show non-matching lines)
    pub invert_match: bool,
    /// Maximum number of matches to find
    pub max_count: Option<usize>,
    /// Context lines before match
    pub before_context: Option<usize>,
    /// Context lines after match
    pub after_context: Option<usize>,
    /// Context lines before and after match
    pub context: Option<usize>,
    /// Follow symbolic links
    pub follow: bool,
    /// Search hidden files and directories
    pub hidden: bool,
    /// Respect .gitignore files
    pub no_ignore: bool,
    /// No heading (filename) for matches
    pub no_heading: bool,
    /// No filename for matches
    pub no_filename: bool,
    /// Force filename display
    pub with_filename: bool,
    /// Enable AI-enhanced search
    pub ai_enhanced: bool,
    /// Force natural language processing
    pub natural_language: bool,
    /// Launch interactive TUI mode
    pub tui: bool,
    /// Enable watch mode
    pub watch: bool,
    /// Watch mode timeout in seconds
    pub watch_timeout: Option<u64>,
    /// Clear screen between watch mode updates
    pub watch_clear: bool,
    /// Fuzzy search tolerance
    pub fuzzy: Option<usize>,
    /// Output format (text, json)
    pub format: OutputFormat,
    /// Color output
    pub color: ColorChoice,
    /// Threads to use
    pub threads: Option<usize>,
    /// Build search index
    pub index_build: bool,
    /// Update existing index
    pub index_update: bool,
    /// Watch mode for continuous indexing
    pub index_watch: bool,
    /// Show indexing status
    pub index_status: bool,
    /// Replace pattern (for replace operations)
    pub replace: Option<String>,
    /// Preview replace operations without executing
    pub preview: bool,
    /// Force replace operations (skip safety checks)
    pub force: bool,
}



impl Args {
    /// Parse command line arguments with configuration support
    pub fn parse() -> Result<Self, RiceGrepError> {
        let matches = Self::command().get_matches();

        // Load configuration for defaults
        let config = RiceGrepConfig::load().unwrap_or_default();

        let pattern = matches
            .get_one::<String>("pattern")
            .cloned()
            .unwrap_or_default();

        let paths = matches
            .get_many::<String>("path")
            .unwrap_or_default()
            .map(PathBuf::from)
            .collect();

        // Use configuration defaults with CLI overrides
        let case_insensitive = matches.get_flag("ignore-case");
        let case_sensitive = matches.get_flag("case-sensitive");
        let word_regexp = matches.get_flag("word-regexp");
        let fixed_strings = matches.get_flag("fixed-strings");
        let line_number_flag = matches.get_flag("line-number");
        let no_line_number = matches.get_flag("no-line-number");
        let line_number = line_number_flag; // Will be adjusted in main.rs for defaults
        let files = matches.get_flag("files");
        let files_with_matches = matches.get_flag("files-with-matches");
        let files_without_match = matches.get_flag("files-without-match");
        let count = matches.get_flag("count");
        let invert_match = matches.get_flag("invert-match");

        let max_count = matches
            .get_one::<String>("max-count")
            .and_then(|s| s.parse().ok())
            .or(config.max_search_results);

        let before_context = matches
            .get_one::<String>("before-context")
            .and_then(|s| s.parse().ok());

        let after_context = matches
            .get_one::<String>("after-context")
            .and_then(|s| s.parse().ok());

        let context = matches
            .get_one::<String>("context")
            .and_then(|s| s.parse().ok());

        let follow = matches.get_flag("follow");
        let hidden = matches.get_flag("hidden");
        let no_ignore = matches.get_flag("no-ignore");
        let no_heading = matches.get_flag("no-heading");
        let no_filename = matches.get_flag("no-filename");
        let with_filename = matches.get_flag("with-filename");

        // AI and watch mode with configuration defaults
        let ai_enhanced = matches.get_flag("ai-enhanced") ||
                         (config.ai_enabled && !matches.get_flag("no-ai"));
        let natural_language = matches.get_flag("natural-language");
        let tui = matches.get_flag("tui");

        let watch = matches.get_flag("watch") ||
                   (config.watch_enabled && !matches.get_flag("no-watch"));
        let watch_timeout = matches.get_one::<String>("watch-timeout")
            .and_then(|s| s.parse().ok())
            .or(config.watch_timeout_seconds);
        let watch_clear = matches.get_flag("watch-clear") ||
                         config.watch_clear_screen;

        let fuzzy = matches
            .get_one::<String>("fuzzy")
            .and_then(|s| s.parse::<usize>().ok());

        let format = if matches.get_flag("json") {
            OutputFormat::Json
        } else {
            config.output_format
        };

        let color = match matches.get_one::<String>("color").map(|s| s.as_str()) {
            Some("never") => ColorChoice::Never,
            Some("always") => ColorChoice::Always,
            _ => ColorChoice::Auto,
        };

        let threads = matches
            .get_one::<String>("threads")
            .and_then(|s| s.parse().ok());

        let index_build = matches.get_flag("index-build");
        let index_update = matches.get_flag("index-update");
        let index_watch = matches.get_flag("index-watch");
        let index_status = matches.get_flag("index-status");

        let replace = matches.get_one::<String>("replace").cloned();
        let preview = matches.get_flag("preview");
        let force = matches.get_flag("force");

        Ok(Args {
            pattern,
            paths,
            case_insensitive,
            case_sensitive,
            word_regexp,
            fixed_strings,
            line_number,
            line_number_flag,
            no_line_number,
            files,
            files_with_matches,
            files_without_match,
            count,
            invert_match,
            max_count,
            before_context,
            after_context,
            context,
            follow,
            hidden,
            no_ignore,
            no_heading,
            no_filename,
            with_filename,
            ai_enhanced,
            natural_language,
            tui,
            watch,
            watch_timeout,
            watch_clear,
            fuzzy,
            format,
             color,
             threads,
             index_build,
             index_update,
             index_watch,
             index_status,
             replace,
             preview,
             force,
        })
    }

    /// Build the clap command with all arguments
    fn command() -> Command {
        Command::new("ricegrep")
            .version(env!("CARGO_PKG_VERSION"))
            .author("RiceCoder")
            .about("AI-enhanced code search tool with ripgrep compatibility\n\nEXAMPLES:\n    ricegrep 'fn main' src/           # Search for function definitions\n    ricegrep --ignore-case 'TODO' .    # Case-insensitive search\n    ricegrep --replace 'new_name' 'old_name' file.rs  # Replace text\n    ricegrep --index-build .           # Build search index\n    ricegrep --ai-enhanced 'find all functions'  # AI-powered search")
            .arg(
                Arg::new("pattern")
                    .help("Search pattern (regex or literal string)")
                    .required_unless_present_any(["tui", "index-build", "index-update", "index-watch", "index-status"])
                    .index(1),
            )
            .arg(
                Arg::new("path")
                    .help("Files or directories to search")
                    .index(2)
                    .num_args(1..),
            )
            .arg(
                Arg::new("ignore-case")
                    .short('i')
                    .long("ignore-case")
                    .help("Case insensitive search")
                    .action(clap::ArgAction::SetTrue),
            )
            .arg(
                Arg::new("case-sensitive")
                    .short('s')
                    .long("case-sensitive")
                    .help("Case sensitive search")
                    .action(clap::ArgAction::SetTrue),
            )
            .arg(
                Arg::new("word-regexp")
                    .short('w')
                    .long("word-regexp")
                    .help("Match whole words only")
                    .action(clap::ArgAction::SetTrue),
            )
            .arg(
                Arg::new("fixed-strings")
                    .short('F')
                    .long("fixed-strings")
                    .help("Treat pattern as literal string, not regex")
                    .action(clap::ArgAction::SetTrue),
            )
            .arg(
                Arg::new("line-number")
                    .short('n')
                    .long("line-number")
                    .help("Show line numbers")
                    .action(clap::ArgAction::SetTrue),
            )
            .arg(
                Arg::new("no-line-number")
                    .long("no-line-number")
                    .help("Suppress line numbers")
                    .action(clap::ArgAction::SetTrue),
            )
            .arg(
                Arg::new("files")
                    .short('l')
                    .long("files")
                    .help("Only show filenames")
                    .action(clap::ArgAction::SetTrue),
            )
            .arg(
                Arg::new("files-with-matches")
                    .long("files-with-matches")
                    .help("Only show filenames with matches")
                    .action(clap::ArgAction::SetTrue),
            )
            .arg(
                Arg::new("files-without-match")
                    .long("files-without-match")
                    .help("Only show filenames without matches")
                    .action(clap::ArgAction::SetTrue),
            )
            .arg(
                Arg::new("count")
                    .short('c')
                    .long("count")
                    .help("Count matches per file")
                    .action(clap::ArgAction::SetTrue),
            )
            .arg(
                Arg::new("invert-match")
                    .short('v')
                    .long("invert-match")
                    .help("Invert match (show non-matching lines)")
                    .action(clap::ArgAction::SetTrue),
            )
            .arg(
                Arg::new("max-count")
                    .short('m')
                    .long("max-count")
                    .help("Maximum number of matches to find")
                    .value_name("NUM"),
            )
            .arg(
                Arg::new("before-context")
                    .short('B')
                    .long("before-context")
                    .help("Lines of context before match")
                    .value_name("NUM"),
            )
            .arg(
                Arg::new("after-context")
                    .short('A')
                    .long("after-context")
                    .help("Lines of context after match")
                    .value_name("NUM"),
            )
            .arg(
                Arg::new("context")
                    .short('C')
                    .long("context")
                    .help("Lines of context before and after match")
                    .value_name("NUM"),
            )
            .arg(
                Arg::new("follow")
                    .short('L')
                    .long("follow")
                    .help("Follow symbolic links")
                    .action(clap::ArgAction::SetTrue),
            )
            .arg(
                Arg::new("hidden")
                    .long("hidden")
                    .help("Search hidden files and directories")
                    .action(clap::ArgAction::SetTrue),
            )
            .arg(
                Arg::new("no-ignore")
                    .long("no-ignore")
                    .help("Don't respect .gitignore files")
                    .action(clap::ArgAction::SetTrue),
            )
            .arg(
                Arg::new("no-heading")
                    .long("no-heading")
                    .help("Don't group matches by file")
                    .action(clap::ArgAction::SetTrue),
            )
             .arg(
                 Arg::new("no-filename")
                     .long("no-filename")
                     .help("Never show filenames")
                     .action(clap::ArgAction::SetTrue),
             )
             .arg(
                 Arg::new("with-filename")
                     .short('H')
                     .long("with-filename")
                     .help("Always show filenames")
                     .action(clap::ArgAction::SetTrue),
             )
             .arg(
                 Arg::new("ai-enhanced")
                     .long("ai-enhanced")
                     .help("Enable AI-enhanced search with intelligent reranking and context")
                     .action(clap::ArgAction::SetTrue),
             )
             .arg(
                 Arg::new("natural-language")
                     .long("natural-language")
                     .help("Process query as natural language (e.g., 'find all functions')")
                     .action(clap::ArgAction::SetTrue),
             )
            .arg(
                Arg::new("tui")
                    .long("tui")
                    .help("Launch interactive TUI mode")
                    .action(clap::ArgAction::SetTrue),
            )
            .arg(
                Arg::new("watch")
                    .long("watch")
                    .help("Watch for file changes and automatically update search results")
                    .action(clap::ArgAction::SetTrue),
            )
            .arg(
                Arg::new("no-watch")
                    .long("no-watch")
                    .help("Disable watch mode")
                    .action(clap::ArgAction::SetTrue),
            )
            .arg(
                Arg::new("watch-timeout")
                    .long("watch-timeout")
                    .help("Timeout for watch mode in seconds")
                    .value_name("SECONDS")
                    .value_parser(value_parser!(u64)),
            )
            .arg(
                Arg::new("watch-clear")
                    .long("watch-clear")
                    .help("Clear screen between watch mode updates")
                    .action(clap::ArgAction::SetTrue),
            )
            .arg(
                Arg::new("fuzzy")
                    .long("fuzzy")
                    .help("Enable fuzzy search with given edit distance")
                    .value_name("DISTANCE"),
            )
            .arg(
                Arg::new("json")
                    .long("json")
                    .help("Output results in JSON format")
                    .action(clap::ArgAction::SetTrue),
            )
            .arg(
                Arg::new("color")
                    .long("color")
                    .help("Colorize output")
                    .value_name("WHEN")
                    .value_parser(["never", "auto", "always"])
                    .default_value("auto"),
            )
             .arg(
                 Arg::new("threads")
                     .short('j')
                     .long("threads")
                     .help("Number of threads to use")
                     .value_name("NUM"),
             )
             .arg(
                 Arg::new("index-build")
                     .long("index-build")
                     .help("Build search index for faster queries on large codebases")
                     .action(clap::ArgAction::SetTrue),
             )
             .arg(
                 Arg::new("index-update")
                     .long("index-update")
                     .help("Update existing search index with new/changed files")
                     .action(clap::ArgAction::SetTrue),
             )
             .arg(
                 Arg::new("index-watch")
                     .long("index-watch")
                     .help("Watch for file changes and automatically update index")
                     .action(clap::ArgAction::SetTrue),
             )
             .arg(
                 Arg::new("index-status")
                     .long("index-status")
                     .help("Show current indexing status and statistics")
                     .action(clap::ArgAction::SetTrue),
             )
             .arg(
                 Arg::new("replace")
                     .short('r')
                     .long("replace")
                     .help("Replace matches with specified pattern (use --preview first)")
                     .value_name("PATTERN"),
             )
             .arg(
                 Arg::new("preview")
                     .long("preview")
                     .help("Show replace operations without making changes")
                     .action(clap::ArgAction::SetTrue),
             )
             .arg(
                 Arg::new("force")
                     .long("force")
                     .help("Execute replace operations (use with caution)")
                     .action(clap::ArgAction::SetTrue),
             )
    }
}

use crate::error::RiceGrepError;