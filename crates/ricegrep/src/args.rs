//! Command-line argument parsing for RiceGrep
//!
//! This module provides ripgrep-compatible command line argument parsing
//! with additional AI-specific options.

use clap::{Arg, ArgMatches, Command, ArgAction, value_parser};
use std::path::PathBuf;
use crate::config::{RiceGrepConfig, OutputFormat, ColorChoice};

/// RiceGrep subcommands
#[derive(Debug, Clone, PartialEq)]
pub enum RiceGrepCommand {
    /// Search command (default behavior)
    Search(SearchArgs),
    /// Replace/symbol rename command
    Replace(ReplaceArgs),
    /// Watch mode command
    Watch(WatchArgs),
    /// MCP server command
    Mcp(McpArgs),
    /// Index management command
    Index(IndexArgs),
    /// Install plugin command
    Install(InstallArgs),
    /// Uninstall plugin command
    Uninstall(UninstallArgs),
    /// Legacy option-based interface (for backward compatibility)
    Legacy(LegacyArgs),
}

/// Arguments for search subcommand
#[derive(Debug, Clone, PartialEq)]
pub struct SearchArgs {
    pub pattern: String,
    pub paths: Vec<PathBuf>,
    pub case_insensitive: bool,
    pub case_sensitive: bool,
    pub word_regexp: bool,
    pub fixed_strings: bool,
    pub line_number: bool,
    pub invert_match: bool,
    pub count: bool,
    pub max_count: Option<usize>,
    pub before_context: Option<usize>,
    pub after_context: Option<usize>,
    pub context: Option<usize>,
    pub content: bool,
    pub syntax_highlight: bool,
    pub answer: bool,
    pub no_rerank: bool,
    pub ai_enhanced: bool,
    pub natural_language: bool,
    pub replace: Option<String>,
    pub preview: bool,
    pub force: bool,
    pub ignore_file: Option<PathBuf>,
    pub quiet: bool,
    pub dry_run: bool,
    pub max_file_size: Option<u64>,
    pub max_files: Option<usize>,
    pub max_matches: Option<usize>,
    pub max_lines: Option<usize>,
}

/// Arguments for replace subcommand
#[derive(Debug, Clone, PartialEq)]
pub struct ReplaceArgs {
    pub old_symbol: String,
    pub new_symbol: String,
    pub file_path: PathBuf,
    pub language: Option<String>,
    pub preview: bool,
    pub force: bool,
    pub dry_run: bool,
}

/// Arguments for watch subcommand
#[derive(Debug, Clone, PartialEq)]
pub struct WatchArgs {
    pub paths: Vec<PathBuf>,
    pub timeout: Option<u64>,
    pub clear_screen: bool,
    pub max_file_size: Option<u64>,
}

/// Arguments for MCP subcommand
#[derive(Debug, Clone, PartialEq)]
pub struct McpArgs {
    pub port: Option<u16>,
    pub host: String,
    pub no_watch: bool,
}

/// Index management commands
#[derive(Debug, Clone, PartialEq)]
pub enum IndexCommand {
    Build,
    Update,
    Clear,
    Status,
}

/// Arguments for index subcommand
#[derive(Debug, Clone, PartialEq)]
pub struct IndexArgs {
    pub command: IndexCommand,
    pub paths: Vec<PathBuf>,
}


/// Arguments for install subcommand
#[derive(Debug, Clone, PartialEq)]
pub struct InstallArgs {
    pub plugin: String,
    pub version: Option<String>,
    pub force: bool,
}

/// Arguments for uninstall subcommand
#[derive(Debug, Clone, PartialEq)]
pub struct UninstallArgs {
    pub plugin: String,
    pub force: bool,
}

/// Legacy arguments for backward compatibility
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyArgs {
    pub pattern: String,
    pub paths: Vec<PathBuf>,
    pub case_insensitive: bool,
    pub case_sensitive: bool,
    pub word_regexp: bool,
    pub fixed_strings: bool,
    pub line_number: bool,
    pub line_number_flag: bool,
    pub no_line_number: bool,
    pub files: bool,
    pub files_with_matches: bool,
    pub files_without_match: bool,
    pub count: bool,
    pub invert_match: bool,
    pub max_count: Option<usize>,
    pub before_context: Option<usize>,
    pub after_context: Option<usize>,
    pub context: Option<usize>,
    pub follow: bool,
    pub hidden: bool,
    pub no_ignore: bool,
    pub color: ColorChoice,
    pub threads: Option<usize>,
    pub ai_enhanced: bool,
    pub natural_language: bool,
    pub fuzzy: Option<usize>,
    pub output_format: OutputFormat,
    pub index_build: bool,
    pub index_update: bool,
    pub index_watch: bool,
    pub index_status: bool,
    pub replace: Option<String>,
    pub preview: bool,
    pub force: bool,
    pub with_filename: bool,
    pub no_filename: bool,
}

