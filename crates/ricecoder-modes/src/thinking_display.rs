use crate::{
    error::Result,
    models::{ModeAction, ModeResponse, ThinkingDepth},
};

/// Formats and displays thinking content for user consumption
#[derive(Debug, Clone)]
pub struct ThinkingDisplay;

impl ThinkingDisplay {
    /// Format thinking content for display
    pub fn format_thinking(content: &str, depth: ThinkingDepth) -> String {
        let header = match depth {
            ThinkingDepth::Light => "💭 Quick Thinking",
            ThinkingDepth::Medium => "🧠 Thinking",
            ThinkingDepth::Deep => "🔬 Deep Analysis",
        };

        format!(
            "{}\n{}\n{}\n{}",
            "─".repeat(50),
            header,
            "─".repeat(50),
            content
        )
    }

    /// Add thinking content to a response
    pub fn add_thinking_to_response(
        response: &mut ModeResponse,
        thinking_content: &str,
        depth: ThinkingDepth,
    ) -> Result<()> {
        // Add thinking as metadata
        response.metadata.think_more_used = true;
        response.metadata.thinking_content = Some(thinking_content.to_string());

        // Add thinking as an action for display
        let formatted = Self::format_thinking(thinking_content, depth);
        response.add_action(ModeAction::DisplayThinking { content: formatted });

        Ok(())
    }

    /// Format thinking content with line numbers for readability
    pub fn format_thinking_with_line_numbers(content: &str, depth: ThinkingDepth) -> String {
        let header = match depth {
            ThinkingDepth::Light => "💭 Quick Thinking",
            ThinkingDepth::Medium => "🧠 Thinking",
            ThinkingDepth::Deep => "🔬 Deep Analysis",
        };

        let lines: Vec<&str> = content.lines().collect();
        let max_line_num = lines.len().to_string().len();

        let formatted_lines: Vec<String> = lines
            .iter()
            .enumerate()
            .map(|(i, line)| format!("{:width$} | {}", i + 1, line, width = max_line_num))
            .collect();

        format!(
            "{}\n{}\n{}\n{}",
            "─".repeat(50),
            header,
            "─".repeat(50),
            formatted_lines.join("\n")
        )
    }

    /// Format thinking content as a collapsible section
    pub fn format_thinking_collapsible(content: &str, depth: ThinkingDepth) -> String {
        let header = match depth {
            ThinkingDepth::Light => "💭 Quick Thinking",
            ThinkingDepth::Medium => "🧠 Thinking",
            ThinkingDepth::Deep => "🔬 Deep Analysis",
        };

        format!("▼ {}\n{}\n{}", header, "─".repeat(50), content)
    }

    /// Extract key insights from thinking content
    pub fn extract_insights(content: &str) -> Vec<String> {
        content
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                // Extract lines that look like conclusions or key points
                trimmed.starts_with("→")
                    || trimmed.starts_with("•")
                    || trimmed.starts_with("✓")
                    || trimmed.starts_with("Key:")
                    || trimmed.starts_with("Conclusion:")
            })
            .map(|line| line.to_string())
            .collect()
    }

    /// Summarize thinking content
    pub fn summarize_thinking(content: &str, max_lines: usize) -> String {
        let lines: Vec<&str> = content.lines().collect();
        if lines.len() <= max_lines {
            return content.to_string();
        }

        let mut summary = lines[..max_lines].join("\n");
        summary.push_str(&format!("\n... ({} more lines)", lines.len() - max_lines));
        summary
    }

    /// Format thinking content with emphasis on important sections
    pub fn format_thinking_with_emphasis(content: &str, depth: ThinkingDepth) -> String {
        let header = match depth {
            ThinkingDepth::Light => "💭 Quick Thinking",
            ThinkingDepth::Medium => "🧠 Thinking",
            ThinkingDepth::Deep => "🔬 Deep Analysis",
        };

        let emphasized_lines: Vec<String> = content
            .lines()
            .map(|line| {
                let trimmed = line.trim();
                if trimmed.starts_with("→") || trimmed.starts_with("✓") {
                    format!("  ⭐ {}", trimmed)
                } else if trimmed.starts_with("Key:") || trimmed.starts_with("Conclusion:") {
                    format!("  🔑 {}", trimmed)
                } else {
                    format!("     {}", trimmed)
                }
            })
            .collect();

        format!(
            "{}\n{}\n{}\n{}",
            "─".repeat(50),
            header,
            "─".repeat(50),
            emphasized_lines.join("\n")
        )
    }

    /// Check if thinking content is empty or minimal
    pub fn is_empty_or_minimal(content: &str) -> bool {
        content.trim().is_empty() || content.lines().count() < 2
    }

    /// Get thinking statistics
    pub fn get_statistics(content: &str) -> ThinkingStatistics {
        let lines = content.lines().count();
        let words = content.split_whitespace().count();
        let chars = content.len();

        ThinkingStatistics {
            lines,
            words,
            chars,
        }
    }
}

