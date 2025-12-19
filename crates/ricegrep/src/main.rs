//! RiceGrep main binary
//!
//! Provides the command-line interface for RiceGrep with ripgrep-compatible
//! search functionality and AI enhancements.

use ricegrep::args::{Args, RiceGrepCommand, SearchArgs, ReplaceArgs, WatchArgs, McpArgs, InstallArgs, UninstallArgs, LegacyArgs};
use ricegrep::mcp::RiceGrepMcpServer;
use ricegrep::search::{RegexSearchEngine, SearchEngine, SearchQuery, ProgressVerbosity};
use ricegrep::output::OutputFormatter;
use ricegrep::config::{OutputFormat, ColorChoice};
use ricegrep::ai::RiceGrepAIProcessor;
use ricegrep::tui::RiceGrepTUI;
use ricegrep::watch::WatchEngine;
use ricegrep::replace::{ReplaceEngine, SymbolRenameOperation};
// use ricegrep::database::{DatabaseManager, DatabaseConfig, SearchHistory, UserPreferences, IndexMetadata, IndexStatus}; // Disabled
use std::sync::Arc;
use detect_lang::Language;
use ricegrep::error::RiceGrepError;
use std::io::{self, Write};
use std::path::PathBuf;
use tokio;

async fn handle_search_command(args: SearchArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Handle replace operations if specified
    if args.replace.is_some() {
        return handle_replace_in_search(args).await;
    }

    // Handle answer generation if requested
    if args.answer {
        return handle_answer_command(args).await;
    }

    // Determine display options (ripgrep compatibility)
    let is_multiple_files = args.paths.len() > 1 ||
        (args.paths.len() == 1 && std::fs::metadata(&args.paths[0]).map(|m| m.is_dir()).unwrap_or(false));

    // Show line numbers by default for multiple files
    let show_line_numbers = args.line_number || is_multiple_files;

    // Create search query from args
    let query = SearchQuery {
        pattern: args.pattern.clone(),
        paths: args.paths,
        case_insensitive: args.case_insensitive,
        case_sensitive: args.case_sensitive,
        word_regexp: args.word_regexp,
        fixed_strings: args.fixed_strings,
        follow: false, // Not implemented in subcommand yet
        hidden: false, // Not implemented in subcommand yet
        no_ignore: false, // Not implemented in subcommand yet
        ignore_file: args.ignore_file.clone(),
        quiet: args.quiet,
        dry_run: args.dry_run,
        max_file_size: args.max_file_size,
        progress_verbosity: if args.quiet { ProgressVerbosity::Quiet } else { ProgressVerbosity::Normal },
        max_files: args.max_files,
        max_matches: args.max_matches,
        max_lines: args.max_lines,
        invert_match: args.invert_match,
        ai_enhanced: args.ai_enhanced || args.natural_language,
        no_rerank: args.no_rerank,
        fuzzy: None,
        max_count: args.max_count,
        spelling_correction: None,
    };

    // Create search engine - AI processor is lazy-loaded only when needed
    let mut search_engine = RegexSearchEngine::new();

    // Execute search with performance monitoring
    let search_start = std::time::Instant::now();
    let results = search_engine.search(query).await?;
    let search_duration = search_start.elapsed();

    // Store search history in database if available (disabled)
    // TODO: Re-enable when database is fixed
    // if let Some(db_manager) = &database_manager {
    //     let search_history = SearchHistory {
    //         id: uuid::Uuid::new_v4(),
    //         user_id: std::env::var("RICEGREP_USER_ID").ok(),
    //         query: args.pattern.clone(),
    //         results_count: results.total_matches,
    //         execution_time_ms: search_duration.as_millis() as u64,
    //         timestamp: chrono::Utc::now(),
    //         ai_used: args.ai_enhanced || args.natural_language,
    //         success: true,
    //     };
    //
    //     if let Err(e) = db_manager.store_search_history(search_history).await {
    //         eprintln!("Warning: Failed to store search history: {}", e);
    //     }
    // }

    // Log performance metrics if requested
    if std::env::var("RICEGREP_PERF").is_ok() {
        eprintln!("Search completed in {:.2}ms, found {} matches in {} files",
                  search_duration.as_secs_f64() * 1000.0,
                  results.total_matches,
                  results.files_searched);
    }

    // Create output formatter
    let formatter = OutputFormatter::with_syntax_highlight(
        OutputFormat::Text,
        ColorChoice::Auto,
        show_line_numbers,
        true, // heading
        is_multiple_files, // filename
        args.ai_enhanced || args.natural_language,
        args.count,
        args.content,
        args.max_lines,
        args.syntax_highlight,
    );

    // Output results
    formatter.write_results(&results)?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args = Args::parse()?;

    // For now, only handle search command
    match args.command {
        RiceGrepCommand::Search(search_args) => {
            // Database disabled for now due to compatibility issues
            // TODO: Re-enable when Scylla compatibility is fixed

            // Determine if we're searching multiple files
            let is_multiple_files =
                search_args.paths.len() > 1 ||
                (search_args.paths.len() == 1 && std::fs::metadata(&search_args.paths[0]).map(|m| m.is_dir()).unwrap_or(false));

            // Show line numbers by default for multiple files
            let show_line_numbers = search_args.line_number || is_multiple_files;

            // Create search query from args
            let query = SearchQuery {
                pattern: search_args.pattern.clone(),
                paths: search_args.paths,
                case_insensitive: search_args.case_insensitive,
                case_sensitive: search_args.case_sensitive,
                word_regexp: search_args.word_regexp,
                fixed_strings: search_args.fixed_strings,
                follow: false, // Not implemented in subcommand yet
                hidden: false, // Not implemented in subcommand yet
                no_ignore: false, // Not implemented in subcommand yet
                ignore_file: None, // TODO: Add ignore_file to SearchArgs
                quiet: false, // TODO: Add quiet to SearchArgs
                dry_run: false, // TODO: Add dry_run to SearchArgs
                max_file_size: None, // TODO: Add max_file_size to SearchArgs
                progress_verbosity: ProgressVerbosity::Normal,
                max_files: None, // TODO: Add max_files to SearchArgs
                max_matches: search_args.max_count,
                max_lines: None, // TODO: Add max_lines to SearchArgs
                invert_match: search_args.invert_match,
                ai_enhanced: search_args.ai_enhanced || search_args.natural_language,
                no_rerank: search_args.no_rerank,
                fuzzy: None,
                max_count: search_args.max_count,
                spelling_correction: None,
            };

            // Create search engine - AI processor is lazy-loaded only when needed
            let mut search_engine = RegexSearchEngine::new();

            // Execute search with performance monitoring
            let search_start = std::time::Instant::now();
            let results = search_engine.search(query).await?;
            let search_duration = search_start.elapsed();

            // Store search history in database if available (disabled)
            // TODO: Re-enable when database is fixed

            // Output results
            let formatter = OutputFormatter::new(
                OutputFormat::Text,
                ColorChoice::Auto,
                show_line_numbers,
                true, // heading
                true, // filename
                search_args.ai_enhanced || search_args.natural_language, // ai_enabled
                search_args.count,
                search_args.content,
                None, // max_lines
            );
            formatter.write_results(&results)?;

            // Log performance metrics if requested
            if std::env::var("RICEGREP_PERF").is_ok() {
                eprintln!("Search completed in {:.2}ms, found {} matches in {} files",
                          search_duration.as_secs_f64() * 1000.0,
                          results.total_matches,
                          results.files_searched);
            }
        }
        RiceGrepCommand::Replace(replace_args) => {
            handle_replace_command(replace_args).await?;
        }
        RiceGrepCommand::Watch(watch_args) => {
            handle_watch_command(watch_args).await?;
        }
        RiceGrepCommand::Mcp(mcp_args) => {
            handle_mcp_command(mcp_args).await?;
        }
        RiceGrepCommand::Install(install_args) => {
            handle_install_command(install_args).await?;
        }
        RiceGrepCommand::Uninstall(uninstall_args) => {
            handle_uninstall_command(uninstall_args).await?;
        }
        RiceGrepCommand::Legacy(legacy_args) => {
            handle_legacy_command(legacy_args).await?;
        }
    }

    Ok(())
}

