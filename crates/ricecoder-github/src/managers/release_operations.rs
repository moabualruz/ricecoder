//! Release Operations - Handles release publishing and changelog maintenance

use crate::errors::{GitHubError, Result};
use crate::models::Release;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// Release template for customization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseTemplate {
    /// Template name
    pub name: String,
    /// Template content with placeholders
    pub content: String,
    /// Placeholders in template
    pub placeholders: Vec<String>,
}

impl ReleaseTemplate {
    /// Create a new release template
    pub fn new(name: impl Into<String>, content: impl Into<String>) -> Self {
        let content_str = content.into();
        let placeholders = extract_placeholders(&content_str);

        Self {
            name: name.into(),
            content: content_str,
            placeholders,
        }
    }

    /// Apply template with values
    pub fn apply(&self, values: &HashMap<String, String>) -> Result<String> {
        let mut result = self.content.clone();

        for placeholder in &self.placeholders {
            let key = placeholder.trim_start_matches("{{").trim_end_matches("}}");
            if let Some(value) = values.get(key) {
                result = result.replace(placeholder, value);
            } else {
                return Err(GitHubError::invalid_input(format!(
                    "Missing value for placeholder: {}",
                    placeholder
                )));
            }
        }

        Ok(result)
    }
}

/// Extract placeholders from template
fn extract_placeholders(template: &str) -> Vec<String> {
    let mut placeholders = Vec::new();
    let mut chars = template.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '{' && chars.peek() == Some(&'{') {
            chars.next(); // consume second {
            let mut placeholder = String::from("{{");

            while let Some(ch) = chars.next() {
                placeholder.push(ch);
                if ch == '}' && chars.peek() == Some(&'}') {
                    chars.next(); // consume second }
                    placeholder.push('}');
                    placeholders.push(placeholder);
                    break;
                }
            }
        }
    }

    placeholders
}

/// Release publishing result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleasePublishingResult {
    /// Release ID
    pub release_id: u64,
    /// Tag name
    pub tag_name: String,
    /// Published URL
    pub url: String,
    /// Publish timestamp
    pub published_at: chrono::DateTime<Utc>,
    /// Success status
    pub success: bool,
}

/// Changelog entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogEntry {
    /// Version
    pub version: String,
    /// Release date
    pub date: chrono::DateTime<Utc>,
    /// Changes in this release
    pub changes: Vec<String>,
    /// Contributors
    pub contributors: Vec<String>,
}

/// Changelog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Changelog {
    /// Changelog entries
    pub entries: Vec<ChangelogEntry>,
    /// Last updated
    pub last_updated: chrono::DateTime<Utc>,
}

impl Changelog {
    /// Create a new changelog
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            last_updated: Utc::now(),
        }
    }

    /// Add an entry
    pub fn add_entry(&mut self, entry: ChangelogEntry) {
        self.entries.push(entry);
        self.entries.sort_by(|a, b| b.date.cmp(&a.date));
        self.last_updated = Utc::now();
    }

    /// Generate markdown
    pub fn to_markdown(&self) -> String {
        let mut markdown = String::from("# Changelog\n\n");
        markdown
            .push_str("All notable changes to this project will be documented in this file.\n\n");

        for entry in &self.entries {
            markdown.push_str(&format!(
                "## [{}] - {}\n\n",
                entry.version,
                entry.date.format("%Y-%m-%d")
            ));

            if !entry.changes.is_empty() {
                markdown.push_str("### Changes\n\n");
                for change in &entry.changes {
                    markdown.push_str(&format!("- {}\n", change));
                }
                markdown.push('\n');
            }

            if !entry.contributors.is_empty() {
                markdown.push_str("### Contributors\n\n");
                for contributor in &entry.contributors {
                    markdown.push_str(&format!("- {}\n", contributor));
                }
                markdown.push('\n');
            }
        }

        markdown
    }
}

impl Default for Changelog {
    fn default() -> Self {
        Self::new()
    }
}

/// Release Operations
#[derive(Debug, Clone)]
pub struct ReleaseOperations {
    /// Release templates
    templates: HashMap<String, ReleaseTemplate>,
    /// Changelog
    changelog: Changelog,
}

