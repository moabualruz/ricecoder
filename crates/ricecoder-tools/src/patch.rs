//! Patch tool for applying unified diff patches
//!
//! Provides functionality to parse and apply unified diff patches with conflict detection.

use crate::error::ToolError;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Input for patch operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchInput {
    /// Path to the file to patch
    pub file_path: String,
    /// Unified diff patch content
    pub patch_content: String,
}

/// Output from patch operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchOutput {
    /// Whether the patch was successfully applied
    pub success: bool,
    /// Number of hunks successfully applied
    pub applied_hunks: usize,
    /// Number of hunks that failed to apply
    pub failed_hunks: usize,
    /// Details about failed hunks
    pub failed_hunk_details: Vec<FailedHunkInfo>,
}

/// Information about a failed hunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedHunkInfo {
    /// Hunk number (1-indexed)
    pub hunk_number: usize,
    /// Starting line number in the original file
    pub line_number: usize,
    /// Error message
    pub error: String,
    /// Context lines around the failure
    pub context: Option<String>,
}

/// Represents a single hunk in a unified diff
#[derive(Debug, Clone)]
struct Hunk {
    /// Original file starting line number
    pub orig_start: usize,
    /// Lines in the hunk (with +/- prefix)
    pub lines: Vec<String>,
}

/// Patch tool for applying unified diff patches
pub struct PatchTool;

impl PatchTool {
    /// Create a new patch tool
    pub fn new() -> Self {
        Self
    }

    /// Parse a unified diff patch
    fn parse_patch(patch_content: &str) -> Result<Vec<Hunk>, ToolError> {
        let mut hunks = Vec::new();
        let mut lines = patch_content.lines().peekable();

        // Skip file headers (--- and +++ lines)
        while let Some(line) = lines.peek() {
            if line.starts_with("---") || line.starts_with("+++") {
                lines.next();
            } else if line.starts_with("@@") {
                break;
            } else if line.is_empty() || line.starts_with("diff ") || line.starts_with("index ") {
                lines.next();
            } else {
                lines.next();
            }
        }

        // Parse hunks
        while let Some(line) = lines.next() {
            if !line.starts_with("@@") {
                continue;
            }

            // Parse hunk header: @@ -orig_start,orig_count +new_start,new_count @@
            let hunk_header = line.trim_start_matches("@@").trim_end_matches("@@").trim();

            let parts: Vec<&str> = hunk_header.split_whitespace().collect();
            if parts.len() < 2 {
                return Err(
                    ToolError::new("INVALID_PATCH", "Invalid hunk header format")
                        .with_details(format!("Hunk header: {}", line))
                        .with_suggestion("Ensure patch is in unified diff format"),
                );
            }

            let (orig_start, _orig_count) = Self::parse_range(parts[0])?;
            let (_new_start, _new_count) = Self::parse_range(parts[1])?;

            let mut hunk_lines = Vec::new();

            // Read hunk lines
            while let Some(hunk_line) = lines.peek() {
                if hunk_line.starts_with("@@") {
                    break;
                }
                if hunk_line.starts_with("\\") {
                    // Skip "\ No newline at end of file" markers
                    lines.next();
                    continue;
                }
                if hunk_line.is_empty()
                    || hunk_line.starts_with("-")
                    || hunk_line.starts_with("+")
                    || hunk_line.starts_with(" ")
                {
                    hunk_lines.push(lines.next().unwrap().to_string());
                } else {
                    break;
                }
            }

            hunks.push(Hunk {
                orig_start,
                lines: hunk_lines,
            });
        }

        if hunks.is_empty() {
            return Err(ToolError::new("INVALID_PATCH", "No hunks found in patch")
                .with_suggestion("Ensure patch contains at least one hunk"));
        }

        Ok(hunks)
    }

    /// Parse a range specification (e.g., "-10,5" or "+20,3")
    fn parse_range(range_spec: &str) -> Result<(usize, usize), ToolError> {
        let range_spec = range_spec.trim_start_matches('-').trim_start_matches('+');
        let parts: Vec<&str> = range_spec.split(',').collect();

        match parts.len() {
            1 => {
                let start = parts[0].parse::<usize>().map_err(|_| {
                    ToolError::new("INVALID_PATCH", "Invalid line number in hunk header")
                })?;
                Ok((start, 1))
            }
            2 => {
                let start = parts[0].parse::<usize>().map_err(|_| {
                    ToolError::new("INVALID_PATCH", "Invalid line number in hunk header")
                })?;
                let count = parts[1].parse::<usize>().map_err(|_| {
                    ToolError::new("INVALID_PATCH", "Invalid line count in hunk header")
                })?;
                Ok((start, count))
            }
            _ => Err(
                ToolError::new("INVALID_PATCH", "Invalid range specification")
                    .with_details(format!("Range: {}", range_spec)),
            ),
        }
    }

