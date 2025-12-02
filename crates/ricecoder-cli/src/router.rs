// Command routing and dispatch
// Adapted from automation/src/cli/router.rs

use clap::{Parser, Subcommand};
use crate::error::{CliError, CliResult};
use crate::commands::*;

/// RiceCoder - Terminal-first, spec-driven coding assistant
#[derive(Parser, Debug)]
#[command(name = "rice")]
#[command(bin_name = "rice")]
#[command(about = "Terminal-first, spec-driven coding assistant")]
#[command(long_about = "RiceCoder: A terminal-first, spec-driven coding assistant.\n\nGenerate code from specifications, refactor existing code, and get AI-powered code reviews.\n\nFor more information, visit: https://ricecoder.dev")]
#[command(version)]
#[command(author = "RiceCoder Contributors")]
#[command(arg_required_else_help = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

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

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize a new ricecoder project
    #[command(about = "Initialize a new ricecoder project with default configuration")]
    Init {
        /// Project path (default: current directory)
        #[arg(value_name = "PATH")]
        path: Option<String>,
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
}

#[derive(Subcommand, Debug)]
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

#[derive(Subcommand, Debug)]
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
        match &cli.command {
            Commands::Init { path } => {
                let cmd = InitCommand::new(path.clone());
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
                crate::completion::generate_completions(shell)
                    .map_err(|e| CliError::Internal(e))
            }
            Commands::Custom { action } => {
                let custom_action = match action {
                    Some(CustomSubcommand::List) | None => custom::CustomAction::List,
                    Some(CustomSubcommand::Info { name }) => custom::CustomAction::Info(name.clone()),
                    Some(CustomSubcommand::Run { name, args }) => {
                        custom::CustomAction::Run(name.clone(), args.clone())
                    }
                    Some(CustomSubcommand::Load { file }) => custom::CustomAction::Load(file.clone()),
                    Some(CustomSubcommand::Search { query }) => {
                        custom::CustomAction::Search(query.clone())
                    }
                };
                let cmd = custom::CustomCommandHandler::new(custom_action);
                cmd.execute()
            }
        }
    }

    /// Find similar command for suggestions
    pub fn find_similar(command: &str) -> Option<String> {
        let commands = vec!["init", "gen", "chat", "refactor", "review", "config"];
        
        // Simple similarity check: commands that start with same letter
        commands
            .iter()
            .find(|c| c.starts_with(&command[0..1.min(command.len())]))
            .map(|s| s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_similar_command() {
        assert_eq!(CommandRouter::find_similar("i"), Some("init".to_string()));
        assert_eq!(CommandRouter::find_similar("g"), Some("gen".to_string()));
        assert_eq!(CommandRouter::find_similar("c"), Some("chat".to_string()));
    }
}
