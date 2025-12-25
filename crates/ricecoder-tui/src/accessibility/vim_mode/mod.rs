//! Vim-like input mode management

use std::collections::HashMap;

/// Vim-like input mode manager
#[derive(Debug, Clone)]
pub struct VimModeManager {
    /// Current input mode
    current_mode: InputMode,
    /// Previous mode (for switching back)
    previous_mode: InputMode,
    /// Mode-specific keybindings
    mode_keybindings: HashMap<InputMode, HashMap<String, ModeAction>>,
    /// Mode indicators
    mode_indicators: HashMap<InputMode, ModeIndicator>,
}

/// Input modes (vim-like)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputMode {
    /// Normal mode (navigation and commands)
    Normal,
    /// Insert mode (text input)
    Insert,
    /// Visual mode (text selection)
    Visual,
    /// Command mode (ex commands)
    Command,
    /// Replace mode (replace text)
    Replace,
}

/// Actions that can be performed in different modes
#[derive(Debug, Clone)]
pub enum ModeAction {
    SwitchMode(InputMode),
    ExecuteCommand(String),
    MoveCursor(Movement),
    InsertText(String),
    DeleteText(DeleteOperation),
    Custom(String),
}

/// Cursor movement operations
#[derive(Debug, Clone)]
pub enum Movement {
    Left,
    Right,
    Up,
    Down,
    WordForward,
    WordBackward,
    LineStart,
    LineEnd,
    DocumentStart,
    DocumentEnd,
}

/// Text deletion operations
#[derive(Debug, Clone)]
pub enum DeleteOperation {
    Character,
    Word,
    Line,
    Selection,
}

/// Mode indicator for UI display
#[derive(Debug, Clone)]
pub struct ModeIndicator {
    pub text: String,
    pub style: ModeIndicatorStyle,
}

/// Visual style for mode indicators
#[derive(Debug, Clone)]
pub enum ModeIndicatorStyle {
    Normal,
    Insert,
    Visual,
    Command,
    Replace,
}

impl VimModeManager {
    /// Create a new vim mode manager
    pub fn new() -> Self {
        let mut manager = Self {
            current_mode: InputMode::Normal,
            previous_mode: InputMode::Normal,
            mode_keybindings: HashMap::new(),
            mode_indicators: HashMap::new(),
        };

        manager.initialize_default_keybindings();
        manager.initialize_mode_indicators();
        manager
    }

    /// Initialize default keybindings for each mode
    fn initialize_default_keybindings(&mut self) {
        // Normal mode keybindings
        let mut normal_bindings = HashMap::new();
        normal_bindings.insert("i".to_string(), ModeAction::SwitchMode(InputMode::Insert));
        normal_bindings.insert("v".to_string(), ModeAction::SwitchMode(InputMode::Visual));
        normal_bindings.insert(":".to_string(), ModeAction::SwitchMode(InputMode::Command));
        normal_bindings.insert("R".to_string(), ModeAction::SwitchMode(InputMode::Replace));
        normal_bindings.insert("h".to_string(), ModeAction::MoveCursor(Movement::Left));
        normal_bindings.insert("j".to_string(), ModeAction::MoveCursor(Movement::Down));
        normal_bindings.insert("k".to_string(), ModeAction::MoveCursor(Movement::Up));
        normal_bindings.insert("l".to_string(), ModeAction::MoveCursor(Movement::Right));
        normal_bindings.insert("w".to_string(), ModeAction::MoveCursor(Movement::WordForward));
        normal_bindings.insert("b".to_string(), ModeAction::MoveCursor(Movement::WordBackward));
        normal_bindings.insert("0".to_string(), ModeAction::MoveCursor(Movement::LineStart));
        normal_bindings.insert("$".to_string(), ModeAction::MoveCursor(Movement::LineEnd));
        normal_bindings.insert("gg".to_string(), ModeAction::MoveCursor(Movement::DocumentStart));
        normal_bindings.insert("G".to_string(), ModeAction::MoveCursor(Movement::DocumentEnd));
        normal_bindings.insert("x".to_string(), ModeAction::DeleteText(DeleteOperation::Character));
        normal_bindings.insert("dw".to_string(), ModeAction::DeleteText(DeleteOperation::Word));
        normal_bindings.insert("dd".to_string(), ModeAction::DeleteText(DeleteOperation::Line));

        self.mode_keybindings.insert(InputMode::Normal, normal_bindings);

        // Insert mode keybindings
        let mut insert_bindings = HashMap::new();
        insert_bindings.insert("Esc".to_string(), ModeAction::SwitchMode(InputMode::Normal));
        self.mode_keybindings.insert(InputMode::Insert, insert_bindings);

        // Visual mode keybindings
        let mut visual_bindings = HashMap::new();
        visual_bindings.insert("Esc".to_string(), ModeAction::SwitchMode(InputMode::Normal));
        visual_bindings.insert("d".to_string(), ModeAction::DeleteText(DeleteOperation::Selection));
        self.mode_keybindings.insert(InputMode::Visual, visual_bindings);

        // Command mode keybindings
        let mut command_bindings = HashMap::new();
        command_bindings.insert("Esc".to_string(), ModeAction::SwitchMode(InputMode::Normal));
        command_bindings.insert("Enter".to_string(), ModeAction::ExecuteCommand("execute_command".to_string()));
        self.mode_keybindings.insert(InputMode::Command, command_bindings);
    }

