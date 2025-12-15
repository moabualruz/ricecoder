use ricecoder_providers::*;

use ricecoder_providers::*;

    #[test]
    fn test_google_provider_creation() {
        let provider = GoogleProvider::new("test-key".to_string());
        assert!(provider.is_ok());
    }

    #[test]
    fn test_google_provider_creation_empty_key() {
        let provider = GoogleProvider::new("".to_string());
        assert!(provider.is_err());
    }

    #[test]
    fn test_google_provider_id() {
        let provider = GoogleProvider::new("test-key".to_string()).unwrap();
        assert_eq!(provider.id(), "google");
    }

    #[test]
    fn test_google_provider_name() {
        let provider = GoogleProvider::new("test-key".to_string()).unwrap();
        assert_eq!(provider.name(), "Google");
    }

    #[test]
    fn test_google_models() {
        let provider = GoogleProvider::new("test-key".to_string()).unwrap();
        let models = provider.models();
        assert_eq!(models.len(), 4);
        assert!(models.iter().any(|m| m.id == "gemini-2.0-flash"));
        assert!(models.iter().any(|m| m.id == "gemini-1.5-pro"));
        assert!(models.iter().any(|m| m.id == "gemini-1.5-flash"));
        assert!(models.iter().any(|m| m.id == "gemini-1.0-pro"));
    }

    #[test]
    fn test_token_counting() {
        let provider = GoogleProvider::new("test-key".to_string()).unwrap();
        let tokens = provider
            .count_tokens("Hello, world!", "gemini-1.5-pro")
            .unwrap();
        assert!(tokens > 0);
    }

    #[test]
    fn test_token_counting_invalid_model() {
        let provider = GoogleProvider::new("test-key".to_string()).unwrap();
        let result = provider.count_tokens("Hello, world!", "invalid-model");
        assert!(result.is_err());
    }