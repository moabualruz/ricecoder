//! Model and agent cycling UI for RiceCoder TUI
//!
//! Handles cycling through:
//! - AI models (Tab to cycle)
//! - Agents (build, plan, etc.)
//! - Providers
//!
//! # DDD Layer: Application
//! Model and agent selection cycling.

use ratatui::style::Color;
use serde::{Deserialize, Serialize};

/// Cycle direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CycleDirection {
    Forward,
    Backward,
}

impl CycleDirection {
    pub fn as_i8(&self) -> i8 {
        match self {
            Self::Forward => 1,
            Self::Backward => -1,
        }
    }
}

/// An agent definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    /// Agent ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Description
    pub description: String,
    /// Agent color (hex or name)
    pub color: String,
    /// System prompt (optional)
    pub system_prompt: Option<String>,
    /// Whether agent is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool { true }

impl Agent {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            color: "cyan".to_string(),
            system_prompt: None,
            enabled: true,
        }
    }
    
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }
    
    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.color = color.into();
        self
    }
    
    /// Get ratatui color
    pub fn ratatui_color(&self) -> Color {
        parse_color(&self.color)
    }
}

/// Parse color string to ratatui Color
pub fn parse_color(s: &str) -> Color {
    match s.to_lowercase().as_str() {
        "red" => Color::Red,
        "green" => Color::Green,
        "blue" => Color::Blue,
        "yellow" => Color::Yellow,
        "magenta" | "purple" => Color::Magenta,
        "cyan" => Color::Cyan,
        "white" => Color::White,
        "gray" | "grey" => Color::Gray,
        "black" => Color::Black,
        s if s.starts_with('#') => {
            // Parse hex color
            let hex = s.trim_start_matches('#');
            if hex.len() == 6 {
                if let (Ok(r), Ok(g), Ok(b)) = (
                    u8::from_str_radix(&hex[0..2], 16),
                    u8::from_str_radix(&hex[2..4], 16),
                    u8::from_str_radix(&hex[4..6], 16),
                ) {
                    return Color::Rgb(r, g, b);
                }
            }
            Color::Cyan
        }
        _ => Color::Cyan,
    }
}

/// Agent manager
#[derive(Debug, Default)]
pub struct AgentManager {
    agents: Vec<Agent>,
    current: usize,
}

impl AgentManager {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Create with default agents
    pub fn with_defaults() -> Self {
        let mut manager = Self::new();
        manager.agents = default_agents();
        manager
    }
    
    /// Add an agent
    pub fn add(&mut self, agent: Agent) {
        self.agents.push(agent);
    }
    
    /// Get current agent
    pub fn current(&self) -> Option<&Agent> {
        self.agents.get(self.current)
    }
    
    /// Get agent by ID
    pub fn get(&self, id: &str) -> Option<&Agent> {
        self.agents.iter().find(|a| a.id == id)
    }
    
    /// Get all agents
    pub fn all(&self) -> &[Agent] {
        &self.agents
    }
    
    /// Get enabled agents
    pub fn enabled(&self) -> impl Iterator<Item = &Agent> {
        self.agents.iter().filter(|a| a.enabled)
    }
    
    /// Set current agent by ID
    pub fn set_current(&mut self, id: &str) -> bool {
        if let Some(idx) = self.agents.iter().position(|a| a.id == id) {
            self.current = idx;
            true
        } else {
            false
        }
    }
    
    /// Cycle to next/previous agent
    pub fn cycle(&mut self, direction: CycleDirection) -> Option<&Agent> {
        let enabled: Vec<usize> = self.agents
            .iter()
            .enumerate()
            .filter(|(_, a)| a.enabled)
            .map(|(i, _)| i)
            .collect();
        
        if enabled.is_empty() {
            return None;
        }
        
        let current_pos = enabled.iter().position(|&i| i == self.current).unwrap_or(0);
        let new_pos = match direction {
            CycleDirection::Forward => (current_pos + 1) % enabled.len(),
            CycleDirection::Backward => (current_pos + enabled.len() - 1) % enabled.len(),
        };
        
        self.current = enabled[new_pos];
        self.current()
    }
    
    /// Get color for agent name
    pub fn color(&self, name: &str) -> Color {
        self.agents
            .iter()
            .find(|a| a.name.eq_ignore_ascii_case(name) || a.id.eq_ignore_ascii_case(name))
            .map(|a| a.ratatui_color())
            .unwrap_or(Color::Cyan)
    }
}

