//! Border constants and styling for RiceCoder TUI
//!
//! This module provides reusable border character sets and styling configurations
//! for consistent border rendering across the application.
//!
//! # Examples
//!
//! ```ignore
//! use ricecoder_tui::tui::border::{EmptyBorder, SplitBorder};
//! use ratatui::widgets::{Block, Borders};
//!
//! // Create block with empty border
//! let block = Block::default()
//!     .borders(Borders::ALL)
//!     .border_set(EmptyBorder);
//!
//! // Create split view with vertical border
//! let block = Block::default()
//!     .borders(Borders::LEFT | Borders::RIGHT)
//!     .border_set(SplitBorder::vertical());
//! ```

use ratatui::symbols::border;

/// Empty border set with no visible characters except horizontal spacing
pub const EMPTY_BORDER: border::Set = border::Set {
    top_left: "",
    top_right: "",
    bottom_left: "",
    bottom_right: "",
    vertical_left: "",
    vertical_right: "",
    horizontal_top: " ",
    horizontal_bottom: " ",
};

/// Split border set for vertical dividers
#[derive(Debug, Clone, Copy)]
pub struct SplitBorder;

impl SplitBorder {
    /// Get vertical split border set with thick vertical lines
    pub const fn vertical() -> border::Set {
        border::Set {
            top_left: "",
            top_right: "",
            bottom_left: "",
            bottom_right: "",
            vertical_left: "┃",
            vertical_right: "┃",
            horizontal_top: "",
            horizontal_bottom: "",
        }
    }

    /// Get horizontal split border set with thick horizontal lines
    pub const fn horizontal() -> border::Set {
        border::Set {
            top_left: "",
            top_right: "",
            bottom_left: "",
            bottom_right: "",
            vertical_left: "",
            vertical_right: "",
            horizontal_top: "━",
            horizontal_bottom: "━",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_border() {
        assert_eq!(EMPTY_BORDER.top_left, "");
        assert_eq!(EMPTY_BORDER.horizontal_top, " ");
    }

    #[test]
    fn test_split_border_vertical() {
        let border = SplitBorder::vertical();
        assert_eq!(border.vertical_left, "┃");
        assert_eq!(border.vertical_right, "┃");
    }

    #[test]
    fn test_split_border_horizontal() {
        let border = SplitBorder::horizontal();
        assert_eq!(border.horizontal_top, "━");
        assert_eq!(border.horizontal_bottom, "━");
    }
}
