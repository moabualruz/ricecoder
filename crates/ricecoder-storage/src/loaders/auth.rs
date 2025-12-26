//! Authentication loader for RiceCoder
//!
//! Loads provider credentials from `auth/providers.yaml` in the user config folder.
//! Supports yaml, yml, json, toml, and jsonc formats.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{StorageError, StorageResult};
use crate::manager::PathResolver;
use crate::types::{ConfigFormat, StorageDirectory};

/// Provider authentication configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProvidersAuth {
    /// Provider-specific configurations
    #[serde(default)]
    pub providers: HashMap<String, ProviderAuth>,
}

/// Authentication for a single provider
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProviderAuth {
    /// API key for the provider
    #[serde(default)]
    pub api_key: Option<String>,
    /// Base URL override for the provider
    #[serde(default)]
    pub base_url: Option<String>,
    /// Organization ID (for OpenAI, etc.)
    #[serde(default)]
    pub organization: Option<String>,
    /// Additional custom settings
    #[serde(default, flatten)]
    pub custom: HashMap<String, serde_json::Value>,
}

/// Loader for provider authentication
pub struct AuthLoader {
    auth_dir: PathBuf,
}

impl AuthLoader {
    /// Supported auth file extensions in priority order
    const AUTH_EXTENSIONS: &'static [&'static str] = &["yaml", "yml", "json", "toml", "jsonc"];

    /// Create a new auth loader with the given auth directory
    pub fn new(auth_dir: PathBuf) -> Self {
        Self { auth_dir }
    }

    /// Create an auth loader using the default path
    pub fn with_default_path() -> StorageResult<Self> {
        let base_path = PathResolver::resolve_global_path()?;
        let auth_dir = base_path.join(StorageDirectory::Auth.dir_name());
        Ok(Self::new(auth_dir))
    }

    /// Load provider authentication from the auth directory
    pub fn load(&self) -> StorageResult<ProvidersAuth> {
        if !self.auth_dir.exists() {
            return Ok(ProvidersAuth::default());
        }

        // Try each extension in priority order
        for ext in Self::AUTH_EXTENSIONS {
            let auth_file = self.auth_dir.join(format!("providers.{}", ext));
            if auth_file.exists() {
                return self.load_from_file(&auth_file);
            }
        }

        Ok(ProvidersAuth::default())
    }

    /// Load from a specific file
    fn load_from_file(&self, path: &Path) -> StorageResult<ProvidersAuth> {
        let content = fs::read_to_string(path).map_err(|e| {
            StorageError::io_error(path.to_path_buf(), crate::error::IoOperation::Read, e)
        })?;

        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("yaml");

        let format = ConfigFormat::from_extension(extension).unwrap_or(ConfigFormat::Yaml);

        self.parse(&content, format, path)
    }

    /// Parse content based on format
    fn parse(&self, content: &str, format: ConfigFormat, path: &Path) -> StorageResult<ProvidersAuth> {
        match format {
            ConfigFormat::Yaml => serde_yaml::from_str(content)
                .map_err(|e| StorageError::parse_error(path.to_path_buf(), "YAML", e.to_string())),
            ConfigFormat::Toml => toml::from_str(content)
                .map_err(|e| StorageError::parse_error(path.to_path_buf(), "TOML", e.to_string())),
            ConfigFormat::Json | ConfigFormat::Jsonc => serde_json::from_str(content)
                .map_err(|e| StorageError::parse_error(path.to_path_buf(), "JSON", e.to_string())),
        }
    }

    /// Get API key for a provider
    pub fn get_api_key(&self, provider: &str) -> StorageResult<Option<String>> {
        let auth = self.load()?;
        Ok(auth
            .providers
            .get(provider)
            .and_then(|p| p.api_key.clone()))
    }

    /// Get base URL for a provider
    pub fn get_base_url(&self, provider: &str) -> StorageResult<Option<String>> {
        let auth = self.load()?;
        Ok(auth
            .providers
            .get(provider)
            .and_then(|p| p.base_url.clone()))
    }

    /// Save provider authentication
    pub fn save(&self, auth: &ProvidersAuth) -> StorageResult<()> {
        // Ensure auth directory exists
        if !self.auth_dir.exists() {
            fs::create_dir_all(&self.auth_dir).map_err(|e| {
                StorageError::directory_creation_failed(self.auth_dir.clone(), e)
            })?;
        }

        let auth_file = self.auth_dir.join("providers.yaml");
        let content = serde_yaml::to_string(auth)
            .map_err(|e| StorageError::Internal(format!("Failed to serialize auth: {}", e)))?;

        fs::write(&auth_file, content).map_err(|e| {
            StorageError::io_error(auth_file, crate::error::IoOperation::Write, e)
        })
    }

    /// Set API key for a provider
    pub fn set_api_key(&self, provider: &str, api_key: &str) -> StorageResult<()> {
        let mut auth = self.load()?;

        let provider_auth = auth
            .providers
            .entry(provider.to_string())
            .or_insert_with(ProviderAuth::default);

        provider_auth.api_key = Some(api_key.to_string());

        self.save(&auth)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_load_empty() {
        let temp_dir = TempDir::new().unwrap();
        let loader = AuthLoader::new(temp_dir.path().join("auth"));
        let auth = loader.load().unwrap();
        assert!(auth.providers.is_empty());
    }

    #[test]
    fn test_load_yaml() {
        let temp_dir = TempDir::new().unwrap();
        let auth_dir = temp_dir.path().join("auth");
        fs::create_dir_all(&auth_dir).unwrap();

        let content = r#"
providers:
  openai:
    api_key: sk-test-key
    organization: org-123
  anthropic:
    api_key: sk-ant-test
"#;
        fs::write(auth_dir.join("providers.yaml"), content).unwrap();

        let loader = AuthLoader::new(auth_dir);
        let auth = loader.load().unwrap();

        assert_eq!(auth.providers.len(), 2);
        assert_eq!(
            auth.providers.get("openai").unwrap().api_key,
            Some("sk-test-key".to_string())
        );
        assert_eq!(
            auth.providers.get("anthropic").unwrap().api_key,
            Some("sk-ant-test".to_string())
        );
    }

    #[test]
    fn test_set_api_key() {
        let temp_dir = TempDir::new().unwrap();
        let auth_dir = temp_dir.path().join("auth");
        let loader = AuthLoader::new(auth_dir.clone());

        loader.set_api_key("openai", "sk-new-key").unwrap();

        // Reload and verify
        let auth = loader.load().unwrap();
        assert_eq!(
            auth.providers.get("openai").unwrap().api_key,
            Some("sk-new-key".to_string())
        );
    }
}
