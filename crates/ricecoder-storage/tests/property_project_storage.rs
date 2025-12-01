//! Property-based tests for project storage
//! **Feature: ricecoder-storage, Property 2: Project Resource Storage Consistency**
//! **Feature: ricecoder-storage, Property 12: Folder Creation On-Demand**
//! **Validates: Requirements 2.2, 2.5, 2.6**

use proptest::prelude::*;
use ricecoder_storage::{ProjectStore, ResourceType};
use tempfile::TempDir;

/// Strategy for generating valid resource names
fn resource_name_strategy() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9_\-]+" // Alphanumeric, underscore, dash
        .prop_map(|s| format!("{}.txt", s))
        .prop_filter("Name should be non-empty", |s| !s.is_empty())
}

/// Strategy for generating valid folder names
fn folder_name_strategy() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9_\-]+" // Alphanumeric, underscore, dash
        .prop_filter("Name should be non-empty", |s| !s.is_empty())
}

/// Strategy for generating valid resource content
fn resource_content_strategy() -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(any::<u8>(), 0..1000)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property 2: Project Resource Storage Consistency
    /// For any project configuration or spec, storing it in the project store should persist it
    /// to the correct location (./.agent/specs/ or ./.agent/config.yaml) and the resource
    /// should be retrievable with identical content.
    #[test]
    fn prop_project_resource_storage_consistency(
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
        let store = ProjectStore::new(temp_dir.path().to_path_buf());
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

    /// Property 12: Folder Creation On-Demand
    /// For any valid folder name, creating the folder should result in a directory
    /// that exists and can be verified with folder_exists()
    #[test]
    fn prop_folder_creation_on_demand(folder_name in folder_name_strategy()) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let store = ProjectStore::new(temp_dir.path().to_path_buf());
        store.initialize().expect("Failed to initialize store");

        // Initially should not exist
        assert!(
            !store.folder_exists(&folder_name),
            "Folder should not exist initially"
        );

        // Create the folder
        let folder_path = store
            .create_folder(&folder_name)
            .expect("Failed to create folder");

        // Verify the folder was created
        assert!(folder_path.exists(), "Folder should exist after creation");
        assert!(
            store.folder_exists(&folder_name),
            "folder_exists should return true after creation"
        );

        // Verify it's a directory
        assert!(
            folder_path.is_dir(),
            "Created path should be a directory"
        );
    }

    /// Property: Folder creation is idempotent
    /// Creating the same folder multiple times should not cause errors
    #[test]
    fn prop_folder_creation_idempotent(folder_name in folder_name_strategy()) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let store = ProjectStore::new(temp_dir.path().to_path_buf());
        store.initialize().expect("Failed to initialize store");

        // Create the folder multiple times
        let path1 = store
            .create_folder(&folder_name)
            .expect("First create failed");
        let path2 = store
            .create_folder(&folder_name)
            .expect("Second create failed");
        let path3 = store
            .create_folder(&folder_name)
            .expect("Third create failed");

        // All paths should be identical
        assert_eq!(path1, path2, "First and second paths should match");
        assert_eq!(path2, path3, "Second and third paths should match");

        // Folder should still exist
        assert!(store.folder_exists(&folder_name));
    }

    /// Property: Project resource storage is idempotent
    /// Storing the same resource multiple times should result in the same content
    #[test]
    fn prop_project_resource_storage_idempotent(
        resource_type in prop_oneof![
            Just(ResourceType::Template),
            Just(ResourceType::Standard),
        ],
        name in resource_name_strategy(),
        content in resource_content_strategy(),
    ) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let store = ProjectStore::new(temp_dir.path().to_path_buf());
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
    fn prop_project_resource_existence_accurate(
        resource_type in prop_oneof![
            Just(ResourceType::Template),
            Just(ResourceType::Standard),
        ],
        name in resource_name_strategy(),
        content in resource_content_strategy(),
    ) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let store = ProjectStore::new(temp_dir.path().to_path_buf());
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
}
