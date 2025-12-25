//! Prompt parts - domain entities for prompt content
//!
//! Implements the part system from OpenCode's prompt, supporting:
//! - Text parts (pasted content with summarization)
//! - File parts (images, documents)
//! - Agent parts (@mentions)
//!
//! # DDD Layer: Domain
//! These are core domain entities representing the semantic content of a prompt.

use serde::{Deserialize, Serialize};

/// Source location for text within the prompt input
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextSource {
    /// Start offset in the input string
    pub start: usize,
    /// End offset in the input string
    pub end: usize,
    /// Virtual text displayed (may differ from actual content)
    pub value: String,
}

impl TextSource {
    /// Create a new text source
    pub fn new(start: usize, end: usize, value: impl Into<String>) -> Self {
        Self {
            start,
            end,
            value: value.into(),
        }
    }

    /// Get the length of the virtual text
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Update positions after text insertion
    pub fn shift(&mut self, offset: isize) {
        if offset >= 0 {
            self.start = self.start.saturating_add(offset as usize);
            self.end = self.end.saturating_add(offset as usize);
        } else {
            let abs_offset = (-offset) as usize;
            self.start = self.start.saturating_sub(abs_offset);
            self.end = self.end.saturating_sub(abs_offset);
        }
    }
}

/// Source information for file parts
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileSource {
    /// Source type (always "file")
    #[serde(rename = "type")]
    pub source_type: String,
    /// File path
    pub path: String,
    /// Text location in prompt
    pub text: Option<TextSource>,
}

impl FileSource {
    /// Create a new file source
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            source_type: "file".to_string(),
            path: path.into(),
            text: None,
        }
    }

    /// With text source
    pub fn with_text(mut self, text: TextSource) -> Self {
        self.text = Some(text);
        self
    }
}

/// Source information for agent mentions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentSource {
    /// Start offset
    pub start: usize,
    /// End offset
    pub end: usize,
    /// Display value (@agent_name)
    pub value: String,
}

impl AgentSource {
    /// Create a new agent source
    pub fn new(start: usize, end: usize, value: impl Into<String>) -> Self {
        Self {
            start,
            end,
            value: value.into(),
        }
    }
}

/// A file part in the prompt (image, document, etc.)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilePart {
    /// MIME type
    pub mime: String,
    /// Optional filename
    pub filename: Option<String>,
    /// Data URL (base64 encoded)
    pub url: String,
    /// Source information
    pub source: Option<FileSource>,
}

impl FilePart {
    /// Create a new file part from base64 data
    pub fn from_base64(mime: impl Into<String>, data: impl Into<String>) -> Self {
        let mime = mime.into();
        let data = data.into();
        Self {
            url: format!("data:{};base64,{}", mime, data),
            mime,
            filename: None,
            source: None,
        }
    }

    /// Set filename
    pub fn with_filename(mut self, filename: impl Into<String>) -> Self {
        self.filename = Some(filename.into());
        self
    }

    /// Set source
    pub fn with_source(mut self, source: FileSource) -> Self {
        self.source = Some(source);
        self
    }

    /// Check if this is an image
    pub fn is_image(&self) -> bool {
        self.mime.starts_with("image/")
    }

    /// Get display text for this part
    pub fn display_text(&self, index: usize) -> String {
        if self.is_image() {
            format!("[Image {}]", index + 1)
        } else {
            format!("[File: {}]", self.filename.as_deref().unwrap_or("unknown"))
        }
    }
}

/// An agent mention part (@agent_name)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentPart {
    /// Agent name
    pub name: String,
    /// Source information
    pub source: Option<AgentSource>,
}

impl AgentPart {
    /// Create a new agent part
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            source: None,
        }
    }

    /// With source
    pub fn with_source(mut self, source: AgentSource) -> Self {
        self.source = Some(source);
        self
    }

    /// Get display text
    pub fn display_text(&self) -> String {
        format!("@{}", self.name)
    }
}

/// A text part (pasted content, possibly summarized)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextPart {
    /// Full text content
    pub text: String,
    /// Source information (for virtual text display)
    pub source: Option<TextSource>,
}

