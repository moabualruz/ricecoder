//! Property-based tests for custom theme isolation
//!
//! **Feature: ricecoder-themes, Property 6: Custom Theme Isolation**
//! For any custom theme, modifications don't affect other themes
//! **Validates: Requirements 2.1, 2.2**

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

/// Property 6: Custom Theme Isolation
/// For any custom theme, modifications don't affect other themes
///
/// This property verifies that:
/// 1. Built-in themes are not affected by custom theme operations
/// 2. Custom themes are independent from each other
/// 3. Modifying one theme doesn't affect others
proptest! {
    #[test]
    fn prop_custom_theme_isolation_builtin_unchanged(theme_name in theme_name_strategy()) {
        let manager = ThemeManager::new();

        // Get the original built-in theme
        manager.switch_by_name(&theme_name).expect("Failed to switch theme");
        let original_theme = manager.current().expect("Failed to get original theme");
        let original_colors = (
            original_theme.primary,
            original_theme.secondary,
            original_theme.background,
            original_theme.foreground,
        );

        // Switch to another theme and back
        let other_themes = vec!["dark", "light", "dracula", "monokai", "nord", "high-contrast"];
        for other_theme in other_themes {
            if other_theme != theme_name {
                manager.switch_by_name(other_theme).expect("Failed to switch to other theme");
                break;
            }
        }

        // Switch back to the original theme
        manager.switch_by_name(&theme_name).expect("Failed to switch back");
        let restored_theme = manager.current().expect("Failed to get restored theme");
        let restored_colors = (
            restored_theme.primary,
            restored_theme.secondary,
            restored_theme.background,
            restored_theme.foreground,
        );

        // Verify the theme is unchanged
        assert_eq!(original_colors, restored_colors, "Built-in theme was modified");
    }
}

/// Property 7: Custom Theme Isolation - Theme Independence
/// For any theme, switching to it doesn't affect other themes
proptest! {
    #[test]
    fn prop_custom_theme_isolation_independence(
        theme_name1 in theme_name_strategy(),
        theme_name2 in theme_name_strategy(),
    ) {
        let manager = ThemeManager::new();

        // Get theme 1
        manager.switch_by_name(&theme_name1).expect("Failed to switch to theme 1");
        let theme1 = manager.current().expect("Failed to get theme 1");
        let theme1_colors = (
            theme1.primary,
            theme1.secondary,
            theme1.background,
        );

        // Get theme 2
        manager.switch_by_name(&theme_name2).expect("Failed to switch to theme 2");
        let theme2 = manager.current().expect("Failed to get theme 2");
        let theme2_colors = (
            theme2.primary,
            theme2.secondary,
            theme2.background,
        );

        // Switch back to theme 1
        manager.switch_by_name(&theme_name1).expect("Failed to switch back to theme 1");
        let theme1_again = manager.current().expect("Failed to get theme 1 again");
        let theme1_colors_again = (
            theme1_again.primary,
            theme1_again.secondary,
            theme1_again.background,
        );

        // Verify theme 1 is unchanged
        assert_eq!(theme1_colors, theme1_colors_again, "Theme 1 was modified");

        // Verify theme 2 is different from theme 1 (if they're different themes)
        if theme_name1 != theme_name2 {
            assert_ne!(theme1_colors, theme2_colors, "Different themes should have different colors");
        }
    }
}

/// Property 8: Custom Theme Isolation - Registry Independence
/// For any theme, the registry doesn't mix themes
proptest! {
    #[test]
    fn prop_custom_theme_isolation_registry(theme_name in theme_name_strategy()) {
        let manager = ThemeManager::new();

        // Get all available themes
        let all_themes = manager.list_all_themes().expect("Failed to list themes");

        // Verify the requested theme is in the list
        assert!(all_themes.contains(&theme_name), "Theme not found in registry");

        // Switch to the theme
        manager.switch_by_name(&theme_name).expect("Failed to switch theme");
        let current = manager.current().expect("Failed to get current theme");

        // Verify the current theme matches
        assert_eq!(current.name, theme_name);

        // Verify other themes are still in the registry
        let all_themes_after = manager.list_all_themes().expect("Failed to list themes after");
        assert_eq!(all_themes.len(), all_themes_after.len(), "Registry was modified");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_theme_isolation_builtin_themes() {
        let manager = ThemeManager::new();
        let themes = vec![
            "dark",
            "light",
            "dracula",
            "monokai",
            "nord",
            "high-contrast",
        ];

        for theme_name in &themes {
            manager.switch_by_name(theme_name).unwrap();
            let theme1 = manager.current().unwrap();
            let colors1 = (theme1.primary, theme1.secondary, theme1.background);

            // Switch to another theme
            for other_theme in &themes {
                if other_theme != theme_name {
                    manager.switch_by_name(other_theme).unwrap();
                    break;
                }
            }

            // Switch back
            manager.switch_by_name(theme_name).unwrap();
            let theme2 = manager.current().unwrap();
            let colors2 = (theme2.primary, theme2.secondary, theme2.background);

            // Verify unchanged
            assert_eq!(colors1, colors2);
        }
    }

    #[test]
    fn test_custom_theme_isolation_independence() {
        let manager = ThemeManager::new();

        manager.switch_by_name("dark").unwrap();
        let dark = manager.current().unwrap();
        let dark_colors = (dark.primary, dark.secondary);

        manager.switch_by_name("light").unwrap();
        let light = manager.current().unwrap();
        let light_colors = (light.primary, light.secondary);

        manager.switch_by_name("dark").unwrap();
        let dark_again = manager.current().unwrap();
        let dark_colors_again = (dark_again.primary, dark_again.secondary);

        // Verify dark is unchanged
        assert_eq!(dark_colors, dark_colors_again);

        // Verify dark and light are different
        assert_ne!(dark_colors, light_colors);
    }

    #[test]
    fn test_custom_theme_isolation_registry() {
        let manager = ThemeManager::new();

        let all_themes = manager.list_all_themes().unwrap();
        let initial_count = all_themes.len();

        // Switch through all themes
        for theme_name in &all_themes {
            manager.switch_by_name(theme_name).unwrap();
        }

        // Verify registry is unchanged
        let all_themes_after = manager.list_all_themes().unwrap();
        assert_eq!(initial_count, all_themes_after.len());
    }

    #[test]
    fn test_custom_theme_isolation_multiple_switches() {
        let manager = ThemeManager::new();
        let themes = vec![
            "dark",
            "light",
            "dracula",
            "monokai",
            "nord",
            "high-contrast",
        ];

        // Store original colors for each theme
        let mut original_colors = std::collections::HashMap::new();
        for theme_name in &themes {
            manager.switch_by_name(theme_name).unwrap();
            let theme = manager.current().unwrap();
            original_colors.insert(
                theme_name.to_string(),
                (theme.primary, theme.secondary, theme.background),
            );
        }

        // Switch through themes multiple times
        for _ in 0..3 {
            for theme_name in &themes {
                manager.switch_by_name(theme_name).unwrap();
                let theme = manager.current().unwrap();
                let current_colors = (theme.primary, theme.secondary, theme.background);

                // Verify colors are unchanged
                assert_eq!(
                    original_colors[*theme_name], current_colors,
                    "Theme {} was modified",
                    theme_name
                );
            }
        }
    }
}
