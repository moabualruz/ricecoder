//! Template engine module for code generation
//!
//! Provides template parsing, rendering, and boilerplate management.

pub mod boilerplate;
pub mod cache;
pub mod discovery;
pub mod engine;
pub mod error;
pub mod loader;
pub mod parser;
pub mod resolver;
pub mod validation;

// Re-export public API
pub use boilerplate::{BoilerplateManager, FileConflict, ScaffoldingResult};
pub use cache::{CacheStats, TemplateCache};
pub use discovery::{BoilerplateDiscovery, DiscoveryResult, TemplateDiscovery};
pub use engine::TemplateEngine;
pub use error::{BoilerplateError, TemplateError};
pub use loader::{CacheStats as LoaderCacheStats, TemplateLoader};
pub use parser::{ParsedTemplate, TemplateElement, TemplateParser};
pub use resolver::{CaseTransform, Placeholder, PlaceholderResolver};
pub use validation::ValidationEngine;
