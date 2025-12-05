/// Ghost text functionality for inline completion suggestions
///
/// Ghost text displays the top completion suggestion in a lighter color,
/// allowing users to see what will be inserted before accepting it.
use crate::types::{CompletionItem, GhostText, Position, Range};

/// Trait for generating ghost text from completions
pub trait GhostTextGenerator: Send + Sync {
    /// Generate ghost text from a completion item
    fn generate_ghost_text(&self, completion: &CompletionItem, position: Position) -> GhostText;

    /// Generate ghost text for multi-line completions
    fn generate_multiline_ghost_text(
        &self,
        completion: &CompletionItem,
        position: Position,
    ) -> GhostText;
}

/// Basic ghost text generator implementation
pub struct BasicGhostTextGenerator;

impl BasicGhostTextGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for BasicGhostTextGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl GhostTextGenerator for BasicGhostTextGenerator {
    fn generate_ghost_text(&self, completion: &CompletionItem, position: Position) -> GhostText {
        // Ghost text starts at the current position and extends to the end of the inserted text
        let text = completion.insert_text.clone();
        let end_position = Position::new(position.line, position.character + text.len() as u32);

        GhostText::new(text, Range::new(position, end_position))
    }

    fn generate_multiline_ghost_text(
        &self,
        completion: &CompletionItem,
        position: Position,
    ) -> GhostText {
        let text = completion.insert_text.clone();
        let lines: Vec<&str> = text.lines().collect();

        let end_position = if lines.len() > 1 {
            // For multi-line text, end position is on the last line
            let last_line = lines[lines.len() - 1];
            Position::new(
                position.line + (lines.len() - 1) as u32,
                last_line.len() as u32,
            )
        } else {
            // Single line
            Position::new(position.line, position.character + text.len() as u32)
        };

        GhostText::new(text, Range::new(position, end_position))
    }
}

/// Ghost text styling information
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GhostTextStyle {
    /// Lighter color (typical ghost text)
    #[default]
    Faded,
    /// Italicized
    Italic,
    /// Dimmed
    Dimmed,
    /// Custom styling
    Custom,
}

/// Ghost text renderer trait for UI integration
pub trait GhostTextRenderer: Send + Sync {
    /// Render ghost text with specified styling
    fn render(&self, ghost_text: &GhostText, style: GhostTextStyle) -> String;

    /// Get the styled representation of ghost text
    fn get_styled_text(&self, ghost_text: &GhostText, style: GhostTextStyle) -> String;
}

/// Basic ghost text renderer
pub struct BasicGhostTextRenderer;

impl BasicGhostTextRenderer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for BasicGhostTextRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl GhostTextRenderer for BasicGhostTextRenderer {
    fn render(&self, ghost_text: &GhostText, style: GhostTextStyle) -> String {
        match style {
            GhostTextStyle::Faded => format!("(faded) {}", ghost_text.text),
            GhostTextStyle::Italic => format!("(italic) {}", ghost_text.text),
            GhostTextStyle::Dimmed => format!("(dimmed) {}", ghost_text.text),
            GhostTextStyle::Custom => ghost_text.text.clone(),
        }
    }

    fn get_styled_text(&self, ghost_text: &GhostText, _style: GhostTextStyle) -> String {
        ghost_text.text.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ghost_text_generation_single_line() {
        let generator = BasicGhostTextGenerator::new();
        let completion = CompletionItem::new(
            "println".to_string(),
            crate::types::CompletionItemKind::Function,
            "println!(\"Hello\")".to_string(),
        );
        let position = Position::new(0, 5);

        let ghost_text = generator.generate_ghost_text(&completion, position);

        assert_eq!(ghost_text.text, "println!(\"Hello\")");
        assert_eq!(ghost_text.range.start, position);
        assert_eq!(
            ghost_text.range.end.character,
            position.character + "println!(\"Hello\")".len() as u32
        );
    }

    #[test]
    fn test_ghost_text_generation_multiline() {
        let generator = BasicGhostTextGenerator::new();
        let completion = CompletionItem::new(
            "fn".to_string(),
            crate::types::CompletionItemKind::Keyword,
            "fn main() {\n    \n}".to_string(),
        );
        let position = Position::new(0, 0);

        let ghost_text = generator.generate_multiline_ghost_text(&completion, position);

        assert_eq!(ghost_text.text, "fn main() {\n    \n}");
        assert_eq!(ghost_text.range.start, position);
        // End position should be on line 2 (0-indexed)
        assert_eq!(ghost_text.range.end.line, 2);
    }

    #[test]
    fn test_ghost_text_renderer_faded() {
        let renderer = BasicGhostTextRenderer::new();
        let ghost_text = GhostText::new(
            "test".to_string(),
            Range::new(Position::new(0, 0), Position::new(0, 4)),
        );

        let rendered = renderer.render(&ghost_text, GhostTextStyle::Faded);
        assert!(rendered.contains("test"));
    }

    #[test]
    fn test_ghost_text_renderer_italic() {
        let renderer = BasicGhostTextRenderer::new();
        let ghost_text = GhostText::new(
            "test".to_string(),
            Range::new(Position::new(0, 0), Position::new(0, 4)),
        );

        let rendered = renderer.render(&ghost_text, GhostTextStyle::Italic);
        assert!(rendered.contains("test"));
    }
}
