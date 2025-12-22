use ricecoder_tui::*;

use crate::terminal_state::{
    CapabilityOverrides, ColorSupport, TerminalCapabilities, TerminalType,
};

mod tests {
    use super::*;

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
        assert!(image_fallbacks
            .unwrap()
            .contains(&RenderingStrategy::GraphicsProtocol));
        assert!(image_fallbacks
            .unwrap()
            .contains(&RenderingStrategy::TextOnly));
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
        assert_eq!(
            pe.is_feature_enabled("mouse_support"),
            pe.feature_toggles().mouse_support
        );
        assert_eq!(
            pe.is_feature_enabled("graphics_support"),
            pe.feature_toggles().graphics_support
        );
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
}