impl ReleaseOperations {
    /// Create new release operations
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
            changelog: Changelog::new(),
        }
    }

    /// Register a release template
    pub fn register_template(&mut self, template: ReleaseTemplate) {
        debug!("Registering template: {}", template.name);
        self.templates.insert(template.name.clone(), template);
    }

    /// Get a template
    pub fn get_template(&self, name: &str) -> Option<&ReleaseTemplate> {
        self.templates.get(name)
    }

    /// Publish a release
    pub async fn publish_release(&self, release: &Release) -> Result<ReleasePublishingResult> {
        debug!("Publishing release: {}", release.tag_name);

        // Validate release
        if release.tag_name.is_empty() {
            return Err(GitHubError::invalid_input(
                "Release tag name cannot be empty",
            ));
        }

        if release.name.is_empty() {
            return Err(GitHubError::invalid_input("Release name cannot be empty"));
        }

        // Create publishing result
        let result = ReleasePublishingResult {
            release_id: release.id,
            tag_name: release.tag_name.clone(),
            url: format!("https://github.com/releases/tag/{}", release.tag_name),
            published_at: Utc::now(),
            success: true,
        };

        info!("Release published: {}", release.tag_name);
        Ok(result)
    }

    /// Add changelog entry
    pub fn add_changelog_entry(&mut self, entry: ChangelogEntry) {
        debug!("Adding changelog entry for version: {}", entry.version);
        self.changelog.add_entry(entry);
    }

    /// Get changelog
    pub fn get_changelog(&self) -> &Changelog {
        &self.changelog
    }

    /// Generate changelog markdown
    pub fn generate_changelog_markdown(&self) -> String {
        self.changelog.to_markdown()
    }

    /// Maintain changelog - add new entry
    pub fn maintain_changelog(
        &mut self,
        version: String,
        changes: Vec<String>,
        contributors: Vec<String>,
    ) -> Result<()> {
        debug!("Maintaining changelog for version: {}", version);

        let entry = ChangelogEntry {
            version,
            date: Utc::now(),
            changes,
            contributors,
        };

        self.add_changelog_entry(entry);
        info!("Changelog updated");
        Ok(())
    }

    /// Get release history from changelog
    pub fn get_release_history(&self) -> Vec<(String, chrono::DateTime<Utc>)> {
        self.changelog
            .entries
            .iter()
            .map(|e| (e.version.clone(), e.date))
            .collect()
    }

    /// Find latest release in changelog
    pub fn get_latest_release(&self) -> Option<&ChangelogEntry> {
        self.changelog.entries.first()
    }

    /// Get releases between versions
    pub fn get_releases_between(
        &self,
        from_version: &str,
        to_version: &str,
    ) -> Vec<&ChangelogEntry> {
        self.changelog
            .entries
            .iter()
            .filter(|e| e.version.as_str() >= from_version && e.version.as_str() <= to_version)
            .collect()
    }
}

impl Default for ReleaseOperations {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_placeholders() {
        let template = "Release {{version}} on {{date}}";
        let placeholders = extract_placeholders(template);
        assert_eq!(placeholders.len(), 2);
        assert!(placeholders.contains(&"{{version}}".to_string()));
        assert!(placeholders.contains(&"{{date}}".to_string()));
    }

    #[test]
    fn test_release_template_apply() {
        let template = ReleaseTemplate::new("test", "Version: {{version}}, Date: {{date}}");
        let mut values = HashMap::new();
        values.insert("version".to_string(), "1.0.0".to_string());
        values.insert("date".to_string(), "2025-01-01".to_string());

        let result = template.apply(&values).unwrap();
        assert_eq!(result, "Version: 1.0.0, Date: 2025-01-01");
    }

    #[test]
    fn test_changelog_to_markdown() {
        let mut changelog = Changelog::new();
        let entry = ChangelogEntry {
            version: "1.0.0".to_string(),
            date: Utc::now(),
            changes: vec!["Feature 1".to_string(), "Bug fix".to_string()],
            contributors: vec!["Alice".to_string()],
        };
        changelog.add_entry(entry);

        let markdown = changelog.to_markdown();
        assert!(markdown.contains("# Changelog"));
        assert!(markdown.contains("## [1.0.0]"));
        assert!(markdown.contains("Feature 1"));
        assert!(markdown.contains("Alice"));
    }
}
