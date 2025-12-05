//! Output mapping and transformation

pub mod transformer;
pub mod json_path;
pub mod completion;
pub mod diagnostics;
pub mod hover;

pub use transformer::OutputTransformer;
pub use json_path::JsonPathParser;
pub use completion::CompletionMapper;
pub use diagnostics::DiagnosticsMapper;
pub use hover::HoverMapper;