impl TextPart {
    /// Create a new text part
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            source: None,
        }
    }

    /// With source (virtual text)
    pub fn with_source(mut self, source: TextSource) -> Self {
        self.source = Some(source);
        self
    }

    /// Create summarized text part
    pub fn summarized(text: impl Into<String>, line_count: usize, start: usize) -> Self {
        let text = text.into();
        let virtual_text = format!("[Pasted ~{} lines]", line_count);
        let end = start + virtual_text.len();
        Self {
            text,
            source: Some(TextSource::new(start, end, virtual_text)),
        }
    }
}

/// A prompt part - union of all part types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum PromptPart {
    /// Plain text part
    Text(TextPart),
    /// File attachment
    File(FilePart),
    /// Agent mention
    Agent(AgentPart),
}

impl PromptPart {
    /// Get the part type as a string
    pub fn part_type(&self) -> &'static str {
        match self {
            PromptPart::Text(_) => "text",
            PromptPart::File(_) => "file",
            PromptPart::Agent(_) => "agent",
        }
    }

    /// Check if this is a text part
    pub fn is_text(&self) -> bool {
        matches!(self, PromptPart::Text(_))
    }

    /// Check if this is a file part
    pub fn is_file(&self) -> bool {
        matches!(self, PromptPart::File(_))
    }

    /// Check if this is an agent part
    pub fn is_agent(&self) -> bool {
        matches!(self, PromptPart::Agent(_))
    }

    /// Get text source if available
    pub fn text_source(&self) -> Option<&TextSource> {
        match self {
            PromptPart::Text(p) => p.source.as_ref(),
            PromptPart::File(p) => p.source.as_ref().and_then(|s| s.text.as_ref()),
            PromptPart::Agent(p) => p.source.as_ref().map(|s| {
                // Convert AgentSource to TextSource reference - this is a bit awkward
                // In a real impl we'd use a trait
                unsafe { &*(s as *const AgentSource as *const TextSource) }
            }),
        }
    }

    /// Update source positions
    pub fn shift_source(&mut self, offset: isize) {
        match self {
            PromptPart::Text(p) => {
                if let Some(ref mut src) = p.source {
                    src.shift(offset);
                }
            }
            PromptPart::File(p) => {
                if let Some(ref mut src) = p.source {
                    if let Some(ref mut text) = src.text {
                        text.shift(offset);
                    }
                }
            }
            PromptPart::Agent(p) => {
                if let Some(ref mut src) = p.source {
                    if offset >= 0 {
                        src.start = src.start.saturating_add(offset as usize);
                        src.end = src.end.saturating_add(offset as usize);
                    } else {
                        let abs = (-offset) as usize;
                        src.start = src.start.saturating_sub(abs);
                        src.end = src.end.saturating_sub(abs);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_source_shift() {
        let mut src = TextSource::new(10, 20, "test");
        src.shift(5);
        assert_eq!(src.start, 15);
        assert_eq!(src.end, 25);

        src.shift(-10);
        assert_eq!(src.start, 5);
        assert_eq!(src.end, 15);
    }

    #[test]
    fn test_file_part_is_image() {
        let img = FilePart::from_base64("image/png", "abc123");
        assert!(img.is_image());

        let doc = FilePart::from_base64("application/pdf", "abc123");
        assert!(!doc.is_image());
    }

    #[test]
    fn test_text_part_summarized() {
        let part = TextPart::summarized("line1\nline2\nline3", 3, 0);
        assert!(part.source.is_some());
        let src = part.source.unwrap();
        assert_eq!(src.value, "[Pasted ~3 lines]");
    }

    #[test]
    fn test_prompt_part_types() {
        let text = PromptPart::Text(TextPart::new("hello"));
        assert!(text.is_text());
        assert_eq!(text.part_type(), "text");

        let file = PromptPart::File(FilePart::from_base64("image/png", "data"));
        assert!(file.is_file());

        let agent = PromptPart::Agent(AgentPart::new("build"));
        assert!(agent.is_agent());
    }
}
