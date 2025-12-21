use ricecoder_themes::*;

#[test]
fn test_theme_default() {
    let theme = Theme::default();
    assert_eq!(theme.name, "dark");
    // Test that colors are set
    assert!(matches!(
        theme.primary,
        ratatui::style::Color::Rgb(255, 255, 255)
    ));
}

#[test]
fn test_theme_light() {
    let theme = Theme::light();
    assert_eq!(theme.name, "light");
    assert!(matches!(theme.primary, ratatui::style::Color::Rgb(0, 0, 0)));
}

#[test]
fn test_theme_by_name() {
    assert!(Theme::by_name("dark").is_some());
    assert!(Theme::by_name("light").is_some());
    assert!(Theme::by_name("invalid").is_none());
}

#[test]
fn test_theme_available_themes() {
    let themes = Theme::available_themes();
    assert!(themes.contains(&"dark"));
    assert!(themes.contains(&"light"));
}

#[test]
fn test_theme_validate() {
    let mut theme = Theme::default();
    assert!(theme.validate().is_ok());

    theme.name = "".to_string();
    assert!(theme.validate().is_err());
}
