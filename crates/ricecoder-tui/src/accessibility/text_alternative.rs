//! Text alternatives for visual elements
//!
//! Provides semantic descriptions of UI elements for screen readers and
//! assistive technologies. Supports short and long descriptions along with
//! element type information for proper semantic structure.

/// Text alternative for visual elements
#[derive(Debug, Clone, PartialEq)]
pub struct TextAlternative {
    /// Unique identifier for the element
    pub id: String,
    /// Short description (for screen readers)
    pub short_description: String,
    /// Long description (for detailed context)
    pub long_description: Option<String>,
    /// Element type (button, input, list, etc.)
    pub element_type: ElementType,
}

impl TextAlternative {
    /// Create a new text alternative
    pub fn new(
        id: impl Into<String>,
        short_desc: impl Into<String>,
        element_type: ElementType,
    ) -> Self {
        Self {
            id: id.into(),
            short_description: short_desc.into(),
            long_description: None,
            element_type,
        }
    }

    /// Add a long description
    pub fn with_long_description(mut self, desc: impl Into<String>) -> Self {
        self.long_description = Some(desc.into());
        self
    }

    /// Get the full description for screen readers
    pub fn full_description(&self) -> String {
        match &self.long_description {
            Some(long) => format!("{}: {}", self.short_description, long),
            None => self.short_description.clone(),
        }
    }
}

/// Element type for semantic structure
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementType {
    /// Button element
    Button,
    /// Input field
    Input,
    /// List or menu
    List,
    /// List item
    ListItem,
    /// Tab
    Tab,
    /// Tab panel
    TabPanel,
    /// Dialog
    Dialog,
    /// Text content
    Text,
    /// Heading
    Heading,
    /// Code block
    CodeBlock,
    /// Message (chat)
    Message,
    /// Status indicator
    Status,
}

impl ElementType {
    /// Get the semantic role name
    pub fn role(&self) -> &'static str {
        match self {
            ElementType::Button => "button",
            ElementType::Input => "textbox",
            ElementType::List => "list",
            ElementType::ListItem => "listitem",
            ElementType::Tab => "tab",
            ElementType::TabPanel => "tabpanel",
            ElementType::Dialog => "dialog",
            ElementType::Text => "text",
            ElementType::Heading => "heading",
            ElementType::CodeBlock => "code",
            ElementType::Message => "article",
            ElementType::Status => "status",
        }
    }
}
