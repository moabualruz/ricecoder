//! Patch tool for applying unified diff patches
//!
//! Provides functionality to parse and apply unified diff patches with conflict detection.
//! Supports both single-file and multi-file unified diffs.

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::ToolError;

/// Input for single-file patch operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchInput {
    /// Path to the file to patch
    pub file_path: String,
    /// Unified diff patch content
    pub patch_content: String,
}

/// Input for multi-file patch operations (e.g., from `git diff`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiFilePatchInput {
    /// Unified diff patch content containing multiple file sections
    pub patch_content: String,
    /// Optional base directory for relative file paths
    #[serde(default)]
    pub base_dir: Option<String>,
}

/// Represents a parsed file section within a multi-file patch
#[derive(Debug, Clone)]
pub struct FilePatch {
    /// Original file path (from --- line)
    pub old_path: String,
    /// New file path (from +++ line)
    pub new_path: String,
    /// Hunks for this file
    pub hunks: Vec<Hunk>,
    /// Whether this is a new file (old_path is /dev/null)
    pub is_new_file: bool,
    /// Whether this is a deleted file (new_path is /dev/null)
    pub is_deleted_file: bool,
}

/// Output from multi-file patch operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiFilePatchOutput {
    /// Overall success (all files patched successfully)
    pub success: bool,
    /// Number of files successfully patched
    pub files_patched: usize,
    /// Number of files that failed
    pub files_failed: usize,
    /// Per-file results
    pub file_results: Vec<FileResult>,
}

/// Result for a single file in multi-file patch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileResult {
    /// File path
    pub file_path: String,
    /// Whether this file was patched successfully
    pub success: bool,
    /// Number of hunks applied
    pub applied_hunks: usize,
    /// Number of hunks that failed
    pub failed_hunks: usize,
    /// Error message if failed
    pub error: Option<String>,
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
pub struct Hunk {
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

// ============================================================================
// Multi-file patch support
// ============================================================================

impl PatchTool {
    /// Parse a multi-file unified diff into separate file patches
    ///
    /// Handles standard git diff format with multiple file sections:
    /// ```text
    /// diff --git a/file1.rs b/file1.rs
    /// --- a/file1.rs
    /// +++ b/file1.rs
    /// @@ -1,3 +1,4 @@
    /// ...
    /// diff --git a/file2.rs b/file2.rs
    /// --- a/file2.rs
    /// +++ b/file2.rs
    /// @@ -5,2 +5,3 @@
    /// ...
    /// ```
    pub fn parse_multi_file_patch(patch_content: &str) -> Result<Vec<FilePatch>, ToolError> {
        let mut file_patches = Vec::new();
        let mut lines = patch_content.lines().peekable();
        
        while lines.peek().is_some() {
            // Skip until we find a file header (--- line or diff --git line)
            let mut old_path: Option<String> = None;
            let mut new_path: Option<String> = None;
            
            // Look for the start of a file section
            while let Some(line) = lines.peek() {
                if line.starts_with("diff --git") {
                    // Parse git diff header: diff --git a/path b/path
                    lines.next();
                    continue;
                } else if line.starts_with("index ") || line.starts_with("new file mode") 
                    || line.starts_with("deleted file mode") || line.starts_with("old mode")
                    || line.starts_with("new mode") || line.starts_with("similarity index")
                    || line.starts_with("rename from") || line.starts_with("rename to")
                    || line.starts_with("copy from") || line.starts_with("copy to") {
                    lines.next();
                    continue;
                } else if line.starts_with("---") {
                    old_path = Some(Self::parse_file_path(line, "---"));
                    lines.next();
                    break;
                } else if line.is_empty() {
                    lines.next();
                    continue;
                } else {
                    // Unknown line before file header, skip
                    lines.next();
                    continue;
                }
            }
            
            // If we didn't find a --- line, we're done
            let old_path = match old_path {
                Some(p) => p,
                None => break,
            };
            
            // Expect +++ line next
            if let Some(line) = lines.next() {
                if line.starts_with("+++") {
                    new_path = Some(Self::parse_file_path(line, "+++"));
                } else {
                    return Err(ToolError::new(
                        "INVALID_PATCH",
                        "Expected +++ line after --- line"
                    ).with_details(format!("Found: {}", line)));
                }
            }
            
            let new_path = match new_path {
                Some(p) => p,
                None => return Err(ToolError::new(
                    "INVALID_PATCH", 
                    "Missing +++ line in patch"
                )),
            };
            
            // Determine if new/deleted file
            let is_new_file = old_path == "/dev/null";
            let is_deleted_file = new_path == "/dev/null";
            
            // Parse hunks for this file until we hit another file section or EOF
            let mut hunks = Vec::new();
            
            while let Some(line) = lines.peek() {
                if line.starts_with("diff --git") || line.starts_with("---") {
                    // Start of next file section
                    break;
                }
                
                if line.starts_with("@@") {
                    // Parse hunk
                    let hunk_line = lines.next().unwrap();
                    let hunk_header = hunk_line.trim_start_matches("@@").trim_end_matches("@@").trim();
                    
                    let parts: Vec<&str> = hunk_header.split_whitespace().collect();
                    if parts.len() < 2 {
                        return Err(ToolError::new(
                            "INVALID_PATCH",
                            "Invalid hunk header format"
                        ).with_details(format!("Hunk header: {}", hunk_line)));
                    }
                    
                    let (orig_start, _) = Self::parse_range(parts[0])?;
                    
                    let mut hunk_lines = Vec::new();
                    
                    // Read hunk content
                    while let Some(hunk_content) = lines.peek() {
                        if hunk_content.starts_with("@@") 
                            || hunk_content.starts_with("diff --git")
                            || hunk_content.starts_with("---") {
                            break;
                        }
                        
                        if hunk_content.starts_with("\\") {
                            // Skip "\ No newline at end of file"
                            lines.next();
                            continue;
                        }
                        
                        if hunk_content.is_empty()
                            || hunk_content.starts_with("-")
                            || hunk_content.starts_with("+")
                            || hunk_content.starts_with(" ") {
                            hunk_lines.push(lines.next().unwrap().to_string());
                        } else {
                            break;
                        }
                    }
                    
                    hunks.push(Hunk {
                        orig_start,
                        lines: hunk_lines,
                    });
                } else {
                    // Skip unrecognized lines between hunks
                    lines.next();
                }
            }
            
            // Only add if we have hunks (or it's a new/deleted file indicator)
            if !hunks.is_empty() || is_new_file || is_deleted_file {
                file_patches.push(FilePatch {
                    old_path,
                    new_path,
                    hunks,
                    is_new_file,
                    is_deleted_file,
                });
            }
        }
        
        if file_patches.is_empty() {
            return Err(ToolError::new(
                "INVALID_PATCH",
                "No file sections found in multi-file patch"
            ).with_suggestion("Ensure patch is in unified diff format with --- and +++ headers"));
        }
        
        Ok(file_patches)
    }
    
