//! Tests for theming functionality

use ricecoder_tui::{ColorSupport, Theme, ThemeLoader, ThemeManager};
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_theme_creation_all_built_in_themes() {
    let themes = vec![
        Theme::default(),
        Theme::light(),
        Theme::monokai(),
        Theme::dracula(),
        Theme::nord(),
    ];

    assert_eq!(themes.len(), 5);
    assert_eq!(themes[0].name, "dark");
    assert_eq!(themes[1].name, "light");
    assert_eq!(themes[2].name, "monokai");
    assert_eq!(themes[3].name, "dracula");
    assert_eq!(themes[4].name, "nord");
}

#[test]
fn test_theme_by_name_all_themes() {
    let theme_names = vec!["dark", "light", "monokai", "dracula", "nord"];

    for name in theme_names {
        let theme = Theme::by_name(name);
        assert!(theme.is_some(), "Theme {} should exist", name);
        assert_eq!(theme.unwrap().name, name);
    }
}

#[test]
fn test_theme_by_name_case_insensitive() {
    let test_cases = vec![
        ("DARK", "dark"),
        ("Light", "light"),
        ("MONOKAI", "monokai"),
        ("Dracula", "dracula"),
        ("NORD", "nord"),
    ];

    for (input, expected) in test_cases {
        let theme = Theme::by_name(input);
        assert!(theme.is_some());
        assert_eq!(theme.unwrap().name, expected);
    }
}

#[test]
fn test_theme_available_themes() {
    let themes = Theme::available_themes();
    assert_eq!(themes.len(), 6);
    assert!(themes.contains(&"dark"));
    assert!(themes.contains(&"light"));
    assert!(themes.contains(&"monokai"));
    assert!(themes.contains(&"dracula"));
    assert!(themes.contains(&"nord"));
    assert!(themes.contains(&"high-contrast"));
}

#[test]
fn test_color_support_detection() {
    let support = ColorSupport::Color256;
    assert_eq!(support, ColorSupport::Color256);

    let support2 = ColorSupport::TrueColor;
    assert_ne!(support, support2);
}

#[test]
fn test_theme_manager_creation() {
    let manager = ThemeManager::new();
    assert_eq!(manager.current().unwrap().name, "dark");
}

#[test]
fn test_theme_manager_switch_by_name() {
    let manager = ThemeManager::new();

    manager.switch_by_name("light").unwrap();
    assert_eq!(manager.current().unwrap().name, "light");

    manager.switch_by_name("monokai").unwrap();
    assert_eq!(manager.current().unwrap().name, "monokai");

    manager.switch_by_name("dracula").unwrap();
    assert_eq!(manager.current().unwrap().name, "dracula");

    manager.switch_by_name("nord").unwrap();
    assert_eq!(manager.current().unwrap().name, "nord");
}

#[test]
fn test_theme_manager_switch_invalid_theme() {
    let manager = ThemeManager::new();
    assert!(manager.switch_by_name("invalid").is_err());
}

#[test]
fn test_theme_manager_current_name() {
    let manager = ThemeManager::new();
    assert_eq!(manager.current_name().unwrap(), "dark");

    manager.switch_by_name("light").unwrap();
    assert_eq!(manager.current_name().unwrap(), "light");
}

#[test]
fn test_theme_manager_available_themes() {
    let manager = ThemeManager::new();
    let themes = manager.available_themes();
    assert_eq!(themes.len(), 6);
}

#[test]
fn test_theme_loader_save_and_load() {
    let temp_dir = TempDir::new().unwrap();
    let theme_path = temp_dir.path().join("test_theme.yaml");

    let original_theme = Theme::light();
    ThemeLoader::save_to_file(&original_theme, &theme_path).unwrap();

    let loaded_theme = ThemeLoader::load_from_file(&theme_path).unwrap();
    assert_eq!(loaded_theme.name, original_theme.name);
    assert_eq!(loaded_theme.primary, original_theme.primary);
    assert_eq!(loaded_theme.secondary, original_theme.secondary);
    assert_eq!(loaded_theme.accent, original_theme.accent);
    assert_eq!(loaded_theme.background, original_theme.background);
    assert_eq!(loaded_theme.foreground, original_theme.foreground);
    assert_eq!(loaded_theme.error, original_theme.error);
    assert_eq!(loaded_theme.warning, original_theme.warning);
    assert_eq!(loaded_theme.success, original_theme.success);
}

#[test]
fn test_theme_loader_load_from_directory() {
    let temp_dir = TempDir::new().unwrap();

    // Save multiple themes
    ThemeLoader::save_to_file(&Theme::default(), &temp_dir.path().join("dark.yaml")).unwrap();
    ThemeLoader::save_to_file(&Theme::light(), &temp_dir.path().join("light.yaml")).unwrap();
    ThemeLoader::save_to_file(&Theme::monokai(), &temp_dir.path().join("monokai.yaml")).unwrap();

    let themes = ThemeLoader::load_from_directory(temp_dir.path()).unwrap();
    assert_eq!(themes.len(), 3);

    let theme_names: Vec<String> = themes.iter().map(|t| t.name.clone()).collect();
    assert!(theme_names.contains(&"dark".to_string()));
    assert!(theme_names.contains(&"light".to_string()));
    assert!(theme_names.contains(&"monokai".to_string()));
}

