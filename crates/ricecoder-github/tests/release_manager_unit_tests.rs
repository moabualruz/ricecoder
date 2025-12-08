//! Unit tests for Release Manager
//! Tests release creation, notes generation, and version tagging

use ricecoder_github::{ReleaseManager, ReleaseOptions, SemanticVersion, ReleaseOperations, ChangelogEntry};
use chrono::Utc;

#[tokio::test]
async fn test_release_creation_success() {
    let mut manager = ReleaseManager::new();
    let options = ReleaseOptions {
        tag_name: "v1.0.0".to_string(),
        name: "Release 1.0.0".to_string(),
        body: "## Changes\n\n- Feature 1\n- Bug fix".to_string(),
        draft: false,
        prerelease: false,
    };

    let result = manager.create_release(options).await;
    assert!(result.is_ok());

    let release = result.unwrap();
    assert_eq!(release.tag_name, "v1.0.0");
    assert_eq!(release.name, "Release 1.0.0");
    assert!(!release.draft);
    assert!(!release.prerelease);
}

#[tokio::test]
async fn test_release_creation_with_prerelease() {
    let mut manager = ReleaseManager::new();
    let options = ReleaseOptions {
        tag_name: "v1.0.0-beta".to_string(),
        name: "Release 1.0.0 Beta".to_string(),
        body: "Beta release".to_string(),
        draft: false,
        prerelease: true,
    };

    let result = manager.create_release(options).await;
    assert!(result.is_ok());

    let release = result.unwrap();
    assert!(release.prerelease);
}

#[tokio::test]
async fn test_release_creation_with_draft() {
    let mut manager = ReleaseManager::new();
    let options = ReleaseOptions {
        tag_name: "v1.0.0".to_string(),
        name: "Draft Release".to_string(),
        body: "Draft content".to_string(),
        draft: true,
        prerelease: false,
    };

    let result = manager.create_release(options).await;
    assert!(result.is_ok());

    let release = result.unwrap();
    assert!(release.draft);
}

