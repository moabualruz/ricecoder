use ricecoder_tui::*;

mod tests {
    use super::*;

    #[test]
    fn test_accessibility_config_default() {
        let config = AccessibilityConfig::default();
        assert!(!config.screen_reader_enabled);
        assert!(!config.high_contrast_enabled);
        assert!(!config.animations_disabled);
        assert!(config.announcements_enabled);
    }

    #[test]
    fn test_focus_indicator_style() {
        assert_eq!(FocusIndicatorStyle::Bracket.prefix(), "[");
        assert_eq!(FocusIndicatorStyle::Bracket.suffix(), "]");
        assert_eq!(FocusIndicatorStyle::Arrow.prefix(), "> ");
        assert_eq!(FocusIndicatorStyle::Arrow.suffix(), "");
    }

    #[test]
    fn test_text_alternative() {
        let alt = TextAlternative::new("btn1", "Submit button", ElementType::Button)
            .with_long_description("Click to submit the form");
        assert_eq!(alt.id, "btn1");
        assert_eq!(alt.short_description, "Submit button");
        assert!(alt.long_description.is_some());
    }

    #[test]
    fn test_element_type_role() {
        assert_eq!(ElementType::Button.role(), "button");
        assert_eq!(ElementType::Input.role(), "textbox");
        assert_eq!(ElementType::List.role(), "list");
    }

    #[test]
    fn test_screen_reader_announcer() {
        let mut announcer = ScreenReaderAnnouncer::new(true);
        announcer.announce("Test announcement", AnnouncementPriority::Normal);
        assert_eq!(announcer.announcements().len(), 1);
        assert_eq!(
            announcer.last_announcement().unwrap().text,
            "Test announcement"
        );
    }

    #[test]
    fn test_screen_reader_announcer_disabled() {
        let mut announcer = ScreenReaderAnnouncer::new(false);
        announcer.announce("Test", AnnouncementPriority::Normal);
        assert_eq!(announcer.announcements().len(), 0);
    }

    #[test]
    fn test_keyboard_navigation_manager() {
        let mut manager = KeyboardNavigationManager::new();
        let alt1 = TextAlternative::new("btn1", "Button 1", ElementType::Button);
        let alt2 = TextAlternative::new("btn2", "Button 2", ElementType::Button);

        manager.register_element(alt1);
        manager.register_element(alt2);

        assert!(manager.focus("btn1"));
        assert_eq!(manager.focused_element, Some("btn1".to_string()));

        let next = manager.focus_next();
        assert!(next.is_some());
        assert_eq!(manager.focused_element, Some("btn2".to_string()));
    }

    #[test]
    fn test_keyboard_navigation_wrap_around() {
        let mut manager = KeyboardNavigationManager::new();
        manager.register_element(TextAlternative::new(
            "btn1",
            "Button 1",
            ElementType::Button,
        ));
        manager.register_element(TextAlternative::new(
            "btn2",
            "Button 2",
            ElementType::Button,
        ));

        manager.focus("btn2");
        let _next = manager.focus_next();
        assert_eq!(manager.focused_element, Some("btn1".to_string()));
    }

    #[test]
    fn test_animation_config_default() {
        let config = AnimationConfig::default();
        assert!(config.enabled);
        assert_eq!(config.speed, 1.0);
        assert!(!config.reduce_motion);
    }

    #[test]
    fn test_animation_duration_calculation() {
        let config = AnimationConfig {
            enabled: true,
            speed: 2.0,
            reduce_motion: false,
        };
        // Base 100ms at 2x speed should be 50ms
        assert_eq!(config.duration_ms(100), 50);
    }

    #[test]
    fn test_animation_disabled() {
        let config = AnimationConfig {
            enabled: false,
            speed: 1.0,
            reduce_motion: false,
        };
        // Disabled animations should return 0 duration
        assert_eq!(config.duration_ms(100), 0);
    }

