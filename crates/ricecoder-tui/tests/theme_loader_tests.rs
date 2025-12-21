use ricecoder_tui::{ColorSupport, Theme, ThemeLoader, ThemeYaml};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_load_from_string() {
    let yaml_content = r#"name: test
primary: '#0078ff'
secondary: '#5ac8fa'
accent: '#ff2d55'
background: '#111827'
foreground: '#f3f4f6'
error: '#ef4444'
warning: '#f59e0b'
success: '#22c55e'
"#;

    let theme = ThemeLoader::load_from_string(yaml_content).unwrap();
    assert_eq!(theme.name, "test");
    assert_eq!(theme.primary.r, 0);
    assert_eq!(theme.primary.g, 120);
    assert_eq!(theme.primary.b, 255);
}

#[test]
fn test_theme_yaml_to_theme() {
    let theme_yaml = ThemeYaml {
        name: "test".to_string(),
        primary: "#0078ff".to_string(),
        secondary: "#5ac8fa".to_string(),
        accent: "#ff2d55".to_string(),
        background: "#111827".to_string(),
        foreground: "#f3f4f6".to_string(),
        error: "#ef4444".to_string(),
        warning: "#f59e0b".to_string(),
        success: "#22c55e".to_string(),
    };

    let theme = theme_yaml.to_theme().unwrap();
    assert_eq!(theme.name, "test");
    assert_eq!(theme.primary.r, 0);
    assert_eq!(theme.primary.g, 120);
    assert_eq!(theme.primary.b, 255);
}

#[test]
fn test_theme_to_yaml() {
    let theme = Theme::default();
    let yaml = ThemeYaml::from(&theme);
    assert_eq!(yaml.name, theme.name);
    assert_eq!(yaml.primary, theme.primary.to_hex());
}

#[test]
fn test_save_and_load_theme() {
    let temp_dir = TempDir::new().unwrap();
    let theme_path = temp_dir.path().join("test_theme.yaml");

    let theme = Theme::light();
    ThemeLoader::save_to_file(&theme, &theme_path).unwrap();

    let loaded = ThemeLoader::load_from_file(&theme_path).unwrap();
    assert_eq!(loaded.name, theme.name);
    assert_eq!(loaded.primary, theme.primary);
}

#[test]
fn test_load_from_directory() {
    let temp_dir = TempDir::new().unwrap();

    // Save multiple themes
    ThemeLoader::save_to_file(&Theme::default(), &temp_dir.path().join("dark.yaml")).unwrap();
    ThemeLoader::save_to_file(&Theme::light(), &temp_dir.path().join("light.yaml")).unwrap();

    let themes = ThemeLoader::load_from_directory(temp_dir.path()).unwrap();
    assert_eq!(themes.len(), 2);
}

#[test]
fn test_validate_theme_invalid_color() {
    let yaml_content = r#"name: test
primary: 'invalid'
secondary: '#5ac8fa'
accent: '#ff2d55'
background: '#111827'
foreground: '#f3f4f6'
error: '#ef4444'
warning: '#f59e0b'
success: '#22c55e'
"#;

    assert!(ThemeLoader::load_from_string(yaml_content).is_err());
}

#[test]
fn test_validate_theme_empty_name() {
    let yaml_content = r#"name: ''
primary: '#0078ff'
secondary: '#5ac8fa'
accent: '#ff2d55'
background: '#111827'
foreground: '#f3f4f6'
error: '#ef4444'
warning: '#f59e0b'
success: '#22c55e'
"#;

    assert!(ThemeLoader::load_from_string(yaml_content).is_err());
}

#[test]
fn test_load_adapted() {
    let yaml_content = r#"name: test
primary: '#0078ff'
secondary: '#5ac8fa'
accent: '#ff2d55'
background: '#111827'
foreground: '#f3f4f6'
error: '#ef4444'
warning: '#f59e0b'
success: '#22c55e'
"#;

    // Test 16-color adaptation (should map to ANSI 16 colors)
    let theme = ThemeLoader::load_from_string_adapted(yaml_content, ColorSupport::Color16).unwrap();
    assert_eq!(theme.name, "test");
    // Primary #0078ff (Blue-ish) should probably map to Blue (0, 0, 255)
    assert_eq!(theme.primary.r, 0);
    assert_eq!(theme.primary.g, 0);
    assert_eq!(theme.primary.b, 255);
}
