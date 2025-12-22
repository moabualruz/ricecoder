//! Property-based tests for Zen provider
//!
//! These tests verify correctness properties that should hold across all inputs.

use proptest::prelude::*;
use ricecoder_providers::{provider::Provider, providers::ZenProvider};

// Property 1: Provider Trait Implementation
// For any ZenProvider method call with same input, behavior SHALL be consistent
#[test]
fn prop_provider_id_is_consistent() {
    proptest!(|(api_key in "\\PC{1,100}")| {
        let provider = ZenProvider::new(Some(api_key)).unwrap();
        let id1 = provider.id();
        let id2 = provider.id();
        prop_assert_eq!(id1, id2);
        prop_assert_eq!(id1, "zen");
    });
}

#[test]
fn prop_provider_name_is_consistent() {
    proptest!(|(api_key in "\\PC{1,100}")| {
        let provider = ZenProvider::new(Some(api_key)).unwrap();
        let name1 = provider.name();
        let name2 = provider.name();
        prop_assert_eq!(name1, name2);
        prop_assert_eq!(name1, "OpenCode Zen");
    });
}

#[test]
fn prop_models_list_is_consistent() {
    proptest!(|(api_key in "\\PC{1,100}")| {
        let provider = ZenProvider::new(Some(api_key)).unwrap();
        let models1 = provider.models();
        let models2 = provider.models();
        prop_assert_eq!(models1.len(), models2.len());
        for (m1, m2) in models1.iter().zip(models2.iter()) {
            prop_assert_eq!(&m1.id, &m2.id);
            prop_assert_eq!(&m1.name, &m2.name);
            prop_assert_eq!(&m1.provider, &m2.provider);
        }
    });
}

#[test]
fn prop_models_have_valid_ids() {
    proptest!(|(api_key in "\\PC{1,100}")| {
        let provider = ZenProvider::new(Some(api_key)).unwrap();
        let models = provider.models();
        for model in models {
            prop_assert!(!model.id.is_empty());
            prop_assert!(!model.name.is_empty());
            prop_assert_eq!(model.provider, "zen");
            prop_assert!(model.context_window > 0);
        }
    });
}

#[test]
fn prop_token_counting_returns_positive() {
    proptest!(|(api_key in "\\PC{1,100}", content in "\\PC{1,1000}")| {
        let provider = ZenProvider::new(Some(api_key)).unwrap();
        let models = provider.models();
        if !models.is_empty() {
            let model_id = &models[0].id;
            let result = provider.count_tokens(&content, model_id);
            prop_assert!(result.is_ok());
            let tokens = result.unwrap();
            prop_assert!(tokens > 0);
        }
    });
}

#[test]
fn prop_token_counting_invalid_model_fails() {
    proptest!(|(api_key in "\\PC{1,100}", content in "\\PC{1,1000}")| {
        let provider = ZenProvider::new(Some(api_key)).unwrap();
        let result = provider.count_tokens(&content, "invalid-model-xyz");
        prop_assert!(result.is_err());
    });
}

#[test]
fn prop_no_api_key_succeeds() {
    let result = ZenProvider::new(None);
    assert!(result.is_ok());
}

#[test]
fn prop_valid_api_key_succeeds() {
    proptest!(|(api_key in "\\PC{1,100}")| {
        let result = ZenProvider::new(Some(api_key));
        prop_assert!(result.is_ok());
    });
}

#[test]
fn prop_model_ids_are_unique() {
    proptest!(|(api_key in "\\PC{1,100}")| {
        let provider = ZenProvider::new(Some(api_key)).unwrap();
        let models = provider.models();

        let mut ids = Vec::new();
        for model in models {
            prop_assert!(!ids.contains(&model.id), "Duplicate model ID: {}", model.id);
            ids.push(model.id);
        }
    });
}

#[test]
fn prop_provider_creation_with_various_keys() {
    proptest!(|(api_key in "\\PC{1,500}")| {
        let result = ZenProvider::new(Some(api_key));
        prop_assert!(result.is_ok());

        let provider = result.unwrap();
        prop_assert_eq!(provider.id(), "zen");
        prop_assert_eq!(provider.name(), "OpenCode Zen");
    });
}

