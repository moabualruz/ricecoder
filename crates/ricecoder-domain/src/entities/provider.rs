//! Provider entity for AI provider configuration

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Provider configuration entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    pub id: String,
    pub name: String,
    pub provider_type: ProviderType,
    pub base_url: Option<String>,
    pub models: Vec<ModelInfo>,
    pub config: HashMap<String, serde_json::Value>,
    pub is_active: bool,
}

impl Provider {
    /// Create a new provider
    pub fn new(id: String, name: String, provider_type: ProviderType) -> Self {
        Self {
            id,
            name,
            provider_type,
            base_url: None,
            models: Vec::new(),
            config: HashMap::new(),
            is_active: true,
        }
    }

    /// Add a model to the provider
    pub fn add_model(&mut self, model: ModelInfo) {
        self.models.push(model);
    }

    /// Get a model by ID
    pub fn get_model(&self, model_id: &str) -> Option<&ModelInfo> {
        self.models.iter().find(|m| m.id == model_id)
    }

    /// Check if provider supports a model
    pub fn supports_model(&self, model_id: &str) -> bool {
        self.models.iter().any(|m| m.id == model_id)
    }
}

/// Provider type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderType {
    OpenAI,
    Anthropic,
    Google,
    Local,
    Custom,
}

/// Model information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub context_window: usize,
    pub supports_function_calling: bool,
    pub supports_vision: bool,
    pub cost_per_1m_input: Option<f64>,
    pub cost_per_1m_output: Option<f64>,
}

impl ModelInfo {
    /// Create a new model info
    pub fn new(id: String, name: String, context_window: usize) -> Self {
        Self {
            id,
            name,
            context_window,
            supports_function_calling: false,
            supports_vision: false,
            cost_per_1m_input: None,
            cost_per_1m_output: None,
        }
    }

    /// Enable function calling
    pub fn with_function_calling(mut self) -> Self {
        self.supports_function_calling = true;
        self
    }

    /// Enable vision
    pub fn with_vision(mut self) -> Self {
        self.supports_vision = true;
        self
    }

    /// Set pricing
    pub fn with_pricing(mut self, input_cost: f64, output_cost: f64) -> Self {
        self.cost_per_1m_input = Some(input_cost);
        self.cost_per_1m_output = Some(output_cost);
        self
    }
}