async fn handle_replace_command(args: ReplaceArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Detect language if not specified
    let language = if let Some(_lang_str) = args.language {
        // For now, use a generic language - full language detection can be added later
        Language("generic", "generic")
    } else {
        // Try to detect from file extension
        detect_lang::from_path(&args.file_path).unwrap_or(Language("unknown", "unknown"))
    };

    // Create symbol rename operation
    let operation = SymbolRenameOperation {
        file_path: args.file_path.clone(),
        old_symbol: args.old_symbol.clone(),
        new_symbol: args.new_symbol.clone(),
        language,
    };

    // Create replace engine
    let engine = ReplaceEngine::new();

    // Validate the operation
    engine.validate_symbol_rename(&operation)?;

    // Handle preview mode
    if args.preview || args.dry_run {
        println!("Preview of symbol rename operation:");
        println!("  File: {}", args.file_path.display());
        println!("  Language: {}", operation.language.0);
        println!("  Symbol: '{}' -> '{}'", args.old_symbol, args.new_symbol);
        println!();

        if args.dry_run {
            println!("Dry-run mode: No changes will be made.");
            return Ok(());
        }

        // For preview, we would need to show the actual changes
        // For now, just show what would happen
        println!("This would rename all occurrences of '{}' to '{}' in the file.", args.old_symbol, args.new_symbol);
        return Ok(());
    }

    // Execute the rename operation
    let result = engine.execute_symbol_rename(operation).await?;

    // Report results
    if result.symbols_renamed > 0 {
        println!("Successfully renamed symbol '{}' to '{}' in {} file(s)",
                args.old_symbol, args.new_symbol, result.files_modified);

        if let Some(impact) = result.impact_summary {
            println!("Impact: {}", impact);
        }
    } else {
        println!("No symbols were renamed. The symbol '{}' was not found in the file.", args.old_symbol);
    }

    // Report any errors
    for error in result.errors {
        eprintln!("Error: {}", error);
    }

    Ok(())
}

