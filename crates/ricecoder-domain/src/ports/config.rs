//! Configuration source port interfaces
//!
//! This module defines the contracts for configuration access.
//! Implementations in infrastructure crates provide file-based,
//! environment-based, or remote configuration sources.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::errors::*;

// ============================================================================
// Configuration Value Objects
// ============================================================================

/// Configuration value that can be of different types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ConfigValue {
    /// String value
    String(String),
    /// Integer value
    Integer(i64),
    /// Float value
    Float(f64),
    /// Boolean value
    Boolean(bool),
    /// Array of values
    Array(Vec<ConfigValue>),
    /// Nested configuration object
    Object(HashMap<String, ConfigValue>),
    /// Null/missing value
    Null,
}

impl ConfigValue {
    /// Try to get as string
    pub fn as_str(&self) -> Option<&str> {
        match self {
            ConfigValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Try to get as integer
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            ConfigValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Try to get as float
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            ConfigValue::Float(f) => Some(*f),
            ConfigValue::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }

    /// Try to get as boolean
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ConfigValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Check if value is null
    pub fn is_null(&self) -> bool {
        matches!(self, ConfigValue::Null)
    }
}

/// Configuration source metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSourceInfo {
    /// Source identifier (e.g., "file", "env", "remote")
    pub source_type: String,
    /// Source location (e.g., file path, URL)
    pub location: Option<String>,
    /// Last loaded timestamp
    pub last_loaded: chrono::DateTime<chrono::Utc>,
    /// Whether source supports hot reload
    pub supports_reload: bool,
}

// ============================================================================
// Configuration Source Ports (ISP-Compliant)
// ============================================================================

/// Read-only configuration source (ISP: 5 methods max)
#[async_trait]
pub trait ConfigReader: Send + Sync {
    /// Get a configuration value by key
    ///
    /// Keys use dot notation: "database.host", "logging.level"
    async fn get(&self, key: &str) -> DomainResult<Option<ConfigValue>>;

    /// Get a configuration value with a default
    async fn get_or_default(&self, key: &str, default: ConfigValue) -> DomainResult<ConfigValue> {
        Ok(self.get(key).await?.unwrap_or(default))
    }

    /// Check if a configuration key exists
    async fn contains(&self, key: &str) -> DomainResult<bool>;

    /// Get all keys under a prefix
    ///
    /// For example, `keys("database")` returns ["database.host", "database.port"]
    async fn keys(&self, prefix: &str) -> DomainResult<Vec<String>>;

    /// Get source information
    fn source_info(&self) -> ConfigSourceInfo;
}

/// Configuration writer for mutable sources
#[async_trait]
pub trait ConfigWriter: Send + Sync {
    /// Set a configuration value
    async fn set(&self, key: &str, value: ConfigValue) -> DomainResult<()>;

    /// Remove a configuration key
    async fn remove(&self, key: &str) -> DomainResult<bool>;

    /// Save/persist configuration changes
    async fn save(&self) -> DomainResult<()>;
}

/// Configuration source with reload capability
#[async_trait]
pub trait ConfigReloader: Send + Sync {
    /// Reload configuration from source
    async fn reload(&self) -> DomainResult<()>;

    /// Register a callback for configuration changes
    ///
    /// Returns a handle that can be used to unregister
    fn on_change(&self, callback: Box<dyn Fn(&str, &ConfigValue) + Send + Sync>) -> u64;

    /// Unregister a change callback
    fn off_change(&self, handle: u64);
}

/// Combined configuration source trait
pub trait ConfigSource: ConfigReader + ConfigWriter + ConfigReloader {}

/// Blanket implementation for types implementing all sub-traits
impl<T> ConfigSource for T where T: ConfigReader + ConfigWriter + ConfigReloader {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_value_as_str() {
        let value = ConfigValue::String("test".to_string());
        assert_eq!(value.as_str(), Some("test"));

        let value = ConfigValue::Integer(42);
        assert_eq!(value.as_str(), None);
    }

    #[test]
    fn test_config_value_as_i64() {
        let value = ConfigValue::Integer(42);
        assert_eq!(value.as_i64(), Some(42));

        let value = ConfigValue::String("test".to_string());
        assert_eq!(value.as_i64(), None);
    }

    #[test]
    fn test_config_value_as_f64() {
        let value = ConfigValue::Float(3.14);
        assert_eq!(value.as_f64(), Some(3.14));

        // Integer should convert to float
        let value = ConfigValue::Integer(42);
        assert_eq!(value.as_f64(), Some(42.0));
    }

    #[test]
    fn test_config_value_is_null() {
        assert!(ConfigValue::Null.is_null());
        assert!(!ConfigValue::Boolean(true).is_null());
    }
}
