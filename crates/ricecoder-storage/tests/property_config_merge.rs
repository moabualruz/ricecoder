//! Property-based tests for configuration merging
//!
//! **Feature: ricecoder-storage, Property 5: Configuration Merge Precedence**
//! **Feature: ricecoder-storage, Property 13: Merge Decision Logging**
//! **Validates: Requirements 2.7, 3.3, 3.4**

use proptest::prelude::*;
use ricecoder_storage::config::{Config, ConfigMerger, DefaultsConfig, ProvidersConfig};
use std::collections::HashMap;

/// Strategy for generating valid configurations
fn config_strategy() -> impl Strategy<Value = Config> {
    (
        prop::option::of("[a-z_]+"),
        prop::option::of("[a-z_]+"),
        prop::option::of(0.0f32..1.0f32),
    )
        .prop_map(|(provider, model, temp)| {
            let mut providers = ProvidersConfig {
                api_keys: HashMap::new(),
                endpoints: HashMap::new(),
                default_provider: provider.clone(),
            };

            if let Some(p) = provider {
                providers.api_keys.insert(p, "test-key".to_string());
            }

            Config {
                providers,
                defaults: DefaultsConfig {
                    model,
                    temperature: temp,
                    max_tokens: None,
                },
                steering: Vec::new(),
                tui: Default::default(),
                custom: HashMap::new(),
            }
        })
}

proptest! {
    /// Property: Project config overrides global config
    ///
    /// For any global and project configurations, merging should result in
    /// project values overriding global values.
    #[test]
    fn prop_project_overrides_global(
        global in config_strategy(),
        project in config_strategy(),
    ) {
        let defaults = Config::default();
        let (merged, _) = ConfigMerger::merge(defaults, Some(global.clone()), Some(project.clone()), None);

        // Project values should override global values
        if let Some(ref project_provider) = project.providers.default_provider {
            assert_eq!(merged.providers.default_provider, Some(project_provider.clone()));
        }
        if let Some(ref project_model) = project.defaults.model {
            assert_eq!(merged.defaults.model, Some(project_model.clone()));
        }
    }

    /// Property: Environment config overrides all
    ///
    /// For any combination of configs, environment config should override all others.
    #[test]
    fn prop_env_overrides_all(
        global in config_strategy(),
        project in config_strategy(),
        env in config_strategy(),
    ) {
        let defaults = Config::default();
        let (merged, _) = ConfigMerger::merge(defaults, Some(global), Some(project), Some(env.clone()));

        // Environment values should override all
        if let Some(ref env_provider) = env.providers.default_provider {
            assert_eq!(merged.providers.default_provider, Some(env_provider.clone()));
        }
        if let Some(ref env_model) = env.defaults.model {
            assert_eq!(merged.defaults.model, Some(env_model.clone()));
        }
    }

    /// Property: Merge decisions are logged
    ///
    /// For any merge operation, merge decisions should be recorded for all
    /// values that were overridden.
    #[test]
    fn prop_merge_decisions_logged(
        global in config_strategy(),
        project in config_strategy(),
    ) {
        let defaults = Config::default();
        let (_, decisions) = ConfigMerger::merge(defaults, Some(global.clone()), Some(project.clone()), None);

        // If project overrides global, there should be a decision
        if let Some(ref project_provider) = project.providers.default_provider {
            if project_provider != &global.providers.default_provider.clone().unwrap_or_default() {
                let has_decision = decisions.iter().any(|d| {
                    d.key == "providers.default_provider" && d.source == "project"
                });
                // Only assert if global had a different value
                if global.providers.default_provider.is_some() &&
                   global.providers.default_provider != project.providers.default_provider {
                    assert!(has_decision);
                }
            }
        }
    }

    /// Property: Global config is applied when no project config
    ///
    /// For any global configuration without project config, the global values
    /// should be used.
    #[test]
    fn prop_global_applied_without_project(global in config_strategy()) {
        let defaults = Config::default();
        let (merged, _) = ConfigMerger::merge(defaults, Some(global.clone()), None, None);

        // Global values should be applied
        assert_eq!(merged.providers.default_provider, global.providers.default_provider);
        assert_eq!(merged.defaults.model, global.defaults.model);
    }

    /// Property: Defaults are used when no other config
    ///
    /// For any default configuration without global or project config,
    /// the default values should be used.
    #[test]
    fn prop_defaults_used_without_other_config(defaults in config_strategy()) {
        let (merged, _) = ConfigMerger::merge(defaults.clone(), None, None, None);

        // Defaults should be used
        assert_eq!(merged.providers.default_provider, defaults.providers.default_provider);
        assert_eq!(merged.defaults.model, defaults.defaults.model);
    }

    /// Property: API keys are merged, not replaced
    ///
    /// For any global and project configurations with different API keys,
    /// merging should combine them. If they have the same key, project overrides.
    #[test]
    fn prop_api_keys_merged(
        global_key in "[a-z_]+",
        global_value in "[a-zA-Z0-9_\\-]+",
        project_key in "[a-z_]+",
        project_value in "[a-zA-Z0-9_\\-]+",
    ) {
        let mut global = Config::default();
        global.providers.api_keys.insert(global_key.clone(), global_value.clone());

        let mut project = Config::default();
        project.providers.api_keys.insert(project_key.clone(), project_value.clone());

        let defaults = Config::default();
        let (merged, _) = ConfigMerger::merge(defaults, Some(global), Some(project), None);

        // If keys are different, both should be present
        if global_key != project_key {
            assert_eq!(merged.providers.api_keys.get(&global_key), Some(&global_value));
            assert_eq!(merged.providers.api_keys.get(&project_key), Some(&project_value));
        } else {
            // If keys are the same, project should override
            assert_eq!(merged.providers.api_keys.get(&global_key), Some(&project_value));
        }
    }

    /// Property: Merge decisions include source information
    ///
    /// For any merge operation, all decisions should include the source
    /// (global, project, or environment).
    #[test]
    fn prop_merge_decisions_have_source(
        global in config_strategy(),
        project in config_strategy(),
    ) {
        let defaults = Config::default();
        let (_, decisions) = ConfigMerger::merge(defaults, Some(global), Some(project), None);

        // All decisions should have a source
        for decision in decisions {
            assert!(!decision.source.is_empty());
            assert!(
                decision.source == "global" ||
                decision.source == "project" ||
                decision.source == "environment"
            );
        }
    }

    /// Property: Merge decisions include key information
    ///
    /// For any merge operation, all decisions should include the configuration key.
    #[test]
    fn prop_merge_decisions_have_key(
        global in config_strategy(),
        project in config_strategy(),
    ) {
        let defaults = Config::default();
        let (_, decisions) = ConfigMerger::merge(defaults, Some(global), Some(project), None);

        // All decisions should have a key
        for decision in decisions {
            assert!(!decision.key.is_empty());
        }
    }
}
