use ricecoder_tui::style::{Color, Theme, TextStyle, ProgressIndicator, ColorSupport};

#[test]
fn test_color_creation() {
    let color = Color::new(255, 128, 64);
    assert_eq!(color.r, 255);
    assert_eq!(color.g, 128);
    assert_eq!(color.b, 64);
}

#[test]
fn test_color_hex() {
    let color = Color::new(255, 128, 64);
    assert_eq!(color.to_hex(), "#ff8040");

    let parsed = Color::from_hex("#ff8040").unwrap();
    assert_eq!(parsed, color);
}

#[test]
fn test_text_style() {
    let color = Color::new(255, 0, 0);
    let style = TextStyle::new().fg(color).bold().underline();
    assert_eq!(style.fg, Some(color));
    assert!(style.bold);
    assert!(style.underline);
    assert!(!style.italic);
}

#[test]
fn test_progress_indicator() {
    let mut progress = ProgressIndicator::new(100);
    assert_eq!(progress.progress, 0);

    progress.update(50);
    assert_eq!(progress.progress, 50);
    assert_eq!(progress.current, 50);

    progress.update(150);
    assert_eq!(progress.current, 100);
    assert_eq!(progress.progress, 100);
}

#[test]
fn test_progress_bar() {
    let mut progress = ProgressIndicator::new(100);
    progress.update(50);
    let bar = progress.bar(10);
    assert_eq!(bar, "[=====     ]");
}

#[test]
fn test_theme_default() {
    let theme = Theme::default();
    assert_eq!(theme.name, "dark");
}

#[test]
fn test_theme_light() {
    let theme = Theme::light();
    assert_eq!(theme.name, "light");
}

#[test]
fn test_theme_monokai() {
    let theme = Theme::monokai();
    assert_eq!(theme.name, "monokai");
}

#[test]
fn test_theme_dracula() {
    let theme = Theme::dracula();
    assert_eq!(theme.name, "dracula");
}

#[test]
fn test_theme_nord() {
    let theme = Theme::nord();
    assert_eq!(theme.name, "nord");
}

#[test]
fn test_color_support_detection() {
    let support = ColorSupport::Color256;
    assert_eq!(support, ColorSupport::Color256);
}

#[test]
fn test_theme_by_name() {
    assert!(Theme::by_name("dark").is_some());
    assert!(Theme::by_name("light").is_some());
    assert!(Theme::by_name("monokai").is_some());
    assert!(Theme::by_name("dracula").is_some());
    assert!(Theme::by_name("nord").is_some());
    assert!(Theme::by_name("catppuccin-mocha").is_some());
    assert!(Theme::by_name("invalid").is_none());
}

#[test]
fn test_theme_by_name_case_insensitive() {
    assert!(Theme::by_name("DARK").is_some());
    assert!(Theme::by_name("Light").is_some());
    assert!(Theme::by_name("MONOKAI").is_some());
    assert!(Theme::by_name("Catppuccin-Mocha").is_some());
}

#[test]
fn test_available_themes() {
    let themes = Theme::available_themes();
    assert!(themes.len() >= 6);
    assert!(themes.contains(&"dark"));
    assert!(themes.contains(&"light"));
    assert!(themes.contains(&"monokai"));
    assert!(themes.contains(&"dracula"));
    assert!(themes.contains(&"nord"));
    assert!(themes.contains(&"high-contrast"));
    assert!(themes.contains(&"catppuccin-mocha"));
}

#[test]
fn test_color_contrast_ratio() {
    let white = Color::new(255, 255, 255);
    let black = Color::new(0, 0, 0);
    let contrast = white.contrast_ratio(&black);
    // White on black should have maximum contrast (21:1)
    assert!(contrast > 20.0);
}

#[test]
fn test_wcag_aa_compliance() {
    let white = Color::new(255, 255, 255);
    let black = Color::new(0, 0, 0);
    assert!(white.meets_wcag_aa(&black));
    assert!(white.meets_wcag_aaa(&black));
}

#[test]
fn test_high_contrast_theme_wcag_compliance() {
    let theme = Theme::high_contrast();
    // High contrast theme should meet at least AA standards
    assert!(theme.meets_wcag_aa());
}

#[test]
fn test_theme_contrast_ratios() {
    let theme = Theme::high_contrast();
    let fg_contrast = theme.foreground_contrast();
    let primary_contrast = theme.primary_contrast();
    let error_contrast = theme.error_contrast();

    // Foreground and primary should meet WCAG AAA standards (7:1)
    assert!(fg_contrast >= 7.0);
    assert!(primary_contrast >= 7.0);
    // Error should at least meet AA standards (4.5:1)
    assert!(error_contrast >= 4.5);
}
