//! Provider management for RiceCoder TUI
//!
//! Manages AI providers, models, and their status:
//! - Provider connection status
//! - Model selection and cycling
//! - Provider configuration
//!
//! # DDD Layer: Application
//! Provider and model management for the prompt system.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use ratatui::style::Color;

/// Provider status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ProviderStatus {
    #[default]
    Unknown,
    Connected,
    Disconnected,
    Error,
    RateLimited,
}

impl ProviderStatus {
    /// Get status color
    pub fn color(&self) -> Color {
        match self {
            Self::Connected => Color::Green,
            Self::Disconnected => Color::Gray,
            Self::Error => Color::Red,
            Self::RateLimited => Color::Yellow,
            Self::Unknown => Color::DarkGray,
        }
    }
    
    /// Get status indicator
    pub fn indicator(&self) -> &'static str {
        match self {
            Self::Connected => "●",
            Self::Disconnected => "○",
            Self::Error => "✗",
            Self::RateLimited => "◐",
            Self::Unknown => "?",
        }
    }
}

/// A provider definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    pub id: String,
    pub name: String,
    pub models: Vec<Model>,
    #[serde(default)]
    pub status: ProviderStatus,
    #[serde(default)]
    pub api_key_set: bool,
}

/// A model definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub id: String,
    pub name: String,
    pub provider_id: String,
    #[serde(default)]
    pub context_length: Option<u32>,
    #[serde(default)]
    pub supports_vision: bool,
    #[serde(default)]
    pub supports_tools: bool,
}

impl Model {
    /// Parse model string "provider/model" format
    pub fn parse(s: &str) -> Option<(String, String)> {
        let parts: Vec<&str> = s.splitn(2, '/').collect();
        if parts.len() == 2 {
            Some((parts[0].to_string(), parts[1].to_string()))
        } else {
            None
        }
    }
    
    /// Format as "provider/model"
    pub fn format(&self) -> String {
        format!("{}/{}", self.provider_id, self.id)
    }
}

/// Provider manager
#[derive(Debug, Default)]
pub struct ProviderManager {
    providers: HashMap<String, Provider>,
    current_provider: Option<String>,
    current_model: Option<String>,
    model_override: Option<String>,
}

impl ProviderManager {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Register a provider
    pub fn register(&mut self, provider: Provider) {
        self.providers.insert(provider.id.clone(), provider);
    }
    
    /// Get provider by ID
    pub fn get(&self, id: &str) -> Option<&Provider> {
        self.providers.get(id)
    }
    
    /// Get all providers
    pub fn all(&self) -> impl Iterator<Item = &Provider> {
        self.providers.values()
    }
    
    /// Get current provider
    pub fn current_provider(&self) -> Option<&Provider> {
        self.current_provider.as_ref().and_then(|id| self.providers.get(id))
    }
    
    /// Get current model
    pub fn current_model(&self) -> Option<&Model> {
        let provider = self.current_provider()?;
        let model_id = self.current_model.as_ref()?;
        provider.models.iter().find(|m| &m.id == model_id)
    }
    
    /// Set current provider and model
    pub fn set_current(&mut self, provider_id: &str, model_id: &str) {
        self.current_provider = Some(provider_id.to_string());
        self.current_model = Some(model_id.to_string());
    }
    
    /// Set model override (temporary)
    pub fn set_override(&mut self, model: Option<String>) {
        self.model_override = model;
    }
    
    /// Get effective model (override or current)
    pub fn effective_model(&self) -> Option<&Model> {
        if let Some(override_str) = &self.model_override {
            if let Some((provider_id, model_id)) = Model::parse(override_str) {
                if let Some(provider) = self.providers.get(&provider_id) {
                    return provider.models.iter().find(|m| m.id == model_id);
                }
            }
        }
        self.current_model()
    }
    
