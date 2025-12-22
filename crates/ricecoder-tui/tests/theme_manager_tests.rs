use std::sync::Mutex;

use ricecoder_storage::{ThemePreference, ThemeStorage};
use ricecoder_tui::{Theme, ThemeManager, TuiConfig};
use tempfile::TempDir;

// Mutex to ensure test isolation when modifying environment variables
static TEST_LOCK: Mutex<()> = Mutex::new(());

fn with_temp_home<F>(f: F)
where
    F: FnOnce(),
{
    let _guard = TEST_LOCK.lock().unwrap();
    let temp_dir = TempDir::new().unwrap();
    let old_home = std::env::var("RICECODER_HOME").ok();
    std::env::set_var("RICECODER_HOME", temp_dir.path());

    f();

    // Restore environment
    if let Some(home) = old_home {
        std::env::set_var("RICECODER_HOME", home);
    } else {
        std::env::remove_var("RICECODER_HOME");
    }
}

#[test]
fn test_theme_manager_creation() {
    let manager = ThemeManager::new();
    assert_eq!(manager.current().unwrap().name, "dark");
}

#[test]
fn test_theme_manager_with_theme() {
    let theme = Theme::light();
    let manager = ThemeManager::with_theme(theme);
    assert_eq!(manager.current().unwrap().name, "light");
}

#[test]
fn test_switch_by_name() {
    let manager = ThemeManager::new();
    manager.switch_by_name("light").unwrap();
    assert_eq!(manager.current().unwrap().name, "light");

    manager.switch_by_name("monokai").unwrap();
    assert_eq!(manager.current().unwrap().name, "monokai");
}

#[test]
fn test_switch_by_invalid_name() {
    let manager = ThemeManager::new();
    assert!(manager.switch_by_name("invalid").is_err());
}

#[test]
fn test_switch_to() {
    let manager = ThemeManager::new();
    let theme = Theme::dracula();
    manager.switch_to(theme).unwrap();
    assert_eq!(manager.current().unwrap().name, "dracula");
}

#[test]
fn test_available_themes() {
    let manager = ThemeManager::new();
    let themes = manager.available_themes();
    assert_eq!(themes.len(), Theme::available_themes().len());
}

#[test]
fn test_current_name() {
    let manager = ThemeManager::new();
    assert_eq!(manager.current_name().unwrap(), "dark");

    manager.switch_by_name("nord").unwrap();
    assert_eq!(manager.current_name().unwrap(), "nord");
}

#[test]
fn test_load_from_config() {
    let manager = ThemeManager::new();
    let config = TuiConfig {
        theme: "dracula".to_string(),
        ..Default::default()
    };
    manager.load_from_config(&config).unwrap();
    assert_eq!(manager.current().unwrap().name, "dracula");
}

#[test]
fn test_save_to_config() {
    let manager = ThemeManager::new();
    manager.switch_by_name("monokai").unwrap();

    let mut config = TuiConfig::default();
    manager.save_to_config(&mut config).unwrap();
    assert_eq!(config.theme, "monokai");
}

#[test]
fn test_save_and_load_custom_theme() {
    let temp_dir = TempDir::new().unwrap();
    let theme_path = temp_dir.path().join("custom.yaml");

    let manager = ThemeManager::new();
    manager.switch_by_name("dracula").unwrap();
    manager.save_custom_theme(&theme_path).unwrap();

    let manager2 = ThemeManager::new();
    manager2.load_custom_theme(&theme_path).unwrap();
    assert_eq!(manager2.current().unwrap().name, "dracula");
}

#[test]
fn test_load_custom_themes_from_directory() {
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

    let themes = manager
        .load_custom_themes_from_directory(temp_dir.path())
        .unwrap();
    assert_eq!(themes.len(), 2);
}

#[test]
fn test_reset_colors() {
    let manager = ThemeManager::new();
    let original_primary = manager.current().unwrap().primary;

    // Modify current theme (indirectly via switch since we can't access private mutex directly easily,
    // but actually the test in src used internal access. Here we must use public API.
    // Wait, ThemeManager exposes public methods. But it doesn't expose a way to MODIFY the current theme's fields directly
    // unless we switch to a modified theme.)

    let mut theme = manager.current().unwrap();
    theme.primary = ricecoder_tui::style::Color::new(255, 0, 0);
    manager.switch_to(theme).unwrap();

    // Verify modification
    assert_ne!(manager.current().unwrap().primary, original_primary);

    // Reset colors
    manager.reset_colors().unwrap();

    // Verify reset
    assert_eq!(manager.current().unwrap().primary, original_primary);
}

