//! Response merging from multiple sources

pub mod completion;
pub mod diagnostics;
pub mod hover;

pub use completion::CompletionMerger;
pub use diagnostics::DiagnosticsMerger;
pub use hover::HoverMerger;
