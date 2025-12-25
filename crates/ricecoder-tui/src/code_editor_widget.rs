//! Code editor widget with edtui wrapper for full vim editing support
//!
//! This module wraps edtui's EditorState and EditorView to provide a vim-capable
//! code editor widget compatible with ratatui 0.29.

use edtui::{EditorEventHandler, EditorState, EditorView, Lines};
use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyEvent as CrosstermKeyEvent,
    layout::Rect,
    widgets::{Block, Borders, Widget},
};

/// Syntax highlighting theme (re-exported for API compatibility)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxTheme {
    Dark,
    Light,
    Monokai,
}

/// Supported programming languages (re-exported for API compatibility)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Go,
    Java,
    Kotlin,
    C,
    Shell,
    Yaml,
    Json,
    Markdown,
    Sql,
    Html,
    Css,
    Php,
    Ruby,
    Swift,
    Scala,
    Cpp,
    PlainText,
    Unknown,
}

impl Language {
    /// Detect language from file extension
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "rs" => Language::Rust,
            "py" => Language::Python,
            "js" => Language::JavaScript,
            "ts" => Language::TypeScript,
            "go" => Language::Go,
            "java" => Language::Java,
            "c" | "cpp" | "cc" | "cxx" | "h" | "hpp" => Language::C,
            "sh" | "bash" => Language::Shell,
            "yaml" | "yml" => Language::Yaml,
            "json" => Language::Json,
            "md" | "markdown" => Language::Markdown,
            "sql" => Language::Sql,
            "html" | "htm" => Language::Html,
            "css" => Language::Css,
            _ => Language::PlainText,
        }
    }

    /// Get the language name
    pub fn name(&self) -> &'static str {
        match self {
            Language::Rust => "Rust",
            Language::Python => "Python",
            Language::JavaScript => "JavaScript",
            Language::TypeScript => "TypeScript",
            Language::Go => "Go",
            Language::Java => "Java",
            Language::Kotlin => "Kotlin",
            Language::C => "C/C++",
            Language::Shell => "Shell",
            Language::Yaml => "YAML",
            Language::Json => "JSON",
            Language::Markdown => "Markdown",
            Language::Sql => "SQL",
            Language::Html => "HTML",
            Language::Css => "CSS",
            Language::Php => "PHP",
            Language::Ruby => "Ruby",
            Language::Swift => "Swift",
            Language::Scala => "Scala",
            Language::Cpp => "C++",
            Language::Unknown => "Unknown",
            Language::PlainText => "Plain Text",
        }
    }
}

/// Code line (re-exported for API compatibility)
#[derive(Debug, Clone)]
pub struct CodeLine {
    pub line_number: usize,
    pub content: String,
    pub language: Language,
    pub highlighted: bool,
}

impl CodeLine {
    pub fn new(line_number: usize, content: impl Into<String>, language: Language) -> Self {
        Self {
            line_number,
            content: content.into(),
            language,
            highlighted: false,
        }
    }
}

/// Code editor widget wrapping edtui for full vim editing capabilities
pub struct CodeEditorWidget {
    editor_state: EditorState,
    event_handler: EditorEventHandler,
    title: String,
    show_borders: bool,
}

impl CodeEditorWidget {
    /// Create a new code editor widget
    pub fn new(_language: Language) -> Self {
        Self {
            editor_state: EditorState::default(),
            event_handler: EditorEventHandler::default(),
            title: "Code".to_string(),
            show_borders: true,
        }
    }

    /// Set the code content
    pub fn set_content(&mut self, content: &str) {
        self.editor_state = EditorState::new(Lines::from(content));
    }

    /// Get the code content
    pub fn get_content(&self) -> String {
        // Convert Lines back to String
        // Note: edtui doesn't expose this directly, so we store content separately
        // For now, return empty string - full implementation would track content separately
        String::new()
    }

    /// Handle key event for vim keybindings
    pub fn handle_key_event(&mut self, key: CrosstermKeyEvent) {
        // Convert crossterm KeyEvent to edtui KeyEvent
        let edtui_key = edtui::events::KeyEvent::from(key);
        self.event_handler.on_key_event(edtui_key, &mut self.editor_state);
    }

    /// Set the title
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
    }

    /// Set whether to show borders
    pub fn set_show_borders(&mut self, show: bool) {
        self.show_borders = show;
    }

    /// Get the title
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Get mutable reference to editor state for advanced operations
    pub fn editor_state_mut(&mut self) -> &mut EditorState {
        &mut self.editor_state
    }

    /// Get reference to editor state
    pub fn editor_state(&self) -> &EditorState {
        &self.editor_state
    }
}

impl Default for CodeEditorWidget {
    fn default() -> Self {
        Self::new(Language::PlainText)
    }
}

impl Widget for &mut CodeEditorWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut block = Block::default().title(self.title.as_str());

        if self.show_borders {
            block = block.borders(Borders::ALL);
        }

        let inner_area = block.inner(area);
        block.render(area, buf);

        // Render the edtui editor view
        let editor_view = EditorView::new(&mut self.editor_state);
        editor_view.render(inner_area, buf);
    }
}
