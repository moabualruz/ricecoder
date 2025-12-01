//! Property-based tests for storage relocation
//! **Feature: ricecoder-storage, Property 11: Relocation Round-Trip**
//! **Validates: Requirements 6.5**

use proptest::prelude::*;
use ricecoder_storage::RelocationService;
use std::fs;
use tempfile::TempDir;

/// Strategy for generating valid file names
fn file_name_strategy() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9_\-]{1,10}"
        .prop_map(|s| format!("{}.txt", s))
        .prop_filter("Name should be non-empty", |s| !s.is_empty())
}

/// Strategy for generating valid file content
fn file_content_strategy() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9 \n\t]*"
        .prop_filter("Content should be valid", |s| !s.is_empty())
}

/// Strategy for generating file structures with unique names
fn file_structure_strategy() -> impl Strategy<Value = Vec<(String, String)>> {
    prop::collection::vec((file_name_strategy(), file_content_strategy()), 1..10)
        .prop_map(|mut files| {
            // Ensure unique file names by adding index
            for (i, (name, _)) in files.iter_mut().enumerate() {
                let base = name.trim_end_matches(".txt");
                *name = format!("{}_{}.txt", base, i);
            }
            files
        })
}

/// Property 11: Relocation Round-Trip
/// For any storage with data, relocating to a new path should move all data
/// to the new location, and all data should be accessible from the new location
/// with identical content.
#[test]
fn prop_relocation_roundtrip() {
    proptest!(|(files in file_structure_strategy())| {
        let source_dir = TempDir::new().expect("Failed to create source temp dir");
        let target_parent = TempDir::new().expect("Failed to create target parent temp dir");

        // Create source files
        for (name, content) in &files {
            fs::write(source_dir.path().join(name), content)
                .expect("Failed to write source file");
        }

        let source_path = source_dir.path().to_path_buf();
        let target_path = target_parent.path().join("relocated_storage");

        // Perform relocation
        RelocationService::relocate(&source_path, &target_path)
            .expect("Relocation should succeed");

        // Verify all files exist in target with identical content
        for (name, content) in &files {
            let target_file = target_path.join(name);
            assert!(
                target_file.exists(),
                "File {} should exist in target location",
                name
            );

            let retrieved_content = fs::read_to_string(&target_file)
                .expect("Failed to read target file");

            assert_eq!(
                &retrieved_content, content,
                "Content of {} should be identical after relocation",
                name
            );
        }
    });
}

/// Property: Relocation preserves directory structure
/// Relocating storage with nested directories should preserve the structure
#[test]
fn prop_relocation_preserves_structure() {
    proptest!(|(files in file_structure_strategy())| {
        let source_dir = TempDir::new().expect("Failed to create source temp dir");
        let target_parent = TempDir::new().expect("Failed to create target parent temp dir");

        // Create nested structure
        fs::create_dir(source_dir.path().join("subdir1")).expect("Failed to create subdir1");
        fs::create_dir(source_dir.path().join("subdir2")).expect("Failed to create subdir2");

        // Create files in root and subdirectories
        for (i, (name, content)) in files.iter().enumerate() {
            let subdir = if i % 2 == 0 { "subdir1" } else { "subdir2" };
            let file_path = source_dir.path().join(subdir).join(name);
            fs::write(&file_path, content)
                .expect("Failed to write file in subdir");
        }

        let source_path = source_dir.path().to_path_buf();
        let target_path = target_parent.path().join("relocated_storage");

        // Perform relocation
        RelocationService::relocate(&source_path, &target_path)
            .expect("Relocation should succeed");

        // Verify directory structure is preserved
        assert!(
            target_path.join("subdir1").exists(),
            "subdir1 should exist in target"
        );
        assert!(
            target_path.join("subdir2").exists(),
            "subdir2 should exist in target"
        );

        // Verify files are in correct subdirectories
        for (i, (name, content)) in files.iter().enumerate() {
            let subdir = if i % 2 == 0 { "subdir1" } else { "subdir2" };
            let target_file = target_path.join(subdir).join(name);

            assert!(
                target_file.exists(),
                "File {} should exist in {} after relocation",
                name,
                subdir
            );

            let retrieved_content = fs::read_to_string(&target_file)
                .expect("Failed to read target file");

            assert_eq!(
                &retrieved_content, content,
                "Content should be preserved in subdirectory"
            );
        }
    });
}

/// Property: Relocation is idempotent for file count
/// The number of files in source and target should be identical after relocation
#[test]
fn prop_relocation_file_count_preserved() {
    proptest!(|(files in file_structure_strategy())| {
        let source_dir = TempDir::new().expect("Failed to create source temp dir");
        let target_parent = TempDir::new().expect("Failed to create target parent temp dir");

        // Create source files
        for (name, content) in &files {
            fs::write(source_dir.path().join(name), content)
                .expect("Failed to write source file");
        }

        let source_path = source_dir.path().to_path_buf();
        let target_path = target_parent.path().join("relocated_storage");

        // Count files before relocation
        let source_count = files.len();

        // Perform relocation
        RelocationService::relocate(&source_path, &target_path)
            .expect("Relocation should succeed");

        // Count files in target
        let target_count = fs::read_dir(&target_path)
            .expect("Failed to read target directory")
            .count();

        assert_eq!(
            source_count, target_count,
            "File count should be preserved after relocation"
        );
    });
}

/// Property: Relocation fails gracefully with non-empty target
/// Attempting to relocate to a non-empty target should fail
#[test]
fn prop_relocation_fails_with_nonempty_target() {
    proptest!(|(files in file_structure_strategy())| {
        let source_dir = TempDir::new().expect("Failed to create source temp dir");
        let target_dir = TempDir::new().expect("Failed to create target temp dir");

        // Create source files
        for (name, content) in &files {
            fs::write(source_dir.path().join(name), content)
                .expect("Failed to write source file");
        }

        // Create a file in target to make it non-empty
        fs::write(target_dir.path().join("existing.txt"), "existing content")
            .expect("Failed to create existing file in target");

        let source_path = source_dir.path().to_path_buf();
        let target_path = target_dir.path().to_path_buf();

        // Attempt relocation should fail
        let result = RelocationService::relocate(&source_path, &target_path);
        assert!(
            result.is_err(),
            "Relocation to non-empty target should fail"
        );
    });
}

/// Property: Relocation fails gracefully with non-existent source
/// Attempting to relocate from a non-existent source should fail
#[test]
fn prop_relocation_fails_with_nonexistent_source() {
    let target_parent = TempDir::new().expect("Failed to create target parent temp dir");

    let source_path = target_parent.path().join("nonexistent_source");
    let target_path = target_parent.path().join("target");

    // Attempt relocation should fail
    let result = RelocationService::relocate(&source_path, &target_path);
    assert!(
        result.is_err(),
        "Relocation from non-existent source should fail"
    );
}
