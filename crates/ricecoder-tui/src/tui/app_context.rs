//! TUI Application Context - Central State Management
//!
//! This module provides the central context that wires the TUI to all backend crates:
//! - ricecoder-sessions: Session management, messages, history
//! - ricecoder-providers: AI provider integration, model selection
//! - ricecoder-mcp: MCP server status and management
//! - ricecoder-agents: Agent registry and selection
//! - ricecoder-config: Configuration and settings
//!
//! Inspired by OpenCode's context provider architecture.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use ricecoder_agents::{AgentConfig, AgentMetadata, AgentRegistry};
use ricecoder_config::{ConfigManager, TuiConfig};
use ricecoder_mcp::{HealthStatus, ServerManager, ServerState, MCPToolExecutor, ToolExecutionContext, ToolExecutor};
use ricecoder_providers::{
    AnthropicProvider, ChatRequest, ChatResponse, ModelInfo, OllamaProvider, OpenAiProvider,
    Provider, ProviderManager, ProviderRegistry, ProviderStatus, TokenUsage as ProviderTokenUsage,
    ZenProvider, Message as ProviderMessage, WordStream,
};
use ricecoder_sessions::{
    Message, MessageRole, Session, SessionContext, SessionManager, SessionMode, SessionStatus,
    TuiSessionManager,
};

/// MCP Server status for display
#[derive(Debug, Clone, Default)]
pub struct McpServerStatus {
    /// Server name
    pub name: String,
    /// Connection status: connected, disconnected, error, loading
    pub status: McpConnectionStatus,
    /// Error message if failed
    pub error: Option<String>,
    /// Tool count if connected
    pub tool_count: usize,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum McpConnectionStatus {
    #[default]
    Disconnected,
    Connected,
    Error,
    Loading,
}

/// Provider status for display
#[derive(Debug, Clone)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub connected: bool,
    pub models: Vec<ModelDisplayInfo>,
    pub default_model: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ModelDisplayInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

/// Agent display info
#[derive(Debug, Clone)]
pub struct AgentInfo {
    pub name: String,
    pub description: String,
    pub color: (u8, u8, u8), // RGB color
    pub is_default: bool,
}

/// Slash command info for display and autocomplete
#[derive(Debug, Clone)]
pub struct SlashCommandInfo {
    /// Command name (without leading /)
    pub name: String,
    /// Description
    pub description: String,
    /// Optional argument hint (e.g., "<filename>")
    pub argument_hint: Option<String>,
    /// Whether this command runs as a subtask
    pub is_subtask: bool,
}

/// Sync status (mirrors OpenCode pattern)
#[derive(Debug, Clone, PartialEq)]
pub enum SyncStatus {
    Loading,
    Partial, // Core data loaded
    Complete,
}

/// Central application state
/// Mirrors OpenCode's sync store pattern
#[derive(Debug, Clone)]
pub struct AppState {
    /// Sync status
    pub sync_status: SyncStatus,

    // === Session Data ===
    /// All sessions
    pub sessions: Vec<SessionSummary>,
    /// Current session (if in session route)
    pub current_session: Option<Session>,
    /// Messages for current session
    pub messages: Vec<Message>,
    /// Session status (idle, running, etc.)
    pub session_status: SessionStatus,

    // === Provider Data ===
    /// Available providers
    pub providers: Vec<ProviderInfo>,
    /// Current provider ID
    pub current_provider_id: Option<String>,
    /// Current model ID
    pub current_model_id: Option<String>,
    /// Recent models (for cycling)
    pub recent_models: Vec<(String, String)>, // (provider_id, model_id)
    /// Favorite models
    pub favorite_models: Vec<(String, String)>,

    // === Agent Data ===
    /// Available agents
    pub agents: Vec<AgentInfo>,
    /// Current agent name
    pub current_agent: String,

    // === MCP Data ===
    /// MCP server statuses
    pub mcp_servers: HashMap<String, McpServerStatus>,

    // === Config ===
    /// Current working directory
    pub directory: String,
    /// Version string
    pub version: String,
    /// VCS branch (if in git repo)
    pub vcs_branch: Option<String>,

    // === Token Tracking ===
    /// Total tokens used in current session
    pub session_tokens: SessionTokenUsage,

    // === UI State ===
    /// Command palette visible
    pub command_palette_visible: bool,
    /// Tips hidden
    pub tips_hidden: bool,
    /// Terminal title enabled
    pub terminal_title_enabled: bool,

    // === Commands ===
    /// Available slash commands (loaded from config/commands/*.md)
    pub slash_commands: Vec<SlashCommandInfo>,
}

/// Token usage tracking for the current session
#[derive(Debug, Clone, Default)]
pub struct SessionTokenUsage {
    /// Total prompt tokens used
    pub prompt_tokens: usize,
    /// Total completion tokens used
    pub completion_tokens: usize,
    /// Total tokens used
    pub total_tokens: usize,
    /// Estimated cost in USD
    pub estimated_cost: f64,
    /// Context limit for current model
    pub context_limit: usize,
}

impl SessionTokenUsage {
    /// Get usage percentage of context limit
    pub fn usage_percentage(&self) -> f64 {
        if self.context_limit == 0 {
            0.0
        } else {
            (self.total_tokens as f64 / self.context_limit as f64) * 100.0
        }
    }

    /// Format tokens for display (e.g., "12.5K")
    pub fn format_tokens(&self) -> String {
        if self.total_tokens >= 1_000_000 {
            format!("{:.1}M", self.total_tokens as f64 / 1_000_000.0)
        } else if self.total_tokens >= 1_000 {
            format!("{:.1}K", self.total_tokens as f64 / 1_000.0)
        } else {
            format!("{}", self.total_tokens)
        }
    }

