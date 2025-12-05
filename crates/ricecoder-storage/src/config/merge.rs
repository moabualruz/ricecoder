//! Configuration merging with precedence rules
//!
//! This module provides configuration merging with the following precedence:
//! environment > project > legacy > global > defaults

use super::Config;
use tracing::debug;

/// Configuration merger
pub struct ConfigMerger;

/// Merge decision for logging
#[derive(Debug, Clone)]
pub struct MergeDecision {
    /// The key that was merged
    pub key: String,
    /// The source of the value
    pub source: String,
    /// The value that was applied
    pub value: String,
}

impl ConfigMerger {
    /// Merge configurations with precedence rules
    ///
    /// Precedence: env > project > legacy > global > defaults
    ///
    /// Returns the merged configuration and a list of merge decisions for logging.
    pub fn merge(
        defaults: Config,
        global: Option<Config>,
        project: Option<Config>,
        env_overrides: Option<Config>,
    ) -> (Config, Vec<MergeDecision>) {
        let mut decisions = Vec::new();
        let mut result = defaults;

        // Apply global config
        if let Some(global_config) = global {
            Self::merge_into(&mut result, &global_config, "global", &mut decisions);
        }

        // Apply project config (overrides global)
        if let Some(project_config) = project {
            Self::merge_into(&mut result, &project_config, "project", &mut decisions);
        }

        // Apply environment overrides (highest priority)
        if let Some(env_config) = env_overrides {
            Self::merge_into(&mut result, &env_config, "environment", &mut decisions);
        }

        // Log merge decisions
        for decision in &decisions {
            debug!(
                key = %decision.key,
                source = %decision.source,
                value = %decision.value,
                "Configuration merged"
            );
        }

        (result, decisions)
    }

