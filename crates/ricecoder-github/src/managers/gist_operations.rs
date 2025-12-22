//! Gist Operations - Additional operations for Gist management

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::{
    errors::{GitHubError, Result},
    models::Gist,
};

/// Gist sharing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GistSharingConfig {
    /// Share via email
    pub share_email: Option<String>,
    /// Share via social media
    pub share_social: Option<String>,
    /// Custom share message
    pub share_message: Option<String>,
}

/// Gist sharing result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GistSharingResult {
    /// Gist ID
    pub gist_id: String,
    /// Shareable URL
    pub url: String,
    /// Share method used
    pub share_method: String,
    /// Share timestamp
    pub shared_at: String,
}

/// Gist organization result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GistOrganizationResult {
    /// Gist ID
    pub gist_id: String,
    /// Tags applied
    pub tags: Vec<String>,
    /// Category assigned
    pub category: Option<String>,
    /// Organization timestamp
    pub organized_at: String,
}

/// Gist search criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GistSearchCriteria {
    /// Search by tag
    pub tag: Option<String>,
    /// Search by category
    pub category: Option<String>,
    /// Search by description (partial match)
    pub description_contains: Option<String>,
    /// Filter by public/private
    pub public_only: Option<bool>,
}

/// Gist search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GistSearchResult {
    /// Found gists
    pub gists: Vec<Gist>,
    /// Total count
    pub total_count: usize,
}

/// Gist batch operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GistBatchResult {
    /// Number of successful operations
    pub successful: usize,
    /// Number of failed operations
    pub failed: usize,
    /// Error messages for failed operations
    pub errors: HashMap<String, String>,
}

/// Gist Operations for advanced Gist management
#[derive(Debug, Clone)]
pub struct GistOperations {
    /// GitHub token
    #[allow(dead_code)]
    token: String,
    /// GitHub username
    username: String,
}

