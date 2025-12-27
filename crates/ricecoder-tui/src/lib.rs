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

// === Old TEA System Modules (commented out - migrated to src/tui/) ===
// pub mod accessibility;  // Depends on old model::AppMessage
// pub mod components;     // Depends on old model::AppMessage, AppModel
// pub mod command_palette; // Depends on old model::AppMessage
// pub mod diff;           // Depends on old model::AppMessage, AppModel
// pub mod error_handling; // Depends on old model, tea::ReactiveState
// pub mod file_picker;    // Depends on old model::AppMessage
// pub mod plugins;        // Depends on old model::AppMessage, AppModel
// pub mod prompt;         // Depends on old model::AppMode
// pub mod reactive_ui_updates; // Depends on old model, tea, StateDiff
// pub mod render_pipeline; // Depends on old model::AppModel, StateChange
// pub mod widgets;        // Depends on old model, StateDiff

// === Core Modules (keep) ===
pub mod banner;
pub mod clipboard;
pub mod code_editor_widget;
pub mod command_blocks;
pub mod di;
pub mod error;
pub mod error_handling_stub; // Minimal error types for compatibility
pub mod lifecycle;
pub mod menu_stub; // Minimal menu types for dialog compatibility
pub mod model_stub; // Minimal model types for CLI compatibility
pub mod tui; // New TUI system

// Stub module aliases for old imports
pub mod components {
    pub mod menu {
        pub use crate::menu_stub::{MenuItem, MenuWidget};
    }
}
pub mod error_handling {
    pub use crate::error_handling_stub::{ErrorCategory, ErrorManager, ErrorSeverity, RiceError};
}
pub mod model {
    pub use crate::model_stub::{ProviderConnectionState, ProviderInfo};
}

// === Utility Modules (keep) ===
pub mod image_integration;
pub mod image_widget;
pub mod input;
pub mod layout;
pub mod logger_widget;
pub mod markdown;
pub mod monitoring;
pub mod performance;
pub mod popup_widget;
pub mod progressive_enhancement;
pub mod project_bootstrap;
pub mod real_time_updates;
pub mod scrollview_widget;
pub mod status_bar;
pub mod style;
pub mod terminal_state;
pub mod textarea_widget;
pub mod theme;
pub mod tree_widget;
pub mod ui_components;

// Re-export commonly used types
// Accessibility exports removed - depends on old TEA system
// pub use accessibility::{...};
pub use banner::{BannerArea, BannerComponent, BannerComponentConfig};
pub use clipboard::{ClipboardError, ClipboardManager, CopyFeedback, CopyOperation};
pub use code_editor_widget::{CodeEditorWidget, CodeLine, Language, SyntaxTheme};
pub use command_blocks::{Command, CommandBlock, CommandBlocksWidget, CommandStatus};
// Old TEA system exports removed
// pub use command_palette::{CommandPaletteWidget, PaletteCommand};
// pub use components::{...};
// pub use diff::{DiffHunk, DiffLine, DiffLineType, DiffViewType, DiffWidget};
pub use error::{KeybindError, StorageError, ToolError, TuiError, TuiResult};
// pub use file_picker::FilePickerWidget; // Old TEA system
pub use image_integration::ImageIntegration;
pub use image_widget::{ImageFormat, ImageWidget, RenderMode};
pub use input::{ChatInputWidget, InputAnalyzer, Intent};
pub use layout::{Constraint, Layout, Rect};
pub use lifecycle::{
    get_tui_lifecycle_manager, initialize_tui_lifecycle_manager, register_tui_component,
    TuiLifecycleComponent, TuiLifecycleManager, TuiLifecycleState,
};
pub use logger_widget::{LogEntry, LogLevel, LoggerWidget};
pub use markdown::{MarkdownElement, MarkdownParser};
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
// Plugins removed - depends on old TEA system
// pub use plugins::{...};
pub use popup_widget::{PopupButton, PopupType, PopupWidget};
pub use progressive_enhancement::{
    FeatureLevel, FeatureToggles, ProgressiveEnhancement, RenderingStrategy,
};
pub use project_bootstrap::{BootstrapResult, ProjectBootstrap, ProjectInfo};
// Old TEA system exports removed
// pub use prompt::{ContextIndicators, PromptConfig, PromptWidget};
// pub use reactive_ui_updates::{...};
// pub use render_pipeline::LazyLoader;
pub use real_time_updates::{
    OperationInfo, OperationStatus, ProgressIndicator, RealTimeStats, RealTimeStream,
    RealTimeUpdates, StreamData, StreamType,
};
pub use ricecoder_storage::config::TuiConfig;
// ProviderIntegration is now exported from ricecoder-providers
pub use scrollview_widget::ScrollViewWidget;
// Session exports moved to ricecoder-sessions crate
pub use ricecoder_themes::Theme;
pub use style::ColorSupport;
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
// Old TEA system exports removed
// pub use widgets::{ChatWidget, Message, MessageAuthor};