    /// Parse file path from --- or +++ line
    fn parse_file_path(line: &str, prefix: &str) -> String {
        let path = line.trim_start_matches(prefix).trim();
        // Remove a/ or b/ prefix commonly used in git diffs
        let path = path.trim_start_matches("a/").trim_start_matches("b/");
        // Handle timestamp suffix (e.g., "file.txt\t2024-01-01 00:00:00")
        let path = path.split('\t').next().unwrap_or(path);
        path.to_string()
    }
    
    /// Apply a multi-file patch with atomic rollback on failure
    pub fn apply_multi_file_patch(input: &MultiFilePatchInput) -> Result<MultiFilePatchOutput, ToolError> {
        let file_patches = Self::parse_multi_file_patch(&input.patch_content)?;
        
        let base_dir = input.base_dir.as_deref().unwrap_or(".");
        let base_path = Path::new(base_dir);
        
        let mut file_results = Vec::new();
        let mut all_backups: HashMap<String, String> = HashMap::new();
        let mut files_patched = 0;
        let mut files_failed = 0;
        
        // First pass: validate all files exist and create backups
        for file_patch in &file_patches {
            let target_path = if file_patch.is_new_file {
                &file_patch.new_path
            } else {
                &file_patch.new_path
            };
            
            let full_path = base_path.join(target_path);
            
            // For existing files, create backup
            if !file_patch.is_new_file && full_path.exists() {
                match std::fs::read_to_string(&full_path) {
                    Ok(content) => {
                        all_backups.insert(full_path.to_string_lossy().to_string(), content);
                    }
                    Err(e) => {
                        return Err(ToolError::new(
                            "FILE_READ_ERROR",
                            format!("Failed to read file for backup: {}", target_path)
                        ).with_details(e.to_string()));
                    }
                }
            }
        }
        
        // Second pass: apply patches
        for file_patch in &file_patches {
            let target_path = &file_patch.new_path;
            let full_path = base_path.join(target_path);
            let full_path_str = full_path.to_string_lossy().to_string();
            
            // Handle deleted files
            if file_patch.is_deleted_file {
                if full_path.exists() {
                    match std::fs::remove_file(&full_path) {
                        Ok(()) => {
                            file_results.push(FileResult {
                                file_path: target_path.clone(),
                                success: true,
                                applied_hunks: 0,
                                failed_hunks: 0,
                                error: None,
                            });
                            files_patched += 1;
                        }
                        Err(e) => {
                            // Rollback all changes
                            Self::rollback_changes(&all_backups);
                            return Err(ToolError::new(
                                "FILE_DELETE_ERROR",
                                format!("Failed to delete file: {}", target_path)
                            ).with_details(e.to_string()));
                        }
                    }
                }
                continue;
            }
            
            // Handle new files
            if file_patch.is_new_file {
                // Create parent directories if needed
                if let Some(parent) = full_path.parent() {
                    if !parent.exists() {
                        if let Err(e) = std::fs::create_dir_all(parent) {
                            Self::rollback_changes(&all_backups);
                            return Err(ToolError::new(
                                "DIR_CREATE_ERROR",
                                format!("Failed to create directory for: {}", target_path)
                            ).with_details(e.to_string()));
                        }
                    }
                }
                
                // New file: collect all added lines
                let new_content: Vec<String> = file_patch.hunks.iter()
                    .flat_map(|h| h.lines.iter())
                    .filter(|l| l.starts_with("+"))
                    .map(|l| l[1..].to_string())
                    .collect();
                
                match std::fs::write(&full_path, new_content.join("\n")) {
                    Ok(()) => {
                        file_results.push(FileResult {
                            file_path: target_path.clone(),
                            success: true,
                            applied_hunks: file_patch.hunks.len(),
                            failed_hunks: 0,
                            error: None,
                        });
                        files_patched += 1;
                    }
                    Err(e) => {
                        Self::rollback_changes(&all_backups);
                        return Err(ToolError::new(
                            "FILE_WRITE_ERROR",
                            format!("Failed to create new file: {}", target_path)
                        ).with_details(e.to_string()));
                    }
                }
                continue;
            }
            
            // Apply patch to existing file using single-file logic
            let single_input = PatchInput {
                file_path: full_path_str.clone(),
                patch_content: Self::reconstruct_single_file_patch(&file_patch),
            };
            
            match Self::apply_patch(&single_input) {
                Ok(output) => {
                    if output.success {
                        file_results.push(FileResult {
                            file_path: target_path.clone(),
                            success: true,
                            applied_hunks: output.applied_hunks,
                            failed_hunks: 0,
                            error: None,
                        });
                        files_patched += 1;
                    } else {
                        // Rollback all changes on failure
                        Self::rollback_changes(&all_backups);
                        files_failed += 1;
                        
                        let error_details = output.failed_hunk_details
                            .iter()
                            .map(|h| format!("Hunk {}: {}", h.hunk_number, h.error))
                            .collect::<Vec<_>>()
                            .join("; ");
                        
                        file_results.push(FileResult {
                            file_path: target_path.clone(),
                            success: false,
                            applied_hunks: output.applied_hunks,
                            failed_hunks: output.failed_hunks,
                            error: Some(error_details),
                        });
                        
                        return Ok(MultiFilePatchOutput {
                            success: false,
                            files_patched,
                            files_failed,
                            file_results,
                        });
                    }
                }
                Err(e) => {
                    Self::rollback_changes(&all_backups);
                    files_failed += 1;
                    
                    file_results.push(FileResult {
                        file_path: target_path.clone(),
                        success: false,
                        applied_hunks: 0,
                        failed_hunks: file_patch.hunks.len(),
                        error: Some(e.message.clone()),
                    });
                    
                    return Ok(MultiFilePatchOutput {
                        success: false,
                        files_patched,
                        files_failed,
                        file_results,
                    });
                }
            }
        }
        
        Ok(MultiFilePatchOutput {
            success: files_failed == 0,
            files_patched,
            files_failed,
            file_results,
        })
    }
    
