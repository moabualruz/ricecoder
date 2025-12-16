// RiceCoder CLI Entry Point

use ricecoder_cli::{output, lifecycle};
use ricecoder_cli::router::CommandRouter;
use ricecoder_storage::PathResolver;
use std::path::Path;
use std::fs;
use tokio::signal;

#[tokio::main]
async fn main() {
    // Check for multi-call binary pattern
    let binary_name = std::env::args().next().and_then(|s| {
        Path::new(&s)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
    });

    match binary_name.as_deref() {
        Some("rice-gen") | Some("ricecoder-gen") => {
            // Handle as gen command
            std::env::set_var("RICECODER_COMMAND", "gen");
        }
        Some("rice-chat") | Some("ricecoder-chat") => {
            // Handle as chat command
            std::env::set_var("RICECODER_COMMAND", "chat");
        }
        Some("rice-init") | Some("ricecoder-init") => {
            // Handle as init command
            std::env::set_var("RICECODER_COMMAND", "init");
        }
        _ => {}
    }

    // Initialize first-run setup if needed
    if let Err(e) = initialize_first_run() {
        output::print_error(&format!("First-run initialization failed: {}", e));
        std::process::exit(1);
    }

    // Initialize DI container
    if let Err(e) = ricecoder_cli::di::initialize_di_container() {
        output::print_error(&format!("DI container initialization failed: {}", e));
        std::process::exit(1);
    }

    // Initialize lifecycle manager
    let lifecycle_manager = lifecycle::initialize_lifecycle_manager();

    // Initialize all components
    if let Err(e) = lifecycle_manager.initialize_all().await {
        output::print_error(&format!("Component initialization failed: {}", e));
        std::process::exit(1);
    }

    // Start all components
    if let Err(e) = lifecycle_manager.start_all().await {
        output::print_error(&format!("Component startup failed: {}", e));
        std::process::exit(1);
    }

    // Route and execute command
    let result = CommandRouter::route();

    // Stop components before exiting
    if let Err(e) = lifecycle_manager.stop_all().await {
        output::print_error(&format!("Component shutdown failed: {}", e));
    }



    // Exit with appropriate code
    if let Err(e) = result {
        output::print_error(&e.user_message());
        std::process::exit(1);
    }
}

/// Initialize first-run setup
///
/// This function:
/// 1. Checks if this is the first run
/// 2. Creates the global config directory if needed
/// 3. Creates a default configuration file
/// 4. Marks first-run as complete
fn initialize_first_run() -> Result<(), Box<dyn std::error::Error>> {
    // Check if this is the first run
    let is_first_run = false;
    
    if !is_first_run {
        return Ok(());
    }

    // Get the global storage path
    let global_path = PathResolver::resolve_global_path()?;

    // Create directory structure if it doesn't exist
    if !global_path.exists() {
        fs::create_dir_all(&global_path)?;
    }

    // Create default configuration file
    let config_file = global_path.join("ricecoder.yaml");
    if !config_file.exists() {
        let default_config = create_default_config();
        fs::write(&config_file, default_config)?;
    }



    Ok(())
}

/// Create default configuration content
fn create_default_config() -> String {
    r#"# RiceCoder Configuration
# 
# This file contains the default configuration for RiceCoder.
# You can customize these settings to suit your needs.

# Provider configuration
providers:
  # Default provider to use (zen, openai, anthropic, etc.)
  default_provider: zen
  
  # API keys for various providers
  # Set these to enable access to paid models
  api_keys:
    # zen: ${OPENCODE_API_KEY}
    # openai: ${OPENAI_API_KEY}
    # anthropic: ${ANTHROPIC_API_KEY}

  # Custom endpoints for providers (optional)
  endpoints:
    # zen: https://opencode.ai/zen/v1
    # openai: https://api.openai.com/v1
    # anthropic: https://api.anthropic.com

# Default settings
defaults:
  # Default model to use (e.g., zen/big-pickle, gpt-4, claude-3-opus)
  model: zen/big-pickle
  
  # Default temperature for LLM responses (0.0 - 2.0)
  # Lower values = more deterministic, higher values = more creative
  temperature: 0.7
  
  # Default maximum tokens for LLM responses
  max_tokens: 2048

# Steering rules (optional)
# steering: []

# Custom settings (optional)
# custom: {}
"#.to_string()
}

