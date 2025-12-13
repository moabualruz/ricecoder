use ricecoder_config::*;
use ricecoder_config::types::EditorConfig;

#[test]
fn test_app_config_default() {
    let config = AppConfig::default();
    assert_eq!(config.editor.tab_size, 4);
    assert!(config.editor.insert_spaces);
    assert_eq!(config.ui.theme, "dark");
    assert_eq!(config.theme.current, "dark");
}

#[test]
fn test_editor_config_default() {
    let config = EditorConfig::default();
    assert_eq!(config.tab_size, 4);
    assert!(config.insert_spaces);
    assert!(config.line_numbers);
}

#[test]
fn test_config_validation() {
    let mut config = AppConfig::default();
    assert!(ConfigManager::validate_config(&ConfigManager::new(), &config).is_ok());

    config.editor.tab_size = 0;
    assert!(ConfigManager::validate_config(&ConfigManager::new(), &config).is_err());
}

#[test]
fn test_tui_config_default() {
    let config = TuiConfig::default();
    assert_eq!(config.theme, "dark");
    assert!(config.animations);
    assert!(config.mouse);
}