#[test]
fn test_reset_theme() {
    let manager = ThemeManager::new();
    manager.switch_by_name("light").unwrap();

    let original_theme = Theme::light();

    // Modify current theme
    let mut theme = manager.current().unwrap();
    theme.primary = ricecoder_tui::style::Color::new(255, 0, 0);
    theme.background = ricecoder_tui::style::Color::new(100, 100, 100);
    manager.switch_to(theme).unwrap();

    // Verify modification
    let modified = manager.current().unwrap();
    assert_ne!(modified.primary, original_theme.primary);
    assert_ne!(modified.background, original_theme.background);

    // Reset theme
    manager.reset_theme().unwrap();

    // Verify reset
    let reset = manager.current().unwrap();
    assert_eq!(reset.primary, original_theme.primary);
    assert_eq!(reset.background, original_theme.background);
}

#[test]
fn test_reset_color() {
    let manager = ThemeManager::new();
    let original_error = manager.current().unwrap().error;

    // Modify error color
    let mut theme = manager.current().unwrap();
    theme.error = ricecoder_tui::style::Color::new(255, 0, 0);
    manager.switch_to(theme).unwrap();

    // Verify modification
    assert_ne!(manager.current().unwrap().error, original_error);

    // Reset error color
    manager.reset_color("error").unwrap();

    // Verify reset
    assert_eq!(manager.current().unwrap().error, original_error);
}

#[test]
fn test_get_default_color() {
    let manager = ThemeManager::new();
    let default_primary = manager.get_default_color("primary").unwrap();
    let current_primary = manager.current().unwrap().primary;
    assert_eq!(default_primary, current_primary);
}

#[test]
fn test_reset_notifies_listeners() {
    let manager = ThemeManager::new();
    let listener_called = std::sync::Arc::new(std::sync::Mutex::new(false));
    let listener_called_clone = listener_called.clone();

    manager
        .on_theme_changed(move |_theme| {
            *listener_called_clone.lock().unwrap() = true;
        })
        .unwrap();

    manager.reset_colors().unwrap();

    assert!(*listener_called.lock().unwrap());
}

#[test]
fn test_load_from_storage() {
    with_temp_home(|| {
        // Save a preference
        let pref = ThemePreference {
            current_theme: "light".to_string(),
            last_updated: None,
        };
        ThemeStorage::save_preference(&pref).unwrap();

        // Load it with theme manager
        let manager = ThemeManager::new();
        manager.load_from_storage().unwrap();
        assert_eq!(manager.current().unwrap().name, "light");
    });
}

#[test]
fn test_save_to_storage() {
    with_temp_home(|| {
        let manager = ThemeManager::new();
        manager.switch_by_name("dracula").unwrap();
        manager.save_to_storage().unwrap();

        // Verify it was saved
        let loaded_pref = ThemeStorage::load_preference().unwrap();
        assert_eq!(loaded_pref.current_theme, "dracula");
    });
}

#[test]
fn test_save_custom_theme_to_storage() {
    with_temp_home(|| {
        let manager = ThemeManager::new();
        manager.switch_by_name("monokai").unwrap();
        manager.save_custom_theme_to_storage("my_custom").unwrap();

        // Verify it was saved
        assert!(ThemeStorage::custom_theme_exists("my_custom").unwrap());
    });
}

#[test]
fn test_load_custom_themes_from_storage() {
    with_temp_home(|| {
        // Save some custom themes
        ThemeStorage::save_custom_theme(
            "custom1",
            "name: custom1\nprimary: \"#0078ff\"\nsecondary: \"#5ac8fa\"\naccent: \"#ff2d55\"\nbackground: \"#111827\"\nforeground: \"#f3f4f6\"\nerror: \"#ef4444\"\nwarning: \"#f59e0b\"\nsuccess: \"#22c55e\""
        ).unwrap();

        let manager = ThemeManager::new();
        let loaded = manager.load_custom_themes_from_storage().unwrap();

        assert_eq!(loaded.len(), 1);
        assert!(loaded.contains(&"custom1".to_string()));
    });
}

#[test]
fn test_delete_custom_theme_from_storage() {
    with_temp_home(|| {
        // Save a custom theme
        ThemeStorage::save_custom_theme(
            "to_delete",
            "name: to_delete\nprimary: \"#0078ff\"\nsecondary: \"#5ac8fa\"\naccent: \"#ff2d55\"\nbackground: \"#111827\"\nforeground: \"#f3f4f6\"\nerror: \"#ef4444\"\nwarning: \"#f59e0b\"\nsuccess: \"#22c55e\""
        ).unwrap();

        let manager = ThemeManager::new();
        manager
            .delete_custom_theme_from_storage("to_delete")
            .unwrap();

        // Verify it was deleted
        assert!(!ThemeStorage::custom_theme_exists("to_delete").unwrap());
    });
}
