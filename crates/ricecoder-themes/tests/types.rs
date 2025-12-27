use ricecoder_themes::*;

#[test]
fn test_theme_default() {
    let theme = Theme::default();
    assert_eq!(theme.name, "fallback");
    // Test that colors are set
    assert!(matches!(
        theme.primary,
        ratatui::style::Color::Rgb(255, 255, 255)
    ));
}

#[test]
fn test_theme_fallback() {
    let theme = Theme::fallback();
    assert_eq!(theme.name, "fallback");
    assert!(matches!(theme.primary, ratatui::style::Color::Rgb(255, 255, 255)));
}

#[test]
fn test_registry_get_themes() {
    let registry = ThemeRegistry::new();
    // Registry should have at least the fallback theme
    assert!(registry.builtin_count() >= 1);
    
    // Can get themes through registry
    let themes = registry.list_builtin();
    assert!(!themes.is_empty());
}

#[test]
fn test_theme_validate() {
    let mut theme = Theme::default();
    assert!(theme.validate().is_ok());

    theme.name = "".to_string();
    assert!(theme.validate().is_err());
}
