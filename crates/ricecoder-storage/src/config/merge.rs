//! Configuration merging with precedence rules
//!
//! This module provides configuration merging with the following precedence:
//! environment > project > legacy > global > defaults

use tracing::debug;

use super::Config;

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
    /// Precedence: CLI > env > project > user > global > defaults
    ///
    /// Returns the merged configuration and a list of merge decisions for logging.
    pub fn merge_with_cli(
        defaults: Config,
        global: Option<Config>,
        user: Option<Config>,
        project: Option<Config>,
        env_overrides: Option<Config>,
        cli_overrides: Option<Config>,
    ) -> (Config, Vec<MergeDecision>) {
        let mut decisions = Vec::new();
        let mut result = defaults;

        // Apply global config
        if let Some(global_config) = global {
            Self::merge_into(&mut result, &global_config, "global", &mut decisions);
        }

        // Apply user config (overrides global)
        if let Some(user_config) = user {
            Self::merge_into(&mut result, &user_config, "user", &mut decisions);
        }

        // Apply project config (overrides user)
        if let Some(project_config) = project {
            Self::merge_into(&mut result, &project_config, "project", &mut decisions);
        }

        // Apply environment overrides (overrides project)
        if let Some(env_config) = env_overrides {
            Self::merge_into(&mut result, &env_config, "environment", &mut decisions);
        }

        // Apply CLI overrides (highest priority)
        if let Some(cli_config) = cli_overrides {
            Self::merge_into(&mut result, &cli_config, "cli", &mut decisions);
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

    /// Merge configurations with old precedence rules (for backward compatibility)
    ///
    /// Precedence: env > project > global > defaults
    ///
    /// Returns the merged configuration and a list of merge decisions for logging.
    pub fn merge(
        defaults: Config,
        global: Option<Config>,
        project: Option<Config>,
        env_overrides: Option<Config>,
    ) -> (Config, Vec<MergeDecision>) {
        Self::merge_with_cli(defaults, global, None, project, env_overrides, None)
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

        // Merge Governance
        for rule in &source.Governance {
            if !target.Governance.iter().any(|r| r.name == rule.name) {
                decisions.push(MergeDecision {
                    key: format!("Governance.{}", rule.name),
                    source: source_name.to_string(),
                    value: format!("{} bytes", rule.content.len()),
                });
                target.Governance.push(rule.clone());
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