/// Parsed command-line arguments for RiceGrep
#[derive(Debug, Clone)]
pub struct Args {
    /// The subcommand to execute
    pub command: RiceGrepCommand,
}



impl Args {
    /// Parse command line arguments with subcommand support
    pub fn parse() -> Result<Self, RiceGrepError> {
        let matches = Self::command().get_matches();

        // Determine which subcommand was used
        let command = if let Some(search_matches) = matches.subcommand_matches("search") {
            RiceGrepCommand::Search(Self::parse_search_args(search_matches)?)
        } else if let Some(replace_matches) = matches.subcommand_matches("replace") {
            RiceGrepCommand::Replace(Self::parse_replace_args(replace_matches)?)
        } else if let Some(watch_matches) = matches.subcommand_matches("watch") {
            RiceGrepCommand::Watch(Self::parse_watch_args(watch_matches)?)
        } else if let Some(mcp_matches) = matches.subcommand_matches("mcp") {
            RiceGrepCommand::Mcp(Self::parse_mcp_args(mcp_matches)?)
        } else if let Some(index_matches) = matches.subcommand_matches("index") {
            RiceGrepCommand::Index(Self::parse_index_args(index_matches)?)
        } else if let Some(install_matches) = matches.subcommand_matches("install") {
            RiceGrepCommand::Install(Self::parse_install_args(install_matches)?)
        } else if let Some(uninstall_matches) = matches.subcommand_matches("uninstall") {
            RiceGrepCommand::Uninstall(Self::parse_uninstall_args(uninstall_matches)?)
        } else {
            // No subcommand specified - check if we have legacy options
            if matches.get_one::<String>("pattern").is_some() {
                RiceGrepCommand::Legacy(Self::parse_legacy_args(&matches)?)
            } else {
                // Default to search subcommand
                RiceGrepCommand::Search(SearchArgs {
                    pattern: String::new(),
                    paths: vec![],
                    case_insensitive: false,
                    case_sensitive: false,
                    word_regexp: false,
                    fixed_strings: false,
                    line_number: true,
                    invert_match: false,
                    count: false,
                    max_count: None,
                    before_context: None,
                    after_context: None,
                    context: None,
                    content: false,
                    syntax_highlight: false,
                    answer: false,
                    no_rerank: false,
                    ai_enhanced: false,
                    natural_language: false,
                    replace: None,
                    preview: false,
                    force: false,
                    ignore_file: None,
                    quiet: false,
                    dry_run: false,
                    max_file_size: None,
                    max_files: None,
                    max_matches: None,
                    max_lines: None,
                })
            }
        };

        Ok(Self { command })
    }

    /// Parse replace subcommand arguments
    fn parse_replace_args(matches: &ArgMatches) -> Result<ReplaceArgs, crate::error::RiceGrepError> {
        Ok(ReplaceArgs {
            old_symbol: matches
                .get_one::<String>("old_symbol")
                .ok_or_else(|| crate::error::RiceGrepError::Search {
                    message: "old_symbol is required".to_string()
                })?
                .clone(),
            new_symbol: matches
                .get_one::<String>("new_symbol")
                .ok_or_else(|| crate::error::RiceGrepError::Search {
                    message: "new_symbol is required".to_string()
                })?
                .clone(),
            file_path: matches
                .get_one::<PathBuf>("file")
                .ok_or_else(|| crate::error::RiceGrepError::Search {
                    message: "file is required".to_string()
                })?
                .clone(),
            language: matches.get_one::<String>("language").cloned(),
            preview: matches.get_flag("preview"),
            force: matches.get_flag("force"),
            dry_run: matches.get_flag("dry-run"),
        })
    }

