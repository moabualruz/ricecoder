//! Custom theme loading from YAML and JSON files
//!
//! Supports two theme formats:
//! 1. Simple YAML format (legacy)
//! 2. OpenCode-compatible JSON format with `defs` and `theme` sections

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};
use ratatui::style::{Color, Color as ColorSupport};
use serde::{Deserialize, Serialize};
use serde_json;

use crate::types::{AgentColors, DiffColors, SyntaxTheme, Theme};

// ============================================================================
// OpenCode-compatible JSON Theme Format
// ============================================================================

/// Color value that can be a hex color or a reference to a def
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ColorValue {
    /// Direct hex color like "#ff0000"
    Direct(String),
    /// Dark/light mode variants
    Variants { dark: String, light: String },
}

/// OpenCode-compatible JSON theme format
#[derive(Debug, Clone, Deserialize)]
pub struct ThemeJson {
    /// Color definitions (palette)
    #[serde(default)]
    pub defs: HashMap<String, String>,
    /// Theme color mappings
    pub theme: ThemeColors,
}

/// Theme color mappings (semantic colors)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeColors {
    pub primary: ColorValue,
    pub secondary: ColorValue,
    pub accent: ColorValue,
    pub error: ColorValue,
    pub warning: ColorValue,
    pub success: ColorValue,
    #[serde(default)]
    pub info: Option<ColorValue>,
    pub text: ColorValue,
    #[serde(default)]
    pub text_muted: Option<ColorValue>,
    pub background: ColorValue,
    #[serde(default)]
    pub background_panel: Option<ColorValue>,
    #[serde(default)]
    pub background_element: Option<ColorValue>,
    pub border: ColorValue,
    #[serde(default)]
    pub border_active: Option<ColorValue>,
    #[serde(default)]
    pub border_subtle: Option<ColorValue>,
    // Diff colors
    #[serde(default)]
    pub diff_added: Option<ColorValue>,
    #[serde(default)]
    pub diff_removed: Option<ColorValue>,
    #[serde(default)]
    pub diff_context: Option<ColorValue>,
    #[serde(default)]
    pub diff_hunk_header: Option<ColorValue>,
    #[serde(default)]
    pub diff_highlight_added: Option<ColorValue>,
    #[serde(default)]
    pub diff_highlight_removed: Option<ColorValue>,
    #[serde(default)]
    pub diff_added_bg: Option<ColorValue>,
    #[serde(default)]
    pub diff_removed_bg: Option<ColorValue>,
    // Syntax colors
    #[serde(default)]
    pub syntax_comment: Option<ColorValue>,
    #[serde(default)]
    pub syntax_keyword: Option<ColorValue>,
    #[serde(default)]
    pub syntax_function: Option<ColorValue>,
    #[serde(default)]
    pub syntax_variable: Option<ColorValue>,
    #[serde(default)]
    pub syntax_string: Option<ColorValue>,
    #[serde(default)]
    pub syntax_number: Option<ColorValue>,
    #[serde(default)]
    pub syntax_type: Option<ColorValue>,
    #[serde(default)]
    pub syntax_operator: Option<ColorValue>,
}

impl ThemeJson {
    /// Resolve a color value to a hex string, looking up defs if needed
    fn resolve_color(&self, value: &ColorValue, mode: &str) -> String {
        let raw = match value {
            ColorValue::Direct(s) => s.clone(),
            ColorValue::Variants { dark, light } => {
                if mode == "light" { light.clone() } else { dark.clone() }
            }
        };
        
        // If it's a hex color, return it directly
        if raw.starts_with('#') {
            return raw;
        }
        
        // Otherwise, look it up in defs
        self.defs.get(&raw).cloned().unwrap_or_else(|| {
            // Return a fallback color if not found
            tracing::warn!("Color def '{}' not found, using fallback", raw);
            "#808080".to_string()
        })
    }
    
    /// Resolve an optional color value with a fallback
    fn resolve_optional(&self, value: &Option<ColorValue>, fallback: &str, mode: &str) -> String {
        match value {
            Some(v) => self.resolve_color(v, mode),
            None => fallback.to_string(),
        }
    }
    
