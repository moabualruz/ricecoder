//! Router and command parsing tests
//!
//! Tests for CLI command parsing using clap.

use clap::Parser;
use ricecoder_cli::router::{Cli, Commands, ConfigSubcommand, SessionsSubcommand};

#[test]
fn test_parse_init_command() {
    let cli = Cli::parse_from(["rice", "init"]);
    assert!(matches!(cli.command, Some(Commands::Init { .. })));
}

#[test]
fn test_parse_init_with_path() {
    let cli = Cli::parse_from(["rice", "init", "/path/to/project"]);
    if let Some(Commands::Init { path, .. }) = cli.command {
        assert_eq!(path, Some("/path/to/project".to_string()));
    } else {
        panic!("Expected Init command");
    }
}

#[test]
fn test_parse_init_interactive() {
    let cli = Cli::parse_from(["rice", "init", "--interactive"]);
    if let Some(Commands::Init { interactive, .. }) = cli.command {
        assert!(interactive);
    } else {
        panic!("Expected Init command");
    }
}

#[test]
fn test_parse_chat_command() {
    let cli = Cli::parse_from(["rice", "chat"]);
    assert!(matches!(cli.command, Some(Commands::Chat { .. })));
}

#[test]
fn test_parse_chat_with_message() {
    let cli = Cli::parse_from(["rice", "chat", "Hello world"]);
    if let Some(Commands::Chat { message, .. }) = cli.command {
        assert_eq!(message, Some("Hello world".to_string()));
    } else {
        panic!("Expected Chat command");
    }
}

#[test]
fn test_parse_chat_with_provider() {
    let cli = Cli::parse_from(["rice", "chat", "--provider", "openai"]);
    if let Some(Commands::Chat { provider, .. }) = cli.command {
        assert_eq!(provider, Some("openai".to_string()));
    } else {
        panic!("Expected Chat command");
    }
}

#[test]
fn test_parse_gen_command() {
    let cli = Cli::parse_from(["rice", "gen", "spec.md"]);
    if let Some(Commands::Gen { spec }) = cli.command {
        assert_eq!(spec, "spec.md");
    } else {
        panic!("Expected Gen command");
    }
}

#[test]
fn test_parse_refactor_command() {
    let cli = Cli::parse_from(["rice", "refactor", "src/main.rs"]);
    if let Some(Commands::Refactor { file }) = cli.command {
        assert_eq!(file, "src/main.rs");
    } else {
        panic!("Expected Refactor command");
    }
}

#[test]
fn test_parse_review_command() {
    let cli = Cli::parse_from(["rice", "review", "src/lib.rs"]);
    if let Some(Commands::Review { file }) = cli.command {
        assert_eq!(file, "src/lib.rs");
    } else {
        panic!("Expected Review command");
    }
}

#[test]
fn test_parse_config_list() {
    let cli = Cli::parse_from(["rice", "config", "list"]);
    if let Some(Commands::Config { action }) = cli.command {
        assert!(matches!(action, Some(ConfigSubcommand::List)));
    } else {
        panic!("Expected Config command");
    }
}

#[test]
fn test_parse_config_get() {
    let cli = Cli::parse_from(["rice", "config", "get", "provider.default"]);
    if let Some(Commands::Config { action }) = cli.command {
        if let Some(ConfigSubcommand::Get { key }) = action {
            assert_eq!(key, "provider.default");
        } else {
            panic!("Expected Get subcommand");
        }
    } else {
        panic!("Expected Config command");
    }
}

#[test]
fn test_parse_config_set() {
    let cli = Cli::parse_from(["rice", "config", "set", "provider.default", "openai"]);
    if let Some(Commands::Config { action }) = cli.command {
        if let Some(ConfigSubcommand::Set { key, value }) = action {
            assert_eq!(key, "provider.default");
            assert_eq!(value, "openai");
        } else {
            panic!("Expected Set subcommand");
        }
    } else {
        panic!("Expected Config command");
    }
}

#[test]
fn test_parse_sessions_list() {
    let cli = Cli::parse_from(["rice", "sessions", "list"]);
    if let Some(Commands::Sessions { action }) = cli.command {
        assert!(matches!(action, Some(SessionsSubcommand::List)));
    } else {
        panic!("Expected Sessions command");
    }
}

#[test]
fn test_parse_sessions_create() {
    let cli = Cli::parse_from(["rice", "sessions", "create", "my-session"]);
    if let Some(Commands::Sessions { action }) = cli.command {
        if let Some(SessionsSubcommand::Create { name }) = action {
            assert_eq!(name, "my-session");
        } else {
            panic!("Expected Create subcommand");
        }
    } else {
        panic!("Expected Sessions command");
    }
}

#[test]
fn test_parse_tui_command() {
    let cli = Cli::parse_from(["rice", "tui"]);
    assert!(matches!(cli.command, Some(Commands::Tui { .. })));
}

#[test]
fn test_parse_tui_with_theme() {
    let cli = Cli::parse_from(["rice", "tui", "--theme", "dark"]);
    if let Some(Commands::Tui { theme, .. }) = cli.command {
        assert_eq!(theme, Some("dark".to_string()));
    } else {
        panic!("Expected Tui command");
    }
}

#[test]
fn test_parse_tui_vim_mode() {
    let cli = Cli::parse_from(["rice", "tui", "--vim-mode"]);
    if let Some(Commands::Tui { vim_mode, .. }) = cli.command {
        assert!(vim_mode);
    } else {
        panic!("Expected Tui command");
    }
}

#[test]
fn test_parse_lsp_command() {
    let cli = Cli::parse_from(["rice", "lsp"]);
    assert!(matches!(cli.command, Some(Commands::Lsp { .. })));
}

#[test]
fn test_parse_lsp_with_debug() {
    let cli = Cli::parse_from(["rice", "lsp", "--debug"]);
    if let Some(Commands::Lsp { debug, .. }) = cli.command {
        assert!(debug);
    } else {
        panic!("Expected Lsp command");
    }
}

#[test]
fn test_parse_help_command() {
    let cli = Cli::parse_from(["rice", "help"]);
    assert!(matches!(cli.command, Some(Commands::Help { .. })));
}

#[test]
fn test_parse_help_with_topic() {
    let cli = Cli::parse_from(["rice", "help", "chat"]);
    if let Some(Commands::Help { topic }) = cli.command {
        assert_eq!(topic, Some("chat".to_string()));
    } else {
        panic!("Expected Help command");
    }
}

#[test]
fn test_global_verbose_flag() {
    let cli = Cli::parse_from(["rice", "--verbose", "chat"]);
    assert!(cli.verbose);
}

#[test]
fn test_global_quiet_flag() {
    let cli = Cli::parse_from(["rice", "--quiet", "chat"]);
    assert!(cli.quiet);
}

#[test]
fn test_global_dry_run_flag() {
    let cli = Cli::parse_from(["rice", "--dry-run", "chat"]);
    assert!(cli.dry_run);
}

#[test]
fn test_no_command_defaults_to_none() {
    let cli = Cli::parse_from(["rice"]);
    // Without a command, it should default to None (which triggers TUI in router)
    assert!(cli.command.is_none());
}
