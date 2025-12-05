//! Hook configuration loading and management
//!
//! This module provides configuration loading for hooks, supporting a hierarchy
//! of configuration sources: Runtime → Project → User → Built-in → Fallback.
//!
//! Configuration files are stored in YAML format and loaded using the
//! ricecoder-storage PathResolver for cross-platform compatibility.

pub mod loader;
pub mod reloader;
pub mod templates;
pub mod validator;

pub use loader::ConfigLoader;
pub use reloader::ConfigReloader;
pub use templates::{HookTemplate, TemplateManager, TemplateParameter};
pub use validator::ConfigValidator;