    /// Parse hex color string to ratatui Color
    fn parse_hex(hex: &str) -> Result<Color> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return Err(anyhow!("Invalid hex color length: {}", hex));
        }
        let rgb = u32::from_str_radix(hex, 16)
            .map_err(|_| anyhow!("Invalid hex color: {}", hex))?;
        let r = ((rgb >> 16) & 0xff) as u8;
        let g = ((rgb >> 8) & 0xff) as u8;
        let b = (rgb & 0xff) as u8;
        Ok(Color::Rgb(r, g, b))
    }
    
    /// Convert JSON theme to Theme struct
    pub fn to_theme(&self, name: &str, mode: &str) -> Result<Theme> {
        let bg_hex = self.resolve_color(&self.theme.background, mode);
        let bg = Self::parse_hex(&bg_hex)?;
        
        // Derive panel and element backgrounds if not specified
        let panel_hex = self.resolve_optional(&self.theme.background_panel, &bg_hex, mode);
        let element_hex = self.resolve_optional(&self.theme.background_element, &bg_hex, mode);
        
        // Parse all required colors
        let primary = Self::parse_hex(&self.resolve_color(&self.theme.primary, mode))?;
        let secondary = Self::parse_hex(&self.resolve_color(&self.theme.secondary, mode))?;
        let accent = Self::parse_hex(&self.resolve_color(&self.theme.accent, mode))?;
        let foreground = Self::parse_hex(&self.resolve_color(&self.theme.text, mode))?;
        let error = Self::parse_hex(&self.resolve_color(&self.theme.error, mode))?;
        let warning = Self::parse_hex(&self.resolve_color(&self.theme.warning, mode))?;
        let success = Self::parse_hex(&self.resolve_color(&self.theme.success, mode))?;
        let info = Self::parse_hex(&self.resolve_optional(&self.theme.info, 
            &self.resolve_color(&self.theme.accent, mode), mode))?;
        
        // Parse UI colors
        let text_muted = Self::parse_hex(&self.resolve_optional(&self.theme.text_muted,
            &self.resolve_color(&self.theme.secondary, mode), mode))?;
        let background_panel = Self::parse_hex(&panel_hex)?;
        let background_element = Self::parse_hex(&element_hex)?;
        let border = Self::parse_hex(&self.resolve_color(&self.theme.border, mode))?;
        let border_active = Self::parse_hex(&self.resolve_optional(&self.theme.border_active,
            &self.resolve_color(&self.theme.primary, mode), mode))?;
        let border_subtle = Self::parse_hex(&self.resolve_optional(&self.theme.border_subtle,
            &self.resolve_color(&self.theme.border, mode), mode))?;
        
        // Parse diff colors with defaults
        let diff = DiffColors {
            added: Self::parse_hex(&self.resolve_optional(&self.theme.diff_added_bg, "#1a3a1a", mode))?,
            removed: Self::parse_hex(&self.resolve_optional(&self.theme.diff_removed_bg, "#3a1a1a", mode))?,
            context: Color::Reset,
            hunk_header: Self::parse_hex(&self.resolve_optional(&self.theme.diff_hunk_header, "#6272a4", mode))?,
            highlight_added: Self::parse_hex(&self.resolve_optional(&self.theme.diff_highlight_added, "#50fa7b", mode))?,
            highlight_removed: Self::parse_hex(&self.resolve_optional(&self.theme.diff_highlight_removed, "#ff5555", mode))?,
            line_number_added: Self::parse_hex(&self.resolve_optional(&self.theme.diff_added, "#50fa7b", mode))?,
            line_number_removed: Self::parse_hex(&self.resolve_optional(&self.theme.diff_removed, "#ff5555", mode))?,
        };
        
        // Parse syntax colors with defaults
        let syntax = SyntaxTheme {
            keyword: Self::parse_hex(&self.resolve_optional(&self.theme.syntax_keyword, "#ff79c6", mode))?,
            string: Self::parse_hex(&self.resolve_optional(&self.theme.syntax_string, "#f1fa8c", mode))?,
            number: Self::parse_hex(&self.resolve_optional(&self.theme.syntax_number, "#bd93f9", mode))?,
            comment: Self::parse_hex(&self.resolve_optional(&self.theme.syntax_comment, "#6272a4", mode))?,
            function: Self::parse_hex(&self.resolve_optional(&self.theme.syntax_function, "#50fa7b", mode))?,
            variable: Self::parse_hex(&self.resolve_optional(&self.theme.syntax_variable, "#f8f8f2", mode))?,
            r#type: Self::parse_hex(&self.resolve_optional(&self.theme.syntax_type, "#8be9fd", mode))?,
            constant: Self::parse_hex(&self.resolve_optional(&self.theme.syntax_number, "#bd93f9", mode))?,
        };
        
        // Derive menu background (between bg and panel)
        let menu_bg = match bg {
            Color::Rgb(r, g, b) => {
                let is_dark = (r as u16 + g as u16 + b as u16) / 3 < 128;
                if is_dark {
                    Color::Rgb(r.saturating_add(15), g.saturating_add(15), b.saturating_add(15))
                } else {
                    Color::Rgb(r.saturating_sub(5), g.saturating_sub(5), b.saturating_sub(5))
                }
            }
            _ => Color::DarkGray,
        };
        
        Ok(Theme {
            name: name.to_string(),
            primary,
            secondary,
            accent,
            background: bg,
            foreground,
            error,
            warning,
            success,
            info,
            syntax,
            text_muted,
            selected_list_item_text: foreground,
            background_panel,
            background_element,
            background_menu: menu_bg,
            border,
            border_active,
            border_subtle,
            agent_colors: AgentColors::default(),
            diff,
        })
    }
}

