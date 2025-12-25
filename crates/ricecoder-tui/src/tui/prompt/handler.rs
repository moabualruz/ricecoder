//! Event handling for the prompt
//!
//! Integrates all prompt subsystems:
//! - Input with extmarks
//! - Clipboard paste processing
//! - History navigation
//! - Autocomplete triggers
//! - External editor
//! - Command execution
//!
//! # DDD Layer: Application
//! Orchestrates prompt behavior and event handling.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::clipboard::{Clipboard, PasteConfig, PastedContent};
use super::commands::{CommandContext, CommandResult, PromptCommandId};
use super::editor::{EditorConfig, EditorResult, ExternalEditor};
use super::extmarks::{Extmark, ExtmarkManager, ExtmarkStyle};
use super::input::PromptInput;
use super::parts::{AgentPart, AgentSource, FilePart, FileSource, PromptPart, TextPart, TextSource};
use super::state::{PromptInfo, PromptMode, PromptState};

/// Events emitted by the prompt handler
#[derive(Debug, Clone)]
pub enum PromptEvent {
    /// Prompt was submitted
    Submit {
        text: String,
        parts: Vec<PromptPart>,
        mode: PromptMode,
    },
    /// Shell command was submitted
    ShellSubmit { command: String },
    /// Slash command was submitted
    CommandSubmit { command: String, args: String },
    /// Request to open dialog
    OpenDialog(DialogRequest),
    /// Request to show toast
    ShowToast { message: String, variant: ToastVariant },
    /// Exit requested
    Exit,
    /// Interrupt session
    Interrupt { abort: bool },
    /// Content changed
    ContentChanged,
    /// Mode changed
    ModeChanged(PromptMode),
    /// Focus changed
    FocusChanged(bool),
}

/// Dialog requests
#[derive(Debug, Clone)]
pub enum DialogRequest {
    CommandPalette,
    ProviderSelect,
    AgentSelect,
    StashList,
}

/// Toast variants
#[derive(Debug, Clone, Copy)]
pub enum ToastVariant {
    Info,
    Warning,
    Error,
    Success,
}

/// Keybind configuration for prompt
#[derive(Debug, Clone)]
pub struct KeybindConfig {
    /// Submit keybind
    pub submit: Vec<KeyEvent>,
    /// Newline keybind
    pub newline: Vec<KeyEvent>,
    /// Paste keybind
    pub paste: Vec<KeyEvent>,
    /// Clear input keybind
    pub clear: Vec<KeyEvent>,
    /// Open editor keybind
    pub editor: Vec<KeyEvent>,
    /// History previous
    pub history_prev: Vec<KeyEvent>,
    /// History next
    pub history_next: Vec<KeyEvent>,
    /// Interrupt session
    pub interrupt: Vec<KeyEvent>,
    /// Exit application
    pub exit: Vec<KeyEvent>,
}

impl Default for KeybindConfig {
    fn default() -> Self {
        Self {
            submit: vec![KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)],
            newline: vec![KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT)],
            paste: vec![KeyEvent::new(KeyCode::Char('v'), KeyModifiers::CONTROL)],
            clear: vec![KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL)],
            editor: vec![KeyEvent::new(KeyCode::Char('e'), KeyModifiers::CONTROL)],
            history_prev: vec![KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)],
            history_next: vec![KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)],
            interrupt: vec![KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)],
            exit: vec![KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL)],
        }
    }
}

impl KeybindConfig {
    /// Check if key matches any keybind in list
    fn matches(bindings: &[KeyEvent], key: &KeyEvent) -> bool {
        bindings.iter().any(|b| b.code == key.code && b.modifiers == key.modifiers)
    }
}

/// The prompt handler - orchestrates all prompt behavior
pub struct PromptHandler<'a> {
    /// Input component
    input: PromptInput<'a>,
    /// Extmark manager
    extmarks: ExtmarkManager,
    /// Prompt state
    state: PromptState,
    /// Paste configuration
    paste_config: PasteConfig,
    /// Keybind configuration
    keybind_config: KeybindConfig,
    /// Registered extmark type ID
    prompt_part_type_id: u32,
    /// History entries (external - just index tracking)
    history_index: Option<usize>,
    /// Pending events
    pending_events: Vec<PromptEvent>,
    /// Whether autocomplete is visible
    autocomplete_visible: bool,
    /// Session ID
    session_id: Option<String>,
}