    /// Parse search subcommand arguments
    fn parse_search_args(matches: &ArgMatches) -> Result<SearchArgs, RiceGrepError> {
        let pattern = matches
            .get_one::<String>("pattern")
            .cloned()
            .unwrap_or_default();

        let paths = matches
            .get_many::<String>("path")
            .unwrap_or_default()
            .map(PathBuf::from)
            .collect();

        // Validate ignore file if specified
        let ignore_file = if let Some(ignore_file_str) = matches.get_one::<String>("ignore-file") {
            let path = PathBuf::from(ignore_file_str);
            if !path.exists() {
                return Err(RiceGrepError::Search {
                    message: format!("Ignore file '{}' does not exist", path.display())
                });
            }
            if !path.is_file() {
                return Err(RiceGrepError::Search {
                    message: format!("Ignore file '{}' is not a regular file", path.display())
                });
            }
            Some(path)
        } else {
            None
        };

        Ok(SearchArgs {
            pattern,
            paths,
            case_insensitive: matches.get_flag("ignore-case"),
            case_sensitive: matches.get_flag("case-sensitive"),
            word_regexp: matches.get_flag("word-regexp"),
            fixed_strings: matches.get_flag("fixed-strings"),
            line_number: matches.get_flag("line-number"),
            invert_match: matches.get_flag("invert-match"),
            count: matches.get_flag("count"),
            max_count: matches.get_one::<String>("max-count")
                .and_then(|s| s.parse().ok()),
            before_context: matches.get_one::<String>("before-context")
                .and_then(|s| s.parse().ok()),
            after_context: matches.get_one::<String>("after-context")
                .and_then(|s| s.parse().ok()),
            context: matches.get_one::<String>("context")
                .and_then(|s| s.parse().ok()),
            content: matches.get_flag("content"),
            syntax_highlight: matches.get_flag("syntax-highlight"),
            answer: matches.get_flag("answer"),
            no_rerank: matches.get_flag("no-rerank"),
            ai_enhanced: matches.get_flag("ai-enhanced"),
            natural_language: matches.get_flag("natural-language"),
            replace: matches.get_one::<String>("replace").cloned(),
            preview: matches.get_flag("preview"),
            force: matches.get_flag("force"),
            ignore_file: matches.get_one::<String>("ignore-file")
                .map(PathBuf::from),
            quiet: matches.get_flag("quiet"),
            dry_run: matches.get_flag("dry-run"),
            max_file_size: matches.get_one::<String>("max-file-size")
                .and_then(|s| s.parse().ok()),
            max_files: matches.get_one::<String>("max-files")
                .and_then(|s| s.parse().ok()),
            max_matches: matches.get_one::<String>("max-matches")
                .and_then(|s| s.parse().ok()),
            max_lines: matches.get_one::<String>("max-lines")
                .and_then(|s| s.parse().ok()),
        })
    }

    /// Parse watch subcommand arguments
    fn parse_watch_args(matches: &ArgMatches) -> Result<WatchArgs, RiceGrepError> {
        let paths = matches
            .get_many::<PathBuf>("paths")
            .unwrap_or_default()
            .cloned()
            .collect();

        Ok(WatchArgs {
            paths,
            timeout: matches.get_one::<u64>("timeout").copied(),
            clear_screen: matches.get_flag("clear-screen"),
            max_file_size: matches.get_one::<u64>("max-file-size").copied(),
        })
    }

    /// Parse MCP subcommand arguments
    fn parse_mcp_args(matches: &ArgMatches) -> Result<McpArgs, crate::error::RiceGrepError> {
        Ok(McpArgs {
            port: matches.get_one::<u16>("port").copied(),
            host: matches
                .get_one::<String>("host")
                .cloned()
                .unwrap_or_else(|| "localhost".to_string()),
            no_watch: matches.get_flag("no-watch"),
        })
    }

    fn parse_index_args(matches: &ArgMatches) -> Result<IndexArgs, crate::error::RiceGrepError> {
        let command = if let Some(_) = matches.subcommand_matches("build") {
            IndexCommand::Build
        } else if let Some(_) = matches.subcommand_matches("update") {
            IndexCommand::Update
        } else if let Some(_) = matches.subcommand_matches("clear") {
            IndexCommand::Clear
        } else if let Some(_) = matches.subcommand_matches("status") {
            IndexCommand::Status
        } else {
            return Err(crate::error::RiceGrepError::Search {
                message: "No index subcommand specified".to_string(),
            });
        };

        let paths = matches
            .get_many::<PathBuf>("paths")
            .unwrap_or_default()
            .cloned()
            .collect();

        Ok(IndexArgs { command, paths })
    }

