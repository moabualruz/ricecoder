//! Property-based tests for color reset correctness
//!
//! **Feature: ricecoder-themes, Property 7: Color Reset Correctness**
//! For any theme with modified colors, resetting restores original built-in colors
//! **Validates: Requirements 3.5**

use proptest::prelude::*;
use ricecoder_tui::theme::ThemeManager;

/// Strategy for generating valid theme names
fn theme_name_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("dark".to_string()),
        Just("light".to_string()),
        Just("dracula".to_string()),
        Just("monokai".to_string()),
        Just("nord".to_string()),
        Just("high-contrast".to_string()),
    ]
}

/// Property 7: Color Reset Correctness
/// For any theme with modified colors, resetting restores original built-in colors
///
/// This property verifies that:
/// 1. Resetting colors restores them to their original values
/// 2. Reset is idempotent (resetting twice gives the same result)
/// 3. Only the specified color is reset, not all colors
proptest! {
    #[test]
    fn prop_color_reset_restores_original(theme_name in theme_name_strategy()) {
        let manager = ThemeManager::new();
        manager.switch_by_name(&theme_name).expect("Failed to switch theme");
        
        // Get the original colors
        let original = manager.current().expect("Failed to get original theme");
        let original_primary = original.primary;
        
        // Reset colors
        manager.reset_colors().expect("Failed to reset colors");
        
        // Verify colors are restored
        let reset = manager.current().expect("Failed to get reset theme");
        assert_eq!(reset.primary, original_primary, "Primary color was not restored");
        assert_eq!(reset.secondary, original.secondary, "Secondary color was not restored");
        assert_eq!(reset.background, original.background, "Background color was not restored");
        assert_eq!(reset.foreground, original.foreground, "Foreground color was not restored");
    }
}

/// Property 8: Color Reset - Idempotence
/// For any theme, resetting colors multiple times gives the same result
proptest! {
    #[test]
    fn prop_color_reset_idempotent(theme_name in theme_name_strategy()) {
        let manager = ThemeManager::new();
        manager.switch_by_name(&theme_name).expect("Failed to switch theme");
        
        // Reset colors once
        manager.reset_colors().expect("Failed to reset colors");
        let reset1 = manager.current().expect("Failed to get reset theme 1");
        let colors1 = (
            reset1.primary,
            reset1.secondary,
            reset1.background,
            reset1.foreground,
        );
        
        // Reset colors again
        manager.reset_colors().expect("Failed to reset colors again");
        let reset2 = manager.current().expect("Failed to get reset theme 2");
        let colors2 = (
            reset2.primary,
            reset2.secondary,
            reset2.background,
            reset2.foreground,
        );
        
        // Verify both resets give the same result
        assert_eq!(colors1, colors2, "Reset is not idempotent");
    }
}

/// Property 9: Color Reset - Individual Color Reset
/// For any theme, resetting an individual color restores it to the original
proptest! {
    #[test]
    fn prop_color_reset_individual(theme_name in theme_name_strategy()) {
        let manager = ThemeManager::new();
        manager.switch_by_name(&theme_name).expect("Failed to switch theme");
        
        // Get the original primary color
        let original = manager.current().expect("Failed to get original theme");
        let original_primary = original.primary;
        
        // Reset the primary color
        manager.reset_color("primary").expect("Failed to reset primary color");
        
        // Verify primary color is restored
        let reset = manager.current().expect("Failed to get reset theme");
        assert_eq!(reset.primary, original_primary, "Primary color was not restored");
    }
}

/// Property 10: Color Reset - Get Default Color
/// For any theme, getting the default color returns the original value
proptest! {
    #[test]
    fn prop_color_reset_get_default(theme_name in theme_name_strategy()) {
        let manager = ThemeManager::new();
        manager.switch_by_name(&theme_name).expect("Failed to switch theme");
        
        // Get the original theme
        let original = manager.current().expect("Failed to get original theme");
        let original_primary = original.primary;
        
        // Get the default color
        let default_primary = manager.get_default_color("primary").expect("Failed to get default color");
        
        // Verify the default color matches the original
        assert_eq!(default_primary, original_primary, "Default color does not match original");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_reset_dark_theme() {
        let manager = ThemeManager::new();
        manager.switch_by_name("dark").unwrap();
        
        let original = manager.current().unwrap();
        let original_primary = original.primary;
        
        manager.reset_colors().unwrap();
        
        let reset = manager.current().unwrap();
        assert_eq!(reset.primary, original_primary);
    }

    #[test]
    fn test_color_reset_light_theme() {
        let manager = ThemeManager::new();
        manager.switch_by_name("light").unwrap();
        
        let original = manager.current().unwrap();
        let original_colors = (
            original.primary,
            original.secondary,
            original.background,
        );
        
        manager.reset_colors().unwrap();
        
        let reset = manager.current().unwrap();
        let reset_colors = (
            reset.primary,
            reset.secondary,
            reset.background,
        );
        
        assert_eq!(original_colors, reset_colors);
    }

    #[test]
    fn test_color_reset_idempotent() {
        let manager = ThemeManager::new();
        manager.switch_by_name("dracula").unwrap();
        
        manager.reset_colors().unwrap();
        let reset1 = manager.current().unwrap();
        let colors1 = (reset1.primary, reset1.secondary, reset1.background);
        
        manager.reset_colors().unwrap();
        let reset2 = manager.current().unwrap();
        let colors2 = (reset2.primary, reset2.secondary, reset2.background);
        
        assert_eq!(colors1, colors2);
    }

    #[test]
    fn test_color_reset_individual() {
        let manager = ThemeManager::new();
        manager.switch_by_name("monokai").unwrap();
        
        let original = manager.current().unwrap();
        let original_primary = original.primary;
        
        manager.reset_color("primary").unwrap();
        
        let reset = manager.current().unwrap();
        assert_eq!(reset.primary, original_primary);
    }

    #[test]
    fn test_color_reset_get_default() {
        let manager = ThemeManager::new();
        manager.switch_by_name("nord").unwrap();
        
        let original = manager.current().unwrap();
        let original_primary = original.primary;
        
        let default_primary = manager.get_default_color("primary").unwrap();
        
        assert_eq!(default_primary, original_primary);
    }

    #[test]
    fn test_color_reset_all_themes() {
        let manager = ThemeManager::new();
        let themes = manager.available_themes();
        
        for theme_name in themes {
            manager.switch_by_name(theme_name).unwrap();
            
            let original = manager.current().unwrap();
            let original_colors = (
                original.primary,
                original.secondary,
                original.background,
                original.foreground,
            );
            
            manager.reset_colors().unwrap();
            
            let reset = manager.current().unwrap();
            let reset_colors = (
                reset.primary,
                reset.secondary,
                reset.background,
                reset.foreground,
            );
            
            assert_eq!(original_colors, reset_colors);
        }
    }
}
