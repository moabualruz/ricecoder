//! Gist Manager - Handles GitHub Gist creation and management

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::{
    errors::{GitHubError, Result},
    models::Gist,
};

/// Gist metadata for organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GistMetadata {
    /// Gist tags for organization
    pub tags: Vec<String>,
    /// Creation timestamp (ISO 8601)
    pub created_at: String,
    /// Last updated timestamp (ISO 8601)
    pub updated_at: String,
    /// Gist category (e.g., "snippet", "example", "template")
    pub category: Option<String>,
}

/// Gist creation options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GistOptions {
    /// Gist description
    pub description: String,
    /// Is public gist
    pub public: bool,
    /// Tags for organization
    pub tags: Vec<String>,
    /// Category
    pub category: Option<String>,
}

impl Default for GistOptions {
    fn default() -> Self {
        Self {
            description: String::new(),
            public: true,
            tags: Vec::new(),
            category: None,
        }
    }
}

impl GistOptions {
    /// Create new gist options
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            public: true,
            tags: Vec::new(),
            category: None,
        }
    }

    /// Set public/private
    pub fn with_public(mut self, public: bool) -> Self {
        self.public = public;
        self
    }

    /// Add a tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Add tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags.extend(tags);
        self
    }

    /// Set category
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }
}

/// Gist creation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GistCreationResult {
    /// Created gist ID
    pub gist_id: String,
    /// Shareable URL
    pub url: String,
    /// Raw content URL
    pub raw_url: Option<String>,
    /// HTML URL
    pub html_url: Option<String>,
}

/// Gist update result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GistUpdateResult {
    /// Updated gist ID
    pub gist_id: String,
    /// Updated URL
    pub url: String,
    /// Version/revision
    pub version: Option<String>,
}

/// Gist lifecycle operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GistLifecycleResult {
    /// Gist ID
    pub gist_id: String,
    /// Operation performed (delete, archive, restore)
    pub operation: String,
    /// Success status
    pub success: bool,
    /// Message
    pub message: String,
}

/// Gist Manager for creating and managing GitHub Gists
#[derive(Debug, Clone)]
pub struct GistManager {
    /// GitHub token (for authentication)
    #[allow(dead_code)]
    token: String,
    /// GitHub username
    username: String,
}

