//! Configuration registry for storing and retrieving parsed configurations

use crate::markdown_config::error::{MarkdownConfigError, MarkdownConfigResult};
use crate::markdown_config::types::{AgentConfig, CommandConfig, ModeConfig};
use std::collections::HashMap;
use std::sync::RwLock;

/// Central registry for all loaded configurations
///
/// Provides thread-safe storage and retrieval of agent, mode, and command configurations.
/// Uses RwLock for concurrent access patterns where reads are more frequent than writes.
#[derive(Debug)]
pub struct ConfigRegistry {
    /// Registered agent configurations
    agents: RwLock<HashMap<String, AgentConfig>>,
    /// Registered mode configurations
    modes: RwLock<HashMap<String, ModeConfig>>,
    /// Registered command configurations
    commands: RwLock<HashMap<String, CommandConfig>>,
}

impl ConfigRegistry {
    /// Create a new configuration registry
    pub fn new() -> Self {
        Self {
            agents: RwLock::new(HashMap::new()),
            modes: RwLock::new(HashMap::new()),
            commands: RwLock::new(HashMap::new()),
        }
    }

    // ============ Agent Registration ============

    /// Register an agent configuration
    ///
    /// # Arguments
    /// * `config` - The agent configuration to register
    ///
    /// # Errors
    /// Returns an error if:
    /// - An agent with the same name already exists
    /// - The configuration is invalid
    pub fn register_agent(&self, config: AgentConfig) -> MarkdownConfigResult<()> {
        // Validate configuration
        config.validate().map_err(|e| {
            MarkdownConfigError::validation_error(format!("Invalid agent configuration: {}", e))
        })?;

        let mut agents = self.agents.write().map_err(|e| {
            MarkdownConfigError::registration_error(format!("Failed to acquire write lock: {}", e))
        })?;

        // Check for duplicate registration
        if agents.contains_key(&config.name) {
            return Err(MarkdownConfigError::registration_error(format!(
                "Agent '{}' is already registered",
                config.name
            )));
        }

        agents.insert(config.name.clone(), config);
        Ok(())
    }

    /// Get an agent configuration by name
    ///
    /// # Arguments
    /// * `name` - The name of the agent to retrieve
    ///
    /// # Returns
    /// Returns the agent configuration if found, None otherwise
    pub fn get_agent(&self, name: &str) -> MarkdownConfigResult<Option<AgentConfig>> {
        let agents = self.agents.read().map_err(|e| {
            MarkdownConfigError::registration_error(format!("Failed to acquire read lock: {}", e))
        })?;

        Ok(agents.get(name).cloned())
    }

    /// Get all registered agents
    pub fn get_all_agents(&self) -> MarkdownConfigResult<Vec<AgentConfig>> {
        let agents = self.agents.read().map_err(|e| {
            MarkdownConfigError::registration_error(format!("Failed to acquire read lock: {}", e))
        })?;

        Ok(agents.values().cloned().collect())
    }

    /// Check if an agent is registered
    pub fn has_agent(&self, name: &str) -> MarkdownConfigResult<bool> {
        let agents = self.agents.read().map_err(|e| {
            MarkdownConfigError::registration_error(format!("Failed to acquire read lock: {}", e))
        })?;

        Ok(agents.contains_key(name))
    }

    /// Remove an agent configuration
    pub fn remove_agent(&self, name: &str) -> MarkdownConfigResult<Option<AgentConfig>> {
        let mut agents = self.agents.write().map_err(|e| {
            MarkdownConfigError::registration_error(format!("Failed to acquire write lock: {}", e))
        })?;

        Ok(agents.remove(name))
    }

    // ============ Mode Registration ============

    /// Register a mode configuration
    ///
    /// # Arguments
    /// * `config` - The mode configuration to register
    ///
    /// # Errors
    /// Returns an error if:
    /// - A mode with the same name already exists
    /// - The configuration is invalid
    pub fn register_mode(&self, config: ModeConfig) -> MarkdownConfigResult<()> {
        // Validate configuration
        config.validate().map_err(|e| {
            MarkdownConfigError::validation_error(format!("Invalid mode configuration: {}", e))
        })?;

        let mut modes = self.modes.write().map_err(|e| {
            MarkdownConfigError::registration_error(format!("Failed to acquire write lock: {}", e))
        })?;

        // Check for duplicate registration
        if modes.contains_key(&config.name) {
            return Err(MarkdownConfigError::registration_error(format!(
                "Mode '{}' is already registered",
                config.name
            )));
        }

        modes.insert(config.name.clone(), config);
        Ok(())
    }

    /// Get a mode configuration by name
    ///
    /// # Arguments
    /// * `name` - The name of the mode to retrieve
    ///
    /// # Returns
    /// Returns the mode configuration if found, None otherwise
    pub fn get_mode(&self, name: &str) -> MarkdownConfigResult<Option<ModeConfig>> {
        let modes = self.modes.read().map_err(|e| {
            MarkdownConfigError::registration_error(format!("Failed to acquire read lock: {}", e))
        })?;

        Ok(modes.get(name).cloned())
    }

