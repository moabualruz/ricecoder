//! RiceCoder Terminal User Interface (TUI)
//!
//! This crate provides a beautiful, responsive terminal user interface for RiceCoder
//! with support for chat interface, code diffing, theming, and interactive components.

pub mod app;
pub mod event;
pub mod render;
pub mod style;
pub mod theme;
pub mod theme_loader;
pub mod widgets;
pub mod config;
pub mod layout;
pub mod markdown;
pub mod input;
pub mod prompt;
pub mod diff;
pub mod components;
pub mod integration;
pub mod command_blocks;
pub mod clipboard;
pub mod sessions;
pub mod session_manager;
pub mod session_integration;
pub mod accessibility;
pub mod performance;
pub mod provider_integration;

// Re-export commonly used types
pub use app::{App, AppMode};
pub use config::TuiConfig;
pub use style::{Theme, ColorSupport};
pub use theme::ThemeManager;
pub use theme_loader::{ThemeLoader, ThemeYaml};
pub use layout::{Layout, Rect, Constraint};
pub use widgets::{ChatWidget, Message, MessageAuthor};
pub use markdown::{MarkdownParser, MarkdownElement};
pub use input::{Intent, InputAnalyzer, ChatInputWidget};
pub use prompt::{PromptWidget, ContextIndicators, PromptConfig};
pub use diff::{DiffWidget, DiffLine, DiffHunk, DiffViewType, DiffLineType};
pub use components::{MenuWidget, ListWidget, DialogWidget, SplitViewWidget, TabWidget, DialogType, ModeIndicator, ModeSelectionMenu};
pub use integration::{WidgetContainer, LayoutCoordinator, WidgetIntegration, StateSynchronizer, LayoutInfo};
pub use command_blocks::{CommandBlocksWidget, CommandBlock, Command, CommandStatus};
pub use clipboard::{ClipboardManager, ClipboardError, CopyFeedback, CopyOperation};
pub use sessions::{SessionWidget, Session, SessionStatus, SessionDisplayMode};
pub use session_manager::{SessionManager, SessionData};
pub use session_integration::SessionIntegration;
pub use accessibility::{
    AccessibilityConfig, AnimationConfig, FocusIndicatorStyle, TextAlternative, ElementType,
    ScreenReaderAnnouncer, Announcement, AnnouncementPriority, KeyboardNavigationManager,
    StateChangeEvent, FocusManager,
};
pub use performance::{
    LazyLoadConfig, LazyMessageHistory, DiffRenderOptimizer, ThemeSwitchPerformance,
};
pub use provider_integration::ProviderIntegration;
