//! Code Action Applier
//!
//! This module provides functionality for applying code actions to code.

use crate::types::{CodeAction, TextEdit, WorkspaceEdit};
use super::{CodeActionsError, CodeActionsResult};

/// Apply a code action to code
pub fn apply_code_action(code: &str, action: &CodeAction) -> CodeActionsResult<String> {
    apply_workspace_edit(code, &action.edit)
}

/// Apply a workspace edit to code
pub fn apply_workspace_edit(code: &str, edit: &WorkspaceEdit) -> CodeActionsResult<String> {
    // For now, we'll handle single-file edits
    // In a real implementation, this would handle multi-file edits

    if edit.changes.is_empty() {
        return Ok(code.to_string());
    }

    // Get the first (and usually only) file's edits
    if let Some((_, edits)) = edit.changes.iter().next() {
        apply_text_edits(code, edits)
    } else {
        Ok(code.to_string())
    }
}

/// Apply text edits to code
pub fn apply_text_edits(code: &str, edits: &[TextEdit]) -> CodeActionsResult<String> {
    // Sort edits in reverse order to apply from end to start
    // This prevents range shifts when applying multiple edits
    let mut sorted_edits = edits.to_vec();
    sorted_edits.sort_by(|a, b| {
        // Sort by line descending, then by character descending
        match b.range.start.line.cmp(&a.range.start.line) {
            std::cmp::Ordering::Equal => b.range.start.character.cmp(&a.range.start.character),
            other => other,
        }
    });

    let mut result = code.to_string();

    for edit in sorted_edits {
        result = apply_single_text_edit(&result, &edit)?;
    }

    Ok(result)
}

/// Apply a single text edit to code
fn apply_single_text_edit(code: &str, edit: &TextEdit) -> CodeActionsResult<String> {
    // Validate the range
    let lines: Vec<&str> = code.lines().collect();

    let start_line = edit.range.start.line as usize;
    let start_char = edit.range.start.character as usize;
    let end_line = edit.range.end.line as usize;
    let end_char = edit.range.end.character as usize;

    // Check bounds
    if start_line >= lines.len() {
        return Err(CodeActionsError::ApplicationFailed(
            format!("Start line {} out of bounds", start_line),
        ));
    }

    if end_line >= lines.len() {
        return Err(CodeActionsError::ApplicationFailed(
            format!("End line {} out of bounds", end_line),
        ));
    }

    // Calculate byte positions
    let mut byte_start = 0;
    for (i, line) in lines.iter().enumerate() {
        if i == start_line {
            byte_start += start_char;
            break;
        }
        byte_start += line.len() + 1; // +1 for newline
    }

    let mut byte_end = 0;
    for (i, line) in lines.iter().enumerate() {
        if i == end_line {
            byte_end += end_char;
            break;
        }
        byte_end += line.len() + 1; // +1 for newline
    }

    // Ensure byte positions are valid
    if byte_start > code.len() || byte_end > code.len() || byte_start > byte_end {
        return Err(CodeActionsError::ApplicationFailed(
            "Invalid byte positions for edit".to_string(),
        ));
    }

    // Apply the edit
    let mut result = String::new();
    result.push_str(&code[..byte_start]);
    result.push_str(&edit.new_text);
    result.push_str(&code[byte_end..]);

    Ok(result)
}

/// Validate that an edit range is valid for the given code
pub fn validate_edit_range(code: &str, edit: &TextEdit) -> CodeActionsResult<()> {
    let lines: Vec<&str> = code.lines().collect();

    let start_line = edit.range.start.line as usize;
    let start_char = edit.range.start.character as usize;
    let end_line = edit.range.end.line as usize;
    let end_char = edit.range.end.character as usize;

    // Check line bounds
    if start_line >= lines.len() {
        return Err(CodeActionsError::ApplicationFailed(
            format!("Start line {} out of bounds (total lines: {})", start_line, lines.len()),
        ));
    }

    if end_line >= lines.len() {
        return Err(CodeActionsError::ApplicationFailed(
            format!("End line {} out of bounds (total lines: {})", end_line, lines.len()),
        ));
    }

    // Check character bounds
    if start_char > lines[start_line].len() {
        return Err(CodeActionsError::ApplicationFailed(
            format!(
                "Start character {} out of bounds (line length: {})",
                start_char,
                lines[start_line].len()
            ),
        ));
    }

    if end_char > lines[end_line].len() {
        return Err(CodeActionsError::ApplicationFailed(
            format!(
                "End character {} out of bounds (line length: {})",
                end_char,
                lines[end_line].len()
            ),
        ));
    }

    // Check that start <= end
    if start_line > end_line || (start_line == end_line && start_char > end_char) {
        return Err(CodeActionsError::ApplicationFailed(
            "Start position is after end position".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Position, Range};

    #[test]
    fn test_apply_single_text_edit_replace() {
        let code = "hello world";
        let edit = TextEdit {
            range: Range::new(Position::new(0, 0), Position::new(0, 5)),
            new_text: "goodbye".to_string(),
        };

        let result = apply_single_text_edit(code, &edit);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "goodbye world");
    }

    #[test]
    fn test_apply_single_text_edit_insert() {
        let code = "hello world";
        let edit = TextEdit {
            range: Range::new(Position::new(0, 5), Position::new(0, 5)),
            new_text: " there".to_string(),
        };

        let result = apply_single_text_edit(code, &edit);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello there world");
    }

    #[test]
    fn test_apply_single_text_edit_delete() {
        let code = "hello world";
        let edit = TextEdit {
            range: Range::new(Position::new(0, 5), Position::new(0, 11)),
            new_text: String::new(),
        };

        let result = apply_single_text_edit(code, &edit);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello");
    }

    #[test]
    fn test_validate_edit_range_valid() {
        let code = "hello\nworld";
        let edit = TextEdit {
            range: Range::new(Position::new(0, 0), Position::new(0, 5)),
            new_text: "goodbye".to_string(),
        };

        let result = validate_edit_range(code, &edit);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_edit_range_out_of_bounds() {
        let code = "hello\nworld";
        let edit = TextEdit {
            range: Range::new(Position::new(5, 0), Position::new(5, 5)),
            new_text: "test".to_string(),
        };

        let result = validate_edit_range(code, &edit);
        assert!(result.is_err());
    }

    #[test]
    fn test_apply_text_edits_multiple() {
        let code = "hello world";
        let edits = vec![
            TextEdit {
                range: Range::new(Position::new(0, 0), Position::new(0, 5)),
                new_text: "goodbye".to_string(),
            },
            TextEdit {
                range: Range::new(Position::new(0, 6), Position::new(0, 11)),
                new_text: "universe".to_string(),
            },
        ];

        let result = apply_text_edits(code, &edits);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "goodbye universe");
    }
}
