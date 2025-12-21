//! Core data models for keybinds

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::error::ParseError;

/// Represents a keyboard modifier (Ctrl, Shift, Alt, Meta)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Modifier {
    Ctrl,
    Shift,
    Alt,
    Meta,
}

impl fmt::Display for Modifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Modifier::Ctrl => write!(f, "Ctrl"),
            Modifier::Shift => write!(f, "Shift"),
            Modifier::Alt => write!(f, "Alt"),
            Modifier::Meta => write!(f, "Meta"),
        }
    }
}

impl FromStr for Modifier {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ctrl" | "control" => Ok(Modifier::Ctrl),
            "shift" => Ok(Modifier::Shift),
            "alt" => Ok(Modifier::Alt),
            "meta" | "cmd" | "command" => Ok(Modifier::Meta),
            _ => Err(ParseError::InvalidModifier(s.to_string())),
        }
    }
}

/// Represents a key on the keyboard
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Key {
    Char(char),
    F(u8),
    Enter,
    Escape,
    Tab,
    Backspace,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,
    Up,
    Down,
    Left,
    Right,
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Key::Char(c) => write!(f, "{}", c),
            Key::F(n) => write!(f, "F{}", n),
            Key::Enter => write!(f, "Enter"),
            Key::Escape => write!(f, "Escape"),
            Key::Tab => write!(f, "Tab"),
            Key::Backspace => write!(f, "Backspace"),
            Key::Delete => write!(f, "Delete"),
            Key::Home => write!(f, "Home"),
            Key::End => write!(f, "End"),
            Key::PageUp => write!(f, "PageUp"),
            Key::PageDown => write!(f, "PageDown"),
            Key::Up => write!(f, "Up"),
            Key::Down => write!(f, "Down"),
            Key::Left => write!(f, "Left"),
            Key::Right => write!(f, "Right"),
        }
    }
}

impl FromStr for Key {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "enter" | "return" => Ok(Key::Enter),
            "escape" | "esc" => Ok(Key::Escape),
            "tab" => Ok(Key::Tab),
            "backspace" | "bksp" => Ok(Key::Backspace),
            "delete" | "del" => Ok(Key::Delete),
            "home" => Ok(Key::Home),
            "end" => Ok(Key::End),
            "pageup" | "page_up" => Ok(Key::PageUp),
            "pagedown" | "page_down" => Ok(Key::PageDown),
            "up" => Ok(Key::Up),
            "down" => Ok(Key::Down),
            "left" => Ok(Key::Left),
            "right" => Ok(Key::Right),
            s if s.starts_with('f') && s.len() > 1 => {
                let num: u8 = s[1..].parse().map_err(|_| {
                    ParseError::InvalidKeySyntax(format!("Invalid function key: {}", s))
                })?;
                if (1..=12).contains(&num) {
                    Ok(Key::F(num))
                } else {
                    Err(ParseError::InvalidKeySyntax(format!(
                        "Function key must be F1-F12, got: {}",
                        s
                    )))
                }
            }
            s if s.len() == 1 => Ok(Key::Char(s.chars().next().unwrap())),
            _ => Err(ParseError::InvalidKeySyntax(format!("Unknown key: {}", s))),
        }
    }
}

/// Represents a key combination (modifiers + key)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyCombo {
    pub modifiers: Vec<Modifier>,
    pub key: Key,
}

impl fmt::Display for KeyCombo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for modifier in &self.modifiers {
            write!(f, "{}+", modifier)?;
        }
        write!(f, "{}", self.key)
    }
}

impl FromStr for KeyCombo {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('+').collect();
        if parts.is_empty() {
            return Err(ParseError::InvalidKeySyntax(
                "Empty key combination".to_string(),
            ));
        }

        let mut modifiers = Vec::new();
        for part in &parts[..parts.len() - 1] {
            modifiers.push(Modifier::from_str(part)?);
        }

        let key = Key::from_str(parts[parts.len() - 1])?;

        Ok(KeyCombo { modifiers, key })
    }
}

/// UI context for keybindings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Context {
    /// Global context - applies everywhere
    Global,
    /// Input context - text input fields
    Input,
    /// Chat context - chat interface
    Chat,
    /// Dialog context - modal dialogs
    Dialog,
    /// Command palette context - command search
    CommandPalette,
}

