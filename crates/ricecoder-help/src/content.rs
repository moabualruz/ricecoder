//! Help content data structures and management

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// A single help item with title and content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HelpItem {
    pub title: String,
    pub content: String,
    pub keywords: Vec<String>,
}

impl HelpItem {
    /// Create a new help item
    pub fn new(title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            content: content.into(),
            keywords: Vec::new(),
        }
    }

    /// Add keywords for search
    pub fn with_keywords(mut self, keywords: Vec<String>) -> Self {
        self.keywords = keywords;
        self
    }

    /// Check if this item matches a search query
    pub fn matches(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();

        // Check title
        if self.title.to_lowercase().contains(&query_lower) {
            return true;
        }

        // Check content
        if self.content.to_lowercase().contains(&query_lower) {
            return true;
        }

        // Check keywords
        for keyword in &self.keywords {
            if keyword.to_lowercase().contains(&query_lower) {
                return true;
            }
        }

        false
    }
}

/// A category of help items
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HelpCategory {
    pub name: String,
    pub description: String,
    pub items: Vec<HelpItem>,
}

impl HelpCategory {
    /// Create a new help category
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            items: Vec::new(),
        }
    }

    /// Set category description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Add a help item to this category
    pub fn add_item(mut self, title: impl Into<String>, content: impl Into<String>) -> Self {
        self.items.push(HelpItem::new(title, content));
        self
    }

    /// Add a help item with keywords
    pub fn add_item_with_keywords(
        mut self,
        title: impl Into<String>,
        content: impl Into<String>,
        keywords: Vec<String>,
    ) -> Self {
        self.items
            .push(HelpItem::new(title, content).with_keywords(keywords));
        self
    }

    /// Search for items in this category
    pub fn search(&self, query: &str) -> Vec<&HelpItem> {
        self.items
            .iter()
            .filter(|item| item.matches(query))
            .collect()
    }
}

/// Complete help content with all categories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HelpContent {
    pub categories: Vec<HelpCategory>,
    pub shortcuts: HashMap<String, String>,
}

impl HelpContent {
    /// Create new empty help content
    pub fn new() -> Self {
        Self {
            categories: Vec::new(),
            shortcuts: HashMap::new(),
        }
    }

    /// Add a category to the help content
    pub fn add_category(mut self, category: HelpCategory) -> Self {
        self.categories.push(category);
        self
    }

    /// Add a keyboard shortcut
    pub fn add_shortcut(mut self, key: impl Into<String>, description: impl Into<String>) -> Self {
        self.shortcuts.insert(key.into(), description.into());
        self
    }

    /// Search across all categories
    pub fn search(&self, query: &str) -> Vec<(&HelpCategory, &HelpItem)> {
        let mut results = Vec::new();

        for category in &self.categories {
            for item in category.search(query) {
                results.push((category, item));
            }
        }

        results
    }

    /// Get a category by name
    pub fn get_category(&self, name: &str) -> Option<&HelpCategory> {
        self.categories.iter().find(|cat| cat.name == name)
    }