/// YAML theme format for custom themes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeYaml {
    /// Theme name
    pub name: String,
    /// Primary color (hex format)
    pub primary: String,
    /// Secondary color (hex format)
    pub secondary: String,
    /// Accent color (hex format)
    pub accent: String,
    /// Background color (hex format)
    pub background: String,
    /// Foreground color (hex format)
    pub foreground: String,
    /// Error color (hex format)
    pub error: String,
    /// Warning color (hex format)
    pub warning: String,
    /// Success color (hex format)
    pub success: String,
}

impl ThemeYaml {
    /// Parse hex color string to ratatui Color
    fn parse_color(hex: &str) -> Result<Color> {
        if hex.starts_with('#') {
            let hex = &hex[1..];
            if hex.len() == 6 {
                if let Ok(rgb) = u32::from_str_radix(hex, 16) {
                    let r = ((rgb >> 16) & 0xff) as u8;
                    let g = ((rgb >> 8) & 0xff) as u8;
                    let b = (rgb & 0xff) as u8;
                    return Ok(Color::Rgb(r, g, b));
                }
            }
        }
        Err(anyhow!("Invalid hex color: {}", hex))
    }

    /// Convert color to hex string
    fn color_to_hex(color: &Color) -> String {
        match color {
            Color::Rgb(r, g, b) => format!("#{:02x}{:02x}{:02x}", r, g, b),
            _ => "#000000".to_string(), // fallback
        }
    }

    /// Convert YAML theme to Theme struct
    pub fn to_theme(&self) -> Result<Theme> {
        // Derive UI colors from the base colors
        let bg = Self::parse_color(&self.background)?;
        let (panel_bg, element_bg, border_color) = match bg {
            Color::Rgb(r, g, b) => {
                // If dark background, make panels slightly lighter
                // If light background, make panels slightly darker
                let is_dark = (r as u16 + g as u16 + b as u16) / 3 < 128;
                if is_dark {
                    (
                        Color::Rgb(r.saturating_add(20), g.saturating_add(20), b.saturating_add(20)),
                        Color::Rgb(r.saturating_add(35), g.saturating_add(35), b.saturating_add(35)),
                        Color::Rgb(r.saturating_add(50), g.saturating_add(50), b.saturating_add(50)),
                    )
                } else {
                    (
                        Color::Rgb(r.saturating_sub(10), g.saturating_sub(10), b.saturating_sub(10)),
                        Color::Rgb(r.saturating_sub(20), g.saturating_sub(20), b.saturating_sub(20)),
                        Color::Rgb(r.saturating_sub(55), g.saturating_sub(55), b.saturating_sub(55)),
                    )
                }
            }
            _ => (Color::DarkGray, Color::Gray, Color::Gray),
        };

        // Derive menu background (between bg and panel)
        let menu_bg = match bg {
            Color::Rgb(r, g, b) => {
                let is_dark = (r as u16 + g as u16 + b as u16) / 3 < 128;
                if is_dark {
                    Color::Rgb(r.saturating_add(15), g.saturating_add(15), b.saturating_add(15))
                } else {
                    Color::Rgb(r.saturating_sub(5), g.saturating_sub(5), b.saturating_sub(5))
                }
            }
            _ => Color::DarkGray,
        };

        // Derive subtle border (less prominent than main border)
        let border_subtle = match border_color {
            Color::Rgb(r, g, b) => {
                let is_dark = (r as u16 + g as u16 + b as u16) / 3 < 128;
                if is_dark {
                    Color::Rgb(r.saturating_sub(15), g.saturating_sub(15), b.saturating_sub(15))
                } else {
                    Color::Rgb(r.saturating_add(15), g.saturating_add(15), b.saturating_add(15))
                }
            }
            _ => Color::Gray,
        };

        Ok(Theme {
            name: self.name.clone(),
            primary: Self::parse_color(&self.primary)?,
            secondary: Self::parse_color(&self.secondary)?,
            accent: Self::parse_color(&self.accent)?,
            background: bg,
            foreground: Self::parse_color(&self.foreground)?,
            error: Self::parse_color(&self.error)?,
            warning: Self::parse_color(&self.warning)?,
            success: Self::parse_color(&self.success)?,
            info: Self::parse_color(&self.accent)?, // Derive info from accent
            syntax: SyntaxTheme {
                keyword: Self::parse_color("#ff6600")?,
                string: Self::parse_color("#00ff00")?,
                number: Self::parse_color("#ffff00")?,
                comment: Self::parse_color("#888888")?,
                function: Self::parse_color("#ff00ff")?,
                variable: Self::parse_color("#ffffff")?,
                r#type: Self::parse_color("#00ffff")?,
                constant: Self::parse_color("#ff6600")?,
            },
            // Derived UI colors
            text_muted: Self::parse_color(&self.secondary)?,
            selected_list_item_text: Self::parse_color(&self.foreground)?,
            background_panel: panel_bg,
            background_element: element_bg,
            background_menu: menu_bg,
            border: border_color,
            border_active: Self::parse_color(&self.accent)?,
            border_subtle,
            agent_colors: AgentColors::default(),
            diff: DiffColors::default(),
        })
    }
}

