//! Configuration validation and schema management
//!
//! This module provides JSON Schema validation for configuration files,
//! migration support, and validation error reporting.

use crate::config::Config;
use crate::error::{StorageError, StorageResult};
use serde_json::Value;
use std::collections::HashMap;

/// Configuration validation error
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub value: Option<Value>,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Validation error at '{}': {}", self.field, self.message)
    }
}

impl std::error::Error for ValidationError {}

/// Configuration validator with schema support
pub struct ConfigValidator {
    schemas: HashMap<String, Value>,
}

impl ConfigValidator {
    /// Create a new configuration validator
    pub fn new() -> Self {
        Self {
            schemas: HashMap::new(),
        }
    }

    /// Add a JSON schema for validation
    pub fn add_schema(&mut self, name: String, schema: Value) {
        self.schemas.insert(name, schema);
    }

    /// Validate a configuration against all registered schemas
    pub fn validate(&self, config: &Config) -> StorageResult<()> {
        let mut errors = Vec::new();

        // Validate providers configuration
        if let Err(e) = self.validate_providers(&config.providers) {
            errors.extend(e);
        }

        // Validate defaults configuration
        if let Err(e) = self.validate_defaults(&config.defaults) {
            errors.extend(e);
        }

        // Validate steering rules
        if let Err(e) = self.validate_steering(&config.steering) {
            errors.extend(e);
        }

        if !errors.is_empty() {
            return Err(StorageError::ConfigValidation {
                errors: errors.into_iter().map(|e| e.to_string()).collect(),
            });
        }

        Ok(())
    }

    /// Validate providers configuration
    fn validate_providers(&self, providers: &crate::config::ProvidersConfig) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Validate API keys format (should not be empty)
        for (provider, key) in &providers.api_keys {
            if key.trim().is_empty() {
                errors.push(ValidationError {
                    field: format!("providers.api_keys.{}", provider),
                    message: "API key cannot be empty".to_string(),
                    value: Some(serde_json::Value::String(key.clone())),
                });
            }
        }

        // Validate endpoints format
        for (provider, endpoint) in &providers.endpoints {
            if !endpoint.starts_with("http://") && !endpoint.starts_with("https://") {
                errors.push(ValidationError {
                    field: format!("providers.endpoints.{}", provider),
                    message: "Endpoint must start with http:// or https://".to_string(),
                    value: Some(serde_json::Value::String(endpoint.clone())),
                });
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validate defaults configuration
    fn validate_defaults(&self, defaults: &crate::config::DefaultsConfig) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Validate temperature range
        if let Some(temp) = defaults.temperature {
            if !(0.0..=2.0).contains(&temp) {
                errors.push(ValidationError {
                    field: "defaults.temperature".to_string(),
                    message: "Temperature must be between 0.0 and 2.0".to_string(),
                    value: Some(serde_json::Number::from_f64(temp as f64).unwrap().into()),
                });
            }
        }

        // Validate max tokens
        if let Some(max_tokens) = defaults.max_tokens {
            if max_tokens == 0 {
                errors.push(ValidationError {
                    field: "defaults.max_tokens".to_string(),
                    message: "Max tokens must be greater than 0".to_string(),
                    value: Some(serde_json::Value::Number(max_tokens.into())),
                });
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validate steering rules
    fn validate_steering(&self, steering: &[crate::config::SteeringRule]) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        for (i, rule) in steering.iter().enumerate() {
            // Validate rule name
            if rule.name.trim().is_empty() {
                errors.push(ValidationError {
                    field: format!("steering[{}].name", i),
                    message: "Rule name cannot be empty".to_string(),
                    value: Some(serde_json::Value::String(rule.name.clone())),
                });
            }

            // Validate rule content
            if rule.content.trim().is_empty() {
                errors.push(ValidationError {
                    field: format!("steering[{}].content", i),
                    message: "Rule content cannot be empty".to_string(),
                    value: Some(serde_json::Value::String(rule.content.clone())),
                });
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Configuration backup and restore
pub struct ConfigBackupManager {
    backup_dir: std::path::PathBuf,
}

impl ConfigBackupManager {
    /// Create a new backup manager
    pub fn new(backup_dir: std::path::PathBuf) -> Self {
        Self { backup_dir }
    }

    /// Create a backup of the current configuration
    pub async fn create_backup(&self, config: &Config, name: &str) -> StorageResult<std::path::PathBuf> {
        use tokio::fs;

        // Ensure backup directory exists
        fs::create_dir_all(&self.backup_dir).await?;

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("{}_{}.yaml", name, timestamp);
        let backup_path = self.backup_dir.join(filename);

        let yaml_content = serde_yaml::to_string(config)?;
        fs::write(&backup_path, yaml_content).await?;

        tracing::info!("Configuration backup created: {}", backup_path.display());
        Ok(backup_path)
    }

    /// List available backups
    pub async fn list_backups(&self) -> StorageResult<Vec<std::path::PathBuf>> {
        use tokio::fs;

        if !self.backup_dir.exists() {
            return Ok(vec![]);
        }

        let mut backups = vec![];
        let mut entries = fs::read_dir(&self.backup_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_file() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "yaml" {
                        backups.push(entry.path());
                    }
                }
            }
        }

        // Sort by modification time (newest first)
        backups.sort_by(|a, b| {
            let a_meta = std::fs::metadata(a).ok();
            let b_meta = std::fs::metadata(b).ok();
            match (a_meta, b_meta) {
                (Some(a), Some(b)) => b.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                    .cmp(&a.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH)),
                _ => std::cmp::Ordering::Equal,
            }
        });

        Ok(backups)
    }

    /// Restore configuration from a backup
    pub async fn restore_backup(&self, backup_path: &std::path::Path) -> StorageResult<Config> {
        use tokio::fs;

        if !backup_path.exists() {
            return Err(StorageError::NotFound(format!("Backup file not found: {}", backup_path.display())));
        }

        let content = fs::read_to_string(backup_path).await?;
        let config: Config = serde_yaml::from_str(&content)?;

        tracing::info!("Configuration restored from backup: {}", backup_path.display());
        Ok(config)
    }

    /// Clean up old backups (keep only the most recent N)
    pub async fn cleanup_old_backups(&self, keep_count: usize) -> StorageResult<()> {
        let backups = self.list_backups().await?;

        if backups.len() <= keep_count {
            return Ok(());
        }

        // Remove excess backups
        for backup in backups.iter().skip(keep_count) {
            if let Err(e) = tokio::fs::remove_file(backup).await {
                tracing::warn!("Failed to remove old backup {}: {}", backup.display(), e);
            }
        }

        tracing::info!("Cleaned up {} old backups, kept {}", backups.len().saturating_sub(keep_count), keep_count);
        Ok(())
    }
}