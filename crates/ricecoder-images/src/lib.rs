//! Image support for ricecoder with drag-and-drop, analysis, caching, and terminal display.
//!
//! This crate provides centralized image handling including:
//! - Format validation (PNG, JPG, GIF, WebP)
//! - Image analysis via AI providers
//! - Smart caching with LRU eviction
//! - Terminal display with ASCII fallback
//! - Multi-image support

pub mod error;
pub mod config;
pub mod formats;
pub mod models;
pub mod handler;
pub mod cache;
pub mod analyzer;
pub mod display;
pub mod provider_integration;
pub mod token_counting;
pub mod audit_logging;
pub mod session_integration;
pub mod session_manager;

pub use error::{ImageError, ImageResult};
pub use config::{ImageConfig, DisplayConfig};
pub use formats::ImageFormat;
pub use models::{ImageMetadata, ImageAnalysisResult, ImageCacheEntry};
pub use handler::ImageHandler;
pub use cache::ImageCache;
pub use analyzer::{ImageAnalyzer, AnalysisRetryContext};
pub use display::ImageDisplay;
pub use provider_integration::{
    ImageData, ChatRequestWithImages, ProviderImageFormat, ImageAuditLogEntry,
};
pub use token_counting::ImageTokenCounter;
pub use audit_logging::ImageAuditLogger;
pub use session_integration::{
    MessageImageMetadata, MessageImages, SessionImageContext,
};
pub use session_manager::{SessionImageManager, MultiSessionImageManager};
