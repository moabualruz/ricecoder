//! RiceCoder TUI - Terminal User Interface entry point

use anyhow::Result;
use ricecoder_tui::{
    performance::RenderPerformanceTracker,
    render::Renderer,
    App,
    TerminalState,
    AppModel,
    view,
    event_to_message,
    tea::ReactiveState,
};
use ricecoder_storage::TuiConfig;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

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

    // Create initial TEA model
    let config = TuiConfig::load()?;
    let theme = ricecoder_tui::Theme::default();
    let terminal_caps = terminal_state.capabilities().clone();
    let initial_model = AppModel::init(config, theme, terminal_caps);

    // Create reactive state manager
    let reactive_state = ReactiveState::new(initial_model);

    // Create event channels for TEA
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();

    // Create cancellation token for graceful shutdown
    let cancel_token = CancellationToken::new();
    let cancel_token_clone = cancel_token.clone();

    // Install Ctrl+C handler with cancellation token
    let shutdown_flag_clone = shutdown_flag.clone();
    ctrlc::set_handler(move || {
        tracing::info!("Ctrl+C received, initiating graceful shutdown");
        shutdown_flag_clone.store(true, Ordering::SeqCst);
        cancel_token_clone.cancel();
    })
    .expect("Error setting Ctrl+C handler");

    // Start event polling task
    let event_tx_clone = event_tx.clone();
    let cancel_token_for_events = cancel_token.clone();
    tokio::spawn(async move {
        let mut event_loop = ricecoder_tui::EventLoop::new();
        loop {
            tokio::select! {
                _ = cancel_token_for_events.cancelled() => {
                    break;
                }
                event = event_loop.poll() => {
                    match event {
                        Ok(Some(evt)) => {
                            let message = event_to_message(evt);
                            if event_tx_clone.send(message).is_err() {
                                break;
                            }
                        }
                        Ok(None) => break,
                        Err(e) => {
                            tracing::error!("Event polling error: {}", e);
                            break;
                        }
                    }
                }
            }
        }
    });

    // Run TEA event loop
    let result = run_tea_event_loop(
        reactive_state,
        &mut event_rx,
        &cancel_token,
        terminal_state.capabilities(),
    ).await;

    // Restore terminal state on exit (normal, Ctrl+C, or error)
    // Requirements: 10.2, 10.3 - Restore terminal on normal exit, Ctrl+C, and error exit
    if let Err(e) = terminal_state.restore() {
        tracing::error!("Failed to restore terminal state: {}", e);
    }

    result
}

/// Run TEA-based event loop with graceful shutdown support
///
/// This function implements the Elm Architecture event loop:
/// 1. Poll for events and convert to messages
/// 2. Update model with messages
/// 3. Render view based on current model
/// 4. Handle graceful shutdown with cancellation tokens
async fn run_tea_event_loop(
    mut reactive_state: ReactiveState,
    event_rx: &mut mpsc::UnboundedReceiver<ricecoder_tui::AppMessage>,
    cancel_token: &CancellationToken,
    capabilities: &ricecoder_tui::TerminalCapabilities,
) -> Result<()> {
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

    // Initialize performance tracker for 60 FPS target
    let mut perf_tracker = RenderPerformanceTracker::new();

    // TEA Event Loop with tokio::select! for cancellation
    let result = async {
        loop {
            let frame_start = Instant::now();

            // Use tokio::select! to handle events and cancellation concurrently
            tokio::select! {
                // Handle cancellation
                _ = cancel_token.cancelled() => {
                    tracing::info!("Cancellation received, exiting gracefully");
                    break;
                }

                // Handle incoming messages
                message = event_rx.recv() => {
                    match message {
                        Some(msg) => {
                            // Update model with message
                            match reactive_state.update(msg) {
                                Ok(diff) => {
                                    tracing::debug!("Model updated with diff: {:?}", diff.changes);
                                }
                                Err(e) => {
                                    tracing::error!("Failed to update model: {}", e);
                                }
                            }
                        }
                        None => {
                            tracing::info!("Event channel closed, exiting");
                            break;
                        }
                    }
                }

                // Periodic tick for animations/updates
                _ = tokio::time::sleep(Duration::from_millis(250)) => {
                    // Send tick message
                    let tick_msg = ricecoder_tui::AppMessage::Tick;
                    if let Err(e) = reactive_state.update(tick_msg) {
                        tracing::error!("Failed to process tick: {}", e);
                    }
                }
            }

            // Render the current model state
            let current_model = reactive_state.current();
            terminal.draw(|f| {
                view(f, current_model);
            })?;

            // Record frame time for performance tracking
            let frame_time = frame_start.elapsed();
            perf_tracker.record_frame(frame_time);

            // Log performance warnings if not meeting 60 FPS target
            if !perf_tracker.is_meeting_target() && perf_tracker.frame_count % 60 == 0 {
                let metrics = perf_tracker.metrics();
                tracing::warn!(
                    "Performance warning: {:.1} FPS (target: 60 FPS), avg frame time: {:.1}ms",
                    metrics.current_fps,
                    metrics.average_frame_time_ms
                );
            }

            // Throttle to ~60 FPS to prevent excessive CPU usage
            let target_frame_time = Duration::from_millis(16); // ~60 FPS
            if frame_time < target_frame_time {
                tokio::time::sleep(target_frame_time - frame_time).await;
            }
        }

        tracing::info!("TEA event loop exited successfully");
        Ok::<(), anyhow::Error>(())
    }.await;

    // Clean up terminal
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
