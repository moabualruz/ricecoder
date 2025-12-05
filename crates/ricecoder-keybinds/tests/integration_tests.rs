//! Integration tests for keybind customization
//! Tests end-to-end workflows combining multiple components

use ricecoder_keybinds::{
    Keybind, KeybindEngine, KeyCombo, FileSystemPersistence, KeybindPersistence,
    ConflictDetector,
};
use std::str::FromStr;
use tempfile::TempDir;

/// Helper to create test keybinds
fn create_test_keybinds() -> Vec<Keybind> {
    vec![
        Keybind::new("editor.save", "Ctrl+S", "editing", "Save file"),
        Keybind::new("editor.undo", "Ctrl+Z", "editing", "Undo change"),
        Keybind::new("editor.redo", "Ctrl+Y", "editing", "Redo change"),
        Keybind::new("nav.next", "Tab", "navigation", "Move to next item"),
        Keybind::new("nav.prev", "Shift+Tab", "navigation", "Move to previous item"),
        Keybind::new("search.find", "Ctrl+F", "search", "Find text"),
        Keybind::new("search.replace", "Ctrl+H", "search", "Replace text"),
    ]
}

/// Helper to create JSON configuration
fn create_json_config(keybinds: &[Keybind]) -> String {
    serde_json::json!({
        "version": "1.0",
        "keybinds": keybinds
    })
    .to_string()
}

#[test]
fn test_end_to_end_keybind_customization() {
    // Create engine
    let mut engine = KeybindEngine::new();

    // Create test keybinds
    let keybinds = create_test_keybinds();

    // Apply keybinds
    assert!(engine.apply_keybinds(keybinds.clone()).is_ok());

    // Verify keybinds are applied
    assert_eq!(engine.keybind_count(), 7);

    // Lookup by action
    let save_keybind = engine.get_keybind("editor.save");
    assert!(save_keybind.is_some());
    assert_eq!(save_keybind.unwrap().key, "Ctrl+S");

    // Lookup by key
    let key_combo = KeyCombo::from_str("Ctrl+S").unwrap();
    let action = engine.get_action(&key_combo);
    assert_eq!(action, Some("editor.save"));

    // Get all keybinds
    let all = engine.all_keybinds();
    assert_eq!(all.len(), 7);
}

#[test]
fn test_profile_switching_with_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().to_path_buf();

    // Create engine
    let mut engine = KeybindEngine::new();

    // Create persistence layer
    let persistence = FileSystemPersistence::new(&config_dir).unwrap();

    // Create two profiles with different keybinds
    let profile1_keybinds = vec![
        Keybind::new("editor.save", "Ctrl+S", "editing", "Save"),
        Keybind::new("editor.undo", "Ctrl+Z", "editing", "Undo"),
    ];

    let profile2_keybinds = vec![
        Keybind::new("editor.save", "Cmd+S", "editing", "Save"),
        Keybind::new("editor.undo", "Cmd+Z", "editing", "Undo"),
    ];

    // Create profiles
    assert!(engine.create_profile("profile1", profile1_keybinds.clone()).is_ok());
    assert!(engine.create_profile("profile2", profile2_keybinds.clone()).is_ok());

    // Select profile1 and verify keybinds
    assert!(engine.select_profile("profile1").is_ok());
    assert_eq!(engine.active_profile_name(), Some("profile1"));

    let key_combo = KeyCombo::from_str("Ctrl+S").unwrap();
    assert_eq!(engine.get_action(&key_combo), Some("editor.save"));

    // Switch to profile2 and verify keybinds changed
    assert!(engine.select_profile("profile2").is_ok());
    assert_eq!(engine.active_profile_name(), Some("profile2"));

    let key_combo = KeyCombo::from_str("Cmd+S").unwrap();
    assert_eq!(engine.get_action(&key_combo), Some("editor.save"));

    // Verify old keybind is gone
    let old_key = KeyCombo::from_str("Ctrl+S").unwrap();
    assert_eq!(engine.get_action(&old_key), None);

    // Persist profiles
    let profile1 = engine.get_profile("profile1").unwrap();
    assert!(persistence.save_profile(profile1).is_ok());

    let profile2 = engine.get_profile("profile2").unwrap();
    assert!(persistence.save_profile(profile2).is_ok());

    // Verify profiles were persisted
    let loaded_profiles = persistence.list_profiles().unwrap();
    assert_eq!(loaded_profiles.len(), 2);
    assert!(loaded_profiles.contains(&"profile1".to_string()));
    assert!(loaded_profiles.contains(&"profile2".to_string()));

    // Load profile1 and verify
    let loaded_profile1 = persistence.load_profile("profile1").unwrap();
    assert_eq!(loaded_profile1.keybinds.len(), 2);
    assert_eq!(loaded_profile1.keybinds[0].action_id, "editor.save");
}

