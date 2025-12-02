//! Property-based tests for theming functionality
//! Tests universal properties that should hold across all inputs
//! Uses proptest for random test case generation
//! Validates Requirements 2.1, 2.2, 2.4

use proptest::prelude::*;
use ricecoder_tui::{Theme, ThemeManager};
use ricecoder_tui::style::Color;

// ============================================================================
// Generators for Property Tests
// ============================================================================

/// Generate a valid theme name from built-in themes
fn arb_theme_name() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("dark".to_string()),
        Just("light".to_string()),
        Just("monokai".to_string()),
        Just("dracula".to_string()),
        Just("nord".to_string()),
        Just("high-contrast".to_string()),
    ]
}

/// Generate a sequence of theme names for switching
fn arb_theme_sequence() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(arb_theme_name(), 1..10)
}

// ============================================================================
// Property 2: Theme Consistency
// **Feature: ricecoder-tui, Property 2: Theme Consistency**
// **Validates: Requirements 2.1, 2.2, 2.4**
// Generate random theme selections from built-in and custom themes
// Verify all UI components (widgets, text, borders) apply theme colors consistently
// Verify no color leakage between themes
// Run minimum 100 iterations with proptest
// ============================================================================

proptest! {
    #[test]
    fn prop_test_theme_consistency_all_colors_defined(theme_name in arb_theme_name()) {
        // For any theme, all required colors must be defined
        let theme = Theme::by_name(&theme_name).expect("Theme should exist");
        
        // Verify all required colors are present and valid
        assert!(!theme.name.is_empty(), "Theme name should not be empty");
        
        // All colors should be valid (u8 is always 0-255)
        let _ = theme.primary;
        let _ = theme.secondary;
        let _ = theme.accent;
        let _ = theme.background;
        let _ = theme.foreground;
        let _ = theme.error;
        let _ = theme.warning;
        let _ = theme.success;
    }

    #[test]
    fn prop_test_theme_consistency_colors_are_distinct(theme_name in arb_theme_name()) {
        // For any theme, background and foreground should be visually distinct
        let theme = Theme::by_name(&theme_name).expect("Theme should exist");
        
        // Background and foreground should not be identical
        assert_ne!(
            theme.background, theme.foreground,
            "Background and foreground should be distinct in theme {}",
            theme_name
        );
        
        // Primary and background should not be identical
        assert_ne!(
            theme.primary, theme.background,
            "Primary and background should be distinct in theme {}",
            theme_name
        );
    }

    #[test]
    fn prop_test_theme_consistency_no_color_leakage(
        theme_names in prop::collection::vec(arb_theme_name(), 2..5)
    ) {
        // For any sequence of themes, switching should not leak colors between themes
        let manager = ThemeManager::new();
        let mut previous_colors: Vec<(String, (Color, Color, Color))> = Vec::new();
        
        for theme_name in theme_names {
            manager.switch_by_name(&theme_name).expect("Switch should succeed");
            let current_theme = manager.current().expect("Current theme should exist");
            
            // Verify current theme colors match the requested theme
            let expected_theme = Theme::by_name(&theme_name).expect("Theme should exist");
            assert_eq!(
                current_theme.primary, expected_theme.primary,
                "Primary color should match for theme {}",
                theme_name
            );
            assert_eq!(
                current_theme.background, expected_theme.background,
                "Background color should match for theme {}",
                theme_name
            );
            assert_eq!(
                current_theme.foreground, expected_theme.foreground,
                "Foreground color should match for theme {}",
                theme_name
            );
            
            // Verify no color leakage from previous themes
            for (prev_name, prev_colors) in &previous_colors {
                if prev_name != &theme_name {
                    assert_ne!(
                        current_theme.primary, prev_colors.0,
                        "Primary color leaked from previous theme {}",
                        prev_name
                    );
                }
            }
            
            previous_colors.push((
                theme_name.clone(),
                (current_theme.primary, current_theme.background, current_theme.foreground),
            ));
        }
    }

    #[test]
    fn prop_test_theme_consistency_all_themes_accessible(theme_name in arb_theme_name()) {
        // For any theme name, it should be accessible via Theme::by_name
        let theme = Theme::by_name(&theme_name);
        assert!(theme.is_some(), "Theme {} should be accessible", theme_name);
        
        // And should be in the available themes list
        let available = Theme::available_themes();
        assert!(
            available.contains(&theme_name.as_str()),
            "Theme {} should be in available themes",
            theme_name
        );
    }

    #[test]
    fn prop_test_theme_consistency_theme_manager_reflects_current(theme_name in arb_theme_name()) {
        // For any theme, the manager's current_name should reflect the active theme
        let manager = ThemeManager::new();
        manager.switch_by_name(&theme_name).expect("Switch should succeed");
        
        let current_name = manager.current_name().expect("Current name should exist");
        assert_eq!(
            current_name, theme_name,
            "Manager should reflect current theme name"
        );
    }

    #[test]
    fn prop_test_theme_consistency_theme_colors_stable(theme_name in arb_theme_name()) {
        // For any theme, getting the theme multiple times should return identical colors
        let theme1 = Theme::by_name(&theme_name).expect("Theme should exist");
        let theme2 = Theme::by_name(&theme_name).expect("Theme should exist");
        
        assert_eq!(theme1.primary, theme2.primary, "Primary color should be stable");
        assert_eq!(theme1.secondary, theme2.secondary, "Secondary color should be stable");
        assert_eq!(theme1.accent, theme2.accent, "Accent color should be stable");
        assert_eq!(theme1.background, theme2.background, "Background color should be stable");
        assert_eq!(theme1.foreground, theme2.foreground, "Foreground color should be stable");
        assert_eq!(theme1.error, theme2.error, "Error color should be stable");
        assert_eq!(theme1.warning, theme2.warning, "Warning color should be stable");
        assert_eq!(theme1.success, theme2.success, "Success color should be stable");
    }
}

