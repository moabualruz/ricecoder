//! Popup widget for dialogs and notifications
//!
//! This module provides a popup widget for displaying confirmation dialogs, notifications,
//! and other modal interactions.

/// Popup type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopupType {
    /// Confirmation dialog
    Confirmation,
    /// Information dialog
    Information,
    /// Warning dialog
    Warning,
    /// Error dialog
    Error,
    /// Input dialog
    Input,
}

impl PopupType {
    /// Get the popup type name
    pub fn name(&self) -> &'static str {
        match self {
            PopupType::Confirmation => "Confirmation",
            PopupType::Information => "Information",
            PopupType::Warning => "Warning",
            PopupType::Error => "Error",
            PopupType::Input => "Input",
        }
    }

    /// Get the color code for the popup type
    pub fn color_code(&self) -> &'static str {
        match self {
            PopupType::Confirmation => "\x1b[36m",  // Cyan
            PopupType::Information => "\x1b[32m",    // Green
            PopupType::Warning => "\x1b[33m",        // Yellow
            PopupType::Error => "\x1b[31m",          // Red
            PopupType::Input => "\x1b[36m",          // Cyan
        }
    }
}

/// Popup button
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PopupButton {
    /// Button label
    pub label: String,
    /// Button ID (for identifying which button was clicked)
    pub id: String,
    /// Whether this is the default button
    pub is_default: bool,
}

impl PopupButton {
    /// Create a new button
    pub fn new(label: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            id: id.into(),
            is_default: false,
        }
    }

    /// Mark as default button
    pub fn with_default(mut self, is_default: bool) -> Self {
        self.is_default = is_default;
        self
    }
}

/// Popup widget for dialogs and notifications
pub struct PopupWidget {
    /// Popup type
    popup_type: PopupType,
    /// Title
    title: String,
    /// Message
    message: String,
    /// Buttons
    buttons: Vec<PopupButton>,
    /// Selected button index
    selected_button: usize,
    /// Whether popup is visible
    visible: bool,
    /// Input text (for input dialogs)
    input_text: String,
    /// Maximum width
    max_width: u16,
    /// Maximum height
    max_height: u16,
}

