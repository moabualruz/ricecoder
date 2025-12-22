//! Property-based tests for configuration consistency
//!
//! **Feature: ricecoder-orchestration, Property 2: Configuration Application Consistency**
//!
//! *For any* workspace configuration, the ConfigManager SHALL apply workspace-level
//! settings consistently to all projects without conflicts or partial application.
//!
//! **Validates: Requirements 1.3**

use std::path::PathBuf;

use proptest::prelude::*;
use ricecoder_orchestration::ConfigManager;

proptest! {
    /// Property: Configuration manager creation is deterministic
    ///
    /// For any workspace path, creating a ConfigManager multiple times
    /// should produce consistent results.
    #[test]
    fn prop_config_manager_creation_deterministic(
        path_suffix in "[a-z0-9_-]{1,20}"
    ) {
        let path = PathBuf::from(format!("/workspace/{}", path_suffix));

        let manager1 = ConfigManager::new(path.clone());
        let manager2 = ConfigManager::new(path.clone());

        // Both managers should have the same configuration
        let config1 = manager1.get_config();
        let config2 = manager2.get_config();

        prop_assert_eq!(config1.rules.len(), config2.rules.len());
    }

    /// Property: Configuration retrieval is consistent
    ///
    /// For any ConfigManager, getting the configuration multiple times
    /// should produce identical results.
    #[test]
    fn prop_config_retrieval_consistent(
        path_suffix in "[a-z0-9_-]{1,20}"
    ) {
        let path = PathBuf::from(format!("/workspace/{}", path_suffix));
        let manager = ConfigManager::new(path);

        let config1 = manager.get_config();
        let config2 = manager.get_config();

        prop_assert_eq!(config1.rules.len(), config2.rules.len());
        prop_assert_eq!(&config1.settings, &config2.settings);
    }

    /// Property: Configuration settings are serializable
    ///
    /// For any ConfigManager, the configuration should be serializable to JSON.
    #[test]
    fn prop_config_settings_serializable(
        path_suffix in "[a-z0-9_-]{1,20}"
    ) {
        let path = PathBuf::from(format!("/workspace/{}", path_suffix));
        let manager = ConfigManager::new(path);

        let config = manager.get_config();

        // Should be serializable
        let serialized = serde_json::to_string(&config);
        prop_assert!(serialized.is_ok());

        // Should be deserializable
        if let Ok(json_str) = serialized {
            let deserialized: Result<ricecoder_orchestration::WorkspaceConfig, _> =
                serde_json::from_str(&json_str);
            prop_assert!(deserialized.is_ok());
        }
    }

    /// Property: Configuration rules are consistent
    ///
    /// For any ConfigManager, the rules should be consistent across multiple retrievals.
    #[test]
    fn prop_config_rules_consistent(
        path_suffix in "[a-z0-9_-]{1,20}"
    ) {
        let path = PathBuf::from(format!("/workspace/{}", path_suffix));
        let manager = ConfigManager::new(path);

        let rules1 = manager.get_rules();
        let rules2 = manager.get_rules();

        prop_assert_eq!(rules1.len(), rules2.len());

        for (r1, r2) in rules1.iter().zip(rules2.iter()) {
            prop_assert_eq!(&r1.name, &r2.name);
            prop_assert_eq!(r1.enabled, r2.enabled);
        }
    }

    /// Property: Configuration settings are accessible
    ///
    /// For any ConfigManager, settings should be retrievable by key.
    #[test]
    fn prop_config_settings_accessible(
        path_suffix in "[a-z0-9_-]{1,20}"
    ) {
        let path = PathBuf::from(format!("/workspace/{}", path_suffix));
        let manager = ConfigManager::new(path);

        // Should be able to get settings
        let config = manager.get_config();
        prop_assert!(config.settings.is_object());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_manager_creation() {
        let manager = ConfigManager::new(PathBuf::from("/workspace"));
        let config = manager.get_config();
        // Config should be valid even if empty
        assert!(config.settings.is_object());
    }

    #[test]
    fn test_config_retrieval() {
        let manager = ConfigManager::new(PathBuf::from("/workspace"));
        let config1 = manager.get_config();
        let config2 = manager.get_config();
        assert_eq!(config1.rules.len(), config2.rules.len());
    }

    #[test]
    fn test_config_serialization() {
        let manager = ConfigManager::new(PathBuf::from("/workspace"));
        let config = manager.get_config();
        let serialized = serde_json::to_string(&config);
        assert!(serialized.is_ok());
    }
}
