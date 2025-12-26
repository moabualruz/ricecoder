//! RiceCoder Terminal User Interface (TUI) - Pure UI Layer
#![forbid(unsafe_code)]

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
pub mod clipboard;
pub mod code_editor_widget;
pub mod command_blocks;
pub mod command_palette;
pub mod components;
pub mod di;
pub mod lifecycle;
pub mod model;
pub mod tui;
pub mod update;
pub mod view;

pub mod diff;
pub mod error;
pub mod error_handling;
pub mod event;
pub mod event_dispatcher;
pub mod providers;
// executor moved to ricecoder-commands
// help moved to ricecoder-help
// keybinds moved to ricecoder-keybinds
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
pub mod project_bootstrap;
pub mod prompt;
pub mod prompt_context;

pub mod reactive_ui_updates;
pub mod real_time_updates;
pub mod render;
pub mod render_pipeline;
pub mod scrollview_widget;
// Session modules moved to ricecoder-sessions crate
pub mod status_bar;
pub mod style;
pub mod tea;
pub mod terminal_state;
pub mod textarea_widget;
pub mod theme;
// Theme modules moved to ricecoder-themes crate
// pub mod theme;
// pub mod theme_loader;
// pub mod theme_registry;
// pub mod theme_reset;
pub mod monitoring;
pub mod plugins;
pub mod tree_widget;
pub mod ui_components;
pub mod widgets;

