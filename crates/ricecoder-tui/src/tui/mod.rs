//! TUI module organization
//!
//! This module provides high-level organization for TUI components and widgets.

pub mod border;
pub mod context;
pub mod did_you_know;
pub mod prompt;
pub mod routes;
pub mod todo_item;

pub use border::{SplitBorder, EMPTY_BORDER};
pub use context::{
    Args, ArgsProvider, LazyProvider, LocalProvider, PromptRef, PromptRefProvider, SdkProvider,
    SimpleProvider,
};
pub use did_you_know::DidYouKnow;
pub use routes::{DialogSubagent, Home, HomeState, HomeTheme, HomeView, McpStatus, SubagentAction};
pub use todo_item::{TodoItem, TodoStatus};
