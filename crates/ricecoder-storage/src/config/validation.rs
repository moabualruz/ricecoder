//! Configuration validation and schema management
//!
//! This module provides JSON Schema validation for configuration files,
//! migration support, and validation error reporting.

use crate::config::Config;
use crate::error::{StorageError, StorageResult};
use serde_json::Value;
use std::collections::HashMap;
use jsonschema::{Draft, JSONSchema};

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
    schemas: HashMap<String, JSONSchema>,
}

impl ConfigValidator {
    /// Create a new configuration validator
    pub fn new() -> Self {
        Self {
            schemas: HashMap::new(),
        }
    }

    /// Add a JSON schema for validation
    pub fn add_schema(&mut self, name: String, schema: Value) -> StorageResult<()> {
        let compiled_schema = JSONSchema::options()
            .with_draft(Draft::Draft7)
            .compile(&schema)
            .map_err(|e| StorageError::ConfigValidation {
                errors: vec![format!("Invalid schema '{}': {}", name, e)],
            })?;
        self.schemas.insert(name, compiled_schema);
        Ok(())
    }

    /// Load built-in schemas
    pub fn load_builtin_schemas(&mut self) -> StorageResult<()> {
        // Load provider schema
        let provider_schema = Self::get_provider_schema();
        self.add_schema("providers".to_string(), provider_schema)?;

        // Load defaults schema
        let defaults_schema = Self::get_defaults_schema();
        self.add_schema("defaults".to_string(), defaults_schema)?;

        // Load steering schema
        let steering_schema = Self::get_steering_schema();
        self.add_schema("steering".to_string(), steering_schema)?;

        Ok(())
    }

    /// Validate a configuration against all registered schemas
    pub fn validate(&self, config: &Config) -> StorageResult<()> {
        let mut errors = Vec::new();

        // Validate providers configuration
        if let Some(schema) = self.schemas.get("providers") {
            let providers_value = serde_json::to_value(&config.providers)
                .map_err(|e| StorageError::Internal(format!("Failed to serialize providers: {}", e)))?;
            if let Err(validation_errors) = schema.validate(&providers_value) {
                for error in validation_errors {
                    errors.push(format!("providers.{}: {}", error.instance_path, error.to_string()));
                }
            }
        }

        // Validate defaults configuration
        if let Some(schema) = self.schemas.get("defaults") {
            let defaults_value = serde_json::to_value(&config.defaults)
                .map_err(|e| StorageError::Internal(format!("Failed to serialize defaults: {}", e)))?;
            if let Err(validation_errors) = schema.validate(&defaults_value) {
                for error in validation_errors {
                    errors.push(format!("defaults.{}: {}", error.instance_path, error.to_string()));
                }
            }
        }

        // Validate steering rules
        if let Some(schema) = self.schemas.get("steering") {
            let steering_value = serde_json::to_value(&config.steering)
                .map_err(|e| StorageError::Internal(format!("Failed to serialize steering: {}", e)))?;
            if let Err(validation_errors) = schema.validate(&steering_value) {
                for error in validation_errors {
                    errors.push(format!("steering{}: {}", error.instance_path, error.to_string()));
                }
            }
        }

        if !errors.is_empty() {
            return Err(StorageError::ConfigValidation { errors });
        }

        Ok(())
    }

    /// Get JSON schema for providers configuration
    fn get_provider_schema() -> Value {
        serde_json::json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "default_provider": {
                    "type": ["string", "null"],
                    "enum": ["openai", "anthropic", "ollama", "google", null]
                },
                "api_keys": {
                    "type": "object",
                    "patternProperties": {
                        ".*": {
                            "type": "string",
                            "minLength": 1,
                            "description": "API key must not be empty"
                        }
                    }
                },
                "endpoints": {
                    "type": "object",
                    "patternProperties": {
                        ".*": {
                            "type": "string",
                            "pattern": "^https?://",
                            "description": "Endpoint must be a valid HTTP/HTTPS URL"
                        }
                    }
                }
            },
            "required": ["api_keys", "endpoints"],
            "additionalProperties": false
        })
    }

    /// Get JSON schema for defaults configuration
    fn get_defaults_schema() -> Value {
        serde_json::json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "model": {
                    "type": ["string", "null"],
                    "minLength": 1
                },
                "temperature": {
                    "type": ["number", "null"],
                    "minimum": 0.0,
                    "maximum": 2.0
                },
                "max_tokens": {
                    "type": ["integer", "null"],
                    "minimum": 1
                },
                "top_p": {
                    "type": ["number", "null"],
                    "minimum": 0.0,
                    "maximum": 1.0
                },
                "frequency_penalty": {
                    "type": ["number", "null"],
                    "minimum": -2.0,
                    "maximum": 2.0
                },
                "presence_penalty": {
                    "type": ["number", "null"],
                    "minimum": -2.0,
                    "maximum": 2.0
                }
            },
            "additionalProperties": false
        })
    }

    /// Get JSON schema for steering rules
    fn get_steering_schema() -> Value {
        serde_json::json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "array",
            "items": {
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "minLength": 1,
                        "description": "Rule name cannot be empty"
                    },
                    "content": {
                        "type": "string",
                        "minLength": 1,
                        "description": "Rule content cannot be empty"
                    },
                    "priority": {
                        "type": "integer",
                        "minimum": 0,
                        "maximum": 100
                    },
                    "enabled": {
                        "type": "boolean"
                    },
                    "conditions": {
                        "type": "object",
                        "additionalProperties": true
                    }
                },
                "required": ["name", "content"],
                "additionalProperties": false
            }
        })
    }
}

