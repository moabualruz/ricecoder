use ricecoder_tui::{Theme, ThemeRegistry};

#[test]
fn test_registry_creation() {
    let registry = ThemeRegistry::new();
    assert_eq!(registry.builtin_count(), Theme::available_themes().len());
    assert_eq!(registry.custom_count().unwrap(), 0);
}

#[test]
fn test_get_builtin_theme() {
    let registry = ThemeRegistry::new();
    assert!(registry.get("dark").is_some());
    assert!(registry.get("light").is_some());
    assert!(registry.get("monokai").is_some());
    assert!(registry.get("dracula").is_some());
    assert!(registry.get("nord").is_some());
    assert!(registry.get("high-contrast").is_some());
}

#[test]
fn test_get_nonexistent_theme() {
    let registry = ThemeRegistry::new();
    assert!(registry.get("nonexistent").is_none());
}

#[test]
fn test_register_custom_theme() {
    let registry = ThemeRegistry::new();
    let custom_theme = Theme::light();
    let mut custom = custom_theme.clone();
    custom.name = "my-custom".to_string();

    registry.register(custom).unwrap();
    assert_eq!(registry.custom_count().unwrap(), 1);
    assert!(registry.get("my-custom").is_some());
}

#[test]
fn test_unregister_custom_theme() {
    let registry = ThemeRegistry::new();
    let custom_theme = Theme::light();
    let mut custom = custom_theme.clone();
    custom.name = "my-custom".to_string();

    registry.register(custom).unwrap();
    assert_eq!(registry.custom_count().unwrap(), 1);

    registry.unregister("my-custom").unwrap();
    assert_eq!(registry.custom_count().unwrap(), 0);
}

#[test]
fn test_list_all_themes() {
    let registry = ThemeRegistry::new();
    let custom_theme = Theme::light();
    let mut custom = custom_theme.clone();
    custom.name = "my-custom".to_string();
    registry.register(custom).unwrap();

    let all = registry.list_all().unwrap();
    assert_eq!(all.len(), Theme::available_themes().len() + 1);
}

#[test]
fn test_list_builtin_themes() {
    let registry = ThemeRegistry::new();
    let builtin = registry.list_builtin();
    assert_eq!(builtin.len(), Theme::available_themes().len());
}

#[test]
fn test_list_custom_themes() {
    let registry = ThemeRegistry::new();
    let custom_theme = Theme::light();
    let mut custom = custom_theme.clone();
    custom.name = "my-custom".to_string();
    registry.register(custom).unwrap();

    let custom_list = registry.list_custom().unwrap();
    assert_eq!(custom_list.len(), 1);
}

#[test]
fn test_exists() {
    let registry = ThemeRegistry::new();
    assert!(registry.exists("dark"));
    assert!(!registry.exists("nonexistent"));
}

#[test]
fn test_is_builtin() {
    let registry = ThemeRegistry::new();
    assert!(registry.is_builtin("dark"));
    assert!(!registry.is_builtin("nonexistent"));
}

#[test]
fn test_is_custom() {
    let registry = ThemeRegistry::new();
    let custom_theme = Theme::light();
    let mut custom = custom_theme.clone();
    custom.name = "my-custom".to_string();
    registry.register(custom).unwrap();

    assert!(registry.is_custom("my-custom").unwrap());
    assert!(!registry.is_custom("dark").unwrap());
}

#[test]
fn test_reset_to_default() {
    let registry = ThemeRegistry::new();
    let custom_theme = Theme::light();
    let mut custom = custom_theme.clone();
    custom.name = "dark".to_string();
    registry.register(custom).unwrap();

    registry.reset_to_default("dark").unwrap();
    let theme = registry.get("dark").unwrap();
    assert_eq!(theme.name, "dark");
}

#[test]
fn test_clear_custom() {
    let registry = ThemeRegistry::new();
    let custom_theme = Theme::light();
    let mut custom = custom_theme.clone();
    custom.name = "my-custom".to_string();
    registry.register(custom).unwrap();

    assert_eq!(registry.custom_count().unwrap(), 1);
    registry.clear_custom().unwrap();
    assert_eq!(registry.custom_count().unwrap(), 0);
}
