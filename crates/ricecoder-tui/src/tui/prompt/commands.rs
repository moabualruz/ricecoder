//! Command palette commands for prompt
//!
//! Registers prompt-related commands in the command palette:
//! - Clear prompt
//! - Submit prompt
//! - Paste (text/image)
//! - Stash/restore
//! - Open editor
//! - Interrupt session
//!
//! # DDD Layer: Application
//! Command registration for the prompt system.

use std::fmt;

/// Command category for prompt commands
pub const CATEGORY_PROMPT: &str = "Prompt";
/// Command category for session commands
pub const CATEGORY_SESSION: &str = "Session";

/// A prompt command definition
#[derive(Debug, Clone)]
pub struct PromptCommand {
    /// Command identifier
    pub id: &'static str,
    /// Display title
    pub title: &'static str,
    /// Category for grouping
    pub category: &'static str,
    /// Keybind config key (for display)
    pub keybind: Option<&'static str>,
    /// Whether command is currently disabled
    pub disabled: bool,
}

impl PromptCommand {
    /// Create a new prompt command
    pub const fn new(id: &'static str, title: &'static str, category: &'static str) -> Self {
        Self {
            id,
            title,
            category,
            keybind: None,
            disabled: false,
        }
    }

    /// With keybind
    pub const fn with_keybind(mut self, keybind: &'static str) -> Self {
        self.keybind = Some(keybind);
        self
    }

    /// Set disabled state
    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// All prompt-related commands
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PromptCommandId {
    /// Clear the prompt input
    Clear,
    /// Submit the prompt
    Submit,
    /// Paste from clipboard
    Paste,
    /// Interrupt the current session
    Interrupt,
    /// Open external editor
    OpenEditor,
    /// Stash current prompt
    Stash,
    /// Pop from stash
    StashPop,
    /// Show stash list
    StashList,
    /// Toggle shell mode
    ToggleShell,
    /// Clear input (not prompt)
    ClearInput,
    /// Exit application
    Exit,
}

impl PromptCommandId {
    /// Get command definition
    pub fn definition(&self) -> PromptCommand {
        match self {
            Self::Clear => PromptCommand::new("prompt.clear", "Clear prompt", CATEGORY_PROMPT)
                .disabled(true),
            Self::Submit => PromptCommand::new("prompt.submit", "Submit prompt", CATEGORY_PROMPT)
                .with_keybind("input_submit")
                .disabled(true),
            Self::Paste => PromptCommand::new("prompt.paste", "Paste", CATEGORY_PROMPT)
                .with_keybind("input_paste")
                .disabled(true),
            Self::Interrupt => PromptCommand::new("session.interrupt", "Interrupt session", CATEGORY_SESSION)
                .with_keybind("session_interrupt"),
            Self::OpenEditor => PromptCommand::new("prompt.editor", "Open editor", CATEGORY_SESSION)
                .with_keybind("editor_open"),
            Self::Stash => PromptCommand::new("prompt.stash", "Stash prompt", CATEGORY_PROMPT),
            Self::StashPop => PromptCommand::new("prompt.stash.pop", "Stash pop", CATEGORY_PROMPT),
            Self::StashList => PromptCommand::new("prompt.stash.list", "Stash list", CATEGORY_PROMPT),
            Self::ToggleShell => PromptCommand::new("prompt.shell", "Toggle shell mode", CATEGORY_PROMPT),
            Self::ClearInput => PromptCommand::new("prompt.clear_input", "Clear input", CATEGORY_PROMPT)
                .with_keybind("input_clear"),
            Self::Exit => PromptCommand::new("app.exit", "Exit", CATEGORY_SESSION)
                .with_keybind("app_exit"),
        }
    }

    /// Get all command IDs
    pub fn all() -> &'static [PromptCommandId] {
        &[
            Self::Clear,
            Self::Submit,
            Self::Paste,
            Self::Interrupt,
            Self::OpenEditor,
            Self::Stash,
            Self::StashPop,
            Self::StashList,
            Self::ToggleShell,
            Self::ClearInput,
            Self::Exit,
        ]
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "prompt.clear" => Some(Self::Clear),
            "prompt.submit" => Some(Self::Submit),
            "prompt.paste" => Some(Self::Paste),
            "session.interrupt" => Some(Self::Interrupt),
            "prompt.editor" => Some(Self::OpenEditor),
            "prompt.stash" => Some(Self::Stash),
            "prompt.stash.pop" => Some(Self::StashPop),
            "prompt.stash.list" => Some(Self::StashList),
            "prompt.shell" => Some(Self::ToggleShell),
            "prompt.clear_input" => Some(Self::ClearInput),
            "app.exit" => Some(Self::Exit),
            _ => None,
        }
    }
}