    #[test]
    fn test_animation_reduce_motion() {
        let config = AnimationConfig {
            enabled: true,
            speed: 1.0,
            reduce_motion: true,
        };
        // Reduce motion should return 0 duration
        assert_eq!(config.duration_ms(100), 0);
        assert!(!config.should_animate());
    }

    #[test]
    fn test_accessibility_config_animations() {
        let config = AccessibilityConfig::default();
        assert!(config.animations.enabled);
        assert!(config.animations.should_animate());
    }

    #[test]
    fn test_state_change_event() {
        let event = StateChangeEvent::new(
            "button",
            "disabled",
            "enabled",
            AnnouncementPriority::Normal,
        );
        assert_eq!(event.component, "button");
        assert_eq!(event.previous_state, "disabled");
        assert_eq!(event.new_state, "enabled");
        assert!(event.announcement_text().contains("button"));
    }

    #[test]
    fn test_focus_manager() {
        let mut manager = FocusManager::new();
        assert!(manager.focused_element.is_none());

        manager.set_focus("btn1");
        assert_eq!(manager.focused_element, Some("btn1".to_string()));

        manager.set_focus("btn2");
        assert_eq!(manager.focused_element, Some("btn2".to_string()));

        let restored = manager.restore_focus();
        assert_eq!(restored, Some("btn1".to_string()));
    }

    #[test]
    fn test_focus_manager_clear() {
        let mut manager = FocusManager::new();
        manager.set_focus("btn1");
        manager.clear_focus();
        assert!(manager.focused_element.is_none());
    }

    #[test]
    fn test_enhanced_keyboard_navigation() {
        let mut nav = super::EnhancedKeyboardNavigation::new();

        nav.register_element(
            "btn1".to_string(),
            TextAlternative::new("btn1", "Button 1", ElementType::Button),
        );
        nav.register_element(
            "btn2".to_string(),
            TextAlternative::new("btn2", "Button 2", ElementType::Button),
        );

        // Test tab navigation
        let _ = nav.tab_next();
        assert_eq!(nav.current_focus().map(|alt| alt.id.as_str()), Some("btn1"));

        let _ = nav.tab_next();
        assert_eq!(nav.current_focus().map(|alt| alt.id.as_str()), Some("btn2"));

        // Test wrap around
        let _ = nav.tab_next();
        assert_eq!(nav.current_focus().map(|alt| alt.id.as_str()), Some("btn1"));
    }

    #[test]
    fn test_high_contrast_theme_manager() {
        let manager = super::HighContrastThemeManager::new();

        let themes = manager.available_themes();
        assert!(themes.contains(&"high-contrast-dark".to_string()));
        assert!(themes.contains(&"high-contrast-light".to_string()));
        assert!(themes.contains(&"high-contrast-yellow-blue".to_string()));

        let theme = manager.current_theme();
        assert_eq!(theme.name, "High Contrast Dark");
    }

    #[test]
    fn test_keyboard_shortcut_customizer() {
        let mut customizer = super::KeyboardShortcutCustomizer::new();

        // Test default shortcuts
        let shortcut = customizer.get_shortcut("mode.chat");
        assert!(shortcut.is_some());

        // Test custom shortcut
        let custom_keys = vec![crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Char('x'),
            modifiers: crossterm::event::KeyModifiers::CONTROL,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        }];

        assert!(customizer
            .set_shortcut("test.action".to_string(), custom_keys.clone())
            .is_ok());
        assert_eq!(customizer.get_shortcut("test.action"), Some(&custom_keys));
    }

    #[test]
    fn test_keyboard_shortcut_key_conversion() {
        let customizer = super::KeyboardShortcutCustomizer::new();

        let key = crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Char('a'),
            modifiers: crossterm::event::KeyModifiers::CONTROL,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        };

        let key_string = customizer.key_to_string(&key);
        assert_eq!(key_string, "Ctrl+A");

        let converted_back = customizer.string_to_key(&key_string);
        assert!(converted_back.is_ok());
        assert_eq!(
            converted_back.unwrap().code,
            crossterm::event::KeyCode::Char('a')
        );
    }
}
