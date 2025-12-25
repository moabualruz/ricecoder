//! File Reading Tools
//!
//! This module provides safe file reading capabilities with content filtering,
//! size limits, and binary file detection for enhanced security.

use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::error::ToolError;

/// Input for file read operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileReadInput {
    /// Path to the file to read (accepts both `file_path` and `filePath`)
    #[serde(alias = "filePath")]
    pub file_path: String,
    /// Optional start line (1-indexed, inclusive) - legacy parameter
    pub start_line: Option<usize>,
    /// Optional end line (1-indexed, inclusive) - legacy parameter
    pub end_line: Option<usize>,
    /// Optional line offset (0-based)
    /// Takes precedence over start_line if both specified
    pub offset: Option<usize>,
    /// Optional number of lines to read (default: 2000)
    /// Takes precedence over end_line if both specified
    pub limit: Option<usize>,
    /// Maximum file size in bytes (default: None = unlimited, OpenCode parity)
    pub max_size_bytes: Option<usize>,
    /// Whether to detect and reject binary files (default: true)
    pub detect_binary: Option<bool>,
    /// Content type filter
    pub content_filter: Option<ContentFilter>,
    /// Whether to format output with line numbers (cat -n style)
    /// Default: true for OpenCode parity (00001| format)
    pub line_numbers: Option<bool>,
    /// Maximum characters per line before truncation (default: 2000)
    pub max_line_length: Option<usize>,
    /// Working directory for resolving relative paths
    pub working_dir: Option<String>,
    /// Whether to block .env files (except whitelisted)
    pub block_env_files: Option<bool>,
    /// Whether to return base64 attachments for images/PDFs
    pub return_attachments: Option<bool>,
}

/// Content filtering options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentFilter {
    /// Allow all content
    All,
    /// Only allow text content (reject binary)
    TextOnly,
    /// Allow specific file extensions
    AllowedExtensions(Vec<String>),
    /// Reject specific file extensions
    RejectedExtensions(Vec<String>),
}

/// Output from file read operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileReadOutput {
    /// Whether the read was successful
    pub success: bool,
    /// File content (if successfully read)
    pub content: Option<String>,
    /// File size in bytes
    pub file_size: u64,
    /// Detected MIME type (if available)
    pub mime_type: Option<String>,
    /// Whether file was detected as binary
    pub is_binary: bool,
    /// Number of lines read
    pub lines_read: usize,
    /// Total lines in file (if read completely)
    pub total_lines: Option<usize>,
    /// Error message if failed
    pub error: Option<String>,
    /// Content preview (first 20 lines for metadata, OpenCode parity)
    pub preview: Option<String>,
    /// Base64 attachment for images/PDFs (OpenCode parity)
    pub attachment: Option<FileAttachment>,
}

/// Batch file read input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchFileReadInput {
    /// List of files to read
    pub files: Vec<FileReadInput>,
    /// Whether to continue on individual failures
    pub continue_on_error: bool,
    /// Global max size limit
    pub global_max_size: Option<usize>,
}

/// File attachment for images/PDFs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAttachment {
    /// MIME type
    pub mime: String,
    /// Data URL (data:{mime};base64,{content})
    pub url: String,
}

/// Batch file read output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchFileReadOutput {
    /// Results for each file
    pub results: Vec<FileReadResult>,
    /// Summary statistics
    pub summary: BatchReadSummary,
}

/// Individual file read result for batch operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileReadResult {
    /// Input that was processed
    pub input: FileReadInput,
    /// Output result
    pub output: FileReadOutput,
}

/// Summary for batch read operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchReadSummary {
    /// Total files processed
    pub total_files: usize,
    /// Successfully read files
    pub successful_reads: usize,
    /// Failed reads
    pub failed_reads: usize,
    /// Binary files detected and skipped
    pub binary_files_skipped: usize,
    /// Files exceeding size limits
    pub size_limit_exceeded: usize,
    /// Total bytes read
    pub total_bytes_read: u64,
}

/// File reading tool with safety checks and content filtering
pub struct FileReadTool;

impl FileReadTool {
    /// Default line limit (2000 lines, OpenCode parity)
    const DEFAULT_LINE_LIMIT: usize = 2000;

    /// Default max line length before truncation (2000 chars, OpenCode parity)
    const DEFAULT_MAX_LINE_LENGTH: usize = 2000;

