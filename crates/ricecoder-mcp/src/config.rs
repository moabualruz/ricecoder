//! Configuration management for MCP

use crate::error::{Error, Result};
use ricecoder_storage::types::ConfigFormat;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info};

/// MCP Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPConfig {
    pub servers: Vec<MCPServerConfig>,
    pub custom_tools: Vec<CustomToolConfig>,
    pub permissions: Vec<PermissionConfig>,
}

/// MCP Server Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPServerConfig {
    pub id: String,
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub timeout_ms: u64,
    pub auto_reconnect: bool,
    pub max_retries: u32,
}

/// Custom Tool Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomToolConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub parameters: Vec<ParameterConfig>,
    pub return_type: String,
    pub handler: String,
}

/// Parameter Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterConfig {
    pub name: String,
    pub type_: String,
    pub description: String,
    pub required: bool,
    pub default: Option<serde_json::Value>,
}

/// Permission Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionConfig {
    pub pattern: String,
    pub level: String,
    pub agent_id: Option<String>,
}

impl MCPConfig {
    /// Creates a new empty MCP configuration
    pub fn new() -> Self {
        Self {
            servers: Vec::new(),
            custom_tools: Vec::new(),
            permissions: Vec::new(),
        }
    }

    /// Adds an MCP server configuration
    pub fn add_server(&mut self, server: MCPServerConfig) {
        self.servers.push(server);
    }

    /// Adds a custom tool configuration
    pub fn add_custom_tool(&mut self, tool: CustomToolConfig) {
        self.custom_tools.push(tool);
    }

    /// Adds a permission configuration
    pub fn add_permission(&mut self, permission: PermissionConfig) {
        self.permissions.push(permission);
    }
}

impl Default for MCPConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration loader for MCP using ricecoder-storage
pub struct MCPConfigLoader;

impl MCPConfigLoader {
    /// Load MCP configuration from a file
    ///
    /// Supports YAML and JSON formats. Automatically detects format based on file extension.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<MCPConfig> {
        let path = path.as_ref();
        debug!("Loading MCP configuration from: {:?}", path);

