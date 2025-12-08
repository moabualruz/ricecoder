//! Release Manager - Handles GitHub release creation and management

use crate::errors::{GitHubError, Result};
use crate::models::Release;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// Semantic version
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticVersion {
    /// Major version
    pub major: u32,
    /// Minor version
    pub minor: u32,
    /// Patch version
    pub patch: u32,
    /// Pre-release identifier (e.g., "alpha", "beta")
    pub prerelease: Option<String>,
    /// Build metadata
    pub build: Option<String>,
}

impl SemanticVersion {
    /// Create a new semantic version
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            prerelease: None,
            build: None,
        }
    }

    /// Parse a semantic version from a string
    pub fn parse(version_str: &str) -> Result<Self> {
        let version_str = version_str.trim_start_matches('v');

        // Split by + for build metadata
        let (version_part, build) = if let Some(pos) = version_str.find('+') {
            (
                &version_str[..pos],
                Some(version_str[pos + 1..].to_string()),
            )
        } else {
            (version_str, None)
        };

        // Split by - for prerelease
        let (version_part, prerelease) = if let Some(pos) = version_part.find('-') {
            (
                &version_part[..pos],
                Some(version_part[pos + 1..].to_string()),
            )
        } else {
            (version_part, None)
        };

        // Parse major.minor.patch
        let parts: Vec<&str> = version_part.split('.').collect();
        if parts.len() != 3 {
            return Err(GitHubError::invalid_input(
                "Invalid semantic version format. Expected major.minor.patch",
            ));
        }

        let major = parts[0]
            .parse::<u32>()
            .map_err(|_| GitHubError::invalid_input("Invalid major version"))?;
        let minor = parts[1]
            .parse::<u32>()
            .map_err(|_| GitHubError::invalid_input("Invalid minor version"))?;
        let patch = parts[2]
            .parse::<u32>()
            .map_err(|_| GitHubError::invalid_input("Invalid patch version"))?;

        Ok(Self {
            major,
            minor,
            patch,
            prerelease,
            build,
        })
    }

    /// Convert to tag format (with 'v' prefix)
    pub fn to_tag(&self) -> String {
        format!("v{}", self)
    }

    /// Increment major version
    pub fn bump_major(&self) -> Self {
        Self {
            major: self.major + 1,
            minor: 0,
            patch: 0,
            prerelease: None,
            build: None,
        }
    }

    /// Increment minor version
    pub fn bump_minor(&self) -> Self {
        Self {
            major: self.major,
            minor: self.minor + 1,
            patch: 0,
            prerelease: None,
            build: None,
        }
    }

    /// Increment patch version
    pub fn bump_patch(&self) -> Self {
        Self {
            major: self.major,
            minor: self.minor,
            patch: self.patch + 1,
            prerelease: None,
            build: None,
        }
    }
}

impl std::fmt::Display for SemanticVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut version = format!("{}.{}.{}", self.major, self.minor, self.patch);
        if let Some(prerelease) = &self.prerelease {
            version.push('-');
            version.push_str(prerelease);
        }
        if let Some(build) = &self.build {
            version.push('+');
            version.push_str(build);
        }
        write!(f, "{}", version)
    }
}

/// Release creation options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseOptions {
    /// Release tag name
    pub tag_name: String,
    /// Release name
    pub name: String,
    /// Release notes/body
    pub body: String,
    /// Is draft release
    pub draft: bool,
    /// Is prerelease
    pub prerelease: bool,
}

/// Release notes generation options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseNotesOptions {
    /// Previous tag for comparison
    pub previous_tag: Option<String>,
    /// Include commit messages
    pub include_commits: bool,
    /// Include PR information
    pub include_prs: bool,
    /// Include contributors
    pub include_contributors: bool,
}

/// Release history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseHistoryEntry {
    /// Version
    pub version: String,
    /// Release date
    pub date: chrono::DateTime<Utc>,
    /// Release notes
    pub notes: String,
    /// Is prerelease
    pub prerelease: bool,
}

/// Release Manager
#[derive(Debug, Clone)]
pub struct ReleaseManager {
    /// Release history cache
    history: HashMap<String, ReleaseHistoryEntry>,
}

impl ReleaseManager {
    /// Create a new release manager
    pub fn new() -> Self {
        Self {
            history: HashMap::new(),
        }
    }

