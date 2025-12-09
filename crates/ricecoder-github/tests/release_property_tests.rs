//! Property-based tests for Release Management
//! **Feature: ricecoder-github, Property 41-45: Release Management**

use proptest::prelude::*;
use ricecoder_github::{
    ReleaseManager, ReleaseOptions, SemanticVersion, ReleaseOperations, ChangelogEntry,
};
use chrono::Utc;

// Strategy for generating valid semantic versions
fn semantic_version_strategy() -> impl Strategy<Value = (u32, u32, u32)> {
    (0u32..100, 0u32..100, 0u32..100)
}

// Strategy for generating valid tag names
fn tag_name_strategy() -> impl Strategy<Value = String> {
    semantic_version_strategy().prop_map(|(major, minor, patch)| {
        format!("v{}.{}.{}", major, minor, patch)
    })
}

// Strategy for generating release names
fn release_name_strategy() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9 \-]+"
        .prop_map(|s| format!("Release {}", s))
}

// Strategy for generating release notes
fn release_notes_strategy() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9 \-\.\,\n]+"
        .prop_map(|s| format!("## Changes\n\n{}", s))
}

proptest! {
    /// Property 41: Release Creation
    /// *For any* release request, the system SHALL create a GitHub release with the specified version tag.
    /// **Validates: Requirements 9.1**
    #[test]
    fn prop_release_creation_integrity(
        tag_name in tag_name_strategy(),
        name in release_name_strategy(),
        body in release_notes_strategy(),
    ) {
        let mut manager = ReleaseManager::new();
        let options = ReleaseOptions {
            tag_name: tag_name.clone(),
            name: name.clone(),
            body: body.clone(),
            draft: false,
            prerelease: false,
        };

        // Create release
        let result = tokio::runtime::Runtime::new().unwrap().block_on(async {
            manager.create_release(options).await
        });

        // Verify release was created successfully
        prop_assert!(result.is_ok());
        let release = result.unwrap();

        // Verify release properties match input
        prop_assert_eq!(release.tag_name, tag_name);
        prop_assert_eq!(release.name, name);
        prop_assert_eq!(release.body, body);
        prop_assert!(!release.draft);
        prop_assert!(!release.prerelease);
    }

    /// Property 42: Release Notes Generation
    /// *For any* release, the system SHALL generate release notes from commits and PRs since the last release.
    /// **Validates: Requirements 9.2**
    #[test]
    fn prop_release_notes_generation(
        include_commits in any::<bool>(),
        include_prs in any::<bool>(),
        include_contributors in any::<bool>(),
    ) {
        let manager = ReleaseManager::new();
        let options = ricecoder_github::ReleaseNotesOptions {
            previous_tag: None,
            include_commits,
            include_prs,
            include_contributors,
        };

        // Generate release notes
        let result = tokio::runtime::Runtime::new().unwrap().block_on(async {
            manager.generate_release_notes(options).await
        });

        // Verify notes were generated
        prop_assert!(result.is_ok());
        let notes = result.unwrap();

        // Verify notes are non-empty
        prop_assert!(!notes.is_empty());

        // Verify notes contain expected sections based on options
        if include_commits {
            prop_assert!(notes.contains("Commits"));
        }
        if include_prs {
            prop_assert!(notes.contains("Pull Requests"));
        }
        if include_contributors {
            prop_assert!(notes.contains("Contributors"));
        }
    }

    /// Property 43: Semantic Version Tagging
    /// *For any* release, the created tag SHALL follow semantic versioning format (major.minor.patch).
    /// **Validates: Requirements 9.3**
    #[test]
    fn prop_semantic_version_tagging(
        major in 0u32..100,
        minor in 0u32..100,
        patch in 0u32..100,
    ) {
        let version = SemanticVersion::new(major, minor, patch);
        let tag = version.to_tag();

        // Verify tag starts with 'v'
        prop_assert!(tag.starts_with('v'));

        // Verify tag format is valid
        let version_part = tag.trim_start_matches('v');
        let parts: Vec<&str> = version_part.split('.').collect();
        prop_assert_eq!(parts.len(), 3);

        // Verify each part is a valid number
        prop_assert_eq!(parts[0].parse::<u32>().unwrap(), major);
        prop_assert_eq!(parts[1].parse::<u32>().unwrap(), minor);
        prop_assert_eq!(parts[2].parse::<u32>().unwrap(), patch);

        // Verify parsing round-trip
        let parsed = SemanticVersion::parse(version_part).unwrap();
        prop_assert_eq!(parsed.major, major);
        prop_assert_eq!(parsed.minor, minor);
        prop_assert_eq!(parsed.patch, patch);
    }

    /// Property 44: Release Publishing
    /// *For any* release, the system SHALL publish the release to GitHub.
    /// **Validates: Requirements 9.4**
    #[test]
    fn prop_release_publishing(
        tag_name in tag_name_strategy(),
        name in release_name_strategy(),
        body in release_notes_strategy(),
    ) {
        let mut manager = ReleaseManager::new();
        let release = ricecoder_github::Release {
            id: 0,
            tag_name: tag_name.clone(),
            name: name.clone(),
            body: body.clone(),
            draft: false,
            prerelease: false,
            created_at: Utc::now(),
        };

        // Publish release
        let result = tokio::runtime::Runtime::new().unwrap().block_on(async {
            manager.publish_release(release.clone()).await
        });

        // Verify publishing succeeded
        prop_assert!(result.is_ok());
        let published = result.unwrap();

        // Verify published release matches input
        prop_assert_eq!(published.tag_name, tag_name);
        prop_assert_eq!(published.name, name);
        prop_assert_eq!(published.body, body);
    }

    /// Property 45: Release History Maintenance
    /// *For any* repository, the system SHALL maintain release history and changelog.
    /// **Validates: Requirements 9.5**
    #[test]
    fn prop_release_history_maintenance(
        versions in prop::collection::vec(tag_name_strategy(), 1..10),
    ) {
        let mut operations = ReleaseOperations::new();

        // Add entries for each version
        for (idx, version) in versions.iter().enumerate() {
            let version_str = version.trim_start_matches('v').to_string();
            let entry = ChangelogEntry {
                version: version_str,
                date: Utc::now(),
                changes: vec![format!("Change {}", idx)],
                contributors: vec![format!("Contributor {}", idx)],
            };
            operations.add_changelog_entry(entry);
        }

        // Verify history is maintained
        let history = operations.get_release_history();
        prop_assert_eq!(history.len(), versions.len());

        // Verify changelog can be generated
        let changelog = operations.generate_changelog_markdown();
        prop_assert!(!changelog.is_empty());
        prop_assert!(changelog.contains("# Changelog"));

        // Verify latest release is accessible
        let latest = operations.get_latest_release();
        prop_assert!(latest.is_some());
    }

    /// Property: Semantic Version Parsing Round-Trip
    /// *For any* semantic version, parsing and converting back to string should produce equivalent version.
    /// **Validates: Requirements 9.3**
    #[test]
    fn prop_semantic_version_round_trip(
        major in 0u32..100,
        minor in 0u32..100,
        patch in 0u32..100,
    ) {
        let original = SemanticVersion::new(major, minor, patch);
        let version_str = original.to_string();
        let parsed = SemanticVersion::parse(&version_str).unwrap();

        prop_assert_eq!(parsed.major, original.major);
        prop_assert_eq!(parsed.minor, original.minor);
        prop_assert_eq!(parsed.patch, original.patch);
        prop_assert_eq!(parsed.prerelease, original.prerelease);
        prop_assert_eq!(parsed.build, original.build);
    }

    /// Property: Version Bumping Consistency
    /// *For any* semantic version, bumping major/minor/patch should produce correct new version.
    /// **Validates: Requirements 9.3**
    #[test]
    fn prop_version_bumping(
        major in 0u32..100,
        minor in 0u32..100,
        patch in 0u32..100,
    ) {
        let version = SemanticVersion::new(major, minor, patch);

        // Test major bump
        let bumped_major = version.bump_major();
        prop_assert_eq!(bumped_major.major, major + 1);
        prop_assert_eq!(bumped_major.minor, 0);
        prop_assert_eq!(bumped_major.patch, 0);

        // Test minor bump
        let bumped_minor = version.bump_minor();
        prop_assert_eq!(bumped_minor.major, major);
        prop_assert_eq!(bumped_minor.minor, minor + 1);
        prop_assert_eq!(bumped_minor.patch, 0);

        // Test patch bump
        let bumped_patch = version.bump_patch();
        prop_assert_eq!(bumped_patch.major, major);
        prop_assert_eq!(bumped_patch.minor, minor);
        prop_assert_eq!(bumped_patch.patch, patch + 1);
    }

    /// Property: Release Tag Validation
    /// *For any* valid tag name, validation should succeed and produce correct version.
    /// **Validates: Requirements 9.3**
    #[test]
    fn prop_release_tag_validation(
        major in 0u32..100,
        minor in 0u32..100,
        patch in 0u32..100,
    ) {
        let tag = format!("v{}.{}.{}", major, minor, patch);
        let result = ReleaseManager::validate_version_tag(&tag);

        prop_assert!(result.is_ok());
        let version = result.unwrap();
        prop_assert_eq!(version.major, major);
        prop_assert_eq!(version.minor, minor);
        prop_assert_eq!(version.patch, patch);
    }

    /// Property: Invalid Tag Rejection
    /// *For any* invalid tag name, validation should fail.
    /// **Validates: Requirements 9.3**
    #[test]
    fn prop_invalid_tag_rejection(
        invalid_tag in r"[a-z0-9\-\.]+",
    ) {
        // Skip if it happens to be valid
        if invalid_tag.starts_with('v') {
            return Ok(());
        }

        let result = ReleaseManager::validate_version_tag(&invalid_tag);
        prop_assert!(result.is_err());
    }
}