    /// Parse install subcommand arguments
    fn parse_install_args(matches: &ArgMatches) -> Result<InstallArgs, RiceGrepError> {
        let plugin = matches
            .get_one::<String>("plugin")
            .cloned()
            .ok_or_else(|| RiceGrepError::Search {
                message: "Plugin name required".to_string()
            })?;

        Ok(InstallArgs {
            plugin,
            version: matches.get_one::<String>("version").cloned(),
            force: matches.get_flag("force"),
        })
    }

    /// Parse uninstall subcommand arguments
    fn parse_uninstall_args(matches: &ArgMatches) -> Result<UninstallArgs, RiceGrepError> {
        let plugin = matches
            .get_one::<String>("plugin")
            .cloned()
            .ok_or_else(|| RiceGrepError::Search {
                message: "Plugin name required".to_string()
            })?;

        Ok(UninstallArgs {
            plugin,
            force: matches.get_flag("force"),
        })
    }

    /// Parse legacy arguments for backward compatibility
    fn parse_legacy_args(matches: &ArgMatches) -> Result<LegacyArgs, RiceGrepError> {
        let pattern = matches
            .get_one::<String>("pattern")
            .cloned()
            .unwrap_or_default();

        let paths = matches
            .get_many::<String>("path")
            .unwrap_or_default()
            .map(PathBuf::from)
            .collect();

        Ok(LegacyArgs {
            pattern,
            paths,
            case_insensitive: matches.get_flag("ignore-case"),
            case_sensitive: matches.get_flag("case-sensitive"),
            word_regexp: matches.get_flag("word-regexp"),
            fixed_strings: matches.get_flag("fixed-strings"),
            line_number: matches.get_flag("line-number"),
            line_number_flag: matches.get_flag("line-number"),
            no_line_number: matches.get_flag("no-line-number"),
            files: matches.get_flag("files"),
            files_with_matches: matches.get_flag("files-with-matches"),
            files_without_match: matches.get_flag("files-without-match"),
            count: matches.get_flag("count"),
            invert_match: matches.get_flag("invert-match"),
            max_count: matches.get_one::<String>("max-count")
                .and_then(|s| s.parse().ok()),
            before_context: matches.get_one::<String>("before-context")
                .and_then(|s| s.parse().ok()),
            after_context: matches.get_one::<String>("after-context")
                .and_then(|s| s.parse().ok()),
            context: matches.get_one::<String>("context")
                .and_then(|s| s.parse().ok()),
            follow: matches.get_flag("follow"),
            hidden: matches.get_flag("hidden"),
            no_ignore: matches.get_flag("no-ignore"),
            color: ColorChoice::Auto, // Default for now
            threads: matches.get_one::<String>("threads")
                .and_then(|s| s.parse().ok()),
            ai_enhanced: matches.get_flag("ai-enhanced"),
            natural_language: matches.get_flag("natural-language"),
            fuzzy: matches.get_one::<String>("fuzzy")
                .and_then(|s| s.parse().ok()),
            output_format: if matches.get_flag("json") {
                OutputFormat::Json
            } else {
                OutputFormat::Text
            },
            index_build: matches.get_flag("index-build"),
            index_update: matches.get_flag("index-update"),
            index_watch: matches.get_flag("index-watch"),
            index_status: matches.get_flag("index-status"),
            replace: matches.get_one::<String>("replace").cloned(),
            preview: matches.get_flag("preview"),
            force: matches.get_flag("force"),
            with_filename: matches.get_flag("with-filename"),
            no_filename: matches.get_flag("no-filename"),
        })
    }

