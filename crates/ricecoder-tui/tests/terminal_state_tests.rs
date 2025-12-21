use ricecoder_tui::*;

mod tests {
    use super::*;

    #[test]
    fn test_terminal_capabilities_detection() {
        // Test that capability detection doesn't panic
        let capabilities = TerminalCapabilities::detect();

        // Basic sanity checks
        assert!(capabilities.size.0 > 0);
        assert!(capabilities.size.1 > 0);

        // Test that we can detect some basic capabilities
        // These should work in most environments
        println!("Detected capabilities: {:?}", capabilities);
    }

    #[test]
    fn test_terminal_type_detection() {
        let terminal_type = TerminalCapabilities::detect_terminal_type();
        // Should not panic and return some value
        println!("Detected terminal type: {:?}", terminal_type);
    }

    #[test]
    fn test_color_support_detection() {
        let color_support = TerminalCapabilities::detect_color_support();
        // Should not panic and return some value
        println!("Detected color support: {:?}", color_support);
    }

    #[test]
    fn test_ssh_detection() {
        let is_ssh = TerminalCapabilities::detect_ssh_session();
        // Should not panic
        println!("SSH session detected: {}", is_ssh);
    }

    #[test]
    fn test_tmux_detection() {
        let is_tmux = TerminalCapabilities::detect_tmux_session();
        // Should not panic
        println!("TMUX session detected: {}", is_tmux);
    }

    #[test]
    fn test_terminal_state_creation() {
        // This test verifies that TerminalState can be created
        // Note: This test may fail in non-TTY environments
        if atty::is(atty::Stream::Stdout) {
            let state = TerminalState::capture();
            // We don't assert success here because the test environment may not support it
            // But we verify the code compiles and runs
            if let Ok(state) = state {
                assert!(state.capabilities.size.0 > 0);
                assert!(state.capabilities.size.1 > 0);
            }
        }
    }

    #[test]
    fn test_terminal_state_drop() {
        // This test verifies that TerminalState implements Drop correctly
        if atty::is(atty::Stream::Stdout) {
            let _state = TerminalState::capture();
            // Drop is called automatically when _state goes out of scope
        }
    }

    #[test]
    fn test_capability_methods() {
        let capabilities = TerminalCapabilities::detect();

        // Test that methods don't panic
        let _should_reduce = capabilities.should_reduce_graphics();
        let _should_wrap = capabilities.should_wrap_osc52();
        let _color_mode = capabilities.get_color_mode();
        let _optimizations = capabilities.get_optimizations();

        println!("Capability methods work correctly");
    }

    #[test]
    fn test_capability_overrides() {
        let overrides = CapabilityOverrides {
            color_support: Some(ColorSupport::TrueColor),
            mouse_support: Some(false),
            sixel_support: Some(true),
            unicode_support: Some(false),
            force_reduced_graphics: Some(true),
        };

        let capabilities = TerminalCapabilities::detect_with_overrides(overrides.clone());

        // Verify overrides were applied
        assert_eq!(capabilities.color_support, ColorSupport::TrueColor);
        assert_eq!(capabilities.mouse_support, false);
        assert_eq!(capabilities.sixel_support, true);
        assert_eq!(capabilities.unicode_support, false);
        assert!(capabilities.should_reduce_graphics());

        println!("Capability overrides work correctly");
    }

    #[test]
    fn test_tmux_escape_sequence_wrapping() {
        // Create capabilities with TMUX enabled
        let mut capabilities = TerminalCapabilities::detect();
        capabilities.is_tmux = true;

        let sequence = "\x1b[31mred text\x1b[0m";
        let wrapped = capabilities.wrap_escape_sequence(sequence);

        // Should be wrapped with TMUX passthrough
        assert!(wrapped.starts_with("\x1bPtmux;\x1b"));
        assert!(wrapped.ends_with("\x1b\\"));

        println!("TMUX escape sequence wrapping works correctly");
    }

    #[test]
    fn test_tmux_passthrough_methods() {
        let mut capabilities = TerminalCapabilities::detect();
        capabilities.is_tmux = true;

        assert_eq!(
            capabilities.get_tmux_passthrough_prefix(),
            Some("\x1bPtmux;\x1b")
        );
        assert_eq!(capabilities.get_tmux_passthrough_suffix(), Some("\x1b\\"));

        // Test without TMUX
        capabilities.is_tmux = false;
        assert_eq!(capabilities.get_tmux_passthrough_prefix(), None);
        assert_eq!(capabilities.get_tmux_passthrough_suffix(), None);

        println!("TMUX passthrough methods work correctly");
    }

    #[test]
    fn test_terminal_optimizations() {
        let capabilities = TerminalCapabilities::detect();
        let optimizations = capabilities.get_optimizations();

        // Should contain expected optimization keys
        assert!(optimizations.contains_key("mouse_support"));
        assert!(optimizations.contains_key("sixel_graphics"));
        assert!(optimizations.contains_key("unicode_chars"));
        assert!(optimizations.contains_key("reduced_graphics"));
        assert!(optimizations.contains_key("tmux_mode"));

        println!("Terminal optimizations work correctly");
    }
}
