use ricecoder_storage::*;

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