        let content = std::fs::read_to_string(path)
            .map_err(|e| Error::ConfigError(format!("Failed to read config file: {}", e)))?;

        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| Error::ConfigError("Config file has no extension".to_string()))?;

        let format = match extension {
            "yaml" | "yml" => ConfigFormat::Yaml,
            "json" => ConfigFormat::Json,
            _ => {
                return Err(Error::ConfigError(format!(
                    "Unsupported config format: {}",
                    extension
                )))
            }
        };

        Self::load_from_string(&content, format)
    }

    /// Load MCP configuration from a string
    pub fn load_from_string(content: &str, format: ConfigFormat) -> Result<MCPConfig> {
        debug!("Parsing MCP configuration from string");

        match format {
            ConfigFormat::Yaml => serde_yaml::from_str(content)
                .map_err(|e| Error::ConfigValidationError(format!("Failed to parse YAML: {}", e))),
            ConfigFormat::Json => serde_json::from_str(content)
                .map_err(|e| Error::ConfigValidationError(format!("Failed to parse JSON: {}", e))),
            ConfigFormat::Jsonc => serde_json::from_str(content)
                .map_err(|e| Error::ConfigValidationError(format!("Failed to parse JSONC: {}", e))),
            ConfigFormat::Toml => Err(Error::ConfigValidationError(
                "TOML format is not supported for MCP configuration".to_string(),
            )),
        }
    }

    /// Load MCP configuration from multiple sources with precedence
    ///
    /// Loads configuration from project, user, and default locations.
    /// Later sources override earlier ones.
    ///
    /// Precedence (highest to lowest):
    /// 1. Project-level: `.ricecoder/mcp-servers.yaml`, `.ricecoder/custom-tools.json`, `.ricecoder/permissions.yaml`
    /// 2. User-level: `~/.ricecoder/mcp-servers.yaml`, `~/.ricecoder/custom-tools.json`, `~/.ricecoder/permissions.yaml`
    /// 3. Built-in defaults
    pub fn load_with_precedence(
        project_dir: Option<&Path>,
        user_dir: Option<&Path>,
    ) -> Result<MCPConfig> {
        let mut config = MCPConfig::new();

        // Load from user-level first (lowest priority)
        if let Some(user_dir) = user_dir {
            if let Ok(user_config) = Self::load_from_directory(user_dir) {
                info!("Loaded user-level MCP configuration");
                config = Self::merge_configs(config, user_config);
            }
        }

        // Load from project-level (highest priority)
        if let Some(project_dir) = project_dir {
            if let Ok(project_config) = Self::load_from_directory(project_dir) {
                info!("Loaded project-level MCP configuration");
                config = Self::merge_configs(config, project_config);
            }
        }

        Ok(config)
    }

    /// Load MCP configuration from a directory
    ///
    /// Looks for:
    /// - `mcp-servers.yaml` or `mcp-servers.json` for server configurations
    /// - `custom-tools.json` or `custom-tools.md` for custom tool definitions
    /// - `permissions.yaml` or `permissions.json` for permission configurations
    pub fn load_from_directory<P: AsRef<Path>>(dir: P) -> Result<MCPConfig> {
        let dir = dir.as_ref();
        let mut config = MCPConfig::new();

        // Try to load MCP servers configuration
        let servers_yaml = dir.join("mcp-servers.yaml");
        let servers_json = dir.join("mcp-servers.json");

        if servers_yaml.exists() {
            debug!("Loading MCP servers from: {:?}", servers_yaml);
            if let Ok(servers_config) = Self::load_from_file(&servers_yaml) {
                config.servers.extend(servers_config.servers);
            }
        } else if servers_json.exists() {
            debug!("Loading MCP servers from: {:?}", servers_json);
            if let Ok(servers_config) = Self::load_from_file(&servers_json) {
                config.servers.extend(servers_config.servers);
            }
        }

        // Try to load custom tools configuration
        let custom_tools_json = dir.join("custom-tools.json");
        let custom_tools_md = dir.join("custom-tools.md");

        if custom_tools_json.exists() {
            debug!("Loading custom tools from: {:?}", custom_tools_json);
            if let Ok(tools_config) = Self::load_from_file(&custom_tools_json) {
                config.custom_tools.extend(tools_config.custom_tools);
            }
        } else if custom_tools_md.exists() {
            debug!("Loading custom tools from markdown: {:?}", custom_tools_md);
            if let Ok(tools_config) = Self::load_custom_tools_from_markdown(&custom_tools_md) {
                config.custom_tools.extend(tools_config);
            }
        }

        // Try to load permissions configuration
        let permissions_yaml = dir.join("permissions.yaml");
        let permissions_json = dir.join("permissions.json");

        if permissions_yaml.exists() {
            debug!("Loading permissions from: {:?}", permissions_yaml);
            if let Ok(perms_config) = Self::load_from_file(&permissions_yaml) {
                config.permissions.extend(perms_config.permissions);
            }
        } else if permissions_json.exists() {
            debug!("Loading permissions from: {:?}", permissions_json);
            if let Ok(perms_config) = Self::load_from_file(&permissions_json) {
                config.permissions.extend(perms_config.permissions);
            }
        }

        Ok(config)
    }

    /// Load custom tools from a Markdown file with YAML frontmatter
    fn load_custom_tools_from_markdown<P: AsRef<Path>>(path: P) -> Result<Vec<CustomToolConfig>> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .map_err(|e| Error::ConfigError(format!("Failed to read markdown file: {}", e)))?;

        // Parse YAML frontmatter if present
        if content.starts_with("---") {
            if let Some(end_idx) = content[3..].find("---") {
                let frontmatter = &content[3..end_idx + 3];
                let tools: Vec<CustomToolConfig> = serde_yaml::from_str(frontmatter)
                    .map_err(|e| Error::ConfigError(format!("Failed to parse markdown frontmatter: {}", e)))?;
                return Ok(tools);
            }
        }

        // If no frontmatter, try parsing the entire content as YAML
        serde_yaml::from_str(&content)
            .map_err(|e| Error::ConfigError(format!("Failed to parse markdown content: {}", e)))
    }

    /// Merge two MCP configurations
    ///
    /// Later configuration overrides earlier one for servers and permissions.
    /// Custom tools are accumulated.
    fn merge_configs(mut base: MCPConfig, override_config: MCPConfig) -> MCPConfig {
        // For servers and permissions, override by ID/pattern
        for server in override_config.servers {
            if let Some(pos) = base.servers.iter().position(|s| s.id == server.id) {
                base.servers[pos] = server;
            } else {
                base.servers.push(server);
            }
        }

        for permission in override_config.permissions {
            if let Some(pos) = base
                .permissions
                .iter()
                .position(|p| p.pattern == permission.pattern && p.agent_id == permission.agent_id)
            {
                base.permissions[pos] = permission;
            } else {
                base.permissions.push(permission);
            }
        }

        // Accumulate custom tools
        base.custom_tools.extend(override_config.custom_tools);

        base
    }

    /// Validate MCP configuration
    pub fn validate(config: &MCPConfig) -> Result<()> {
        // Validate servers
        for server in &config.servers {
            if server.id.is_empty() {
                return Err(Error::ValidationError(
                    "Server ID cannot be empty".to_string(),
                ));
            }
            if server.command.is_empty() {
                return Err(Error::ValidationError(format!(
                    "Server '{}' has no command",
                    server.id
                )));
            }
        }

        // Validate custom tools
        for tool in &config.custom_tools {
            if tool.id.is_empty() {
                return Err(Error::ValidationError(
                    "Custom tool ID cannot be empty".to_string(),
                ));
            }
            if tool.handler.is_empty() {
                return Err(Error::ValidationError(format!(
                    "Custom tool '{}' has no handler",
                    tool.id
                )));
            }
        }

        // Validate permissions
        for perm in &config.permissions {
            if perm.pattern.is_empty() {
                return Err(Error::ValidationError(
                    "Permission pattern cannot be empty".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Save MCP configuration to a file
    pub fn save_to_file<P: AsRef<Path>>(config: &MCPConfig, path: P) -> Result<()> {
        let path = path.as_ref();
        debug!("Saving MCP configuration to: {:?}", path);

        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| Error::ConfigError("Config file has no extension".to_string()))?;

        let content = match extension {
            "yaml" | "yml" => serde_yaml::to_string(config)
                .map_err(|e| Error::ConfigError(format!("Failed to serialize to YAML: {}", e)))?,
            "json" => serde_json::to_string_pretty(config)
                .map_err(|e| Error::ConfigError(format!("Failed to serialize to JSON: {}", e)))?,
            _ => {
                return Err(Error::ConfigError(format!(
                    "Unsupported config format: {}",
                    extension
                )))
            }
        };

        std::fs::write(path, content)
            .map_err(|e| Error::ConfigError(format!("Failed to write config file: {}", e)))?;

        info!("MCP configuration saved to: {:?}", path);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_config() {
        let config = MCPConfig::new();
        assert_eq!(config.servers.len(), 0);
        assert_eq!(config.custom_tools.len(), 0);
        assert_eq!(config.permissions.len(), 0);
    }

    #[test]
    fn test_add_server() {
        let mut config = MCPConfig::new();
        let server = MCPServerConfig {
            id: "test-server".to_string(),
            name: "Test Server".to_string(),
            command: "test".to_string(),
            args: vec![],
            env: HashMap::new(),
            timeout_ms: 5000,
            auto_reconnect: true,
            max_retries: 3,
        };

        config.add_server(server);
        assert_eq!(config.servers.len(), 1);
    }

    #[test]
    fn test_add_custom_tool() {
        let mut config = MCPConfig::new();
        let tool = CustomToolConfig {
            id: "test-tool".to_string(),
            name: "Test Tool".to_string(),
            description: "A test tool".to_string(),
            category: "test".to_string(),
            parameters: vec![],
            return_type: "string".to_string(),
            handler: "test::handler".to_string(),
        };

        config.add_custom_tool(tool);
        assert_eq!(config.custom_tools.len(), 1);
    }

    #[test]
    fn test_load_yaml_config() {
        let yaml_content = r#"
servers:
  - id: test-server
    name: Test Server
    command: test
    args: []
    env: {}
    timeout_ms: 5000
    auto_reconnect: true
    max_retries: 3
custom_tools: []
permissions: []
"#;
        let config = MCPConfigLoader::load_from_string(yaml_content, ConfigFormat::Yaml)
            .expect("Failed to load YAML config");
        assert_eq!(config.servers.len(), 1);
        assert_eq!(config.servers[0].id, "test-server");
    }

    #[test]
    fn test_load_json_config() {
        let json_content = r#"{
  "servers": [
    {
      "id": "test-server",
      "name": "Test Server",
      "command": "test",
      "args": [],
      "env": {},
      "timeout_ms": 5000,
      "auto_reconnect": true,
      "max_retries": 3
    }
  ],
  "custom_tools": [],
  "permissions": []
}"#;
        let config = MCPConfigLoader::load_from_string(json_content, ConfigFormat::Json)
            .expect("Failed to load JSON config");
        assert_eq!(config.servers.len(), 1);
        assert_eq!(config.servers[0].id, "test-server");
    }

    #[test]
    fn test_validate_config_valid() {
        let mut config = MCPConfig::new();
        config.add_server(MCPServerConfig {
            id: "test-server".to_string(),
            name: "Test Server".to_string(),
            command: "test".to_string(),
            args: vec![],
            env: HashMap::new(),
            timeout_ms: 5000,
            auto_reconnect: true,
            max_retries: 3,
        });

        assert!(MCPConfigLoader::validate(&config).is_ok());
    }

    #[test]
    fn test_validate_config_empty_server_id() {
        let mut config = MCPConfig::new();
        config.add_server(MCPServerConfig {
            id: "".to_string(),
            name: "Test Server".to_string(),
            command: "test".to_string(),
            args: vec![],
            env: HashMap::new(),
            timeout_ms: 5000,
            auto_reconnect: true,
            max_retries: 3,
        });

        assert!(MCPConfigLoader::validate(&config).is_err());
    }

    #[test]
    fn test_validate_config_empty_command() {
        let mut config = MCPConfig::new();
        config.add_server(MCPServerConfig {
            id: "test-server".to_string(),
            name: "Test Server".to_string(),
            command: "".to_string(),
            args: vec![],
            env: HashMap::new(),
            timeout_ms: 5000,
            auto_reconnect: true,
            max_retries: 3,
        });

        assert!(MCPConfigLoader::validate(&config).is_err());
    }

    #[test]
    fn test_save_and_load_yaml() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("config.yaml");

        let mut config = MCPConfig::new();
        config.add_server(MCPServerConfig {
            id: "test-server".to_string(),
            name: "Test Server".to_string(),
            command: "test".to_string(),
            args: vec!["arg1".to_string()],
            env: HashMap::new(),
            timeout_ms: 5000,
            auto_reconnect: true,
            max_retries: 3,
        });

        MCPConfigLoader::save_to_file(&config, &config_path)
            .expect("Failed to save config");
        assert!(config_path.exists());

        let loaded_config = MCPConfigLoader::load_from_file(&config_path)
            .expect("Failed to load config");
        assert_eq!(loaded_config.servers.len(), 1);
        assert_eq!(loaded_config.servers[0].id, "test-server");
    }

    #[test]
    fn test_merge_configs() {
        let mut base = MCPConfig::new();
        base.add_server(MCPServerConfig {
            id: "server1".to_string(),
            name: "Server 1".to_string(),
            command: "cmd1".to_string(),
            args: vec![],
            env: HashMap::new(),
            timeout_ms: 5000,
            auto_reconnect: true,
            max_retries: 3,
        });

        let mut override_config = MCPConfig::new();
        override_config.add_server(MCPServerConfig {
            id: "server1".to_string(),
            name: "Server 1 Updated".to_string(),
            command: "cmd1_updated".to_string(),
            args: vec![],
            env: HashMap::new(),
            timeout_ms: 10000,
            auto_reconnect: false,
            max_retries: 5,
        });

        let merged = MCPConfigLoader::merge_configs(base, override_config);
        assert_eq!(merged.servers.len(), 1);
        assert_eq!(merged.servers[0].name, "Server 1 Updated");
        assert_eq!(merged.servers[0].timeout_ms, 10000);
    }
}
