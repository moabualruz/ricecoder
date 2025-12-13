use ricecoder_cli::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyboard_shortcuts_not_empty() {
        assert!(!KeyboardShortcuts::all().is_empty());
    }

    #[test]
    fn test_keyboard_shortcuts_by_category() {
        let nav_shortcuts = KeyboardShortcuts::by_category("Navigation");
        assert!(!nav_shortcuts.is_empty());
    }

    #[test]
    fn test_accessibility_settings_default() {
        let settings = AccessibilitySettings::default();
        assert!(!settings.screen_reader);
        assert!(!settings.high_contrast);
        assert!(!settings.reduced_motion);
    }
}