async fn handle_answer_command(args: SearchArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Create search query from args
    let query = SearchQuery {
        pattern: args.pattern.clone(),
        paths: args.paths,
        case_insensitive: args.case_insensitive,
        case_sensitive: args.case_sensitive,
        word_regexp: args.word_regexp,
        fixed_strings: args.fixed_strings,
        follow: false, // Not implemented in subcommand yet
        hidden: false, // Not implemented in subcommand yet
        no_ignore: false, // Not implemented in subcommand yet
        ignore_file: args.ignore_file.clone(),
        quiet: args.quiet,
        dry_run: args.dry_run,
        max_file_size: args.max_file_size,
        progress_verbosity: if args.quiet { ProgressVerbosity::Quiet } else { ProgressVerbosity::Normal },
        max_files: args.max_files,
        max_matches: args.max_matches,
        max_lines: args.max_lines,
        invert_match: args.invert_match,
        ai_enhanced: args.ai_enhanced || args.natural_language,
        no_rerank: args.no_rerank,
        fuzzy: None,
        max_count: args.max_count,
        spelling_correction: None,
    };

    // Create search engine
    let mut search_engine = RegexSearchEngine::new();

    // Execute search
    let results = search_engine.search(query).await?;

    // Generate AI answer
    match search_engine.generate_answer(&args.pattern, &results).await {
        Ok(answer) => {
            println!("{}", answer);
        }
        Err(e) => {
            eprintln!("Failed to generate AI answer: {}", e);
            // Fall back to regular output
            let formatter = OutputFormatter::with_syntax_highlight(
                OutputFormat::Text,
                ColorChoice::Auto,
                true, // line numbers
                true, // heading
                true, // filename
                args.ai_enhanced || args.natural_language,
                args.count,
                args.content,
                args.max_lines,
                args.syntax_highlight,
            );
            formatter.write_results(&results)?;
        }
    }

    Ok(())
}

