//! Progressive enhancement system for RiceCoder TUI
//!
//! This module implements:
//! - Capability-based feature toggles
//! - Fallback rendering strategies
//! - Graceful degradation based on terminal capabilities
//! - Feature detection and adaptation

use crate::terminal_state::{TerminalCapabilities, ColorSupport, TerminalType};
use std::collections::HashMap;

/// Feature toggle levels for progressive enhancement
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FeatureLevel {
    /// Minimal features - basic text interface
    Minimal = 0,
    /// Basic features - colors, mouse, basic graphics
    Basic = 1,
    /// Enhanced features - advanced graphics, animations
    Enhanced = 2,
    /// Full features - all available capabilities
    Full = 3,
}

/// Rendering strategy for different capability levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderingStrategy {
    /// Text-only rendering
    TextOnly,
    /// Basic ANSI rendering with colors
    AnsiBasic,
    /// Enhanced ANSI with 256 colors
    AnsiEnhanced,
    /// True color rendering
    TrueColor,
    /// Graphics protocol rendering (sixel, kitty, etc.)
    GraphicsProtocol,
}

/// Feature toggle configuration
#[derive(Debug, Clone)]
pub struct FeatureToggles {
    /// Enable mouse interactions
    pub mouse_support: bool,
    /// Enable graphics rendering
    pub graphics_support: bool,
    /// Enable animations and transitions
    pub animations: bool,
    /// Enable advanced Unicode characters
    pub unicode_support: bool,
    /// Enable keyboard shortcuts
    pub keyboard_shortcuts: bool,
    /// Enable screen reader support
    pub screen_reader: bool,
    /// Enable high contrast themes
    pub high_contrast: bool,
    /// Enable virtual scrolling
    pub virtual_scrolling: bool,
    /// Enable lazy loading
    pub lazy_loading: bool,
    /// Enable optimistic updates
    pub optimistic_updates: bool,
}

impl Default for FeatureToggles {
    fn default() -> Self {
        Self {
            mouse_support: true,
            graphics_support: true,
            animations: true,
            unicode_support: true,
            keyboard_shortcuts: true,
            screen_reader: false,
            high_contrast: false,
            virtual_scrolling: true,
            lazy_loading: true,
            optimistic_updates: true,
        }
    }
}

/// Progressive enhancement manager
pub struct ProgressiveEnhancement {
    capabilities: TerminalCapabilities,
    feature_level: FeatureLevel,
    feature_toggles: FeatureToggles,
    rendering_strategy: RenderingStrategy,
    fallback_strategies: HashMap<String, Vec<RenderingStrategy>>,
}

impl ProgressiveEnhancement {
    /// Create a new progressive enhancement manager
    pub fn new(capabilities: TerminalCapabilities) -> Self {
        let feature_level = Self::determine_feature_level(&capabilities);
        let feature_toggles = Self::determine_feature_toggles(&capabilities, feature_level);
        let rendering_strategy = Self::determine_rendering_strategy(&capabilities, feature_level);

        let mut fallback_strategies = HashMap::new();
        Self::setup_fallback_strategies(&mut fallback_strategies);

        Self {
            capabilities,
            feature_level,
            feature_toggles,
            rendering_strategy,
            fallback_strategies,
        }
    }

    /// Determine the appropriate feature level based on terminal capabilities
    fn determine_feature_level(capabilities: &TerminalCapabilities) -> FeatureLevel {
        // SSH sessions get reduced features
        if capabilities.is_ssh {
            return FeatureLevel::Basic;
        }

        // Determine based on terminal type and capabilities
        match capabilities.terminal_type {
            TerminalType::Kitty | TerminalType::WezTerm => {
                if capabilities.color_support == ColorSupport::TrueColor {
                    FeatureLevel::Full
                } else {
                    FeatureLevel::Enhanced
                }
            }
            TerminalType::ITerm2 | TerminalType::WindowsTerminal => {
                if capabilities.color_support >= ColorSupport::Ansi256 {
                    FeatureLevel::Enhanced
                } else {
                    FeatureLevel::Basic
                }
            }
            TerminalType::Xterm | TerminalType::Alacritty | TerminalType::Foot => {
                if capabilities.color_support >= ColorSupport::Ansi16 {
                    FeatureLevel::Basic
                } else {
                    FeatureLevel::Minimal
                }
            }
            _ => {
                // Unknown or basic terminals
                if capabilities.color_support >= ColorSupport::Ansi16 {
                    FeatureLevel::Basic
                } else {
                    FeatureLevel::Minimal
                }
            }
        }
    }