    /// Format cost for display
    pub fn format_cost(&self) -> String {
        if self.estimated_cost >= 1.0 {
            format!("${:.2}", self.estimated_cost)
        } else if self.estimated_cost >= 0.01 {
            format!("${:.3}", self.estimated_cost)
        } else if self.estimated_cost > 0.0 {
            format!("${:.4}", self.estimated_cost)
        } else {
            "$0.00".to_string()
        }
    }
}

/// Session summary for list display
#[derive(Debug, Clone)]
pub struct SessionSummary {
    pub id: String,
    pub title: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub message_count: usize,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            sync_status: SyncStatus::Loading,
            sessions: Vec::new(),
            current_session: None,
            messages: Vec::new(),
            session_status: SessionStatus::Active,
            providers: Vec::new(),
            current_provider_id: None,
            current_model_id: None,
            recent_models: Vec::new(),
            favorite_models: Vec::new(),
            agents: vec![
                AgentInfo {
                    name: "build".to_string(),
                    description: "Full access agent for development work".to_string(),
                    color: (0, 255, 255), // Cyan
                    is_default: true,
                },
                AgentInfo {
                    name: "plan".to_string(),
                    description: "Read-only agent for analysis and exploration".to_string(),
                    color: (255, 200, 100), // Orange
                    is_default: false,
                },
            ],
            current_agent: "build".to_string(),
            mcp_servers: HashMap::new(),
            session_tokens: SessionTokenUsage::default(),
            directory: std::env::current_dir()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| ".".to_string()),
            version: env!("CARGO_PKG_VERSION").to_string(),
            vcs_branch: None,
            command_palette_visible: false,
            tips_hidden: false,
            terminal_title_enabled: true,
            slash_commands: Vec::new(),
        }
    }
}

impl AppState {
    /// Get connected MCP server count
    pub fn connected_mcp_count(&self) -> usize {
        self.mcp_servers
            .values()
            .filter(|s| s.status == McpConnectionStatus::Connected)
            .count()
    }

    /// Check if any MCP servers have errors
    pub fn has_mcp_errors(&self) -> bool {
        self.mcp_servers
            .values()
            .any(|s| s.status == McpConnectionStatus::Error)
    }

    /// Get current provider display name
    pub fn current_provider_name(&self) -> String {
        self.current_provider_id
            .as_ref()
            .and_then(|id| self.providers.iter().find(|p| &p.id == id))
            .map(|p| p.name.clone())
            .unwrap_or_else(|| "No provider".to_string())
    }

    /// Get current model display name
    pub fn current_model_name(&self) -> String {
        let provider_id = match &self.current_provider_id {
            Some(id) => id,
            None => return "(no model)".to_string(),
        };
        let model_id = match &self.current_model_id {
            Some(id) => id,
            None => return "(no model)".to_string(),
        };

        self.providers
            .iter()
            .find(|p| &p.id == provider_id)
            .and_then(|p| p.models.iter().find(|m| &m.id == model_id))
            .map(|m| m.name.clone())
            .unwrap_or_else(|| model_id.clone())
    }

    /// Get current agent color
    pub fn current_agent_color(&self) -> (u8, u8, u8) {
        self.agents
            .iter()
            .find(|a| a.name == self.current_agent)
            .map(|a| a.color)
            .unwrap_or((0, 255, 255))
    }

    /// Cycle to next agent
    pub fn cycle_agent(&mut self) {
        let current_idx = self
            .agents
            .iter()
            .position(|a| a.name == self.current_agent)
            .unwrap_or(0);
        let next_idx = (current_idx + 1) % self.agents.len();
        self.current_agent = self.agents[next_idx].name.clone();
    }

    /// Check if this is a first-time user (no sessions)
    pub fn is_first_time_user(&self) -> bool {
        self.sessions.is_empty()
    }

    /// Get formatted token usage for display (e.g., "12.5K tokens | $0.03")
    pub fn token_display(&self) -> String {
        if self.session_tokens.total_tokens == 0 {
            String::new()
        } else {
            format!(
                "{} tokens | {}",
                self.session_tokens.format_tokens(),
                self.session_tokens.format_cost()
            )
        }
    }

    /// Reset token tracking (for new session)
    pub fn reset_token_tracking(&mut self) {
        self.session_tokens = SessionTokenUsage::default();
    }

    /// Get slash command by name
    pub fn get_command(&self, name: &str) -> Option<&SlashCommandInfo> {
        self.slash_commands.iter().find(|c| c.name == name)
    }

    /// Get slash commands matching a prefix (for autocomplete)
    pub fn filter_commands(&self, prefix: &str) -> Vec<&SlashCommandInfo> {
        let prefix_lower = prefix.to_lowercase();
        self.slash_commands
            .iter()
            .filter(|c| c.name.to_lowercase().starts_with(&prefix_lower))
            .collect()
    }
}

/// Application context that manages all backend integrations
pub struct AppContext {
    /// Shared application state
    pub state: Arc<RwLock<AppState>>,

    /// Session manager (backend)
    session_manager: Arc<RwLock<SessionManager>>,

    /// Provider manager
    provider_manager: Arc<RwLock<Option<ProviderManager>>>,

    /// MCP server manager
    mcp_manager: Arc<RwLock<Option<ServerManager>>>,

    /// Agent registry
    agent_registry: Arc<RwLock<Option<AgentRegistry>>>,

    /// Config manager
    config_manager: Arc<RwLock<Option<ConfigManager>>>,

