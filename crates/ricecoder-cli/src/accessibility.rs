// Accessibility features and keyboard shortcuts documentation

use crate::output::OutputStyle;

/// Keyboard shortcuts for RiceCoder
pub struct KeyboardShortcuts;

impl KeyboardShortcuts {
    /// Get all keyboard shortcuts
    pub fn all() -> Vec<(&'static str, &'static str, &'static str)> {
        vec![
            // Navigation
            ("Navigation", "↑/↓", "Navigate through items"),
            ("Navigation", "Page Up/Down", "Scroll through content"),
            ("Navigation", "Home/End", "Jump to start/end"),
            ("Navigation", "Tab", "Move to next field"),
            ("Navigation", "Shift+Tab", "Move to previous field"),
            
            // Editing
            ("Editing", "Ctrl+A", "Select all"),
            ("Editing", "Ctrl+C", "Copy"),
            ("Editing", "Ctrl+V", "Paste"),
            ("Editing", "Ctrl+X", "Cut"),
            ("Editing", "Ctrl+Z", "Undo"),
            ("Editing", "Ctrl+Y", "Redo"),
            
            // Chat Mode
            ("Chat", "Enter", "Send message"),
            ("Chat", "Shift+Enter", "New line in message"),
            ("Chat", "Ctrl+L", "Clear chat history"),
            ("Chat", "Ctrl+P", "Previous message"),
            ("Chat", "Ctrl+N", "Next message"),
            ("Chat", "Escape", "Cancel input"),
            
            // General
            ("General", "Ctrl+H", "Show help"),
            ("General", "Ctrl+Q", "Quit"),
            ("General", "Ctrl+D", "Exit"),
            ("General", "?", "Show help"),
            ("General", "Ctrl+/", "Toggle help"),
        ]
    }

    /// Get shortcuts for a specific category
    pub fn by_category(category: &str) -> Vec<(&'static str, &'static str)> {
        Self::all()
            .into_iter()
            .filter(|(cat, _, _)| *cat == category)
            .map(|(_, key, desc)| (key, desc))
            .collect()
    }

    /// Print all shortcuts
    pub fn print_all() {
        let style = OutputStyle::default();
        println!("{}", style.section("Keyboard Shortcuts"));
        println!();

        let mut current_category = "";
        for (category, key, description) in Self::all() {
            if category != current_category {
                println!("{}", style.header(category));
                current_category = category;
            }
            println!("  {:<20} {}", key, description);
        }
        println!();
    }

    /// Print shortcuts for a specific category
    pub fn print_category(category: &str) {
        let style = OutputStyle::default();
        println!("{}", style.section(&format!("{} Shortcuts", category)));
        println!();

        for (key, description) in Self::by_category(category) {
            println!("  {:<20} {}", key, description);
        }
        println!();
    }
}

/// Accessibility features
pub struct AccessibilityFeatures;

impl AccessibilityFeatures {
    /// Check if screen reader mode is enabled
    pub fn screen_reader_enabled() -> bool {
        std::env::var("RICECODER_SCREEN_READER")
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(false)
    }

    /// Check if high contrast mode is enabled
    pub fn high_contrast_enabled() -> bool {
        std::env::var("RICECODER_HIGH_CONTRAST")
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(false)
    }

    /// Check if reduced motion is preferred
    pub fn reduced_motion_enabled() -> bool {
        std::env::var("RICECODER_REDUCED_MOTION")
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(false)
    }

    /// Get accessibility settings
    pub fn get_settings() -> AccessibilitySettings {
        AccessibilitySettings {
            screen_reader: Self::screen_reader_enabled(),
            high_contrast: Self::high_contrast_enabled(),
            reduced_motion: Self::reduced_motion_enabled(),
        }
    }

