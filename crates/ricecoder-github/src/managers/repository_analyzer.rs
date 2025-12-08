//! Repository Analysis Manager
//!
//! Analyzes GitHub repositories for context, including metadata, dependencies, and code patterns.

use crate::errors::{GitHubError, Result};
use crate::models::{Dependency, ProjectStructure, Repository};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Code pattern information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodePattern {
    /// Pattern name
    pub name: String,
    /// Pattern description
    pub description: String,
    /// Frequency (how often this pattern appears)
    pub frequency: u32,
    /// Example code
    pub example: Option<String>,
}

/// Codebase summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodebaseSummary {
    /// Total lines of code
    pub total_lines: u64,
    /// Number of files
    pub file_count: u32,
    /// Primary language
    pub primary_language: Option<String>,
    /// Languages used
    pub languages: Vec<String>,
    /// Key patterns found
    pub patterns: Vec<CodePattern>,
    /// Architecture overview
    pub architecture: String,
    /// Key modules/components
    pub components: Vec<String>,
}

/// Repository analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryAnalysis {
    /// Repository metadata
    pub repository: Repository,
    /// Codebase summary
    pub summary: CodebaseSummary,
    /// Analysis timestamp
    pub analyzed_at: chrono::DateTime<chrono::Utc>,
}

/// Cache entry with metadata
#[derive(Debug, Clone)]
struct CacheEntry {
    /// The cached analysis
    analysis: RepositoryAnalysis,
    /// When the cache entry was created
    created_at: chrono::DateTime<chrono::Utc>,
    /// Cache TTL in seconds (0 = no expiration)
    ttl_seconds: u64,
}

impl CacheEntry {
    /// Check if this cache entry is still valid
    fn is_valid(&self) -> bool {
        if self.ttl_seconds == 0 {
            return true;
        }
        let now = chrono::Utc::now();
        let elapsed = (now - self.created_at).num_seconds() as u64;
        elapsed < self.ttl_seconds
    }
}

/// Repository Analyzer
///
/// Analyzes GitHub repositories for context, including:
/// - Repository metadata and structure
/// - Project dependencies and versions
/// - Code patterns and conventions
/// - Codebase summaries
/// - Analysis result caching with TTL
/// - Incremental updates
pub struct RepositoryAnalyzer {
    /// Cache for analysis results with TTL
    cache: HashMap<String, CacheEntry>,
    /// Default cache TTL in seconds (3600 = 1 hour)
    default_ttl_seconds: u64,
}