    /// Cycle to next provider
    pub fn cycle_provider(&mut self, direction: i8) -> Option<&Provider> {
        let ids: Vec<_> = self.providers.keys().cloned().collect();
        if ids.is_empty() { return None; }
        
        let current_idx = self.current_provider
            .as_ref()
            .and_then(|id| ids.iter().position(|i| i == id))
            .unwrap_or(0);
        
        let new_idx = if direction > 0 {
            (current_idx + 1) % ids.len()
        } else {
            (current_idx + ids.len() - 1) % ids.len()
        };
        
        self.current_provider = Some(ids[new_idx].clone());
        if let Some(provider) = self.providers.get(&ids[new_idx]) {
            if let Some(model) = provider.models.first() {
                self.current_model = Some(model.id.clone());
            }
        }
        self.current_provider()
    }
    
    /// Cycle to next model within current provider
    pub fn cycle_model(&mut self, direction: i8) -> Option<&Model> {
        let provider = self.current_provider()?;
        let models = &provider.models;
        if models.is_empty() { return None; }
        
        let current_idx = self.current_model
            .as_ref()
            .and_then(|id| models.iter().position(|m| &m.id == id))
            .unwrap_or(0);
        
        let new_idx = if direction > 0 {
            (current_idx + 1) % models.len()
        } else {
            (current_idx + models.len() - 1) % models.len()
        };
        
        self.current_model = Some(models[new_idx].id.clone());
        self.current_model()
    }
    
    /// Update provider status
    pub fn update_status(&mut self, provider_id: &str, status: ProviderStatus) {
        if let Some(provider) = self.providers.get_mut(provider_id) {
            provider.status = status;
        }
    }
    
    /// Get parsed model info for display
    pub fn parsed(&self) -> ParsedModel {
        if let Some(model) = self.effective_model() {
            ParsedModel {
                provider: model.provider_id.clone(),
                model: model.name.clone(),
                model_id: model.id.clone(),
            }
        } else {
            ParsedModel::default()
        }
    }
}

/// Parsed model for display
#[derive(Debug, Clone, Default)]
pub struct ParsedModel {
    pub provider: String,
    pub model: String,
    pub model_id: String,
}

/// Default providers
pub fn default_providers() -> Vec<Provider> {
    vec![
        Provider {
            id: "anthropic".to_string(),
            name: "Anthropic".to_string(),
            status: ProviderStatus::Unknown,
            api_key_set: false,
            models: vec![
                Model {
                    id: "claude-sonnet-4-20250514".to_string(),
                    name: "Claude Sonnet 4".to_string(),
                    provider_id: "anthropic".to_string(),
                    context_length: Some(200000),
                    supports_vision: true,
                    supports_tools: true,
                },
                Model {
                    id: "claude-3-5-sonnet-20241022".to_string(),
                    name: "Claude 3.5 Sonnet".to_string(),
                    provider_id: "anthropic".to_string(),
                    context_length: Some(200000),
                    supports_vision: true,
                    supports_tools: true,
                },
            ],
        },
        Provider {
            id: "openai".to_string(),
            name: "OpenAI".to_string(),
            status: ProviderStatus::Unknown,
            api_key_set: false,
            models: vec![
                Model {
                    id: "gpt-4o".to_string(),
                    name: "GPT-4o".to_string(),
                    provider_id: "openai".to_string(),
                    context_length: Some(128000),
                    supports_vision: true,
                    supports_tools: true,
                },
            ],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_provider_status_color() {
        assert_eq!(ProviderStatus::Connected.color(), Color::Green);
        assert_eq!(ProviderStatus::Error.color(), Color::Red);
    }
    
    #[test]
    fn test_model_parse() {
        let (provider, model) = Model::parse("anthropic/claude-3").unwrap();
        assert_eq!(provider, "anthropic");
        assert_eq!(model, "claude-3");
    }
    
    #[test]
    fn test_provider_manager_cycle() {
        let mut manager = ProviderManager::new();
        for provider in default_providers() {
            manager.register(provider);
        }
        manager.set_current("anthropic", "claude-sonnet-4-20250514");
        
        manager.cycle_provider(1);
        assert!(manager.current_provider().is_some());
    }
    
    #[test]
    fn test_model_cycle() {
        let mut manager = ProviderManager::new();
        for provider in default_providers() {
            manager.register(provider);
        }
        manager.set_current("anthropic", "claude-sonnet-4-20250514");
        
        manager.cycle_model(1);
        assert_eq!(manager.current_model().unwrap().id, "claude-3-5-sonnet-20241022");
    }
}