    /// Reconstruct a single-file patch from a FilePatch for reuse of existing logic
    fn reconstruct_single_file_patch(file_patch: &FilePatch) -> String {
        let mut patch = String::new();
        patch.push_str(&format!("--- a/{}\n", file_patch.old_path));
        patch.push_str(&format!("+++ b/{}\n", file_patch.new_path));
        
        for hunk in &file_patch.hunks {
            // Reconstruct hunk header (simplified)
            let orig_count = hunk.lines.iter()
                .filter(|l| l.starts_with("-") || l.starts_with(" "))
                .count();
            let new_count = hunk.lines.iter()
                .filter(|l| l.starts_with("+") || l.starts_with(" "))
                .count();
            
            patch.push_str(&format!(
                "@@ -{},{} +{},{} @@\n",
                hunk.orig_start, orig_count, hunk.orig_start, new_count
            ));
            
            for line in &hunk.lines {
                patch.push_str(line);
                patch.push('\n');
            }
        }
        
        patch
    }
    
    /// Rollback all file changes using backups
    fn rollback_changes(backups: &HashMap<String, String>) {
        for (path, content) in backups {
            // Best effort rollback - ignore errors
            let _ = std::fs::write(path, content);
        }
    }
    
    /// Apply a multi-file patch with timeout enforcement
    pub async fn apply_multi_file_patch_with_timeout(
        input: &MultiFilePatchInput,
    ) -> Result<MultiFilePatchOutput, ToolError> {
        let timeout_duration = std::time::Duration::from_secs(10); // Longer timeout for multi-file
        
        let input_clone = input.clone();
        match tokio::time::timeout(timeout_duration, async move {
            Self::apply_multi_file_patch(&input_clone)
        })
        .await
        {
            Ok(result) => result,
            Err(_) => Err(
                ToolError::new("TIMEOUT", "Multi-file patch operation exceeded 10 second timeout")
                    .with_suggestion("Try applying patches in smaller batches"),
            ),
        }
    }
}

/// Provider implementation for patch tool
pub mod provider {
    use std::sync::Arc;

