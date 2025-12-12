//! RiceCoder Terminal User Interface (TUI) - Pure UI Layer
//!
//! This crate provides a beautiful, responsive terminal user interface for RiceCoder
//! built with [ratatui](https://github.com/ratatui-org/ratatui). **Important**: This crate
//! contains only UI components and has been architecturally separated from business logic.
//!
//! ## Architecture
//!
//! After the TUI isolation refactoring:
//! - ✅ **UI Components**: Widgets, layouts, themes, input handling
//! - ✅ **Terminal Management**: Rendering, accessibility, cross-platform support
//! - ❌ **Business Logic**: Session management moved to `ricecoder-sessions`
//! - ❌ **AI Integration**: Provider logic moved to `ricecoder-providers`
//! - ❌ **LSP Features**: Language support moved to `ricecoder-lsp`
//!
//! ## Dependencies
//!
//! `ricecoder-tui` only depends on infrastructure crates and has no business logic dependencies.
//! Business logic is injected through interfaces or dependency injection patterns.

pub mod accessibility;
pub mod app;
pub mod banner;
pub mod model;
pub mod update;
pub mod view;
pub mod clipboard;
pub mod code_editor_widget;
pub mod command_blocks;
pub mod command_palette;
pub mod components;

pub mod diff;
pub mod error;
pub mod error_handling;
pub mod event;
pub mod event_dispatcher;
pub mod executor;
pub mod file_picker;
pub mod image_integration;
pub mod image_widget;
pub mod input;
pub mod integration;
pub mod layout;
pub mod logger_widget;
pub mod markdown;
pub mod performance;
pub mod popup_widget;
pub mod progressive_enhancement;
pub mod prompt;
pub mod prompt_context;
pub mod project_bootstrap;

pub mod reactive_ui_updates;
pub mod real_time_updates;
pub mod render;
pub mod render_pipeline;
pub mod scrollview_widget;
pub mod session_integration;
pub mod session_manager;
pub mod sessions;
pub mod status_bar;
pub mod style;
pub mod tea;
pub mod terminal_state;
pub mod textarea_widget;
pub mod theme;
pub mod theme_loader;
pub mod theme_registry;
pub mod theme_reset;
pub mod tree_widget;
pub mod widgets;
pub mod plugins;
pub mod monitoring;

