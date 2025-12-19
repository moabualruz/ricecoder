//! Output formatting for RiceGrep results
//!
//! This module handles formatting and displaying search results in various
//! formats (text, JSON) with optional colorization.

use crate::config::{OutputFormat, ColorChoice};
use crate::search::{SearchResults, SearchMatch};
use std::io::{self, Write};

/// Output formatter for search results
pub struct OutputFormatter {
    format: OutputFormat,
    color: ColorChoice,
    line_numbers: bool,
    heading: bool,
    filename: bool,
    ai_enabled: bool,
    count: bool,
}

impl OutputFormatter {
    /// Create a new output formatter
    pub fn new(format: OutputFormat, color: ColorChoice, line_numbers: bool, heading: bool, filename: bool, ai_enabled: bool, count: bool) -> Self {
        Self {
            format,
            color,
            line_numbers,
            heading,
            filename,
            ai_enabled,
            count,
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

        // Print summary only when AI is enabled
        if self.ai_enabled {
            writeln!(handle, "Found {} matches in {} files (searched in {:.2}ms)",
                     results.total_matches,
                     results.files_searched,
                     results.search_time.as_secs_f64() * 1000.0)?;

            if results.ai_reranked {
                writeln!(handle, "Results enhanced with AI reranking")?;
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
}

use crate::error::RiceGrepError;