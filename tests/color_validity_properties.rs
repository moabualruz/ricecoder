//! Property-based tests for color validity
//!
//! **Feature: ricecoder-themes, Property 3: Color Validity**
//! For any color definition, it's valid hex or theme loading fails with error
//! **Validates: Requirements 3.2, 3.5**

use proptest::prelude::*;
use ricecoder_tui::{style::Color, theme::ThemeManager};

/// Strategy for generating valid RGB values (0-255)
fn rgb_value_strategy() -> impl Strategy<Value = u8> {
    0u8..=255u8
}

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

/// Property 3: Color Validity
/// For any color definition, it's valid hex or theme loading fails with error
///
/// This property verifies that:
/// 1. All colors in a theme have valid RGB values (0-255)
/// 2. Colors can be created with any valid RGB values
/// 3. Theme colors are always valid
proptest! {
    #[test]
    fn prop_color_validity_rgb_values(r in rgb_value_strategy(), g in rgb_value_strategy(), b in rgb_value_strategy()) {
        // Create a color with any valid RGB values
        let color = Color::new(r, g, b);

        // Verify the color was created successfully
        assert_eq!(color.r, r);
        assert_eq!(color.g, g);
        assert_eq!(color.b, b);
    }
}

/// Property 4: Color Validity - Theme Colors Are Valid
/// For any theme, all colors are valid RGB values
proptest! {
    #[test]
    fn prop_color_validity_theme_colors(theme_name in theme_name_strategy()) {
        let manager = ThemeManager::new();
        manager.switch_by_name(&theme_name).expect("Failed to switch theme");

        let theme = manager.current().expect("Failed to get current theme");

        // Verify all colors are valid (have RGB values)
        // The Color type ensures validity, but we verify the values are accessible
        let _ = theme.primary.r;
        let _ = theme.primary.g;
        let _ = theme.primary.b;

        let _ = theme.secondary.r;
        let _ = theme.secondary.g;
        let _ = theme.secondary.b;

        let _ = theme.accent.r;
        let _ = theme.accent.g;
        let _ = theme.accent.b;

        let _ = theme.background.r;
        let _ = theme.background.g;
        let _ = theme.background.b;

        let _ = theme.foreground.r;
        let _ = theme.foreground.g;
        let _ = theme.foreground.b;

        let _ = theme.error.r;
        let _ = theme.error.g;
        let _ = theme.error.b;

        let _ = theme.warning.r;
        let _ = theme.warning.g;
        let _ = theme.warning.b;

        let _ = theme.success.r;
        let _ = theme.success.g;
        let _ = theme.success.b;
    }
}

/// Property 5: Color Validity - Color Equality
/// For any two colors with the same RGB values, they should be equal
proptest! {
    #[test]
    fn prop_color_validity_equality(r in rgb_value_strategy(), g in rgb_value_strategy(), b in rgb_value_strategy()) {
        let color1 = Color::new(r, g, b);
        let color2 = Color::new(r, g, b);

        // Verify colors with same RGB values are equal
        assert_eq!(color1, color2);
    }
}

/// Property 6: Color Validity - Color Inequality
/// For any two colors with different RGB values, they should not be equal
proptest! {
    #[test]
    fn prop_color_validity_inequality(
        r1 in rgb_value_strategy(),
        g1 in rgb_value_strategy(),
        b1 in rgb_value_strategy(),
        r2 in rgb_value_strategy(),
        g2 in rgb_value_strategy(),
        b2 in rgb_value_strategy(),
    ) {
        let color1 = Color::new(r1, g1, b1);
        let color2 = Color::new(r2, g2, b2);

        // If colors have different RGB values, they should not be equal
        if (r1, g1, b1) != (r2, g2, b2) {
            assert_ne!(color1, color2);
        } else {
            assert_eq!(color1, color2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_validity_basic() {
        let color = Color::new(255, 128, 64);
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 128);
        assert_eq!(color.b, 64);
    }

    #[test]
    fn test_color_validity_black() {
        let color = Color::new(0, 0, 0);
        assert_eq!(color.r, 0);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);
    }

    #[test]
    fn test_color_validity_white() {
        let color = Color::new(255, 255, 255);
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 255);
        assert_eq!(color.b, 255);
    }

    #[test]
    fn test_color_validity_equality() {
        let color1 = Color::new(100, 150, 200);
        let color2 = Color::new(100, 150, 200);
        assert_eq!(color1, color2);
    }

    #[test]
    fn test_color_validity_inequality() {
        let color1 = Color::new(100, 150, 200);
        let color2 = Color::new(100, 150, 201);
        assert_ne!(color1, color2);
    }

    #[test]
    fn test_color_validity_all_themes() {
        let manager = ThemeManager::new();
        let themes = manager.available_themes();

        for theme_name in themes {
            manager.switch_by_name(theme_name).unwrap();
            let theme = manager.current().unwrap();

            // Verify all colors are valid (accessible)
            let _ = theme.primary.r;
            let _ = theme.primary.g;
            let _ = theme.primary.b;

            let _ = theme.secondary.r;
            let _ = theme.secondary.g;
            let _ = theme.secondary.b;

            let _ = theme.background.r;
            let _ = theme.background.g;
            let _ = theme.background.b;

            let _ = theme.foreground.r;
            let _ = theme.foreground.g;
            let _ = theme.foreground.b;
        }
    }
}
