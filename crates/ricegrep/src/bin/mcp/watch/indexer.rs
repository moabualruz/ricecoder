//! Index Update Logic for Watch Mode
//!
//! Handles incremental index updates when files change during watch mode.

use anyhow::{Context, Result};
use ricegrep::admin::AdminToolset;
use std::path::{Path, PathBuf};

/// Update index for changed files with metadata gating optimization
///
/// Uses FileChangeFilter to skip unchanged files based on stored metadata,
/// significantly reducing re-indexing overhead in watch mode.
pub async fn update_index_for_changes(
    toolset: &std::sync::Arc<AdminToolset>,
    root_path: &Path,
    changed_files: &[PathBuf],
) -> Result<usize> {
    let count = changed_files.len();

    if count == 0 {
        return Ok(0);
    }

    // Initialize metadata store with auto-save enabled
    let metadata_path = root_path.join(".ricegrep/metadata.json");

    // Ensure metadata directory exists
    if let Some(parent) = metadata_path.parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }

    // Load or create metadata store
    let store = ricegrep::indexing_optimization::MetadataStore::new(metadata_path.clone(), true);
    match store.load() {
        Ok(_) => {
            tracing::debug!("Loaded existing metadata store");
        }
        Err(e) => {
            tracing::debug!("Creating new metadata store: {}", e);
        }
    }

    // Apply metadata gating filter
    let filter = ricegrep::indexing_optimization::FileChangeFilter::new(store);
    let filter_result = filter.filter_changes(changed_files);

    let files_to_reindex = &filter_result.files_to_reindex;
    let reindex_count = files_to_reindex.len();
    let skipped_count = filter_result.skipped_count();

    // Log filtering results
    if skipped_count > 0 {
        tracing::info!(
            "Metadata gating: {} file(s) unchanged (skipped), {} file(s) need re-indexing",
            skipped_count,
            reindex_count
        );

        // Log skipped files (up to 5)
        for (path, reason) in filter_result.skipped_files.iter().take(5) {
            tracing::debug!("  Skipped: {} ({})", path.display(), reason);
        }
        if filter_result.skipped_files.len() > 5 {
            tracing::debug!(
                "  ... and {} more skipped files",
                filter_result.skipped_files.len() - 5
            );
        }
    } else {
        tracing::info!(
            "No unchanged files detected, re-indexing {} file(s)",
            reindex_count
        );
    }

    // For performance: if too many files still need re-indexing, do full re-index
    if reindex_count > 100 {
        tracing::info!(
            "Many files still need re-indexing ({}), performing full re-index",
            reindex_count
        );
        let metadata_path = root_path.join(".ricegrep/metadata.bin");

        // Ensure metadata directory exists
        if let Some(parent) = metadata_path.parent() {
            let _ = tokio::fs::create_dir_all(parent).await;
        }

        match toolset
            .reindex_repository_with_metadata(root_path, &metadata_path)
            .await
        {
            Ok(_) => {
                tracing::info!("Full index re-index completed successfully");
                Ok(count)
            }
            Err(e) => {
                tracing::warn!(
                    "Full re-index failed for repository at {}: {}",
                    root_path.display(),
                    e
                );
                Err(anyhow::anyhow!(
                    "Full re-index failed for repository at {}: {}",
                    root_path.display(),
                    e
                ))
            }
        }
    } else {
        // For small changes: attempt incremental re-index
        tracing::info!(
            "Re-indexing {} file(s) with potential changes",
            reindex_count
        );
        let metadata_path = root_path.join(".ricegrep/metadata.bin");

        // Ensure metadata directory exists
        if let Some(parent) = metadata_path.parent() {
            let _ = tokio::fs::create_dir_all(parent).await;
        }

        match toolset
            .reindex_repository_with_metadata(root_path, &metadata_path)
            .await
        {
            Ok(_) => {
                tracing::info!("Index update completed successfully");

                // Update metadata store with successfully re-indexed files
                let (success, errors) = filter.update_metadata_batch(files_to_reindex);
                if errors > 0 {
                    tracing::warn!("Failed to update metadata for {} file(s)", errors);
                } else {
                    tracing::debug!("Updated metadata for {} file(s)", success);
                }

                Ok(count)
            }
            Err(e) => {
                tracing::warn!(
                    "Index update failed for repository at {} ({} file changes): {}",
                    root_path.display(),
                    reindex_count,
                    e
                );
                Err(anyhow::anyhow!(
                    "Index update failed for repository at {} ({} file changes): {}",
                    root_path.display(),
                    reindex_count,
                    e
                ))
            }
        }
    }
}