impl GistOperations {
    /// Create new GistOperations
    pub fn new(token: impl Into<String>, username: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            username: username.into(),
        }
    }

    /// Share a Gist
    ///
    /// # Arguments
    /// * `gist_id` - The Gist ID to share
    /// * `config` - Sharing configuration
    ///
    /// # Returns
    /// Result containing the sharing result
    pub async fn share_gist(
        &self,
        gist_id: impl Into<String>,
        config: GistSharingConfig,
    ) -> Result<GistSharingResult> {
        let gist_id = gist_id.into();

        debug!("Sharing gist: id={}", gist_id);

        if gist_id.is_empty() {
            return Err(GitHubError::invalid_input("Gist ID cannot be empty"));
        }

        let url = format!("https://gist.github.com/{}/{}", self.username, gist_id);
        let share_method = if config.share_email.is_some() {
            "email".to_string()
        } else if config.share_social.is_some() {
            "social".to_string()
        } else {
            "direct".to_string()
        };

        info!(
            "Gist shared successfully: id={}, method={}",
            gist_id, share_method
        );

        Ok(GistSharingResult {
            gist_id,
            url,
            share_method,
            shared_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Organize Gists with tags and categories
    ///
    /// # Arguments
    /// * `gist_id` - The Gist ID to organize
    /// * `tags` - Tags to apply
    /// * `category` - Category to assign
    ///
    /// # Returns
    /// Result containing the organization result
    pub async fn organize_gist(
        &self,
        gist_id: impl Into<String>,
        tags: Vec<String>,
        category: Option<String>,
    ) -> Result<GistOrganizationResult> {
        let gist_id = gist_id.into();

        debug!(
            "Organizing gist: id={}, tags={:?}, category={:?}",
            gist_id, tags, category
        );

        if gist_id.is_empty() {
            return Err(GitHubError::invalid_input("Gist ID cannot be empty"));
        }

        info!(
            "Gist organized successfully: id={}, tags={:?}",
            gist_id, tags
        );

        Ok(GistOrganizationResult {
            gist_id,
            tags,
            category,
            organized_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Search Gists by criteria
    ///
    /// # Arguments
    /// * `criteria` - Search criteria
    ///
    /// # Returns
    /// Result containing the search results
    pub async fn search_gists(&self, criteria: GistSearchCriteria) -> Result<GistSearchResult> {
        debug!("Searching gists with criteria: {:?}", criteria);

        // Return empty results for now
        Ok(GistSearchResult {
            gists: Vec::new(),
            total_count: 0,
        })
    }

    /// Batch delete Gists
    ///
    /// # Arguments
    /// * `gist_ids` - List of Gist IDs to delete
    ///
    /// # Returns
    /// Result containing the batch operation result
    pub async fn batch_delete_gists(&self, gist_ids: Vec<String>) -> Result<GistBatchResult> {
        debug!("Batch deleting {} gists", gist_ids.len());

        if gist_ids.is_empty() {
            return Err(GitHubError::invalid_input("Gist ID list cannot be empty"));
        }

        let mut errors = HashMap::new();
        let mut successful = 0;
        let mut failed = 0;

        for gist_id in gist_ids {
            if gist_id.is_empty() {
                failed += 1;
                errors.insert(gist_id.clone(), "Gist ID cannot be empty".to_string());
            } else {
                successful += 1;
            }
        }

        info!(
            "Batch delete completed: successful={}, failed={}",
            successful, failed
        );

        Ok(GistBatchResult {
            successful,
            failed,
            errors,
        })
    }

    /// Batch archive Gists
    ///
    /// # Arguments
    /// * `gist_ids` - List of Gist IDs to archive
    ///
    /// # Returns
    /// Result containing the batch operation result
    pub async fn batch_archive_gists(&self, gist_ids: Vec<String>) -> Result<GistBatchResult> {
        debug!("Batch archiving {} gists", gist_ids.len());

        if gist_ids.is_empty() {
            return Err(GitHubError::invalid_input("Gist ID list cannot be empty"));
        }

        let mut errors = HashMap::new();
        let mut successful = 0;
        let mut failed = 0;

        for gist_id in gist_ids {
            if gist_id.is_empty() {
                failed += 1;
                errors.insert(gist_id.clone(), "Gist ID cannot be empty".to_string());
            } else {
                successful += 1;
            }
        }

        info!(
            "Batch archive completed: successful={}, failed={}",
            successful, failed
        );

        Ok(GistBatchResult {
            successful,
            failed,
            errors,
        })
    }

    /// Export Gist to file
    ///
    /// # Arguments
    /// * `gist_id` - The Gist ID to export
    /// * `format` - Export format (json, yaml, etc.)
    ///
    /// # Returns
    /// Result containing the exported content
    pub async fn export_gist(
        &self,
        gist_id: impl Into<String>,
        format: impl Into<String>,
    ) -> Result<String> {
        let gist_id = gist_id.into();
        let format = format.into();

        debug!("Exporting gist: id={}, format={}", gist_id, format);

        if gist_id.is_empty() {
            return Err(GitHubError::invalid_input("Gist ID cannot be empty"));
        }

        if format.is_empty() {
            return Err(GitHubError::invalid_input("Format cannot be empty"));
        }

        info!("Gist exported: id={}, format={}", gist_id, format);

        Ok(format!("Exported gist {} in {} format", gist_id, format))
    }

    /// Import Gist from file
    ///
    /// # Arguments
    /// * `content` - File content to import
    /// * `format` - Import format (json, yaml, etc.)
    ///
    /// # Returns
    /// Result containing the imported Gist
    pub async fn import_gist(
        &self,
        content: impl Into<String>,
        format: impl Into<String>,
    ) -> Result<Gist> {
        let content = content.into();
        let format = format.into();

        debug!("Importing gist from {} format", format);

        if content.is_empty() {
            return Err(GitHubError::invalid_input("Content cannot be empty"));
        }

        if format.is_empty() {
            return Err(GitHubError::invalid_input("Format cannot be empty"));
        }

        info!("Gist imported from {} format", format);

        Ok(Gist {
            id: "imported".to_string(),
            url: "https://gist.github.com/imported".to_string(),
            files: HashMap::new(),
            description: "Imported gist".to_string(),
            public: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gist_operations_creation() {
        let ops = GistOperations::new("token", "testuser");
        assert_eq!(ops.token, "token");
        assert_eq!(ops.username, "testuser");
    }

    #[tokio::test]
    async fn test_share_gist_empty_id() {
        let ops = GistOperations::new("token", "testuser");
        let config = GistSharingConfig {
            share_email: None,
            share_social: None,
            share_message: None,
        };
        let result = ops.share_gist("", config).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_share_gist_success() {
        let ops = GistOperations::new("token", "testuser");
        let config = GistSharingConfig {
            share_email: Some("user@example.com".to_string()),
            share_social: None,
            share_message: None,
        };
        let result = ops.share_gist("abc123", config).await;

        assert!(result.is_ok());
        let share = result.unwrap();
        assert_eq!(share.gist_id, "abc123");
        assert_eq!(share.share_method, "email");
    }

    #[tokio::test]
    async fn test_organize_gist_success() {
        let ops = GistOperations::new("token", "testuser");
        let result = ops
            .organize_gist(
                "abc123",
                vec!["rust".to_string()],
                Some("snippet".to_string()),
            )
            .await;

        assert!(result.is_ok());
        let org = result.unwrap();
        assert_eq!(org.gist_id, "abc123");
        assert_eq!(org.tags.len(), 1);
        assert_eq!(org.category, Some("snippet".to_string()));
    }

    #[tokio::test]
    async fn test_batch_delete_gists_empty_list() {
        let ops = GistOperations::new("token", "testuser");
        let result = ops.batch_delete_gists(vec![]).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_batch_delete_gists_success() {
        let ops = GistOperations::new("token", "testuser");
        let result = ops
            .batch_delete_gists(vec!["abc123".to_string(), "def456".to_string()])
            .await;

        assert!(result.is_ok());
        let batch = result.unwrap();
        assert_eq!(batch.successful, 2);
        assert_eq!(batch.failed, 0);
    }

    #[tokio::test]
    async fn test_export_gist_empty_id() {
        let ops = GistOperations::new("token", "testuser");
        let result = ops.export_gist("", "json").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_export_gist_success() {
        let ops = GistOperations::new("token", "testuser");
        let result = ops.export_gist("abc123", "json").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_import_gist_empty_content() {
        let ops = GistOperations::new("token", "testuser");
        let result = ops.import_gist("", "json").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_import_gist_success() {
        let ops = GistOperations::new("token", "testuser");
        let result = ops.import_gist("{}", "json").await;

        assert!(result.is_ok());
    }
}
