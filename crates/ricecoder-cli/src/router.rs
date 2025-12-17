// Command routing and dispatch
// Adapted from automation/src/cli/router.rs

use crate::commands::*;
use ricecoder_mcp::compliance::ComplianceReportType;
use crate::error::{CliError, CliResult};
use clap::{Parser, Subcommand};

/// RiceCoder - Terminal-first, spec-driven coding assistant
#[derive(Parser, Debug)]
#[command(name = "rice")]
#[command(bin_name = "rice")]
#[command(about = "Terminal-first, spec-driven coding assistant")]
#[command(
    long_about = "RiceCoder: A terminal-first, spec-driven coding assistant.\n\nGenerate code from specifications, refactor existing code, and get AI-powered code reviews.\n\n🚀 Quick Start:\n  • rice init          Initialize a new project\n  • rice chat          Start interactive AI chat\n  • rice refactor      AI-powered code refactoring\n  • rice sessions list Manage coding sessions\n  • rice providers     Configure AI providers\n\nFor more information, visit: https://github.com/moabualruz/ricecoder/wiki"
)]
#[command(version)]
#[command(author = "RiceCoder Contributors")]
#[command(disable_help_subcommand = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Minimize output
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Preview changes without applying them
    #[arg(long, global = true)]
    pub dry_run: bool,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Initialize a new ricecoder project
    #[command(about = "Initialize a new ricecoder project with default configuration")]
    Init {
        /// Project path (default: current directory)
        #[arg(value_name = "PATH")]
        path: Option<String>,

        /// Enable interactive setup wizard
        #[arg(short, long)]
        interactive: bool,

        /// AI provider to use (default: zen)
        #[arg(long, default_value = "zen")]
        provider: String,

        /// Model to use
        #[arg(long)]
        model: Option<String>,

        /// Force overwrite existing configuration
        #[arg(long)]
        force: bool,
    },

    /// Generate code from a specification
    #[command(about = "Generate code from a specification file")]
    Gen {
        /// Path to specification file
        #[arg(value_name = "SPEC")]
        spec: String,
    },

    /// Interactive chat mode with spec awareness
    #[command(about = "Enter interactive chat mode for free-form coding assistance")]
    Chat {
        /// Initial message to send
        #[arg(value_name = "MESSAGE")]
        message: Option<String>,

        /// AI provider to use (openai, anthropic, local)
        #[arg(short, long)]
        provider: Option<String>,

        /// Model to use
        #[arg(short, long)]
        model: Option<String>,
    },

    /// Refactor existing code
    #[command(about = "Refactor existing code using AI assistance")]
    Refactor {
        /// File to refactor
        #[arg(value_name = "FILE")]
        file: String,
    },

    /// Review code for improvements
    #[command(about = "Review code for improvements and best practices")]
    Review {
        /// File to review
        #[arg(value_name = "FILE")]
        file: String,
    },

    /// Manage configuration settings
    #[command(about = "View and manage ricecoder configuration")]
    Config {
        #[command(subcommand)]
        action: Option<ConfigSubcommand>,
    },

    /// Compliance reporting and automation
    #[command(about = "Generate compliance reports and run automated compliance checks")]
    Compliance {
        #[command(subcommand)]
        action: Option<ComplianceSubcommand>,
    },

    /// Generate shell completions
    #[command(about = "Generate shell completion scripts")]
    Completions {
        /// Shell to generate completions for (bash, zsh, fish, powershell)
        #[arg(value_name = "SHELL")]
        shell: String,
    },

    /// Manage and execute custom commands
    #[command(about = "Manage and execute custom commands")]
    Custom {
        #[command(subcommand)]
        action: Option<CustomSubcommand>,
    },

    /// Launch the terminal user interface
    #[command(about = "Launch the beautiful terminal user interface")]
    Tui {
        /// Theme to use (dark, light, monokai, dracula, nord)
        #[arg(short, long)]
        theme: Option<String>,

        /// Enable vim keybindings
        #[arg(long)]
        vim_mode: bool,

        /// Custom config file path
        #[arg(short, long)]
        config: Option<String>,

        /// AI provider to use (openai, anthropic, local)
        #[arg(short, long)]
        provider: Option<String>,

        /// Model to use
        #[arg(short, long)]
        model: Option<String>,
    },

    /// Manage sessions
    #[command(about = "Manage ricecoder sessions")]
    Sessions {
        #[command(subcommand)]
        action: Option<SessionsSubcommand>,
    },

    /// Manage AI providers
    #[command(about = "Manage AI providers and their configurations")]
    Providers {
        #[command(subcommand)]
        action: Option<ProvidersSubcommand>,
    },

    /// Manage Model Context Protocol servers and tools
    #[command(about = "Manage Model Context Protocol (MCP) servers and tools")]
    Mcp {
        #[command(subcommand)]
        action: Option<McpSubcommand>,
    },

    /// Start the Language Server Protocol server
    #[command(about = "Start the Language Server Protocol server for IDE integration")]
    Lsp {
        /// Log level (trace, debug, info, warn, error)
        #[arg(short, long, default_value = "info")]
        log_level: Option<String>,

        /// Port for TCP transport (future support)
        #[arg(short, long)]
        port: Option<u16>,

        /// Enable debug mode for verbose logging
        #[arg(long)]
        debug: bool,
    },

    /// Manage hooks for event-driven automation
    #[command(about = "Manage hooks for event-driven automation")]
    Hooks {
        #[command(subcommand)]
        action: Option<HooksSubcommand>,
    },

    /// Show help and tutorials
    #[command(about = "Show help, tutorials, and troubleshooting guides")]
    Help {
        /// Topic to get help on (command name, 'tutorial', 'troubleshooting')
        #[arg(value_name = "TOPIC")]
        topic: Option<String>,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum HooksSubcommand {
    /// List all hooks
    #[command(about = "List all registered hooks")]
    List {
        /// Output format (table or json)
        #[arg(short, long)]
        format: Option<String>,
    },

    /// Inspect a specific hook
    #[command(about = "Inspect a specific hook")]
    Inspect {
        /// Hook ID
        #[arg(value_name = "ID")]
        id: String,

        /// Output format (table or json)
        #[arg(short, long)]
        format: Option<String>,
    },

    /// Enable a hook
    #[command(about = "Enable a hook")]
    Enable {
        /// Hook ID
        #[arg(value_name = "ID")]
        id: String,
    },

    /// Disable a hook
    #[command(about = "Disable a hook")]
    Disable {
        /// Hook ID
        #[arg(value_name = "ID")]
        id: String,
    },

    /// Delete a hook
    #[command(about = "Delete a hook")]
    Delete {
        /// Hook ID
        #[arg(value_name = "ID")]
        id: String,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum SessionsSubcommand {
    /// List all sessions
    #[command(about = "List all sessions")]
    List,

    /// Create a new session
    #[command(about = "Create a new session")]
    Create {
        /// Session name
        #[arg(value_name = "NAME")]
        name: String,
    },

    /// Delete a session
    #[command(about = "Delete a session")]
    Delete {
        /// Session ID
        #[arg(value_name = "ID")]
        id: String,
    },

    /// Rename a session
    #[command(about = "Rename a session")]
    Rename {
        /// Session ID
        #[arg(value_name = "ID")]
        id: String,

        /// New session name
        #[arg(value_name = "NAME")]
        name: String,
    },

    /// Switch to a session
    #[command(about = "Switch to a session")]
    Switch {
        /// Session ID
        #[arg(value_name = "ID")]
        id: String,
    },

    /// Show session info
    #[command(about = "Show session information")]
    Info {
        /// Session ID
        #[arg(value_name = "ID")]
        id: String,
    },

    /// Share a session with a shareable link
    #[command(about = "Generate a shareable link for the current session")]
    Share {
        /// Expiration time in seconds (optional)
        #[arg(long)]
        expires_in: Option<u64>,

        /// Exclude conversation history from share
        #[arg(long)]
        no_history: bool,

        /// Exclude project context from share
        #[arg(long)]
        no_context: bool,
    },

    /// List all active shares
    #[command(about = "List all active shares for the current user")]
    ShareList,

    /// Revoke a share
    #[command(about = "Revoke a share by ID")]
    ShareRevoke {
        /// Share ID to revoke
        #[arg(value_name = "SHARE_ID")]
        share_id: String,
    },

    /// Show share information
    #[command(about = "Show detailed information about a share")]
    ShareInfo {
        /// Share ID
        #[arg(value_name = "SHARE_ID")]
        share_id: String,
    },

    /// View a shared session
    #[command(about = "View a shared session by share ID")]
    ShareView {
        /// Share ID to view
        #[arg(value_name = "SHARE_ID")]
        share_id: String,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum ProvidersSubcommand {
    /// List all available providers
    #[command(about = "List all configured providers and their status")]
    List,

    /// Switch to a specific provider
    #[command(about = "Switch to a specific AI provider")]
    Switch {
        /// Provider ID
        #[arg(value_name = "PROVIDER_ID")]
        provider_id: String,
    },

    /// Show provider status
    #[command(about = "Show status of a specific provider or current provider")]
    Status {
        /// Provider ID (optional, shows current if not specified)
        #[arg(value_name = "PROVIDER_ID")]
        provider_id: Option<String>,
    },

    /// Show provider performance metrics
    #[command(about = "Show performance metrics for providers")]
    Performance {
        /// Provider ID (optional, shows all if not specified)
        #[arg(value_name = "PROVIDER_ID")]
        provider_id: Option<String>,
    },

    /// Check provider health
    #[command(about = "Check health status of providers")]
    Health {
        /// Provider ID (optional, checks all if not specified)
        #[arg(value_name = "PROVIDER_ID")]
        provider_id: Option<String>,
    },

    /// List available models
    #[command(about = "List available models for providers")]
    Models {
        /// Provider ID (optional, shows all providers if not specified)
        #[arg(value_name = "PROVIDER_ID")]
        provider_id: Option<String>,

        /// Filter models (free, chat, completion)
        #[arg(short, long)]
        filter: Option<String>,
    },

    /// Show failover provider
    #[command(about = "Show failover provider for a failing provider")]
    Failover {
        /// Provider ID
        #[arg(value_name = "PROVIDER_ID")]
        provider_id: String,
    },

    /// Show community analytics
    #[command(about = "Show community provider analytics")]
    Community {
        /// Provider ID (optional, shows popular providers if not specified)
        #[arg(value_name = "PROVIDER_ID")]
        provider_id: Option<String>,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum McpSubcommand {
    /// List configured MCP servers
    #[command(about = "List all configured MCP servers")]
    List,

    /// Add a new MCP server
    #[command(about = "Add a new MCP server configuration")]
    Add {
        /// Server name
        #[arg(value_name = "NAME")]
        name: String,

        /// Server command
        #[arg(value_name = "COMMAND")]
        command: String,

        /// Additional arguments for the server command
        #[arg(value_name = "ARGS")]
        args: Vec<String>,
    },

    /// Remove an MCP server
    #[command(about = "Remove an MCP server configuration")]
    Remove {
        /// Server name
        #[arg(value_name = "NAME")]
        name: String,
    },

    /// Show MCP server information
    #[command(about = "Show detailed information about an MCP server")]
    Info {
        /// Server name
        #[arg(value_name = "NAME")]
        name: String,
    },

    /// Test MCP server connection
    #[command(about = "Test connection to an MCP server")]
    Test {
        /// Server name
        #[arg(value_name = "NAME")]
        name: String,
    },

    /// List available tools from MCP servers
    #[command(about = "List all available tools from configured MCP servers")]
    Tools,

    /// Execute a tool from an MCP server
    #[command(about = "Execute a tool from an MCP server")]
    Execute {
        /// Server name
        #[arg(value_name = "SERVER")]
        server: String,

        /// Tool name
        #[arg(value_name = "TOOL")]
        tool: String,

        /// Tool parameters as JSON
        #[arg(value_name = "PARAMETERS")]
        parameters: String,
    },

    /// Show MCP system status
    #[command(about = "Show overall MCP system status and health")]
    Status,
}

#[derive(Subcommand, Debug, Clone)]
pub enum CustomSubcommand {
    /// List all available custom commands
    #[command(about = "Display all available custom commands")]
    List,

    /// Show info for a specific custom command
    #[command(about = "Show info for a specific custom command")]
    Info {
        /// Command name
        #[arg(value_name = "NAME")]
        name: String,
    },

    /// Execute a custom command
    #[command(about = "Execute a custom command")]
    Run {
        /// Command name
        #[arg(value_name = "NAME")]
        name: String,

        /// Arguments to pass to the command
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Load custom commands from a file
    #[command(about = "Load custom commands from a JSON or Markdown file")]
    Load {
        /// Path to command definition file
        #[arg(value_name = "FILE")]
        file: String,
    },

    /// Search for custom commands
    #[command(about = "Search for custom commands by name or description")]
    Search {
        /// Search query
        #[arg(value_name = "QUERY")]
        query: String,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum ConfigSubcommand {
    /// List all configuration values
    #[command(about = "Display all configuration settings")]
    List,

    /// Get a specific configuration value
    #[command(about = "Get a configuration value by key")]
    Get {
        /// Configuration key (e.g., provider.default, storage.mode)
        #[arg(value_name = "KEY")]
        key: String,
    },

    /// Set a configuration value
    #[command(about = "Set a configuration value")]
    Set {
        /// Configuration key
        #[arg(value_name = "KEY")]
        key: String,

        /// Configuration value
        #[arg(value_name = "VALUE")]
        value: String,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum ComplianceSubcommand {
    /// Generate compliance reports
    #[command(about = "Generate compliance reports for SOC 2, GDPR, HIPAA")]
    Report {
        /// Compliance standard (soc2, gdpr, hipaa)
        #[arg(value_name = "STANDARD")]
        standard: String,
    },

    /// Check compliance status
    #[command(about = "Check compliance status for a specific standard")]
    Check {
        /// Compliance standard (soc2, gdpr, hipaa)
        #[arg(value_name = "STANDARD")]
        standard: String,
    },

    /// Run automated compliance validation
    #[command(about = "Run automated compliance validation for all standards")]
    Validate,

    /// Show compliance monitoring dashboard
    #[command(about = "Show compliance monitoring dashboard and metrics")]
    Monitor,

    /// Generate compliance documentation
    #[command(about = "Generate compliance documentation and guides")]
    Docs,
}

/// Route and execute commands
pub struct CommandRouter;

impl CommandRouter {
    /// Parse CLI arguments and route to appropriate handler
    pub async fn route() -> CliResult<()> {
        let cli = Cli::parse();

        // Initialize logging based on CLI flags
        crate::logging::init_logging(cli.verbose, cli.quiet);

        Self::execute(&cli).await
    }

    /// Execute a command
    pub async fn execute(cli: &Cli) -> CliResult<()> {
        // Default to TUI if no command specified
        let command = cli.command.clone().unwrap_or(Commands::Tui {
            theme: None,
            vim_mode: false,
            config: None,
            provider: None,
            model: None,
        });

        match &command {
            Commands::Init {
                path,
                interactive,
                provider,
                model,
                force,
            } => {
                let cmd = InitCommand::new(path.clone())
                    .with_interactive(*interactive)
                    .with_provider(provider.clone())
                    .with_model(model.clone())
                    .with_force(*force);
                cmd.execute().await
            }
            Commands::Gen { spec } => {
                let cmd = GenCommand::new(spec.clone());
                cmd.execute().await
            }
            Commands::Chat {
                message,
                provider,
                model,
            } => {
                let cmd = ChatCommand::new(message.clone(), provider.clone(), model.clone());
                cmd.execute().await
            }
            Commands::Refactor { file } => {
                let cmd = RefactorCommand::new(file.clone());
                cmd.execute().await
            }
            Commands::Review { file } => {
                let cmd = ReviewCommand::new(file.clone());
                cmd.execute().await
            }
            Commands::Config { action } => {
                let config_action = match action {
                    Some(ConfigSubcommand::List) | None => config::ConfigAction::List,
                    Some(ConfigSubcommand::Get { key }) => config::ConfigAction::Get(key.clone()),
                    Some(ConfigSubcommand::Set { key, value }) => {
                        config::ConfigAction::Set(key.clone(), value.clone())
                    }
                };
                let cmd = ConfigCommand::new(config_action);
                cmd.execute().await
            }
            Commands::Compliance { action } => {
                let compliance_action = match action {
                    Some(ComplianceSubcommand::Report { standard }) => {
                        let report_type = match standard.to_lowercase().as_str() {
                            "soc2" => compliance::ComplianceAction::Report(ComplianceReportType::Soc2Type2),
                            "gdpr" => compliance::ComplianceAction::Report(ComplianceReportType::Gdpr),
                            "hipaa" => compliance::ComplianceAction::Report(ComplianceReportType::Hipaa),
                            _ => return Err(CliError::InvalidArgument { message: format!("Unknown compliance standard: {}", standard) }),
                        };
                        report_type
                    }
                    Some(ComplianceSubcommand::Check { standard }) => {
                        let report_type = match standard.to_lowercase().as_str() {
                            "soc2" => compliance::ComplianceAction::Check(ComplianceReportType::Soc2Type2),
                            "gdpr" => compliance::ComplianceAction::Check(ComplianceReportType::Gdpr),
                            "hipaa" => compliance::ComplianceAction::Check(ComplianceReportType::Hipaa),
                            _ => return Err(CliError::InvalidArgument { message: format!("Unknown compliance standard: {}", standard) }),
                        };
                        report_type
                    }
                    Some(ComplianceSubcommand::Validate) => compliance::ComplianceAction::Validate,
                    Some(ComplianceSubcommand::Monitor) => compliance::ComplianceAction::Monitor,
                    Some(ComplianceSubcommand::Docs) => compliance::ComplianceAction::Docs,
                    None => compliance::ComplianceAction::Validate, // Default to validate
                };
                let cmd = ComplianceCommand::new(compliance_action);
                cmd.execute().await
            }
            Commands::Completions { shell } => {
                crate::completion::generate_completions(shell).map_err(CliError::Internal)
            }
            Commands::Custom { action } => {
                let custom_action = match action {
                    Some(CustomSubcommand::List) | None => custom::CustomAction::List,
                    Some(CustomSubcommand::Info { name }) => {
                        custom::CustomAction::Info(name.clone())
                    }
                    Some(CustomSubcommand::Run { name, args }) => {
                        custom::CustomAction::Run(name.clone(), args.clone())
                    }
                    Some(CustomSubcommand::Load { file }) => {
                        custom::CustomAction::Load(file.clone())
                    }
                    Some(CustomSubcommand::Search { query }) => {
                        custom::CustomAction::Search(query.clone())
                    }
                };
                let cmd = custom::CustomCommandHandler::new(custom_action);
                cmd.execute().await
            }
            Commands::Tui {
                theme,
                vim_mode,
                config,
                provider,
                model,
            } => {
                let config_path = config.as_ref().map(std::path::PathBuf::from);
                let cmd = TuiCommand::new(
                    theme.clone(),
                    *vim_mode,
                    config_path,
                    provider.clone(),
                    model.clone(),
                );
                cmd.execute().await
            }
            Commands::Sessions { action } => {
                let sessions_action = match action {
                    Some(SessionsSubcommand::List) | None => sessions::SessionsAction::List,
                    Some(SessionsSubcommand::Create { name }) => {
                        sessions::SessionsAction::Create { name: name.clone() }
                    }
                    Some(SessionsSubcommand::Delete { id }) => {
                        sessions::SessionsAction::Delete { id: id.clone() }
                    }
                    Some(SessionsSubcommand::Rename { id, name }) => {
                        sessions::SessionsAction::Rename {
                            id: id.clone(),
                            name: name.clone(),
                        }
                    }
                    Some(SessionsSubcommand::Switch { id }) => {
                        sessions::SessionsAction::Switch { id: id.clone() }
                    }
                    Some(SessionsSubcommand::Info { id }) => {
                        sessions::SessionsAction::Info { id: id.clone() }
                    }
                    Some(SessionsSubcommand::Share {
                        expires_in,
                        no_history,
                        no_context,
                    }) => sessions::SessionsAction::Share {
                        expires_in: *expires_in,
                        no_history: *no_history,
                        no_context: *no_context,
                    },
                    Some(SessionsSubcommand::ShareList) => sessions::SessionsAction::ShareList,
                    Some(SessionsSubcommand::ShareRevoke { share_id }) => {
                        sessions::SessionsAction::ShareRevoke {
                            share_id: share_id.clone(),
                        }
                    }
                    Some(SessionsSubcommand::ShareInfo { share_id }) => {
                        sessions::SessionsAction::ShareInfo {
                            share_id: share_id.clone(),
                        }
                    }
                    Some(SessionsSubcommand::ShareView { share_id }) => {
                        sessions::SessionsAction::ShareView {
                            share_id: share_id.clone(),
                        }
                    }
                };
                let cmd = SessionsCommand::new(sessions_action);
                cmd.execute().await
            }
            Commands::Providers { action } => {
                let providers_action = match action {
                    Some(ProvidersSubcommand::List) | None => providers::ProvidersAction::List,
                    Some(ProvidersSubcommand::Switch { provider_id }) => {
                        providers::ProvidersAction::Switch { provider_id: provider_id.clone() }
                    }
                    Some(ProvidersSubcommand::Status { provider_id }) => {
                        providers::ProvidersAction::Status { provider_id: provider_id.clone() }
                    }
                    Some(ProvidersSubcommand::Performance { provider_id }) => {
                        providers::ProvidersAction::Performance { provider_id: provider_id.clone() }
                    }
                    Some(ProvidersSubcommand::Health { provider_id }) => {
                        providers::ProvidersAction::Health { provider_id: provider_id.clone() }
                    }
                    Some(ProvidersSubcommand::Models { provider_id, filter }) => {
                        providers::ProvidersAction::Models {
                            provider_id: provider_id.clone(),
                            filter: filter.clone(),
                        }
                    }
                    Some(ProvidersSubcommand::Failover { provider_id }) => {
                        providers::ProvidersAction::Failover { provider_id: provider_id.clone() }
                    }
                    Some(ProvidersSubcommand::Community { provider_id }) => {
                        providers::ProvidersAction::Community { provider_id: provider_id.clone() }
                    }
                };
                let cmd = ProvidersCommand::new(providers_action);
                cmd.execute().await
            }
            Commands::Mcp { action } => {
                let mcp_action = match action {
                    Some(McpSubcommand::List) | None => mcp::McpAction::List,
                    Some(McpSubcommand::Add { name, command, args }) => {
                        mcp::McpAction::Add {
                            name: name.clone(),
                            command: command.clone(),
                            args: args.clone(),
                        }
                    }
                    Some(McpSubcommand::Remove { name }) => {
                        mcp::McpAction::Remove { name: name.clone() }
                    }
                    Some(McpSubcommand::Info { name }) => {
                        mcp::McpAction::Info { name: name.clone() }
                    }
                    Some(McpSubcommand::Test { name }) => {
                        mcp::McpAction::Test { name: name.clone() }
                    }
                    Some(McpSubcommand::Tools) => mcp::McpAction::Tools,
                    Some(McpSubcommand::Execute { server, tool, parameters }) => {
                        let params: serde_json::Value = serde_json::from_str(parameters)
                            .map_err(|e| CliError::Internal(format!("Invalid JSON parameters: {}", e)))?;
                        mcp::McpAction::Execute {
                            server: server.clone(),
                            tool: tool.clone(),
                            parameters: params,
                        }
                    }
                    Some(McpSubcommand::Status) => mcp::McpAction::Status,
                };
                let cmd = McpCommand::new(mcp_action);
                cmd.execute().await
            }
            Commands::Lsp {
                log_level,
                port,
                debug,
            } => {
                let cmd = lsp::LspCommand::new(log_level.clone(), *port, *debug);
                cmd.execute().await
            }
            Commands::Hooks { action } => {
                let hooks_action = match action {
                    Some(HooksSubcommand::List { format }) => hooks::HooksAction::List {
                        format: format.clone(),
                    },
                    None => hooks::HooksAction::List { format: None },
                    Some(HooksSubcommand::Inspect { id, format }) => hooks::HooksAction::Inspect {
                        id: id.clone(),
                        format: format.clone(),
                    },
                    Some(HooksSubcommand::Enable { id }) => {
                        hooks::HooksAction::Enable { id: id.clone() }
                    }
                    Some(HooksSubcommand::Disable { id }) => {
                        hooks::HooksAction::Disable { id: id.clone() }
                    }
                    Some(HooksSubcommand::Delete { id }) => {
                        hooks::HooksAction::Delete { id: id.clone() }
                    }
                };
                let cmd = hooks::HooksCommand::new(hooks_action);
                cmd.execute()
            }
            Commands::Help { topic } => {
                let cmd = HelpCommand::new(topic.clone());
                cmd.execute().await
            }
        }
    }

    /// Find similar command for suggestions
    pub fn find_similar(command: &str) -> Option<String> {
        let commands = ["init", "gen", "chat", "refactor", "review", "config", "sessions", "providers", "mcp", "tui"];

        // Simple similarity check: commands that start with same letter
        commands
            .iter()
            .find(|c| c.starts_with(&command[0..1.min(command.len())]))
            .map(|s| s.to_string())
    }
}