    /// Keybind engine
    pub keybind_engine: Arc<RwLock<ricecoder_keybinds::KeybindEngine>>,
}

/// Convert MCP ServerState to TUI McpConnectionStatus
fn convert_server_state(state: &ricecoder_mcp::ServerState) -> McpConnectionStatus {
    use ricecoder_mcp::ServerState;
    match state {
        ServerState::Connected => McpConnectionStatus::Connected,
        ServerState::Connecting | ServerState::Starting => McpConnectionStatus::Loading,
        ServerState::Error => McpConnectionStatus::Error,
        ServerState::Disconnected | ServerState::Disabled | ServerState::Stopped 
            => McpConnectionStatus::Disconnected,
    }
}

/// Calculate token cost based on model pricing
/// Prices are per 1M tokens (input/output)
fn calculate_token_cost(model_id: &str, prompt_tokens: usize, completion_tokens: usize) -> f64 {
    // Model pricing per 1M tokens (USD): (input_price, output_price)
    let (input_price, output_price) = match model_id {
        // Anthropic Claude models
        m if m.contains("claude-3-5-sonnet") || m.contains("claude-sonnet-4") => (3.0, 15.0),
        m if m.contains("claude-3-5-haiku") => (0.80, 4.0),
        m if m.contains("claude-3-opus") => (15.0, 75.0),
        m if m.contains("claude-3-sonnet") => (3.0, 15.0),
        m if m.contains("claude-3-haiku") => (0.25, 1.25),
        
        // OpenAI GPT models
        m if m.contains("gpt-4o") => (2.50, 10.0),
        m if m.contains("gpt-4o-mini") => (0.15, 0.60),
        m if m.contains("gpt-4-turbo") => (10.0, 30.0),
        m if m.contains("gpt-4") => (30.0, 60.0),
        m if m.contains("gpt-3.5-turbo") => (0.50, 1.50),
        m if m.contains("o1-preview") => (15.0, 60.0),
        m if m.contains("o1-mini") => (3.0, 12.0),
        m if m.contains("o3-mini") => (1.10, 4.40),
        
        // Ollama/local models (free)
        m if m.contains("llama") || m.contains("mistral") || m.contains("phi") 
            || m.contains("gemma") || m.contains("qwen") => (0.0, 0.0),
        
        // Default pricing for unknown models
        _ => (1.0, 3.0),
    };
    
    // Calculate cost: tokens / 1_000_000 * price_per_million
    let input_cost = (prompt_tokens as f64 / 1_000_000.0) * input_price;
    let output_cost = (completion_tokens as f64 / 1_000_000.0) * output_price;
    
    input_cost + output_cost
}

impl AppContext {
    /// Create a new application context
    pub fn new() -> Self {
        // Initialize keybind engine with defaults
        let mut keybind_engine = ricecoder_keybinds::KeybindEngine::new();
        // Load defaults - log warning if fails but continue
        if let Err(e) = keybind_engine.apply_defaults() {
            tracing::warn!("Failed to apply default keybinds: {}", e);
        }
        let keybind_engine = Arc::new(RwLock::new(keybind_engine));

        // Initialize MCP ServerManager
        let mcp_manager = ServerManager::new();
        tracing::debug!("Initialized MCP ServerManager");

        Self {
            state: Arc::new(RwLock::new(AppState::default())),
            session_manager: Arc::new(RwLock::new(SessionManager::new(100))),
            provider_manager: Arc::new(RwLock::new(None)),
            mcp_manager: Arc::new(RwLock::new(Some(mcp_manager))),
            agent_registry: Arc::new(RwLock::new(None)),
            config_manager: Arc::new(RwLock::new(None)),
            keybind_engine,
        }
    }

    /// Initialize the context - load all data from backend crates
    /// This is the "bootstrap" phase (like OpenCode's sync.tsx)
    pub async fn initialize(&self) -> anyhow::Result<()> {
        // Phase 1: Load configuration (blocking)
        self.load_config().await?;

        // Phase 2: Initialize providers (blocking)
        self.load_providers().await?;

        // Phase 3: Load agents (blocking)
        self.load_agents().await?;

        // Phase 3.5: Load slash commands (blocking)
        self.load_commands().await?;

        // Mark as partial ready
        {
            let mut state = self.state.write().await;
            state.sync_status = SyncStatus::Partial;
        }

        // Phase 4: Load sessions (non-blocking can be made blocking if --continue)
        self.load_sessions().await?;

        // Phase 5: Load MCP status (non-blocking)
        self.load_mcp_status().await?;

        // Phase 6: Detect VCS
        self.detect_vcs().await?;

        // Mark as complete
        {
            let mut state = self.state.write().await;
            state.sync_status = SyncStatus::Complete;
        }

        Ok(())
    }

    /// Load configuration
    async fn load_config(&self) -> anyhow::Result<()> {
        // Try to create config manager
        let config_manager = ConfigManager::new();

        // Store reference
        {
            let mut mgr = self.config_manager.write().await;
            *mgr = Some(config_manager);
        }

        // Load TuiConfig and apply settings
        match TuiConfig::load() {
            Ok(tui_config) => {
                let mut state = self.state.write().await;
                
                // Apply provider/model defaults from config if set
                if let Some(provider) = &tui_config.provider {
                    state.current_provider_id = Some(provider.clone());
                }
                if let Some(model) = &tui_config.model {
                    state.current_model_id = Some(model.clone());
                }
                
                // Apply UI preferences
                state.tips_hidden = tui_config.accessibility.animations_disabled;
                
                tracing::debug!("Loaded TuiConfig: theme={}", tui_config.theme);
            }
            Err(e) => {
                tracing::debug!("Using default config: {}", e);
            }
        }

        Ok(())
    }