async fn handle_replace_in_search(args: SearchArgs) -> Result<(), Box<dyn std::error::Error>> {
    use ricegrep::replace::{ReplaceEngine, ReplaceOperation, ReplaceResult};

    let replace_pattern = args.replace.as_ref()
        .ok_or_else(|| RiceGrepError::Search { message: "Replace pattern not specified".to_string() })?;

    // Create search query
    let query = SearchQuery {
        pattern: args.pattern.clone(),
        paths: args.paths,
        case_insensitive: args.case_insensitive,
        case_sensitive: args.case_sensitive,
        word_regexp: args.word_regexp,
        fixed_strings: args.fixed_strings,
        follow: false, // Not implemented in subcommand yet
        hidden: false, // Not implemented in subcommand yet
        no_ignore: false, // Not implemented in subcommand yet
        ignore_file: args.ignore_file.clone(),
        quiet: args.quiet,
        dry_run: args.dry_run,
        max_file_size: args.max_file_size,
        progress_verbosity: if args.quiet { ProgressVerbosity::Quiet } else { ProgressVerbosity::Normal },
        max_files: args.max_files,
        max_matches: args.max_matches,
        max_lines: args.max_lines,
        invert_match: false, // Not applicable for replace operations
        ai_enhanced: false, // Disable AI for replace operations for safety
        no_rerank: false, // Not applicable for replace operations
        fuzzy: None,
        max_count: None,
        spelling_correction: None,
    };

    // Create search engine and find matches
    let mut search_engine = RegexSearchEngine::new();
    let results = search_engine.search(query).await?;

    if results.matches.is_empty() {
        println!("No matches found for pattern: {}", args.pattern);
        return Ok(());
    }

    // Create replace operations
    let mut operations = Vec::new();
    for match_result in &results.matches {
        operations.push(ReplaceOperation {
            file_path: match_result.file.clone(),
            line_number: match_result.line_number,
            old_content: match_result.line_content.clone(),
            new_content: match_result.line_content.replace(&args.pattern, replace_pattern),
            byte_offset: match_result.byte_offset,
        });
    }

    // Create replace engine
    let replace_engine = ReplaceEngine::new();

    if args.preview || args.dry_run {
        // Preview/dry-run mode - show what would be changed
        let mode = if args.dry_run { "Dry-run" } else { "Preview" };
        println!("{} of replace operations:", mode);
        println!("Pattern: '{}' -> '{}'", args.pattern, replace_pattern);
        println!("Files to modify: {}", operations.iter().map(|op| op.file_path.display().to_string()).collect::<std::collections::HashSet<_>>().len());
        println!("Total operations: {}", operations.len());
        println!();

        for operation in operations.iter().take(10) { // Show first 10
            println!("{}:{}:", operation.file_path.display(), operation.line_number);
            println!("  - {}", operation.old_content.trim());
            println!("  + {}", operation.new_content.trim());
        }

        if operations.len() > 10 {
            println!("... and {} more operations", operations.len() - 10);
        }

        println!();
        println!("Use --force to execute these changes.");
    } else if args.force {
        // Execute replace operations
        println!("Executing replace operations on {} files...", operations.iter().map(|op| op.file_path.display().to_string()).collect::<std::collections::HashSet<_>>().len());
        let start_time = std::time::Instant::now();
        let results = replace_engine.execute_operations(operations).await?;
        let replace_time = start_time.elapsed();

        println!("Replace operations completed in {:.2}s:", replace_time.as_secs_f64());
        println!("  Files modified: {}", results.files_modified);
        println!("  Operations successful: {}", results.operations_successful);
        if results.operations_failed > 0 {
            println!("  Operations failed: {}", results.operations_failed);
        }

        if !results.errors.is_empty() {
            println!("Errors encountered:");
            for error in results.errors.iter().take(3) {
                println!("  {}", error);
            }
            if results.errors.len() > 3 {
                println!("  ... and {} more errors", results.errors.len() - 3);
            }
        } else {
            println!("All operations completed successfully!");
        }
    } else {
        eprintln!("Error: Replace operations require --preview or --force flag.");
        return Err("Replace operations require --preview or --force flag".into());
    }

    Ok(())
}