    /// Get all registered modes
    pub fn get_all_modes(&self) -> MarkdownConfigResult<Vec<ModeConfig>> {
        let modes = self.modes.read().map_err(|e| {
            MarkdownConfigError::registration_error(format!("Failed to acquire read lock: {}", e))
        })?;

        Ok(modes.values().cloned().collect())
    }

    /// Check if a mode is registered
    pub fn has_mode(&self, name: &str) -> MarkdownConfigResult<bool> {
        let modes = self.modes.read().map_err(|e| {
            MarkdownConfigError::registration_error(format!("Failed to acquire read lock: {}", e))
        })?;

        Ok(modes.contains_key(name))
    }

    /// Remove a mode configuration
    pub fn remove_mode(&self, name: &str) -> MarkdownConfigResult<Option<ModeConfig>> {
        let mut modes = self.modes.write().map_err(|e| {
            MarkdownConfigError::registration_error(format!("Failed to acquire write lock: {}", e))
        })?;

        Ok(modes.remove(name))
    }

    // ============ Command Registration ============

    /// Register a command configuration
    ///
    /// # Arguments
    /// * `config` - The command configuration to register
    ///
    /// # Errors
    /// Returns an error if:
    /// - A command with the same name already exists
    /// - The configuration is invalid
    pub fn register_command(&self, config: CommandConfig) -> MarkdownConfigResult<()> {
        // Validate configuration
        config.validate().map_err(|e| {
            MarkdownConfigError::validation_error(format!("Invalid command configuration: {}", e))
        })?;

        let mut commands = self.commands.write().map_err(|e| {
            MarkdownConfigError::registration_error(format!("Failed to acquire write lock: {}", e))
        })?;

        // Check for duplicate registration
        if commands.contains_key(&config.name) {
            return Err(MarkdownConfigError::registration_error(format!(
                "Command '{}' is already registered",
                config.name
            )));
        }

        commands.insert(config.name.clone(), config);
        Ok(())
    }

    /// Get a command configuration by name
    ///
    /// # Arguments
    /// * `name` - The name of the command to retrieve
    ///
    /// # Returns
    /// Returns the command configuration if found, None otherwise
    pub fn get_command(&self, name: &str) -> MarkdownConfigResult<Option<CommandConfig>> {
        let commands = self.commands.read().map_err(|e| {
            MarkdownConfigError::registration_error(format!("Failed to acquire read lock: {}", e))
        })?;

        Ok(commands.get(name).cloned())
    }

    /// Get all registered commands
    pub fn get_all_commands(&self) -> MarkdownConfigResult<Vec<CommandConfig>> {
        let commands = self.commands.read().map_err(|e| {
            MarkdownConfigError::registration_error(format!("Failed to acquire read lock: {}", e))
        })?;

        Ok(commands.values().cloned().collect())
    }

    /// Check if a command is registered
    pub fn has_command(&self, name: &str) -> MarkdownConfigResult<bool> {
        let commands = self.commands.read().map_err(|e| {
            MarkdownConfigError::registration_error(format!("Failed to acquire read lock: {}", e))
        })?;

        Ok(commands.contains_key(name))
    }

    /// Remove a command configuration
    pub fn remove_command(&self, name: &str) -> MarkdownConfigResult<Option<CommandConfig>> {
        let mut commands = self.commands.write().map_err(|e| {
            MarkdownConfigError::registration_error(format!("Failed to acquire write lock: {}", e))
        })?;

        Ok(commands.remove(name))
    }

    // ============ Registry Management ============

    /// Clear all registered configurations
    pub fn clear(&self) -> MarkdownConfigResult<()> {
        let mut agents = self.agents.write().map_err(|e| {
            MarkdownConfigError::registration_error(format!("Failed to acquire write lock: {}", e))
        })?;
        let mut modes = self.modes.write().map_err(|e| {
            MarkdownConfigError::registration_error(format!("Failed to acquire write lock: {}", e))
        })?;
        let mut commands = self.commands.write().map_err(|e| {
            MarkdownConfigError::registration_error(format!("Failed to acquire write lock: {}", e))
        })?;

        agents.clear();
        modes.clear();
        commands.clear();

        Ok(())
    }

    /// Get the total number of registered configurations
    pub fn count(&self) -> MarkdownConfigResult<(usize, usize, usize)> {
        let agents = self.agents.read().map_err(|e| {
            MarkdownConfigError::registration_error(format!("Failed to acquire read lock: {}", e))
        })?;
        let modes = self.modes.read().map_err(|e| {
            MarkdownConfigError::registration_error(format!("Failed to acquire read lock: {}", e))
        })?;
        let commands = self.commands.read().map_err(|e| {
            MarkdownConfigError::registration_error(format!("Failed to acquire read lock: {}", e))
        })?;

        Ok((agents.len(), modes.len(), commands.len()))
    }
}