#[test]
fn test_conflict_detection_and_resolution() {
    // Create keybinds with conflicts
    let keybinds = vec![
        Keybind::new("editor.save", "Ctrl+S", "editing", "Save"),
        Keybind::new("editor.undo", "Ctrl+S", "editing", "Undo"), // Conflict!
        Keybind::new("nav.next", "Tab", "navigation", "Next"),
    ];

    // Detect conflicts
    let conflicts = ConflictDetector::detect(&keybinds);

    // Verify conflict was detected
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].actions.len(), 2);
    assert!(conflicts[0].actions.contains(&"editor.save".to_string()));
    assert!(conflicts[0].actions.contains(&"editor.undo".to_string()));

    // Get resolutions
    let resolutions = ConflictDetector::suggest_resolution(&conflicts[0], &keybinds);

    // Verify resolutions were suggested
    assert_eq!(resolutions.len(), 2);
    for resolution in &resolutions {
        assert!(!resolution.suggested_key.is_empty());
        assert!(!resolution.reason.is_empty());
    }
}

#[test]
fn test_help_system_with_real_keybinds() {
    let mut engine = KeybindEngine::new();
    let keybinds = create_test_keybinds();

    // Apply keybinds
    engine.apply_keybinds(keybinds).unwrap();

    // Get help for all keybinds
    let help_all = engine.get_help_all();
    assert!(help_all.contains("# All Keybinds"));
    assert!(help_all.contains("## editing"));
    assert!(help_all.contains("## navigation"));
    assert!(help_all.contains("editor.save"));

    // Get help by category
    let help_editing = engine.get_help_by_category("editing");
    assert!(help_editing.contains("# editing Keybinds"));
    assert!(help_editing.contains("editor.save"));
    assert!(help_editing.contains("editor.undo"));
    assert!(!help_editing.contains("nav.next"));

    // Search keybinds
    let search_results = engine.search_keybinds("save");
    assert_eq!(search_results.len(), 1);
    assert_eq!(search_results[0].action_id, "editor.save");

    // Paginate keybinds
    let page = engine.get_keybinds_paginated(1, 3);
    assert_eq!(page.current_page, 1);
    assert_eq!(page.total_pages, 3);
    assert_eq!(page.items.len(), 3);
}

#[test]
fn test_default_loading_and_reset() {
    let mut engine = KeybindEngine::new();

    // Create test defaults
    let defaults = vec![
        Keybind::new_default("editor.save", "Ctrl+S", "editing", "Save"),
        Keybind::new_default("editor.undo", "Ctrl+Z", "editing", "Undo"),
        Keybind::new_default("nav.next", "Tab", "navigation", "Next"),
    ];

    // Set defaults
    engine.set_defaults(defaults.clone());

    // Apply defaults
    assert!(engine.apply_defaults().is_ok());

    // Verify defaults were applied
    assert_eq!(engine.keybind_count(), 3);
    assert_eq!(engine.active_profile_name(), Some("default"));

    // Modify keybinds
    let custom = vec![Keybind::new("custom.action", "Ctrl+Q", "custom", "Custom")];
    engine.apply_keybinds(custom).unwrap();
    assert_eq!(engine.keybind_count(), 1);

    // Reset to defaults
    assert!(engine.reset_to_defaults().is_ok());

    // Verify defaults were restored
    assert_eq!(engine.keybind_count(), 3);

    let key_combo = KeyCombo::from_str("Ctrl+S").unwrap();
    assert_eq!(engine.get_action(&key_combo), Some("editor.save"));
}