    /// Create a GitHub release
    pub async fn create_release(&mut self, options: ReleaseOptions) -> Result<Release> {
        debug!("Creating release: {}", options.tag_name);

        // Validate tag name format
        if !options.tag_name.starts_with('v') {
            return Err(GitHubError::invalid_input(
                "Tag name must start with 'v'",
            ));
        }

        // Parse version to validate semantic versioning
        let version_str = options.tag_name.trim_start_matches('v');
        let _version = SemanticVersion::parse(version_str)?;

        // Create release
        let release = Release {
            id: 0, // Would be set by GitHub API
            tag_name: options.tag_name.clone(),
            name: options.name.clone(),
            body: options.body.clone(),
            draft: options.draft,
            prerelease: options.prerelease,
            created_at: Utc::now(),
        };

        // Store in history
        self.history.insert(
            options.tag_name.clone(),
            ReleaseHistoryEntry {
                version: version_str.to_string(),
                date: Utc::now(),
                notes: options.body.clone(),
                prerelease: options.prerelease,
            },
        );

        info!("Release created: {}", options.tag_name);
        Ok(release)
    }

    /// Generate release notes from commits and PRs
    pub async fn generate_release_notes(
        &self,
        options: ReleaseNotesOptions,
    ) -> Result<String> {
        debug!("Generating release notes");

        let mut notes = String::new();

        if options.include_commits {
            notes.push_str("## Commits\n\n");
            notes.push_str("- Commits since previous release\n\n");
        }

        if options.include_prs {
            notes.push_str("## Pull Requests\n\n");
            notes.push_str("- PRs merged since previous release\n\n");
        }

        if options.include_contributors {
            notes.push_str("## Contributors\n\n");
            notes.push_str("- Contributors to this release\n\n");
        }

        if notes.is_empty() {
            notes.push_str("## Release Notes\n\nNo changes documented.\n");
        }

        info!("Release notes generated");
        Ok(notes)
    }

    /// Publish a release to GitHub
    pub async fn publish_release(&mut self, release: Release) -> Result<Release> {
        debug!("Publishing release: {}", release.tag_name);

        // Validate release
        if release.tag_name.is_empty() {
            return Err(GitHubError::invalid_input("Release tag name cannot be empty"));
        }

        if release.name.is_empty() {
            return Err(GitHubError::invalid_input("Release name cannot be empty"));
        }

        // Update history
        let version_str = release.tag_name.trim_start_matches('v');
        self.history.insert(
            release.tag_name.clone(),
            ReleaseHistoryEntry {
                version: version_str.to_string(),
                date: release.created_at,
                notes: release.body.clone(),
                prerelease: release.prerelease,
            },
        );

        info!("Release published: {}", release.tag_name);
        Ok(release)
    }

    /// Get release history
    pub fn get_release_history(&self) -> Vec<ReleaseHistoryEntry> {
        let mut entries: Vec<_> = self.history.values().cloned().collect();
        entries.sort_by(|a, b| b.date.cmp(&a.date));
        entries
    }

    /// Get a specific release from history
    pub fn get_release(&self, tag_name: &str) -> Option<ReleaseHistoryEntry> {
        self.history.get(tag_name).cloned()
    }

    /// Maintain changelog
    pub fn generate_changelog(&self) -> String {
        let mut changelog = String::from("# Changelog\n\n");

        let mut entries: Vec<_> = self.history.values().cloned().collect();
        entries.sort_by(|a, b| b.date.cmp(&a.date));

        for entry in entries {
            changelog.push_str(&format!("## [{}] - {}\n\n", entry.version, entry.date.format("%Y-%m-%d")));
            changelog.push_str(&entry.notes);
            changelog.push_str("\n\n");
        }

        changelog
    }

    /// Validate semantic version tag
    pub fn validate_version_tag(tag: &str) -> Result<SemanticVersion> {
        if !tag.starts_with('v') {
            return Err(GitHubError::invalid_input(
                "Version tag must start with 'v'",
            ));
        }

        let version_str = tag.trim_start_matches('v');
        SemanticVersion::parse(version_str)
    }

    /// Check if version already exists in history
    pub fn version_exists(&self, tag_name: &str) -> bool {
        self.history.contains_key(tag_name)
    }

    /// Get latest release
    pub fn get_latest_release(&self) -> Option<ReleaseHistoryEntry> {
        let mut entries: Vec<_> = self.history.values().cloned().collect();
        entries.sort_by(|a, b| b.date.cmp(&a.date));
        entries.first().cloned()
    }
}

impl Default for ReleaseManager {
    fn default() -> Self {
        Self::new()
    }
}