impl<'a> PromptHandler<'a> {
    /// Create a new prompt handler
    pub fn new() -> Self {
        let mut extmarks = ExtmarkManager::new();
        let prompt_part_type_id = extmarks.register_type("prompt-part");

        Self {
            input: PromptInput::new(),
            extmarks,
            state: PromptState::new(),
            paste_config: PasteConfig::default(),
            keybind_config: KeybindConfig::default(),
            prompt_part_type_id,
            history_index: None,
            pending_events: Vec::new(),
            autocomplete_visible: false,
            session_id: None,
        }
    }

    /// Set session ID
    pub fn set_session_id(&mut self, id: Option<String>) {
        self.session_id = id;
    }

    /// Get current state
    pub fn state(&self) -> &PromptState {
        &self.state
    }

    /// Get mutable state
    pub fn state_mut(&mut self) -> &mut PromptState {
        &mut self.state
    }

    /// Get input
    pub fn input(&self) -> &PromptInput<'a> {
        &self.input
    }

    /// Get mutable input
    pub fn input_mut(&mut self) -> &mut PromptInput<'a> {
        &mut self.input
    }

    /// Get extmarks
    pub fn extmarks(&self) -> &ExtmarkManager {
        &self.extmarks
    }

    /// Set autocomplete visibility
    pub fn set_autocomplete_visible(&mut self, visible: bool) {
        self.autocomplete_visible = visible;
    }

    /// Drain pending events
    pub fn drain_events(&mut self) -> Vec<PromptEvent> {
        std::mem::take(&mut self.pending_events)
    }

    /// Handle key event
    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        // Check if disabled
        if self.state.disabled {
            return false;
        }

        // Check autocomplete first
        if self.autocomplete_visible {
            // Let autocomplete handle it
            return false;
        }

        // Shell mode toggle - ! at start
        if key.code == KeyCode::Char('!') && self.input.cursor().offset == 0 {
            self.state.mode = PromptMode::Shell;
            self.pending_events.push(PromptEvent::ModeChanged(PromptMode::Shell));
            return true;
        }

        // Shell mode exit
        if self.state.mode == PromptMode::Shell {
            if key.code == KeyCode::Esc
                || (key.code == KeyCode::Backspace && self.input.cursor().offset == 0)
            {
                self.state.mode = PromptMode::Normal;
                self.pending_events.push(PromptEvent::ModeChanged(PromptMode::Normal));
                return true;
            }
        }

        // Submit
        if KeybindConfig::matches(&self.keybind_config.submit, &key) {
            return self.submit();
        }

        // Newline
        if KeybindConfig::matches(&self.keybind_config.newline, &key) {
            self.input.widget_mut().insert_newline();
            self.sync_state();
            return true;
        }

        // Clear
        if KeybindConfig::matches(&self.keybind_config.clear, &key) {
            if !self.state.prompt.input.is_empty() {
                self.clear();
                return true;
            }
        }

        // Exit
        if KeybindConfig::matches(&self.keybind_config.exit, &key) {
            if self.state.prompt.input.is_empty() {
                self.pending_events.push(PromptEvent::Exit);
                return true;
            }
        }

        // Interrupt
        if KeybindConfig::matches(&self.keybind_config.interrupt, &key) {
            let abort = self.state.increment_interrupt();
            self.pending_events.push(PromptEvent::Interrupt { abort });
            // Reset after timeout would be handled externally
            return true;
        }

        // History navigation
        if KeybindConfig::matches(&self.keybind_config.history_prev, &key) {
            if self.input.cursor().offset == 0 {
                // Signal to navigate history - actual history handled externally
                return false; // Let parent handle
            }
        }
        if KeybindConfig::matches(&self.keybind_config.history_next, &key) {
            let text_len = self.input.plain_text().len();
            if self.input.cursor().offset == text_len {
                return false; // Let parent handle
            }
        }

        // Default: pass to input
        let handled = self.input.handle_input(key.into());
        if handled {
            self.sync_state();
        }
        handled
    }

    /// Handle paste event
    pub fn handle_paste(&mut self, text: &str) -> bool {
        if self.state.disabled {
            return false;
        }

        let image_count = self.state.prompt.file_parts().count();
        let content = Clipboard::process_text(text, &self.paste_config, image_count);

        match content {
            PastedContent::PlainText(text) => {
                self.input.insert_text(&text);
                self.sync_state();
            }
            PastedContent::SummarizedText {
                virtual_text,
                full_text,
                line_count: _,
            } => {
                self.insert_text_part(full_text, virtual_text);
            }
            PastedContent::Image {
                mime,
                data,
                filename,
                virtual_text: _,
            } => {
                self.insert_file_part(mime, data, filename);
            }
            PastedContent::SvgText {
                virtual_text,
                content,
                filename: _,
            } => {
                self.insert_text_part(content, virtual_text);
            }
        }

        self.pending_events.push(PromptEvent::ContentChanged);
        true
    }

    /// Handle image paste
    pub fn handle_image_paste(&mut self, mime: String, data: String, filename: Option<String>) -> bool {
        if self.state.disabled {
            return false;
        }

        self.insert_file_part(mime, data, filename);
        self.pending_events.push(PromptEvent::ContentChanged);
        true
    }

    /// Insert a text part with virtual text
    fn insert_text_part(&mut self, text: String, virtual_text: String) {
        let offset = self.input.cursor().offset;
        let start = offset;
        let end = start + virtual_text.len();

        // Insert virtual text + space
        self.input.insert_text(&format!("{} ", virtual_text));

        // Create extmark
        let extmark_id = self.extmarks.create(
            start,
            end,
            virtual_text.clone(),
            ExtmarkStyle::Paste,
            self.prompt_part_type_id,
        );

        // Add part to state
        let part = PromptPart::Text(TextPart {
            text,
            source: Some(TextSource::new(start, end, virtual_text)),
        });
        self.state.prompt.parts.push(part);
        self.state.register_extmark(extmark_id, self.state.prompt.parts.len() - 1);

        self.sync_state();
    }

    /// Insert a file part
    fn insert_file_part(&mut self, mime: String, data: String, filename: Option<String>) {
        let offset = self.input.cursor().offset;
        let image_count = self.state.prompt.file_parts().count();
        let virtual_text = super::clipboard::Clipboard::image_virtual_text(image_count);

        let start = offset;
        let end = start + virtual_text.len();

        // Insert virtual text + space
        self.input.insert_text(&format!("{} ", virtual_text));

        // Create extmark
        let extmark_id = self.extmarks.create(
            start,
            end,
            virtual_text.clone(),
            ExtmarkStyle::File,
            self.prompt_part_type_id,
        );

        // Add part to state
        let part = PromptPart::File(FilePart {
            mime,
            filename,
            url: format!("data:{};base64,{}", data, data),
            source: Some(FileSource {
                source_type: "file".to_string(),
                path: String::new(),
                text: Some(TextSource::new(start, end, virtual_text)),
            }),
        });
        self.state.prompt.parts.push(part);
        self.state.register_extmark(extmark_id, self.state.prompt.parts.len() - 1);

        self.sync_state();
    }

    /// Insert an agent mention
    pub fn insert_agent(&mut self, agent_name: &str) {
        let offset = self.input.cursor().offset;
        let virtual_text = format!("@{}", agent_name);
        let start = offset;
        let end = start + virtual_text.len();

        self.input.insert_text(&format!("{} ", virtual_text));

        let extmark_id = self.extmarks.create(
            start,
            end,
            virtual_text.clone(),
            ExtmarkStyle::Agent,
            self.prompt_part_type_id,
        );

        let part = PromptPart::Agent(AgentPart {
            name: agent_name.to_string(),
            source: Some(AgentSource::new(start, end, virtual_text)),
        });
        self.state.prompt.parts.push(part);
        self.state.register_extmark(extmark_id, self.state.prompt.parts.len() - 1);

        self.sync_state();
    }

    /// Submit the prompt
    fn submit(&mut self) -> bool {
        if !self.state.can_submit() {
            return false;
        }

        // Check for special commands
        let input = self.state.prompt.input.trim();

        // Exit commands
        if input == "exit" || input == "quit" || input == ":q" {
            self.pending_events.push(PromptEvent::Exit);
            return true;
        }

        // Shell mode
        if self.state.mode == PromptMode::Shell {
            self.pending_events.push(PromptEvent::ShellSubmit {
                command: self.expand_text_parts(),
            });
        }
        // Slash command
        else if input.starts_with('/') {
            let parts: Vec<&str> = input.splitn(2, ' ').collect();
            let command = parts[0].trim_start_matches('/').to_string();
            let args = parts.get(1).map(|s| s.to_string()).unwrap_or_default();
            self.pending_events.push(PromptEvent::CommandSubmit { command, args });
        }
        // Normal submission
        else {
            self.pending_events.push(PromptEvent::Submit {
                text: self.expand_text_parts(),
                parts: self.state.prompt.non_text_parts().into_iter().cloned().collect(),
                mode: self.state.mode,
            });
        }

        // Clear after submit
        self.clear();
        true
    }

    /// Expand all text parts inline
    fn expand_text_parts(&self) -> String {
        self.state.prompt.expand_text_parts()
    }

    /// Clear the prompt
    pub fn clear(&mut self) {
        self.input.clear();
        self.extmarks.clear();
        self.state.reset();
        self.history_index = None;
        self.pending_events.push(PromptEvent::ContentChanged);
    }

    /// Set prompt from info (e.g., from history)
    pub fn set(&mut self, info: PromptInfo) {
        self.input.set_text(&info.input);
        self.state.set_from_history(info.clone());
        self.restore_extmarks_from_parts(&info.parts);
        self.input.goto_buffer_end();
    }

    /// Restore extmarks from parts
    fn restore_extmarks_from_parts(&mut self, parts: &[PromptPart]) {
        self.extmarks.clear();
        self.state.extmark_to_part_index.clear();

        for (part_index, part) in parts.iter().enumerate() {
            let (start, end, virtual_text, style) = match part {
                PromptPart::Text(p) => {
                    if let Some(src) = &p.source {
                        (src.start, src.end, src.value.clone(), ExtmarkStyle::Paste)
                    } else {
                        continue;
                    }
                }
                PromptPart::File(p) => {
                    if let Some(src) = &p.source {
                        if let Some(text) = &src.text {
                            (text.start, text.end, text.value.clone(), ExtmarkStyle::File)
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    }
                }
                PromptPart::Agent(p) => {
                    if let Some(src) = &p.source {
                        (src.start, src.end, src.value.clone(), ExtmarkStyle::Agent)
                    } else {
                        continue;
                    }
                }
            };

            let extmark_id = self.extmarks.create(
                start,
                end,
                virtual_text,
                style,
                self.prompt_part_type_id,
            );
            self.state.register_extmark(extmark_id, part_index);
        }
    }

    /// Sync state from input
    fn sync_state(&mut self) {
        self.state.prompt.input = self.input.plain_text();

        // Sync extmarks with parts
        self.extmarks.sync_with_parts(|extmark_id, start, end| {
            if let Some(part_index) = self.state.get_part_for_extmark(extmark_id) {
                if let Some(part) = self.state.prompt.parts.get_mut(part_index) {
                    part.shift_source(0); // Update would require more complex logic
                }
            }
        });
    }

    /// Open external editor
    pub fn open_editor(&mut self, new_prompt: bool) {
        let content = if new_prompt {
            String::new()
        } else {
            self.expand_text_parts()
        };

        let config = EditorConfig::with_content(content);
        match ExternalEditor::open(&config) {
            EditorResult::Modified(new_content) => {
                self.input.set_text(&new_content);
                // Keep non-text parts, clear text parts
                let non_text_parts: Vec<_> = self.state.prompt
                    .parts
                    .iter()
                    .filter(|p| !p.is_text())
                    .cloned()
                    .collect();
                self.state.prompt.input = new_content;
                self.state.prompt.parts = non_text_parts;
                // Would need to update extmark positions based on new content
                self.input.goto_buffer_end();
                self.pending_events.push(PromptEvent::ContentChanged);
            }
            EditorResult::Cancelled => {}
            EditorResult::Error(e) => {
                self.pending_events.push(PromptEvent::ShowToast {
                    message: format!("Editor error: {}", e),
                    variant: ToastVariant::Error,
                });
            }
        }
    }

    /// Focus the prompt
    pub fn focus(&mut self) {
        self.input.focus();
        self.state.focused = true;
        self.pending_events.push(PromptEvent::FocusChanged(true));
    }

    /// Blur the prompt
    pub fn blur(&mut self) {
        self.input.blur();
        self.state.focused = false;
        self.pending_events.push(PromptEvent::FocusChanged(false));
    }

    /// Get command context for command palette
    pub fn command_context(&self) -> CommandContext {
        CommandContext {
            input: self.state.prompt.input.clone(),
            focused: self.state.focused,
            session_active: false, // Would be set externally
            stash_count: 0,        // Would be set externally
            interrupt_count: self.state.interrupt_count,
        }
    }
}

