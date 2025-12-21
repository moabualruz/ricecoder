//! File Reading Tools
//!
//! This module provides safe file reading capabilities with content filtering,
//! size limits, and binary file detection for enhanced security.

use crate::error::ToolError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Input for file read operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileReadInput {
    /// Path to the file to read
    pub file_path: String,
    /// Optional start line (1-indexed, inclusive)
    pub start_line: Option<usize>,
    /// Optional end line (1-indexed, inclusive)
    pub end_line: Option<usize>,
    /// Maximum file size in bytes (default: 1MB)
    pub max_size_bytes: Option<usize>,
    /// Whether to detect and reject binary files
    pub detect_binary: Option<bool>,
    /// Content type filter
    pub content_filter: Option<ContentFilter>,
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
    /// Content preview (first 500 chars)
    pub preview: Option<String>,
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
    /// Default maximum file size (1MB)
    const DEFAULT_MAX_SIZE: usize = 1024 * 1024;

    /// Read a single file with safety checks
    pub fn read_file(input: &FileReadInput) -> Result<FileReadOutput, ToolError> {
        // Validate input
        Self::validate_input(input)?;

        let path = Path::new(&input.file_path);

        // Check if file exists
        if !path.exists() {
            return Ok(FileReadOutput {
                success: false,
                content: None,
                file_size: 0,
                mime_type: None,
                is_binary: false,
                lines_read: 0,
                total_lines: None,
                error: Some(format!("File does not exist: {}", input.file_path)),
                preview: None,
            });
        }

        // Get file metadata
        let metadata = fs::metadata(&input.file_path).map_err(|e| {
            ToolError::new("IO_ERROR", format!("Failed to read file metadata: {}", e))
        })?;

        let file_size = metadata.len();

        // Check size limits
        let max_size = input.max_size_bytes.unwrap_or(Self::DEFAULT_MAX_SIZE);
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
            });
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
                });
            }
        }

        // Read file content
        let content_bytes = fs::read(&input.file_path)
            .map_err(|e| ToolError::new("IO_ERROR", format!("Failed to read file: {}", e)))?;

        // Detect if binary
        let detect_binary = input.detect_binary.unwrap_or(true);
        let is_binary = detect_binary && Self::is_binary_content(&content_bytes);

        if is_binary {
            return Ok(FileReadOutput {
                success: false,
                content: None,
                file_size,
                mime_type: Self::detect_mime_type(path),
                is_binary: true,
                lines_read: 0,
                total_lines: None,
                error: Some("Binary file detected".to_string()),
                preview: None,
            });
        }

        // Convert to string
        let content = String::from_utf8(content_bytes).map_err(|e| {
            ToolError::new(
                "ENCODING_ERROR",
                format!("File contains invalid UTF-8: {}", e),
            )
        })?;

        // Apply line filtering if specified
        let (filtered_content, lines_read, total_lines) =
            if input.start_line.is_some() || input.end_line.is_some() {
                Self::filter_lines(&content, input.start_line, input.end_line)?
            } else {
                (
                    content.clone(),
                    content.lines().count(),
                    Some(content.lines().count()),
                )
            };

        // Create preview
        let preview = if filtered_content.len() > 500 {
            Some(filtered_content.chars().take(500).collect::<String>() + "...")
        } else {
            Some(filtered_content.clone())
        };

        Ok(FileReadOutput {
            success: true,
            content: Some(filtered_content),
            file_size,
            mime_type: Self::detect_mime_type(path),
            is_binary: false,
            lines_read,
            total_lines,
            error: None,
            preview,
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

    /// Detect if content is binary by checking for null bytes and non-printable characters
    fn is_binary_content(bytes: &[u8]) -> bool {
        // Check for null bytes (common in binary files)
        if bytes.contains(&0) {
            return true;
        }

        // Check ratio of non-printable characters
        let non_printable = bytes
            .iter()
            .filter(|&&b| {
                // Allow common whitespace and control characters
                !b.is_ascii_graphic()
                    && !b.is_ascii_whitespace()
                    && b != b'\t'
                    && b != b'\n'
                    && b != b'\r'
            })
            .count();

        let ratio = non_printable as f64 / bytes.len() as f64;
        ratio > 0.3 // More than 30% non-printable characters
    }

    /// Detect MIME type based on file extension
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
                _ => None,
            })
    }

    /// Filter content to specific line range
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

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
            max_size_bytes: None,
            detect_binary: None,
            content_filter: None,
        };

        let result = FileReadTool::read_file(&input).unwrap();
        assert!(result.success);
        assert_eq!(result.content.as_ref().unwrap(), content);
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
            max_size_bytes: None,
            detect_binary: None,
            content_filter: None,
        };

        let result = FileReadTool::read_file(&input).unwrap();
        assert!(result.success);
        assert_eq!(result.content.as_ref().unwrap(), "Line 2\nLine 3");
        assert_eq!(result.lines_read, 2);
        assert_eq!(result.total_lines, Some(4));
    }

    #[test]
    fn test_read_nonexistent_file() {
        let input = FileReadInput {
            file_path: "/nonexistent/file.txt".to_string(),
            start_line: None,
            end_line: None,
            max_size_bytes: None,
            detect_binary: None,
            content_filter: None,
        };

        let result = FileReadTool::read_file(&input).unwrap();
        assert!(!result.success);
        assert!(result.error.is_some());
        assert!(result.error.as_ref().unwrap().contains("does not exist"));
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
            max_size_bytes: None,
            detect_binary: Some(true),
            content_filter: None,
        };

        let result = FileReadTool::read_file(&input).unwrap();
        assert!(!result.success);
        assert!(result.is_binary);
        assert!(result
            .error
            .as_ref()
            .unwrap()
            .contains("Binary file detected"));
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
            max_size_bytes: Some(1000), // 1000 byte limit
            detect_binary: None,
            content_filter: None,
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
            max_size_bytes: None,
            detect_binary: None,
            content_filter: Some(ContentFilter::AllowedExtensions(vec!["rs".to_string()])),
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
            max_size_bytes: None,
            detect_binary: None,
            content_filter: Some(ContentFilter::RejectedExtensions(vec!["exe".to_string()])),
        };

        let result = FileReadTool::read_file(&input).unwrap();
        assert!(!result.success);
        assert!(result
            .error
            .as_ref()
            .unwrap()
            .contains("rejected by content filter"));
    }
}
