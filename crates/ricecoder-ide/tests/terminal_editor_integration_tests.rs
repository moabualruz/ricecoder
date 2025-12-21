//! Integration tests for terminal editor integration
//! Tests vim/neovim and emacs communication with ricecoder

use ricecoder_ide::editor_config::{
    CompletionSettings, DiagnosticsSettings, EmacsConfig, HoverSettings, TerminalEditorConfig,
    VimConfig,
};

#[test]
fn test_vim_config_creation() {
    let config = VimConfig::default();

    assert!(config.enabled);
    assert_eq!(config.host, "localhost");
    assert_eq!(config.port, 9000);
    assert_eq!(config.timeout_ms, 5000);
    assert!(config.completion.enabled);
    assert!(config.diagnostics.enabled);
    assert!(config.hover.enabled);
}

#[test]
fn test_vim_config_validation() {
    let mut config = VimConfig::default();

    // Valid config should pass validation
    assert!(config.validate().is_ok());

    // Invalid port should fail
    config.port = 0;
    assert!(config.validate().is_err());

    // Reset and test invalid timeout
    config.port = 9000;
    config.timeout_ms = 0;
    assert!(config.validate().is_err());

    // Reset and test invalid max_items
    config.timeout_ms = 5000;
    config.completion.max_items = 0;
    assert!(config.validate().is_err());
}

#[test]
fn test_emacs_config_creation() {
    let config = EmacsConfig::default();

    assert!(config.enabled);
    assert_eq!(config.host, "localhost");
    assert_eq!(config.port, 9000);
    assert_eq!(config.timeout_ms, 5000);
    assert!(config.completion.enabled);
    assert!(config.diagnostics.enabled);
    assert!(config.hover.enabled);
}

#[test]
fn test_emacs_config_validation() {
    let mut config = EmacsConfig::default();

    // Valid config should pass validation
    assert!(config.validate().is_ok());

    // Invalid port should fail
    config.port = 0;
    assert!(config.validate().is_err());

    // Reset and test invalid timeout
    config.port = 9000;
    config.timeout_ms = 0;
    assert!(config.validate().is_err());

    // Reset and test invalid max_items
    config.timeout_ms = 5000;
    config.completion.max_items = 0;
    assert!(config.validate().is_err());
}

#[test]
fn test_terminal_editor_config_creation() {
    let config = TerminalEditorConfig::default();

    assert!(config.vim.is_some());
    assert!(config.emacs.is_some());

    let vim_config = config.vim().unwrap();
    assert!(vim_config.enabled);

    let emacs_config = config.emacs().unwrap();
    assert!(emacs_config.enabled);
}

#[test]
fn test_terminal_editor_config_validation() {
    let config = TerminalEditorConfig::default();

    // Default config should be valid
    assert!(config.validate().is_ok());
}

#[test]
fn test_completion_settings_default() {
    let settings = CompletionSettings::default();

    assert!(settings.enabled);
    assert_eq!(settings.max_items, 20);
    assert_eq!(settings.trigger_characters.len(), 2);
    assert!(settings.trigger_characters.contains(&".".to_string()));
    assert!(settings.trigger_characters.contains(&":".to_string()));
}

#[test]
fn test_diagnostics_settings_default() {
    let settings = DiagnosticsSettings::default();

    assert!(settings.enabled);
    assert!(settings.show_on_change);
    assert_eq!(settings.min_severity, 1);
}

#[test]
fn test_hover_settings_default() {
    let settings = HoverSettings::default();

    assert!(settings.enabled);
    assert!(!settings.show_on_move);
    assert_eq!(settings.delay_ms, 500);
}

#[test]
fn test_vim_config_custom_keybindings() {
    let mut config = VimConfig::default();

    config
        .keybindings
        .insert("completion".to_string(), "<C-x><C-o>".to_string());
    config
        .keybindings
        .insert("hover".to_string(), "K".to_string());
    config
        .keybindings
        .insert("goto_definition".to_string(), "<C-]>".to_string());

    assert_eq!(config.keybindings.len(), 3);
    assert_eq!(config.keybindings.get("completion").unwrap(), "<C-x><C-o>");
    assert_eq!(config.keybindings.get("hover").unwrap(), "K");
    assert_eq!(config.keybindings.get("goto_definition").unwrap(), "<C-]>");
}