    /// Apply a single hunk to file lines
    fn apply_hunk(
        file_lines: &mut Vec<String>,
        hunk: &Hunk,
        hunk_number: usize,
    ) -> Result<(), FailedHunkInfo> {
        // Convert to 0-indexed
        let mut file_idx = if hunk.orig_start > 0 {
            hunk.orig_start - 1
        } else {
            0
        };

        let mut hunk_idx = 0;
        let mut lines_to_add = Vec::new();

        // First pass: validate the hunk matches the file
        let mut temp_file_idx = file_idx;
        let mut temp_hunk_idx = 0;

        while temp_hunk_idx < hunk.lines.len() {
            let hunk_line = &hunk.lines[temp_hunk_idx];

            if hunk_line.starts_with('-') {
                // Line should be removed
                let expected = &hunk_line[1..];
                if temp_file_idx >= file_lines.len() {
                    return Err(FailedHunkInfo {
                        hunk_number,
                        line_number: hunk.orig_start,
                        error: "File is too short for this hunk".to_string(),
                        context: None,
                    });
                }

                if file_lines[temp_file_idx] != expected {
                    let context = format!(
                        "Expected: '{}', Found: '{}'",
                        expected, file_lines[temp_file_idx]
                    );
                    return Err(FailedHunkInfo {
                        hunk_number,
                        line_number: hunk.orig_start + temp_file_idx - file_idx,
                        error: "Line content mismatch".to_string(),
                        context: Some(context),
                    });
                }
                temp_file_idx += 1;
            } else if hunk_line.starts_with('+') {
                // Line should be added
                lines_to_add.push(hunk_line[1..].to_string());
            } else if hunk_line.starts_with(' ') {
                // Context line should match
                let expected = &hunk_line[1..];
                if temp_file_idx >= file_lines.len() {
                    return Err(FailedHunkInfo {
                        hunk_number,
                        line_number: hunk.orig_start,
                        error: "File is too short for this hunk".to_string(),
                        context: None,
                    });
                }

                if file_lines[temp_file_idx] != expected {
                    let context = format!(
                        "Expected: '{}', Found: '{}'",
                        expected, file_lines[temp_file_idx]
                    );
                    return Err(FailedHunkInfo {
                        hunk_number,
                        line_number: hunk.orig_start + temp_file_idx - file_idx,
                        error: "Context line mismatch".to_string(),
                        context: Some(context),
                    });
                }
                temp_file_idx += 1;
            }
            temp_hunk_idx += 1;
        }

        // Second pass: apply the hunk
        while hunk_idx < hunk.lines.len() {
            let hunk_line = &hunk.lines[hunk_idx];

            if hunk_line.starts_with('-') {
                // Remove line
                if file_idx < file_lines.len() {
                    file_lines.remove(file_idx);
                }
            } else if hunk_line.starts_with('+') {
                // Add line
                file_lines.insert(file_idx, hunk_line[1..].to_string());
                file_idx += 1;
            } else if hunk_line.starts_with(' ') {
                // Context line
                file_idx += 1;
            }
            hunk_idx += 1;
        }

        Ok(())
    }

    /// Apply a patch to a file with timeout enforcement (1 second)
    pub async fn apply_patch_with_timeout(input: &PatchInput) -> Result<PatchOutput, ToolError> {
        let timeout_duration = std::time::Duration::from_secs(1);

        match tokio::time::timeout(timeout_duration, async {
            Self::apply_patch_internal(input)
        })
        .await
        {
            Ok(result) => result,
            Err(_) => Err(
                ToolError::new("TIMEOUT", "Patch operation exceeded 1 second timeout")
                    .with_details(format!("File: {}", input.file_path))
                    .with_suggestion("Try applying the patch again or check file size"),
            ),
        }
    }

