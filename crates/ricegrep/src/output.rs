//! Output formatting for RiceGrep results
//!
//! This module handles formatting and displaying search results in various
//! formats (text, JSON) with optional colorization.

use crate::config::{OutputFormat, ColorChoice};
use crate::search::{SearchResults, SearchMatch};
use std::io::{self, Write};
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

/// Output formatter for search results
pub struct OutputFormatter {
    format: OutputFormat,
    color: ColorChoice,
    line_numbers: bool,
    heading: bool,
    filename: bool,
    ai_enabled: bool,
    count: bool,
    content: bool,
    max_lines: Option<usize>,
    syntax_highlight: bool,
}

impl OutputFormatter {
    /// Create a new output formatter
    pub fn new(format: OutputFormat, color: ColorChoice, line_numbers: bool, heading: bool, filename: bool, ai_enabled: bool, count: bool, content: bool, max_lines: Option<usize>) -> Self {
        Self {
            format,
            color,
            line_numbers,
            heading,
            filename,
            ai_enabled,
            count,
            content,
            max_lines,
            syntax_highlight: false, // Default to no syntax highlighting
        }
    }

    /// Create a new output formatter with syntax highlighting
    pub fn with_syntax_highlight(format: OutputFormat, color: ColorChoice, line_numbers: bool, heading: bool, filename: bool, ai_enabled: bool, count: bool, content: bool, max_lines: Option<usize>, syntax_highlight: bool) -> Self {
        Self {
            format,
            color,
            line_numbers,
            heading,
            filename,
            ai_enabled,
            count,
            content,
            max_lines,
            syntax_highlight,
        }
    }

    /// Format and write search results to stdout
    pub fn write_results(&self, results: &SearchResults) -> Result<(), RiceGrepError> {
        match self.format {
            OutputFormat::Json => self.write_json(results),
            OutputFormat::Text => self.write_text(results),
        }
    }

    /// Write results in JSON format
    fn write_json(&self, results: &SearchResults) -> Result<(), RiceGrepError> {
        if self.content {
            return self.write_json_content_mode(results);
        }

        let stdout = io::stdout();
        let mut handle = stdout.lock();

        // Write each match as a JSON object
        for (i, match_result) in results.matches.iter().enumerate() {
            if i > 0 {
                writeln!(handle)?;
            }

            let json_match = serde_json::json!({
                "type": "match",
                "data": {
                    "path": {
                        "text": match_result.file.display().to_string()
                    },
                    "lines": {
                        "text": match_result.line_content
                    },
                    "line_number": match_result.line_number,
                    "absolute_offset": match_result.byte_offset,
                    "submatches": []
                }
            });

            writeln!(handle, "{}", serde_json::to_string(&json_match)?)?;
        }

        // Write summary
        let summary = serde_json::json!({
            "type": "summary",
            "data": {
                "elapsed_total": {
                    "human": format!("{:.2}s", results.search_time.as_secs_f64()),
                    "seconds": results.search_time.as_secs_f64()
                },
                "stats": {
                    "matches": results.total_matches,
                    "searches": 1,
                    "searches_with_match": if results.total_matches > 0 { 1 } else { 0 },
                    "bytes_searched": 0, // TODO: implement
                    "bytes_printed": 0,  // TODO: implement
                    "files_searched": results.files_searched,
                    "files_with_match": results.matches.iter().map(|m| &m.file).collect::<std::collections::HashSet<_>>().len()
                }
            }
        });

        writeln!(handle, "{}", serde_json::to_string(&summary)?)?;

        Ok(())
    }

    /// Write results in text format
    fn write_text(&self, results: &SearchResults) -> Result<(), RiceGrepError> {
        let stdout = io::stdout();
        let mut handle = stdout.lock();

        if self.count {
            // Count mode: show count per file
            for (file_path, count) in &results.file_counts {
                writeln!(handle, "{}:{}", file_path.display(), count)?;
            }
            return Ok(());
        }

        if results.matches.is_empty() {
            // Only show "No matches found" when AI is enabled, otherwise silent like ripgrep
            if self.ai_enabled {
                writeln!(handle, "No matches found.")?;
            }
            return Ok(());
        }

        // Content display mode: show full file contents
        if self.content {
            return self.write_content_mode(results);
        }

        // Print summary with AI usage indicators
        if self.ai_enabled || results.ai_reranked {
            writeln!(handle, "Found {} matches in {} files (searched in {:.2}ms)",
                     results.total_matches,
                     results.files_searched,
                     results.search_time.as_secs_f64() * 1000.0)?;

            if results.ai_reranked {
                writeln!(handle, "Results enhanced with AI reranking")?;
            } else {
                writeln!(handle, "Results ranked using deterministic algorithms")?;
            }

            if results.degradation_mode {
                writeln!(handle, "Note: Operating in degradation mode - some features unavailable")?;
            }

            writeln!(handle)?;
        }

        let mut current_file = None;

        for match_result in &results.matches {
            // Handle filename display based on heading mode
            if self.heading {
                // Grouped heading mode: show filename above matches
                if current_file.as_ref() != Some(&match_result.file) {
                    if current_file.is_some() {
                        writeln!(handle)?;
                    }
                    current_file = Some(match_result.file.clone());
                    if self.filename {
                        writeln!(handle, "{}", match_result.file.display())?;
                    }
                }

                // Format the match line with indentation
                if self.line_numbers {
                    write!(handle, "  {}:", match_result.line_number)?;
                } else {
                    write!(handle, "  ")?;
                }
            } else {
                // Inline mode: show filename:line: content
                if self.filename {
                    write!(handle, "{}:", match_result.file.display())?;
                }
                if self.line_numbers {
                    write!(handle, "{}:", match_result.line_number)?;
                }
            }

            // Print the line content
            write!(handle, "{}", match_result.line_content)?;

            // Print AI score and context only when AI is enabled
            if self.ai_enabled {
                // Print AI score if available
                if let Some(score) = match_result.ai_score {
                    let score_indicator = if score >= 0.8 {
                        "🟢" // High confidence
                    } else if score >= 0.6 {
                        "🟡" // Medium confidence
                    } else {
                        "🔴" // Low confidence
                    };
                    write!(handle, " {}{:.2}", score_indicator, score)?;
                }

                // Print AI context if available
                if let Some(context) = &match_result.ai_context {
                    write!(handle, " // {}", context)?;
                }
            }

            writeln!(handle)?;
        }

        Ok(())
    }

