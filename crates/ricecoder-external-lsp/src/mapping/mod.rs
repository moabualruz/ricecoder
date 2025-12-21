//! Output mapping and transformation

pub mod completion;
pub mod diagnostics;
pub mod hover;
pub mod json_path;
pub mod transformer;

pub use completion::CompletionMapper;
pub use diagnostics::DiagnosticsMapper;
pub use hover::HoverMapper;
pub use json_path::JsonPathParser;
pub use transformer::OutputTransformer;