    /// Apply a patch to a file (synchronous version)
    pub fn apply_patch(input: &PatchInput) -> Result<PatchOutput, ToolError> {
        Self::apply_patch_internal(input)
    }

    /// Internal patch application logic
    fn apply_patch_internal(input: &PatchInput) -> Result<PatchOutput, ToolError> {
        // Parse the patch
        let hunks = Self::parse_patch(&input.patch_content)?;

        // Read the file
        let file_path = Path::new(&input.file_path);
        let file_content = std::fs::read_to_string(file_path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                ToolError::new(
                    "FILE_NOT_FOUND",
                    format!("File not found: {}", input.file_path),
                )
                .with_suggestion("Ensure the file path is correct")
            } else {
                ToolError::from(e)
            }
        })?;

        let mut file_lines: Vec<String> = file_content.lines().map(|s| s.to_string()).collect();

        let mut applied_hunks = 0;
        let mut failed_hunks = 0;
        let mut failed_hunk_details = Vec::new();

        // Apply each hunk
        for (idx, hunk) in hunks.iter().enumerate() {
            match Self::apply_hunk(&mut file_lines, hunk, idx + 1) {
                Ok(()) => {
                    applied_hunks += 1;
                }
                Err(failed_info) => {
                    failed_hunks += 1;
                    failed_hunk_details.push(failed_info);
                }
            }
        }

        // If any hunks failed, don't write the file
        if failed_hunks > 0 {
            return Ok(PatchOutput {
                success: false,
                applied_hunks,
                failed_hunks,
                failed_hunk_details,
            });
        }

        // Write the patched file
        let patched_content = file_lines.join("\n");
        std::fs::write(file_path, patched_content).map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                ToolError::new("PERMISSION_DENIED", "Permission denied writing to file")
                    .with_suggestion("Check file permissions")
            } else {
                ToolError::from(e)
            }
        })?;

        Ok(PatchOutput {
            success: true,
            applied_hunks,
            failed_hunks: 0,
            failed_hunk_details: Vec::new(),
        })
    }
}

impl Default for PatchTool {
    fn default() -> Self {
        Self::new()
    }
}

/// Provider implementation for patch tool
pub mod provider {
    use super::*;
    use crate::provider::Provider;
    use async_trait::async_trait;
    use std::sync::Arc;
    use tracing::{debug, warn};

    /// Built-in patch provider
    pub struct BuiltinPatchProvider;

    #[async_trait]
    impl Provider for BuiltinPatchProvider {
        async fn execute(&self, input: &str) -> Result<String, ToolError> {
            debug!("Executing patch with built-in provider");

            // Parse input as JSON
            let patch_input: PatchInput = serde_json::from_str(input).map_err(|e| {
                ToolError::new("INVALID_INPUT", "Failed to parse patch input")
                    .with_details(e.to_string())
                    .with_suggestion("Ensure input is valid JSON with file_path and patch_content")
            })?;

            // Apply the patch
            let output = PatchTool::apply_patch(&patch_input)?;

            // Return output as JSON
            serde_json::to_string(&output).map_err(|e| {
                ToolError::new("SERIALIZATION_ERROR", "Failed to serialize patch output")
                    .with_details(e.to_string())
            })
        }
    }

    /// MCP patch provider wrapper
    pub struct McpPatchProvider {
        mcp_provider: Arc<dyn Provider>,
    }

    impl McpPatchProvider {
        /// Create a new MCP patch provider
        pub fn new(mcp_provider: Arc<dyn Provider>) -> Self {
            Self { mcp_provider }
        }
    }

