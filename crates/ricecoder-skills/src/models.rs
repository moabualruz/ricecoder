//! Skill data models (Gap G-17-01, G-17-09)

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Skill information matching OpenCode Skill.Info schema
/// Skills are data (markdown + frontmatter), NOT executable code (Gap G-17-09)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillInfo {
    /// Skill identifier (from frontmatter)
    pub name: String,
    
    /// Human-readable description (from frontmatter)
    pub description: String,
    
    /// Absolute path to the SKILL.md file
    pub location: PathBuf,
}

/// Parsed skill metadata from frontmatter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    /// Skill name (required)
    pub name: String,
    
    /// Skill description (required)
    pub description: String,
    
    /// Optional version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    
    /// Optional author
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    
    /// Optional tags for categorization
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

impl SkillInfo {
    /// Create a new SkillInfo instance
    pub fn new(name: String, description: String, location: PathBuf) -> Self {
        Self {
            name,
            description,
            location,
        }
    }
}

impl SkillMetadata {
    /// Validate required fields
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Skill name cannot be empty".to_string());
        }
        if self.description.is_empty() {
            return Err("Skill description cannot be empty".to_string());
        }
        Ok(())
    }
}
