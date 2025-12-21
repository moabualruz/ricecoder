use ricecoder_providers::*;

use ricecoder_providers::*;

#[test]
fn test_zen_provider_creation() {
    let provider = ZenProvider::new(Some("test-key".to_string()));
    assert!(provider.is_ok());
}

#[test]
fn test_zen_provider_creation_no_key() {
    let provider = ZenProvider::new(None);
    assert!(provider.is_ok());
}

#[test]
fn test_zen_provider_creation_empty_key() {
    let provider = ZenProvider::new(Some("".to_string()));
    assert!(provider.is_ok());
}

#[test]
fn test_zen_provider_id() {
    let provider = ZenProvider::new(Some("test-key".to_string())).unwrap();
    assert_eq!(provider.id(), "zen");
}

#[test]
fn test_zen_provider_name() {
    let provider = ZenProvider::new(Some("test-key".to_string())).unwrap();
    assert_eq!(provider.name(), "OpenCode Zen");
}

#[test]
fn test_zen_models() {
    let provider = ZenProvider::new(Some("test-key".to_string())).unwrap();
    let models = provider.models();
    assert_eq!(models.len(), 2);
    assert!(models.iter().any(|m| m.id == "zen-gpt4"));
    assert!(models.iter().any(|m| m.id == "zen-gpt4-turbo"));
}

#[test]
fn test_token_counting() {
    let provider = ZenProvider::new(Some("test-key".to_string())).unwrap();
    let tokens = provider.count_tokens("Hello, world!", "zen-gpt4").unwrap();
    assert!(tokens > 0);
}

#[test]
fn test_token_counting_invalid_model() {
    let provider = ZenProvider::new(Some("test-key".to_string())).unwrap();
    let result = provider.count_tokens("Hello, world!", "invalid-model");
    assert!(result.is_err());
}

// TODO: Fix ModelCache test API
// #[test]
// fn test_model_cache_creation() {
//     let cache = ModelCache::new();
//     assert!(cache.get().is_none());
// }

// #[test]
// fn test_model_cache_set_and_get() {
//     let mut cache = ModelCache::new();
//     let models = vec![ModelInfo {
//         id: "test".to_string(),
//         name: "Test".to_string(),
//         provider: "zen".to_string(),
//         context_window: 1000,
//         capabilities: vec![],
//         pricing: None,
//         is_free: false,
//     }];
//     cache.set(models.clone());
//     let cached = cache.get();
//     assert!(cached.is_some());
//     let cached_models = cached.unwrap();
//     assert_eq!(cached_models.len(), 1);
//     assert_eq!(cached_models[0].id, "test");
// }

// TODO: Fix HealthCheckCache test API
// #[test]
// fn test_health_check_cache_creation() {
//     let cache = HealthCheckCache::new(Duration::from_secs(300), Duration::from_millis(1000));
//     assert!(cache.get().is_none());
// }

// #[test]
// fn test_health_check_cache_set_and_get() {
//     let mut cache = HealthCheckCache::new();
//     cache.set(true);
//     assert_eq!(cache.get(), Some(true));
// }

// TODO: Fix estimate_tokens test - method is private
// #[test]
// fn test_estimate_tokens() {
//     let provider = ZenProvider::new(Some("test-key".to_string())).unwrap();
//     let tokens = provider.estimate_tokens("Hello, world!");
//     // "Hello, world!" is 13 characters, so (13 + 3) / 4 = 4 tokens
//     assert_eq!(tokens, 4);
// }

// TODO: Fix estimate_tokens test - method is private
// #[test]
// fn test_estimate_tokens() {
//     let provider = ZenProvider::new("test-key".to_string());
//     let tokens = provider.estimate_tokens("Hello, world!");
//     assert_eq!(tokens, 2);
// }

// #[test]
// fn test_estimate_tokens_empty() {
//     let provider = ZenProvider::new("test-key".to_string());
//     let tokens = provider.estimate_tokens("");
//     assert_eq!(tokens, 0);
// }

// #[test]
// fn test_estimate_tokens_single_char() {
//     let provider = ZenProvider::new("test-key".to_string());
//     let tokens = provider.estimate_tokens("a");
//     assert_eq!(tokens, 1);
// }

// TODO: Fix estimate_tokens test - method is private
// #[test]
// fn test_estimate_tokens_single_char() {
//     let provider = ZenProvider::new(Some("test-key".to_string())).unwrap();
//     let tokens = provider.estimate_tokens("a");
//     // (1 + 3) / 4 = 1 token
//     assert_eq!(tokens, 1);
// }

#[test]
fn test_endpoint_for_gpt_model() {
    let provider = ZenProvider::new(Some("test-key".to_string())).unwrap();
    assert_eq!(provider.endpoint_for_model("gpt-4"), "/responses");
    assert_eq!(provider.endpoint_for_model("gpt-3.5-turbo"), "/responses");
    assert_eq!(provider.endpoint_for_model("gpt-4-turbo"), "/responses");
}

#[test]
fn test_endpoint_for_claude_model() {
    let provider = ZenProvider::new(Some("test-key".to_string())).unwrap();
    assert_eq!(provider.endpoint_for_model("claude-3"), "/messages");
    assert_eq!(provider.endpoint_for_model("claude-3-opus"), "/messages");
    assert_eq!(provider.endpoint_for_model("claude-2"), "/messages");
}

#[test]
fn test_endpoint_for_gemini_model() {
    let provider = ZenProvider::new(Some("test-key".to_string())).unwrap();
    assert_eq!(provider.endpoint_for_model("gemini-pro"), "/models");
    assert_eq!(provider.endpoint_for_model("gemini-1.5"), "/models");
}

#[test]
fn test_endpoint_for_zen_model() {
    let provider = ZenProvider::new(Some("test-key".to_string())).unwrap();
    assert_eq!(provider.endpoint_for_model("zen-gpt4"), "/chat/completions");
    assert_eq!(
        provider.endpoint_for_model("zen-gpt4-turbo"),
        "/chat/completions"
    );
}

#[test]
fn test_endpoint_for_generic_model() {
    let provider = ZenProvider::new(Some("test-key".to_string())).unwrap();
    assert_eq!(
        provider.endpoint_for_model("unknown-model"),
        "/chat/completions"
    );
    assert_eq!(
        provider.endpoint_for_model("custom-model"),
        "/chat/completions"
    );
}
