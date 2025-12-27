//! Session route components
//!
//! This module contains UI components for the session route,
//! including dialogs, header, footer, sidebar, and the main session view.

// Dialogs temporarily commented out - need menu stub improvements
// pub mod dialog_fork;
// pub mod dialog_message;
// pub mod dialog_subagent;
// pub mod dialog_timeline;
pub mod footer;
pub mod header;
pub mod sidebar;

// Dialogs (temporarily commented)
// pub use dialog_fork::DialogFork;
// pub use dialog_message::{DialogMessage, MessageAction};
// pub use dialog_subagent::{DialogSubagent, SubagentAction};
// pub use dialog_timeline::DialogTimeline;

// Session components
pub use footer::{KeybindHint, SessionFooter, SessionFooterTheme};
pub use header::{SessionHeader, SessionHeaderTheme};
pub use sidebar::{ItemStatus, SessionSidebar, SidebarItem, SidebarSection, SidebarTheme};
