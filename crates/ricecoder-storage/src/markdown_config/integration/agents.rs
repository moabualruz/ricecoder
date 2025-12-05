//! Integration with ricecoder-agents for markdown-based agent configuration

use crate::markdown_config::error::MarkdownConfigResult;
use crate::markdown_config::loader::{ConfigFile, ConfigFileType, ConfigurationLoader};
use crate::markdown_config::types::AgentConfig;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Type alias for registration results: (success_count, error_count, errors)
pub type RegistrationResult = (usize, usize, Vec<(String, String)>);

/// Trait for registering agent configurations
///
/// This trait allows ricecoder-storage to register agent configurations without
/// directly depending on ricecoder-agents, avoiding circular dependencies.
pub trait AgentRegistrar: Send + Sync {
    /// Register an agent configuration
    fn register_agent(&mut self, agent: AgentConfig) -> Result<(), String>;
}

/// Integration layer for agent configuration with ricecoder-agents
///
/// This struct provides methods to discover, load, and register agent configurations
/// from markdown files with the ricecoder-agents subsystem.
pub struct AgentConfigIntegration {
    loader: Arc<ConfigurationLoader>,
}

impl AgentConfigIntegration {
    /// Create a new agent configuration integration
    pub fn new(loader: Arc<ConfigurationLoader>) -> Self {
        Self { loader }
    }

    /// Discover agent configuration files in the given paths
    ///
    /// # Arguments
    /// * `paths` - Directories to search for agent markdown files
    ///
    /// # Returns
    /// A vector of discovered agent configuration files
    pub fn discover_agent_configs(&self, paths: &[PathBuf]) -> MarkdownConfigResult<Vec<ConfigFile>> {
        let all_files = self.loader.discover(paths)?;

        // Filter to only agent configuration files
        let agent_files: Vec<ConfigFile> = all_files
            .into_iter()
            .filter(|f| f.config_type == ConfigFileType::Agent)
            .collect();

        debug!("Discovered {} agent configuration files", agent_files.len());
        Ok(agent_files)
    }

    /// Load agent configurations from markdown files
    ///
    /// # Arguments
    /// * `paths` - Directories to search for agent markdown files
    ///
    /// # Returns
    /// A tuple of (loaded_agents, errors)
    pub async fn load_agent_configs(
        &self,
        paths: &[PathBuf],
    ) -> MarkdownConfigResult<(Vec<AgentConfig>, Vec<(PathBuf, String)>)> {
        let files = self.discover_agent_configs(paths)?;

        let mut agents = Vec::new();
        let mut errors = Vec::new();

        for file in files {
            match self.loader.load(&file).await {
                Ok(config) => {
                    match config {
                        crate::markdown_config::loader::LoadedConfig::Agent(agent) => {
                            debug!("Loaded agent configuration: {}", agent.name);
                            agents.push(agent);
                        }
                        _ => {
                            warn!("Expected agent configuration but got different type from {}", file.path.display());
                            errors.push((
                                file.path,
                                "Expected agent configuration but got different type".to_string(),
                            ));
                        }
                    }
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    warn!("Failed to load agent configuration from {}: {}", file.path.display(), error_msg);
                    errors.push((file.path, error_msg));
                }
            }
        }

        info!("Loaded {} agent configurations", agents.len());
        Ok((agents, errors))
    }

    /// Register agent configurations with a registrar
    ///
    /// This method registers agent configurations using a generic registrar trait,
    /// allowing integration with any agent registry implementation.
    ///
    /// # Arguments
    /// * `agents` - Agent configurations to register
    /// * `registrar` - The agent registrar to register with
    ///
    /// # Returns
    /// A tuple of (successful_count, error_count, errors)
    pub fn register_agents(
        &self,
        agents: Vec<AgentConfig>,
        registrar: &mut dyn AgentRegistrar,
    ) -> MarkdownConfigResult<RegistrationResult> {
        let mut success_count = 0;
        let mut error_count = 0;
        let mut errors = Vec::new();

        for agent in agents {
            // Validate agent configuration
            if let Err(e) = agent.validate() {
                error_count += 1;
                let error_msg = format!("Invalid agent configuration: {}", e);
                warn!("Failed to register agent '{}': {}", agent.name, error_msg);
                errors.push((agent.name.clone(), error_msg));
                continue;
            }

            debug!("Registering agent: {}", agent.name);

            // Register the agent using the registrar
            match registrar.register_agent(agent.clone()) {
                Ok(_) => {
                    success_count += 1;
                    info!("Registered agent: {}", agent.name);
                }
                Err(e) => {
                    error_count += 1;
                    warn!("Failed to register agent '{}': {}", agent.name, e);
                    errors.push((agent.name.clone(), e));
                }
            }
        }

        debug!(
            "Agent registration complete: {} successful, {} failed",
            success_count, error_count
        );

        Ok((success_count, error_count, errors))
    }

