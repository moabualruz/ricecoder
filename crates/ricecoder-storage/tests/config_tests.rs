use ricecoder_storage::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(config.providers.api_keys.is_empty());
        assert!(config.defaults.model.is_none());
        assert!(config.steering.is_empty());
    }
}