#[test]
fn test_validation_pipeline_with_conflicts() {
    let mut engine = KeybindEngine::new();

    // Create JSON with conflicts
    let json = r#"{
        "version": "1.0",
        "keybinds": [
            {"action_id": "editor.save", "key": "Ctrl+S", "category": "editing", "description": "Save", "is_default": false},
            {"action_id": "editor.undo", "key": "Ctrl+S", "category": "editing", "description": "Undo", "is_default": false}
        ]
    }"#;

    // Validate and apply
    let result = engine.validate_and_apply_from_string(json, "json");

    // Should fail validation due to conflicts
    assert!(result.is_ok());
    let validation = result.unwrap();
    assert!(!validation.is_valid);
    assert_eq!(validation.conflict_count(), 1);

    // Keybinds should not be applied
    assert_eq!(engine.keybind_count(), 0);
}

#[test]
fn test_validation_pipeline_without_conflicts() {
    let mut engine = KeybindEngine::new();

    // Create JSON without conflicts
    let json = r#"{
        "version": "1.0",
        "keybinds": [
            {"action_id": "editor.save", "key": "Ctrl+S", "category": "editing", "description": "Save", "is_default": false},
            {"action_id": "editor.undo", "key": "Ctrl+Z", "category": "editing", "description": "Undo", "is_default": false}
        ]
    }"#;

    // Validate and apply
    let result = engine.validate_and_apply_from_string(json, "json");

    // Should pass validation
    assert!(result.is_ok());
    let validation = result.unwrap();
    assert!(validation.is_valid);
    assert_eq!(validation.conflict_count(), 0);

    // Keybinds should be applied
    assert_eq!(engine.keybind_count(), 2);

    let key_combo = KeyCombo::from_str("Ctrl+S").unwrap();
    assert_eq!(engine.get_action(&key_combo), Some("editor.save"));
}

#[test]
fn test_full_workflow_parse_validate_apply_persist() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().to_path_buf();

    let mut engine = KeybindEngine::new();
    let persistence = FileSystemPersistence::new(&config_dir).unwrap();

    // Create JSON configuration
    let json = r#"{
        "version": "1.0",
        "keybinds": [
            {"action_id": "editor.save", "key": "Ctrl+S", "category": "editing", "description": "Save", "is_default": false},
            {"action_id": "editor.undo", "key": "Ctrl+Z", "category": "editing", "description": "Undo", "is_default": false},
            {"action_id": "nav.next", "key": "Tab", "category": "navigation", "description": "Next", "is_default": false}
        ]
    }"#;

    // Full pipeline: parse → validate → apply → persist
    let result = engine.validate_apply_and_persist_from_string(json, "json", "my_profile", &persistence);

    // Should succeed
    assert!(result.is_ok());
    let validation = result.unwrap();
    assert!(validation.is_valid);
    assert_eq!(validation.applied_keybinds, 3);

    // Verify keybinds are applied
    assert_eq!(engine.keybind_count(), 3);

    // Verify profile was persisted
    let profiles = persistence.list_profiles().unwrap();
    assert!(profiles.contains(&"my_profile".to_string()));

    // Load profile and verify
    let loaded_profile = persistence.load_profile("my_profile").unwrap();
    assert_eq!(loaded_profile.keybinds.len(), 3);
    assert_eq!(loaded_profile.keybinds[0].action_id, "editor.save");
}

#[test]
fn test_categories_and_organization() {
    let mut engine = KeybindEngine::new();
    let keybinds = create_test_keybinds();

    engine.apply_keybinds(keybinds).unwrap();

    // Get categories
    let categories = engine.categories();
    assert_eq!(categories.len(), 3);
    assert!(categories.contains(&"editing".to_string()));
    assert!(categories.contains(&"navigation".to_string()));
    assert!(categories.contains(&"search".to_string()));

    // Get keybinds by category
    let editing = engine.keybinds_by_category("editing");
    assert_eq!(editing.len(), 3);

    let navigation = engine.keybinds_by_category("navigation");
    assert_eq!(navigation.len(), 2);

    let search = engine.keybinds_by_category("search");
    assert_eq!(search.len(), 2);
}