// Re-export commonly used types
pub use accessibility::{
    AccessibilityConfig, AnimationConfig, Announcement, AnnouncementPriority, ElementType,
    FocusIndicatorStyle, FocusManager, KeyboardNavigationManager, ScreenReaderAnnouncer,
    StateChangeEvent, TextAlternative, EnhancedKeyboardNavigation, HighContrastThemeManager,
    KeyboardShortcutCustomizer,
};
pub use app::App;
pub use banner::{BannerArea, BannerComponent, BannerComponentConfig};
pub use clipboard::{ClipboardError, ClipboardManager, CopyFeedback, CopyOperation};
pub use code_editor_widget::{CodeEditorWidget, CodeLine, Language, SyntaxTheme};
pub use command_blocks::{Command, CommandBlock, CommandBlocksWidget, CommandStatus};
// LSP integration moved to ricecoder-lsp crate
// pub use ricecoder_lsp::tui_integration::{
//     DiagnosticDetailWidget, DiagnosticItem, DiagnosticSeverity, DiagnosticsWidget, HoverWidget,
// };
pub use error::{
    KeybindError, StorageError, ToolError, TuiError,
    TuiResult,
};
// Provider and session errors moved to respective crates
// pub use error_handling::{
//     ErrorBoundary, ErrorCategory, ErrorLogger, ErrorManager, ErrorSeverity, RecoveryStrategy,
//     RiceError, RetryMechanism, CrashRecovery, CrashReport, LogEntry as ErrorLogEntry, LogLevel as ErrorLogLevel,
// };
// LSP integration moved to ricecoder-lsp crate
// pub use ricecoder_lsp::tui_integration::{language_from_file_path, lsp_diagnostics_to_tui, lsp_hover_to_text};
pub use plugins::{
    Plugin, PluginContext, PluginId, PluginManager, PluginMessage, PluginMetadata, PluginRegistry,
    PluginSandbox, PluginState,
};
pub use monitoring::{
    MonitoringSystem, PerformanceMonitor, UsageAnalytics, MetricsCollector,
    PerformanceProfiler, UserExperienceMetrics, MonitoringReport, PerformanceReport,
    AnalyticsReport, UserExperienceReport, AnonymousStatistics,
};
pub use command_palette::{CommandPaletteWidget, PaletteCommand};
pub use executor::{
    CommandContext, CommandDefinition, CommandError, CommandExecutionResult, CommandExecutor,
    CommandParameter, CommandRegistry, CommandResult, ExecutionStatus, ParameterPromptHandler,
    ParameterType, ParameterValidation, validate_parameter,
};
pub use file_picker::FilePickerWidget;
pub use components::{
    Component, ComponentId, ComponentRegistry, ComponentEvent, FocusDirection, FocusResult,
    DialogType, DialogWidget, ListWidget, MenuWidget, ModeIndicator, ModeSelectionMenu,
    SplitViewWidget, TabWidget,
};
// TuiConfig is now exported from ricecoder-storage
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
pub use progressive_enhancement::{
    ProgressiveEnhancement, FeatureLevel, FeatureToggles, RenderingStrategy,
};
pub use reactive_ui_updates::{
    ReactiveRenderer, LiveDataSynchronizer, ReactiveUICoordinator, LiveDataEvent,
    FileChangeEvent, FileChangeType, SessionSyncEvent, SessionChangeType,
    ConflictResolution, ConflictInfo, ConflictType, UpdateType, UpdatePriority,
};
pub use real_time_updates::{
    RealTimeUpdates, RealTimeStream, StreamData, StreamType, OperationStatus,
    OperationInfo, ProgressIndicator, RealTimeStats,
};
pub use prompt::{ContextIndicators, PromptConfig, PromptWidget};
pub use prompt_context::PromptContext;
// ProviderIntegration is now exported from ricecoder-providers
pub use scrollview_widget::ScrollViewWidget;
pub use session_integration::SessionIntegration;
pub use status_bar::{ConnectionStatus, InputMode, StatusBarWidget};
pub use session_manager::{SessionData, SessionManager};
pub use model::{AppModel, AppMessage, AppMode, CommandResult as TeaCommandResult, OperationId, StateDiff, StateChange, SessionState, CommandState, UiState, PendingOperation, Subscription};
pub use update::Command as TeaCommand;
pub use view::view;
pub use render_pipeline::{VirtualRenderer, VirtualList, VirtualScroll, LazyLoader, RenderBatch, RenderOperation, RenderPriority, VirtualNode, ComponentType};
pub use event::{event_to_message};
pub use event_dispatcher::{EventDispatcher, EventEnvelope, EventPriority, EventResult, EventSource, EventBatch, BatchType, EventStats, OptimisticUpdater, OptimisticUpdate, LoadingManager, LoadingState};
pub use sessions::{Session, SessionDisplayMode, SessionStatus, SessionWidget};
pub use style::{ColorSupport, Theme};
pub use terminal_state::{ColorSupport as TerminalColorSupport, TerminalCapabilities, TerminalState, TerminalType};
pub use textarea_widget::TextAreaWidget;
pub use theme::ThemeManager;
// VCS integration moved to ricecoder-vcs crate
// pub use ricecoder_vcs::tui_integration::{VcsIntegration, VcsStatus};
// VCS integration moved to ricecoder-vcs crate
// pub use status_bar::StatusBarVcsExt;
pub use theme_loader::{ThemeLoader, ThemeYaml};
pub use theme_registry::ThemeRegistry;
pub use theme_reset::ThemeResetManager;
pub use tree_widget::{TreeNode, TreeWidget};
pub use widgets::{ChatWidget, Message, MessageAuthor};
pub use project_bootstrap::{ProjectBootstrap, BootstrapResult, ProjectInfo};
