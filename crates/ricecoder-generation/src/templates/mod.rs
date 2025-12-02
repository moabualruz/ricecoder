//! Template engine module for code generation
//!
//! Provides template parsing, rendering, and boilerplate management.

pub mod error;
pub mod parser;
pub mod validation;
pub mod resolver;
pub mod engine;
pub mod cache;
pub mod loader;
pub mod discovery;
pub mod boilerplate;

// Re-export public API
pub use error::{TemplateError, BoilerplateError};
pub use parser::{TemplateParser, ParsedTemplate, TemplateElement};
pub use validation::ValidationEngine;
pub use resolver::{PlaceholderResolver, Placeholder, CaseTransform};
pub use engine::TemplateEngine;
pub use cache::{TemplateCache, CacheStats};
pub use loader::{TemplateLoader, CacheStats as LoaderCacheStats};
pub use discovery::{TemplateDiscovery, DiscoveryResult, BoilerplateDiscovery};
pub use boilerplate::{BoilerplateManager, ScaffoldingResult, FileConflict};
