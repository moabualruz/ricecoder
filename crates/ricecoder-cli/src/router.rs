// Command routing and dispatch
// Adapted from automation/src/cli/router.rs

use crate::commands::*;
use crate::error::{CliError, CliResult};
use clap::{Parser, Subcommand};

/// RiceCoder - Terminal-first, spec-driven coding assistant
#[derive(Parser, Debug)]
#[command(name = "rice")]
#[command(bin_name = "rice")]
#[command(about = "Terminal-first, spec-driven coding assistant")]
#[command(
    long_about = "RiceCoder: A terminal-first, spec-driven coding assistant.\n\nGenerate code from specifications, refactor existing code, and get AI-powered code reviews.\n\nFor more information, visit: https://github.com/moabualruz/ricecoder/wiki"
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

/// Route and execute commands
pub struct CommandRouter;

impl CommandRouter {
    /// Parse CLI arguments and route to appropriate handler
    pub fn route() -> CliResult<()> {
        let cli = Cli::parse();

        // Initialize logging based on CLI flags
        crate::logging::init_logging(cli.verbose, cli.quiet);

        Self::execute(&cli)
    }

    /// Execute a command
    pub fn execute(cli: &Cli) -> CliResult<()> {
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
                cmd.execute()
            }
            Commands::Gen { spec } => {
                let cmd = GenCommand::new(spec.clone());
                cmd.execute()
            }
            Commands::Chat {
                message,
                provider,
                model,
            } => {
                let cmd = ChatCommand::new(message.clone(), provider.clone(), model.clone());
                cmd.execute()
            }
            Commands::Refactor { file } => {
                let cmd = RefactorCommand::new(file.clone());
                cmd.execute()
            }
            Commands::Review { file } => {
                let cmd = ReviewCommand::new(file.clone());
                cmd.execute()
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
                cmd.execute()
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
                cmd.execute()
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
                cmd.execute()
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
                cmd.execute()
            }
            Commands::Lsp {
                log_level,
                port,
                debug,
            } => {
                let cmd = lsp::LspCommand::new(log_level.clone(), *port, *debug);
                cmd.execute()
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
                cmd.execute()
            }
        }
    }

    /// Find similar command for suggestions
    pub fn find_similar(command: &str) -> Option<String> {
        let commands = ["init", "gen", "chat", "refactor", "review", "config", "tui"];

        // Simple similarity check: commands that start with same letter
        commands
            .iter()
            .find(|c| c.starts_with(&command[0..1.min(command.len())]))
            .map(|s| s.to_string())
    }
}


