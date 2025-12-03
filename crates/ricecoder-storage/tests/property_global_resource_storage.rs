//! Property-based tests for global resource storage
//! **Feature: ricecoder-storage, Property 1: Global Resource Storage Consistency**
//! **Validates: Requirements 1.2, 1.3, 1.4, 1.5**

use proptest::prelude::*;
use ricecoder_storage::{GlobalStore, ResourceType};
use tempfile::TempDir;

/// Strategy for generating valid resource names
fn resource_name_strategy() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9_\-]+" // Alphanumeric, underscore, dash
        .prop_map(|s| format!("{}.txt", s))
        .prop_filter("Name should be non-empty", |s| !s.is_empty())
}

/// Strategy for generating valid resource content
fn resource_content_strategy() -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(any::<u8>(), 0..1000)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property 1: Global Resource Storage Consistency
    /// For any resource type (template, standard, spec, steering) and any valid resource content,
    /// storing the resource globally should persist it to the correct subdirectory
    /// and the resource should be retrievable with identical content.
    #[test]
    fn prop_global_resource_storage_consistency(
        resource_type in prop_oneof![
            Just(ResourceType::Template),
            Just(ResourceType::Standard),
            Just(ResourceType::Spec),
            Just(ResourceType::Steering),
        ],
        name in resource_name_strategy(),
        content in resource_content_strategy(),
    ) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let store = GlobalStore::new(temp_dir.path().to_path_buf());
        store.initialize().expect("Failed to initialize store");

        // Store the resource
        let stored_path = store
            .store_resource(resource_type, &name, &content)
            .expect("Failed to store resource");

        // Verify the file was created in the correct directory
        assert!(stored_path.exists(), "Stored file should exist");
        assert_eq!(
            stored_path.parent().unwrap(),
            store.resource_path(resource_type),
            "File should be in correct resource directory"
        );

        // Retrieve the resource
        let retrieved = store
            .retrieve_resource(resource_type, &name)
            .expect("Failed to retrieve resource");

        // Verify the content is identical
        assert_eq!(
            retrieved, content,
            "Retrieved content should match stored content"
        );
    }

    /// Property: Resource storage is idempotent
    /// Storing the same resource multiple times should result in the same content
    #[test]
    fn prop_resource_storage_idempotent(
        resource_type in prop_oneof![
            Just(ResourceType::Template),
            Just(ResourceType::Standard),
        ],
        name in resource_name_strategy(),
        content in resource_content_strategy(),
    ) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let store = GlobalStore::new(temp_dir.path().to_path_buf());
        store.initialize().expect("Failed to initialize store");

        // Store the resource multiple times
        store
            .store_resource(resource_type, &name, &content)
            .expect("First store failed");
        store
            .store_resource(resource_type, &name, &content)
            .expect("Second store failed");
        store
            .store_resource(resource_type, &name, &content)
            .expect("Third store failed");

        // Retrieve and verify
        let retrieved = store
            .retrieve_resource(resource_type, &name)
            .expect("Failed to retrieve resource");

        assert_eq!(
            retrieved, content,
            "Content should be identical after multiple stores"
        );
    }

    /// Property: Resource existence check is accurate
    /// After storing a resource, it should exist; after deleting it, it should not exist
    #[test]
    fn prop_resource_existence_accurate(
        resource_type in prop_oneof![
            Just(ResourceType::Template),
            Just(ResourceType::Standard),
        ],
        name in resource_name_strategy(),
        content in resource_content_strategy(),
    ) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let store = GlobalStore::new(temp_dir.path().to_path_buf());
        store.initialize().expect("Failed to initialize store");

        // Initially should not exist
        assert!(
            !store.resource_exists(resource_type, &name),
            "Resource should not exist initially"
        );

        // Store the resource
        store
            .store_resource(resource_type, &name, &content)
            .expect("Failed to store resource");

        // Now should exist
        assert!(
            store.resource_exists(resource_type, &name),
            "Resource should exist after storing"
        );

        // Delete the resource
        store
            .delete_resource(resource_type, &name)
            .expect("Failed to delete resource");

        // Should not exist anymore
        assert!(
            !store.resource_exists(resource_type, &name),
            "Resource should not exist after deletion"
        );
    }

    /// Property: List resources returns all stored resources
    /// After storing multiple resources, listing should return all of them
    #[test]
    fn prop_list_resources_complete(
        resource_type in prop_oneof![
            Just(ResourceType::Template),
            Just(ResourceType::Standard),
        ],
        names in prop::collection::hash_set(resource_name_strategy(), 1..10),
        content in resource_content_strategy(),
    ) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let store = GlobalStore::new(temp_dir.path().to_path_buf());
        store.initialize().expect("Failed to initialize store");

        // Filter names to ensure case-insensitive uniqueness (for Windows compatibility)
        // On case-insensitive file systems, "W.txt" and "w.txt" are the same file
        let mut unique_names = Vec::new();
        let mut seen_lower = std::collections::HashSet::new();
        for name in names {
            let lower = name.to_lowercase();
            if seen_lower.insert(lower) {
                unique_names.push(name);
            }
        }

        // Store all resources
        for name in &unique_names {
            store
                .store_resource(resource_type, name, &content)
                .expect("Failed to store resource");
        }

        // List resources
        let listed = store
            .list_resources(resource_type)
            .expect("Failed to list resources");

        // Verify all stored resources are in the list
        for name in &unique_names {
            assert!(
                listed.contains(name),
                "Stored resource {} should be in list",
                name
            );
        }

        // Verify count matches
        assert_eq!(
            listed.len(),
            unique_names.len(),
            "Listed resources count should match stored count"
        );
    }
}