    /// Build the clap command with all arguments
    pub fn command() -> Command {
        Command::new("ricegrep")
            .version(env!("CARGO_PKG_VERSION"))
            .author("RiceCoder")
            .about("AI-enhanced code search tool with ripgrep compatibility")
            .subcommand_required(false)
            .arg_required_else_help(false)
            .subcommand(
                Command::new("replace")
                    .about("Rename symbols with language awareness")
                    .arg(
                        Arg::new("old_symbol")
                            .help("The symbol to rename")
                            .required(true)
                            .index(1)
                    )
                    .arg(
                        Arg::new("new_symbol")
                            .help("The new symbol name")
                            .required(true)
                            .index(2)
                    )
                    .arg(
                        Arg::new("file")
                            .help("File containing the symbol")
                            .required(true)
                            .index(3)
                            .value_parser(value_parser!(PathBuf))
                    )
                    .arg(
                        Arg::new("language")
                            .long("language")
                            .short('l')
                            .help("Programming language (auto-detected if not specified)")
                            .value_parser(value_parser!(String))
                    )
                    .arg(
                        Arg::new("preview")
                            .long("preview")
                            .short('p')
                            .help("Show preview of changes without applying them")
                            .action(ArgAction::SetTrue)
                    )
                    .arg(
                        Arg::new("force")
                            .long("force")
                            .short('f')
                            .help("Force the rename operation without confirmation")
                            .action(ArgAction::SetTrue)
                    )
                    .arg(
                        Arg::new("dry-run")
                            .long("dry-run")
                            .short('d')
                            .help("Show what would be renamed without making changes")
                            .action(ArgAction::SetTrue)
                    )
            )
            .subcommand(
                Command::new("search")
                    .about("File pattern searcher")
                    .arg(
                        Arg::new("pattern")
                            .help("The pattern to search for")
                            .required(false)
                            .index(1),
                    )
                    .arg(
                        Arg::new("path")
                            .help("The path to search in")
                            .index(2)
                            .num_args(1..),
                    )
                    .arg(
                        Arg::new("ignore-case")
                            .short('i')
                            .long("ignore-case")
                            .help("Makes the search case-insensitive")
                            .action(clap::ArgAction::SetTrue),
                    )
                    .arg(
                        Arg::new("case-sensitive")
                            .short('s')
                            .long("case-sensitive")
                            .help("Makes the search case-sensitive")
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
                        Arg::new("invert-match")
                            .short('v')
                            .long("invert-match")
                            .help("Invert match (show non-matching lines)")
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
                        Arg::new("max-count")
                            .short('m')
                            .long("max-count")
                            .help("The maximum number of results to return")
                            .value_name("max_count"),
                    )
                    .arg(
                        Arg::new("before-context")
                            .short('B')
                            .long("before-context")
                            .help("Lines before match")
                            .value_name("NUM"),
                    )
                    .arg(
                        Arg::new("after-context")
                            .short('A')
                            .long("after-context")
                            .help("Lines after match")
                            .value_name("NUM"),
                    )
                    .arg(
                        Arg::new("context")
                            .short('C')
                            .long("context")
                            .help("Lines before and after match")
                            .value_name("NUM"),
                    )
                    .arg(
                        Arg::new("content")
                            .short('o')
                            .long("content")
                            .help("Show content of the results")
                            .action(clap::ArgAction::SetTrue),
                    )
                    .arg(
                        Arg::new("syntax-highlight")
                            .long("syntax-highlight")
                            .help("Enable syntax highlighting for content display")
                            .action(clap::ArgAction::SetTrue),
                    )
                    .arg(
                        Arg::new("answer")
                            .short('a')
                            .long("answer")
                            .help("Generate an answer to the question based on the results")
                            .action(clap::ArgAction::SetTrue),
                    )
                    .arg(
                        Arg::new("no-rerank")
                            .long("no-rerank")
                            .help("Disable reranking of search results")
                            .action(clap::ArgAction::SetTrue),
                    )
                    .arg(
                        Arg::new("ai-enhanced")
                            .long("ai-enhanced")
                            .help("Enable AI-enhanced search")
                            .action(clap::ArgAction::SetTrue),
                    )
                    .arg(
                        Arg::new("natural-language")
                            .long("natural-language")
                            .help("Process query as natural language")
                            .action(clap::ArgAction::SetTrue),
                    )
                    .arg(
                        Arg::new("replace")
                            .short('r')
                            .long("replace")
                            .help("Replace matches with specified pattern")
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
                            .help("Execute replace operations")
                            .action(clap::ArgAction::SetTrue),
                    )
                    .arg(
                        Arg::new("ignore-file")
                            .long("ignore-file")
                            .help("Specify custom ignore file (e.g., .ricegrepignore)")
                            .value_name("FILE"),
                    )
                    .arg(
                        Arg::new("quiet")
                            .short('q')
                            .long("quiet")
                            .help("Suppress progress output and spinners")
                            .action(clap::ArgAction::SetTrue),
                    )
                    .arg(
                        Arg::new("dry-run")
                            .long("dry-run")
                            .help("Show what would be done without making changes")
                            .action(clap::ArgAction::SetTrue),
                    )
                    .arg(
                        Arg::new("max-file-size")
                            .long("max-file-size")
                            .help("Maximum file size in bytes to search/index")
                            .value_name("BYTES"),
                    )
                    .arg(
                        Arg::new("max-files")
                            .long("max-files")
                            .help("Maximum number of files to process")
                            .value_name("COUNT"),
                    )
                    .arg(
                        Arg::new("max-matches")
                            .long("max-matches")
                            .help("Maximum number of matches to return")
                            .value_name("COUNT"),
                    )
                    .arg(
                        Arg::new("max-lines")
                            .long("max-lines")
                            .help("Maximum number of lines to display per file (for --content)")
                            .value_name("COUNT"),
                    )
            )
            .subcommand(
                Command::new("watch")
                    .about("Watch mode for continuous indexing")
                    .arg(
                        Arg::new("timeout")
                            .long("timeout")
                            .short('t')
                            .help("Watch timeout in seconds")
                            .value_parser(value_parser!(u64))
                    )
                    .arg(
                        Arg::new("clear-screen")
                            .long("clear-screen")
                            .short('c')
                            .help("Clear screen between updates")
                            .action(ArgAction::SetTrue)
                    )
                    .arg(
                        Arg::new("max-file-size")
                            .long("max-file-size")
                            .help("Maximum file size to index (in bytes)")
                            .value_parser(value_parser!(u64))
                    )
                    .arg(
                        Arg::new("paths")
                            .help("Paths to watch")
                            .index(1)
                            .num_args(1..)
                            .value_parser(value_parser!(PathBuf))
                    )
            )
            .subcommand(
                Command::new("mcp")
                    .about("Start MCP server for AI assistant integration")
                    .arg(
                        Arg::new("port")
                            .long("port")
                            .short('p')
                            .help("Port for MCP server (stdio mode if not specified)")
                            .value_parser(value_parser!(u16))
                    )
                    .arg(
                        Arg::new("host")
                            .long("host")
                            .help("Host for MCP server")
                            .default_value("localhost")
                            .value_parser(value_parser!(String))
                    )
                    .arg(
                        Arg::new("no-watch")
                            .long("no-watch")
                            .help("Disable automatic background watch mode")
                            .action(ArgAction::SetTrue)
                    )
            )
            .subcommand(
                Command::new("index")
                    .about("Manage search indexes")
                    .subcommand(
                        Command::new("build")
                            .about("Build search index for faster queries")
                            .arg(
                                Arg::new("paths")
                                    .help("Paths to index")
                                    .index(1)
                                    .num_args(0..)
                                    .value_parser(value_parser!(PathBuf))
                            )
                    )
                    .subcommand(
                        Command::new("update")
                            .about("Update existing search index with changed files")
                            .arg(
                                Arg::new("paths")
                                    .help("Paths to update in index")
                                    .index(1)
                                    .num_args(0..)
                                    .value_parser(value_parser!(PathBuf))
                            )
                    )
                    .subcommand(
                        Command::new("clear")
                            .about("Clear search index")
                    )
                    .subcommand(
                        Command::new("status")
                            .about("Show index status and statistics")
                    )
            )
            .subcommand(
                Command::new("install")
                    .about("Install plugins and integrations")
                    .arg(
                        Arg::new("plugin")
                            .help("Plugin to install")
                            .required(true)
                            .index(1),
                    )
                    .arg(
                        Arg::new("version")
                            .long("version")
                            .help("Version to install")
                            .value_name("VERSION"),
                    )
                    .arg(
                        Arg::new("force")
                            .short('f')
                            .long("force")
                            .help("Force installation")
                            .action(clap::ArgAction::SetTrue),
                    )
            )
            .subcommand(
                Command::new("uninstall")
                    .about("Uninstall plugins and integrations")
                    .arg(
                        Arg::new("plugin")
                            .help("Plugin to uninstall")
                            .required(true)
                            .index(1),
                    )
                    .arg(
                        Arg::new("force")
                            .short('f')
                            .long("force")
                            .help("Force uninstallation")
                            .action(clap::ArgAction::SetTrue),
                    )
            )
            .arg(
                Arg::new("pattern")
                    .help("The pattern to search for (legacy mode)")
                    .index(1),
            )
            .arg(
                Arg::new("path")
                    .help("Files or directories to search (legacy mode)")
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