    /// Determine feature toggles based on capabilities and feature level
    fn determine_feature_toggles(capabilities: &TerminalCapabilities, feature_level: FeatureLevel) -> FeatureToggles {
        let mut toggles = FeatureToggles::default();

        // Adjust toggles based on feature level
        match feature_level {
            FeatureLevel::Minimal => {
                toggles.mouse_support = false;
                toggles.graphics_support = false;
                toggles.animations = false;
                toggles.unicode_support = capabilities.unicode_support;
                toggles.keyboard_shortcuts = true;
                toggles.screen_reader = false;
                toggles.high_contrast = false;
                toggles.virtual_scrolling = false;
                toggles.lazy_loading = false;
                toggles.optimistic_updates = false;
            }
            FeatureLevel::Basic => {
                toggles.mouse_support = capabilities.mouse_support;
                toggles.graphics_support = false; // No graphics in basic mode
                toggles.animations = false;
                toggles.unicode_support = capabilities.unicode_support;
                toggles.keyboard_shortcuts = true;
                toggles.screen_reader = false;
                toggles.high_contrast = false;
                toggles.virtual_scrolling = true;
                toggles.lazy_loading = true;
                toggles.optimistic_updates = false;
            }
            FeatureLevel::Enhanced => {
                toggles.mouse_support = capabilities.mouse_support;
                toggles.graphics_support = capabilities.sixel_support;
                toggles.animations = true;
                toggles.unicode_support = capabilities.unicode_support;
                toggles.keyboard_shortcuts = true;
                toggles.screen_reader = false;
                toggles.high_contrast = false;
                toggles.virtual_scrolling = true;
                toggles.lazy_loading = true;
                toggles.optimistic_updates = true;
            }
            FeatureLevel::Full => {
                toggles.mouse_support = capabilities.mouse_support;
                toggles.graphics_support = capabilities.sixel_support ||
                                         capabilities.kitty_graphics_support ||
                                         capabilities.iterm2_inline_images_support;
                toggles.animations = true;
                toggles.unicode_support = capabilities.unicode_support;
                toggles.keyboard_shortcuts = true;
                toggles.screen_reader = false;
                toggles.high_contrast = false;
                toggles.virtual_scrolling = true;
                toggles.lazy_loading = true;
                toggles.optimistic_updates = true;
            }
        }

        // Override for SSH sessions
        if capabilities.is_ssh {
            toggles.graphics_support = false;
            toggles.animations = false;
            toggles.lazy_loading = false; // Network issues over SSH
        }

        toggles
    }

    /// Determine the appropriate rendering strategy
    fn determine_rendering_strategy(capabilities: &TerminalCapabilities, feature_level: FeatureLevel) -> RenderingStrategy {
        match feature_level {
            FeatureLevel::Minimal => RenderingStrategy::TextOnly,
            FeatureLevel::Basic => {
                match capabilities.color_support {
                    ColorSupport::None => RenderingStrategy::TextOnly,
                    ColorSupport::Ansi16 => RenderingStrategy::AnsiBasic,
                    ColorSupport::Ansi256 => RenderingStrategy::AnsiEnhanced,
                    ColorSupport::TrueColor => RenderingStrategy::TrueColor,
                }
            }
            FeatureLevel::Enhanced | FeatureLevel::Full => {
                // Check for graphics protocols first
                if capabilities.kitty_graphics_support ||
                   capabilities.iterm2_inline_images_support ||
                   capabilities.wezterm_multiplexer_support ||
                   capabilities.sixel_support {
                    RenderingStrategy::GraphicsProtocol
                } else {
                    match capabilities.color_support {
                        ColorSupport::None => RenderingStrategy::TextOnly,
                        ColorSupport::Ansi16 => RenderingStrategy::AnsiBasic,
                        ColorSupport::Ansi256 => RenderingStrategy::AnsiEnhanced,
                        ColorSupport::TrueColor => RenderingStrategy::TrueColor,
                    }
                }
            }
        }
    }

    /// Setup fallback strategies for different features
    fn setup_fallback_strategies(fallback_strategies: &mut HashMap<String, Vec<RenderingStrategy>>) {
        // Image rendering fallbacks
        fallback_strategies.insert(
            "images".to_string(),
            vec![
                RenderingStrategy::GraphicsProtocol,
                RenderingStrategy::TrueColor,
                RenderingStrategy::AnsiEnhanced,
                RenderingStrategy::AnsiBasic,
                RenderingStrategy::TextOnly,
            ]
        );

        // UI element rendering fallbacks
        fallback_strategies.insert(
            "ui_elements".to_string(),
            vec![
                RenderingStrategy::TrueColor,
                RenderingStrategy::AnsiEnhanced,
                RenderingStrategy::AnsiBasic,
                RenderingStrategy::TextOnly,
            ]
        );

        // Text rendering fallbacks
        fallback_strategies.insert(
            "text".to_string(),
            vec![
                RenderingStrategy::TrueColor,
                RenderingStrategy::AnsiEnhanced,
                RenderingStrategy::AnsiBasic,
                RenderingStrategy::TextOnly,
            ]
        );
    }