    /// Create default help content for RiceCoder
    pub fn default_ricecoder_help() -> Self {
        Self::new()
            .add_category(
                HelpCategory::new("Getting Started")
                    .with_description("Basic information to get you started with RiceCoder")
                    .add_item(
                        "Welcome to RiceCoder",
                        "RiceCoder is a terminal-based AI coding assistant that helps you write, debug, and refactor code efficiently."
                    )
                    .add_item(
                        "Basic Usage",
                        "Type your message in the input area and press Enter to send. The AI will respond with helpful suggestions and code."
                    )
                    .add_item(
                        "File References",
                        "Use @filename to reference files in your messages. This includes the file content in your conversation."
                    )
                    .add_item(
                        "Bash Commands",
                        "Use !command to execute bash commands. The output will be displayed in the chat."
                    )
            )
            .add_category(
                HelpCategory::new("Commands")
                    .with_description("Available slash commands for quick actions")
                    .add_item("/help", "Show this help dialog")
                    .add_item("/models", "List and select available AI models")
                    .add_item("/exit or /quit", "Exit RiceCoder")
                    .add_item("/new", "Create a new session")
                    .add_item("/sessions", "List and switch between sessions")
                    .add_item("/themes", "List and select themes")
                    .add_item("/compact", "Compact current session to reduce token usage")
                    .add_item("/export", "Export session to Markdown")
                    .add_item("/undo", "Undo the last message")
                    .add_item("/redo", "Redo the last undone message")
                    .add_item("/details", "Toggle tool execution details visibility")
                    .add_item("/clear", "Clear current session messages")
                    .add_item("/rename", "Rename current session")
                    .add_item("/delete", "Delete current session")
                    .add_item("/copy", "Copy last message to clipboard")
                    .add_item("/settings", "Open settings interface")
                    .add_item("/debug", "Toggle debug mode")
            )
            .add_category(
                HelpCategory::new("Keyboard Shortcuts")
                    .with_description("Keyboard shortcuts for efficient navigation")
                    .add_item("F1", "Show help dialog")
                    .add_item("Ctrl+M", "List available models")
                    .add_item("Ctrl+Q", "Exit RiceCoder")
                    .add_item("Ctrl+N", "Create new session")
                    .add_item("Ctrl+S", "List sessions")
                    .add_item("Ctrl+T", "List themes")
                    .add_item("Ctrl+E", "Export session to Markdown")
                    .add_item("Ctrl+Z", "Undo last message")
                    .add_item("Ctrl+Y", "Redo last undone message")
                    .add_item("Ctrl+D", "Toggle tool execution details")
                    .add_item("Ctrl+F", "Search in help (when help is open)")
                    .add_item("Escape", "Close dialogs or cancel operations")
                    .add_item("Enter", "Send message or confirm action")
                    .add_item("Shift+Enter", "Insert newline in input")
                    .add_item("Page Up/Down", "Scroll through messages")
                    .add_item("Home/End", "Jump to top/bottom of messages")
            )
            .add_category(
                HelpCategory::new("File Operations")
                    .with_description("Working with files and directories")
                    .add_item(
                        "File References (@)",
                        "Type @ followed by a filename to include file content in your message. Use Tab for autocomplete."
                    )
                    .add_item(
                        "File Picker",
                        "When typing @, a file picker will appear. Use arrow keys to navigate and Enter to select."
                    )
                    .add_item(
                        "Multiple Files",
                        "You can reference multiple files in a single message by using @ multiple times."
                    )
                    .add_item(
                        "Large Files",
                        "Files larger than 1MB will show a warning. You can choose to include them or select specific line ranges."
                    )
                    .add_item(
                        "Binary Files",
                        "Binary files are automatically detected and excluded from inclusion to prevent display issues."
                    )
            )
            .add_category(
                HelpCategory::new("Session Management")
                    .with_description("Managing your conversation sessions")
                    .add_item(
                        "Creating Sessions",
                        "Use /new to create a new session. Each session maintains its own conversation history."
                    )
                    .add_item(
                        "Switching Sessions",
                        "Use /sessions to see all sessions and switch between them. Sessions are automatically saved."
                    )
                    .add_item(
                        "Session Compaction",
                        "Use /compact to reduce token usage by summarizing older messages while keeping recent ones."
                    )
                    .add_item(
                        "Exporting Sessions",
                        "Use /export to save your session as a Markdown file for sharing or archiving."
                    )
                    .add_item(
                        "Undo/Redo",
                        "Use /undo and /redo (or Ctrl+Z/Ctrl+Y) to undo and redo messages in your session."
                    )
            )
            .add_category(
                HelpCategory::new("Customization")
                    .with_description("Personalizing your RiceCoder experience")
                    .add_item(
                        "Themes",
                        "Use /themes to browse and select from 30+ built-in themes including Dracula, Nord, Tokyo Night, and more."
                    )
                    .add_item(
                        "Vim Mode",
                        "Enable vim-style keybindings in settings for familiar navigation (hjkl, dd, yy, etc.)."
                    )
                    .add_item(
                        "Configuration",
                        "Customize RiceCoder through configuration files in ~/.ricecoder/ or project-specific .ricecoder/ directories."
                    )
                    .add_item(
                        "Custom Commands",
                        "Define custom slash commands in your configuration for frequently used operations."
                    )
            )
            .add_category(
                HelpCategory::new("Troubleshooting")
                    .with_description("Common issues and solutions")
                    .add_item(
                        "Connection Issues",
                        "If you can't connect to AI providers, check your API keys in settings and verify internet connectivity."
                    )
                    .add_item(
                        "Performance Issues",
                        "For slow performance, try compacting sessions (/compact) or reducing the number of referenced files."
                    )
                    .add_item(
                        "Display Issues",
                        "If text appears garbled, ensure your terminal supports Unicode and has sufficient color support."
                    )
                    .add_item(
                        "File Access Issues",
                        "If files can't be read, check file permissions and ensure paths are correct relative to your project root."
                    )
                    .add_item(
                        "Debug Mode",
                        "Use /debug to enable debug mode for detailed logging and troubleshooting information."
                    )
            )
            .add_shortcut("F1", "Show help")
            .add_shortcut("Ctrl+Q", "Exit")
            .add_shortcut("Ctrl+N", "New session")
            .add_shortcut("Ctrl+S", "List sessions")
            .add_shortcut("Ctrl+T", "List themes")
            .add_shortcut("Ctrl+F", "Search help")
            .add_shortcut("Escape", "Close dialog")
    }
}

impl Default for HelpContent {
    fn default() -> Self {
        Self::default_ricecoder_help()
    }
}

/// Help system trait for managing help content
pub trait HelpSystem {
    /// Get a help topic by name
    fn get_topic(&self, topic: &str) -> Option<&HelpItem>;
    /// Search help topics
    fn search_topics(&self, query: &str) -> Vec<&HelpItem>;
}

impl HelpSystem for HelpContent {
    fn get_topic(&self, topic: &str) -> Option<&HelpItem> {
        for category in &self.categories {
            for item in &category.items {
                if item.title == topic {
                    return Some(item);
                }
            }
        }
        None
    }

    fn search_topics(&self, query: &str) -> Vec<&HelpItem> {
        self.search(query)
            .into_iter()
            .map(|(_, item)| item)
            .collect()
    }
}