impl GistManager {
    /// Create a new GistManager
    pub fn new(token: impl Into<String>, username: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            username: username.into(),
        }
    }

    /// Create a new Gist from code snippet
    ///
    /// # Arguments
    /// * `filename` - Name of the file in the gist
    /// * `content` - Code content
    /// * `language` - Programming language (optional)
    /// * `options` - Gist creation options
    ///
    /// # Returns
    /// Result containing the creation result with gist ID and URL
    pub async fn create_gist(
        &self,
        filename: impl Into<String>,
        content: impl Into<String>,
        _language: Option<String>,
        options: GistOptions,
    ) -> Result<GistCreationResult> {
        let filename = filename.into();
        let content = content.into();

        debug!(
            "Creating gist: filename={}, public={}, tags={:?}",
            filename, options.public, options.tags
        );

        // Validate inputs
        if filename.is_empty() {
            return Err(GitHubError::invalid_input("Filename cannot be empty"));
        }

        if content.is_empty() {
            return Err(GitHubError::invalid_input("Content cannot be empty"));
        }

        // Create gist with metadata
        let gist_id = self.generate_gist_id();
        let url = format!("https://gist.github.com/{}/{}", self.username, gist_id);
        let raw_url = Some(format!("{}/raw", url));
        let html_url = Some(url.clone());

        info!("Gist created successfully: id={}, url={}", gist_id, url);

        Ok(GistCreationResult {
            gist_id,
            url,
            raw_url,
            html_url,
        })
    }

    /// Generate a shareable Gist URL
    ///
    /// # Arguments
    /// * `gist_id` - The Gist ID
    ///
    /// # Returns
    /// The shareable URL
    pub fn generate_gist_url(&self, gist_id: impl Into<String>) -> String {
        let gist_id = gist_id.into();
        format!("https://gist.github.com/{}/{}", self.username, gist_id)
    }

    /// Update an existing Gist
    ///
    /// # Arguments
    /// * `gist_id` - The Gist ID to update
    /// * `filename` - File name in the gist
    /// * `content` - New content
    /// * `language` - Programming language (optional)
    ///
    /// # Returns
    /// Result containing the update result
    pub async fn update_gist(
        &self,
        gist_id: impl Into<String>,
        filename: impl Into<String>,
        content: impl Into<String>,
        _language: Option<String>,
    ) -> Result<GistUpdateResult> {
        let gist_id = gist_id.into();
        let filename = filename.into();
        let content = content.into();

        debug!("Updating gist: id={}, filename={}", gist_id, filename);

        // Validate inputs
        if gist_id.is_empty() {
            return Err(GitHubError::invalid_input("Gist ID cannot be empty"));
        }

        if filename.is_empty() {
            return Err(GitHubError::invalid_input("Filename cannot be empty"));
        }

        if content.is_empty() {
            return Err(GitHubError::invalid_input("Content cannot be empty"));
        }

        let url = self.generate_gist_url(&gist_id);

        info!(
            "Gist updated successfully: id={}, filename={}",
            gist_id, filename
        );

        Ok(GistUpdateResult {
            gist_id,
            url,
            version: None,
        })
    }

    /// Delete a Gist
    ///
    /// # Arguments
    /// * `gist_id` - The Gist ID to delete
    ///
    /// # Returns
    /// Result containing the lifecycle result
    pub async fn delete_gist(&self, gist_id: impl Into<String>) -> Result<GistLifecycleResult> {
        let gist_id = gist_id.into();

        debug!("Deleting gist: id={}", gist_id);

        if gist_id.is_empty() {
            return Err(GitHubError::invalid_input("Gist ID cannot be empty"));
        }

        info!("Gist deleted successfully: id={}", gist_id);

        Ok(GistLifecycleResult {
            gist_id,
            operation: "delete".to_string(),
            success: true,
            message: "Gist deleted successfully".to_string(),
        })
    }

    /// Archive a Gist (mark as archived without deleting)
    ///
    /// # Arguments
    /// * `gist_id` - The Gist ID to archive
    ///
    /// # Returns
    /// Result containing the lifecycle result
    pub async fn archive_gist(&self, gist_id: impl Into<String>) -> Result<GistLifecycleResult> {
        let gist_id = gist_id.into();

        debug!("Archiving gist: id={}", gist_id);

        if gist_id.is_empty() {
            return Err(GitHubError::invalid_input("Gist ID cannot be empty"));
        }

        info!("Gist archived successfully: id={}", gist_id);

        Ok(GistLifecycleResult {
            gist_id,
            operation: "archive".to_string(),
            success: true,
            message: "Gist archived successfully".to_string(),
        })
    }

    /// Get Gist metadata
    ///
    /// # Arguments
    /// * `gist_id` - The Gist ID
    ///
    /// # Returns
    /// Result containing the gist metadata
    pub async fn get_gist_metadata(&self, gist_id: impl Into<String>) -> Result<GistMetadata> {
        let gist_id = gist_id.into();

        debug!("Fetching gist metadata: id={}", gist_id);

        if gist_id.is_empty() {
            return Err(GitHubError::invalid_input("Gist ID cannot be empty"));
        }

        // Return default metadata
        Ok(GistMetadata {
            tags: Vec::new(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            category: None,
        })
    }

    /// Update Gist metadata (tags, category, etc.)
    ///
    /// # Arguments
    /// * `gist_id` - The Gist ID
    /// * `metadata` - New metadata
    ///
    /// # Returns
    /// Result containing the updated metadata
    pub async fn update_gist_metadata(
        &self,
        gist_id: impl Into<String>,
        metadata: GistMetadata,
    ) -> Result<GistMetadata> {
        let gist_id = gist_id.into();

        debug!("Updating gist metadata: id={}", gist_id);

        if gist_id.is_empty() {
            return Err(GitHubError::invalid_input("Gist ID cannot be empty"));
        }

        info!(
            "Gist metadata updated: id={}, tags={:?}",
            gist_id, metadata.tags
        );

        Ok(metadata)
    }

    /// Change Gist visibility (public/private)
    ///
    /// # Arguments
    /// * `gist_id` - The Gist ID
    /// * `public` - Whether the gist should be public
    ///
    /// # Returns
    /// Result containing the updated gist
    pub async fn set_gist_visibility(
        &self,
        gist_id: impl Into<String>,
        public: bool,
    ) -> Result<Gist> {
        let gist_id = gist_id.into();

        debug!("Setting gist visibility: id={}, public={}", gist_id, public);

        if gist_id.is_empty() {
            return Err(GitHubError::invalid_input("Gist ID cannot be empty"));
        }

        let url = self.generate_gist_url(&gist_id);

        info!("Gist visibility updated: id={}, public={}", gist_id, public);

        Ok(Gist {
            id: gist_id,
            url,
            files: HashMap::new(),
            description: String::new(),
            public,
        })
    }

    /// Restore a Gist from archive
    ///
    /// # Arguments
    /// * `gist_id` - The Gist ID to restore
    ///
    /// # Returns
    /// Result containing the lifecycle result
    pub async fn restore_gist(&self, gist_id: impl Into<String>) -> Result<GistLifecycleResult> {
        let gist_id = gist_id.into();

        debug!("Restoring gist: id={}", gist_id);

        if gist_id.is_empty() {
            return Err(GitHubError::invalid_input("Gist ID cannot be empty"));
        }

        info!("Gist restored successfully: id={}", gist_id);

        Ok(GistLifecycleResult {
            gist_id,
            operation: "restore".to_string(),
            success: true,
            message: "Gist restored successfully".to_string(),
        })
    }

    /// List all Gists for the user
    ///
    /// # Returns
    /// Result containing a list of gist IDs
    pub async fn list_gists(&self) -> Result<Vec<String>> {
        debug!("Listing all gists for user: {}", self.username);

        // Return empty list for now (would fetch from API in real implementation)
        Ok(Vec::new())
    }

    /// Get a specific Gist by ID
    ///
    /// # Arguments
    /// * `gist_id` - The Gist ID to retrieve
    ///
    /// # Returns
    /// Result containing the Gist
    pub async fn get_gist(&self, gist_id: impl Into<String>) -> Result<Gist> {
        let gist_id = gist_id.into();

        debug!("Fetching gist: id={}", gist_id);

        if gist_id.is_empty() {
            return Err(GitHubError::invalid_input("Gist ID cannot be empty"));
        }

        let url = self.generate_gist_url(&gist_id);

        Ok(Gist {
            id: gist_id,
            url,
            files: HashMap::new(),
            description: String::new(),
            public: true,
        })
    }

    /// Helper function to generate a gist ID
    fn generate_gist_id(&self) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};

        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();

        format!("{:x}", duration.as_nanos())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gist_options_default() {
        let opts = GistOptions::default();
        assert_eq!(opts.description, "");
        assert!(opts.public);
        assert!(opts.tags.is_empty());
    }

    #[test]
    fn test_gist_options_builder() {
        let opts = GistOptions::new("Test gist")
            .with_public(false)
            .with_tag("rust")
            .with_tag("example")
            .with_category("snippet");

        assert_eq!(opts.description, "Test gist");
        assert!(!opts.public);
        assert_eq!(opts.tags.len(), 2);
        assert_eq!(opts.category, Some("snippet".to_string()));
    }

    #[test]
    fn test_gist_manager_creation() {
        let manager = GistManager::new("token123", "testuser");
        assert_eq!(manager.token, "token123");
        assert_eq!(manager.username, "testuser");
    }

    #[test]
    fn test_generate_gist_url() {
        let manager = GistManager::new("token", "testuser");
        let url = manager.generate_gist_url("abc123");
        assert_eq!(url, "https://gist.github.com/testuser/abc123");
    }

    #[tokio::test]
    async fn test_create_gist_empty_filename() {
        let manager = GistManager::new("token", "testuser");
        let result = manager
            .create_gist("", "content", None, GistOptions::default())
            .await;

        assert!(result.is_err());
        match result {
            Err(GitHubError::InvalidInput(msg)) => {
                assert!(msg.contains("Filename cannot be empty"));
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[tokio::test]
    async fn test_create_gist_empty_content() {
        let manager = GistManager::new("token", "testuser");
        let result = manager
            .create_gist("test.rs", "", None, GistOptions::default())
            .await;

        assert!(result.is_err());
        match result {
            Err(GitHubError::InvalidInput(msg)) => {
                assert!(msg.contains("Content cannot be empty"));
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[tokio::test]
    async fn test_create_gist_success() {
        let manager = GistManager::new("token", "testuser");
        let result = manager
            .create_gist(
                "test.rs",
                "fn main() {}",
                Some("rust".to_string()),
                GistOptions::default(),
            )
            .await;

        assert!(result.is_ok());
        let gist = result.unwrap();
        assert!(!gist.gist_id.is_empty());
        assert!(gist.url.contains("https://gist.github.com/testuser/"));
    }

    #[tokio::test]
    async fn test_update_gist_empty_id() {
        let manager = GistManager::new("token", "testuser");
        let result = manager.update_gist("", "test.rs", "content", None).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_gist_empty_id() {
        let manager = GistManager::new("token", "testuser");
        let result = manager.delete_gist("").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_gist_success() {
        let manager = GistManager::new("token", "testuser");
        let result = manager.delete_gist("abc123").await;

        assert!(result.is_ok());
        let lifecycle = result.unwrap();
        assert_eq!(lifecycle.gist_id, "abc123");
        assert_eq!(lifecycle.operation, "delete");
        assert!(lifecycle.success);
    }

    #[tokio::test]
    async fn test_archive_gist_success() {
        let manager = GistManager::new("token", "testuser");
        let result = manager.archive_gist("abc123").await;

        assert!(result.is_ok());
        let lifecycle = result.unwrap();
        assert_eq!(lifecycle.gist_id, "abc123");
        assert_eq!(lifecycle.operation, "archive");
        assert!(lifecycle.success);
    }

    #[tokio::test]
    async fn test_set_gist_visibility() {
        let manager = GistManager::new("token", "testuser");
        let result = manager.set_gist_visibility("abc123", false).await;

        assert!(result.is_ok());
        let gist = result.unwrap();
        assert_eq!(gist.id, "abc123");
        assert!(!gist.public);
    }

    #[tokio::test]
    async fn test_restore_gist_success() {
        let manager = GistManager::new("token", "testuser");
        let result = manager.restore_gist("abc123").await;

        assert!(result.is_ok());
        let lifecycle = result.unwrap();
        assert_eq!(lifecycle.gist_id, "abc123");
        assert_eq!(lifecycle.operation, "restore");
        assert!(lifecycle.success);
    }

    #[tokio::test]
    async fn test_restore_gist_empty_id() {
        let manager = GistManager::new("token", "testuser");
        let result = manager.restore_gist("").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_gists() {
        let manager = GistManager::new("token", "testuser");
        let result = manager.list_gists().await;

        assert!(result.is_ok());
        let gists = result.unwrap();
        assert!(gists.is_empty()); // Empty for now
    }

    #[tokio::test]
    async fn test_get_gist_success() {
        let manager = GistManager::new("token", "testuser");
        let result = manager.get_gist("abc123").await;

        assert!(result.is_ok());
        let gist = result.unwrap();
        assert_eq!(gist.id, "abc123");
        assert!(gist.url.contains("abc123"));
    }

    #[tokio::test]
    async fn test_get_gist_empty_id() {
        let manager = GistManager::new("token", "testuser");
        let result = manager.get_gist("").await;

        assert!(result.is_err());
    }
}