#[tokio::test]
async fn test_release_creation_invalid_tag() {
    let mut manager = ReleaseManager::new();
    let options = ReleaseOptions {
        tag_name: "1.0.0".to_string(), // Missing 'v' prefix
        name: "Invalid Release".to_string(),
        body: "Invalid tag".to_string(),
        draft: false,
        prerelease: false,
    };

    let result = manager.create_release(options).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_release_notes_generation_with_commits() {
    let manager = ReleaseManager::new();
    let options = ricecoder_github::ReleaseNotesOptions {
        previous_tag: None,
        include_commits: true,
        include_prs: false,
        include_contributors: false,
    };

    let result = manager.generate_release_notes(options).await;
    assert!(result.is_ok());

    let notes = result.unwrap();
    assert!(notes.contains("Commits"));
    assert!(!notes.is_empty());
}

#[tokio::test]
async fn test_release_notes_generation_with_prs() {
    let manager = ReleaseManager::new();
    let options = ricecoder_github::ReleaseNotesOptions {
        previous_tag: None,
        include_commits: false,
        include_prs: true,
        include_contributors: false,
    };

    let result = manager.generate_release_notes(options).await;
    assert!(result.is_ok());

    let notes = result.unwrap();
    assert!(notes.contains("Pull Requests"));
}

#[tokio::test]
async fn test_release_notes_generation_with_contributors() {
    let manager = ReleaseManager::new();
    let options = ricecoder_github::ReleaseNotesOptions {
        previous_tag: None,
        include_commits: false,
        include_prs: false,
        include_contributors: true,
    };

    let result = manager.generate_release_notes(options).await;
    assert!(result.is_ok());

    let notes = result.unwrap();
    assert!(notes.contains("Contributors"));
}

#[tokio::test]
async fn test_release_notes_generation_all_options() {
    let manager = ReleaseManager::new();
    let options = ricecoder_github::ReleaseNotesOptions {
        previous_tag: None,
        include_commits: true,
        include_prs: true,
        include_contributors: true,
    };

    let result = manager.generate_release_notes(options).await;
    assert!(result.is_ok());

    let notes = result.unwrap();
    assert!(notes.contains("Commits"));
    assert!(notes.contains("Pull Requests"));
    assert!(notes.contains("Contributors"));
}

#[tokio::test]
async fn test_release_publishing_success() {
    let mut manager = ReleaseManager::new();
    let release = ricecoder_github::Release {
        id: 123,
        tag_name: "v1.0.0".to_string(),
        name: "Release 1.0.0".to_string(),
        body: "Release notes".to_string(),
        draft: false,
        prerelease: false,
        created_at: Utc::now(),
    };

    let result = manager.publish_release(release).await;
    assert!(result.is_ok());

    let published = result.unwrap();
    assert_eq!(published.tag_name, "v1.0.0");
}

#[tokio::test]
async fn test_release_publishing_empty_tag() {
    let mut manager = ReleaseManager::new();
    let release = ricecoder_github::Release {
        id: 123,
        tag_name: "".to_string(),
        name: "Release".to_string(),
        body: "Notes".to_string(),
        draft: false,
        prerelease: false,
        created_at: Utc::now(),
    };

    let result = manager.publish_release(release).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_release_publishing_empty_name() {
    let mut manager = ReleaseManager::new();
    let release = ricecoder_github::Release {
        id: 123,
        tag_name: "v1.0.0".to_string(),
        name: "".to_string(),
        body: "Notes".to_string(),
        draft: false,
        prerelease: false,
        created_at: Utc::now(),
    };

    let result = manager.publish_release(release).await;
    assert!(result.is_err());
}

#[test]
fn test_semantic_version_parsing() {
    let version = SemanticVersion::parse("1.2.3").unwrap();
    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 2);
    assert_eq!(version.patch, 3);
    assert_eq!(version.prerelease, None);
    assert_eq!(version.build, None);
}

#[test]
fn test_semantic_version_parsing_with_prerelease() {
    let version = SemanticVersion::parse("1.2.3-alpha").unwrap();
    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 2);
    assert_eq!(version.patch, 3);
    assert_eq!(version.prerelease, Some("alpha".to_string()));
}

#[test]
fn test_semantic_version_parsing_with_build() {
    let version = SemanticVersion::parse("1.2.3+build.123").unwrap();
    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 2);
    assert_eq!(version.patch, 3);
    assert_eq!(version.build, Some("build.123".to_string()));
}

#[test]
fn test_semantic_version_parsing_with_v_prefix() {
    let version = SemanticVersion::parse("v1.2.3").unwrap();
    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 2);
    assert_eq!(version.patch, 3);
}

#[test]
fn test_semantic_version_to_string() {
    let version = SemanticVersion::new(1, 2, 3);
    assert_eq!(version.to_string(), "1.2.3");
}

#[test]
fn test_semantic_version_to_tag() {
    let version = SemanticVersion::new(1, 2, 3);
    assert_eq!(version.to_tag(), "v1.2.3");
}

#[test]
fn test_semantic_version_bump_major() {
    let version = SemanticVersion::new(1, 2, 3);
    let bumped = version.bump_major();
    assert_eq!(bumped.major, 2);
    assert_eq!(bumped.minor, 0);
    assert_eq!(bumped.patch, 0);
}

#[test]
fn test_semantic_version_bump_minor() {
    let version = SemanticVersion::new(1, 2, 3);
    let bumped = version.bump_minor();
    assert_eq!(bumped.major, 1);
    assert_eq!(bumped.minor, 3);
    assert_eq!(bumped.patch, 0);
}

#[test]
fn test_semantic_version_bump_patch() {
    let version = SemanticVersion::new(1, 2, 3);
    let bumped = version.bump_patch();
    assert_eq!(bumped.major, 1);
    assert_eq!(bumped.minor, 2);
    assert_eq!(bumped.patch, 4);
}

