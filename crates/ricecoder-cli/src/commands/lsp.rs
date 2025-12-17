//! LSP command - Start the Language Server Protocol server

use crate::commands::Command;
use crate::error::{CliError, CliResult};
use async_trait::async_trait;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::{error, info};

/// LSP server configuration
#[derive(Debug, Clone)]
pub struct LspConfig {
    /// Log level (trace, debug, info, warn, error)
    pub log_level: String,
    /// Port for TCP transport (future support)
    pub port: Option<u16>,
    /// Enable debug mode for verbose logging
    pub debug: bool,
}

impl Default for LspConfig {
    fn default() -> Self {
        Self {
            log_level: "info".to_string(),
            port: None,
            debug: false,
        }
    }
}

/// LSP command handler
pub struct LspCommand {
    log_level: Option<String>,
    port: Option<u16>,
    debug: bool,
}

impl LspCommand {
    /// Create a new LSP command
    pub fn new(log_level: Option<String>, port: Option<u16>, debug: bool) -> Self {
        Self {
            log_level,
            port,
            debug,
        }
    }

    /// Get the LSP configuration
    pub fn get_config(&self) -> LspConfig {
        LspConfig {
            log_level: self.log_level.clone().unwrap_or_else(|| {
                if self.debug {
                    "debug".to_string()
                } else {
                    "info".to_string()
                }
            }),
            port: self.port,
            debug: self.debug,
        }
    }
}
#[async_trait::async_trait]
impl Command for LspCommand {
    async fn execute(&self) -> CliResult<()> {
        // Build LSP configuration
        let config = self.get_config();

        // Start the LSP server
        start_lsp_server(config).await
    }
}

/// Start the LSP server
async fn start_lsp_server(config: LspConfig) -> CliResult<()> {
        // Initialize logging with configured level
        init_lsp_logging(&config)?;

        info!("Starting LSP server");
        info!("Log level: {}", config.log_level);
        if let Some(port) = config.port {
            info!("TCP port: {}", port);
        }
        info!("Debug mode: {}", config.debug);

        // Create shutdown signal handler
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_clone = shutdown.clone();

        // Set up signal handlers for graceful shutdown
        let _shutdown_handle = tokio::spawn(async move {
            match tokio::signal::ctrl_c().await {
                Ok(()) => {
                    info!("Received shutdown signal (SIGINT)");
                    shutdown_clone.store(true, Ordering::SeqCst);
                }
                Err(e) => {
                    error!("Failed to listen for shutdown signal: {}", e);
                }
            }
        });

        // Import the LSP server
        use ricecoder_lsp::LspServer;

        // Create and run the LSP server
        let mut server = LspServer::new();

        info!("LSP server initialized");
        info!("Listening on stdio transport");

        // Run the server
        match server.run().await {
            Ok(()) => {
                info!("LSP server shut down gracefully");
                Ok(())
            }
            Err(e) => {
                error!("LSP server error: {}", e);
                Err(CliError::Internal(format!("LSP server error: {}", e)))
            }
        }
}

/// Initialize logging for LSP server
fn init_lsp_logging(config: &LspConfig) -> CliResult<()> {
    use tracing_subscriber::fmt;

    // Parse log level
    let level = match config.log_level.to_lowercase().as_str() {
        "trace" => tracing::Level::TRACE,
        "debug" => tracing::Level::DEBUG,
        "info" => tracing::Level::INFO,
        "warn" => tracing::Level::WARN,
        "error" => tracing::Level::ERROR,
        _ => tracing::Level::INFO,
    };

    // Initialize tracing subscriber with the specified level
    fmt()
        .with_max_level(level)
        .with_target(config.debug)
        .with_thread_ids(config.debug)
        .with_file(config.debug)
        .with_line_number(config.debug)
        .with_writer(std::io::stderr)
        .init();

    Ok(())
}