impl From<&Theme> for ThemeYaml {
    fn from(theme: &Theme) -> Self {
        Self {
            name: theme.name.clone(),
            primary: Self::color_to_hex(&theme.primary),
            secondary: Self::color_to_hex(&theme.secondary),
            accent: Self::color_to_hex(&theme.accent),
            background: Self::color_to_hex(&theme.background),
            foreground: Self::color_to_hex(&theme.foreground),
            error: Self::color_to_hex(&theme.error),
            warning: Self::color_to_hex(&theme.warning),
            success: Self::color_to_hex(&theme.success),
        }
    }
}

/// Custom theme loader
pub struct ThemeLoader;

impl ThemeLoader {
    // ========================================================================
    // JSON Theme Loading (OpenCode-compatible format)
    // ========================================================================
    
    /// Load a theme from a JSON string (OpenCode-compatible format)
    pub fn load_json_from_string(content: &str, name: &str, mode: &str) -> Result<Theme> {
        let theme_json: ThemeJson = serde_json::from_str(content)
            .map_err(|e| anyhow!("Failed to parse JSON theme: {}", e))?;
        theme_json.to_theme(name, mode)
    }
    
    /// Load a theme from a JSON file (OpenCode-compatible format)
    /// 
    /// The theme name is derived from the filename (without extension).
    /// Mode defaults to "dark" unless the filename contains "light".
    pub fn load_json_from_file(path: &Path) -> Result<Theme> {
        if !path.exists() {
            return Err(anyhow!("Theme file not found: {}", path.display()));
        }
        
        if !path.extension().is_some_and(|ext| ext == "json") {
            return Err(anyhow!("Theme file must be JSON format (.json)"));
        }
        
        let content = fs::read_to_string(path)?;
        
        // Derive theme name from filename
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("custom");
        
        // Detect mode from filename (e.g., "dracula-light.json" -> "light")
        let mode = if name.contains("light") { "light" } else { "dark" };
        
        Self::load_json_from_string(&content, name, mode)
    }
    
    // ========================================================================
    // YAML Theme Loading (legacy format)
    // ========================================================================
    
    /// Load a theme from a YAML string
    pub fn load_from_string(content: &str) -> Result<Theme> {
        let theme_yaml: ThemeYaml = serde_yaml::from_str(content)?;

        // Validate theme
        Self::validate_theme(&theme_yaml)?;

        theme_yaml.to_theme()
    }

    /// Load a theme from a YAML string and adapt it to terminal capabilities
    pub fn load_from_string_adapted(content: &str, _support: ColorSupport) -> Result<Theme> {
        // let mut theme = Self::load_from_string(content)?;
        // theme.adapt(support); // TODO: implement adapt method
        Self::load_from_string(content)
    }

    /// Load a theme from a YAML file
    pub fn load_yaml_from_file(path: &Path) -> Result<Theme> {
        if !path.exists() {
            return Err(anyhow!("Theme file not found: {}", path.display()));
        }

        if !path
            .extension()
            .is_some_and(|ext| ext == "yaml" || ext == "yml")
        {
            return Err(anyhow!("Theme file must be YAML format (.yaml or .yml)"));
        }

        let content = fs::read_to_string(path)?;
        Self::load_from_string(&content)
    }
    