    /// Get the current feature level
    pub fn feature_level(&self) -> FeatureLevel {
        self.feature_level
    }

    /// Get the current rendering strategy
    pub fn rendering_strategy(&self) -> RenderingStrategy {
        self.rendering_strategy
    }

    /// Get feature toggles
    pub fn feature_toggles(&self) -> &FeatureToggles {
        &self.feature_toggles
    }

    /// Check if a specific feature is enabled
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        match feature {
            "mouse_support" => self.feature_toggles.mouse_support,
            "graphics_support" => self.feature_toggles.graphics_support,
            "animations" => self.feature_toggles.animations,
            "unicode_support" => self.feature_toggles.unicode_support,
            "keyboard_shortcuts" => self.feature_toggles.keyboard_shortcuts,
            "screen_reader" => self.feature_toggles.screen_reader,
            "high_contrast" => self.feature_toggles.high_contrast,
            "virtual_scrolling" => self.feature_toggles.virtual_scrolling,
            "lazy_loading" => self.feature_toggles.lazy_loading,
            "optimistic_updates" => self.feature_toggles.optimistic_updates,
            _ => false,
        }
    }

    /// Get fallback strategies for a feature
    pub fn get_fallback_strategies(&self, feature: &str) -> Option<&Vec<RenderingStrategy>> {
        self.fallback_strategies.get(feature)
    }

    /// Check if we should use reduced functionality mode
    pub fn should_use_reduced_mode(&self) -> bool {
        self.capabilities.should_reduce_graphics() || self.feature_level <= FeatureLevel::Basic
    }

    /// Get the maximum supported rendering strategy for a feature
    pub fn get_max_rendering_strategy(&self, feature: &str) -> RenderingStrategy {
        if let Some(strategies) = self.fallback_strategies.get(feature) {
            for strategy in strategies {
                if self.supports_rendering_strategy(*strategy) {
                    return *strategy;
                }
            }
        }
        RenderingStrategy::TextOnly
    }

    /// Check if the terminal supports a specific rendering strategy
    pub fn supports_rendering_strategy(&self, strategy: RenderingStrategy) -> bool {
        match strategy {
            RenderingStrategy::TextOnly => true, // Always supported
            RenderingStrategy::AnsiBasic => self.capabilities.color_support >= ColorSupport::Ansi16,
            RenderingStrategy::AnsiEnhanced => self.capabilities.color_support >= ColorSupport::Ansi256,
            RenderingStrategy::TrueColor => self.capabilities.color_support == ColorSupport::TrueColor,
            RenderingStrategy::GraphicsProtocol => {
                self.capabilities.sixel_support ||
                self.capabilities.kitty_graphics_support ||
                self.capabilities.iterm2_inline_images_support ||
                self.capabilities.wezterm_multiplexer_support
            }
        }
    }

    /// Get a human-readable description of the current capabilities
    pub fn get_capability_description(&self) -> String {
        let mut description = format!("Terminal: {:?}, ", self.capabilities.terminal_type);
        description.push_str(&format!("Colors: {:?}, ", self.capabilities.color_support));
        description.push_str(&format!("Feature Level: {:?}, ", self.feature_level));
        description.push_str(&format!("Rendering: {:?}", self.rendering_strategy));

        if self.capabilities.is_ssh {
            description.push_str(" (SSH session)");
        }
        if self.capabilities.is_tmux {
            description.push_str(" (TMUX session)");
        }

        description
    }

    /// Force a specific feature level (for testing or configuration)
    pub fn force_feature_level(&mut self, level: FeatureLevel) {
        self.feature_level = level;
        self.feature_toggles = Self::determine_feature_toggles(&self.capabilities, level);
        self.rendering_strategy = Self::determine_rendering_strategy(&self.capabilities, level);
    }

    /// Get terminal capabilities reference
    pub fn capabilities(&self) -> &TerminalCapabilities {
        &self.capabilities
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal_state::{TerminalCapabilities, TerminalType, ColorSupport, CapabilityOverrides};

    #[test]
    fn test_feature_level_determination() {
        // Test minimal capabilities
        let minimal_caps = TerminalCapabilities {
            terminal_type: TerminalType::Unknown,
            color_support: ColorSupport::None,
            mouse_support: false,
            sixel_support: false,
            kitty_graphics_support: false,
            iterm2_inline_images_support: false,
            wezterm_multiplexer_support: false,
            unicode_support: false,
            is_ssh: false,
            is_tmux: false,
            tmux_version: None,
            size: (80, 24),
            overrides_applied: CapabilityOverrides::default(),
        };

        let pe = ProgressiveEnhancement::new(minimal_caps);
        assert_eq!(pe.feature_level(), FeatureLevel::Minimal);
        assert_eq!(pe.rendering_strategy(), RenderingStrategy::TextOnly);
        assert!(!pe.feature_toggles().graphics_support);
    }

    #[test]
    fn test_kitty_full_features() {
        let kitty_caps = TerminalCapabilities {
            terminal_type: TerminalType::Kitty,
            color_support: ColorSupport::TrueColor,
            mouse_support: true,
            sixel_support: true,
            kitty_graphics_support: true,
            iterm2_inline_images_support: false,
            wezterm_multiplexer_support: false,
            unicode_support: true,
            is_ssh: false,
            is_tmux: false,
            tmux_version: None,
            size: (120, 30),
            overrides_applied: CapabilityOverrides::default(),
        };

        let pe = ProgressiveEnhancement::new(kitty_caps);
        assert_eq!(pe.feature_level(), FeatureLevel::Full);
        assert_eq!(pe.rendering_strategy(), RenderingStrategy::GraphicsProtocol);
        assert!(pe.feature_toggles().graphics_support);
        assert!(pe.feature_toggles().animations);
    }

    #[test]
    fn test_ssh_reduced_features() {
        let ssh_caps = TerminalCapabilities {
            terminal_type: TerminalType::Xterm,
            color_support: ColorSupport::TrueColor,
            mouse_support: true,
            sixel_support: true,
            kitty_graphics_support: false,
            iterm2_inline_images_support: false,
            wezterm_multiplexer_support: false,
            unicode_support: true,
            is_ssh: true, // SSH session
            is_tmux: false,
            tmux_version: None,
            size: (80, 24),
            overrides_applied: CapabilityOverrides::default(),
        };

        let pe = ProgressiveEnhancement::new(ssh_caps);
        assert_eq!(pe.feature_level(), FeatureLevel::Basic); // Reduced due to SSH
        assert!(!pe.feature_toggles().graphics_support); // Graphics disabled over SSH
        assert!(!pe.feature_toggles().animations); // Animations disabled over SSH
    }

    #[test]
    fn test_fallback_strategies() {
        let caps = TerminalCapabilities::detect();
        let pe = ProgressiveEnhancement::new(caps);

        let image_fallbacks = pe.get_fallback_strategies("images");
        assert!(image_fallbacks.is_some());
        assert!(image_fallbacks.unwrap().contains(&RenderingStrategy::GraphicsProtocol));
        assert!(image_fallbacks.unwrap().contains(&RenderingStrategy::TextOnly));
    }

    #[test]
    fn test_rendering_strategy_support() {
        let caps = TerminalCapabilities::detect();
        let pe = ProgressiveEnhancement::new(caps);

        // Text-only should always be supported
        assert!(pe.supports_rendering_strategy(RenderingStrategy::TextOnly));

        // Test color support levels
        if caps.color_support >= ColorSupport::Ansi16 {
            assert!(pe.supports_rendering_strategy(RenderingStrategy::AnsiBasic));
        }
        if caps.color_support >= ColorSupport::Ansi256 {
            assert!(pe.supports_rendering_strategy(RenderingStrategy::AnsiEnhanced));
        }
        if caps.color_support == ColorSupport::TrueColor {
            assert!(pe.supports_rendering_strategy(RenderingStrategy::TrueColor));
        }
    }

    #[test]
    fn test_feature_toggle_checks() {
        let caps = TerminalCapabilities::detect();
        let pe = ProgressiveEnhancement::new(caps);

        // Test feature toggle access
        assert!(pe.is_feature_enabled("keyboard_shortcuts")); // Should always be enabled
        assert_eq!(pe.is_feature_enabled("mouse_support"), pe.feature_toggles().mouse_support);
        assert_eq!(pe.is_feature_enabled("graphics_support"), pe.feature_toggles().graphics_support);
    }

    #[test]
    fn test_force_feature_level() {
        let caps = TerminalCapabilities::detect();
        let mut pe = ProgressiveEnhancement::new(caps.clone());

        let original_level = pe.feature_level();
        pe.force_feature_level(FeatureLevel::Minimal);

        assert_eq!(pe.feature_level(), FeatureLevel::Minimal);
        assert!(!pe.feature_toggles().graphics_support);
        assert!(!pe.feature_toggles().animations);

        // Restore original level
        pe.force_feature_level(original_level);
        assert_eq!(pe.feature_level(), original_level);
    }
}</content>
<parameter name="filePath">projects/ricecoder/crates/ricecoder-tui/src/progressive_enhancement.rs