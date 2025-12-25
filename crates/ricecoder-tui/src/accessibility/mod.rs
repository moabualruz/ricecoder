//! Accessibility module for ricecoder-tui
//!
//! Provides comprehensive accessibility support including:
//! - Animation configuration for reduced motion
//! - ARIA properties and roles
//! - Focus management and indicators
//! - Screen reader support
//! - Keyboard navigation
//! - Vim-like input modes
//! - High contrast themes
//! - Keyboard shortcut customization

pub mod animation;
pub mod aria;
pub mod config;
pub mod focus;
pub mod navigation;
pub mod screen_reader;
pub mod shortcuts;
pub mod text_alternative;
pub mod themes;
pub mod vim_mode;

// Re-export commonly used types at module root
pub use animation::AnimationConfig;
pub use aria::{AriaLive, AriaProperties, AriaRelevant, AriaRole};
pub use config::AccessibilityConfig;
pub use focus::{FocusIndicatorStyle, FocusManager};
pub use navigation::{
    EnhancedKeyboardNavigation, Heading, KeyboardNavigationManager, Landmark,
    NavigationDirection, NavigationPosition, SemanticNavigator,
};
pub use screen_reader::{Announcement, AnnouncementPriority, LiveRegion, ScreenReaderAnnouncer};
pub use shortcuts::{
    initialize_default_shortcuts, KeyboardShortcut, KeyboardShortcutCustomizer,
    KeyboardShortcutHelp,
};
pub use text_alternative::{ElementType, TextAlternative};
pub use themes::{Color, HighContrastTheme, HighContrastThemeManager};
pub use vim_mode::{
    DeleteOperation, InputMode, ModeAction, ModeIndicator, ModeIndicatorStyle, Movement,
    VimModeManager,
};