    #[async_trait]
    impl Provider for McpPatchProvider {
        async fn execute(&self, input: &str) -> Result<String, ToolError> {
            debug!("Executing patch with MCP provider");

            match self.mcp_provider.execute(input).await {
                Ok(result) => {
                    debug!("MCP patch provider succeeded");
                    Ok(result)
                }
                Err(e) => {
                    warn!(
                        "MCP patch provider failed, would fall back to built-in: {}",
                        e
                    );
                    Err(e)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_range_single_line() {
        let (start, count) = PatchTool::parse_range("-10").unwrap();
        assert_eq!(start, 10);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_parse_range_multiple_lines() {
        let (start, count) = PatchTool::parse_range("-10,5").unwrap();
        assert_eq!(start, 10);
        assert_eq!(count, 5);
    }

    #[test]
    fn test_parse_simple_patch() {
        let patch = r#"--- a/test.txt
+++ b/test.txt
@@ -1,3 +1,3 @@
 line 1
-line 2
+line 2 modified
 line 3"#;

        let hunks = PatchTool::parse_patch(patch).unwrap();
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].orig_start, 1);
        assert!(!hunks[0].lines.is_empty());
    }

    #[test]
    fn test_apply_simple_patch() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "line 1").unwrap();
        writeln!(file, "line 2").unwrap();
        writeln!(file, "line 3").unwrap();
        file.flush().unwrap();

        let patch = r#"--- a/test.txt
+++ b/test.txt
@@ -1,3 +1,3 @@
 line 1
-line 2
+line 2 modified
 line 3"#;

        let input = PatchInput {
            file_path: file.path().to_string_lossy().to_string(),
            patch_content: patch.to_string(),
        };

        let output = PatchTool::apply_patch(&input).unwrap();
        assert!(output.success);
        assert_eq!(output.applied_hunks, 1);
        assert_eq!(output.failed_hunks, 0);

        // Verify the file was modified
        let content = std::fs::read_to_string(file.path()).unwrap();
        assert!(content.contains("line 2 modified"));
    }

    #[test]
    fn test_patch_conflict_detection() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "line 1").unwrap();
        writeln!(file, "different line").unwrap();
        writeln!(file, "line 3").unwrap();
        file.flush().unwrap();

        let patch = r#"--- a/test.txt
+++ b/test.txt
@@ -1,3 +1,3 @@
 line 1
-line 2
+line 2 modified
 line 3"#;

        let input = PatchInput {
            file_path: file.path().to_string_lossy().to_string(),
            patch_content: patch.to_string(),
        };

        let output = PatchTool::apply_patch(&input).unwrap();
        assert!(!output.success);
        assert_eq!(output.failed_hunks, 1);
        assert!(!output.failed_hunk_details.is_empty());
    }

    #[test]
    fn test_patch_file_not_found() {
        let patch = r#"--- a/test.txt
+++ b/test.txt
@@ -1,3 +1,3 @@
 line 1
-line 2
+line 2 modified
 line 3"#;

        let input = PatchInput {
            file_path: "/nonexistent/file.txt".to_string(),
            patch_content: patch.to_string(),
        };

        let result = PatchTool::apply_patch(&input);
        assert!(result.is_err());
        if let Err(err) = result {
            assert_eq!(err.code, "FILE_NOT_FOUND");
        }
    }

    #[test]
    fn test_invalid_patch_format() {
        let patch = "invalid patch content";

        let result = PatchTool::parse_patch(patch);
        assert!(result.is_err());
        if let Err(err) = result {
            assert_eq!(err.code, "INVALID_PATCH");
        }
    }

    #[tokio::test]
    async fn test_builtin_provider() {
        use crate::patch::provider::BuiltinPatchProvider;
        use crate::provider::Provider;

        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "line 1").unwrap();
        writeln!(file, "line 2").unwrap();
        writeln!(file, "line 3").unwrap();
        file.flush().unwrap();

        let patch = r#"--- a/test.txt
+++ b/test.txt
@@ -1,3 +1,3 @@
 line 1
-line 2
+line 2 modified
 line 3"#;

        let input = PatchInput {
            file_path: file.path().to_string_lossy().to_string(),
            patch_content: patch.to_string(),
        };

        let provider = BuiltinPatchProvider;
        let input_json = serde_json::to_string(&input).unwrap();
        let result = provider.execute(&input_json).await.unwrap();

        let output: PatchOutput = serde_json::from_str(&result).unwrap();
        assert!(output.success);
        assert_eq!(output.applied_hunks, 1);
    }

    #[tokio::test]
    async fn test_patch_timeout_enforcement() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "line 1").unwrap();
        writeln!(file, "line 2").unwrap();
        writeln!(file, "line 3").unwrap();
        file.flush().unwrap();

        let patch = r#"--- a/test.txt
+++ b/test.txt
@@ -1,3 +1,3 @@
 line 1
-line 2
+line 2 modified
 line 3"#;

        let input = PatchInput {
            file_path: file.path().to_string_lossy().to_string(),
            patch_content: patch.to_string(),
        };

        // Test that timeout enforcement works (should complete well within 1 second)
        let result = PatchTool::apply_patch_with_timeout(&input).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.success);
    }
}
