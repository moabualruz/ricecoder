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
            _ => Err(ParseError::InvalidKeySyntax(format!(
                "Unknown key: {}",
                s
            ))),
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

/// Represents a single keybind configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Keybind {
    pub action_id: String,
    pub key: String,
    pub category: String,
    pub description: String,
    #[serde(default)]
    pub is_default: bool,
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modifier_from_str() {
        assert_eq!(Modifier::from_str("ctrl").unwrap(), Modifier::Ctrl);
        assert_eq!(Modifier::from_str("Shift").unwrap(), Modifier::Shift);
        assert_eq!(Modifier::from_str("alt").unwrap(), Modifier::Alt);
        assert_eq!(Modifier::from_str("meta").unwrap(), Modifier::Meta);
        assert_eq!(Modifier::from_str("cmd").unwrap(), Modifier::Meta);
    }

    #[test]
    fn test_key_from_str() {
        assert_eq!(Key::from_str("enter").unwrap(), Key::Enter);
        assert_eq!(Key::from_str("F1").unwrap(), Key::F(1));
        assert_eq!(Key::from_str("a").unwrap(), Key::Char('a'));
        assert!(Key::from_str("F13").is_err());
    }

    #[test]
    fn test_key_combo_from_str() {
        let combo = KeyCombo::from_str("Ctrl+S").unwrap();
        assert_eq!(combo.modifiers.len(), 1);
        assert_eq!(combo.modifiers[0], Modifier::Ctrl);
        assert_eq!(combo.key, Key::Char('s'));

        let combo = KeyCombo::from_str("Ctrl+Shift+Z").unwrap();
        assert_eq!(combo.modifiers.len(), 2);
        assert_eq!(combo.key, Key::Char('z'));
    }

    #[test]
    fn test_keybind_creation() {
        let kb = Keybind::new("editor.save", "Ctrl+S", "editing", "Save file");
        assert_eq!(kb.action_id, "editor.save");
        assert_eq!(kb.key, "Ctrl+S");
        assert!(!kb.is_default);

        let kb = Keybind::new_default("editor.undo", "Ctrl+Z", "editing", "Undo");
        assert!(kb.is_default);
    }
}
