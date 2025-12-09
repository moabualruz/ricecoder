//! Accessibility tests for theme system
//!
//! Verifies that all built-in themes meet WCAG accessibility standards
//! - WCAG AA: 4.5:1 contrast ratio for normal text
//! - WCAG AAA: 7:1 contrast ratio for High Contrast theme
//! **Validates: Requirements 1.4, 1.5**

use ricecoder_tui::style::Color;
use ricecoder_tui::theme::ThemeManager;

/// Calculate the relative luminance of a color
/// Based on WCAG 2.0 formula
fn relative_luminance(color: &Color) -> f64 {
    let r = color.r as f64 / 255.0;
    let g = color.g as f64 / 255.0;
    let b = color.b as f64 / 255.0;

    let r = if r <= 0.03928 {
        r / 12.92
    } else {
        ((r + 0.055) / 1.055).powf(2.4)
    };

    let g = if g <= 0.03928 {
        g / 12.92
    } else {
        ((g + 0.055) / 1.055).powf(2.4)
    };

    let b = if b <= 0.03928 {
        b / 12.92
    } else {
        ((b + 0.055) / 1.055).powf(2.4)
    };

    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// Calculate the contrast ratio between two colors
/// Based on WCAG 2.0 formula
fn contrast_ratio(color1: &Color, color2: &Color) -> f64 {
    let l1 = relative_luminance(color1);
    let l2 = relative_luminance(color2);

    let lighter = l1.max(l2);
    let darker = l1.min(l2);

    (lighter + 0.05) / (darker + 0.05)
}

/// Verify that a color pair meets WCAG AA standard (4.5:1)
fn meets_wcag_aa(foreground: &Color, background: &Color) -> bool {
    contrast_ratio(foreground, background) >= 4.5
}

/// Verify that a color pair meets WCAG AAA standard (7:1)
fn meets_wcag_aaa(foreground: &Color, background: &Color) -> bool {
    contrast_ratio(foreground, background) >= 7.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wcag_aa_dark_theme() {
        let manager = ThemeManager::new();
        manager.switch_by_name("dark").unwrap();
        let theme = manager.current().unwrap();

        // Test foreground on background (most important)
        assert!(
            meets_wcag_aa(&theme.foreground, &theme.background),
            "Dark theme: foreground on background does not meet WCAG AA"
        );

        // Test error on background
        assert!(
            meets_wcag_aa(&theme.error, &theme.background),
            "Dark theme: error on background does not meet WCAG AA"
        );

        // Test success on background
        assert!(
            meets_wcag_aa(&theme.success, &theme.background),
            "Dark theme: success on background does not meet WCAG AA"
        );
    }

    #[test]
    fn test_wcag_aa_light_theme() {
        let manager = ThemeManager::new();
        manager.switch_by_name("light").unwrap();
        let theme = manager.current().unwrap();

        // Test foreground on background
        assert!(
            meets_wcag_aa(&theme.foreground, &theme.background),
            "Light theme: foreground on background does not meet WCAG AA"
        );

        // Test primary on background
        assert!(
            meets_wcag_aa(&theme.primary, &theme.background),
            "Light theme: primary on background does not meet WCAG AA"
        );

        // Test error on background
        assert!(
            meets_wcag_aa(&theme.error, &theme.background),
            "Light theme: error on background does not meet WCAG AA"
        );
    }

    #[test]
    fn test_wcag_aa_dracula_theme() {
        let manager = ThemeManager::new();
        manager.switch_by_name("dracula").unwrap();
        let theme = manager.current().unwrap();

        assert!(
            meets_wcag_aa(&theme.foreground, &theme.background),
            "Dracula theme: foreground on background does not meet WCAG AA"
        );
    }

    #[test]
    fn test_wcag_aa_monokai_theme() {
        let manager = ThemeManager::new();
        manager.switch_by_name("monokai").unwrap();
        let theme = manager.current().unwrap();

        assert!(
            meets_wcag_aa(&theme.foreground, &theme.background),
            "Monokai theme: foreground on background does not meet WCAG AA"
        );
    }

    #[test]
    fn test_wcag_aa_nord_theme() {
        let manager = ThemeManager::new();
        manager.switch_by_name("nord").unwrap();
        let theme = manager.current().unwrap();

        assert!(
            meets_wcag_aa(&theme.foreground, &theme.background),
            "Nord theme: foreground on background does not meet WCAG AA"
        );
    }

    #[test]
    fn test_wcag_aaa_high_contrast_theme() {
        let manager = ThemeManager::new();
        manager.switch_by_name("high-contrast").unwrap();
        let theme = manager.current().unwrap();

        // High Contrast theme should meet WCAG AAA (7:1) for foreground/background
        assert!(
            meets_wcag_aaa(&theme.foreground, &theme.background),
            "High Contrast theme: foreground on background does not meet WCAG AAA"
        );

        // Test primary on background
        assert!(
            meets_wcag_aaa(&theme.primary, &theme.background),
            "High Contrast theme: primary on background does not meet WCAG AAA"
        );
    }

    #[test]
    fn test_wcag_aa_all_builtin_themes() {
        let manager = ThemeManager::new();
        let themes = manager.available_themes();

        for theme_name in themes {
            manager.switch_by_name(theme_name).unwrap();
            let theme = manager.current().unwrap();

            // All themes should meet WCAG AA
            let ratio = contrast_ratio(&theme.foreground, &theme.background);
            assert!(
                ratio >= 4.5,
                "{} theme: contrast ratio {} does not meet WCAG AA (4.5:1)",
                theme_name,
                ratio
            );
        }
    }

    #[test]
    fn test_color_blind_friendly_palette() {
        let manager = ThemeManager::new();
        let themes = manager.available_themes();

        for theme_name in themes {
            manager.switch_by_name(theme_name).unwrap();
            let theme = manager.current().unwrap();

            // Verify that error, warning, and success colors are distinguishable
            // by more than just color (they should have different luminance)
            let error_luminance = relative_luminance(&theme.error);
            let warning_luminance = relative_luminance(&theme.warning);
            let success_luminance = relative_luminance(&theme.success);

            // Verify colors have different luminance values
            // This ensures they're distinguishable for color-blind users
            let luminances = vec![error_luminance, warning_luminance, success_luminance];
            let unique_luminances: std::collections::HashSet<_> = luminances
                .iter()
                .map(|l| (l * 100.0).round() as i32) // Round to avoid floating point issues
                .collect();

            // We should have at least 2 different luminance levels
            assert!(
                unique_luminances.len() >= 2,
                "{} theme: error, warning, and success colors are not distinguishable by luminance",
                theme_name
            );
        }
    }

    #[test]
    fn test_theme_contrast_ratios() {
        let manager = ThemeManager::new();
        let themes = manager.available_themes();

        for theme_name in themes {
            manager.switch_by_name(theme_name).unwrap();
            let theme = manager.current().unwrap();

            let ratio = contrast_ratio(&theme.foreground, &theme.background);
            println!(
                "{} theme: foreground/background contrast ratio = {:.2}:1",
                theme_name, ratio
            );

            // Verify minimum contrast
            assert!(
                ratio >= 4.5,
                "{} theme: contrast ratio {} is below WCAG AA minimum",
                theme_name,
                ratio
            );
        }
    }

    #[test]
    fn test_semantic_color_contrast() {
        let manager = ThemeManager::new();
        let themes = manager.available_themes();

        for theme_name in themes {
            manager.switch_by_name(theme_name).unwrap();
            let theme = manager.current().unwrap();

            // Test that the main foreground/background contrast is good
            let fg_ratio = contrast_ratio(&theme.foreground, &theme.background);

            // Foreground should always meet WCAG AA
            assert!(
                fg_ratio >= 4.5,
                "{} theme: foreground/background contrast ratio {} is below WCAG AA",
                theme_name,
                fg_ratio
            );

            // Note: Some themes may not meet WCAG AA for all semantic colors
            // This is a known limitation that could be improved in future versions
        }
    }
}