/// Configuration migration manager
pub struct ConfigMigrationManager {
    migrations: Vec<Box<dyn ConfigMigration>>,
}

impl ConfigMigrationManager {
    /// Create a new migration manager
    pub fn new() -> Self {
        Self {
            migrations: Vec::new(),
        }
    }

    /// Add a migration
    pub fn add_migration(&mut self, migration: Box<dyn ConfigMigration>) {
        self.migrations.push(migration);
    }

    /// Load built-in migrations
    pub fn load_builtin_migrations(&mut self) {
        // Migration from v1.0 to v1.1: Add new fields with defaults
        self.add_migration(Box::new(V1_0ToV1_1Migration));

        // Migration from v1.1 to v1.2: Rename fields
        self.add_migration(Box::new(V1_1ToV1_2Migration));
    }

    /// Apply all applicable migrations to a configuration
    pub fn migrate(&self, config: &mut Config, from_version: &str, to_version: &str) -> StorageResult<()> {
        for migration in &self.migrations {
            if migration.applies_to(from_version, to_version) {
                migration.migrate(config)?;
                tracing::info!("Applied migration: {}", migration.name());
            }
        }
        Ok(())
    }
}

/// Trait for configuration migrations
pub trait ConfigMigration {
    /// Name of the migration
    fn name(&self) -> &str;

    /// Check if this migration applies to the version transition
    fn applies_to(&self, from_version: &str, to_version: &str) -> bool;

    /// Apply the migration to the configuration
    fn migrate(&self, config: &mut Config) -> StorageResult<()>;
}

/// Migration from v1.0 to v1.1: Add new fields with defaults
struct V1_0ToV1_1Migration;

impl ConfigMigration for V1_0ToV1_1Migration {
    fn name(&self) -> &str {
        "v1.0 -> v1.1: Add new fields with defaults"
    }

    fn applies_to(&self, from_version: &str, to_version: &str) -> bool {
        from_version == "1.0" && to_version == "1.1"
    }

    fn migrate(&self, config: &mut Config) -> StorageResult<()> {
        // Add new fields that were introduced in v1.1
        if config.defaults.top_p.is_none() {
            config.defaults.top_p = Some(1.0);
        }
        if config.defaults.frequency_penalty.is_none() {
            config.defaults.frequency_penalty = Some(0.0);
        }
        if config.defaults.presence_penalty.is_none() {
            config.defaults.presence_penalty = Some(0.0);
        }

        // Ensure all steering rules have priority and enabled fields
        for rule in &mut config.steering {
            if rule.priority == 0 {
                rule.priority = 50; // Default priority
            }
            // enabled field is already defaulted in the struct
        }

        Ok(())
    }
}

/// Migration from v1.1 to v1.2: Rename fields
struct V1_1ToV1_2Migration;

impl ConfigMigration for V1_1ToV1_2Migration {
    fn name(&self) -> &str {
        "v1.1 -> v1.2: Rename deprecated fields"
    }

    fn applies_to(&self, from_version: &str, to_version: &str) -> bool {
        from_version == "1.1" && to_version == "1.2"
    }

    fn migrate(&self, config: &mut Config) -> StorageResult<()> {
        // Rename any deprecated field names
        // For example, if we had "api_key" instead of "api_keys"
        // This is a placeholder for actual field renames

        Ok(())
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