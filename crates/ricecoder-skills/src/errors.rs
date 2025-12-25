//! Skill-specific error types (Gap G-17-07)

use thiserror::Error;

/// Skill-specific errors matching OpenCode error taxonomy
#[derive(Error, Debug)]
pub enum SkillError {
    /// Skill validation failed
    #[error("Skill validation failed at {path}: {message}")]
    InvalidSkill {
        path: String,
        message: String,
    },

    /// Skill name mismatch between frontmatter and expected
    #[error("Skill name mismatch at {path}: expected '{expected}', found '{actual}'")]
    NameMismatch {
        path: String,
        expected: String,
        actual: String,
    },

    /// Skill not found
    #[error("Skill '{name}' not found. Available skills: {available}")]
    NotFound {
        name: String,
        available: String,
    },

    /// Permission denied
    #[error("Access to skill '{skill}' is denied for agent '{agent}'")]
    PermissionDenied {
        skill: String,
        agent: String,
    },

    /// IO error during skill loading
    #[error("IO error loading skill: {0}")]
    Io(#[from] std::io::Error),

    /// YAML parsing error
    #[error("YAML parsing error in skill frontmatter: {0}")]
    YamlParse(String),

    /// Missing required field
    #[error("Missing required field '{field}' in skill at {path}")]
    MissingField {
        field: String,
        path: String,
    },
}

impl SkillError {
    /// Create a skill not found error with available skill names
    pub fn not_found(name: impl Into<String>, available_skills: &[String]) -> Self {
        Self::NotFound {
            name: name.into(),
            available: if available_skills.is_empty() {
                "none".to_string()
            } else {
                available_skills.join(", ")
            },
        }
    }

    /// Create a permission denied error
    pub fn permission_denied(skill: impl Into<String>, agent: impl Into<String>) -> Self {
        Self::PermissionDenied {
            skill: skill.into(),
            agent: agent.into(),
        }
    }
}
