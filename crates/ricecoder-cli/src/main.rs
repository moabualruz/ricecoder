// RiceCoder CLI Entry Point

use std::path::Path;

use ricecoder_cli::{lifecycle, output, router::CommandRouter};
use ricecoder_storage::DefaultsManager;


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
    let result = CommandRouter::route().await;

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

/// Initialize first-run setup using DefaultsManager
///
/// This function:
/// 1. Creates a DefaultsManager for the global storage path
/// 2. Initializes the folder structure and default files if needed
/// 3. Does NOT overwrite existing user files
fn initialize_first_run() -> Result<(), Box<dyn std::error::Error>> {
    // Create defaults manager with the default global path
    let defaults_manager = DefaultsManager::with_default_path()?;

    // Initialize folder structure and default files
    // This is idempotent - it won't overwrite existing files
    defaults_manager.initialize()?;

    Ok(())
}