impl PopupWidget {
    /// Create a new popup widget
    pub fn new(popup_type: PopupType, title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            popup_type,
            title: title.into(),
            message: message.into(),
            buttons: Vec::new(),
            selected_button: 0,
            visible: false,
            input_text: String::new(),
            max_width: 80,
            max_height: 24,
        }
    }

    /// Create a confirmation dialog
    pub fn confirmation(title: impl Into<String>, message: impl Into<String>) -> Self {
        let mut popup = Self::new(PopupType::Confirmation, title, message);
        popup.add_button(PopupButton::new("Yes", "yes").with_default(true));
        popup.add_button(PopupButton::new("No", "no"));
        popup
    }

    /// Create an information dialog
    pub fn information(title: impl Into<String>, message: impl Into<String>) -> Self {
        let mut popup = Self::new(PopupType::Information, title, message);
        popup.add_button(PopupButton::new("OK", "ok").with_default(true));
        popup
    }

    /// Create a warning dialog
    pub fn warning(title: impl Into<String>, message: impl Into<String>) -> Self {
        let mut popup = Self::new(PopupType::Warning, title, message);
        popup.add_button(PopupButton::new("OK", "ok").with_default(true));
        popup
    }

    /// Create an error dialog
    pub fn error(title: impl Into<String>, message: impl Into<String>) -> Self {
        let mut popup = Self::new(PopupType::Error, title, message);
        popup.add_button(PopupButton::new("OK", "ok").with_default(true));
        popup
    }

    /// Create an input dialog
    pub fn input(title: impl Into<String>, message: impl Into<String>) -> Self {
        let mut popup = Self::new(PopupType::Input, title, message);
        popup.add_button(PopupButton::new("OK", "ok").with_default(true));
        popup.add_button(PopupButton::new("Cancel", "cancel"));
        popup
    }

    /// Add a button
    pub fn add_button(&mut self, button: PopupButton) {
        self.buttons.push(button);
    }

    /// Get the buttons
    pub fn buttons(&self) -> &[PopupButton] {
        &self.buttons
    }

    /// Get the selected button
    pub fn selected_button(&self) -> Option<&PopupButton> {
        self.buttons.get(self.selected_button)
    }

    /// Get the selected button ID
    pub fn selected_button_id(&self) -> Option<&str> {
        self.selected_button().map(|b| b.id.as_str())
    }

    /// Select the next button
    pub fn select_next_button(&mut self) {
        if self.selected_button < self.buttons.len() - 1 {
            self.selected_button += 1;
        }
    }

    /// Select the previous button
    pub fn select_prev_button(&mut self) {
        if self.selected_button > 0 {
            self.selected_button -= 1;
        }
    }

    /// Select a button by ID
    pub fn select_button_by_id(&mut self, id: &str) -> bool {
        if let Some(pos) = self.buttons.iter().position(|b| b.id == id) {
            self.selected_button = pos;
            return true;
        }
        false
    }

    /// Show the popup
    pub fn show(&mut self) {
        self.visible = true;
        // Select the default button
        if let Some(pos) = self.buttons.iter().position(|b| b.is_default) {
            self.selected_button = pos;
        }
    }

    /// Hide the popup
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Check if popup is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Get the popup type
    pub fn popup_type(&self) -> PopupType {
        self.popup_type
    }

    /// Get the title
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Get the message
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Set the message
    pub fn set_message(&mut self, message: impl Into<String>) {
        self.message = message.into();
    }

    /// Get the input text
    pub fn input_text(&self) -> &str {
        &self.input_text
    }

    /// Set the input text
    pub fn set_input_text(&mut self, text: impl Into<String>) {
        self.input_text = text.into();
    }

    /// Append to input text
    pub fn append_input(&mut self, c: char) {
        self.input_text.push(c);
    }

    /// Remove last character from input
    pub fn backspace_input(&mut self) {
        self.input_text.pop();
    }

    /// Clear input text
    pub fn clear_input(&mut self) {
        self.input_text.clear();
    }

    /// Set the maximum width
    pub fn set_max_width(&mut self, width: u16) {
        self.max_width = width;
    }

    /// Set the maximum height
    pub fn set_max_height(&mut self, height: u16) {
        self.max_height = height;
    }

    /// Get the maximum width
    pub fn max_width(&self) -> u16 {
        self.max_width
    }

    /// Get the maximum height
    pub fn max_height(&self) -> u16 {
        self.max_height
    }

    /// Get the popup width
    pub fn width(&self) -> u16 {
        let content_width = self.message.len() as u16 + 4;
        std::cmp::min(content_width, self.max_width)
    }

    /// Get the popup height
    pub fn height(&self) -> u16 {
        let button_height = if self.buttons.is_empty() { 0 } else { 3 };
        let content_height = 3 + button_height;
        std::cmp::min(content_height, self.max_height)
    }

    /// Get the formatted display text
    pub fn format_display(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("┌─ {} ─┐\n", self.title));
        output.push_str(&format!("│ {} │\n", self.message));

        if !self.buttons.is_empty() {
            output.push_str("├─────────────────┤\n");
            let button_text = self
                .buttons
                .iter()
                .enumerate()
                .map(|(idx, btn)| {
                    if idx == self.selected_button {
                        format!("[{}]", btn.label)
                    } else {
                        format!(" {} ", btn.label)
                    }
                })
                .collect::<Vec<_>>()
                .join("  ");
            output.push_str(&format!("│ {} │\n", button_text));
        }

        output.push_str("└─────────────────┘");
        output
    }
}

impl Default for PopupWidget {
    fn default() -> Self {
        Self::information("Information", "")
    }
}