    /// Merge one configuration into another
    fn merge_into(
        target: &mut Config,
        source: &Config,
        source_name: &str,
        decisions: &mut Vec<MergeDecision>,
    ) {
        // Merge providers
        if let Some(ref provider) = source.providers.default_provider {
            if target.providers.default_provider != source.providers.default_provider {
                decisions.push(MergeDecision {
                    key: "providers.default_provider".to_string(),
                    source: source_name.to_string(),
                    value: provider.clone(),
                });
                target.providers.default_provider = Some(provider.clone());
            }
        }

        for (key, value) in &source.providers.api_keys {
            if !target.providers.api_keys.contains_key(key) {
                decisions.push(MergeDecision {
                    key: format!("providers.api_keys.{}", key),
                    source: source_name.to_string(),
                    value: value.clone(),
                });
            }
            target.providers.api_keys.insert(key.clone(), value.clone());
        }

        for (key, value) in &source.providers.endpoints {
            if !target.providers.endpoints.contains_key(key) {
                decisions.push(MergeDecision {
                    key: format!("providers.endpoints.{}", key),
                    source: source_name.to_string(),
                    value: value.clone(),
                });
            }
            target
                .providers
                .endpoints
                .insert(key.clone(), value.clone());
        }

        // Merge defaults
        if let Some(ref model) = source.defaults.model {
            if target.defaults.model != source.defaults.model {
                decisions.push(MergeDecision {
                    key: "defaults.model".to_string(),
                    source: source_name.to_string(),
                    value: model.clone(),
                });
                target.defaults.model = Some(model.clone());
            }
        }

        if let Some(temp) = source.defaults.temperature {
            if target.defaults.temperature != source.defaults.temperature {
                decisions.push(MergeDecision {
                    key: "defaults.temperature".to_string(),
                    source: source_name.to_string(),
                    value: temp.to_string(),
                });
                target.defaults.temperature = Some(temp);
            }
        }

        if let Some(tokens) = source.defaults.max_tokens {
            if target.defaults.max_tokens != source.defaults.max_tokens {
                decisions.push(MergeDecision {
                    key: "defaults.max_tokens".to_string(),
                    source: source_name.to_string(),
                    value: tokens.to_string(),
                });
                target.defaults.max_tokens = Some(tokens);
            }
        }

        // Merge steering
        for rule in &source.steering {
            if !target.steering.iter().any(|r| r.name == rule.name) {
                decisions.push(MergeDecision {
                    key: format!("steering.{}", rule.name),
                    source: source_name.to_string(),
                    value: format!("{} bytes", rule.content.len()),
                });
                target.steering.push(rule.clone());
            }
        }

        // Merge custom settings
        for (key, value) in &source.custom {
            if !target.custom.contains_key(key) {
                decisions.push(MergeDecision {
                    key: key.clone(),
                    source: source_name.to_string(),
                    value: value.to_string(),
                });
            }
            target.custom.insert(key.clone(), value.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_global_into_defaults() {
        let defaults = Config::default();
        let mut global = Config::default();
        global.defaults.model = Some("gpt-4".to_string());

        let (result, decisions) = ConfigMerger::merge(defaults, Some(global), None, None);

        assert_eq!(result.defaults.model, Some("gpt-4".to_string()));
        assert_eq!(decisions.len(), 1);
        assert_eq!(decisions[0].source, "global");
    }

    #[test]
    fn test_merge_project_overrides_global() {
        let defaults = Config::default();
        let mut global = Config::default();
        global.defaults.model = Some("gpt-4".to_string());

        let mut project = Config::default();
        project.defaults.model = Some("gpt-3.5".to_string());

        let (result, decisions) = ConfigMerger::merge(defaults, Some(global), Some(project), None);

        assert_eq!(result.defaults.model, Some("gpt-3.5".to_string()));
        // Should have 2 decisions: one for global, one for project override
        assert!(decisions.iter().any(|d| d.source == "project"));
    }

    #[test]
    fn test_merge_env_overrides_all() {
        let defaults = Config::default();
        let mut global = Config::default();
        global.defaults.model = Some("gpt-4".to_string());

        let mut env = Config::default();
        env.defaults.model = Some("gpt-3.5-turbo".to_string());

        let (result, decisions) = ConfigMerger::merge(defaults, Some(global), None, Some(env));

        assert_eq!(result.defaults.model, Some("gpt-3.5-turbo".to_string()));
        assert!(decisions.iter().any(|d| d.source == "environment"));
    }

    #[test]
    fn test_merge_api_keys() {
        let defaults = Config::default();
        let mut global = Config::default();
        global
            .providers
            .api_keys
            .insert("openai".to_string(), "key1".to_string());

        let mut project = Config::default();
        project
            .providers
            .api_keys
            .insert("anthropic".to_string(), "key2".to_string());

        let (result, _) = ConfigMerger::merge(defaults, Some(global), Some(project), None);

        assert_eq!(
            result.providers.api_keys.get("openai"),
            Some(&"key1".to_string())
        );
        assert_eq!(
            result.providers.api_keys.get("anthropic"),
            Some(&"key2".to_string())
        );
    }

    #[test]
    fn test_merge_decisions_logged() {
        let defaults = Config::default();
        let mut global = Config::default();
        global.defaults.model = Some("gpt-4".to_string());
        global.defaults.temperature = Some(0.7);

        let (_, decisions) = ConfigMerger::merge(defaults, Some(global), None, None);

        assert_eq!(decisions.len(), 2);
        assert!(decisions.iter().any(|d| d.key == "defaults.model"));
        assert!(decisions.iter().any(|d| d.key == "defaults.temperature"));
    }

    #[test]
    fn test_merge_no_duplicate_decisions() {
        let defaults = Config::default();
        let mut global = Config::default();
        global.defaults.model = Some("gpt-4".to_string());

        let mut project = Config::default();
        project.defaults.model = Some("gpt-4".to_string()); // Same as global

        let (_, decisions) = ConfigMerger::merge(defaults, Some(global), Some(project), None);

        // Should only have one decision for model (from global), not from project
        let model_decisions: Vec<_> = decisions
            .iter()
            .filter(|d| d.key == "defaults.model")
            .collect();
        assert_eq!(model_decisions.len(), 1);
    }
}