async fn handle_replace_in_legacy(args: LegacyArgs) -> Result<(), Box<dyn std::error::Error>> {
    use ricegrep::replace::{ReplaceEngine, ReplaceOperation, ReplaceResult};

    let replace_pattern = args.replace.as_ref()
        .ok_or_else(|| RiceGrepError::Search { message: "Replace pattern not specified".to_string() })?;

    // Create search query
    let query = SearchQuery {
        pattern: args.pattern.clone(),
        paths: args.paths,
        case_insensitive: args.case_insensitive,
        case_sensitive: args.case_sensitive,
        word_regexp: args.word_regexp,
        fixed_strings: args.fixed_strings,
        follow: args.follow,
        hidden: args.hidden,
        no_ignore: args.no_ignore,
        ignore_file: None, // Legacy mode doesn't support custom ignore files
        quiet: false, // Legacy mode doesn't support quiet flag
        dry_run: false, // Legacy mode doesn't support dry-run flag
        max_file_size: None, // Legacy mode doesn't support max file size
        progress_verbosity: ProgressVerbosity::Quiet, // Legacy mode doesn't support progress verbosity
        max_files: None, // Legacy mode doesn't support file quotas
        max_matches: None, // Legacy mode doesn't support match quotas
        max_lines: None, // Legacy mode doesn't support line limits
        invert_match: false,
        ai_enhanced: false, // Disable AI for replace operations for safety
        no_rerank: false, // Not applicable for replace operations
        fuzzy: None,
        max_count: None,
        spelling_correction: None,
    };

    // Create search engine and find matches
    let mut search_engine = RegexSearchEngine::new();
    let results = search_engine.search(query).await?;

    if results.matches.is_empty() {
        println!("No matches found for pattern: {}", args.pattern);
        return Ok(());
    }

    // Create replace operations
    let mut operations = Vec::new();
    for match_result in &results.matches {
        operations.push(ReplaceOperation {
            file_path: match_result.file.clone(),
            line_number: match_result.line_number,
            old_content: match_result.line_content.clone(),
            new_content: match_result.line_content.replace(&args.pattern, replace_pattern),
            byte_offset: match_result.byte_offset,
        });
    }

    // Create replace engine
    let replace_engine = ReplaceEngine::new();

    if args.preview {
        // Preview mode - show what would be changed
        println!("Preview of replace operations:");
        println!("Pattern: '{}' -> '{}'", args.pattern, replace_pattern);
        println!("Files to modify: {}", operations.iter().map(|op| op.file_path.display().to_string()).collect::<std::collections::HashSet<_>>().len());
        println!("Total operations: {}", operations.len());
        println!();

        for operation in operations.iter().take(10) { // Show first 10
            println!("{}:{}:", operation.file_path.display(), operation.line_number);
            println!("  - {}", operation.old_content.trim());
            println!("  + {}", operation.new_content.trim());
        }

        if operations.len() > 10 {
            println!("... and {} more operations", operations.len() - 10);
        }

        println!();
        println!("Use --force to execute these changes.");
    } else if args.force {
        // Execute replace operations
        println!("Executing replace operations on {} files...", operations.iter().map(|op| op.file_path.display().to_string()).collect::<std::collections::HashSet<_>>().len());
        let start_time = std::time::Instant::now();
        let results = replace_engine.execute_operations(operations).await?;
        let replace_time = start_time.elapsed();

        println!("Replace operations completed in {:.2}s:", replace_time.as_secs_f64());
        println!("  Files modified: {}", results.files_modified);
        println!("  Operations successful: {}", results.operations_successful);
        if results.operations_failed > 0 {
            println!("  Operations failed: {}", results.operations_failed);
        }

        if !results.errors.is_empty() {
            println!("Errors encountered:");
            for error in results.errors.iter().take(3) {
                println!("  {}", error);
            }
            if results.errors.len() > 3 {
                println!("  ... and {} more errors", results.errors.len() - 3);
            }
        } else {
            println!("All operations completed successfully!");
        }
    } else {
        eprintln!("Error: Replace operations require --preview or --force flag.");
        std::process::exit(1);
    }

    Ok(())
}