impl Context {
    /// Get the priority of this context (higher = more specific)
    pub fn priority(&self) -> u8 {
        match self {
            Context::Global => 0,
            Context::Input => 1,
            Context::Chat => 2,
            Context::Dialog => 3,
            Context::CommandPalette => 4,
        }
    }

    /// Check if this context inherits from another context
    pub fn inherits_from(&self, other: &Context) -> bool {
        self.priority() >= other.priority()
    }
}

impl fmt::Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Context::Global => write!(f, "global"),
            Context::Input => write!(f, "input"),
            Context::Chat => write!(f, "chat"),
            Context::Dialog => write!(f, "dialog"),
            Context::CommandPalette => write!(f, "command_palette"),
        }
    }
}

impl FromStr for Context {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "global" => Ok(Context::Global),
            "input" => Ok(Context::Input),
            "chat" => Ok(Context::Chat),
            "dialog" => Ok(Context::Dialog),
            "command_palette" | "commandpalette" => Ok(Context::CommandPalette),
            _ => Err(ParseError::InvalidModifier(format!(
                "Unknown context: {}",
                s
            ))),
        }
    }
}

/// Represents a single keybind configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Keybind {
    pub action_id: String,
    pub key: String,
    pub category: String,
    pub description: String,
    #[serde(default)]
    pub is_default: bool,
    /// Contexts where this keybind applies (empty = global)
    #[serde(default)]
    pub contexts: Vec<Context>,
}

impl Keybind {
    /// Parse the key string into a KeyCombo
    pub fn parse_key(&self) -> Result<KeyCombo, ParseError> {
        KeyCombo::from_str(&self.key)
    }

    /// Create a new keybind
    pub fn new(
        action_id: impl Into<String>,
        key: impl Into<String>,
        category: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Keybind {
            action_id: action_id.into(),
            key: key.into(),
            category: category.into(),
            description: description.into(),
            is_default: false,
            contexts: Vec::new(),
        }
    }

    /// Create a new keybind with contexts
    pub fn new_with_contexts(
        action_id: impl Into<String>,
        key: impl Into<String>,
        category: impl Into<String>,
        description: impl Into<String>,
        contexts: Vec<Context>,
    ) -> Self {
        Keybind {
            action_id: action_id.into(),
            key: key.into(),
            category: category.into(),
            description: description.into(),
            is_default: false,
            contexts,
        }
    }

    /// Create a new default keybind
    pub fn new_default(
        action_id: impl Into<String>,
        key: impl Into<String>,
        category: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Keybind {
            action_id: action_id.into(),
            key: key.into(),
            category: category.into(),
            description: description.into(),
            is_default: true,
            contexts: Vec::new(),
        }
    }

    /// Create a new default keybind with contexts
    pub fn new_default_with_contexts(
        action_id: impl Into<String>,
        key: impl Into<String>,
        category: impl Into<String>,
        description: impl Into<String>,
        contexts: Vec<Context>,
    ) -> Self {
        Keybind {
            action_id: action_id.into(),
            key: key.into(),
            category: category.into(),
            description: description.into(),
            is_default: true,
            contexts,
        }
    }

    /// Check if this keybind applies to the given context
    pub fn applies_to_context(&self, context: &Context) -> bool {
        // If no contexts specified, applies to global
        if self.contexts.is_empty() {
            return *context == Context::Global;
        }
        // Check if the requested context is in our contexts
        self.contexts.contains(context)
    }

    /// Check if this keybind applies to any of the given contexts
    pub fn applies_to_any_context(&self, contexts: &[Context]) -> bool {
        if self.contexts.is_empty() {
            // Global keybind applies if global is in the list
            return contexts.contains(&Context::Global);
        }
        // Check if any of our contexts match the requested contexts
        self.contexts.iter().any(|ctx| contexts.contains(ctx))
    }

    /// Get the most specific context for this keybind
    pub fn primary_context(&self) -> Context {
        self.contexts
            .iter()
            .max_by_key(|ctx| ctx.priority())
            .copied()
            .unwrap_or(Context::Global)
    }
}

/// Keybind manager trait for managing keybindings
pub trait KeybindManager {
    /// Bind an action to a key combination
    fn bind(
        &mut self,
        action: String,
        key_combo: KeyCombo,
    ) -> Result<(), crate::error::RegistryError>;
    /// Get the key binding for an action
    fn get_binding(&self, action: &str) -> Option<&Keybind>;
    /// Resolve an action from a key combination
    fn resolve_action(&self, key_combo: &KeyCombo, context: &Context) -> Option<&str>;
}