#[test]
fn test_release_manager_get_release_history() {
    let mut manager = ReleaseManager::new();

    // Create multiple releases
    let options1 = ReleaseOptions {
        tag_name: "v1.0.0".to_string(),
        name: "Release 1.0.0".to_string(),
        body: "First release".to_string(),
        draft: false,
        prerelease: false,
    };

    let options2 = ReleaseOptions {
        tag_name: "v1.1.0".to_string(),
        name: "Release 1.1.0".to_string(),
        body: "Second release".to_string(),
        draft: false,
        prerelease: false,
    };

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        manager.create_release(options1).await.unwrap();
        manager.create_release(options2).await.unwrap();
    });

    let history = manager.get_release_history();
    assert_eq!(history.len(), 2);
}

#[test]
fn test_release_manager_get_latest_release() {
    let mut manager = ReleaseManager::new();

    let options = ReleaseOptions {
        tag_name: "v1.0.0".to_string(),
        name: "Release 1.0.0".to_string(),
        body: "Release".to_string(),
        draft: false,
        prerelease: false,
    };

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        manager.create_release(options).await.unwrap();
    });

    let latest = manager.get_latest_release();
    assert!(latest.is_some());
    assert_eq!(latest.unwrap().version, "1.0.0");
}

#[test]
fn test_release_manager_version_exists() {
    let mut manager = ReleaseManager::new();

    let options = ReleaseOptions {
        tag_name: "v1.0.0".to_string(),
        name: "Release 1.0.0".to_string(),
        body: "Release".to_string(),
        draft: false,
        prerelease: false,
    };

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        manager.create_release(options).await.unwrap();
    });

    assert!(manager.version_exists("v1.0.0"));
    assert!(!manager.version_exists("v2.0.0"));
}

#[test]
fn test_release_manager_validate_version_tag() {
    let result = ReleaseManager::validate_version_tag("v1.2.3");
    assert!(result.is_ok());

    let version = result.unwrap();
    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 2);
    assert_eq!(version.patch, 3);
}

#[test]
fn test_release_manager_validate_version_tag_invalid() {
    let result = ReleaseManager::validate_version_tag("1.2.3");
    assert!(result.is_err());
}

#[test]
fn test_release_operations_add_changelog_entry() {
    let mut operations = ReleaseOperations::new();

    let entry = ChangelogEntry {
        version: "1.0.0".to_string(),
        date: Utc::now(),
        changes: vec!["Feature 1".to_string()],
        contributors: vec!["Alice".to_string()],
    };

    operations.add_changelog_entry(entry);

    let history = operations.get_release_history();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].0, "1.0.0");
}

#[test]
fn test_release_operations_generate_changelog() {
    let mut operations = ReleaseOperations::new();

    let entry = ChangelogEntry {
        version: "1.0.0".to_string(),
        date: Utc::now(),
        changes: vec!["Feature 1".to_string()],
        contributors: vec!["Alice".to_string()],
    };

    operations.add_changelog_entry(entry);

    let changelog = operations.generate_changelog_markdown();
    assert!(changelog.contains("# Changelog"));
    assert!(changelog.contains("1.0.0"));
    assert!(changelog.contains("Feature 1"));
    assert!(changelog.contains("Alice"));
}

#[test]
fn test_release_operations_get_latest_release() {
    let mut operations = ReleaseOperations::new();

    let entry = ChangelogEntry {
        version: "1.0.0".to_string(),
        date: Utc::now(),
        changes: vec!["Feature 1".to_string()],
        contributors: vec![],
    };

    operations.add_changelog_entry(entry);

    let latest = operations.get_latest_release();
    assert!(latest.is_some());
    assert_eq!(latest.unwrap().version, "1.0.0");
}

#[tokio::test]
async fn test_release_operations_publish_release() {
    let operations = ReleaseOperations::new();

    let release = ricecoder_github::Release {
        id: 123,
        tag_name: "v1.0.0".to_string(),
        name: "Release 1.0.0".to_string(),
        body: "Release notes".to_string(),
        draft: false,
        prerelease: false,
        created_at: Utc::now(),
    };

    let result = operations.publish_release(&release).await;
    assert!(result.is_ok());

    let published = result.unwrap();
    assert_eq!(published.tag_name, "v1.0.0");
    assert!(published.success);
}