impl RepositoryAnalyzer {
    /// Create a new RepositoryAnalyzer with default TTL (1 hour)
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            default_ttl_seconds: 3600,
        }
    }

    /// Create a new RepositoryAnalyzer with custom TTL
    ///
    /// # Arguments
    /// * `ttl_seconds` - Cache TTL in seconds (0 = no expiration)
    pub fn with_ttl(ttl_seconds: u64) -> Self {
        Self {
            cache: HashMap::new(),
            default_ttl_seconds: ttl_seconds,
        }
    }

    /// Set the default cache TTL
    ///
    /// # Arguments
    /// * `ttl_seconds` - Cache TTL in seconds (0 = no expiration)
    pub fn set_ttl(&mut self, ttl_seconds: u64) {
        self.default_ttl_seconds = ttl_seconds;
    }

    /// Fetch repository metadata and structure
    ///
    /// # Arguments
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    ///
    /// # Returns
    /// Repository metadata including name, owner, description, language, and structure
    ///
    /// # Errors
    /// Returns error if repository cannot be fetched
    pub async fn fetch_repository_metadata(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<Repository> {
        // Validate inputs
        if owner.is_empty() {
            return Err(GitHubError::invalid_input("Owner cannot be empty"));
        }
        if repo.is_empty() {
            return Err(GitHubError::invalid_input("Repository name cannot be empty"));
        }

        // In a real implementation, this would call the GitHub API
        // For now, we return a placeholder that demonstrates the structure
        Ok(Repository {
            name: repo.to_string(),
            owner: owner.to_string(),
            description: format!("Repository {}/{}", owner, repo),
            url: format!("https://github.com/{}/{}", owner, repo),
            language: Some("Rust".to_string()),
            dependencies: Vec::new(),
            structure: ProjectStructure {
                language: Some("Rust".to_string()),
                project_type: "library".to_string(),
                directories: vec!["src".to_string(), "tests".to_string()],
                files: vec!["Cargo.toml".to_string(), "README.md".to_string()],
            },
        })
    }

    /// Identify project dependencies and versions
    ///
    /// # Arguments
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    ///
    /// # Returns
    /// List of dependencies with their versions
    ///
    /// # Errors
    /// Returns error if dependencies cannot be identified
    pub async fn identify_dependencies(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<Vec<Dependency>> {
        // Validate inputs
        if owner.is_empty() {
            return Err(GitHubError::invalid_input("Owner cannot be empty"));
        }
        if repo.is_empty() {
            return Err(GitHubError::invalid_input("Repository name cannot be empty"));
        }

        // In a real implementation, this would parse manifest files (Cargo.toml, package.json, etc.)
        // For now, return an empty list
        Ok(Vec::new())
    }

    /// Extract code patterns and conventions
    ///
    /// # Arguments
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    ///
    /// # Returns
    /// List of code patterns found in the repository
    ///
    /// # Errors
    /// Returns error if patterns cannot be extracted
    pub async fn extract_code_patterns(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<Vec<CodePattern>> {
        // Validate inputs
        if owner.is_empty() {
            return Err(GitHubError::invalid_input("Owner cannot be empty"));
        }
        if repo.is_empty() {
            return Err(GitHubError::invalid_input("Repository name cannot be empty"));
        }

        // In a real implementation, this would analyze code files for patterns
        // For now, return an empty list
        Ok(Vec::new())
    }

    /// Generate codebase summary
    ///
    /// # Arguments
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    ///
    /// # Returns
    /// Summary of the codebase including lines of code, file count, languages, and patterns
    ///
    /// # Errors
    /// Returns error if summary cannot be generated
    pub async fn generate_codebase_summary(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<CodebaseSummary> {
        // Validate inputs
        if owner.is_empty() {
            return Err(GitHubError::invalid_input("Owner cannot be empty"));
        }
        if repo.is_empty() {
            return Err(GitHubError::invalid_input("Repository name cannot be empty"));
        }

        // In a real implementation, this would analyze the repository
        Ok(CodebaseSummary {
            total_lines: 0,
            file_count: 0,
            primary_language: Some("Rust".to_string()),
            languages: vec!["Rust".to_string()],
            patterns: Vec::new(),
            architecture: "Modular architecture".to_string(),
            components: Vec::new(),
        })
    }

    /// Perform complete repository analysis
    ///
    /// # Arguments
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    ///
    /// # Returns
    /// Complete analysis including metadata, dependencies, patterns, and summary
    ///
    /// # Errors
    /// Returns error if analysis cannot be performed
    pub async fn analyze_repository(
        &mut self,
        owner: &str,
        repo: &str,
    ) -> Result<RepositoryAnalysis> {
        // Check cache first
        let cache_key = format!("{}/{}", owner, repo);
        if let Some(entry) = self.cache.get(&cache_key) {
            if entry.is_valid() {
                return Ok(entry.analysis.clone());
            }
        }

        // Fetch metadata
        let repository = self.fetch_repository_metadata(owner, repo).await?;

        // Generate summary
        let summary = self.generate_codebase_summary(owner, repo).await?;

        let analysis = RepositoryAnalysis {
            repository,
            summary,
            analyzed_at: chrono::Utc::now(),
        };

        // Cache the result with TTL
        self.cache.insert(
            cache_key,
            CacheEntry {
                analysis: analysis.clone(),
                created_at: chrono::Utc::now(),
                ttl_seconds: self.default_ttl_seconds,
            },
        );

        Ok(analysis)
    }

    /// Perform incremental repository analysis (updates existing analysis)
    ///
    /// # Arguments
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    ///
    /// # Returns
    /// Updated analysis
    ///
    /// # Errors
    /// Returns error if analysis cannot be performed
    pub async fn update_repository_analysis(
        &mut self,
        owner: &str,
        repo: &str,
    ) -> Result<RepositoryAnalysis> {
        // Always perform fresh analysis for updates
        let cache_key = format!("{}/{}", owner, repo);
        self.cache.remove(&cache_key);
        self.analyze_repository(owner, repo).await
    }

    /// Get cached analysis result if valid
    ///
    /// # Arguments
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    ///
    /// # Returns
    /// Cached analysis if available and valid
    pub fn get_cached_analysis(&self, owner: &str, repo: &str) -> Option<RepositoryAnalysis> {
        let cache_key = format!("{}/{}", owner, repo);
        self.cache.get(&cache_key).and_then(|entry| {
            if entry.is_valid() {
                Some(entry.analysis.clone())
            } else {
                None
            }
        })
    }

    /// Get cache statistics
    ///
    /// # Returns
    /// Tuple of (total_entries, valid_entries, expired_entries)
    pub fn cache_stats(&self) -> (usize, usize, usize) {
        let total = self.cache.len();
        let valid = self.cache.values().filter(|e| e.is_valid()).count();
        let expired = total - valid;
        (total, valid, expired)
    }

    /// Clear all cache entries
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Clear specific cache entry
    ///
    /// # Arguments
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    pub fn clear_cache_entry(&mut self, owner: &str, repo: &str) {
        let cache_key = format!("{}/{}", owner, repo);
        self.cache.remove(&cache_key);
    }

    /// Remove expired cache entries
    pub fn cleanup_expired_entries(&mut self) {
        self.cache.retain(|_, entry| entry.is_valid());
    }
}

impl Default for RepositoryAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_repository_metadata_success() {
        let analyzer = RepositoryAnalyzer::new();
        let result = analyzer.fetch_repository_metadata("owner", "repo").await;
        assert!(result.is_ok());
        let repo = result.unwrap();
        assert_eq!(repo.name, "repo");
        assert_eq!(repo.owner, "owner");
    }

    #[tokio::test]
    async fn test_fetch_repository_metadata_empty_owner() {
        let analyzer = RepositoryAnalyzer::new();
        let result = analyzer.fetch_repository_metadata("", "repo").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fetch_repository_metadata_empty_repo() {
        let analyzer = RepositoryAnalyzer::new();
        let result = analyzer.fetch_repository_metadata("owner", "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_identify_dependencies_success() {
        let analyzer = RepositoryAnalyzer::new();
        let result = analyzer.identify_dependencies("owner", "repo").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_identify_dependencies_empty_owner() {
        let analyzer = RepositoryAnalyzer::new();
        let result = analyzer.identify_dependencies("", "repo").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_extract_code_patterns_success() {
        let analyzer = RepositoryAnalyzer::new();
        let result = analyzer.extract_code_patterns("owner", "repo").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_extract_code_patterns_empty_owner() {
        let analyzer = RepositoryAnalyzer::new();
        let result = analyzer.extract_code_patterns("", "repo").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_generate_codebase_summary_success() {
        let analyzer = RepositoryAnalyzer::new();
        let result = analyzer.generate_codebase_summary("owner", "repo").await;
        assert!(result.is_ok());
        let summary = result.unwrap();
        assert_eq!(summary.primary_language, Some("Rust".to_string()));
    }

    #[tokio::test]
    async fn test_generate_codebase_summary_empty_owner() {
        let analyzer = RepositoryAnalyzer::new();
        let result = analyzer.generate_codebase_summary("", "repo").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_analyze_repository_success() {
        let mut analyzer = RepositoryAnalyzer::new();
        let result = analyzer.analyze_repository("owner", "repo").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_repository_caching() {
        let mut analyzer = RepositoryAnalyzer::new();
        let result1 = analyzer.analyze_repository("owner", "repo").await;
        assert!(result1.is_ok());

        let cached = analyzer.get_cached_analysis("owner", "repo");
        assert!(cached.is_some());
    }

    #[tokio::test]
    async fn test_clear_cache() {
        let mut analyzer = RepositoryAnalyzer::new();
        let _ = analyzer.analyze_repository("owner", "repo").await;
        analyzer.clear_cache();
        let cached = analyzer.get_cached_analysis("owner", "repo");
        assert!(cached.is_none());
    }

    #[tokio::test]
    async fn test_clear_cache_entry() {
        let mut analyzer = RepositoryAnalyzer::new();
        let _ = analyzer.analyze_repository("owner", "repo").await;
        analyzer.clear_cache_entry("owner", "repo");
        let cached = analyzer.get_cached_analysis("owner", "repo");
        assert!(cached.is_none());
    }

    #[tokio::test]
    async fn test_with_ttl() {
        let analyzer = RepositoryAnalyzer::with_ttl(3600);
        assert_eq!(analyzer.default_ttl_seconds, 3600);
    }

    #[tokio::test]
    async fn test_set_ttl() {
        let mut analyzer = RepositoryAnalyzer::new();
        analyzer.set_ttl(7200);
        assert_eq!(analyzer.default_ttl_seconds, 7200);
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let mut analyzer = RepositoryAnalyzer::new();
        let _ = analyzer.analyze_repository("owner", "repo").await;
        let (total, valid, expired) = analyzer.cache_stats();
        assert_eq!(total, 1);
        assert_eq!(valid, 1);
        assert_eq!(expired, 0);
    }

    #[tokio::test]
    async fn test_update_repository_analysis() {
        let mut analyzer = RepositoryAnalyzer::new();
        let result1 = analyzer.analyze_repository("owner", "repo").await;
        assert!(result1.is_ok());

        let result2 = analyzer.update_repository_analysis("owner", "repo").await;
        assert!(result2.is_ok());
    }

    #[tokio::test]
    async fn test_cleanup_expired_entries() {
        let mut analyzer = RepositoryAnalyzer::with_ttl(0); // No expiration
        let _ = analyzer.analyze_repository("owner", "repo").await;
        analyzer.cleanup_expired_entries();
        let (total, _, _) = analyzer.cache_stats();
        assert_eq!(total, 1); // Should still be there with TTL 0
    }
}
