//! CLI argument parsing for configuration overrides
//!
//! This module provides CLI argument definitions that can override configuration
//! values with the highest priority.

use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// CLI arguments that can override configuration values
#[derive(Parser, Debug, Clone, Serialize, Deserialize, Default)]
#[command(name = "ricecoder")]
#[command(about = "AI-powered development tool")]
pub struct CliArgs {
    /// Configuration file path (overrides default search paths)
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Default AI provider
    #[arg(long, value_name = "PROVIDER")]
    pub provider: Option<String>,

    /// Default AI model
    #[arg(long, value_name = "MODEL")]
    pub model: Option<String>,

    /// API key for the provider
    #[arg(long, value_name = "KEY")]
    pub api_key: Option<String>,

    /// Temperature for AI responses (0.0 to 2.0)
    #[arg(long, value_name = "TEMP")]
    pub temperature: Option<f64>,

    /// Maximum tokens for AI responses
    #[arg(long, value_name = "TOKENS")]
    pub max_tokens: Option<u32>,

    /// Theme name
    #[arg(long, value_name = "THEME")]
    pub theme: Option<String>,

    /// Enable verbose logging
    #[arg(long)]
    pub verbose: bool,

    /// Log level
    #[arg(long, value_name = "LEVEL")]
    pub log_level: Option<String>,

    /// Disable telemetry
    #[arg(long)]
    pub no_telemetry: bool,

    /// Project directory (defaults to current directory)
    #[arg(long, value_name = "DIR")]
    pub project_dir: Option<PathBuf>,

    /// User config directory
    #[arg(long, value_name = "DIR")]
    pub user_config_dir: Option<PathBuf>,

    /// Skip configuration validation
    #[arg(long)]
    pub skip_validation: bool,

    /// Enable experimental features
    #[arg(long)]
    pub experimental: bool,
}

impl CliArgs {
    /// Parse CLI arguments
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }

    /// Check if any configuration overrides are provided
    pub fn has_overrides(&self) -> bool {
        self.provider.is_some()
            || self.model.is_some()
            || self.api_key.is_some()
            || self.temperature.is_some()
            || self.max_tokens.is_some()
            || self.theme.is_some()
            || self.log_level.is_some()
            || self.no_telemetry
            || self.experimental
    }
}