async fn handle_watch_command(args: WatchArgs) -> Result<(), Box<dyn std::error::Error>> {
    use ricegrep::watch::{WatchConfig, WatchEngine};
    use std::path::PathBuf;

    println!("Starting RiceGrep watch mode...");
    println!("Watching paths: {:?}", args.paths);
    if let Some(timeout) = args.timeout {
        println!("Timeout: {} seconds", timeout);
    }
    if args.clear_screen {
        println!("Clear screen mode enabled");
    }

    // Create watch configuration
    let watch_config = WatchConfig {
        paths: args.paths.clone(),
        timeout: args.timeout,
        clear_screen: args.clear_screen,
        debounce_ms: 500, // Default debounce
    };

    // Create index directory
    let index_dir = std::env::current_dir()?.join(".ricegrep");
    std::fs::create_dir_all(&index_dir)?;

    // Create watch engine
    let mut watch_engine = WatchEngine::new(watch_config, index_dir);

    // Start watching
    println!("Initializing file watcher...");
    watch_engine.start().await?;

    Ok(())
}

async fn handle_mcp_command(args: McpArgs) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting RiceGrep MCP server...");
    println!("Port: {:?}", args.port);
    println!("Host: {}", args.host);

    // For now, start the stdio server (most common for MCP)
    let server = RiceGrepMcpServer::new();
    server.start_stdio_server().await?;

    Ok(())
}

async fn handle_install_command(args: InstallArgs) -> Result<(), Box<dyn std::error::Error>> {
    match args.plugin.as_str() {
        "claude-code" => install_claude_code().await,
        "opencode" => install_opencode().await,
        "codex" => install_codex().await,
        "factory-droid" => install_factory_droid().await,
        "cursor" => install_cursor().await,
        "windsurf" => install_windsurf().await,
        _ => {
            println!("Unknown plugin: {}", args.plugin);
            println!("Available plugins: claude-code, opencode, codex, factory-droid, cursor, windsurf");
            Ok(())
        }
    }
}

async fn install_claude_code() -> Result<(), Box<dyn std::error::Error>> {
    println!("Installing RiceGrep integration for Claude Code...");
    println!("");
    println!("This will:");
    println!("1. Add RiceGrep to Claude Code's marketplace");
    println!("2. Install the RiceGrep plugin");
    println!("3. Configure skills for search and replace operations");
    println!("");
    println!("Note: This is a placeholder implementation.");
    println!("Full integration requires Claude Code plugin marketplace setup.");
    println!("");
    println!("To complete manually:");
    println!("1. Open Claude Code");
    println!("2. Run: /plugin marketplace add ricegrep");
    println!("3. Run: /plugin install ricegrep");
    println!("");
    println!("Claude Code integration ready (manual setup required).");

    Ok(())
}

async fn install_opencode() -> Result<(), Box<dyn std::error::Error>> {
    println!("Installing RiceGrep integration for OpenCode...");
    println!("");
    println!("This will:");
    println!("1. Create plugin files in .opencode/plugin/");
    println!("2. Set up event hooks for file operations");
    println!("3. Configure search and replace tools");
    println!("");
    println!("Note: This is a placeholder implementation.");
    println!("Full integration requires OpenCode plugin system.");
    println!("");
    println!("OpenCode integration ready (manual setup required).");

    Ok(())
}