impl Default for ConfigRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_agent(name: &str) -> AgentConfig {
        AgentConfig {
            name: name.to_string(),
            description: Some("Test agent".to_string()),
            prompt: "You are a helpful assistant".to_string(),
            model: Some("gpt-4".to_string()),
            temperature: Some(0.7),
            max_tokens: Some(2000),
            tools: vec![],
        }
    }

    fn create_test_mode(name: &str) -> ModeConfig {
        ModeConfig {
            name: name.to_string(),
            description: Some("Test mode".to_string()),
            prompt: "Focus on the task".to_string(),
            keybinding: Some("C-f".to_string()),
            enabled: true,
        }
    }

    fn create_test_command(name: &str) -> CommandConfig {
        CommandConfig {
            name: name.to_string(),
            description: Some("Test command".to_string()),
            template: "echo {{message}}".to_string(),
            parameters: vec![],
            keybinding: Some("C-t".to_string()),
        }
    }

    #[test]
    fn test_registry_creation() {
        let registry = ConfigRegistry::new();
        let (agents, modes, commands) = registry.count().unwrap();
        assert_eq!(agents, 0);
        assert_eq!(modes, 0);
        assert_eq!(commands, 0);
    }

    #[test]
    fn test_register_agent() {
        let registry = ConfigRegistry::new();
        let agent = create_test_agent("test-agent");

        let result = registry.register_agent(agent.clone());
        assert!(result.is_ok());

        let retrieved = registry.get_agent("test-agent").unwrap();
        assert_eq!(retrieved, Some(agent));
    }

    #[test]
    fn test_register_duplicate_agent() {
        let registry = ConfigRegistry::new();
        let agent = create_test_agent("test-agent");

        registry.register_agent(agent.clone()).unwrap();
        let result = registry.register_agent(agent);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_nonexistent_agent() {
        let registry = ConfigRegistry::new();
        let result = registry.get_agent("nonexistent").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_has_agent() {
        let registry = ConfigRegistry::new();
        let agent = create_test_agent("test-agent");

        registry.register_agent(agent).unwrap();
        assert!(registry.has_agent("test-agent").unwrap());
        assert!(!registry.has_agent("nonexistent").unwrap());
    }

    #[test]
    fn test_remove_agent() {
        let registry = ConfigRegistry::new();
        let agent = create_test_agent("test-agent");

        registry.register_agent(agent.clone()).unwrap();
        let removed = registry.remove_agent("test-agent").unwrap();
        assert_eq!(removed, Some(agent));
        assert!(!registry.has_agent("test-agent").unwrap());
    }

    #[test]
    fn test_get_all_agents() {
        let registry = ConfigRegistry::new();
        let agent1 = create_test_agent("agent1");
        let agent2 = create_test_agent("agent2");

        registry.register_agent(agent1).unwrap();
        registry.register_agent(agent2).unwrap();

        let all = registry.get_all_agents().unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_register_mode() {
        let registry = ConfigRegistry::new();
        let mode = create_test_mode("test-mode");

        let result = registry.register_mode(mode.clone());
        assert!(result.is_ok());

        let retrieved = registry.get_mode("test-mode").unwrap();
        assert_eq!(retrieved, Some(mode));
    }

    #[test]
    fn test_register_duplicate_mode() {
        let registry = ConfigRegistry::new();
        let mode = create_test_mode("test-mode");

        registry.register_mode(mode.clone()).unwrap();
        let result = registry.register_mode(mode);
        assert!(result.is_err());
    }

    #[test]
    fn test_register_command() {
        let registry = ConfigRegistry::new();
        let command = create_test_command("test-command");

        let result = registry.register_command(command.clone());
        assert!(result.is_ok());

        let retrieved = registry.get_command("test-command").unwrap();
        assert_eq!(retrieved, Some(command));
    }

    #[test]
    fn test_register_duplicate_command() {
        let registry = ConfigRegistry::new();
        let command = create_test_command("test-command");

        registry.register_command(command.clone()).unwrap();
        let result = registry.register_command(command);
        assert!(result.is_err());
    }

    #[test]
    fn test_clear_registry() {
        let registry = ConfigRegistry::new();
        registry.register_agent(create_test_agent("agent1")).unwrap();
        registry.register_mode(create_test_mode("mode1")).unwrap();
        registry.register_command(create_test_command("command1")).unwrap();

        registry.clear().unwrap();

        let (agents, modes, commands) = registry.count().unwrap();
        assert_eq!(agents, 0);
        assert_eq!(modes, 0);
        assert_eq!(commands, 0);
    }

    #[test]
    fn test_count() {
        let registry = ConfigRegistry::new();
        registry.register_agent(create_test_agent("agent1")).unwrap();
        registry.register_agent(create_test_agent("agent2")).unwrap();
        registry.register_mode(create_test_mode("mode1")).unwrap();
        registry.register_command(create_test_command("command1")).unwrap();

        let (agents, modes, commands) = registry.count().unwrap();
        assert_eq!(agents, 2);
        assert_eq!(modes, 1);
        assert_eq!(commands, 1);
    }
}
