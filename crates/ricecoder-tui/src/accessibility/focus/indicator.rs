//! Focus indicator styles for keyboard navigation
//!
//! Provides visual indicators for focused elements to improve keyboard navigation
//! and accessibility. Supports multiple indicator styles (brackets, asterisks,
//! underlines, arrows) to accommodate different user preferences and visual needs.

use serde::{Deserialize, Serialize};

/// Focus indicator style for keyboard navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FocusIndicatorStyle {
    /// Use brackets: [focused element]
    Bracket,
    /// Use asterisks: *focused element*
    Asterisk,
    /// Use underline: _focused element_
    Underline,
    /// Use arrow: > focused element
    Arrow,
}

impl FocusIndicatorStyle {
    /// Get the prefix for this style
    pub fn prefix(&self) -> &'static str {
        match self {
            FocusIndicatorStyle::Bracket => "[",
            FocusIndicatorStyle::Asterisk => "*",
            FocusIndicatorStyle::Underline => "_",
            FocusIndicatorStyle::Arrow => "> ",
        }
    }

    /// Get the suffix for this style
    pub fn suffix(&self) -> &'static str {
        match self {
            FocusIndicatorStyle::Bracket => "]",
            FocusIndicatorStyle::Asterisk => "*",
            FocusIndicatorStyle::Underline => "_",
            FocusIndicatorStyle::Arrow => "",
        }
    }
}
