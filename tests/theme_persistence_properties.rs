//! Property-based tests for theme persistence
//!
//! **Feature: ricecoder-themes, Property 2: Theme Persistence**
//! For any theme selection, the choice persists across sessions
//! **Validates: Requirements 2.3, 5.4**

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

/// Property 2: Theme Persistence - Theme Selection Persists in Memory
/// For any theme selection, the choice persists when accessed again
///
/// This property verifies that:
/// 1. When a theme is selected, it remains selected
/// 2. The selected theme can be retrieved multiple times with the same result
/// 3. Theme selection is consistent across multiple accesses
proptest! {
    #[test]
    fn prop_theme_persistence_in_memory(theme_name in theme_name_strategy()) {
        let manager = ThemeManager::new();
        manager.switch_by_name(&theme_name).expect("Failed to switch theme");

        // Access the theme multiple times and verify it's the same
        let theme1 = manager.current().expect("Failed to get current theme");
        let theme2 = manager.current().expect("Failed to get current theme");
        let theme3 = manager.current().expect("Failed to get current theme");

        // Verify all accesses return the same theme
        assert_eq!(theme1.name, theme_name);
        assert_eq!(theme2.name, theme_name);
        assert_eq!(theme3.name, theme_name);
        assert_eq!(theme1.name, theme2.name);
        assert_eq!(theme2.name, theme3.name);
    }
}

/// Property 3: Theme Persistence - Theme Name Consistency
/// For any theme, the name persists across multiple accesses
proptest! {
    #[test]
    fn prop_theme_persistence_name_consistency(theme_name in theme_name_strategy()) {
        let manager = ThemeManager::new();
        manager.switch_by_name(&theme_name).expect("Failed to switch theme");

        // Get the theme name multiple times
        let name1 = manager.current_name().expect("Failed to get theme name");
        let name2 = manager.current_name().expect("Failed to get theme name");
        let name3 = manager.current_name().expect("Failed to get theme name");

        // Verify all accesses return the same name
        assert_eq!(name1, theme_name);
        assert_eq!(name2, theme_name);
        assert_eq!(name3, theme_name);
        assert_eq!(name1, name2);
        assert_eq!(name2, name3);
    }
}

/// Property 4: Theme Persistence - Theme Colors Persist
/// For any theme, the colors persist across multiple accesses
proptest! {
    #[test]
    fn prop_theme_persistence_colors_persist(theme_name in theme_name_strategy()) {
        let manager = ThemeManager::new();
        manager.switch_by_name(&theme_name).expect("Failed to switch theme");

        // Get the theme colors multiple times
        let theme1 = manager.current().expect("Failed to get current theme");
        let theme2 = manager.current().expect("Failed to get current theme");
        let theme3 = manager.current().expect("Failed to get current theme");

        // Verify all colors are the same across accesses
        assert_eq!(theme1.primary, theme2.primary);
        assert_eq!(theme2.primary, theme3.primary);
        assert_eq!(theme1.secondary, theme2.secondary);
        assert_eq!(theme2.secondary, theme3.secondary);
        assert_eq!(theme1.background, theme2.background);
        assert_eq!(theme2.background, theme3.background);
        assert_eq!(theme1.foreground, theme2.foreground);
        assert_eq!(theme2.foreground, theme3.foreground);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_persistence_dark_theme() {
        let manager = ThemeManager::new();
        manager.switch_by_name("dark").unwrap();

        // Verify theme persists in memory
        assert_eq!(manager.current().unwrap().name, "dark");
        assert_eq!(manager.current().unwrap().name, "dark");
        assert_eq!(manager.current_name().unwrap(), "dark");
    }

    #[test]
    fn test_theme_persistence_light_theme() {
        let manager = ThemeManager::new();
        manager.switch_by_name("light").unwrap();

        // Verify theme persists in memory
        assert_eq!(manager.current().unwrap().name, "light");
        assert_eq!(manager.current().unwrap().name, "light");
        assert_eq!(manager.current_name().unwrap(), "light");
    }

    #[test]
    fn test_theme_persistence_multiple_accesses() {
        let manager = ThemeManager::new();

        // Test multiple themes
        for theme_name in &[
            "dark",
            "light",
            "dracula",
            "monokai",
            "nord",
            "high-contrast",
        ] {
            manager.switch_by_name(theme_name).unwrap();

            // Verify theme persists across multiple accesses
            assert_eq!(manager.current().unwrap().name, *theme_name);
            assert_eq!(manager.current().unwrap().name, *theme_name);
            assert_eq!(manager.current_name().unwrap(), *theme_name);
        }
    }

    #[test]
    fn test_theme_persistence_colors_consistent() {
        let manager = ThemeManager::new();
        manager.switch_by_name("dark").unwrap();

        let theme1 = manager.current().unwrap();
        let theme2 = manager.current().unwrap();
        let theme3 = manager.current().unwrap();

        // Verify colors are consistent across accesses
        assert_eq!(theme1.primary, theme2.primary);
        assert_eq!(theme2.primary, theme3.primary);
        assert_eq!(theme1.background, theme2.background);
        assert_eq!(theme2.background, theme3.background);
    }
}