impl fmt::Display for PromptCommandId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.definition().id)
    }
}

/// Command execution result
#[derive(Debug, Clone)]
pub enum CommandResult {
    /// Command executed successfully
    Success,
    /// Command was handled but no action taken
    NoOp,
    /// Command requires confirmation
    NeedsConfirm(String),
    /// Command failed with error
    Error(String),
    /// Command is disabled
    Disabled,
}

/// Command context passed to command handlers
#[derive(Debug, Clone)]
pub struct CommandContext {
    /// Current prompt input text
    pub input: String,
    /// Whether prompt is focused
    pub focused: bool,
    /// Whether session is active (not idle)
    pub session_active: bool,
    /// Number of stashed prompts
    pub stash_count: usize,
    /// Current interrupt count
    pub interrupt_count: u8,
}

impl Default for CommandContext {
    fn default() -> Self {
        Self {
            input: String::new(),
            focused: true,
            session_active: false,
            stash_count: 0,
            interrupt_count: 0,
        }
    }
}

impl CommandContext {
    /// Check if a command should be disabled
    pub fn is_disabled(&self, cmd: PromptCommandId) -> bool {
        match cmd {
            PromptCommandId::Clear => self.input.is_empty(),
            PromptCommandId::Submit => self.input.trim().is_empty(),
            PromptCommandId::Paste => !self.focused,
            PromptCommandId::Interrupt => !self.session_active,
            PromptCommandId::Stash => self.input.is_empty(),
            PromptCommandId::StashPop | PromptCommandId::StashList => self.stash_count == 0,
            _ => false,
        }
    }
}

/// Get all prompt commands with current disabled state
pub fn get_commands(ctx: &CommandContext) -> Vec<PromptCommand> {
    PromptCommandId::all()
        .iter()
        .map(|id| {
            let mut cmd = id.definition();
            cmd.disabled = ctx.is_disabled(*id);
            cmd
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_command_new() {
        let cmd = PromptCommand::new("test.cmd", "Test Command", "Test");
        assert_eq!(cmd.id, "test.cmd");
        assert_eq!(cmd.title, "Test Command");
        assert_eq!(cmd.category, "Test");
        assert!(cmd.keybind.is_none());
        assert!(!cmd.disabled);
    }

    #[test]
    fn test_command_id_definition() {
        let def = PromptCommandId::Submit.definition();
        assert_eq!(def.id, "prompt.submit");
        assert_eq!(def.keybind, Some("input_submit"));
    }

    #[test]
    fn test_command_id_from_str() {
        assert_eq!(PromptCommandId::from_str("prompt.submit"), Some(PromptCommandId::Submit));
        assert_eq!(PromptCommandId::from_str("invalid"), None);
    }

    #[test]
    fn test_command_id_all() {
        let all = PromptCommandId::all();
        assert!(all.len() >= 10);
        assert!(all.contains(&PromptCommandId::Submit));
        assert!(all.contains(&PromptCommandId::Stash));
    }

    #[test]
    fn test_context_is_disabled() {
        let ctx = CommandContext {
            input: String::new(),
            session_active: false,
            stash_count: 0,
            ..Default::default()
        };

        assert!(ctx.is_disabled(PromptCommandId::Clear));
        assert!(ctx.is_disabled(PromptCommandId::Submit));
        assert!(ctx.is_disabled(PromptCommandId::Interrupt));
        assert!(ctx.is_disabled(PromptCommandId::StashPop));

        let ctx_with_input = CommandContext {
            input: "hello".to_string(),
            session_active: true,
            stash_count: 2,
            ..Default::default()
        };

        assert!(!ctx_with_input.is_disabled(PromptCommandId::Clear));
        assert!(!ctx_with_input.is_disabled(PromptCommandId::Submit));
        assert!(!ctx_with_input.is_disabled(PromptCommandId::Interrupt));
        assert!(!ctx_with_input.is_disabled(PromptCommandId::StashPop));
    }

    #[test]
    fn test_get_commands() {
        let ctx = CommandContext::default();
        let commands = get_commands(&ctx);
        assert!(!commands.is_empty());
        
        // Verify disabled states are applied
        let submit = commands.iter().find(|c| c.id == "prompt.submit").unwrap();
        assert!(submit.disabled); // Empty input
    }
}
