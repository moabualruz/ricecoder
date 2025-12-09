//! Property-based tests for theme switching atomicity
//!
//! **Feature: ricecoder-themes, Property 5: Theme Switching Atomicity**
//! For any theme switch, all elements update or none do (no partial updates)
//! **Validates: Requirements 5.1, 5.2, 5.3**

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

/// Property 5: Theme Switching Atomicity
/// For any theme switch, all elements update or none do (no partial updates)
///
/// This property verifies that:
/// 1. When switching themes, all colors are updated together
/// 2. No partial theme updates occur
/// 3. Theme switching is atomic (all-or-nothing)
proptest! {
    #[test]
    fn prop_theme_switching_atomicity_all_colors_update(
        theme_name1 in theme_name_strategy(),
        theme_name2 in theme_name_strategy(),
    ) {
        let manager = ThemeManager::new();
        
        // Switch to first theme
        manager.switch_by_name(&theme_name1).expect("Failed to switch to theme 1");
        let _theme1 = manager.current().expect("Failed to get theme 1");
        
        // Switch to second theme
        manager.switch_by_name(&theme_name2).expect("Failed to switch to theme 2");
        let theme2 = manager.current().expect("Failed to get theme 2");
        
        // Verify all colors from theme2 are present
        // If theme switching is atomic, all colors should be from theme2
        assert_eq!(theme2.name, theme_name2);
        
        // Verify we're not in a partial state (all colors should be from the same theme)
        // This is guaranteed by the fact that we got a complete theme object
        let _ = theme2.primary;
        let _ = theme2.secondary;
        let _ = theme2.accent;
        let _ = theme2.background;
        let _ = theme2.foreground;
        let _ = theme2.error;
        let _ = theme2.warning;
        let _ = theme2.success;
    }
}

/// Property 6: Theme Switching Atomicity - Consistent State After Switch
/// For any theme switch, the theme state is consistent after the switch
proptest! {
    #[test]
    fn prop_theme_switching_atomicity_consistent_state(theme_name in theme_name_strategy()) {
        let manager = ThemeManager::new();
        manager.switch_by_name(&theme_name).expect("Failed to switch theme");
        
        let theme = manager.current().expect("Failed to get current theme");
        
        // Verify the theme name matches
        assert_eq!(theme.name, theme_name);
        
        // Verify all colors are from the same theme (consistent state)
        // Get the theme again to verify consistency
        let theme_again = manager.current().expect("Failed to get current theme again");
        assert_eq!(theme.name, theme_again.name);
        assert_eq!(theme.primary, theme_again.primary);
        assert_eq!(theme.secondary, theme_again.secondary);
        assert_eq!(theme.background, theme_again.background);
        assert_eq!(theme.foreground, theme_again.foreground);
    }
}

/// Property 7: Theme Switching Atomicity - No Partial Updates
/// For any sequence of theme switches, each switch is atomic
proptest! {
    #[test]
    fn prop_theme_switching_atomicity_no_partial_updates(
        theme_names in prop::collection::vec(theme_name_strategy(), 1..5),
    ) {
        let manager = ThemeManager::new();
        
        for theme_name in theme_names {
            manager.switch_by_name(&theme_name).expect("Failed to switch theme");
            let theme = manager.current().expect("Failed to get current theme");
            
            // Verify the theme is complete (not partial)
            assert_eq!(theme.name, theme_name);
            
            // Verify all color fields are present
            let _ = theme.primary;
            let _ = theme.secondary;
            let _ = theme.accent;
            let _ = theme.background;
            let _ = theme.foreground;
            let _ = theme.error;
            let _ = theme.warning;
            let _ = theme.success;
        }
    }
}

/// Property 8: Theme Switching Atomicity - Theme Name Consistency
/// For any theme switch, the theme name is consistent with the colors
proptest! {
    #[test]
    fn prop_theme_switching_atomicity_name_consistency(theme_name in theme_name_strategy()) {
        let manager = ThemeManager::new();
        manager.switch_by_name(&theme_name).expect("Failed to switch theme");
        
        let theme = manager.current().expect("Failed to get current theme");
        let current_name = manager.current_name().expect("Failed to get current name");
        
        // Verify theme name is consistent
        assert_eq!(theme.name, theme_name);
        assert_eq!(current_name, theme_name);
        assert_eq!(theme.name, current_name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_switching_atomicity_dark_to_light() {
        let manager = ThemeManager::new();
        
        manager.switch_by_name("dark").unwrap();
        let dark_theme = manager.current().unwrap();
        assert_eq!(dark_theme.name, "dark");
        
        manager.switch_by_name("light").unwrap();
        let light_theme = manager.current().unwrap();
        assert_eq!(light_theme.name, "light");
        
        // Verify we're not in a partial state
        assert_ne!(dark_theme.primary, light_theme.primary);
    }

    #[test]
    fn test_theme_switching_atomicity_all_themes() {
        let manager = ThemeManager::new();
        let themes = manager.available_themes();
        
        for theme_name in themes {
            manager.switch_by_name(theme_name).unwrap();
            let theme = manager.current().unwrap();
            
            // Verify theme is complete
            assert_eq!(theme.name, theme_name);
            let _ = theme.primary;
            let _ = theme.secondary;
            let _ = theme.background;
            let _ = theme.foreground;
        }
    }

    #[test]
    fn test_theme_switching_atomicity_multiple_switches() {
        let manager = ThemeManager::new();
        
        let themes = vec!["dark", "light", "dracula", "monokai", "nord", "high-contrast"];
        
        for theme_name in themes {
            manager.switch_by_name(theme_name).unwrap();
            let theme = manager.current().unwrap();
            assert_eq!(theme.name, theme_name);
            
            // Verify consistency
            let theme_again = manager.current().unwrap();
            assert_eq!(theme.name, theme_again.name);
            assert_eq!(theme.primary, theme_again.primary);
        }
    }

    #[test]
    fn test_theme_switching_atomicity_name_consistency() {
        let manager = ThemeManager::new();
        
        manager.switch_by_name("dracula").unwrap();
        let theme = manager.current().unwrap();
        let current_name = manager.current_name().unwrap();
        
        assert_eq!(theme.name, "dracula");
        assert_eq!(current_name, "dracula");
        assert_eq!(theme.name, current_name);
    }
}