    /// Write results in JSON content mode
    fn write_json_content_mode(&self, results: &SearchResults) -> Result<(), RiceGrepError> {
        use std::fs;
        use std::collections::HashSet;

        let stdout = io::stdout();
        let mut handle = stdout.lock();

        // Get unique files that have matches
        let files_with_matches: HashSet<_> = results.matches.iter()
            .map(|m| &m.file)
            .collect();

        for file_path in files_with_matches {
            match fs::read_to_string(file_path) {
                Ok(content) => {
                    let json_file = serde_json::json!({
                        "type": "file",
                        "path": file_path,
                        "content": content
                    });
                    writeln!(handle, "{}", serde_json::to_string(&json_file)?)?;
                }
                Err(e) => {
                    let json_error = serde_json::json!({
                        "type": "error",
                        "path": file_path,
                        "error": e.to_string()
                    });
                    writeln!(handle, "{}", serde_json::to_string(&json_error)?)?;
                }
            }
        }

        Ok(())
    }

    /// Write results in content mode (full file display)
    fn write_content_mode(&self, results: &SearchResults) -> Result<(), RiceGrepError> {
        use std::fs;
        use std::collections::HashSet;

        let stdout = io::stdout();
        let mut handle = stdout.lock();

        // Get unique files that have matches
        let files_with_matches: HashSet<_> = results.matches.iter()
            .map(|m| &m.file)
            .collect();

        for file_path in files_with_matches {
            // Show filename header
            if self.filename {
                writeln!(handle, "{}", file_path.display())?;
                if self.heading {
                    writeln!(handle, "{}", "=".repeat(file_path.display().to_string().len()))?;
                }
            }

            // Read and display full file content
            match fs::read_to_string(file_path) {
                Ok(content) => {
                    if self.syntax_highlight && self.color != ColorChoice::Never {
                        self.write_syntax_highlighted_content(&mut handle, &content, file_path)?;
                    } else {
                        self.write_plain_content(&mut handle, &content)?;
                    }

                    // Indicate if content was truncated
                    if let Some(max_lines) = self.max_lines {
                        let total_lines = content.lines().count();
                        if total_lines > max_lines {
                            writeln!(handle, "... (content truncated to {} lines)", max_lines)?;
                        }
                    }

                    writeln!(handle)?; // Add blank line between files
                }
                Err(e) => {
                    writeln!(handle, "Error reading file {}: {}", file_path.display(), e)?;
                }
            }
        }

        Ok(())
    }

    /// Write content with syntax highlighting
    fn write_syntax_highlighted_content(&self, handle: &mut impl Write, content: &str, file_path: &std::path::Path) -> Result<(), RiceGrepError> {
        // Initialize syntax highlighting
        let ps = SyntaxSet::load_defaults_newlines();
        let ts = ThemeSet::load_defaults();

        // Try to detect syntax based on file extension
        let syntax = file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| ps.find_syntax_by_extension(ext))
            .unwrap_or_else(|| ps.find_syntax_plain_text());

        let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);

        let lines: Vec<&str> = content.lines().collect();

        // Apply max_lines limit if specified
        let lines_to_show = if let Some(max_lines) = self.max_lines {
            &lines[..lines.len().min(max_lines)]
        } else {
            &lines[..]
        };

        // Add line numbers if requested
        if self.line_numbers {
            for (i, line) in lines_to_show.iter().enumerate() {
                let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ps).map_err(|_| {
                    RiceGrepError::Config { message: "Syntax highlighting error".to_string() }
                })?;
                let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
                writeln!(handle, "{:6}: {}", i + 1, escaped)?;
            }
        } else {
            for line in lines_to_show {
                let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ps).map_err(|_| {
                    RiceGrepError::Config { message: "Syntax highlighting error".to_string() }
                })?;
                let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
                writeln!(handle, "{}", escaped)?;
            }
        }

        Ok(())
    }

    /// Write plain content without syntax highlighting
    fn write_plain_content(&self, handle: &mut impl Write, content: &str) -> Result<(), RiceGrepError> {
        let lines: Vec<&str> = content.lines().collect();

        // Apply max_lines limit if specified
        let lines_to_show = if let Some(max_lines) = self.max_lines {
            &lines[..lines.len().min(max_lines)]
        } else {
            &lines[..]
        };

        // Add line numbers if requested
        if self.line_numbers {
            for (i, line) in lines_to_show.iter().enumerate() {
                writeln!(handle, "{:6}: {}", i + 1, *line)?;
            }
        } else {
            for line in lines_to_show {
                writeln!(handle, "{}", line)?;
            }
        }

        Ok(())
    }
}

use crate::error::RiceGrepError;