#[test]
fn test_emacs_config_custom_keybindings() {
    let mut config = EmacsConfig::default();

    config.keybindings.insert(
        "completion".to_string(),
        "M-x completion-at-point".to_string(),
    );
    config
        .keybindings
        .insert("hover".to_string(), "C-c C-h".to_string());
    config
        .keybindings
        .insert("goto_definition".to_string(), "C-c C-d".to_string());

    assert_eq!(config.keybindings.len(), 3);
    assert_eq!(
        config.keybindings.get("completion").unwrap(),
        "M-x completion-at-point"
    );
    assert_eq!(config.keybindings.get("hover").unwrap(), "C-c C-h");
    assert_eq!(
        config.keybindings.get("goto_definition").unwrap(),
        "C-c C-d"
    );
}

#[test]
fn test_vim_config_custom_host_and_port() {
    let config = VimConfig {
        host: "192.168.1.1".to_string(),
        port: 8080,
        ..Default::default()
    };

    assert_eq!(config.host, "192.168.1.1");
    assert_eq!(config.port, 8080);
    assert!(config.validate().is_ok());
}

#[test]
fn test_emacs_config_custom_host_and_port() {
    let config = EmacsConfig {
        host: "192.168.1.1".to_string(),
        port: 8080,
        ..Default::default()
    };

    assert_eq!(config.host, "192.168.1.1");
    assert_eq!(config.port, 8080);
    assert!(config.validate().is_ok());
}

#[test]
fn test_vim_config_completion_settings() {
    let mut config = VimConfig::default();

    config.completion.max_items = 50;
    config.completion.trigger_characters.push("(".to_string());

    assert_eq!(config.completion.max_items, 50);
    assert_eq!(config.completion.trigger_characters.len(), 3);
    assert!(config.validate().is_ok());
}

#[test]
fn test_emacs_config_diagnostics_settings() {
    let mut config = EmacsConfig::default();

    config.diagnostics.show_on_change = false;
    config.diagnostics.min_severity = 2;

    assert!(!config.diagnostics.show_on_change);
    assert_eq!(config.diagnostics.min_severity, 2);
    assert!(config.validate().is_ok());
}

#[test]
fn test_vim_config_hover_settings() {
    let mut config = VimConfig::default();

    config.hover.show_on_move = true;
    config.hover.delay_ms = 1000;

    assert!(config.hover.show_on_move);
    assert_eq!(config.hover.delay_ms, 1000);
    assert!(config.validate().is_ok());
}

#[test]
fn test_terminal_editor_config_vim_only() {
    let config = TerminalEditorConfig {
        vim: Some(VimConfig::default()),
        emacs: None,
    };

    assert!(config.vim.is_some());
    assert!(config.emacs.is_none());
    assert!(config.validate().is_ok());
}

#[test]
fn test_terminal_editor_config_emacs_only() {
    let config = TerminalEditorConfig {
        vim: None,
        emacs: Some(EmacsConfig::default()),
    };

    assert!(config.vim.is_none());
    assert!(config.emacs.is_some());
    assert!(config.validate().is_ok());
}

#[test]
fn test_terminal_editor_config_both_disabled() {
    let vim_config = VimConfig {
        enabled: false,
        ..Default::default()
    };

    let emacs_config = EmacsConfig {
        enabled: false,
        ..Default::default()
    };

    let config = TerminalEditorConfig {
        vim: Some(vim_config),
        emacs: Some(emacs_config),
    };

    assert!(config.validate().is_ok());
}

#[test]
fn test_vim_config_serialization() {
    let config = VimConfig::default();

    // Serialize to JSON
    let json = serde_json::to_string(&config).unwrap();
    assert!(!json.is_empty());

    // Deserialize back
    let deserialized: VimConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.host, config.host);
    assert_eq!(deserialized.port, config.port);
}

#[test]
fn test_emacs_config_serialization() {
    let config = EmacsConfig::default();

    // Serialize to JSON
    let json = serde_json::to_string(&config).unwrap();
    assert!(!json.is_empty());

    // Deserialize back
    let deserialized: EmacsConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.host, config.host);
    assert_eq!(deserialized.port, config.port);
}

