//! Image support for ricecoder with drag-and-drop, analysis, caching, and terminal display.
//!
//! This crate provides centralized image handling including:
//! - Format validation (PNG, JPG, GIF, WebP)
//! - Image analysis via AI providers
//! - Smart caching with LRU eviction
//! - Terminal display with ASCII fallback
//! - Multi-image support

pub mod analyzer;
pub mod audit_logging;
pub mod banner;
pub mod cache;
pub mod config;
pub mod display;
pub mod error;
pub mod formats;
pub mod handler;
pub mod models;
pub mod provider_integration;
pub mod session_integration;
pub mod session_manager;
pub mod token_counting;

pub use analyzer::{AnalysisRetryContext, ImageAnalyzer};
pub use audit_logging::ImageAuditLogger;
pub use banner::{
    BannerCache, BannerConfig, BannerOutput, BannerRenderer, ColorDepth, TerminalCapabilities,
    ThemeColors,
};
pub use cache::ImageCache;
pub use config::{DisplayConfig, ImageConfig};
pub use display::ImageDisplay;
pub use error::{ImageError, ImageResult};
pub use formats::ImageFormat;
pub use handler::ImageHandler;
pub use models::{ImageAnalysisResult, ImageCacheEntry, ImageMetadata};
pub use provider_integration::{
    ChatRequestWithImages, ImageAuditLogEntry, ImageData, ProviderImageFormat,
};
pub use session_integration::{MessageImageMetadata, MessageImages, SessionImageContext};
pub use session_manager::{MultiSessionImageManager, SessionImageManager};
pub use token_counting::ImageTokenCounter;