impl<'a> Default for PromptHandler<'a> {
    fn default() -> Self {
        Self::new()
    }
}

/// PromptRef - external control interface (like React ref)
pub struct PromptRef<'a> {
    handler: &'a mut PromptHandler<'a>,
}

impl<'a> PromptRef<'a> {
    /// Check if focused
    pub fn focused(&self) -> bool {
        self.handler.state.focused
    }

    /// Get current prompt info
    pub fn current(&self) -> &PromptInfo {
        &self.handler.state.prompt
    }

    /// Set prompt
    pub fn set(&mut self, info: PromptInfo) {
        self.handler.set(info);
    }

    /// Reset prompt
    pub fn reset(&mut self) {
        self.handler.clear();
    }

    /// Focus prompt
    pub fn focus(&mut self) {
        self.handler.focus();
    }

    /// Blur prompt
    pub fn blur(&mut self) {
        self.handler.blur();
    }

    /// Submit prompt
    pub fn submit(&mut self) -> bool {
        self.handler.submit()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_handler_new() {
        let handler = PromptHandler::new();
        assert!(handler.state().prompt.is_empty());
        assert_eq!(handler.state().mode, PromptMode::Normal);
    }

    #[test]
    fn test_shell_mode_toggle() {
        let mut handler = PromptHandler::new();

        // Type ! at start
        let key = KeyEvent::new(KeyCode::Char('!'), KeyModifiers::NONE);
        handler.handle_key(key);

        assert_eq!(handler.state().mode, PromptMode::Shell);

        // Escape to exit
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        handler.handle_key(key);

        assert_eq!(handler.state().mode, PromptMode::Normal);
    }

    #[test]
    fn test_handle_paste_plain() {
        let mut handler = PromptHandler::new();
        handler.handle_paste("hello world");

        assert_eq!(handler.state().prompt.input, "hello world");
    }

    #[test]
    fn test_handle_paste_summarized() {
        let mut handler = PromptHandler::new();
        let long_text = "line1\nline2\nline3\nline4\nline5";
        handler.handle_paste(long_text);

        // Should create a text part with virtual text
        assert!(!handler.state().prompt.parts.is_empty());
        assert!(handler.state().prompt.input.contains("[Pasted"));
    }

    #[test]
    fn test_insert_agent() {
        let mut handler = PromptHandler::new();
        handler.insert_agent("build");

        assert!(handler.state().prompt.input.contains("@build"));
        assert_eq!(handler.state().prompt.parts.len(), 1);
    }

    #[test]
    fn test_clear() {
        let mut handler = PromptHandler::new();
        handler.handle_paste("hello");
        handler.clear();

        assert!(handler.state().prompt.is_empty());
        assert_eq!(handler.extmarks().all().count(), 0);
    }

    #[test]
    fn test_keybind_config_default() {
        let config = KeybindConfig::default();
        assert!(!config.submit.is_empty());
        assert!(!config.interrupt.is_empty());
    }

    #[test]
    fn test_events_drain() {
        let mut handler = PromptHandler::new();
        handler.clear();

        let events = handler.drain_events();
        assert!(!events.is_empty());

        // Second drain should be empty
        let events2 = handler.drain_events();
        assert!(events2.is_empty());
    }
}
