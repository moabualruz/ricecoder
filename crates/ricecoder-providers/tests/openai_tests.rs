use ricecoder_providers::*;

use ricecoder_providers::*;

    #[test]
    fn test_openai_provider_creation() {
        let provider = OpenAiProvider::new("test-key".to_string());
        assert!(provider.is_ok());
    }

    #[test]
    fn test_openai_provider_creation_empty_key() {
        let provider = OpenAiProvider::new("".to_string());
        assert!(provider.is_err());
    }

    #[test]
    fn test_openai_provider_id() {
        let provider = OpenAiProvider::new("test-key".to_string()).unwrap();
        assert_eq!(provider.id(), "openai");
    }

    #[test]
    fn test_openai_provider_name() {
        let provider = OpenAiProvider::new("test-key".to_string()).unwrap();
        assert_eq!(provider.name(), "OpenAI");
    }

    #[test]
    fn test_openai_models() {
        let provider = OpenAiProvider::new("test-key".to_string()).unwrap();
        let models = provider.models();
        assert_eq!(models.len(), 4);
        assert!(models.iter().any(|m| m.id == "gpt-4"));
        assert!(models.iter().any(|m| m.id == "gpt-4-turbo"));
        assert!(models.iter().any(|m| m.id == "gpt-4o"));
        assert!(models.iter().any(|m| m.id == "gpt-3.5-turbo"));
    }

    #[test]
    fn test_token_counting() {
        let provider = OpenAiProvider::new("test-key".to_string()).unwrap();
        let tokens = provider.count_tokens("Hello, world!", "gpt-4").unwrap();
        assert!(tokens > 0);
    }

    #[test]
    fn test_token_counting_invalid_model() {
        let provider = OpenAiProvider::new("test-key".to_string()).unwrap();
        let result = provider.count_tokens("Hello, world!", "invalid-model");
        assert!(result.is_err());
    }