    /// Print accessibility settings
    pub fn print_settings() {
        let style = OutputStyle::default();
        let settings = Self::get_settings();

        println!("{}", style.section("Accessibility Settings"));
        println!();
        println!(
            "{}",
            style.key_value(
                "Screen Reader",
                if settings.screen_reader { "Enabled" } else { "Disabled" }
            )
        );
        println!(
            "{}",
            style.key_value(
                "High Contrast",
                if settings.high_contrast { "Enabled" } else { "Disabled" }
            )
        );
        println!(
            "{}",
            style.key_value(
                "Reduced Motion",
                if settings.reduced_motion { "Enabled" } else { "Disabled" }
            )
        );
        println!();

        println!("{}", style.section("How to Enable"));
        println!();
        println!("Set environment variables:");
        println!();
        println!("  # Enable screen reader mode");
        println!("  export RICECODER_SCREEN_READER=true");
        println!();
        println!("  # Enable high contrast mode");
        println!("  export RICECODER_HIGH_CONTRAST=true");
        println!();
        println!("  # Enable reduced motion");
        println!("  export RICECODER_REDUCED_MOTION=true");
        println!();
    }

    /// Print accessibility guide
    pub fn print_guide() {
        let style = OutputStyle::default();
        println!("{}", style.section("Accessibility Guide"));
        println!();

        println!("{}", style.header("Screen Reader Support"));
        println!();
        println!("RiceCoder supports screen readers through:");
        println!("{}", style.list_item("Clear, descriptive text labels"));
        println!("{}", style.list_item("Semantic HTML structure"));
        println!("{}", style.list_item("ARIA attributes for dynamic content"));
        println!();
        println!("Enable screen reader mode:");
        println!("  export RICECODER_SCREEN_READER=true");
        println!();

        println!("{}", style.header("High Contrast Mode"));
        println!();
        println!("For users with low vision:");
        println!("{}", style.list_item("Increased color contrast"));
        println!("{}", style.list_item("Larger text"));
        println!("{}", style.list_item("Bold fonts"));
        println!();
        println!("Enable high contrast mode:");
        println!("  export RICECODER_HIGH_CONTRAST=true");
        println!();

        println!("{}", style.header("Keyboard Navigation"));
        println!();
        println!("Full keyboard support:");
        println!("{}", style.list_item("Tab to navigate"));
        println!("{}", style.list_item("Arrow keys to move"));
        println!("{}", style.list_item("Enter to select"));
        println!("{}", style.list_item("Escape to cancel"));
        println!();
        println!("View all shortcuts:");
        println!("  rice help shortcuts");
        println!();

        println!("{}", style.header("Reduced Motion"));
        println!();
        println!("For users sensitive to motion:");
        println!("{}", style.list_item("Minimal animations"));
        println!("{}", style.list_item("No auto-scrolling"));
        println!("{}", style.list_item("Instant transitions"));
        println!();
        println!("Enable reduced motion:");
        println!("  export RICECODER_REDUCED_MOTION=true");
        println!();

        println!("{}", style.header("Text Size"));
        println!();
        println!("Adjust terminal font size:");
        println!("{}", style.list_item("Most terminals: Ctrl+Plus to increase"));
        println!("{}", style.list_item("Most terminals: Ctrl+Minus to decrease"));
        println!();

        println!("{}", style.header("Color Blindness"));
        println!();
        println!("RiceCoder uses symbols in addition to colors:");
        println!("{}", style.list_item("✓ for success"));
        println!("{}", style.list_item("✗ for errors"));
        println!("{}", style.list_item("⚠ for warnings"));
        println!("{}", style.list_item("ℹ for information"));
        println!();

        println!("{}", style.header("Getting Help"));
        println!();
        println!("For accessibility issues:");
        println!("{}", style.list_item("Report on GitHub: https://github.com/ricecoder/ricecoder/issues"));
        println!("{}", style.list_item("Include your accessibility needs"));
        println!("{}", style.list_item("Describe the issue in detail"));
        println!();
    }
}

/// Accessibility settings
#[derive(Debug, Clone)]
pub struct AccessibilitySettings {
    pub screen_reader: bool,
    pub high_contrast: bool,
    pub reduced_motion: bool,
}

impl AccessibilitySettings {
    /// Create default settings
    pub fn default() -> Self {
        Self {
            screen_reader: false,
            high_contrast: false,
            reduced_motion: false,
        }
    }

    /// Load settings from environment
    pub fn from_env() -> Self {
        AccessibilityFeatures::get_settings()
    }
}


