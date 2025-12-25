//! Edit tool implementation
//!
//! Handles safe file editing with atomic writes and timeout support.

use anyhow::{Context, Result};
use tokio::time::Duration;

use super::super::mappers;
use super::super::types::EditToolInput;

/// Apply an edit operation with timeout wrapper
pub async fn apply_edit(input: &EditToolInput) -> Result<String> {
    // Set timeout wrapper
    let timeout_duration = Duration::from_secs(input.timeout_secs.unwrap_or(30));

    tokio::time::timeout(timeout_duration, apply_edit_inner(input))
        .await
        .context(format!(
            "Edit operation timed out after {}s",
            timeout_duration.as_secs()
        ))?
}

/// Inner edit implementation with atomic file operations
pub async fn apply_edit_inner(input: &EditToolInput) -> Result<String> {
    // Read file asynchronously with enhanced error handling
    let content = match tokio::fs::read_to_string(&input.file_path).await {
        Ok(content) => content,
        Err(e) => {
            let message = match e.kind() {
                std::io::ErrorKind::NotFound => {
                    format!(
                        "File not found: {}. Check the path and try again.",
                        input.file_path
                    )
                }
                std::io::ErrorKind::PermissionDenied => {
                    format!(
                        "Permission denied: {}. Check file permissions.",
                        input.file_path
                    )
                }
                std::io::ErrorKind::InvalidData => {
                    format!(
                        "File contains invalid UTF-8: {}. Check file encoding.",
                        input.file_path
                    )
                }
                _ => {
                    // Could be file locked or other I/O error
                    format!(
                        "File is locked or inaccessible: {}. Close other applications and retry. (Error: {})",
                        input.file_path, e
                    )
                }
            };
            return Err(anyhow::anyhow!(message));
        }
    };

    // Handle replace_all parameter
    let new_content = if input.replace_all.unwrap_or(false) {
        content.replace(&input.old_string, &input.new_string)
    } else {
        content.replacen(&input.old_string, &input.new_string, 1)
    };

    // Check if replacement happened
    if new_content == content {
        return Err(anyhow::anyhow!(
            "Pattern not found: '{}' in {}",
            input.old_string,
            input.file_path
        ));
    }

    // Write atomically via temp file with enhanced error handling
    let temp_path = format!("{}.tmp", input.file_path);
    match tokio::fs::write(&temp_path, &new_content).await {
        Ok(_) => {}
        Err(e) => {
            let message = match e.kind() {
                std::io::ErrorKind::NotFound => {
                    format!(
                        "Cannot write to path: {} - parent directory does not exist.",
                        temp_path
                    )
                }
                std::io::ErrorKind::PermissionDenied => {
                    format!(
                        "Permission denied writing to: {}. Check directory permissions.",
                        temp_path
                    )
                }
                std::io::ErrorKind::InvalidInput => {
                    format!("Invalid file path: {}", temp_path)
                }
                _ => {
                    // Could be disk full or file locked
                    format!(
                        "Cannot write file: {} - disk may be full or file locked. (Error: {})",
                        temp_path, e
                    )
                }
            };
            return Err(anyhow::anyhow!(message));
        }
    }

    // Rename with enhanced error handling
    match tokio::fs::rename(&temp_path, &input.file_path).await {
        Ok(_) => {}
        Err(e) => {
            // Try to clean up temp file on failure
            let _ = tokio::fs::remove_file(&temp_path).await;
            let message = match e.kind() {
                std::io::ErrorKind::PermissionDenied => {
                    format!(
                        "Permission denied replacing original file: {}. Check file permissions.",
                        input.file_path
                    )
                }
                std::io::ErrorKind::NotFound => {
                    format!(
                        "Original file was deleted or moved: {}. Temporary file preserved at {}",
                        input.file_path, temp_path
                    )
                }
                _ => {
                    format!(
                        "Failed to complete edit - original file at {} may not be replaced. Temporary file at {} (Error: {})",
                        input.file_path, temp_path, e
                    )
                }
            };
            return Err(anyhow::anyhow!(message));
        }
    }

    let occurrences = content.matches(&input.old_string).count();
    let replaced = if input.replace_all.unwrap_or(false) {
        occurrences
    } else {
        1
    };

    // Use mapper for consistent response formatting
    Ok(mappers::format_edit_response(
        &input.file_path,
        &input.old_string,
        &input.new_string,
        replaced,
        input.replace_all.unwrap_or(false),
    ))
}
