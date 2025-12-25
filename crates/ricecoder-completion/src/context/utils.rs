// Position and offset conversion utilities

use crate::types::Position;

/// Convert a Position to a byte offset in the code
pub fn position_to_byte_offset(code: &str, position: Position) -> usize {
    let mut byte_offset = 0;
    let mut current_line = 0;
    let mut current_char = 0;

    for ch in code.chars() {
        if current_line == position.line && current_char == position.character {
            return byte_offset;
        }

        byte_offset += ch.len_utf8();

        if ch == '\n' {
            current_line += 1;
            current_char = 0;
        } else {
            current_char += 1;
        }
    }

    byte_offset
}

/// Convert a byte offset to a Position in the code
pub fn byte_offset_to_position(code: &str, byte_offset: usize) -> Position {
    let mut current_line = 0;
    let mut current_char = 0;
    let mut current_byte = 0;

    for ch in code.chars() {
        if current_byte >= byte_offset {
            return Position::new(current_line, current_char);
        }

        current_byte += ch.len_utf8();

        if ch == '\n' {
            current_line += 1;
            current_char = 0;
        } else {
            current_char += 1;
        }
    }

    Position::new(current_line, current_char)
}

/// Extract the prefix (partial word) at the cursor position
pub fn extract_prefix(code: &str, position: Position) -> String {
    let byte_offset = position_to_byte_offset(code, position);
    let mut prefix = String::new();

    // Walk backwards from the position to find the start of the word
    for ch in code[..byte_offset].chars().rev() {
        if ch.is_alphanumeric() || ch == '_' {
            prefix.insert(0, ch);
        } else {
            break;
        }
    }

    prefix
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_to_byte_offset() {
        let code = "hello\nworld";
        let pos = Position::new(1, 2);
        let offset = position_to_byte_offset(code, pos);
        assert_eq!(offset, 8); // "hello\nwo"
    }

    #[test]
    fn test_byte_offset_to_position() {
        let code = "hello\nworld";
        let pos = byte_offset_to_position(code, 8);
        assert_eq!(pos, Position::new(1, 2));
    }

    #[test]
    fn test_extract_prefix() {
        let code = "let my_var = ";
        let pos = Position::new(0, 13);
        let prefix = extract_prefix(code, pos);
        assert_eq!(prefix, "");
    }

    #[test]
    fn test_extract_prefix_with_word() {
        let code = "let my_var = my";
        let pos = Position::new(0, 15);
        let prefix = extract_prefix(code, pos);
        assert_eq!(prefix, "my");
    }
}
