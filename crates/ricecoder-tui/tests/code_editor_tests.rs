use ricecoder_tui::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_editor_creation() {
        let editor = CodeEditor::new();
        assert_eq!(editor.lines().len(), 1); // TextArea starts with one empty line
        assert_eq!(editor.cursor(), (0, 0));
    }

    #[test]
    fn test_code_editor_with_language() {
        let editor = CodeEditor::new().with_language("rust");
        assert_eq!(editor.language, Some("rust".to_string()));
    }

    #[test]
    fn test_code_editor_with_filename() {
        let editor = CodeEditor::new().with_filename("main.rs");
        assert_eq!(editor.filename, Some("main.rs".to_string()));
    }

    #[test]
    fn test_code_editor_set_lines() {
        let mut editor = CodeEditor::new();
        let lines = vec![
            "fn main() {".to_string(),
            "    println!(\"Hello\");".to_string(),
            "}".to_string(),
        ];
        editor.set_lines(lines.clone());
        assert_eq!(editor.lines(), lines.as_slice());
    }
}
