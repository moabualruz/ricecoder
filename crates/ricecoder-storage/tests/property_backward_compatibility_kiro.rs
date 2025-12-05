//! Property-based tests for backward compatibility with .kiro/ configurations
//! **Feature: ricecoder-path-resolution, Property 5: Backward Compatibility**
//! **Validates: Requirements 3.1**

use proptest::prelude::*;
use ricecoder_storage::IndustryFileAdapter;
use ricecoder_storage::industry::KiroAdapter;
use std::fs;
use tempfile::TempDir;

/// Strategy for generating valid steering rule names
fn steering_rule_name_strategy() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9_\-]{1,20}"
        .prop_map(|s| format!("rule_{}", s))
}

/// Strategy for generating valid steering rule content
fn steering_rule_content_strategy() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9\s\.\,\-]{10,100}"
        .prop_map(|s| format!("# Steering Rule\n\n{}", s))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Property 5: Backward Compatibility - .kiro/ adapter detection
    /// For any project directory with a .kiro/ subdirectory,
    /// the KiroAdapter should detect it and be able to read its configuration.
    #[test]
    fn prop_kiro_adapter_detects_kiro_directory(
        rule_name in steering_rule_name_strategy(),
        rule_content in steering_rule_content_strategy()
    ) {
        // Create a temporary project directory
        let temp_dir = TempDir::new().expect("Should create temp directory");
        let project_root = temp_dir.path();

        // Create .kiro/steering directory structure
        let kiro_dir = project_root.join(".kiro");
        fs::create_dir(&kiro_dir).expect("Should create .kiro directory");
        let steering_dir = kiro_dir.join("steering");
        fs::create_dir(&steering_dir).expect("Should create steering directory");

        // Write a steering rule file
        let rule_file = steering_dir.join(format!("{}.md", rule_name));
        fs::write(&rule_file, &rule_content).expect("Should write steering rule");

        // Create adapter and verify it can handle the directory
        let adapter = KiroAdapter::new();
        prop_assert!(
            adapter.can_handle(project_root),
            "KiroAdapter should detect .kiro directory"
        );

        // Read the configuration
        let config = adapter.read_config(project_root)
            .expect("Should read configuration");

        // Verify the configuration contains the steering rule
        prop_assert!(
            !config.steering.is_empty(),
            "Configuration should contain steering rules"
        );

        // Verify the content is preserved
        prop_assert!(
            config.steering[0].content.contains(&rule_content),
            "Steering rule content should be preserved"
        );
    }

    /// Property: Backward Compatibility - .kiro/ adapter reads specs
    /// For any project directory with .kiro/specs/ subdirectory,
    /// the KiroAdapter should read all spec files and include them in the configuration.
    #[test]
    fn prop_kiro_adapter_reads_specs(
        spec_name in steering_rule_name_strategy(),
        spec_content in steering_rule_content_strategy()
    ) {
        // Create a temporary project directory
        let temp_dir = TempDir::new().expect("Should create temp directory");
        let project_root = temp_dir.path();

        // Create .kiro/specs directory structure
        let kiro_dir = project_root.join(".kiro");
        fs::create_dir(&kiro_dir).expect("Should create .kiro directory");
        let specs_dir = kiro_dir.join("specs");
        fs::create_dir(&specs_dir).expect("Should create specs directory");

        // Write a spec file
        let spec_file = specs_dir.join(format!("{}.md", spec_name));
        fs::write(&spec_file, &spec_content).expect("Should write spec");

        // Create adapter and read configuration
        let adapter = KiroAdapter::new();
        let config = adapter.read_config(project_root)
            .expect("Should read configuration");

        // Verify the configuration contains the spec
        prop_assert!(
            !config.steering.is_empty(),
            "Configuration should contain specs as steering rules"
        );

        // Verify the content is preserved
        prop_assert!(
            config.steering[0].content.contains(&spec_content),
            "Spec content should be preserved"
        );
    }

    /// Property: Backward Compatibility - .kiro/ adapter handles missing directory
    /// For any project directory without a .kiro/ subdirectory,
    /// the KiroAdapter should return false for can_handle and return default config.
    #[test]
    fn prop_kiro_adapter_handles_missing_directory(_unused in Just(())) {
        // Create a temporary project directory without .kiro
        let temp_dir = TempDir::new().expect("Should create temp directory");
        let project_root = temp_dir.path();

        // Create adapter
        let adapter = KiroAdapter::new();

        // Verify it cannot handle the directory
        prop_assert!(
            !adapter.can_handle(project_root),
            "KiroAdapter should not detect missing .kiro directory"
        );

        // Reading config should still succeed with default config
        let config = adapter.read_config(project_root)
            .expect("Should read configuration");

        // Default config should have no steering rules from .kiro
        prop_assert_eq!(
            config.steering.len(),
            0,
            "Default configuration should have no steering rules"
        );
    }

    /// Property: Backward Compatibility - .kiro/ adapter priority
    /// The KiroAdapter should have the highest priority (100) among industry adapters
    /// to ensure .kiro/ configurations are preferred when present.
    #[test]
    fn prop_kiro_adapter_has_highest_priority(_unused in Just(())) {
        let adapter = KiroAdapter::new();
        let priority = adapter.priority();

        // KiroAdapter should have priority of 100 (highest)
        prop_assert_eq!(
            priority,
            100,
            "KiroAdapter should have highest priority (100)"
        );
    }

    /// Property: Backward Compatibility - .kiro/ adapter name
    /// The KiroAdapter should identify itself with the name "kiro"
    /// for proper adapter selection and logging.
    #[test]
    fn prop_kiro_adapter_name(_unused in Just(())) {
        let adapter = KiroAdapter::new();
        let name = adapter.name();

        prop_assert_eq!(
            name,
            "kiro",
            "KiroAdapter should identify itself as 'kiro'"
        );
    }

    /// Property: Backward Compatibility - .kiro/ adapter preserves multiple files
    /// For any project directory with multiple .kiro/steering/ files,
    /// the KiroAdapter should read all of them and include them in the configuration.
    #[test]
    fn prop_kiro_adapter_reads_multiple_files(
        file_count in 1usize..5,
        content_base in steering_rule_content_strategy()
    ) {
        // Create a temporary project directory
        let temp_dir = TempDir::new().expect("Should create temp directory");
        let project_root = temp_dir.path();

        // Create .kiro/steering directory structure
        let kiro_dir = project_root.join(".kiro");
        fs::create_dir(&kiro_dir).expect("Should create .kiro directory");
        let steering_dir = kiro_dir.join("steering");
        fs::create_dir(&steering_dir).expect("Should create steering directory");

        // Write multiple steering rule files
        for i in 0..file_count {
            let rule_file = steering_dir.join(format!("rule_{}.md", i));
            let content = format!("{}\n\nFile {}", content_base, i);
            fs::write(&rule_file, &content).expect("Should write steering rule");
        }

        // Create adapter and read configuration
        let adapter = KiroAdapter::new();
        let config = adapter.read_config(project_root)
            .expect("Should read configuration");

        // Verify the configuration contains all files
        prop_assert!(
            !config.steering.is_empty(),
            "Configuration should contain steering rules"
        );

        // Verify all file contents are included
        let combined_content = config.steering[0].content.clone();
        for i in 0..file_count {
            prop_assert!(
                combined_content.contains(&format!("File {}", i)),
                "All file contents should be included"
            );
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_kiro_adapter_detects_directory() {
        let temp_dir = TempDir::new().unwrap();
        let kiro_dir = temp_dir.path().join(".kiro");
        fs::create_dir(&kiro_dir).unwrap();

        let adapter = KiroAdapter::new();
        assert!(adapter.can_handle(temp_dir.path()));
    }

    #[test]
    fn test_kiro_adapter_no_directory() {
        let temp_dir = TempDir::new().unwrap();

        let adapter = KiroAdapter::new();
        assert!(!adapter.can_handle(temp_dir.path()));
    }

    #[test]
    fn test_kiro_adapter_reads_steering_and_specs() {
        let temp_dir = TempDir::new().unwrap();
        let kiro_dir = temp_dir.path().join(".kiro");
        fs::create_dir(&kiro_dir).unwrap();

        // Create steering directory
        let steering_dir = kiro_dir.join("steering");
        fs::create_dir(&steering_dir).unwrap();
        fs::write(steering_dir.join("rules.md"), "# Steering Rules").unwrap();

        // Create specs directory
        let specs_dir = kiro_dir.join("specs");
        fs::create_dir(&specs_dir).unwrap();
        fs::write(specs_dir.join("spec1.md"), "# Spec 1").unwrap();

        let adapter = KiroAdapter::new();
        let config = adapter.read_config(temp_dir.path()).unwrap();

        assert_eq!(config.steering.len(), 1);
        assert!(config.steering[0].content.contains("Steering Rules"));
        assert!(config.steering[0].content.contains("Spec 1"));
    }

    #[test]
    fn test_kiro_adapter_precedence() {
        // Verify that KiroAdapter has highest priority
        let adapter = KiroAdapter::new();
        assert_eq!(adapter.priority(), 100);
    }
}
