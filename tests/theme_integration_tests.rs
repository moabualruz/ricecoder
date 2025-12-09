//! Integration tests for theme system
//!
//! Tests end-to-end theme system functionality including:
//! - Loading and applying themes
//! - Switching between themes
//! - Persisting theme preferences
//! - Custom theme management
//! **Validates: Requirements 1.1, 2.1, 5.1**

use ricecoder_tui::theme::ThemeManager;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_system_initialization() {
        let manager = ThemeManager::new();
        
        // Verify manager is initialized with default theme
        let current = manager.current().unwrap();
        assert_eq!(current.name, "dark");
        
        // Verify available themes
        let themes = manager.available_themes();
        assert_eq!(themes.len(), 6);
    }

    #[test]
    fn test_theme_switching_workflow() {
        let manager = ThemeManager::new();
        
        // Start with dark theme
        assert_eq!(manager.current().unwrap().name, "dark");
        
        // Switch to light
        manager.switch_by_name("light").unwrap();
        assert_eq!(manager.current().unwrap().name, "light");
        
        // Switch to dracula
        manager.switch_by_name("dracula").unwrap();
        assert_eq!(manager.current().unwrap().name, "dracula");
        
        // Switch back to dark
        manager.switch_by_name("dark").unwrap();
        assert_eq!(manager.current().unwrap().name, "dark");
    }

    #[test]
    fn test_theme_list_all() {
        let manager = ThemeManager::new();
        
        let all_themes = manager.list_all_themes().unwrap();
        assert_eq!(all_themes.len(), 6);
        
        // Verify all built-in themes are present
        assert!(all_themes.contains(&"dark".to_string()));
        assert!(all_themes.contains(&"light".to_string()));
        assert!(all_themes.contains(&"dracula".to_string()));
        assert!(all_themes.contains(&"monokai".to_string()));
        assert!(all_themes.contains(&"nord".to_string()));
        assert!(all_themes.contains(&"high-contrast".to_string()));
    }

    #[test]
    fn test_theme_registry_operations() {
        let manager = ThemeManager::new();
        
        // Verify theme exists
        assert!(manager.theme_exists("dark"));
        assert!(manager.theme_exists("light"));
        assert!(!manager.theme_exists("nonexistent"));
        
        // Verify is_builtin_theme
        assert!(manager.is_builtin_theme("dark"));
        assert!(manager.is_builtin_theme("light"));
        assert!(!manager.is_builtin_theme("nonexistent"));
        
        // Verify builtin_theme_count
        assert_eq!(manager.builtin_theme_count(), 6);
    }

    #[test]
    fn test_theme_color_access() {
        let manager = ThemeManager::new();
        manager.switch_by_name("dark").unwrap();
        
        let theme = manager.current().unwrap();
        
        // Verify all colors are accessible
        let _ = theme.primary;
        let _ = theme.secondary;
        let _ = theme.accent;
        let _ = theme.background;
        let _ = theme.foreground;
        let _ = theme.error;
        let _ = theme.warning;
        let _ = theme.success;
    }

    #[test]
    fn test_theme_reset_workflow() {
        let manager = ThemeManager::new();
        manager.switch_by_name("light").unwrap();
        
        let original = manager.current().unwrap();
        let original_primary = original.primary;
        
        // Reset colors
        manager.reset_colors().unwrap();
        
        let reset = manager.current().unwrap();
        assert_eq!(reset.primary, original_primary);
    }

    #[test]
    fn test_theme_reset_individual_color() {
        let manager = ThemeManager::new();
        manager.switch_by_name("monokai").unwrap();
        
        let original = manager.current().unwrap();
        let original_primary = original.primary;
        
        // Reset individual color
        manager.reset_color("primary").unwrap();
        
        let reset = manager.current().unwrap();
        assert_eq!(reset.primary, original_primary);
    }

    #[test]
    fn test_theme_get_default_color() {
        let manager = ThemeManager::new();
        manager.switch_by_name("nord").unwrap();
        
        let current = manager.current().unwrap();
        let current_primary = current.primary;
        
        // Get default color
        let default_primary = manager.get_default_color("primary").unwrap();
        
        // Should match current (since we haven't modified it)
        assert_eq!(default_primary, current_primary);
    }

    #[test]
    fn test_theme_switching_with_multiple_elements() {
        let manager = ThemeManager::new();
        
        // Switch through all themes and verify consistency
        let themes = vec!["dark", "light", "dracula", "monokai", "nord", "high-contrast"];
        
        for theme_name in themes {
            manager.switch_by_name(theme_name).unwrap();
            let theme = manager.current().unwrap();
            
            // Verify theme is complete
            assert_eq!(theme.name, theme_name);
            
            // Verify all colors are present
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
    fn test_theme_custom_theme_registration() {
        let manager = ThemeManager::new();
        
        // Get initial count
        let initial_count = manager.custom_theme_count().unwrap();
        
        // Create a custom theme
        let custom_theme = ricecoder_tui::style::Theme::light();
        let mut custom = custom_theme.clone();
        custom.name = "my_custom".to_string();
        
        // Register it
        manager.register_theme(custom).unwrap();
        
        // Verify it's registered
        assert!(manager.theme_exists("my_custom"));
        assert!(manager.is_custom_theme("my_custom").unwrap());
        
        // Verify count increased
        let new_count = manager.custom_theme_count().unwrap();
        assert_eq!(new_count, initial_count + 1);
    }

    #[test]
    fn test_theme_custom_theme_unregistration() {
        let manager = ThemeManager::new();
        
        // Register a custom theme
        let custom_theme = ricecoder_tui::style::Theme::light();
        let mut custom = custom_theme.clone();
        custom.name = "temp_custom".to_string();
        manager.register_theme(custom).unwrap();
        
        // Verify it's registered
        assert!(manager.theme_exists("temp_custom"));
        
        // Unregister it
        manager.unregister_theme("temp_custom").unwrap();
        
        // Verify it's gone
        assert!(!manager.theme_exists("temp_custom"));
    }

    #[test]
    fn test_theme_listener_notification() {
        let manager = ThemeManager::new();
        let listener_called = std::sync::Arc::new(std::sync::Mutex::new(false));
        let listener_called_clone = listener_called.clone();
        
        // Register a listener
        manager
            .on_theme_changed(move |_theme| {
                *listener_called_clone.lock().unwrap() = true;
            })
            .unwrap();
        
        // Switch theme
        manager.switch_by_name("light").unwrap();
        
        // Verify listener was called
        assert!(*listener_called.lock().unwrap());
    }

    #[test]
    fn test_theme_end_to_end_workflow() {
        let manager = ThemeManager::new();
        
        // 1. Initialize with default theme
        assert_eq!(manager.current().unwrap().name, "dark");
        
        // 2. List available themes
        let themes = manager.list_all_themes().unwrap();
        assert!(themes.len() >= 6);
        
        // 3. Switch to different theme
        manager.switch_by_name("light").unwrap();
        assert_eq!(manager.current().unwrap().name, "light");
        
        // 4. Get current theme name
        assert_eq!(manager.current_name().unwrap(), "light");
        
        // 5. Access theme colors
        let theme = manager.current().unwrap();
        let _ = theme.primary;
        let _ = theme.background;
        
        // 6. Reset colors
        manager.reset_colors().unwrap();
        
        // 7. Switch back to original
        manager.switch_by_name("dark").unwrap();
        assert_eq!(manager.current().unwrap().name, "dark");
    }

    #[test]
    fn test_theme_consistency_across_operations() {
        let manager = ThemeManager::new();
        
        // Get initial theme
        manager.switch_by_name("dracula").unwrap();
        let theme1 = manager.current().unwrap();
        let colors1 = (theme1.primary, theme1.secondary, theme1.background);
        
        // Perform various operations
        manager.switch_by_name("nord").unwrap();
        manager.switch_by_name("dracula").unwrap();
        
        // Verify theme is unchanged
        let theme2 = manager.current().unwrap();
        let colors2 = (theme2.primary, theme2.secondary, theme2.background);
        
        assert_eq!(colors1, colors2);
    }
}
