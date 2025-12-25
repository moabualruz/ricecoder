//! Dialog widget implementation

/// Dialog type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogType {
    /// Input dialog
    Input,
    /// Confirmation dialog
    Confirm,
    /// Message dialog
    Message,
}

/// Dialog result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogResult {
    /// Dialog was confirmed
    Confirmed,
    /// Dialog was cancelled
    Cancelled,
    /// Dialog is still open
    Pending,
}

/// Dialog widget
pub struct DialogWidget {
    /// Dialog type
    pub dialog_type: DialogType,
    /// Dialog title
    pub title: String,
    /// Dialog message
    pub message: String,
    /// Input value (for input dialogs)
    pub input: String,
    /// Cursor position
    pub cursor: usize,
    /// Dialog result
    pub result: DialogResult,
    /// Validation function (for input dialogs)
    pub validator: Option<fn(&str) -> bool>,
    /// Error message (if validation fails)
    pub error_message: Option<String>,
    /// Confirmation state (for confirm dialogs)
    pub confirmed: Option<bool>,
}

impl DialogWidget {
    /// Create a new dialog widget
    pub fn new(
        dialog_type: DialogType,
        title: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            dialog_type,
            title: title.into(),
            message: message.into(),
            input: String::new(),
            cursor: 0,
            result: DialogResult::Pending,
            validator: None,
            error_message: None,
            confirmed: None,
        }
    }

    /// Set a validator function
    pub fn with_validator(mut self, validator: fn(&str) -> bool) -> Self {
        self.validator = Some(validator);
        self
    }

    /// Insert character
    pub fn insert_char(&mut self, ch: char) {
        if ch.is_ascii_graphic() || ch == ' ' {
            self.input.insert(self.cursor, ch);
            self.cursor += 1;
            self.error_message = None;
        }
    }

    /// Backspace
    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            self.input.remove(self.cursor - 1);
            self.cursor -= 1;
            self.error_message = None;
        }
    }

    /// Delete character at cursor
    pub fn delete(&mut self) {
        if self.cursor < self.input.len() {
            self.input.remove(self.cursor);
        }
    }

    /// Move cursor left
    pub fn cursor_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    /// Move cursor right
    pub fn cursor_right(&mut self) {
        if self.cursor < self.input.len() {
            self.cursor += 1;
        }
    }

    /// Move cursor to start
    pub fn cursor_start(&mut self) {
        self.cursor = 0;
    }

    /// Move cursor to end
    pub fn cursor_end(&mut self) {
        self.cursor = self.input.len();
    }

    /// Get input value
    pub fn get_input(&self) -> String {
        self.input.clone()
    }

    /// Validate input
    pub fn validate(&mut self) -> bool {
        if let Some(validator) = self.validator {
            if validator(&self.input) {
                self.error_message = None;
                true
            } else {
                self.error_message = Some("Invalid input".to_string());
                false
            }
        } else {
            true
        }
    }

    /// Confirm dialog
    pub fn confirm(&mut self) {
        match self.dialog_type {
            DialogType::Input => {
                if self.validate() {
                    self.result = DialogResult::Confirmed;
                }
            }
            DialogType::Confirm => {
                self.confirmed = Some(true);
                self.result = DialogResult::Confirmed;
            }
            DialogType::Message => {
                self.result = DialogResult::Confirmed;
            }
        }
    }

    /// Cancel dialog
    pub fn cancel(&mut self) {
        if self.dialog_type == DialogType::Confirm {
            self.confirmed = Some(false);
        }
        self.result = DialogResult::Cancelled;
    }

    /// Check if dialog is confirmed
    pub fn is_confirmed(&self) -> bool {
        self.result == DialogResult::Confirmed
    }

    /// Check if dialog is cancelled
    pub fn is_cancelled(&self) -> bool {
        self.result == DialogResult::Cancelled
    }

    /// Check if dialog is pending
    pub fn is_pending(&self) -> bool {
        self.result == DialogResult::Pending
    }

    /// Clear input
    pub fn clear_input(&mut self) {
        self.input.clear();
        self.cursor = 0;
        self.error_message = None;
    }
}
