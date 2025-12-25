//! Fallback helpers for model selection
//!
//! This module provides utilities for:
//! - Finding closest model matches
//! - Selecting smallest available models
//! - Priority-based model sorting

use crate::{
    fuzzy_search::{fuzzy_search_models, MatchScore},
    models::{Capability, ModelInfo},
};

/// Priority list for model selection
const PRIORITY_MODELS: &[&str] = &[
    "gpt-5",
    "claude-sonnet-4",
    "gpt-4-turbo",
    "claude-3-opus",
    "gemini-1.5-pro",
    "gpt-4",
    "claude-3-sonnet",
    "gpt-3.5-turbo",
];

/// Find closest model match using fuzzy search
pub fn closest_model(query: &str, models: &[ModelInfo]) -> Option<ModelInfo> {
    let matches = fuzzy_search_models(query, models, 1);
    matches.first().map(|m| m.item.clone())
}

/// Get smallest model with chat capability
pub fn get_small_model(models: &[ModelInfo]) -> Option<ModelInfo> {
    let mut chat_models: Vec<&ModelInfo> = models
        .iter()
        .filter(|m| m.capabilities.contains(&Capability::Chat))
        .collect();

    // Sort by priority first, then by context window
    chat_models.sort_by_key(|m| {
        let priority_idx = PRIORITY_MODELS
            .iter()
            .position(|&p| m.id.contains(p))
            .unwrap_or(usize::MAX);
        (priority_idx, m.context_window)
    });

    chat_models.first().map(|&m| m.clone())
}

/// Sort models by priority and capability
pub fn sort_by_priority(models: &mut Vec<ModelInfo>) {
    models.sort_by_key(|m| {
        let priority_idx = PRIORITY_MODELS
            .iter()
            .position(|&p| m.id.contains(p))
            .unwrap_or(usize::MAX);
        (priority_idx, m.context_window)
    });
}

/// Get default model from config or auto-select
pub fn default_model(
    config_model: Option<&str>,
    available_models: &[ModelInfo],
) -> Option<ModelInfo> {
    // Try config model first
    if let Some(model_id) = config_model {
        if let Some(model) = available_models.iter().find(|m| m.id == model_id) {
            return Some(model.clone());
        }
    }

    // Fall back to priority list
    for &priority in PRIORITY_MODELS {
        if let Some(model) = available_models.iter().find(|m| m.id.contains(priority)) {
            return Some(model.clone());
        }
    }

    // Last resort: smallest model
    get_small_model(available_models)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_models() -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                id: "gpt-4".to_string(),
                name: "GPT-4".to_string(),
                provider: "openai".to_string(),
                context_window: 8192,
                capabilities: vec![Capability::Chat, Capability::Code],
                pricing: None,
                is_free: false,
            },
            ModelInfo {
                id: "gpt-3.5-turbo".to_string(),
                name: "GPT-3.5 Turbo".to_string(),
                provider: "openai".to_string(),
                context_window: 4096,
                capabilities: vec![Capability::Chat],
                pricing: None,
                is_free: false,
            },
            ModelInfo {
                id: "claude-3-sonnet".to_string(),
                name: "Claude 3 Sonnet".to_string(),
                provider: "anthropic".to_string(),
                context_window: 200000,
                capabilities: vec![Capability::Chat, Capability::Vision],
                pricing: None,
                is_free: false,
            },
        ]
    }

    #[test]
    fn test_closest_model() {
        let models = create_test_models();
        let result = closest_model("gp4", &models);
        assert!(result.is_some());
        assert_eq!(result.unwrap().id, "gpt-4");
    }

    #[test]
    fn test_get_small_model() {
        let models = create_test_models();
        let result = get_small_model(&models);
        assert!(result.is_some());
        // Should prefer priority models over smallest context
        let model = result.unwrap();
        assert!(PRIORITY_MODELS.iter().any(|&p| model.id.contains(p)));
    }

    #[test]
    fn test_sort_by_priority() {
        let mut models = create_test_models();
        sort_by_priority(&mut models);
        // GPT-4 should come before GPT-3.5 due to priority
        assert_eq!(models[0].id, "gpt-4");
    }

    #[test]
    fn test_default_model_with_config() {
        let models = create_test_models();
        let result = default_model(Some("claude-3-sonnet"), &models);
        assert!(result.is_some());
        assert_eq!(result.unwrap().id, "claude-3-sonnet");
    }

    #[test]
    fn test_default_model_fallback() {
        let models = create_test_models();
        let result = default_model(Some("nonexistent"), &models);
        assert!(result.is_some());
        // Should fall back to priority list
    }
}