    /// Load providers and models
    async fn load_providers(&self) -> anyhow::Result<()> {
        // Create provider registry
        let mut registry = ProviderRegistry::new();

        // Auto-detect and register providers based on environment variables
        let mut provider_infos = Vec::new();

        // Check for Anthropic API key
        if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
            if !api_key.is_empty() {
                if let Ok(provider) = AnthropicProvider::new(api_key) {
                    let models: Vec<ModelDisplayInfo> = provider
                        .models()
                        .into_iter()
                        .map(|m| ModelDisplayInfo {
                            id: m.id.clone(),
                            name: m.name.clone(),
                            description: None, // ModelInfo doesn't have description
                        })
                        .collect();
                    let default_model = models.first().map(|m| m.id.clone());
                    provider_infos.push(ProviderInfo {
                        id: "anthropic".to_string(),
                        name: "Anthropic".to_string(),
                        connected: true,
                        models,
                        default_model,
                    });
                    let _ = registry.register(std::sync::Arc::new(provider));
                }
            }
        }

        // Check for OpenAI API key
        if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            if !api_key.is_empty() {
                if let Ok(provider) = OpenAiProvider::new(api_key) {
                    let models: Vec<ModelDisplayInfo> = provider
                        .models()
                        .into_iter()
                        .map(|m| ModelDisplayInfo {
                            id: m.id.clone(),
                            name: m.name.clone(),
                            description: None, // ModelInfo doesn't have description
                        })
                        .collect();
                    let default_model = models.first().map(|m| m.id.clone());
                    provider_infos.push(ProviderInfo {
                        id: "openai".to_string(),
                        name: "OpenAI".to_string(),
                        connected: true,
                        models,
                        default_model,
                    });
                    let _ = registry.register(std::sync::Arc::new(provider));
                }
            }
        }

        // Check for Zen API key (Zen AI provider)
        if let Ok(api_key) = std::env::var("ZEN_API_KEY") {
            if !api_key.is_empty() {
                if let Ok(provider) = ZenProvider::new(Some(api_key)) {
                    let models: Vec<ModelDisplayInfo> = provider
                        .models()
                        .into_iter()
                        .map(|m| ModelDisplayInfo {
                            id: m.id.clone(),
                            name: m.name.clone(),
                            description: None, // ModelInfo doesn't have description
                        })
                        .collect();
                    let default_model = models.first().map(|m| m.id.clone());
                    provider_infos.push(ProviderInfo {
                        id: "zen".to_string(),
                        name: "Zen".to_string(),
                        connected: true,
                        models,
                        default_model,
                    });
                    let _ = registry.register(std::sync::Arc::new(provider));
                }
            }
        }

        // Check for Ollama (local, no API key needed)
        // Try to connect to default Ollama endpoint
        let ollama_url = std::env::var("OLLAMA_HOST")
            .unwrap_or_else(|_| "http://localhost:11434".to_string());
        if let Ok(provider) = OllamaProvider::new(ollama_url) {
            // Only add if Ollama is reachable (check health)
            if provider.health_check().await.unwrap_or(false) {
                let models: Vec<ModelDisplayInfo> = provider
                    .models()
                    .into_iter()
                    .map(|m| ModelDisplayInfo {
                        id: m.id.clone(),
                        name: m.name.clone(),
                        description: None, // ModelInfo doesn't have description
                    })
                    .collect();
                let default_model = models.first().map(|m| m.id.clone());
                provider_infos.push(ProviderInfo {
                    id: "ollama".to_string(),
                    name: "Ollama".to_string(),
                    connected: true,
                    models,
                    default_model,
                });
                let _ = registry.register(std::sync::Arc::new(provider));
            }
        }

        // If no providers found, add placeholders for common providers
        if provider_infos.is_empty() {
            provider_infos = vec![
                ProviderInfo {
                    id: "anthropic".to_string(),
                    name: "Anthropic".to_string(),
                    connected: false,
                    models: vec![
                        ModelDisplayInfo {
                            id: "claude-sonnet-4-20250514".to_string(),
                            name: "Claude Sonnet 4".to_string(),
                            description: Some("Set ANTHROPIC_API_KEY to enable".to_string()),
                        },
                    ],
                    default_model: Some("claude-sonnet-4-20250514".to_string()),
                },
                ProviderInfo {
                    id: "openai".to_string(),
                    name: "OpenAI".to_string(),
                    connected: false,
                    models: vec![
                        ModelDisplayInfo {
                            id: "gpt-4o".to_string(),
                            name: "GPT-4o".to_string(),
                            description: Some("Set OPENAI_API_KEY to enable".to_string()),
                        },
                    ],
                    default_model: Some("gpt-4o".to_string()),
                },
                ProviderInfo {
                    id: "ollama".to_string(),
                    name: "Ollama".to_string(),
                    connected: false,
                    models: vec![
                        ModelDisplayInfo {
                            id: "llama3.1".to_string(),
                            name: "Llama 3.1".to_string(),
                            description: Some("Start Ollama to enable".to_string()),
                        },
                    ],
                    default_model: Some("llama3.1".to_string()),
                },
            ];
        }

        // Store provider manager
        let default_provider_id = provider_infos
            .iter()
            .find(|p| p.connected)
            .or(provider_infos.first())
            .map(|p| p.id.clone())
            .unwrap_or_else(|| "anthropic".to_string());

        let provider_manager = ProviderManager::new(registry, default_provider_id.clone());
        {
            let mut mgr = self.provider_manager.write().await;
            *mgr = Some(provider_manager);
        }

        // Update state
        let mut state = self.state.write().await;
        state.providers = provider_infos;

        // Set default provider/model if available
        let first_connected = state.providers.iter().find(|p| p.connected).cloned();
        let first_any = state.providers.first().cloned();
        if let Some(provider) = first_connected.or(first_any) {
            state.current_provider_id = Some(provider.id);
            state.current_model_id = provider.default_model;
        }

        Ok(())
    }

    /// Load agents from config files and registry
    async fn load_agents(&self) -> anyhow::Result<()> {
        // Create agent registry
        let registry = AgentRegistry::new();
        
        // Store registry
        {
            let mut reg = self.agent_registry.write().await;
            *reg = Some(registry);
        }
        
        // First, try to load agents from config/agents/*.md files
        let mut agent_infos: Vec<AgentInfo> = Vec::new();
        
        let loader = ricecoder_storage::AgentLoader::with_default_path();
        if let Ok(config_agents) = loader.load_all() {
            for (idx, (name, agent)) in config_agents.into_iter().enumerate() {
                let color = match idx % 6 {
                    0 => (0, 255, 255),   // Cyan
                    1 => (255, 200, 100), // Orange
                    2 => (100, 255, 100), // Green
                    3 => (255, 100, 255), // Magenta
                    4 => (100, 100, 255), // Blue
                    _ => (255, 255, 100), // Yellow
                };
                agent_infos.push(AgentInfo {
                    name,
                    description: agent.description,
                    color,
                    is_default: idx == 0,
                });
            }
            tracing::debug!("Loaded {} agents from config files", agent_infos.len());
        }
        
        // If no file-based agents, try from registry
        if agent_infos.is_empty() {
            let reg = self.agent_registry.read().await;
            if let Some(registry) = reg.as_ref() {
                agent_infos = registry.all_agent_metadata()
                    .into_iter()
                    .enumerate()
                    .map(|(idx, metadata)| {
                        let color = match idx % 6 {
                            0 => (0, 255, 255),   // Cyan
                            1 => (255, 200, 100), // Orange
                            2 => (100, 255, 100), // Green
                            3 => (255, 100, 255), // Magenta
                            4 => (100, 100, 255), // Blue
                            _ => (255, 255, 100), // Yellow
                        };
                        AgentInfo {
                            name: metadata.name.clone(),
                            description: metadata.description.clone(),
                            color,
                            is_default: idx == 0,
                        }
                    })
                    .collect();
            }
        }
        
        // Update state with loaded agents
        let mut state = self.state.write().await;
        if !agent_infos.is_empty() {
            state.agents = agent_infos;
            if let Some(first) = state.agents.first() {
                state.current_agent = first.name.clone();
            }
        }
        // If still empty, keep default agents from AppState::default()
        
        Ok(())
    }

    /// Load slash commands from config/commands/*.md files
    async fn load_commands(&self) -> anyhow::Result<()> {
        let loader = ricecoder_storage::CommandLoader::with_default_path();
        
        match loader.load_all() {
            Ok(commands) => {
                let mut command_infos: Vec<SlashCommandInfo> = commands
                    .into_iter()
                    .map(|(name, cmd)| SlashCommandInfo {
                        name,
                        description: cmd.description,
                        argument_hint: cmd.argument_hint,
                        is_subtask: cmd.subtask,
                    })
                    .collect();
                
                // Sort by name for consistent ordering
                command_infos.sort_by(|a, b| a.name.cmp(&b.name));
                
                let mut state = self.state.write().await;
                state.slash_commands = command_infos;
                
                tracing::debug!("Loaded {} slash commands from config files", state.slash_commands.len());
            }
            Err(e) => {
                tracing::debug!("Could not load slash commands: {}", e);
            }
        }
        
        Ok(())
    }

    /// Load sessions
    async fn load_sessions(&self) -> anyhow::Result<()> {
        let session_mgr = self.session_manager.read().await;
        let backend_sessions = session_mgr.list_sessions();
        drop(session_mgr); // Release read lock before acquiring write lock
        
        // Convert backend sessions to TUI SessionSummary format
        let mut state = self.state.write().await;
        state.sessions = backend_sessions
            .into_iter()
            .map(|session| SessionSummary {
                id: session.id.clone(),
                title: session.name.clone(),
                created_at: session.created_at,
                updated_at: session.updated_at,
                message_count: session.history.len(),
            })
            .collect();
        
        Ok(())
    }

    /// Load MCP server status
    async fn load_mcp_status(&self) -> anyhow::Result<()> {
        let mcp_mgr = self.mcp_manager.read().await;
        
        if let Some(manager) = mcp_mgr.as_ref() {
            // Try to list servers
            match manager.list_servers().await {
                Ok(servers) => {
                    let mut mcp_servers = HashMap::new();
                    for registration in servers {
                        let status = McpServerStatus {
                            name: registration.config.name.clone(),
                            status: convert_server_state(&registration.health.state),
                            error: registration.health.last_error.clone(),
                            tool_count: registration.health.tools_available,
                        };
                        mcp_servers.insert(registration.config.id.clone(), status);
                    }
                    
                    let mut state = self.state.write().await;
                    state.mcp_servers = mcp_servers;
                }
                Err(e) => {
                    tracing::debug!("Could not load MCP servers: {}", e);
                }
            }
        }
        
        Ok(())
    }

    /// Detect VCS (git) information
    async fn detect_vcs(&self) -> anyhow::Result<()> {
        // Try to detect git branch
        let output = tokio::process::Command::new("git")
            .args(["branch", "--show-current"])
            .output()
            .await;

        if let Ok(output) = output {
            if output.status.success() {
                let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !branch.is_empty() {
                    let mut state = self.state.write().await;
                    state.vcs_branch = Some(branch);
                }
            }
        }

        Ok(())
    }

    // === Session Operations ===

    /// Create a new session
    pub async fn create_session(&self) -> anyhow::Result<String> {
        let context = SessionContext::new(
            {
                let state = self.state.read().await;
                state.current_provider_id.clone().unwrap_or_else(|| "anthropic".to_string())
            },
            {
                let state = self.state.read().await;
                state.current_model_id.clone().unwrap_or_else(|| "claude-sonnet-4-20250514".to_string())
            },
            SessionMode::Chat,
        );
        
        let mut session_mgr = self.session_manager.write().await;
        let session = session_mgr.create_session("New Session".to_string(), context)?;
        let session_id = session.id.clone();
        
        // Set as current session
        {
            let mut state = self.state.write().await;
            state.current_session = Some(session);
            state.messages = Vec::new();
            state.session_status = SessionStatus::Active;
        }
        
        // Reload sessions list
        drop(session_mgr);
        self.load_sessions().await?;
        
        Ok(session_id)
    }

    /// Switch to a different session
    pub async fn switch_session(&self, session_id: &str) -> anyhow::Result<()> {
        // Get session from SessionManager
        let session = {
            let session_mgr = self.session_manager.read().await;
            session_mgr.get_session(session_id)?
        };
        
        // Switch in SessionManager
        {
            let mut session_mgr = self.session_manager.write().await;
            session_mgr.switch_session(session_id)?;
        }
        
        // Update local state
        {
            let mut state = self.state.write().await;
            // Load messages from session history
            state.messages = session.history.clone();
            state.current_session = Some(session);
            state.session_status = SessionStatus::Active;
        }
        
        Ok(())
    }

    /// Delete a session
    pub async fn delete_session(&self, session_id: &str) -> anyhow::Result<()> {
        // Check if this is the current session before deleting
        let is_current = {
            let state = self.state.read().await;
            state.current_session.as_ref().map(|s| s.id.as_str()) == Some(session_id)
        };
        
        // Delete from SessionManager
        {
            let mut session_mgr = self.session_manager.write().await;
            session_mgr.delete_session(session_id)?;
        }
        
        // If this was the current session, clear state
        if is_current {
            let mut state = self.state.write().await;
            state.current_session = None;
            state.messages.clear();
        }
        
        // Reload sessions list
        self.load_sessions().await?;
        
        Ok(())
    }

    /// Get list of all sessions for UI display
    pub async fn get_session_list(&self) -> Vec<SessionSummary> {
        let state = self.state.read().await;
        state.sessions.clone()
    }

    /// Add a user message to current session
    pub async fn add_user_message(&self, content: String) -> anyhow::Result<()> {
        // Add to local state
        let session_id = {
            let mut state = self.state.write().await;

            // Use Message::new helper which properly constructs MessagePart::Text
            let message = Message::new(MessageRole::User, content.clone());
            state.messages.push(message);

            // Mark as running (Active is the running state in SessionStatus)
            state.session_status = SessionStatus::Active;

            // Get session ID for persistence
            state.current_session.as_ref().map(|s| s.id.clone())
        };

        // Persist to SessionManager if we have a current session
        if let Some(sid) = session_id {
            let mut session_mgr = self.session_manager.write().await;
            if let Ok(mut session) = session_mgr.get_session(&sid) {
                // Convert to backend Message format
                let backend_msg = Message::new(MessageRole::User, content);
                session.history.push(backend_msg);
                let _ = session_mgr.update_session(session);
            }
        }

        Ok(())
    }

    /// Add an assistant message (used for mock responses)
    pub async fn add_assistant_message(&self, content: String) -> anyhow::Result<()> {
        // Add to local state
        let session_id = {
            let mut state = self.state.write().await;

            // Use Message::new helper which properly constructs MessagePart::Text
            let message = Message::new(MessageRole::Assistant, content.clone());
            state.messages.push(message);
            
            // Back to Active (idle equivalent in SessionStatus)
            state.session_status = SessionStatus::Active;

            // Get session ID for persistence
            state.current_session.as_ref().map(|s| s.id.clone())
        };

        // Persist to SessionManager if we have a current session
        if let Some(sid) = session_id {
            let mut session_mgr = self.session_manager.write().await;
            if let Ok(mut session) = session_mgr.get_session(&sid) {
                // Convert to backend Message format
                let backend_msg = Message::new(MessageRole::Assistant, content);
                session.history.push(backend_msg);
                let _ = session_mgr.update_session(session);
            }
        }

        Ok(())
    }

    /// Send message to AI and get response
    pub async fn send_message(&self, content: String) -> anyhow::Result<()> {
        // Add user message
        self.add_user_message(content.clone()).await?;

        // Build chat request from current session messages
        let (model, messages) = {
            let state = self.state.read().await;
            
            let model = state.current_model_id.clone()
                .unwrap_or_else(|| "claude-sonnet-4-20250514".to_string());
            
            // Convert session messages to provider messages
            let provider_messages: Vec<ProviderMessage> = state.messages
                .iter()
                .map(|msg| {
                    let role = match msg.role {
                        MessageRole::User => "user",
                        MessageRole::Assistant => "assistant",
                        MessageRole::System => "system",
                    };
                    ProviderMessage {
                        role: role.to_string(),
                        content: msg.content(),
                    }
                })
                .collect();
            
            (model, provider_messages)
        };

        // Try to send via provider manager
        let response = {
            let mut provider_mgr = self.provider_manager.write().await;
            
            if let Some(mgr) = provider_mgr.as_mut() {
                let request = ChatRequest {
                    model: model.clone(),
                    messages,
                    temperature: Some(0.7),
                    max_tokens: Some(4096),
                    stream: false,
                };

                match mgr.chat(request).await {
                    Ok(chat_response) => {
                        // Update token tracking in state
                        tracing::debug!(
                            "Chat response: {} tokens used (prompt: {}, completion: {})",
                            chat_response.usage.total_tokens,
                            chat_response.usage.prompt_tokens,
                            chat_response.usage.completion_tokens
                        );
                        Ok((chat_response.content, chat_response.usage))
                    }
                    Err(e) => {
                        tracing::error!("Provider chat error: {}", e);
                        Err(anyhow::anyhow!("AI provider error: {}", e))
                    }
                }
            } else {
                // No provider configured - return helpful error
                Err(anyhow::anyhow!(
                    "No AI provider configured. Set ANTHROPIC_API_KEY, OPENAI_API_KEY, or start Ollama."
                ))
            }
        };

        match response {
            Ok((assistant_content, usage)) => {
                // Update token tracking in state
                {
                    let mut state = self.state.write().await;
                    state.session_tokens.prompt_tokens += usage.prompt_tokens;
                    state.session_tokens.completion_tokens += usage.completion_tokens;
                    state.session_tokens.total_tokens += usage.total_tokens;
                    // Calculate cost based on model pricing
                    let model_id = state.current_model_id.clone().unwrap_or_default();
                    state.session_tokens.estimated_cost += calculate_token_cost(
                        &model_id,
                        usage.prompt_tokens,
                        usage.completion_tokens,
                    );
                }
                self.add_assistant_message(assistant_content).await?;
            }
            Err(e) => {
                // Add error as assistant message so user sees it
                let error_msg = format!("⚠️ Error: {}\n\nPlease check your API key configuration.", e);
                self.add_assistant_message(error_msg).await?;
            }
        }

        Ok(())
    }

    /// Send message to AI with streaming response (simulated word-by-word)
    /// 
    /// Returns a WordStream that yields words one at a time for visual typing effect.
    /// The full response is fetched first, then streamed word-by-word to the UI.
    pub async fn send_message_streaming(&self, content: String) -> anyhow::Result<(WordStream, ProviderTokenUsage)> {
        // Add user message
        self.add_user_message(content.clone()).await?;

        // Build chat request from current session messages
        let (model, messages) = {
            let state = self.state.read().await;
            
            let model = state.current_model_id.clone()
                .unwrap_or_else(|| "claude-sonnet-4-20250514".to_string());
            
            // Convert session messages to provider messages
            let provider_messages: Vec<ProviderMessage> = state.messages
                .iter()
                .map(|msg| {
                    let role = match msg.role {
                        MessageRole::User => "user",
                        MessageRole::Assistant => "assistant",
                        MessageRole::System => "system",
                    };
                    ProviderMessage {
                        role: role.to_string(),
                        content: msg.content(),
                    }
                })
                .collect();
            
            (model, provider_messages)
        };

        // Get response from provider (blocking)
        let (assistant_content, usage) = {
            let mut provider_mgr = self.provider_manager.write().await;
            
            if let Some(mgr) = provider_mgr.as_mut() {
                let request = ChatRequest {
                    model: model.clone(),
                    messages,
                    temperature: Some(0.7),
                    max_tokens: Some(4096),
                    stream: false,
                };

                match mgr.chat(request).await {
                    Ok(chat_response) => {
                        tracing::debug!(
                            "Streaming response: {} tokens used",
                            chat_response.usage.total_tokens
                        );
                        (chat_response.content, chat_response.usage)
                    }
                    Err(e) => {
                        tracing::error!("Provider chat error: {}", e);
                        let error_msg = format!("⚠️ Error: {}\n\nPlease check your API key configuration.", e);
                        let empty_usage = ProviderTokenUsage { prompt_tokens: 0, completion_tokens: 0, total_tokens: 0 };
                        return Ok((WordStream::new(error_msg, 30), empty_usage));
                    }
                }
            } else {
                let error_msg = "No AI provider configured. Set ANTHROPIC_API_KEY, OPENAI_API_KEY, or start Ollama.".to_string();
                let empty_usage = ProviderTokenUsage { prompt_tokens: 0, completion_tokens: 0, total_tokens: 0 };
                return Ok((WordStream::new(error_msg, 30), empty_usage));
            }
        };

        // Update token tracking in state
        {
            let mut state = self.state.write().await;
            state.session_tokens.prompt_tokens += usage.prompt_tokens;
            state.session_tokens.completion_tokens += usage.completion_tokens;
            state.session_tokens.total_tokens += usage.total_tokens;
            // Calculate cost based on model pricing
            let model_id = state.current_model_id.clone().unwrap_or_default();
            state.session_tokens.estimated_cost += calculate_token_cost(
                &model_id,
                usage.prompt_tokens,
                usage.completion_tokens,
            );
        }

        // Store the full response for session persistence
        self.add_assistant_message(assistant_content.clone()).await?;

        // Return WordStream for visual streaming effect (50ms per word)
        Ok((WordStream::new(assistant_content, 50), usage))
    }

    // === Agent Operations ===

    /// Switch to next agent
    pub async fn cycle_agent(&self) {
        let mut state = self.state.write().await;
        state.cycle_agent();
    }

    /// Set specific agent
    pub async fn set_agent(&self, name: &str) {
        let mut state = self.state.write().await;
        if state.agents.iter().any(|a| a.name == name) {
            state.current_agent = name.to_string();
        }
    }

    // === Model Operations ===

    /// Switch model
    pub async fn set_model(&self, provider_id: &str, model_id: &str) {
        let mut state = self.state.write().await;
        state.current_provider_id = Some(provider_id.to_string());
        state.current_model_id = Some(model_id.to_string());

        // Add to recent
        let model = (provider_id.to_string(), model_id.to_string());
        state.recent_models.retain(|m| m != &model);
        state.recent_models.insert(0, model);
        if state.recent_models.len() > 10 {
            state.recent_models.pop();
        }
    }

    // === MCP Operations ===

    /// Toggle MCP server
    pub async fn toggle_mcp(&self, server_id: &str) -> anyhow::Result<()> {
        // First check if server is connected
        let is_connected = {
            let mcp_mgr = self.mcp_manager.read().await;
            if let Some(manager) = mcp_mgr.as_ref() {
                if let Ok(server) = manager.get_server(server_id).await {
                    matches!(server.health.state, ServerState::Connected)
                } else {
                    false
                }
            } else {
                return Ok(()); // No manager, nothing to do
            }
        };
        
        // Now toggle the server
        {
            let mcp_mgr = self.mcp_manager.write().await;
            if let Some(manager) = mcp_mgr.as_ref() {
                if is_connected {
                    manager.stop_server(server_id).await?;
                } else {
                    manager.start_server(server_id).await?;
                }
            }
        }
        
        // Reload MCP status
        self.load_mcp_status().await?;
        
        Ok(())
    }

    /// Execute an MCP tool
    pub async fn execute_mcp_tool(
        &self,
        server_id: &str,
        tool_name: &str,
        parameters: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        let mcp_mgr = self.mcp_manager.read().await;
        
        if let Some(manager) = mcp_mgr.as_ref() {
            // Get server and its transport
            let server = manager.get_server(server_id).await?;
            
            if let Some(transport) = &server.transport {
                // Create executor with transport
                let executor = MCPToolExecutor::new(
                    server_id.to_string(),
                    transport.clone(),
                    std::sync::Arc::new(ricecoder_mcp::MCPPermissionManager::new()),
                );
                
                let context = ToolExecutionContext {
                    tool_name: tool_name.to_string(),
                    parameters: if let serde_json::Value::Object(map) = parameters {
                        map.into_iter().collect()
                    } else {
                        std::collections::HashMap::new()
                    },
                    user_id: None,
                    session_id: None,
                    timeout: std::time::Duration::from_secs(30),
                    metadata: std::collections::HashMap::new(),
                };
                
                let result = executor.execute(&context).await?;
                return Ok(result.result.unwrap_or(serde_json::Value::Null));
            }
        }
        
        anyhow::bail!("MCP server or transport not available")
    }

    /// List available tools for an MCP server
    pub async fn list_mcp_tools(&self, server_id: &str) -> anyhow::Result<Vec<String>> {
        let mcp_mgr = self.mcp_manager.read().await;
        
        if let Some(manager) = mcp_mgr.as_ref() {
            let server = manager.get_server(server_id).await?;
            return Ok(server.tools.into_iter().map(|t| t.name).collect());
        }
        
        Ok(vec![])
    }

    // === UI State ===

    /// Toggle command palette
    pub async fn toggle_command_palette(&self) {
        let mut state = self.state.write().await;
        state.command_palette_visible = !state.command_palette_visible;
    }

    /// Close command palette
    pub async fn close_command_palette(&self) {
        let mut state = self.state.write().await;
        state.command_palette_visible = false;
    }

    /// Toggle tips
    pub async fn toggle_tips(&self) {
        let mut state = self.state.write().await;
        state.tips_hidden = !state.tips_hidden;
    }

    /// Get action for a key event using keybind engine
    pub async fn get_action_for_key(&self, event: crossterm::event::KeyEvent) -> Option<String> {
        use crate::tui::keybind_bridge::to_key_combo;
        
        let combo = to_key_combo(event)?;
        let engine = self.keybind_engine.read().await;
        engine.get_action(&combo).map(|s| s.to_string())
    }
}