/// Default agents
pub fn default_agents() -> Vec<Agent> {
    vec![
        Agent::new("build", "Build")
            .with_description("General coding assistant")
            .with_color("cyan"),
        Agent::new("plan", "Plan")
            .with_description("Planning and architecture")
            .with_color("magenta"),
        Agent::new("explore", "Explore")
            .with_description("Codebase exploration")
            .with_color("yellow"),
        Agent::new("document", "Document")
            .with_description("Documentation writer")
            .with_color("green"),
        Agent::new("oracle", "Oracle")
            .with_description("Expert technical advisor")
            .with_color("#FFD700"), // Gold
    ]
}

/// Cycling state for UI
#[derive(Debug, Default)]
pub struct CyclingState {
    /// Whether cycling UI is visible
    pub visible: bool,
    /// Current cycling mode
    pub mode: CyclingMode,
    /// Preview agent/model (before confirming)
    pub preview: Option<String>,
}

/// What we're cycling through
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CyclingMode {
    #[default]
    Agent,
    Model,
    Provider,
}

impl CyclingState {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Show cycling UI for agents
    pub fn show_agents(&mut self) {
        self.visible = true;
        self.mode = CyclingMode::Agent;
    }
    
    /// Show cycling UI for models
    pub fn show_models(&mut self) {
        self.visible = true;
        self.mode = CyclingMode::Model;
    }
    
    /// Hide cycling UI
    pub fn hide(&mut self) {
        self.visible = false;
        self.preview = None;
    }
    
    /// Set preview
    pub fn set_preview(&mut self, id: impl Into<String>) {
        self.preview = Some(id.into());
    }
    
    /// Confirm selection
    pub fn confirm(&mut self) -> Option<String> {
        let result = self.preview.take();
        self.hide();
        result
    }
}

/// Event emitted by cycling
#[derive(Debug, Clone)]
pub enum CyclingEvent {
    /// Agent was selected
    AgentSelected(String),
    /// Model was selected
    ModelSelected(String),
    /// Provider was selected
    ProviderSelected(String),
    /// Cycling was cancelled
    Cancelled,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_agent_new() {
        let agent = Agent::new("test", "Test Agent")
            .with_description("A test agent")
            .with_color("red");
        
        assert_eq!(agent.id, "test");
        assert_eq!(agent.name, "Test Agent");
        assert_eq!(agent.ratatui_color(), Color::Red);
    }
    
    #[test]
    fn test_parse_color() {
        assert_eq!(parse_color("red"), Color::Red);
        assert_eq!(parse_color("RED"), Color::Red);
        assert_eq!(parse_color("#FF0000"), Color::Rgb(255, 0, 0));
        assert_eq!(parse_color("#00ff00"), Color::Rgb(0, 255, 0));
    }
    
    #[test]
    fn test_agent_manager_cycle() {
        let mut manager = AgentManager::with_defaults();
        let initial = manager.current().unwrap().id.clone();
        
        manager.cycle(CycleDirection::Forward);
        let next = manager.current().unwrap().id.clone();
        assert_ne!(initial, next);
        
        manager.cycle(CycleDirection::Backward);
        let back = manager.current().unwrap().id.clone();
        assert_eq!(initial, back);
    }
    
    #[test]
    fn test_agent_manager_set_current() {
        let mut manager = AgentManager::with_defaults();
        
        assert!(manager.set_current("plan"));
        assert_eq!(manager.current().unwrap().id, "plan");
        
        assert!(!manager.set_current("nonexistent"));
    }
    
    #[test]
    fn test_cycling_state() {
        let mut state = CyclingState::new();
        assert!(!state.visible);
        
        state.show_agents();
        assert!(state.visible);
        assert_eq!(state.mode, CyclingMode::Agent);
        
        state.set_preview("build");
        let confirmed = state.confirm();
        assert_eq!(confirmed, Some("build".to_string()));
        assert!(!state.visible);
    }
    
    #[test]
    fn test_default_agents() {
        let agents = default_agents();
        assert!(agents.len() >= 4);
        assert!(agents.iter().any(|a| a.id == "build"));
        assert!(agents.iter().any(|a| a.id == "plan"));
    }
}