async fn install_codex() -> Result<(), Box<dyn std::error::Error>> {
    println!("Installing RiceGrep integration for Codex...");
    println!("");
    println!("This will:");
    println!("1. Add RiceGrep skills to AGENTS.md");
    println!("2. Configure MCP server integration");
    println!("3. Set up background search capabilities");
    println!("");
    println!("Note: This is a placeholder implementation.");
    println!("Full integration requires Codex skills system.");
    println!("");
    println!("Codex integration ready (manual setup required).");

    Ok(())
}

async fn install_factory_droid() -> Result<(), Box<dyn std::error::Error>> {
    println!("Installing RiceGrep integration for Factory Droid...");
    println!("");
    println!("This will:");
    println!("1. Create hooks in .factory/ directory");
    println!("2. Set up Python hook scripts");
    println!("3. Configure background processes");
    println!("");
    println!("Note: This is a placeholder implementation.");
    println!("Full integration requires Factory Droid hooks system.");
    println!("");
    println!("Factory Droid integration ready (manual setup required).");

    Ok(())
}

async fn install_cursor() -> Result<(), Box<dyn std::error::Error>> {
    println!("Installing RiceGrep integration for Cursor...");
    println!("");
    println!("This will:");
    println!("1. Install Cursor extension");
    println!("2. Configure tool definitions");
    println!("3. Set up MCP integration");
    println!("");
    println!("Note: This is a placeholder implementation.");
    println!("Full integration requires Cursor extension system.");
    println!("");
    println!("Cursor integration ready (manual setup required).");

    Ok(())
}

async fn install_windsurf() -> Result<(), Box<dyn std::error::Error>> {
    println!("Installing RiceGrep integration for Windsurf...");
    println!("");
    println!("This will:");
    println!("1. Configure assistant integration");
    println!("2. Set up tool definitions");
    println!("3. Enable search capabilities");
    println!("");
    println!("Note: This is a placeholder implementation.");
    println!("Full integration requires Windsurf assistant system.");
    println!("");
    println!("Windsurf integration ready (manual setup required).");

    Ok(())
}

async fn handle_uninstall_command(args: UninstallArgs) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Implement plugin uninstallation
    println!("Uninstalling plugin: {}", args.plugin);
    Ok(())
}

async fn handle_legacy_command(args: LegacyArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Handle index operations first
    if args.index_build || args.index_update || args.index_watch || args.index_status {
        return handle_index_operations(&args).await;
    }

    // Handle replace operations
    if args.replace.is_some() {
        return handle_replace_in_legacy(args).await;
    }

    // Convert legacy args to search query for backward compatibility
    let is_multiple_files = args.paths.len() > 1 ||
        (args.paths.len() == 1 && std::fs::metadata(&args.paths[0]).map(|m| m.is_dir()).unwrap_or(false));

    let show_line_numbers = args.line_number_flag || (is_multiple_files && !args.no_line_number);
    let show_filename = !args.no_filename && (args.with_filename || is_multiple_files);

    let query = SearchQuery {
        pattern: args.pattern.clone(),
        paths: args.paths,
        case_insensitive: args.case_insensitive,
        case_sensitive: args.case_sensitive,
        word_regexp: args.word_regexp,
        fixed_strings: args.fixed_strings,
        follow: args.follow,
        hidden: args.hidden,
        no_ignore: args.no_ignore,
        ignore_file: None, // Legacy mode doesn't support custom ignore files
        quiet: false, // Legacy mode doesn't support quiet flag
        dry_run: false, // Legacy mode doesn't support dry-run flag
        max_file_size: None, // Legacy mode doesn't support max file size
        progress_verbosity: ProgressVerbosity::Quiet, // Legacy mode doesn't support progress verbosity
        max_files: None, // Legacy mode doesn't support file quotas
        max_matches: None, // Legacy mode doesn't support match quotas
        max_lines: None, // Legacy mode doesn't support line limits
        invert_match: args.invert_match,
        ai_enhanced: args.ai_enhanced || args.natural_language,
        no_rerank: false, // Legacy mode doesn't have no_rerank flag
        fuzzy: args.fuzzy,
        max_count: args.max_count,
        spelling_correction: None,
    };

    let mut search_engine = RegexSearchEngine::new();

    let search_start = std::time::Instant::now();
    let results = search_engine.search(query).await?;
    let search_duration = search_start.elapsed();

    if std::env::var("RICEGREP_PERF").is_ok() {
        eprintln!("Search completed in {:.2}ms, found {} matches in {} files",
                  search_duration.as_secs_f64() * 1000.0,
                  results.total_matches,
                  results.files_searched);
    }

    let formatter = OutputFormatter::new(
        args.output_format,
        args.color,
        show_line_numbers,
        true, // heading
        show_filename,
        args.ai_enhanced || args.natural_language,
        args.count,
        false, // content display not supported in legacy mode
        None, // max_lines not supported in legacy mode
    );

    formatter.write_results(&results)?;

    Ok(())
}