// Re-export commonly used types
pub use accessibility::{
    AccessibilityConfig, AnimationConfig, Announcement, AnnouncementPriority, ElementType,
    EnhancedKeyboardNavigation, FocusIndicatorStyle, FocusManager, HighContrastThemeManager,
    KeyboardNavigationManager, KeyboardShortcutCustomizer, ScreenReaderAnnouncer, TextAlternative,
};
pub use app::App;
pub use banner::{BannerArea, BannerComponent, BannerComponentConfig};
pub use clipboard::{ClipboardError, ClipboardManager, CopyFeedback, CopyOperation};
pub use code_editor_widget::{CodeEditorWidget, CodeLine, Language, SyntaxTheme};
pub use command_blocks::{Command, CommandBlock, CommandBlocksWidget, CommandStatus};
// Provider and session errors moved to respective crates
// pub use error_handling::{
//     ErrorBoundary, ErrorCategory, ErrorLogger, ErrorManager, ErrorSeverity, RecoveryStrategy,
//     RiceError, RetryMechanism, CrashRecovery, CrashReport, LogEntry as ErrorLogEntry, LogLevel as ErrorLogLevel,
// };
// LSP integration moved to ricecoder-lsp crate
// pub use ricecoder_lsp::tui_integration::{language_from_file_path, lsp_diagnostics_to_tui, lsp_hover_to_text};
pub use command_palette::{CommandPaletteWidget, PaletteCommand};
// executor exports moved to ricecoder-commands
pub use components::{
    Component,
    ComponentEvent as ComponentLifecycleEvent,
    ComponentId,
    ComponentRegistry,
    CustomEvent,
    DialogType,
    DialogWidget,
    // Event system
    EventComponent,
    EventContext,
    EventPhase,
    EventPropagation,
    EventResult,
    FocusDirection,
    FocusEvent,
    FocusResult,
    InputArea,
    InputEvent,
    KeyboardEvent,
    // ListWidget,
    MenuWidget,
    ModeIndicator,
    ModeSelectionMenu,
    MouseEvent,
    SplitViewWidget,
    StateChangeEvent,
    TabWidget,
};
// EventDispatcher is from event_dispatcher module, not components
pub use event_dispatcher::EventDispatcher;
// TuiConfig is now exported from ricecoder-storage
pub use diff::{DiffHunk, DiffLine, DiffLineType, DiffViewType, DiffWidget};
// LSP integration moved to ricecoder-lsp crate
// pub use ricecoder_lsp::tui_integration::{
//     DiagnosticDetailWidget, DiagnosticItem, DiagnosticSeverity, DiagnosticsWidget, HoverWidget,
// };
pub use error::{KeybindError, StorageError, ToolError, TuiError, TuiResult};
pub use event::{
    EventLoop, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent as EventMouseEvent,
};
pub use file_picker::FilePickerWidget;
pub use image_integration::ImageIntegration;
pub use image_widget::{ImageFormat, ImageWidget, RenderMode};
pub use input::{ChatInputWidget, InputAnalyzer, Intent};
pub use integration::{
    LayoutCoordinator, LayoutInfo, StateSynchronizer, WidgetContainer, WidgetIntegration,
};
pub use layout::{Constraint, Layout, Rect};
pub use lifecycle::{
    get_tui_lifecycle_manager, initialize_tui_lifecycle_manager, register_tui_component,
    TuiLifecycleComponent, TuiLifecycleManager, TuiLifecycleState,
};
pub use logger_widget::{LogEntry, LogLevel, LoggerWidget};
pub use markdown::{MarkdownElement, MarkdownParser};
pub use model::{AppMessage, AppMode, AppModel, StateDiff};
pub use monitoring::{
    AnalyticsReport, AnonymousStatistics, ComplianceStatus, MemorySafetyMonitor, MetricsCollector,
    MonitoringReport, MonitoringSystem, PerformanceMonitor, PerformanceProfiler, PerformanceReport,
    SafetyCheckResult, SafetyIncident, SafetyIncidentType, SafetySeverity, UsageAnalytics,
    UserExperienceMetrics, UserExperienceReport,
};
pub use performance::{
    ActiveJob, CacheStats, ContentCache, CpuMonitor, CpuSample, CpuStats, DiffRenderOptimizer,
    FileOperationType, HistoryLimits, Job, JobId, JobOutput, JobPriority, JobQueue, JobQueueStats,
    JobResult, JobTask, LazyLoadConfig, LazyMessageHistory, LeakSeverity, MemoryLeak,
    MemoryProfiler, MemorySample, MemoryStats, MemoryTracker, ProfileSpan, ProfileSpanHandle,
    ProfileStats, ProgressReporter, ProgressStats, ProgressStatus, ProgressTracker, ProgressUpdate,
    RenderPerformanceMetrics, RenderPerformanceTracker, ThemeSwitchPerformance,
    VirtualScrollManager,
};
pub use plugins::{
    CommandPlugin,
    CommandResult,
    DiscoveredPlugin,
    EnhancedPluginMetadata,
    // Enhanced plugin architecture
    EnhancedPluginRegistry,
    MarketplaceTheme,
    Plugin,
    PluginCapability,
    PluginCommand,
    PluginContext,
    PluginId,
    PluginManager,
    PluginManifest,
    PluginMessage,
    PluginMetadata,
    PluginOperation,
    PluginSandbox,
    PluginTheme,
    PluginVersion,
    RateLimiter,
    ThemeMarketplace,
    ThemePlugin,
    ThemePluginImpl,
    UiComponentPlugin,
};
pub use popup_widget::{PopupButton, PopupType, PopupWidget};
pub use progressive_enhancement::{
    FeatureLevel, FeatureToggles, ProgressiveEnhancement, RenderingStrategy,
};
// theme::ThemeManager moved to ricecoder-themes
// VCS integration moved to ricecoder-vcs crate
// pub use ricecoder_vcs::tui_integration::{VcsIntegration, VcsStatus};
// VCS integration moved to ricecoder-vcs crate
// pub use status_bar::StatusBarVcsExt;
// theme_loader, theme_registry, theme_reset moved to ricecoder-themes
pub use project_bootstrap::{BootstrapResult, ProjectBootstrap, ProjectInfo};
pub use prompt::{ContextIndicators, PromptConfig, PromptWidget};
pub use prompt_context::PromptContext;
// Provider management components
pub use providers::{
    ProviderFactory, ProviderManager, ProviderPerformanceWidget, ProviderStatusWidget,
};
pub use reactive_ui_updates::{
    ConflictInfo, ConflictResolution, ConflictType, FileChangeEvent, FileChangeType, LiveDataEvent,
    LiveDataSynchronizer, ReactiveRenderer, ReactiveUICoordinator, SessionChangeType,
    SessionSyncEvent, UpdatePriority, UpdateType,
};
pub use real_time_updates::{
    OperationInfo, OperationStatus, ProgressIndicator, RealTimeStats, RealTimeStream,
    RealTimeUpdates, StreamData, StreamType,
};
pub use render_pipeline::LazyLoader;
pub use ricecoder_storage::config::TuiConfig;
// ProviderIntegration is now exported from ricecoder-providers
pub use scrollview_widget::ScrollViewWidget;
// Session exports moved to ricecoder-sessions crate
pub use style::{ColorSupport, Theme};
pub use terminal_state::{
    ColorSupport as TerminalColorSupport, TerminalCapabilities, TerminalState, TerminalType,
};
pub use textarea_widget::TextAreaWidget;
pub use tokio_util::sync::CancellationToken;
pub use tree_widget::{TreeNode, TreeWidget};
pub use ui_components::{
    LoadingManager, OptimisticUpdater, VirtualContent, VirtualList, VirtualNode, VirtualRenderer,
    VirtualStyle,
};
pub use widgets::{ChatWidget, Message, MessageAuthor};
