//! RiceCoder Terminal User Interface (TUI)
//!
//! This crate provides a beautiful, responsive terminal user interface for RiceCoder
//! with support for chat interface, code diffing, theming, and interactive components.

pub mod accessibility;
pub mod app;
pub mod banner;
pub mod clipboard;
pub mod code_editor_widget;
pub mod command_blocks;
pub mod command_palette;
pub mod components;
pub mod executor;
pub mod file_picker;
pub mod config;
pub mod diff;
pub mod event;
pub mod image_integration;
pub mod image_widget;
pub mod input;
pub mod integration;
pub mod layout;
pub mod logger_widget;
pub mod markdown;
pub mod performance;
pub mod popup_widget;
pub mod prompt;
pub mod prompt_context;
pub mod provider_integration;
pub mod render;
pub mod scrollview_widget;
pub mod session_integration;
pub mod status_bar;
pub mod session_manager;
pub mod sessions;
pub mod style;
pub mod terminal_state;
pub mod textarea_widget;
pub mod theme;
pub mod theme_loader;
pub mod theme_registry;
pub mod theme_reset;
pub mod tree_widget;
pub mod widgets;

// Re-export commonly used types
pub use accessibility::{
    AccessibilityConfig, AnimationConfig, Announcement, AnnouncementPriority, ElementType,
    FocusIndicatorStyle, FocusManager, KeyboardNavigationManager, ScreenReaderAnnouncer,
    StateChangeEvent, TextAlternative,
};
pub use app::{App, AppMode};
pub use banner::{BannerArea, BannerComponent, BannerComponentConfig};
pub use clipboard::{ClipboardError, ClipboardManager, CopyFeedback, CopyOperation};
pub use code_editor_widget::{CodeEditorWidget, CodeLine, Language, SyntaxTheme};
pub use command_blocks::{Command, CommandBlock, CommandBlocksWidget, CommandStatus};
pub use command_palette::{CommandPaletteWidget, PaletteCommand};
pub use executor::{
    CommandContext, CommandDefinition, CommandError, CommandExecutionResult, CommandExecutor,
    CommandParameter, CommandRegistry, CommandResult, ExecutionStatus, ParameterPromptHandler,
    ParameterType, ParameterValidation, validate_parameter,
};
pub use file_picker::FilePickerWidget;
pub use components::{
    DialogType, DialogWidget, ListWidget, MenuWidget, ModeIndicator, ModeSelectionMenu,
    SplitViewWidget, TabWidget,
};
pub use config::TuiConfig;
pub use diff::{DiffHunk, DiffLine, DiffLineType, DiffViewType, DiffWidget};
pub use image_integration::ImageIntegration;
pub use image_widget::{ImageFormat, ImageWidget, RenderMode};
pub use input::{ChatInputWidget, InputAnalyzer, Intent};
pub use integration::{
    LayoutCoordinator, LayoutInfo, StateSynchronizer, WidgetContainer, WidgetIntegration,
};
pub use layout::{Constraint, Layout, Rect};
pub use logger_widget::{LogEntry, LogLevel, LoggerWidget};
pub use markdown::{MarkdownElement, MarkdownParser};
pub use performance::{
    DiffRenderOptimizer, LazyLoadConfig, LazyMessageHistory, ThemeSwitchPerformance,
};
pub use popup_widget::{PopupButton, PopupType, PopupWidget};
pub use prompt::{ContextIndicators, PromptConfig, PromptWidget};
pub use prompt_context::PromptContext;
pub use provider_integration::ProviderIntegration;
pub use scrollview_widget::ScrollViewWidget;
pub use session_integration::SessionIntegration;
pub use status_bar::{ConnectionStatus, InputMode, StatusBarWidget, TokenUsage};
pub use session_manager::{SessionData, SessionManager};
pub use sessions::{Session, SessionDisplayMode, SessionStatus, SessionWidget};
pub use style::{ColorSupport, Theme};
pub use terminal_state::{ColorSupport as TerminalColorSupport, TerminalCapabilities, TerminalState, TerminalType};
pub use textarea_widget::TextAreaWidget;
pub use theme::ThemeManager;
pub use theme_loader::{ThemeLoader, ThemeYaml};
pub use theme_registry::ThemeRegistry;
pub use theme_reset::ThemeResetManager;
pub use tree_widget::{TreeNode, TreeWidget};
pub use widgets::{ChatWidget, Message, MessageAuthor};
