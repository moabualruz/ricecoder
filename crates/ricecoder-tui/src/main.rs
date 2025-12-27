//! RiceCoder TUI - Terminal User Interface entry point
//!
//! Modern TUI implementation using route-based architecture and AppContext backend wiring.

use anyhow::Result;
use ricecoder_storage::DefaultsManager;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize storage defaults (folder structure, default configs)
    // This is idempotent - won't overwrite existing user files
    if let Ok(defaults_manager) = DefaultsManager::with_default_path() {
        if let Err(e) = defaults_manager.initialize() {
            eprintln!("Warning: Failed to initialize defaults: {}", e);
            // Continue anyway - this is not fatal
        }
    }

    // Initialize tracing for logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("Starting RiceCoder TUI...");

    // Create and run the TUI application
    let mut app = ricecoder_tui::tui::TuiApp::new()?;
    
    // Run the event loop
    match app.run().await {
        Ok(_) => {
            tracing::info!("TUI exited gracefully");
            Ok(())
        }
        Err(e) => {
            tracing::error!("TUI error: {}", e);
            Err(e)
        }
    }
}
