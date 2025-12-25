//! Edit tool implementation
//!
//! Handles safe file editing with atomic writes and timeout support.
//! Uses ricecoder-tools::format for response formatting.

use anyhow::{Context, Result};
use tokio::time::Duration;

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
            let message = ricecoder_tools::format::format_read_error(&input.file_path, &e);
            return Err(anyhow::anyhow!(message));
        }
    };

    // Handle replace_all parameter
    let replace_all = input.replace_all.unwrap_or(false);
    let new_content = if replace_all {
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
    if let Err(e) = tokio::fs::write(&temp_path, &new_content).await {
        let message = ricecoder_tools::format::format_write_error(&temp_path, &e);
        return Err(anyhow::anyhow!(message));
    }

    // Rename with enhanced error handling
    if let Err(e) = tokio::fs::rename(&temp_path, &input.file_path).await {
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

    let occurrences = content.matches(&input.old_string).count();
    let replaced = if replace_all { occurrences } else { 1 };

    // Use format module for consistent response formatting
    Ok(ricecoder_tools::format::format_edit_response(
        &input.file_path,
        &input.old_string,
        &input.new_string,
        replaced,
        replace_all,
    ))
}