    /// Initialize mode indicators
    fn initialize_mode_indicators(&mut self) {
        self.mode_indicators.insert(InputMode::Normal, ModeIndicator {
            text: "NORMAL".to_string(),
            style: ModeIndicatorStyle::Normal,
        });
        self.mode_indicators.insert(InputMode::Insert, ModeIndicator {
            text: "INSERT".to_string(),
            style: ModeIndicatorStyle::Insert,
        });
        self.mode_indicators.insert(InputMode::Visual, ModeIndicator {
            text: "VISUAL".to_string(),
            style: ModeIndicatorStyle::Visual,
        });
        self.mode_indicators.insert(InputMode::Command, ModeIndicator {
            text: "COMMAND".to_string(),
            style: ModeIndicatorStyle::Command,
        });
        self.mode_indicators.insert(InputMode::Replace, ModeIndicator {
            text: "REPLACE".to_string(),
            style: ModeIndicatorStyle::Replace,
        });
    }

    /// Get current input mode
    pub fn current_mode(&self) -> InputMode {
        self.current_mode
    }

    /// Switch to a different input mode
    pub fn switch_mode(&mut self, mode: InputMode) {
        self.previous_mode = self.current_mode;
        self.current_mode = mode;
    }

    /// Switch back to previous mode
    pub fn switch_to_previous_mode(&mut self) {
        std::mem::swap(&mut self.current_mode, &mut self.previous_mode);
    }

    /// Handle key input based on current mode
    pub fn handle_key(&mut self, key: &str) -> Option<ModeAction> {
        if let Some(bindings) = self.mode_keybindings.get(&self.current_mode) {
            if let Some(action) = bindings.get(key) {
                return Some(action.clone());
            }
        }
        None
    }

    /// Add a custom keybinding for a mode
    pub fn add_keybinding(&mut self, mode: InputMode, key: String, action: ModeAction) {
        self.mode_keybindings
            .entry(mode)
            .or_insert_with(HashMap::new)
            .insert(key, action);
    }

    /// Remove a keybinding
    pub fn remove_keybinding(&mut self, mode: InputMode, key: &str) {
        if let Some(bindings) = self.mode_keybindings.get_mut(&mode) {
            bindings.remove(key);
        }
    }

    /// Get mode indicator for current mode
    pub fn current_mode_indicator(&self) -> Option<&ModeIndicator> {
        self.mode_indicators.get(&self.current_mode)
    }

    /// Get all available keybindings for current mode
    pub fn current_mode_keybindings(&self) -> Option<&HashMap<String, ModeAction>> {
        self.mode_keybindings.get(&self.current_mode)
    }

    /// Check if vim mode is enabled
    pub fn is_vim_mode_enabled(&self) -> bool {
        true
    }

    /// Get mode-specific help text
    pub fn mode_help(&self) -> String {
        match self.current_mode {
            InputMode::Normal => "Normal mode: h/j/k/l to move, i to insert, v for visual, : for command".to_string(),
            InputMode::Insert => "Insert mode: Type to insert text, Esc to return to normal mode".to_string(),
            InputMode::Visual => "Visual mode: Select text, d to delete selection, Esc to exit".to_string(),
            InputMode::Command => "Command mode: Type commands, Enter to execute, Esc to cancel".to_string(),
            InputMode::Replace => "Replace mode: Type to replace text, Esc to return to normal mode".to_string(),
        }
    }
}

impl Default for VimModeManager {
    fn default() -> Self {
        Self::new()
    }
}
