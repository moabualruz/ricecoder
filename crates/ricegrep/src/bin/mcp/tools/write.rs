//! Write tool implementation
//!
//! Handles safe file writing with atomic operations and timeout support.

use anyhow::{Context, Result};
use tokio::time::Duration;

use super::super::mappers;
use super::super::types::WriteToolInput;

/// Apply a write operation with timeout wrapper
pub async fn apply_write(input: &WriteToolInput) -> Result<String> {
    // Set timeout wrapper
    let timeout_duration = Duration::from_secs(input.timeout_secs.unwrap_or(30));

    tokio::time::timeout(timeout_duration, apply_write_inner(input))
        .await
        .context(format!(
            "Write operation timed out after {}s",
            timeout_duration.as_secs()
        ))?
}

/// Inner write implementation with atomic file operations
pub async fn apply_write_inner(input: &WriteToolInput) -> Result<String> {
    // Create parent directory if needed with enhanced error handling
    if let Some(parent) = std::path::Path::new(&input.file_path).parent() {
        if !parent.as_os_str().is_empty() {
            match tokio::fs::create_dir_all(parent).await {
                Ok(_) => {}
                Err(e) => {
                    let message = match e.kind() {
                        std::io::ErrorKind::PermissionDenied => {
                            format!(
                                "Permission denied creating directory: {}. Check directory permissions.",
                                parent.display()
                            )
                        }
                        std::io::ErrorKind::InvalidInput => {
                            format!(
                                "Invalid directory path: {}. Check path is valid.",
                                parent.display()
                            )
                        }
                        _ => {
                            format!(
                                "Failed to create directory: {}. (Error: {})",
                                parent.display(),
                                e
                            )
                        }
                    };
                    return Err(anyhow::anyhow!(message));
                }
            }
        }
    }

    // Write atomically via temp file with enhanced error handling
    let temp_path = format!("{}.tmp", input.file_path);
    match tokio::fs::write(&temp_path, &input.content).await {
        Ok(_) => {}
        Err(e) => {
            let message = match e.kind() {
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
                    // Could be disk full or other I/O error
                    format!(
                        "Cannot write file: {} - disk may be full, read-only filesystem, or other I/O error. (Error: {})",
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
                        "Permission denied replacing file: {}. Check file permissions.",
                        input.file_path
                    )
                }
                std::io::ErrorKind::NotFound => {
                    format!(
                        "Parent directory was deleted or file path is invalid: {}. Temporary file preserved at {}",
                        input.file_path, temp_path
                    )
                }
                _ => {
                    format!(
                        "Failed to complete write - original file at {} may not be replaced. Temporary file at {} (Error: {})",
                        input.file_path, temp_path, e
                    )
                }
            };
            return Err(anyhow::anyhow!(message));
        }
    }

    let byte_count = input.content.len();

    // Use mapper for consistent response formatting
    Ok(mappers::format_write_response(
        &input.file_path,
        byte_count,
        &input.content,
    ))
}
