//! RiceCoder TUI - Terminal User Interface entry point

use anyhow::Result;
use ricecoder_tui::{App, TerminalState};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Capture terminal state before TUI initialization
    // Requirements: 4.1, 10.1 - Detect capabilities and capture terminal state before TUI initialization
    let mut terminal_state = TerminalState::capture()?;
    
    // Log detected capabilities for debugging and adaptation
    // Requirements: 4.1 - Log detected capabilities via ricecoder-logging
    let caps = terminal_state.capabilities();
    tracing::info!(
        "Terminal capabilities detected - Type: {:?}, Colors: {:?}, Mouse: {}, Sixel: {}, Unicode: {}, SSH: {}, TMUX: {}, Size: {}x{}",
        caps.terminal_type,
        caps.color_support,
        caps.mouse_support,
        caps.sixel_support,
        caps.unicode_support,
        caps.is_ssh,
        caps.is_tmux,
        caps.size.0,
        caps.size.1
    );

    // Adapt behavior based on capabilities
    // Requirements: 4.2, 4.3 - Adapt UI based on detected capabilities and handle SSH limitations
    if caps.should_reduce_graphics() {
        tracing::info!("SSH session detected - reducing graphics complexity");
    }
    
    if caps.should_wrap_osc52() {
        tracing::info!("TMUX session detected - will wrap OSC 52 sequences for clipboard");
    }

    // Create a flag to signal graceful shutdown
    // Requirements: 10.1 - Install signal handler for Ctrl+C
    let shutdown_flag = Arc::new(AtomicBool::new(false));
    let shutdown_flag_clone = shutdown_flag.clone();

    // Install Ctrl+C handler
    // Requirements: 10.1 - Install signal handler in ricecoder-tui/src/app.rs (or main.rs)
    ctrlc::set_handler(move || {
        tracing::info!("Ctrl+C received, initiating graceful shutdown");
        shutdown_flag_clone.store(true, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl+C handler");

    // Create and run the application
    let mut app = App::new()?;

    // Run the application with graceful shutdown support
    let result = run_with_shutdown(&mut app, &shutdown_flag, terminal_state.capabilities()).await;

    // Restore terminal state on exit (normal, Ctrl+C, or error)
    // Requirements: 10.2, 10.3 - Restore terminal on normal exit, Ctrl+C, and error exit
    if let Err(e) = terminal_state.restore() {
        tracing::error!("Failed to restore terminal state: {}", e);
    }

    result
}

/// Run the application with graceful shutdown support
///
/// This function runs the application and checks for shutdown signals.
/// When a shutdown signal is received (Ctrl+C), it sets the app's should_exit flag.
///
/// Requirements: 4.2, 10.1 - Adapt UI based on capabilities and graceful shutdown on Ctrl+C
async fn run_with_shutdown(app: &mut App, shutdown_flag: &Arc<AtomicBool>, capabilities: &ricecoder_tui::TerminalCapabilities) -> Result<()> {
    use crossterm::{
        execute,
        event::{EnableMouseCapture, DisableMouseCapture},
        terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    };
    use ratatui::backend::CrosstermBackend;
    use ratatui::Terminal;
    use std::io;

    // Set up terminal
    let mut stdout = io::stdout();
    
    // Enable mouse capture only if supported
    // Requirements: 4.2 - Enable/disable mouse support based on capabilities
    if capabilities.mouse_support {
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        tracing::debug!("Mouse capture enabled");
    } else {
        execute!(stdout, EnterAlternateScreen)?;
        tracing::debug!("Mouse capture disabled - not supported");
    }
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // Main event loop with shutdown check
    let result = async {
        while !app.should_exit {
            // Check for shutdown signal
            if shutdown_flag.load(Ordering::SeqCst) {
                tracing::info!("Shutdown signal received, exiting gracefully");
                app.should_exit = true;
                break;
            }

            // Poll for events
            if let Some(event) = app.event_loop.poll().await? {
                app.handle_event(event)?;
            }

            // Render the UI using the terminal
            terminal.draw(|f| {
                app.renderer.render_frame(f, app);
            })?;
        }

        tracing::info!("RiceCoder TUI exited successfully");
        Ok::<(), anyhow::Error>(())
    }.await;

    // Clean up terminal
    // Requirements: 4.2 - Disable mouse capture only if it was enabled
    if capabilities.mouse_support {
        execute!(
            terminal.backend_mut(),
            DisableMouseCapture,
            LeaveAlternateScreen
        )?;
    } else {
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen
        )?;
    }
    terminal.show_cursor()?;

    result
}