#[test]
fn test_multiple_profiles_isolation() {
    let mut engine = KeybindEngine::new();

    // Create profile 1
    let profile1_keybinds = vec![
        Keybind::new("editor.save", "Ctrl+S", "editing", "Save"),
        Keybind::new("editor.undo", "Ctrl+Z", "editing", "Undo"),
    ];
    engine.create_profile("profile1", profile1_keybinds).unwrap();

    // Create profile 2
    let profile2_keybinds = vec![
        Keybind::new("editor.save", "Cmd+S", "editing", "Save"),
        Keybind::new("editor.undo", "Cmd+Z", "editing", "Undo"),
        Keybind::new("nav.next", "Tab", "navigation", "Next"),
    ];
    engine.create_profile("profile2", profile2_keybinds).unwrap();

    // Create profile 3
    let profile3_keybinds = vec![
        Keybind::new("search.find", "Ctrl+F", "search", "Find"),
    ];
    engine.create_profile("profile3", profile3_keybinds).unwrap();

    // Switch to each profile and verify isolation
    engine.select_profile("profile1").unwrap();
    assert_eq!(engine.keybind_count(), 2);

    engine.select_profile("profile2").unwrap();
    assert_eq!(engine.keybind_count(), 3);

    engine.select_profile("profile3").unwrap();
    assert_eq!(engine.keybind_count(), 1);

    // Switch back to profile1 and verify it's unchanged
    engine.select_profile("profile1").unwrap();
    assert_eq!(engine.keybind_count(), 2);

    let key_combo = KeyCombo::from_str("Ctrl+S").unwrap();
    assert_eq!(engine.get_action(&key_combo), Some("editor.save"));
}

#[test]
fn test_keybind_lookup_consistency() {
    let mut engine = KeybindEngine::new();
    let keybinds = create_test_keybinds();

    engine.apply_keybinds(keybinds).unwrap();

    // Perform multiple lookups and verify consistency
    for _ in 0..10 {
        let key_combo = KeyCombo::from_str("Ctrl+S").unwrap();
        assert_eq!(engine.get_action(&key_combo), Some("editor.save"));

        let keybind = engine.get_keybind("editor.save");
        assert!(keybind.is_some());
        assert_eq!(keybind.unwrap().key, "Ctrl+S");
    }
}

#[test]
fn test_empty_engine_operations() {
    let engine = KeybindEngine::new();

    // Test operations on empty engine
    assert_eq!(engine.keybind_count(), 0);
    assert!(!engine.has_keybinds());

    let key_combo = KeyCombo::from_str("Ctrl+S").unwrap();
    assert_eq!(engine.get_action(&key_combo), None);
    assert_eq!(engine.get_keybind("editor.save"), None);

    let all = engine.all_keybinds();
    assert_eq!(all.len(), 0);

    let categories = engine.categories();
    assert_eq!(categories.len(), 0);

    let help = engine.get_help_all();
    assert!(help.contains("No keybinds configured"));
}

#[test]
fn test_search_functionality() {
    let mut engine = KeybindEngine::new();
    let keybinds = create_test_keybinds();

    engine.apply_keybinds(keybinds).unwrap();

    // Search by action name
    let results = engine.search_keybinds("editor");
    assert_eq!(results.len(), 3);

    // Search by key
    let results = engine.search_keybinds("Ctrl");
    assert_eq!(results.len(), 5);

    // Search by description
    let results = engine.search_keybinds("file");
    assert_eq!(results.len(), 1);

    // Search with no results
    let results = engine.search_keybinds("nonexistent");
    assert_eq!(results.len(), 0);
}

#[test]
fn test_pagination() {
    let mut engine = KeybindEngine::new();
    let keybinds = create_test_keybinds();

    engine.apply_keybinds(keybinds).unwrap();

    // Test pagination
    let page1 = engine.get_keybinds_paginated(1, 3);
    assert_eq!(page1.current_page, 1);
    assert_eq!(page1.total_pages, 3);
    assert_eq!(page1.items.len(), 3);

    let page2 = engine.get_keybinds_paginated(2, 3);
    assert_eq!(page2.current_page, 2);
    assert_eq!(page2.total_pages, 3);
    assert_eq!(page2.items.len(), 3);

    let page3 = engine.get_keybinds_paginated(3, 3);
    assert_eq!(page3.current_page, 3);
    assert_eq!(page3.total_pages, 3);
    assert_eq!(page3.items.len(), 1);

    // Test invalid page
    let invalid = engine.get_keybinds_paginated(10, 3);
    assert_eq!(invalid.items.len(), 0);
}