    /// Load and register agent configurations in one operation
    ///
    /// # Arguments
    /// * `paths` - Directories to search for agent markdown files
    /// * `registrar` - The agent registrar to register with
    ///
    /// # Returns
    /// A tuple of (successful_count, error_count, errors)
    pub async fn load_and_register_agents(
        &self,
        paths: &[PathBuf],
        registrar: &mut dyn AgentRegistrar,
    ) -> MarkdownConfigResult<(usize, usize, Vec<(String, String)>)> {
        let (agents, load_errors) = self.load_agent_configs(paths).await?;

        let (success, errors, mut reg_errors) = self.register_agents(agents, registrar)?;

        // Combine load and registration errors
        for (path, msg) in load_errors {
            reg_errors.push((path.display().to_string(), msg));
        }

        Ok((success, errors, reg_errors))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown_config::registry::ConfigRegistry;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_agent_file(dir: &PathBuf, name: &str, content: &str) -> PathBuf {
        let path = dir.join(format!("{}.agent.md", name));
        fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn test_discover_agent_configs() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().to_path_buf();

        // Create test agent files
        create_test_agent_file(&dir_path, "agent1", "---\nname: agent1\n---\nTest");
        create_test_agent_file(&dir_path, "agent2", "---\nname: agent2\n---\nTest");

        // Create a non-agent file
        fs::write(dir_path.join("mode1.mode.md"), "---\nname: mode1\n---\nTest").unwrap();

        let registry = Arc::new(ConfigRegistry::new());
        let loader = Arc::new(ConfigurationLoader::new(registry));
        let integration = AgentConfigIntegration::new(loader);

        let discovered = integration.discover_agent_configs(&[dir_path]).unwrap();

        assert_eq!(discovered.len(), 2);
        assert!(discovered.iter().all(|f| f.config_type == ConfigFileType::Agent));
    }

    #[tokio::test]
    async fn test_load_agent_configs() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().to_path_buf();

        let agent_content = r#"---
name: test-agent
description: A test agent
model: gpt-4
temperature: 0.7
max_tokens: 2000
---
You are a helpful assistant"#;

        create_test_agent_file(&dir_path, "test-agent", agent_content);

        let registry = Arc::new(ConfigRegistry::new());
        let loader = Arc::new(ConfigurationLoader::new(registry));
        let integration = AgentConfigIntegration::new(loader);

        let (agents, errors) = integration.load_agent_configs(&[dir_path]).await.unwrap();

        assert_eq!(agents.len(), 1);
        assert_eq!(errors.len(), 0);
        assert_eq!(agents[0].name, "test-agent");
        assert_eq!(agents[0].model, Some("gpt-4".to_string()));
    }

    #[tokio::test]
    async fn test_load_agent_configs_with_errors() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().to_path_buf();

        // Create a valid agent file
        let valid_content = r#"---
name: valid-agent
---
Valid agent"#;
        create_test_agent_file(&dir_path, "valid-agent", valid_content);

        // Create an invalid agent file (missing frontmatter)
        fs::write(dir_path.join("invalid.agent.md"), "# No frontmatter\nJust markdown").unwrap();

        let registry = Arc::new(ConfigRegistry::new());
        let loader = Arc::new(ConfigurationLoader::new(registry));
        let integration = AgentConfigIntegration::new(loader);

        let (agents, errors) = integration.load_agent_configs(&[dir_path]).await.unwrap();

        assert_eq!(agents.len(), 1);
        assert_eq!(errors.len(), 1);
        assert_eq!(agents[0].name, "valid-agent");
    }

    #[test]
    fn test_register_agents() {
        let registry = Arc::new(ConfigRegistry::new());
        let loader = Arc::new(ConfigurationLoader::new(registry));
        let integration = AgentConfigIntegration::new(loader);

        let agents = vec![
            AgentConfig {
                name: "agent1".to_string(),
                description: Some("Test agent 1".to_string()),
                prompt: "You are agent 1".to_string(),
                model: Some("gpt-4".to_string()),
                temperature: Some(0.7),
                max_tokens: Some(2000),
                tools: vec![],
            },
            AgentConfig {
                name: "agent2".to_string(),
                description: Some("Test agent 2".to_string()),
                prompt: "You are agent 2".to_string(),
                model: None,
                temperature: None,
                max_tokens: None,
                tools: vec![],
            },
        ];

        struct MockRegistrar;
        impl AgentRegistrar for MockRegistrar {
            fn register_agent(&mut self, _agent: AgentConfig) -> Result<(), String> {
                Ok(())
            }
        }

        let mut registrar = MockRegistrar;
        let (success, errors, error_list) = integration
            .register_agents(agents, &mut registrar)
            .unwrap();

        assert_eq!(success, 2);
        assert_eq!(errors, 0);
        assert_eq!(error_list.len(), 0);
    }

    #[test]
    fn test_register_invalid_agent() {
        let registry = Arc::new(ConfigRegistry::new());
        let loader = Arc::new(ConfigurationLoader::new(registry));
        let integration = AgentConfigIntegration::new(loader);

        let agents = vec![
            AgentConfig {
                name: String::new(), // Invalid: empty name
                description: None,
                prompt: "Test".to_string(),
                model: None,
                temperature: None,
                max_tokens: None,
                tools: vec![],
            },
        ];

        struct MockRegistrar;
        impl AgentRegistrar for MockRegistrar {
            fn register_agent(&mut self, _agent: AgentConfig) -> Result<(), String> {
                Ok(())
            }
        }

        let mut registrar = MockRegistrar;
        let (success, errors, error_list) = integration
            .register_agents(agents, &mut registrar)
            .unwrap();

        assert_eq!(success, 0);
        assert_eq!(errors, 1);
        assert_eq!(error_list.len(), 1);
    }
}
