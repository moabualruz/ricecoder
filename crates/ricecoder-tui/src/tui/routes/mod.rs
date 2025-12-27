//! Route-specific UI components
//!
//! This module contains UI components organized by route/view.

pub mod home;
pub mod session;

// Home route exports
pub use home::{Home, HomeState, HomeTheme, HomeView, McpStatus};

// Session route exports - dialogs (commented out - dialog modules removed during cleanup)
// pub use session::{
//     DialogFork, DialogMessage, DialogSubagent, DialogTimeline, MessageAction, SubagentAction,
// };

// Session route exports - components
pub use session::{
    ItemStatus, KeybindHint, SessionFooter, SessionFooterTheme, SessionHeader, SessionHeaderTheme,
    SessionSidebar, SidebarItem, SidebarSection, SidebarTheme,
};