    /// .env file whitelist (OpenCode parity)
    const ENV_FILE_WHITELIST: &'static [&'static str] = &[
        ".env.sample",
        ".env.example",
        ".env.template",
        ".example",
    ];

    /// Read a single file with safety checks
    pub fn read_file(input: &FileReadInput) -> Result<FileReadOutput, ToolError> {
        // Validate input
        Self::validate_input(input)?;

        // Resolve relative paths (Gap 2: Relative path resolution)
        let file_path = Self::resolve_path(&input.file_path, input.working_dir.as_deref())?;
        let path = Path::new(&file_path);

        // Check if file exists (Gap 11: File-not-found suggestions)
        if !path.exists() {
            let error_msg = Self::generate_file_not_found_error(&path)?;
            return Ok(FileReadOutput {
                success: false,
                content: None,
                file_size: 0,
                mime_type: None,
                is_binary: false,
                lines_read: 0,
                total_lines: None,
                error: Some(error_msg),
                preview: None,
                attachment: None,
            });
        }

        // Gap 9: Block .env files with whitelist
        if input.block_env_files.unwrap_or(true) {
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                let is_whitelisted = Self::ENV_FILE_WHITELIST.iter().any(|w| filename.ends_with(w));
                
                // Block .env* files except whitelisted
                if !is_whitelisted && filename.starts_with(".env") && (filename == ".env" || filename.chars().nth(4) == Some('.')) {
                    return Ok(FileReadOutput {
                        success: false,
                        content: None,
                        file_size: 0,
                        mime_type: None,
                        is_binary: false,
                        lines_read: 0,
                        total_lines: None,
                        error: Some(format!(
                            "The user has blocked you from reading {}, DO NOT make further attempts to read it",
                            input.file_path
                        )),
                        preview: None,
                        attachment: None,
                    });
                }
            }
        }

        // Get file metadata
        let metadata = fs::metadata(&file_path).map_err(|e| {
            ToolError::new("IO_ERROR", format!("Failed to read file metadata: {}", e))
        })?;

        let file_size = metadata.len();

        // Gap 12: Check size limits (optional, OpenCode has none)
        if let Some(max_size) = input.max_size_bytes {
            if file_size > max_size as u64 {
                return Ok(FileReadOutput {
                    success: false,
                    content: None,
                    file_size,
                    mime_type: None,
                    is_binary: false,
                    lines_read: 0,
                    total_lines: None,
                    error: Some(format!(
                        "File size {} exceeds limit of {} bytes",
                        file_size, max_size
                    )),
                    preview: None,
                    attachment: None,
                });
            }
        }

        // Check content filter
        if let Some(filter) = &input.content_filter {
            if !Self::passes_content_filter(path, filter)? {
                return Ok(FileReadOutput {
                    success: false,
                    content: None,
                    file_size,
                    mime_type: None,
                    is_binary: false,
                    lines_read: 0,
                    total_lines: None,
                    error: Some("File rejected by content filter".to_string()),
                    preview: None,
                    attachment: None,
                });
            }
        }

        // Gap 3 & 4: Image/PDF base64 attachments, SVG as text
        let mime_type = Self::detect_mime_type(path);
        let is_image = mime_type.as_ref().map(|m| m.starts_with("image/")).unwrap_or(false);
        let is_svg = mime_type.as_ref().map(|m| m == "image/svg+xml").unwrap_or(false);
        let is_pdf = mime_type.as_ref().map(|m| m == "application/pdf").unwrap_or(false);
        
        // Return base64 attachment for images (except SVG) and PDFs
        if input.return_attachments.unwrap_or(true) && (is_image && !is_svg) || is_pdf {
            let content_bytes = fs::read(&file_path)
                .map_err(|e| ToolError::new("IO_ERROR", format!("Failed to read file: {}", e)))?;
            
            let base64_content = base64::encode(&content_bytes);
            let mime = mime_type.unwrap_or_else(|| "application/octet-stream".to_string());
            let msg = if is_image { "Image read successfully" } else { "PDF read successfully" };
            
            return Ok(FileReadOutput {
                success: true,
                content: Some(msg.to_string()),
                file_size,
                mime_type: Some(mime.clone()),
                is_binary: false,
                lines_read: 0,
                total_lines: None,
                error: None,
                preview: Some(msg.to_string()),
                attachment: Some(FileAttachment {
                    mime: mime.clone(),
                    url: format!("data:{};base64,{}", mime, base64_content),
                }),
            });
        }

        // Read file content
        let content_bytes = fs::read(&file_path)
            .map_err(|e| ToolError::new("IO_ERROR", format!("Failed to read file: {}", e)))?;

        // Gap 5: Binary detection with 4KB sample (OpenCode parity)
        let detect_binary = input.detect_binary.unwrap_or(true);
        let sample_size = 4096.min(content_bytes.len());
        let is_binary = detect_binary && Self::is_binary_content(&content_bytes[..sample_size]);

        if is_binary {
            return Ok(FileReadOutput {
                success: false,
                content: None,
                file_size,
                mime_type,
                is_binary: true,
                lines_read: 0,
                total_lines: None,
                error: Some(format!("Cannot read binary file: {}", file_path)),
                preview: None,
                attachment: None,
            });
        }

        // Convert to string
        let content = String::from_utf8(content_bytes).map_err(|e| {
            ToolError::new(
                "ENCODING_ERROR",
                format!("File contains invalid UTF-8: {}", e),
            )
        })?;

        // Determine line range using offset/limit or start_line/end_line (legacy)
        let (start_idx, line_count) = if input.offset.is_some() || input.limit.is_some() {
            // offset is 0-based, limit is count
            let offset = input.offset.unwrap_or(0);
            let limit = input.limit.unwrap_or(Self::DEFAULT_LINE_LIMIT);
            (offset, Some(limit))
        } else if input.start_line.is_some() || input.end_line.is_some() {
            // Legacy: 1-indexed start/end
            let start = input.start_line.map(|l| l.saturating_sub(1)).unwrap_or(0);
            let end = input.end_line;
            let count = end.map(|e| e.saturating_sub(start));
            (start, count)
        } else {
            // Default: start at 0, read up to DEFAULT_LINE_LIMIT lines
            (0, Some(Self::DEFAULT_LINE_LIMIT))
        };

        // Apply line filtering with truncation
        let max_line_len = input.max_line_length.unwrap_or(Self::DEFAULT_MAX_LINE_LENGTH);
        let use_line_numbers = input.line_numbers.unwrap_or(true); // Default true for OpenCode parity

        let (filtered_content, lines_read, total_lines_val) =
            Self::filter_lines_with_options(&content, start_idx, line_count, max_line_len, use_line_numbers)?;

        // Gap 6: Add OpenCode-style footer
        let mut output = String::from("<file>\n");
        output.push_str(&filtered_content);
        
        let last_read_line = start_idx + lines_read;
        let has_more_lines = total_lines_val.map(|t| t > last_read_line).unwrap_or(false);
        
        if has_more_lines {
            output.push_str(&format!("\n\n(File has more lines. Use 'offset' parameter to read beyond line {})", last_read_line));
        } else if let Some(total) = total_lines_val {
            output.push_str(&format!("\n\n(End of file - total {} lines)", total));
        }
        output.push_str("\n</file>");

        // Gap 8: Create preview metadata (first 20 lines)
        let lines: Vec<&str> = content.lines().collect();
        let preview_lines: Vec<&str> = lines.iter().take(20).cloned().collect();
        let preview = Some(preview_lines.join("\n"));

        Ok(FileReadOutput {
            success: true,
            content: Some(output),
            file_size,
            mime_type,
            is_binary: false,
            lines_read,
            total_lines: total_lines_val,
            error: None,
            preview,
            attachment: None,
        })
    }

    /// Read multiple files in batch
    pub fn batch_read_files(input: &BatchFileReadInput) -> Result<BatchFileReadOutput, ToolError> {
        let mut results = Vec::new();
        let mut summary = BatchReadSummary {
            total_files: input.files.len(),
            successful_reads: 0,
            failed_reads: 0,
            binary_files_skipped: 0,
            size_limit_exceeded: 0,
            total_bytes_read: 0,
        };

        for file_input in &input.files {
            let mut processed_input = file_input.clone();

            // Apply global max size if specified
            if let Some(global_max) = input.global_max_size {
                if processed_input.max_size_bytes.is_none() {
                    processed_input.max_size_bytes = Some(global_max);
                }
            }

            let (output, should_continue) = match Self::read_file(&processed_input) {
                Ok(result) => {
                    let success = result.success;
                    let is_binary = result.is_binary;
                    let file_size = result.file_size;
                    let error_msg = result.error.as_ref().map(|s| s.contains("exceeds limit"));

                    if success {
                        summary.successful_reads += 1;
                        summary.total_bytes_read += file_size;
                    } else {
                        summary.failed_reads += 1;
                        if is_binary {
                            summary.binary_files_skipped += 1;
                        }
                        if error_msg.unwrap_or(false) {
                            summary.size_limit_exceeded += 1;
                        }
                    }
                    (result, !input.continue_on_error && !success)
                }
                Err(e) => {
                    summary.failed_reads += 1;
                    let result = FileReadOutput {
                        success: false,
                        content: None,
                        file_size: 0,
                        mime_type: None,
                        is_binary: false,
                        lines_read: 0,
                        total_lines: None,
                        error: Some(e.to_string()),
                        preview: None,
                        attachment: None,
                    };
                    (result, !input.continue_on_error)
                }
            };

            results.push(FileReadResult {
                input: processed_input,
                output,
            });

            // Stop on first error if not continuing
            if should_continue {
                break;
            }
        }

        Ok(BatchFileReadOutput { results, summary })
    }

    /// Validate input parameters
    fn validate_input(input: &FileReadInput) -> Result<(), ToolError> {
        if input.file_path.is_empty() {
            return Err(ToolError::new("INVALID_INPUT", "File path cannot be empty"));
        }

        // Validate line ranges
        if let (Some(start), Some(end)) = (input.start_line, input.end_line) {
            if start > end {
                return Err(ToolError::new(
                    "INVALID_INPUT",
                    "Start line cannot be greater than end line",
                ));
            }
            if start == 0 {
                return Err(ToolError::new(
                    "INVALID_INPUT",
                    "Line numbers are 1-indexed, start cannot be 0",
                ));
            }
        }

        Ok(())
    }

    /// Check if content passes the content filter
    fn passes_content_filter(path: &Path, filter: &ContentFilter) -> Result<bool, ToolError> {
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        match filter {
            ContentFilter::All => Ok(true),
            ContentFilter::TextOnly => {
                // Consider common text file extensions
                let text_extensions = [
                    "txt", "md", "rs", "py", "js", "ts", "json", "yaml", "yml", "toml", "xml",
                    "html", "css",
                ];
                Ok(text_extensions.contains(&extension.as_str()))
            }
            ContentFilter::AllowedExtensions(allowed) => {
                Ok(allowed.iter().any(|ext| ext.to_lowercase() == extension))
            }
            ContentFilter::RejectedExtensions(rejected) => {
                Ok(!rejected.iter().any(|ext| ext.to_lowercase() == extension))
            }
        }
    }

    /// Resolve relative paths to absolute (Gap 2)
    fn resolve_path(file_path: &str, working_dir: Option<&str>) -> Result<String, ToolError> {
        let path = Path::new(file_path);
        
        if path.is_absolute() {
            return Ok(file_path.to_string());
        }
        
        // Resolve relative to working directory or current directory
        let base_dir = working_dir
            .map(|d| Path::new(d).to_path_buf())
            .or_else(|| std::env::current_dir().ok())
            .ok_or_else(|| ToolError::new("PATH_ERROR", "Cannot determine working directory"))?;
        
        let absolute_path = base_dir.join(path);
        absolute_path
            .to_str()
            .map(|s| s.to_string())
            .ok_or_else(|| ToolError::new("PATH_ERROR", "Invalid path encoding"))
    }

    /// Generate file-not-found error with suggestions (Gap 11)
    fn generate_file_not_found_error(path: &Path) -> Result<String, ToolError> {
        let dir = path.parent().ok_or_else(|| {
            ToolError::new("PATH_ERROR", "Invalid file path")
        })?;
        
        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| ToolError::new("PATH_ERROR", "Invalid filename"))?;
        
        // Try to find similar files in directory
        if dir.exists() {
            let entries = fs::read_dir(dir)
                .map_err(|e| ToolError::new("IO_ERROR", format!("Cannot read directory: {}", e)))?;
            
            let mut suggestions = Vec::new();
            for entry in entries {
                if let Ok(entry) = entry {
                    if let Some(entry_name) = entry.file_name().to_str() {
                        let lower_entry = entry_name.to_lowercase();
                        let lower_filename = filename.to_lowercase();
                        
                        // Simple fuzzy match: contains or is contained
                        if lower_entry.contains(&lower_filename) || lower_filename.contains(&lower_entry) {
                            suggestions.push(entry.path());
                        }
                    }
                }
            }
            
            // Return top 3 suggestions
            suggestions.truncate(3);
            
            if !suggestions.is_empty() {
                let suggestion_list: Vec<String> = suggestions
                    .iter()
                    .filter_map(|p| p.to_str().map(|s| s.to_string()))
                    .collect();
                
                return Ok(format!(
                    "File not found: {}\n\nDid you mean one of these?\n{}",
                    path.display(),
                    suggestion_list.join("\n")
                ));
            }
        }
        
        Ok(format!("File not found: {}", path.display()))
    }

    /// Detect if content is binary by checking for null bytes and non-printable characters
    /// Gap 5: Uses 4KB sample like OpenCode
    fn is_binary_content(bytes: &[u8]) -> bool {
        // Check for null bytes (common in binary files)
        if bytes.contains(&0) {
            return true;
        }

        // Check ratio of non-printable characters
        let mut non_printable = 0;
        for &b in bytes {
            // Allow common whitespace and control characters
            if b < 9 || (b > 13 && b < 32) {
                non_printable += 1;
            }
        }

        let ratio = non_printable as f64 / bytes.len() as f64;
        ratio > 0.3 // More than 30% non-printable characters
    }

    /// Detect MIME type based on file extension (Gap 4: SVG as text)
    fn detect_mime_type(path: &Path) -> Option<String> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| match ext.to_lowercase().as_str() {
                "txt" => Some("text/plain".to_string()),
                "md" => Some("text/markdown".to_string()),
                "rs" => Some("text/rust".to_string()),
                "py" => Some("text/python".to_string()),
                "js" => Some("application/javascript".to_string()),
                "ts" => Some("application/typescript".to_string()),
                "json" => Some("application/json".to_string()),
                "yaml" | "yml" => Some("application/yaml".to_string()),
                "toml" => Some("application/toml".to_string()),
                "xml" => Some("application/xml".to_string()),
                "html" => Some("text/html".to_string()),
                "css" => Some("text/css".to_string()),
                "svg" => Some("image/svg+xml".to_string()),
                "png" => Some("image/png".to_string()),
                "jpg" | "jpeg" => Some("image/jpeg".to_string()),
                "gif" => Some("image/gif".to_string()),
                "webp" => Some("image/webp".to_string()),
                "pdf" => Some("application/pdf".to_string()),
                _ => None,
            })
    }

    /// Filter content to specific line range (legacy method for backward compatibility)
    fn filter_lines(
        content: &str,
        start_line: Option<usize>,
        end_line: Option<usize>,
    ) -> Result<(String, usize, Option<usize>), ToolError> {
        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();

        let start_idx = start_line.map(|l| l.saturating_sub(1)).unwrap_or(0);
        let end_idx = end_line
            .map(|l| l.saturating_sub(1))
            .unwrap_or(total_lines.saturating_sub(1));

        if start_idx >= total_lines {
            return Ok((String::new(), 0, Some(total_lines)));
        }

        let end_idx = end_idx.min(total_lines.saturating_sub(1));
        let filtered_lines: Vec<&str> = lines[start_idx..=end_idx].iter().cloned().collect();
        let filtered_content = filtered_lines.join("\n");

        Ok((filtered_content, filtered_lines.len(), Some(total_lines)))
    }

    /// Filter content with pagination options
    ///
    /// # Arguments
    /// * `content` - File content to filter
    /// * `offset` - 0-based line offset to start from
    /// * `limit` - Optional number of lines to read
    /// * `max_line_length` - Maximum characters per line before truncation
    /// * `line_numbers` - Whether to format with cat -n style line numbers
    ///
    /// # Returns
    /// Tuple of (formatted_content, lines_read, total_lines)
    fn filter_lines_with_options(
        content: &str,
        offset: usize,
        limit: Option<usize>,
        max_line_length: usize,
        line_numbers: bool,
    ) -> Result<(String, usize, Option<usize>), ToolError> {
        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();

        // Handle empty file or offset beyond file
        if offset >= total_lines {
            return Ok((String::new(), 0, Some(total_lines)));
        }

        // Calculate end index based on limit
        let end_idx = if let Some(lim) = limit {
            (offset + lim).min(total_lines)
        } else {
            total_lines
        };

        let mut output_lines = Vec::new();

        for (idx, line) in lines.iter().enumerate().skip(offset).take(end_idx - offset) {
            let line_num = idx + 1; // 1-indexed line number

            // Gap 7: Truncate line if too long (use "..." marker for OpenCode parity)
            let truncated_line = if line.len() > max_line_length {
                format!("{}...", &line[..max_line_length])
            } else {
                line.to_string()
            };

            // Format with line numbers if requested (OpenCode style: 00001| format)
            let formatted_line = if line_numbers {
                format!("{:05}| {}", line_num, truncated_line)
            } else {
                truncated_line
            };

            output_lines.push(formatted_line);
        }

        let lines_read = output_lines.len();
        let filtered_content = output_lines.join("\n");

        Ok((filtered_content, lines_read, Some(total_lines)))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_read_text_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let content = "Hello\nWorld\nTest";
        fs::write(&file_path, content).unwrap();

        let input = FileReadInput {
            file_path: file_path.to_string_lossy().to_string(),
            start_line: None,
            end_line: None,
            offset: None,
            limit: None,
            max_size_bytes: None,
            detect_binary: None,
            content_filter: None,
            line_numbers: Some(false), // Disable for simple test
            max_line_length: None,
            working_dir: None,
            block_env_files: Some(false),
            return_attachments: Some(false),
        };

        let result = FileReadTool::read_file(&input).unwrap();
        assert!(result.success);
        assert!(result.content.is_some());
        assert_eq!(result.lines_read, 3);
        assert_eq!(result.total_lines, Some(3));
        assert!(!result.is_binary);
    }

    #[test]
    fn test_read_file_with_line_range() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let content = "Line 1\nLine 2\nLine 3\nLine 4";
        fs::write(&file_path, content).unwrap();

        let input = FileReadInput {
            file_path: file_path.to_string_lossy().to_string(),
            start_line: Some(2),
            end_line: Some(3),
            offset: None,
            limit: None,
            max_size_bytes: None,
            detect_binary: None,
            content_filter: None,
            line_numbers: Some(false),
            max_line_length: None,
            working_dir: None,
            block_env_files: Some(false),
            return_attachments: Some(false),
        };

        let result = FileReadTool::read_file(&input).unwrap();
        assert!(result.success);
        assert!(result.content.as_ref().unwrap().contains("Line 2"));
        assert!(result.content.as_ref().unwrap().contains("Line 3"));
        assert_eq!(result.lines_read, 2);
        assert_eq!(result.total_lines, Some(4));
    }

    #[test]
    fn test_read_nonexistent_file() {
        let input = FileReadInput {
            file_path: "/nonexistent/file.txt".to_string(),
            start_line: None,
            end_line: None,
            offset: None,
            limit: None,
            max_size_bytes: None,
            detect_binary: None,
            content_filter: None,
            line_numbers: None,
            max_line_length: None,
            working_dir: None,
            block_env_files: Some(false),
            return_attachments: Some(false),
        };

        let result = FileReadTool::read_file(&input).unwrap();
        assert!(!result.success);
        assert!(result.error.is_some());
        assert!(result.error.as_ref().unwrap().contains("not found"));
    }

    #[test]
    fn test_binary_file_detection() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("binary.dat");
        // Create binary content with null bytes
        let content = vec![0, 1, 2, 3, 0, 5, 6, 7];
        fs::write(&file_path, content).unwrap();

        let input = FileReadInput {
            file_path: file_path.to_string_lossy().to_string(),
            start_line: None,
            end_line: None,
            offset: None,
            limit: None,
            max_size_bytes: None,
            detect_binary: Some(true),
            content_filter: None,
            line_numbers: None,
            max_line_length: None,
            working_dir: None,
            block_env_files: Some(false),
            return_attachments: Some(false),
        };

        let result = FileReadTool::read_file(&input).unwrap();
        assert!(!result.success);
        assert!(result.is_binary);
        assert!(result
            .error
            .as_ref()
            .unwrap()
            .contains("binary file"));
    }

    #[test]
    fn test_size_limit() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("large.txt");
        let content = "x".repeat(2000); // 2000 bytes
        fs::write(&file_path, content).unwrap();

        let input = FileReadInput {
            file_path: file_path.to_string_lossy().to_string(),
            start_line: None,
            end_line: None,
            offset: None,
            limit: None,
            max_size_bytes: Some(1000), // 1000 byte limit
            detect_binary: None,
            content_filter: None,
            line_numbers: None,
            max_line_length: None,
            working_dir: None,
            block_env_files: Some(false),
            return_attachments: Some(false),
        };

        let result = FileReadTool::read_file(&input).unwrap();
        assert!(!result.success);
        assert!(result.error.as_ref().unwrap().contains("exceeds limit"));
    }

    #[test]
    fn test_content_filter_allowed_extensions() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        let content = "fn main() {}";
        fs::write(&file_path, content).unwrap();

        let input = FileReadInput {
            file_path: file_path.to_string_lossy().to_string(),
            start_line: None,
            end_line: None,
            offset: None,
            limit: None,
            max_size_bytes: None,
            detect_binary: None,
            content_filter: Some(ContentFilter::AllowedExtensions(vec!["rs".to_string()])),
            line_numbers: None,
            max_line_length: None,
            working_dir: None,
            block_env_files: Some(false),
            return_attachments: Some(false),
        };

        let result = FileReadTool::read_file(&input).unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_content_filter_rejected_extensions() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.exe");
        let content = "binary content";
        fs::write(&file_path, content).unwrap();

        let input = FileReadInput {
            file_path: file_path.to_string_lossy().to_string(),
            start_line: None,
            end_line: None,
            offset: None,
            limit: None,
            max_size_bytes: None,
            detect_binary: None,
            content_filter: Some(ContentFilter::RejectedExtensions(vec!["exe".to_string()])),
            line_numbers: None,
            max_line_length: None,
            working_dir: None,
            block_env_files: Some(false),
            return_attachments: Some(false),
        };

        let result = FileReadTool::read_file(&input).unwrap();
        assert!(!result.success);
        assert!(result
            .error
            .as_ref()
            .unwrap()
            .contains("rejected by content filter"));
    }

    #[test]
    fn test_env_file_blocking() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join(".env");
        fs::write(&file_path, "SECRET=value").unwrap();

        let input = FileReadInput {
            file_path: file_path.to_string_lossy().to_string(),
            start_line: None,
            end_line: None,
            offset: None,
            limit: None,
            max_size_bytes: None,
            detect_binary: None,
            content_filter: None,
            line_numbers: None,
            max_line_length: None,
            working_dir: None,
            block_env_files: Some(true),
            return_attachments: Some(false),
        };

        let result = FileReadTool::read_file(&input).unwrap();
        assert!(!result.success);
        assert!(result.error.as_ref().unwrap().contains("blocked"));
    }

    #[test]
    fn test_env_file_whitelist() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join(".env.example");
        fs::write(&file_path, "EXAMPLE=value").unwrap();

        let input = FileReadInput {
            file_path: file_path.to_string_lossy().to_string(),
            start_line: None,
            end_line: None,
            offset: None,
            limit: None,
            max_size_bytes: None,
            detect_binary: None,
            content_filter: None,
            line_numbers: Some(false),
            max_line_length: None,
            working_dir: None,
            block_env_files: Some(true),
            return_attachments: Some(false),
        };

        let result = FileReadTool::read_file(&input).unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_opencode_line_number_format() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "Line 1\nLine 2").unwrap();

        let input = FileReadInput {
            file_path: file_path.to_string_lossy().to_string(),
            start_line: None,
            end_line: None,
            offset: None,
            limit: None,
            max_size_bytes: None,
            detect_binary: None,
            content_filter: None,
            line_numbers: Some(true), // OpenCode format
            max_line_length: None,
            working_dir: None,
            block_env_files: Some(false),
            return_attachments: Some(false),
        };

        let result = FileReadTool::read_file(&input).unwrap();
        assert!(result.success);
        let content = result.content.unwrap();
        assert!(content.contains("00001| Line 1"));
        assert!(content.contains("00002| Line 2"));
        assert!(content.contains("<file>"));
        assert!(content.contains("</file>"));
    }

    #[test]
    fn test_relative_path_resolution() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "content").unwrap();

        let input = FileReadInput {
            file_path: "test.txt".to_string(), // Relative path
            start_line: None,
            end_line: None,
            offset: None,
            limit: None,
            max_size_bytes: None,
            detect_binary: None,
            content_filter: None,
            line_numbers: Some(false),
            max_line_length: None,
            working_dir: Some(temp_dir.path().to_string_lossy().to_string()),
            block_env_files: Some(false),
            return_attachments: Some(false),
        };

        let result = FileReadTool::read_file(&input).unwrap();
        assert!(result.success);
    }
}
