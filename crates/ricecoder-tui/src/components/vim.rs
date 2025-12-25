//! Vim keybindings implementation

/// Vim keybinding mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VimMode {
    /// Normal mode (navigation)
    Normal,
    /// Insert mode (text input)
    Insert,
    /// Visual mode (selection)
    Visual,
    /// Command mode (commands)
    Command,
}

/// Vim keybindings configuration
pub struct VimKeybindings {
    /// Whether vim mode is enabled
    pub enabled: bool,
    /// Current vim mode
    pub mode: VimMode,
    /// Command buffer for command mode
    pub command_buffer: String,
}

impl VimKeybindings {
    /// Create a new vim keybindings configuration
    pub fn new() -> Self {
        Self {
            enabled: false,
            mode: VimMode::Normal,
            command_buffer: String::new(),
        }
    }

    /// Enable vim mode
    pub fn enable(&mut self) {
        self.enabled = true;
        self.mode = VimMode::Normal;
    }

    /// Disable vim mode
    pub fn disable(&mut self) {
        self.enabled = false;
        self.mode = VimMode::Normal;
        self.command_buffer.clear();
    }

    /// Toggle vim mode
    pub fn toggle(&mut self) {
        if self.enabled {
            self.disable();
        } else {
            self.enable();
        }
    }

    /// Enter insert mode
    pub fn enter_insert(&mut self) {
        if self.enabled {
            self.mode = VimMode::Insert;
        }
    }

    /// Enter normal mode
    pub fn enter_normal(&mut self) {
        if self.enabled {
            self.mode = VimMode::Normal;
            self.command_buffer.clear();
        }
    }

    /// Enter visual mode
    pub fn enter_visual(&mut self) {
        if self.enabled {
            self.mode = VimMode::Visual;
        }
    }

    /// Enter command mode
    pub fn enter_command(&mut self) {
        if self.enabled {
            self.mode = VimMode::Command;
            self.command_buffer.clear();
        }
    }

    /// Add character to command buffer
    pub fn add_to_command(&mut self, ch: char) {
        self.command_buffer.push(ch);
    }

    /// Clear command buffer
    pub fn clear_command(&mut self) {
        self.command_buffer.clear();
    }

    /// Get command buffer
    pub fn get_command(&self) -> &str {
        &self.command_buffer
    }

    /// Check if in normal mode
    pub fn is_normal(&self) -> bool {
        self.enabled && self.mode == VimMode::Normal
    }

    /// Check if in insert mode
    pub fn is_insert(&self) -> bool {
        self.enabled && self.mode == VimMode::Insert
    }

    /// Check if in visual mode
    pub fn is_visual(&self) -> bool {
        self.enabled && self.mode == VimMode::Visual
    }

    /// Check if in command mode
    pub fn is_command(&self) -> bool {
        self.enabled && self.mode == VimMode::Command
    }
}

impl Default for VimKeybindings {
    fn default() -> Self {
        Self::new()
    }
}