    /// Load a theme from a file (auto-detects format from extension)
    pub fn load_from_file(path: &Path) -> Result<Theme> {
        if !path.exists() {
            return Err(anyhow!("Theme file not found: {}", path.display()));
        }
        
        match path.extension().and_then(|e| e.to_str()) {
            Some("json") => Self::load_json_from_file(path),
            Some("yaml") | Some("yml") => Self::load_yaml_from_file(path),
            Some(ext) => Err(anyhow!("Unsupported theme format: .{}", ext)),
            None => Err(anyhow!("Theme file has no extension: {}", path.display())),
        }
    }

    /// Load a theme from a YAML file and adapt it to terminal capabilities
    pub fn load_from_file_adapted(path: &Path, _support: ColorSupport) -> Result<Theme> {
        // let mut theme = Self::load_from_file(path)?;
        // theme.adapt(support); // TODO: implement adapt method
        Self::load_from_file(path)
    }

    /// Save a theme to a YAML file
    pub fn save_to_file(theme: &Theme, path: &Path) -> Result<()> {
        let theme_yaml = ThemeYaml::from(theme);
        let content = serde_yaml::to_string(&theme_yaml)?;
        fs::write(path, content)?;
        Ok(())
    }
    
    // ========================================================================
    // Directory Loading
    // ========================================================================

    /// Load all themes from a directory (supports both JSON and YAML)
    pub fn load_from_directory(dir: &Path) -> Result<Vec<Theme>> {
        if !dir.exists() {
            return Ok(Vec::new());
        }

        if !dir.is_dir() {
            return Err(anyhow!("Path is not a directory: {}", dir.display()));
        }

        let mut themes = Vec::new();

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }
            
            let ext = path.extension().and_then(|e| e.to_str());
            match ext {
                Some("json") => {
                    match Self::load_json_from_file(&path) {
                        Ok(theme) => themes.push(theme),
                        Err(e) => {
                            tracing::warn!("Failed to load JSON theme from {}: {}", path.display(), e);
                        }
                    }
                }
                Some("yaml") | Some("yml") => {
                    match Self::load_yaml_from_file(&path) {
                        Ok(theme) => themes.push(theme),
                        Err(e) => {
                            tracing::warn!("Failed to load YAML theme from {}: {}", path.display(), e);
                        }
                    }
                }
                _ => {} // Skip non-theme files
            }
        }

        Ok(themes)
    }
    
    /// Load themes from multiple directories, with later directories taking precedence
    pub fn load_from_directories(dirs: &[&Path]) -> Result<HashMap<String, Theme>> {
        let mut themes = HashMap::new();
        
        for dir in dirs {
            if let Ok(dir_themes) = Self::load_from_directory(dir) {
                for theme in dir_themes {
                    themes.insert(theme.name.clone(), theme);
                }
            }
        }
        
        Ok(themes)
    }

    /// Get the default themes directory (bundled with app)
    pub fn bundled_themes_directory() -> Option<PathBuf> {
        // In development, use config/themes relative to workspace
        // In production, this would be relative to the executable
        let candidates = [
            PathBuf::from("config/themes"),
            PathBuf::from("../config/themes"),
            PathBuf::from("../../config/themes"),
        ];
        
        candidates.into_iter().find(|p| p.exists())
    }
    
    /// Get the user themes directory (~/.config/ricecoder/themes or ~/.ricecoder/themes)
    pub fn user_themes_directory() -> Result<PathBuf> {
        let config_dir =
            dirs::config_dir().ok_or_else(|| anyhow!("Could not determine config directory"))?;
        Ok(config_dir.join("ricecoder").join("themes"))
    }
    
    /// Deprecated: Use user_themes_directory() instead
    #[deprecated(since = "0.1.0", note = "Use user_themes_directory() instead")]
    pub fn themes_directory() -> Result<PathBuf> {
        Self::user_themes_directory()
    }

    /// Validate a theme YAML
    fn validate_theme(theme: &ThemeYaml) -> Result<()> {
        if theme.name.is_empty() {
            return Err(anyhow!("Theme name cannot be empty"));
        }

        // Validate all colors are valid hex
        let colors = vec![
            ("primary", &theme.primary),
            ("secondary", &theme.secondary),
            ("accent", &theme.accent),
            ("background", &theme.background),
            ("foreground", &theme.foreground),
            ("error", &theme.error),
            ("warning", &theme.warning),
            ("success", &theme.success),
        ];

        for (name, color) in colors {
            if ThemeYaml::parse_color(color).is_err() {
                return Err(anyhow!("Invalid {} color: {}", name, color));
            }
        }

        Ok(())
    }
}
