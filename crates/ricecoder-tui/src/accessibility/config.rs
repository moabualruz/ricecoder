//! Accessibility configuration
//!
//! Provides comprehensive accessibility settings including screen reader support,
//! high contrast mode, animation controls, and font size adjustments.

use serde::{Deserialize, Serialize};

use super::{AnimationConfig, FocusIndicatorStyle};

/// Accessibility configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityConfig {
    /// Enable screen reader support
    pub screen_reader_enabled: bool,
    /// Enable high contrast mode
    pub high_contrast_enabled: bool,
    /// Disable animations
    pub animations_disabled: bool,
    /// Enable state announcements
    pub announcements_enabled: bool,
    /// Focus indicator style
    pub focus_indicator: FocusIndicatorStyle,
    /// Animation configuration
    #[serde(default)]
    pub animations: AnimationConfig,
    /// Font size multiplier (1.0 = normal, 1.5 = 150%, etc.)
    pub font_size_multiplier: f32,
    /// Enable large click targets
    pub large_click_targets: bool,
    /// Enable auto-advance for forms
    pub auto_advance: bool,
}

impl Default for AccessibilityConfig {
    fn default() -> Self {
        Self {
            screen_reader_enabled: false,
            high_contrast_enabled: false,
            animations_disabled: false,
            announcements_enabled: true,
            focus_indicator: FocusIndicatorStyle::Bracket,
            animations: AnimationConfig::default(),
            font_size_multiplier: 1.0,
            large_click_targets: false,
            auto_advance: false,
        }
    }
}

impl AccessibilityConfig {
    /// Enable accessibility features
    pub fn enable(&mut self) {
        self.screen_reader_enabled = true;
        self.high_contrast_enabled = true;
        self.announcements_enabled = true;
        self.animations.reduce_motion = true;
        self.large_click_targets = true;
    }

    /// Disable accessibility features
    pub fn disable(&mut self) {
        self.screen_reader_enabled = false;
        self.high_contrast_enabled = false;
        self.announcements_enabled = false;
        self.animations.reduce_motion = false;
        self.large_click_targets = false;
    }

    /// Set font size multiplier for accessibility
    pub fn set_font_size_multiplier(&mut self, multiplier: f32) {
        self.font_size_multiplier = multiplier.clamp(1.0, 2.0);
    }

    /// Enable large click targets for motor accessibility
    pub fn enable_large_click_targets(&mut self) {
        self.large_click_targets = true;
    }

    /// Enable auto-advance for forms
    pub fn enable_auto_advance(&mut self) {
        self.auto_advance = true;
    }

    /// Check if large text is enabled (WCAG 2.1 AA requires 1.5x normal)
    pub fn is_large_text(&self) -> bool {
        self.font_size_multiplier >= 1.5
    }

    /// Get effective font size
    pub fn effective_font_size(&self, base_size: u16) -> u16 {
        ((base_size as f32) * self.font_size_multiplier) as u16
    }

    /// Merge two accessibility configurations (self takes precedence over other)
    pub fn merge(mut self, other: Self) -> Self {
        // Only override if values are not default
        if other.screen_reader_enabled != Self::default().screen_reader_enabled {
            self.screen_reader_enabled = other.screen_reader_enabled;
        }

        if other.high_contrast_enabled != Self::default().high_contrast_enabled {
            self.high_contrast_enabled = other.high_contrast_enabled;
        }

        if other.animations_disabled != Self::default().animations_disabled {
            self.animations_disabled = other.animations_disabled;
        }

        if other.announcements_enabled != Self::default().announcements_enabled {
            self.announcements_enabled = other.announcements_enabled;
        }

        if other.font_size_multiplier != Self::default().font_size_multiplier {
            self.font_size_multiplier = other.font_size_multiplier;
        }

        if other.large_click_targets != Self::default().large_click_targets {
            self.large_click_targets = other.large_click_targets;
        }

        if other.auto_advance != Self::default().auto_advance {
            self.auto_advance = other.auto_advance;
        }

        self
    }
}