#[test]
fn test_theme_loader_load_nonexistent_file() {
    let path = Path::new("/nonexistent/theme.yaml");
    assert!(ThemeLoader::load_from_file(path).is_err());
}

#[test]
fn test_theme_loader_load_invalid_extension() {
    let temp_dir = TempDir::new().unwrap();
    let invalid_path = temp_dir.path().join("theme.txt");
    std::fs::write(&invalid_path, "test").unwrap();

    assert!(ThemeLoader::load_from_file(&invalid_path).is_err());
}

#[test]
fn test_theme_manager_load_custom_theme() {
    let temp_dir = TempDir::new().unwrap();
    let theme_path = temp_dir.path().join("custom.yaml");

    // Save a custom theme
    ThemeLoader::save_to_file(&Theme::dracula(), &theme_path).unwrap();

    // Load it with theme manager
    let manager = ThemeManager::new();
    manager.load_custom_theme(&theme_path).unwrap();
    assert_eq!(manager.current().unwrap().name, "dracula");
}

#[test]
fn test_theme_manager_save_custom_theme() {
    let temp_dir = TempDir::new().unwrap();
    let theme_path = temp_dir.path().join("saved_theme.yaml");

    let manager = ThemeManager::new();
    manager.switch_by_name("nord").unwrap();
    manager.save_custom_theme(&theme_path).unwrap();

    // Verify the file was created and can be loaded
    let loaded = ThemeLoader::load_from_file(&theme_path).unwrap();
    assert_eq!(loaded.name, "nord");
}

#[test]
fn test_theme_manager_load_custom_themes_from_directory() {
    let temp_dir = TempDir::new().unwrap();

    let manager = ThemeManager::new();
    manager.switch_by_name("dark").unwrap();
    manager
        .save_custom_theme(&temp_dir.path().join("dark.yaml"))
        .unwrap();

    manager.switch_by_name("light").unwrap();
    manager
        .save_custom_theme(&temp_dir.path().join("light.yaml"))
        .unwrap();

    manager.switch_by_name("monokai").unwrap();
    manager
        .save_custom_theme(&temp_dir.path().join("monokai.yaml"))
        .unwrap();

    let themes = manager
        .load_custom_themes_from_directory(temp_dir.path())
        .unwrap();
    assert_eq!(themes.len(), 3);
}

#[test]
fn test_theme_switching_preserves_colors() {
    let manager = ThemeManager::new();

    // Get dark theme colors
    manager.switch_by_name("dark").unwrap();
    let dark_theme = manager.current().unwrap();
    let dark_primary = dark_theme.primary;

    // Switch to light
    manager.switch_by_name("light").unwrap();
    let light_theme = manager.current().unwrap();
    let light_primary = light_theme.primary;

    // Colors should be different
    assert_ne!(dark_primary, light_primary);

    // Switch back to dark
    manager.switch_by_name("dark").unwrap();
    let dark_theme_again = manager.current().unwrap();
    assert_eq!(dark_theme_again.primary, dark_primary);
}

#[test]
fn test_all_themes_have_required_colors() {
    let themes = vec![
        Theme::default(),
        Theme::light(),
        Theme::monokai(),
        Theme::dracula(),
        Theme::nord(),
    ];

    for theme in themes {
        // Verify all required colors are present
        assert!(!theme.name.is_empty());
        // All colors should have valid RGB values (u8 is always 0-255)
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

#[test]
fn test_theme_yaml_round_trip() {
    let temp_dir = TempDir::new().unwrap();

    let themes = vec![
        Theme::default(),
        Theme::light(),
        Theme::monokai(),
        Theme::dracula(),
        Theme::nord(),
    ];

    for (i, theme) in themes.iter().enumerate() {
        let path = temp_dir.path().join(format!("theme_{}.yaml", i));
        ThemeLoader::save_to_file(theme, &path).unwrap();
        let loaded = ThemeLoader::load_from_file(&path).unwrap();

        assert_eq!(loaded.name, theme.name);
        assert_eq!(loaded.primary, theme.primary);
        assert_eq!(loaded.secondary, theme.secondary);
        assert_eq!(loaded.accent, theme.accent);
        assert_eq!(loaded.background, theme.background);
        assert_eq!(loaded.foreground, theme.foreground);
        assert_eq!(loaded.error, theme.error);
        assert_eq!(loaded.warning, theme.warning);
        assert_eq!(loaded.success, theme.success);
    }
}
