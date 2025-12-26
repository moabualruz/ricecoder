//! Bridge between crossterm key events and ricecoder-keybinds
//!
//! Converts crossterm::event::KeyEvent to ricecoder_keybinds::KeyCombo

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ricecoder_keybinds::{Key, KeyCombo, Modifier};

/// Convert a crossterm KeyEvent to a ricecoder-keybinds KeyCombo
pub fn to_key_combo(event: KeyEvent) -> Option<KeyCombo> {
    let mut modifiers = Vec::new();
    
    if event.modifiers.contains(KeyModifiers::CONTROL) {
        modifiers.push(Modifier::Ctrl);
    }
    if event.modifiers.contains(KeyModifiers::SHIFT) {
        modifiers.push(Modifier::Shift);
    }
    if event.modifiers.contains(KeyModifiers::ALT) {
        modifiers.push(Modifier::Alt);
    }
    
    let key = match event.code {
        KeyCode::Char(c) => Key::Char(c),
        KeyCode::Enter => Key::Enter,
        KeyCode::Esc => Key::Escape,
        KeyCode::Tab => Key::Tab,
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Delete => Key::Delete,
        KeyCode::Home => Key::Home,
        KeyCode::End => Key::End,
        KeyCode::PageUp => Key::PageUp,
        KeyCode::PageDown => Key::PageDown,
        KeyCode::Up => Key::Up,
        KeyCode::Down => Key::Down,
        KeyCode::Left => Key::Left,
        KeyCode::Right => Key::Right,
        KeyCode::F(n) => Key::F(n),
        KeyCode::Insert => return None, // Not supported in ricecoder-keybinds
        _ => return None, // Unsupported key
    };
    
    Some(KeyCombo {
        modifiers,
        key,
        leader: false,
    })
}

/// Convert a KeyCombo back to a display string for UI
pub fn key_combo_to_string(combo: &KeyCombo) -> String {
    let mut parts = Vec::new();
    
    for modifier in &combo.modifiers {
        parts.push(match modifier {
            Modifier::Ctrl => "Ctrl",
            Modifier::Shift => "Shift",
            Modifier::Alt => "Alt",
            Modifier::Super => "Super",
            Modifier::Meta => "Meta",
        });
    }
    
    let key_str = match &combo.key {
        Key::Char(c) => c.to_string(),
        Key::Enter => "Enter".to_string(),
        Key::Escape => "Esc".to_string(),
        Key::Tab => "Tab".to_string(),
        Key::Backspace => "Backspace".to_string(),
        Key::Delete => "Del".to_string(),
        Key::Home => "Home".to_string(),
        Key::End => "End".to_string(),
        Key::PageUp => "PgUp".to_string(),
        Key::PageDown => "PgDn".to_string(),
        Key::Up => "↑".to_string(),
        Key::Down => "↓".to_string(),
        Key::Left => "←".to_string(),
        Key::Right => "→".to_string(),
        Key::F(n) => format!("F{}", n),
    };
    
    parts.push(&key_str);
    parts.join("+")
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_key() {
        let event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        let combo = to_key_combo(event).unwrap();
        assert!(combo.modifiers.is_empty());
        assert!(matches!(combo.key, Key::Char('a')));
    }
    
    #[test]
    fn test_ctrl_key() {
        let event = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        let combo = to_key_combo(event).unwrap();
        assert!(combo.modifiers.contains(&Modifier::Ctrl));
    }
    
    #[test]
    fn test_function_key() {
        let event = KeyEvent::new(KeyCode::F(5), KeyModifiers::NONE);
        let combo = to_key_combo(event).unwrap();
        assert!(matches!(combo.key, Key::F(5)));
    }
    
    #[test]
    fn test_combo_to_string() {
        let combo = KeyCombo {
            modifiers: vec![Modifier::Ctrl, Modifier::Shift],
            key: Key::Char('a'),
            leader: false,
        };
        assert_eq!(key_combo_to_string(&combo), "Ctrl+Shift+a");
    }
}