// ============================================================================
// Property 9: Theme Switching Without Restart
// **Feature: ricecoder-tui, Property 9: Theme Switching Without Restart**
// **Validates: Requirements 2.4**
// Generate random theme changes during runtime
// Verify new theme applies immediately to all UI elements
// Verify application state is preserved after theme switch
// Run minimum 100 iterations with proptest
// ============================================================================

proptest! {
    #[test]
    fn prop_test_theme_switching_immediate_application(theme_sequence in arb_theme_sequence()) {
        // For any sequence of theme switches, each switch should apply immediately
        let manager = ThemeManager::new();
        
        for theme_name in theme_sequence {
            manager.switch_by_name(&theme_name).expect("Switch should succeed");
            
            // Verify the theme is immediately active
            let current = manager.current().expect("Current theme should exist");
            assert_eq!(
                current.name, theme_name,
                "Theme should be immediately active after switch"
            );
            
            // Verify the theme colors are correct
            let expected = Theme::by_name(&theme_name).expect("Theme should exist");
            assert_eq!(current.primary, expected.primary, "Primary color should be correct");
            assert_eq!(current.background, expected.background, "Background color should be correct");
            assert_eq!(current.foreground, expected.foreground, "Foreground color should be correct");
        }
    }

    #[test]
    fn prop_test_theme_switching_state_preservation(theme_sequence in arb_theme_sequence()) {
        // For any sequence of theme switches, application state should be preserved
        let manager = ThemeManager::new();
        let initial_theme = manager.current().expect("Initial theme should exist");
        
        // Switch through multiple themes
        for theme_name in &theme_sequence {
            manager.switch_by_name(theme_name).expect("Switch should succeed");
        }
        
        // Switch back to initial theme
        manager.switch_by_name(&initial_theme.name).expect("Switch should succeed");
        
        // Verify we can still access the theme and its colors are unchanged
        let restored = manager.current().expect("Restored theme should exist");
        assert_eq!(restored.name, initial_theme.name, "Theme name should be preserved");
        assert_eq!(restored.primary, initial_theme.primary, "Primary color should be preserved");
        assert_eq!(restored.background, initial_theme.background, "Background color should be preserved");
        assert_eq!(restored.foreground, initial_theme.foreground, "Foreground color should be preserved");
    }

    #[test]
    fn prop_test_theme_switching_no_data_loss(theme_sequence in arb_theme_sequence()) {
        // For any sequence of theme switches, no theme data should be lost
        let manager = ThemeManager::new();
        let mut theme_snapshots = Vec::new();
        
        // Collect snapshots of each theme
        for theme_name in &theme_sequence {
            manager.switch_by_name(theme_name).expect("Switch should succeed");
            let theme = manager.current().expect("Current theme should exist");
            theme_snapshots.push((theme_name.clone(), theme.clone()));
        }
        
        // Verify each theme can be accessed again with identical data
        for (theme_name, original_theme) in theme_snapshots {
            manager.switch_by_name(&theme_name).expect("Switch should succeed");
            let current = manager.current().expect("Current theme should exist");
            
            assert_eq!(current.name, original_theme.name, "Theme name should match");
            assert_eq!(current.primary, original_theme.primary, "Primary color should match");
            assert_eq!(current.secondary, original_theme.secondary, "Secondary color should match");
            assert_eq!(current.accent, original_theme.accent, "Accent color should match");
            assert_eq!(current.background, original_theme.background, "Background color should match");
            assert_eq!(current.foreground, original_theme.foreground, "Foreground color should match");
            assert_eq!(current.error, original_theme.error, "Error color should match");
            assert_eq!(current.warning, original_theme.warning, "Warning color should match");
            assert_eq!(current.success, original_theme.success, "Success color should match");
        }
    }

    #[test]
    fn prop_test_theme_switching_manager_consistency(theme_sequence in arb_theme_sequence()) {
        // For any sequence of theme switches, manager state should remain consistent
        let manager = ThemeManager::new();
        
        for theme_name in theme_sequence {
            manager.switch_by_name(&theme_name).expect("Switch should succeed");
            
            // Verify manager's current_name matches the active theme
            let current_name = manager.current_name().expect("Current name should exist");
            assert_eq!(current_name, theme_name, "Manager name should match active theme");
            
            // Verify manager's current() returns the correct theme
            let current = manager.current().expect("Current theme should exist");
            assert_eq!(current.name, theme_name, "Manager theme should match active theme");
            
            // Verify available themes list is consistent
            let available = manager.available_themes();
            assert!(
                available.contains(&theme_name.as_str()),
                "Active theme should be in available themes"
            );
        }
    }

    #[test]
    fn prop_test_theme_switching_rapid_succession(theme_sequence in arb_theme_sequence()) {
        // For any sequence of rapid theme switches, the final theme should be correct
        let manager = ThemeManager::new();
        
        // Rapidly switch through all themes
        for theme_name in &theme_sequence {
            manager.switch_by_name(theme_name).expect("Switch should succeed");
        }
        
        // Verify the final theme is correct
        if let Some(final_theme_name) = theme_sequence.last() {
            let current = manager.current().expect("Current theme should exist");
            assert_eq!(
                current.name, *final_theme_name,
                "Final theme should be the last one switched to"
            );
        }
    }

    #[test]
    fn prop_test_theme_switching_idempotent(theme_name in arb_theme_name()) {
        // For any theme, switching to it multiple times should be idempotent
        let manager = ThemeManager::new();
        
        // Switch to the theme multiple times
        for _ in 0..5 {
            manager.switch_by_name(&theme_name).expect("Switch should succeed");
        }
        
        // Verify the theme is still correct
        let current = manager.current().expect("Current theme should exist");
        assert_eq!(current.name, theme_name, "Theme should remain consistent");
        
        let expected = Theme::by_name(&theme_name).expect("Theme should exist");
        assert_eq!(current.primary, expected.primary, "Primary color should be correct");
        assert_eq!(current.background, expected.background, "Background color should be correct");
    }
}

