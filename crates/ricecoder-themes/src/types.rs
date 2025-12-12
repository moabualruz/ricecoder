//! Core theme types and data structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A complete theme definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Theme {
    /// Theme metadata
    pub metadata: ThemeMetadata,
    /// Color definitions
    pub colors: ThemeColors,
    /// Syntax highlighting colors
    pub syntax: SyntaxColors,
}

/// Theme metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemeMetadata {
    /// Theme name
    pub name: String,
    /// Theme author
    pub author: String,
    /// Theme description
    pub description: String,
    /// Theme version
    pub version: String,
    /// Compatible RiceCoder versions
    pub ricecoder_version: String,
}

/// Color palette for the theme
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemeColors {
    /// Primary colors
    pub primary: ColorDefinition,
    /// Secondary colors
    pub secondary: ColorDefinition,
    /// Background colors
    pub background: ColorDefinition,
    /// Text colors
    pub text: ColorDefinition,
    /// Accent colors
    pub accent: ColorDefinition,
    /// Error colors
    pub error: ColorDefinition,
    /// Warning colors
    pub warning: ColorDefinition,
    /// Success colors
    pub success: ColorDefinition,
    /// Info colors
    pub info: ColorDefinition,
    /// Muted colors
    pub muted: ColorDefinition,
    /// Border colors
    pub border: ColorDefinition,
}

/// Syntax highlighting colors
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SyntaxColors {
    /// Keywords
    pub keyword: ColorDefinition,
    /// Strings
    pub string: ColorDefinition,
    /// Numbers
    pub number: ColorDefinition,
    /// Comments
    pub comment: ColorDefinition,
    /// Functions
    pub function: ColorDefinition,
    /// Variables
    pub variable: ColorDefinition,
    /// Types
    pub r#type: ColorDefinition,
    /// Constants
    pub constant: ColorDefinition,
}

/// Color definition with foreground, background, and modifiers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ColorDefinition {
    /// Foreground color (hex or named)
    pub fg: Option<String>,
    /// Background color (hex or named)
    pub bg: Option<String>,
    /// Text modifiers
    pub modifiers: Vec<String>,
}

/// Theme configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemeConfig {
    /// Current theme name
    pub current_theme: String,
    /// Custom theme overrides
    pub overrides: HashMap<String, serde_json::Value>,
    /// Theme settings
    pub settings: ThemeSettings,
}

/// Theme settings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemeSettings {
    /// Enable high contrast mode
    pub high_contrast: bool,
    /// Enable accessibility improvements
    pub accessibility: bool,
    /// Animation settings
    pub animations: bool,
}