/// Statistics about thinking content
#[derive(Debug, Clone)]
pub struct ThinkingStatistics {
    /// Number of lines
    pub lines: usize,
    /// Number of words
    pub words: usize,
    /// Number of characters
    pub chars: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_thinking_light() {
        let content = "This is a quick thought";
        let formatted = ThinkingDisplay::format_thinking(content, ThinkingDepth::Light);
        assert!(formatted.contains("💭 Quick Thinking"));
        assert!(formatted.contains(content));
    }

    #[test]
    fn test_format_thinking_medium() {
        let content = "This is a medium thought";
        let formatted = ThinkingDisplay::format_thinking(content, ThinkingDepth::Medium);
        assert!(formatted.contains("🧠 Thinking"));
        assert!(formatted.contains(content));
    }

    #[test]
    fn test_format_thinking_deep() {
        let content = "This is a deep thought";
        let formatted = ThinkingDisplay::format_thinking(content, ThinkingDepth::Deep);
        assert!(formatted.contains("🔬 Deep Analysis"));
        assert!(formatted.contains(content));
    }

    #[test]
    fn test_format_thinking_with_line_numbers() {
        let content = "Line 1\nLine 2\nLine 3";
        let formatted =
            ThinkingDisplay::format_thinking_with_line_numbers(content, ThinkingDepth::Medium);
        assert!(formatted.contains("1 |"));
        assert!(formatted.contains("2 |"));
        assert!(formatted.contains("3 |"));
    }

    #[test]
    fn test_format_thinking_collapsible() {
        let content = "This is collapsible";
        let formatted =
            ThinkingDisplay::format_thinking_collapsible(content, ThinkingDepth::Medium);
        assert!(formatted.contains("▼"));
        assert!(formatted.contains("🧠 Thinking"));
    }

    #[test]
    fn test_extract_insights() {
        let content = "Some text\n→ Key insight 1\nMore text\n• Key insight 2\n✓ Conclusion";
        let insights = ThinkingDisplay::extract_insights(content);
        assert_eq!(insights.len(), 3);
        assert!(insights[0].contains("→"));
    }

    #[test]
    fn test_summarize_thinking() {
        let content = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5";
        let summary = ThinkingDisplay::summarize_thinking(content, 3);
        assert!(summary.contains("Line 1"));
        assert!(summary.contains("Line 3"));
        assert!(summary.contains("2 more lines"));
    }

    #[test]
    fn test_format_thinking_with_emphasis() {
        let content = "Normal line\n→ Important point\nKey: This is important";
        let formatted =
            ThinkingDisplay::format_thinking_with_emphasis(content, ThinkingDepth::Medium);
        assert!(formatted.contains("⭐"));
        assert!(formatted.contains("🔑"));
    }

    #[test]
    fn test_is_empty_or_minimal() {
        assert!(ThinkingDisplay::is_empty_or_minimal(""));
        assert!(ThinkingDisplay::is_empty_or_minimal("   "));
        assert!(!ThinkingDisplay::is_empty_or_minimal("Line 1\nLine 2"));
    }

    #[test]
    fn test_get_statistics() {
        let content = "Line 1\nLine 2\nLine 3";
        let stats = ThinkingDisplay::get_statistics(content);
        assert_eq!(stats.lines, 3);
        assert!(stats.words > 0);
        assert!(stats.chars > 0);
    }

    #[test]
    fn test_add_thinking_to_response() {
        let mut response = ModeResponse::new("Test response".to_string(), "test-mode".to_string());
        ThinkingDisplay::add_thinking_to_response(
            &mut response,
            "Test thinking",
            ThinkingDepth::Medium,
        )
        .unwrap();

        assert!(response.metadata.think_more_used);
        assert!(response.metadata.thinking_content.is_some());
        assert_eq!(response.actions.len(), 1);
    }
}
