//! RiceGrep main binary
//!
//! Provides the command-line interface for RiceGrep with ripgrep-compatible
//! search functionality and AI enhancements.

use ricegrep::args::Args;
use ricegrep::search::{RegexSearchEngine, SearchEngine, SearchQuery};
use ricegrep::output::OutputFormatter;
use ricegrep::config::{OutputFormat, ColorChoice};
use ricegrep::ai::RiceGrepAIProcessor;
use ricegrep::tui::RiceGrepTUI;
use std::io::{self, Write};
use tokio;

async fn handle_index_operations(args: Args) -> Result<(), Box<dyn std::error::Error>> {
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
        search_engine.build_index(&[root_path.clone()]).await?;
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
            search_engine.build_index(&[root_path]).await?;
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
            timeout: args.watch_timeout,
            clear_screen: args.watch_clear,
            debounce_ms: 500, // 500ms debounce
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

async fn handle_replace_operations(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    use ricegrep::search::{RegexSearchEngine, SearchEngine, SearchQuery};
    use ricegrep::replace::{ReplaceEngine, ReplaceOperation, ReplaceResult};
    use std::path::PathBuf;

    // Validate arguments
    if args.pattern.is_empty() {
        eprintln!("Error: Search pattern required for replace operations.");
        eprintln!("Usage: ricegrep [OPTIONS] <pattern> --replace <replacement> [path...]");
        eprintln!("Try 'ricegrep --help' for more information.");
        std::process::exit(1);
    }

    let replace_pattern = args.replace.as_ref().unwrap();

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
        invert_match: false, // Not applicable for replace operations
        ai_enhanced: false, // Disable AI for replace operations for safety
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
        println!("Use --force to execute these changes, or remove --preview to confirm.");
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
        // Interactive confirmation
        println!("Replace operations require confirmation:");
        println!("Pattern: '{}' -> '{}'", args.pattern, replace_pattern);
        println!("Files to modify: {}", operations.iter().map(|op| op.file_path.display().to_string()).collect::<std::collections::HashSet<_>>().len());
        println!("Total operations: {}", operations.len());
        println!();
        println!("First few operations:");
        for operation in operations.iter().take(3) {
            println!("{}:{}: {} -> {}", operation.file_path.display(), operation.line_number,
                    operation.old_content.trim(), operation.new_content.trim());
        }
        println!();
        println!("Execute these changes? (y/N) or use --preview to see all changes:");

        // For now, require explicit confirmation
        eprintln!("Interactive confirmation not implemented. Use --preview to see changes or --force to execute.");
        std::process::exit(1);
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args = Args::parse()?;

    // Check if TUI mode is requested
    if args.tui || (args.pattern.is_empty() && !args.index_build && !args.index_update && !args.index_watch && !args.index_status) {
        if args.pattern.is_empty() && !args.tui {
            // Show helpful error message for missing pattern
            eprintln!("Error: No search pattern specified.");
            eprintln!("Usage: ricegrep [OPTIONS] <pattern> [path...]");
            eprintln!("Try 'ricegrep --help' for more information.");
            std::process::exit(1);
        }

        // Launch interactive TUI
        let ai_processor = Box::new(RiceGrepAIProcessor::new());
        let search_engine = RegexSearchEngine::new().with_ai_processor(ai_processor);
        let mut tui = RiceGrepTUI::new(search_engine);
        tui.run().await?;
        return Ok(());
    }

    // Handle index operations
    if args.index_build || args.index_update || args.index_watch || args.index_status {
        return handle_index_operations(args).await;
    }

    // Handle replace operations
    if args.replace.is_some() {
        return handle_replace_operations(args).await;
    }

    // Pattern validation is handled by clap argument requirements

    // Determine display options (ripgrep compatibility)
    let is_multiple_files = args.paths.len() > 1 ||
        (args.paths.len() == 1 && std::fs::metadata(&args.paths[0]).map(|m| m.is_dir()).unwrap_or(false));

    // Show filename if not disabled and (explicitly requested or searching multiple files/directories)
    let show_filename = !args.no_filename && (args.with_filename || is_multiple_files);

    // Show line numbers only when explicitly requested (ripgrep compatibility)
    let show_line_numbers = args.line_number_flag && !args.no_line_number;

    // Create search query from args
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
        invert_match: args.invert_match,
        ai_enhanced: args.ai_enhanced || args.natural_language,
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

    // Log performance metrics if requested
    if std::env::var("RICEGREP_PERF").is_ok() {
        eprintln!("Search completed in {:.2}ms, found {} matches in {} files",
                 search_duration.as_secs_f64() * 1000.0,
                 results.total_matches,
                 results.files_searched);
    }

    // Create output formatter
    let formatter = OutputFormatter::new(
        args.format,
        args.color,
        show_line_numbers,
        !args.no_heading,
        show_filename,
        args.ai_enhanced || args.natural_language,
        args.count,
    );

    // Output results
    formatter.write_results(&results)?;

    Ok(())
}