    use async_trait::async_trait;
    use tracing::{debug, warn};

    use super::*;
    use crate::provider::Provider;

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

    /// Built-in multi-file patch provider
    pub struct BuiltinMultiFilePatchProvider;

    #[async_trait]
    impl Provider for BuiltinMultiFilePatchProvider {
        async fn execute(&self, input: &str) -> Result<String, ToolError> {
            debug!("Executing multi-file patch with built-in provider");

            // Parse input as JSON
            let patch_input: MultiFilePatchInput = serde_json::from_str(input).map_err(|e| {
                ToolError::new("INVALID_INPUT", "Failed to parse multi-file patch input")
                    .with_details(e.to_string())
                    .with_suggestion("Ensure input is valid JSON with patch_content field")
            })?;

            // Apply the patch
            let output = PatchTool::apply_multi_file_patch(&patch_input)?;

            // Return output as JSON
            serde_json::to_string(&output).map_err(|e| {
                ToolError::new("SERIALIZATION_ERROR", "Failed to serialize patch output")
                    .with_details(e.to_string())
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use tempfile::NamedTempFile;

    use super::*;

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
        use crate::{patch::provider::BuiltinPatchProvider, provider::Provider};

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

    // ========================================================================
    // Multi-file patch tests
    // ========================================================================

    #[test]
    fn test_parse_multi_file_patch() {
        let patch = r#"diff --git a/file1.txt b/file1.txt
--- a/file1.txt
+++ b/file1.txt
@@ -1,3 +1,3 @@
 line 1
-line 2
+line 2 modified
 line 3
diff --git a/file2.txt b/file2.txt
--- a/file2.txt
+++ b/file2.txt
@@ -1,2 +1,2 @@
-old content
+new content
 unchanged"#;

        let file_patches = PatchTool::parse_multi_file_patch(patch).unwrap();
        assert_eq!(file_patches.len(), 2);
        
        assert_eq!(file_patches[0].old_path, "file1.txt");
        assert_eq!(file_patches[0].new_path, "file1.txt");
        assert_eq!(file_patches[0].hunks.len(), 1);
        
        assert_eq!(file_patches[1].old_path, "file2.txt");
        assert_eq!(file_patches[1].new_path, "file2.txt");
        assert_eq!(file_patches[1].hunks.len(), 1);
    }

    #[test]
    fn test_parse_new_file_patch() {
        let patch = r#"diff --git a/newfile.txt b/newfile.txt
new file mode 100644
--- /dev/null
+++ b/newfile.txt
@@ -0,0 +1,3 @@
+line 1
+line 2
+line 3"#;

        let file_patches = PatchTool::parse_multi_file_patch(patch).unwrap();
        assert_eq!(file_patches.len(), 1);
        assert!(file_patches[0].is_new_file);
        assert!(!file_patches[0].is_deleted_file);
        assert_eq!(file_patches[0].new_path, "newfile.txt");
    }

    #[test]
    fn test_parse_deleted_file_patch() {
        let patch = r#"diff --git a/oldfile.txt b/oldfile.txt
deleted file mode 100644
--- a/oldfile.txt
+++ /dev/null
@@ -1,3 +0,0 @@
-line 1
-line 2
-line 3"#;

        let file_patches = PatchTool::parse_multi_file_patch(patch).unwrap();
        assert_eq!(file_patches.len(), 1);
        assert!(!file_patches[0].is_new_file);
        assert!(file_patches[0].is_deleted_file);
        assert_eq!(file_patches[0].old_path, "oldfile.txt");
    }

    #[test]
    fn test_apply_multi_file_patch() {
        use tempfile::TempDir;
        
        let temp_dir = TempDir::new().unwrap();
        
        // Create test files
        let file1_path = temp_dir.path().join("file1.txt");
        let file2_path = temp_dir.path().join("file2.txt");
        
        std::fs::write(&file1_path, "line 1\nline 2\nline 3\n").unwrap();
        std::fs::write(&file2_path, "old content\nunchanged\n").unwrap();
        
        let patch = r#"diff --git a/file1.txt b/file1.txt
--- a/file1.txt
+++ b/file1.txt
@@ -1,3 +1,3 @@
 line 1
-line 2
+line 2 modified
 line 3
diff --git a/file2.txt b/file2.txt
--- a/file2.txt
+++ b/file2.txt
@@ -1,2 +1,2 @@
-old content
+new content
 unchanged"#;

        let input = MultiFilePatchInput {
            patch_content: patch.to_string(),
            base_dir: Some(temp_dir.path().to_string_lossy().to_string()),
        };

        let output = PatchTool::apply_multi_file_patch(&input).unwrap();
        assert!(output.success);
        assert_eq!(output.files_patched, 2);
        assert_eq!(output.files_failed, 0);
        
        // Verify file contents
        let file1_content = std::fs::read_to_string(&file1_path).unwrap();
        assert!(file1_content.contains("line 2 modified"));
        
        let file2_content = std::fs::read_to_string(&file2_path).unwrap();
        assert!(file2_content.contains("new content"));
    }

    #[test]
    fn test_multi_file_patch_rollback_on_failure() {
        use tempfile::TempDir;
        
        let temp_dir = TempDir::new().unwrap();
        
        // Create first file (will succeed)
        let file1_path = temp_dir.path().join("file1.txt");
        std::fs::write(&file1_path, "line 1\nline 2\nline 3\n").unwrap();
        
        // Create second file with wrong content (will fail)
        let file2_path = temp_dir.path().join("file2.txt");
        std::fs::write(&file2_path, "different content\n").unwrap();
        
        let original_file1 = std::fs::read_to_string(&file1_path).unwrap();
        
        let patch = r#"diff --git a/file1.txt b/file1.txt
--- a/file1.txt
+++ b/file1.txt
@@ -1,3 +1,3 @@
 line 1
-line 2
+line 2 modified
 line 3
diff --git a/file2.txt b/file2.txt
--- a/file2.txt
+++ b/file2.txt
@@ -1,2 +1,2 @@
-old content
+new content
 unchanged"#;

        let input = MultiFilePatchInput {
            patch_content: patch.to_string(),
            base_dir: Some(temp_dir.path().to_string_lossy().to_string()),
        };

        let output = PatchTool::apply_multi_file_patch(&input).unwrap();
        assert!(!output.success);
        
        // Verify rollback - file1 should be restored to original
        let file1_after = std::fs::read_to_string(&file1_path).unwrap();
        assert_eq!(file1_after, original_file1);
    }

    #[tokio::test]
    async fn test_builtin_multi_file_provider() {
        use crate::patch::provider::BuiltinMultiFilePatchProvider;
        use crate::provider::Provider;
        use tempfile::TempDir;
        
        let temp_dir = TempDir::new().unwrap();
        
        let file1_path = temp_dir.path().join("test.txt");
        std::fs::write(&file1_path, "line 1\nline 2\nline 3\n").unwrap();
        
        let patch = r#"diff --git a/test.txt b/test.txt
--- a/test.txt
+++ b/test.txt
@@ -1,3 +1,3 @@
 line 1
-line 2
+line 2 modified
 line 3"#;

        let input = MultiFilePatchInput {
            patch_content: patch.to_string(),
            base_dir: Some(temp_dir.path().to_string_lossy().to_string()),
        };

        let provider = BuiltinMultiFilePatchProvider;
        let input_json = serde_json::to_string(&input).unwrap();
        let result = provider.execute(&input_json).await.unwrap();

        let output: MultiFilePatchOutput = serde_json::from_str(&result).unwrap();
        assert!(output.success);
        assert_eq!(output.files_patched, 1);
    }
}
