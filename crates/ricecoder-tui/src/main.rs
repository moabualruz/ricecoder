//! RiceCoder TUI - Terminal User Interface entry point

use anyhow::Result;
use ricecoder_tui::App;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create and run the application
    let mut app = App::new()?;
    app.run().await?;

    Ok(())
}