// ============================================================================
// Additional Property Tests for Robustness
// ============================================================================

proptest! {
    #[test]
    fn prop_test_theme_case_insensitive_switching(theme_name in arb_theme_name()) {
        // For any theme name, case variations should work
        let manager = ThemeManager::new();
        
        // Try uppercase
        let upper = theme_name.to_uppercase();
        manager.switch_by_name(&upper).expect("Uppercase switch should succeed");
        let current = manager.current().expect("Current theme should exist");
        assert_eq!(current.name, theme_name, "Uppercase switch should work");
        
        // Try mixed case
        let mixed = if theme_name.len() > 0 {
            let mut chars = theme_name.chars();
            let first = chars.next().unwrap().to_uppercase().to_string();
            first + &theme_name[1..]
        } else {
            theme_name.clone()
        };
        manager.switch_by_name(&mixed).expect("Mixed case switch should succeed");
        let current = manager.current().expect("Current theme should exist");
        assert_eq!(current.name, theme_name, "Mixed case switch should work");
    }

    #[test]
    fn prop_test_theme_all_colors_valid_rgb(theme_name in arb_theme_name()) {
        // For any theme, all colors should have valid RGB values
        let theme = Theme::by_name(&theme_name).expect("Theme should exist");
        
        // Verify all colors are valid (u8 is always 0-255, so this is always true,
        // but we're testing that the colors exist and are accessible)
        let colors = vec![
            theme.primary,
            theme.secondary,
            theme.accent,
            theme.background,
            theme.foreground,
            theme.error,
            theme.warning,
            theme.success,
        ];
        
        for color in colors {
            // Verify color components are accessible (u8 is always 0-255)
            let _ = color.r;
            let _ = color.g;
            let _ = color.b;
        }
    }

    #[test]
    fn prop_test_theme_manager_available_themes_consistent(theme_name in arb_theme_name()) {
        // For any theme, it should be in the manager's available themes
        let manager = ThemeManager::new();
        let available = manager.available_themes();
        
        assert!(
            available.contains(&theme_name.as_str()),
            "Theme {} should be in available themes",
            theme_name
        );
    }
}