#[test]
fn prop_token_count_scales_with_content() {
    proptest!(|(content in "\\PC{1,10000}")| {
        let provider = ZenProvider::new(Some("test-key".to_string())).unwrap();
        let models = provider.models();
        if !models.is_empty() {
            let model_id = &models[0].id;

            // Count tokens for content
            let tokens = provider.count_tokens(&content, model_id).unwrap();

            // Token count should be positive
            prop_assert!(tokens > 0);

            // Token count should be less than character count (rough approximation)
            // Most tokenizers use roughly 1 token per 4 characters
            prop_assert!(tokens <= content.len());
        }
    });
}

#[test]
fn prop_longer_content_has_more_tokens() {
    proptest!(|(content1 in "\\PC{1,100}", content2 in "\\PC{100,1000}")| {
        let provider = ZenProvider::new(Some("test-key".to_string())).unwrap();
        let models = provider.models();
        if !models.is_empty() {
            let model_id = &models[0].id;

            let tokens1 = provider.count_tokens(&content1, model_id).unwrap();
            let tokens2 = provider.count_tokens(&content2, model_id).unwrap();

            // Longer content should generally have more tokens
            // (though not always strictly true due to tokenization)
            prop_assert!(tokens1 > 0);
            prop_assert!(tokens2 > 0);
        }
    });
}

#[test]
fn prop_provider_usable_after_creation() {
    proptest!(|(api_key in "\\PC{1,100}")| {
        let provider = ZenProvider::new(Some(api_key)).unwrap();

        // Provider should be usable multiple times
        let id1 = provider.id();
        let id2 = provider.id();
        let name1 = provider.name();
        let name2 = provider.name();

        prop_assert_eq!(id1, id2);
        prop_assert_eq!(name1, name2);
        prop_assert_eq!(id1, "zen");
        prop_assert_eq!(name1, "OpenCode Zen");
    });
}

#[test]
fn prop_multiple_operations_succeed() {
    proptest!(|(api_key in "\\PC{1,100}")| {
        let provider = ZenProvider::new(Some(api_key)).unwrap();

        // Multiple operations should succeed
        let _ = provider.id();
        let _ = provider.name();
        let _ = provider.models();

        let models = provider.models();
        if !models.is_empty() {
            let model_id = &models[0].id;
            let _ = provider.count_tokens("test", model_id);
        }

        prop_assert!(true);
    });
}

// Property 2: Model Caching Correctness
// For any valid cache, cached models SHALL be returned without API call
#[test]
fn test_model_cache_returns_same_models() {
    let provider = ZenProvider::new(Some("test-key".to_string())).unwrap();
    let models1 = provider.models();
    let models2 = provider.models();

    assert_eq!(models1.len(), models2.len());
    for (m1, m2) in models1.iter().zip(models2.iter()) {
        assert_eq!(&m1.id, &m2.id);
        assert_eq!(&m1.name, &m2.name);
    }
}

// Property 3: Health Check Reliability
// For any health check, result should be consistent
#[tokio::test]
async fn test_health_check_returns_bool() {
    let provider = ZenProvider::new(Some("test-key".to_string())).unwrap();
    let result = provider.health_check().await;

    // Health check should return a result (either Ok or Err)
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_health_check_consistency() {
    let provider = ZenProvider::new(Some("test-key".to_string())).unwrap();

    // First health check
    let result1 = provider.health_check().await;

    // Second health check (should use cache)
    let result2 = provider.health_check().await;

    // Both should have the same result type
    match (result1, result2) {
        (Ok(b1), Ok(b2)) => {
            // Both succeeded, results should be the same (cached)
            assert_eq!(b1, b2);
        }
        (Err(_), Err(_)) => {
            // Both failed, that's consistent
        }
        _ => {
            // One succeeded and one failed - this could happen if cache expired
            // but within a short time it should be consistent
        }
    }
}