async fn handle_index_operations(args: &LegacyArgs) -> Result<(), Box<dyn std::error::Error>> {
    use ricegrep::search::{IndexManager, RegexSearchEngine};
    use std::path::PathBuf;

    // Create index manager (same as search engine default)
    let index_dir = PathBuf::from(".ricegrep");

    eprintln!("Index directory: {}", index_dir.display());

    // Ensure index directory exists
    if let Err(e) = std::fs::create_dir_all(&index_dir) {
        eprintln!("Failed to create index directory: {}", e);
        return Ok(());
    }

        let mut index_manager = IndexManager::new(index_dir.clone());

    // Determine root path from args
    let root_path = if args.paths.is_empty() {
        PathBuf::from(".")
    } else {
        args.paths[0].clone()
    };

    if args.index_status {
        // Show index status
        let has_index = index_manager.load_index(&root_path).unwrap_or(false);
        if has_index {
            if index_manager.needs_rebuild(&root_path).unwrap_or(true) {
                println!("Index exists but needs rebuilding");
            } else {
                if let Some(index) = index_manager.get_index() {
                    println!("Index status: Valid");
                    println!("Files indexed: {}", index.metadata.file_count);
                    println!("Lines indexed: {}", index.metadata.line_count);
                }
            }
        } else {
            println!("No index found for path: {}", root_path.display());
        }
    } else if args.index_build {
        println!("Building search index for: {}", root_path.display());
        println!("This may take a moment for large codebases...");

        // Create a temporary search engine to build the index
        let mut search_engine = RegexSearchEngine::new();
        let start_time = std::time::Instant::now();
        search_engine.build_index(&[root_path.clone()], ProgressVerbosity::Normal).await?;
        let build_time = start_time.elapsed();

        println!("Index built successfully in {:.2}s", build_time.as_secs_f64());

        // Try to load it back to verify
        let has_index = index_manager.load_index(&root_path).unwrap_or(false);
        if has_index {
            if let Some(index) = index_manager.get_index() {
                println!("Index verification: Found ({} files, {} lines indexed)",
                        index.metadata.file_count, index.metadata.line_count);
            }
        } else {
            println!("Warning: Index verification failed - save may have failed");
        }
    } else if args.index_update {
        println!("Updating search index for: {}", root_path.display());
        let mut search_engine = RegexSearchEngine::new();
        if search_engine.has_index(&[root_path.clone()]) {
            search_engine.build_index(&[root_path], ProgressVerbosity::Normal).await?;
            println!("Index updated successfully");
        } else {
            println!("No existing index to update. Use --index-build instead.");
        }
    } else if args.index_watch {
        // Create watch configuration
        let watch_config = ricegrep::watch::WatchConfig {
            paths: if args.paths.is_empty() {
                vec![PathBuf::from(".")]
            } else {
                args.paths.clone()
            },
            timeout: None, // Not available in legacy args
            clear_screen: false, // Not available in legacy args
            debounce_ms: 500, // Default debounce
        };

        // Create watch engine
        let mut watch_engine = ricegrep::watch::WatchEngine::new(
            watch_config,
            index_dir,
        );

        // Start watch mode
        watch_engine.start().await?;
    }

    Ok(())
}