#[test]
fn test_terminal_editor_config_serialization() {
    let config = TerminalEditorConfig::default();

    // Serialize to JSON
    let json = serde_json::to_string(&config).unwrap();
    assert!(!json.is_empty());

    // Deserialize back
    let deserialized: TerminalEditorConfig = serde_json::from_str(&json).unwrap();
    assert!(deserialized.vim.is_some());
    assert!(deserialized.emacs.is_some());
}

#[test]
fn test_vim_config_yaml_serialization() {
    let config = VimConfig::default();

    // Serialize to YAML
    let yaml = serde_yaml::to_string(&config).unwrap();
    assert!(!yaml.is_empty());

    // Deserialize back
    let deserialized: VimConfig = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(deserialized.host, config.host);
    assert_eq!(deserialized.port, config.port);
}

#[test]
fn test_emacs_config_yaml_serialization() {
    let config = EmacsConfig::default();

    // Serialize to YAML
    let yaml = serde_yaml::to_string(&config).unwrap();
    assert!(!yaml.is_empty());

    // Deserialize back
    let deserialized: EmacsConfig = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(deserialized.host, config.host);
    assert_eq!(deserialized.port, config.port);
}

#[test]
fn test_terminal_editor_config_yaml_serialization() {
    let config = TerminalEditorConfig::default();

    // Serialize to YAML
    let yaml = serde_yaml::to_string(&config).unwrap();
    assert!(!yaml.is_empty());

    // Deserialize back
    let deserialized: TerminalEditorConfig = serde_yaml::from_str(&yaml).unwrap();
    assert!(deserialized.vim.is_some());
    assert!(deserialized.emacs.is_some());
}

#[test]
fn test_vim_config_multiple_trigger_characters() {
    let mut config = VimConfig::default();

    config.completion.trigger_characters = vec![
        ".".to_string(),
        ":".to_string(),
        "(".to_string(),
        "[".to_string(),
    ];

    assert_eq!(config.completion.trigger_characters.len(), 4);
    assert!(config.validate().is_ok());
}

#[test]
fn test_emacs_config_multiple_trigger_characters() {
    let mut config = EmacsConfig::default();

    config.completion.trigger_characters = vec![
        ".".to_string(),
        ":".to_string(),
        "(".to_string(),
        "[".to_string(),
    ];

    assert_eq!(config.completion.trigger_characters.len(), 4);
    assert!(config.validate().is_ok());
}

#[test]
fn test_vim_config_high_timeout() {
    let config = VimConfig {
        timeout_ms: 30000, // 30 seconds
        ..Default::default()
    };

    assert_eq!(config.timeout_ms, 30000);
    assert!(config.validate().is_ok());
}

#[test]
fn test_emacs_config_high_timeout() {
    let config = EmacsConfig {
        timeout_ms: 30000, // 30 seconds
        ..Default::default()
    };

    assert_eq!(config.timeout_ms, 30000);
    assert!(config.validate().is_ok());
}

#[test]
fn test_vim_config_high_max_items() {
    let mut config = VimConfig::default();
    config.completion.max_items = 100;

    assert_eq!(config.completion.max_items, 100);
    assert!(config.validate().is_ok());
}

#[test]
fn test_emacs_config_high_max_items() {
    let mut config = EmacsConfig::default();
    config.completion.max_items = 100;

    assert_eq!(config.completion.max_items, 100);
    assert!(config.validate().is_ok());
}

#[test]
fn test_vim_config_all_settings_disabled() {
    let mut config = VimConfig::default();
    config.completion.enabled = false;
    config.diagnostics.enabled = false;
    config.hover.enabled = false;

    assert!(!config.completion.enabled);
    assert!(!config.diagnostics.enabled);
    assert!(!config.hover.enabled);
    assert!(config.validate().is_ok());
}

#[test]
fn test_emacs_config_all_settings_disabled() {
    let mut config = EmacsConfig::default();
    config.completion.enabled = false;
    config.diagnostics.enabled = false;
    config.hover.enabled = false;

    assert!(!config.completion.enabled);
    assert!(!config.diagnostics.enabled);
    assert!(!config.hover.enabled);
    assert!(config.validate().is_ok());
}