impl Default for AppContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_app_context_creation() {
        let ctx = AppContext::new();
        let state = ctx.state.read().await;
        assert_eq!(state.sync_status, SyncStatus::Loading);
        assert_eq!(state.current_agent, "build");
    }

    #[tokio::test]
    async fn test_agent_cycling() {
        let ctx = AppContext::new();
        {
            let state = ctx.state.read().await;
            assert_eq!(state.current_agent, "build");
        }

        ctx.cycle_agent().await;

        {
            let state = ctx.state.read().await;
            assert_eq!(state.current_agent, "plan");
        }

        ctx.cycle_agent().await;

        {
            let state = ctx.state.read().await;
            assert_eq!(state.current_agent, "build");
        }
    }

    #[tokio::test]
    async fn test_session_creation() {
        let ctx = AppContext::new();
        let session_id = ctx.create_session().await.unwrap();
        assert!(!session_id.is_empty());

        let state = ctx.state.read().await;
        assert!(state.current_session.is_some());
    }

    #[tokio::test]
    async fn test_message_flow() {
        let ctx = AppContext::new();
        ctx.create_session().await.unwrap();

        ctx.send_message("Hello".to_string()).await.unwrap();

        let state = ctx.state.read().await;
        assert_eq!(state.messages.len(), 2); // User + Assistant
    }
}
