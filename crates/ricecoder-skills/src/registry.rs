//! Skill registry with filesystem scanning (Gaps G-17-01, G-17-05, G-17-06)
//!
//! Discovers SKILL.md files from config directories and maintains a registry.
//! Implements OpenCode discovery logic with duplicate name handling.

use crate::errors::SkillError;
use crate::models::{SkillInfo, SkillMetadata};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tracing::{debug, warn};
use walkdir::WalkDir;

/// Static global skill registry (OpenCode uses Instance.state caching)
static SKILL_REGISTRY: Lazy<Arc<RwLock<SkillRegistryState>>> =
    Lazy::new(|| Arc::new(RwLock::new(SkillRegistryState::new())));

/// Internal registry state
struct SkillRegistryState {
    skills: HashMap<String, SkillInfo>,
    initialized: bool,
}

impl SkillRegistryState {
    fn new() -> Self {
        Self {
            skills: HashMap::new(),
            initialized: false,
        }
    }
}

/// Skill registry interface (OpenCode Skill namespace)
pub struct SkillRegistry;

impl SkillRegistry {
    /// Get all skills (OpenCode Skill.all())
    pub async fn all() -> Result<Vec<SkillInfo>, SkillError> {
        Self::ensure_initialized().await?;
        
        let registry = SKILL_REGISTRY
            .read()
            .map_err(|_| SkillError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to acquire registry lock",
            )))?;
            
        Ok(registry.skills.values().cloned().collect())
    }

    /// Get a specific skill by name (OpenCode Skill.get(name))
    pub async fn get(name: &str) -> Result<Option<SkillInfo>, SkillError> {
        Self::ensure_initialized().await?;
        
        let registry = SKILL_REGISTRY
            .read()
            .map_err(|_| SkillError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to acquire registry lock",
            )))?;
            
        Ok(registry.skills.get(name).cloned())
    }

    /// Clear and reinitialize the registry
    pub async fn reload() -> Result<(), SkillError> {
        let mut registry = SKILL_REGISTRY
            .write()
            .map_err(|_| SkillError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to acquire registry lock",
            )))?;
            
        registry.skills.clear();
        registry.initialized = false;
        drop(registry);
        
        Self::ensure_initialized().await
    }

    /// Ensure registry is initialized (Gap G-17-01)
    async fn ensure_initialized() -> Result<(), SkillError> {
        {
            let registry = SKILL_REGISTRY.read().map_err(|_| {
                SkillError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to acquire registry lock",
                ))
            })?;
            
            if registry.initialized {
                return Ok(());
            }
        }

        // Acquire write lock to initialize
        let mut registry = SKILL_REGISTRY
            .write()
            .map_err(|_| SkillError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to acquire registry lock",
            )))?;

        if registry.initialized {
            return Ok(());
        }

        debug!("Initializing skill registry");
        
        // Get config directories (Gap G-17-05 - OpenCode compatibility)
        let directories = Self::get_config_directories()?;
        
        // Scan for SKILL.md files
        for dir in directories {
            Self::scan_directory(&mut registry, &dir)?;
        }

        registry.initialized = true;
        debug!("Skill registry initialized with {} skills", registry.skills.len());
        
        Ok(())
    }

    /// Get config directories (Gap G-17-05 - OpenCode Config.directories())
    fn get_config_directories() -> Result<Vec<PathBuf>, SkillError> {
        let mut directories = Vec::new();

        // 1. Global config directory (OpenCode: ~/.opencode → RiceCoder: ~/.ricecoder)
        if let Some(home) = dirs::home_dir() {
            directories.push(home.join(".ricecoder"));
        }

        // 2. Workspace-local .ricecoder directory (walk up from cwd)
        if let Ok(cwd) = std::env::current_dir() {
            let mut path = cwd.as_path();
            while let Some(parent) = path.parent() {
                let ricecoder_dir = parent.join(".ricecoder");
                if ricecoder_dir.exists() && ricecoder_dir.is_dir() {
                    directories.push(ricecoder_dir);
                }
                path = parent;
            }
        }

        Ok(directories)
    }

    /// Scan a directory for SKILL.md files (OpenCode glob: "skill/**/SKILL.md")
    fn scan_directory(
        registry: &mut SkillRegistryState,
        dir: &Path,
    ) -> Result<(), SkillError> {
        let skill_base = dir.join("skill");
        
        if !skill_base.exists() {
            return Ok(());
        }

        debug!("Scanning for skills in: {}", skill_base.display());

        for entry in WalkDir::new(&skill_base)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_type().is_file() && 
                e.file_name() == "SKILL.md"
            })
        {
            let path = entry.path();
            match Self::parse_skill_file(path) {
                Ok(skill_info) => {
                    // Check for duplicates (Gap G-17-06 - warn + last writer wins)
                    if let Some(existing) = registry.skills.get(&skill_info.name) {
                        warn!(
                            "Duplicate skill name '{}': existing '{}', new '{}'",
                            skill_info.name,
                            existing.location.display(),
                            skill_info.location.display()
                        );
                    }
                    
                    registry.skills.insert(skill_info.name.clone(), skill_info);
                }
                Err(e) => {
                    warn!("Failed to parse skill at {}: {}", path.display(), e);
                }
            }
        }

        Ok(())
    }

    /// Parse a SKILL.md file (frontmatter + content)
    fn parse_skill_file(path: &Path) -> Result<SkillInfo, SkillError> {
        let content = std::fs::read_to_string(path)?;
        
        // Parse YAML frontmatter using gray-matter
        let matter = gray_matter::Matter::<gray_matter::engine::YAML>::new();
        let parsed = matter.parse(&content);
        
        // Extract metadata from frontmatter
        let metadata: SkillMetadata = parsed.data
            .ok_or_else(|| SkillError::MissingField {
                field: "frontmatter".to_string(),
                path: path.display().to_string(),
            })
            .and_then(|data| {
                serde_json::from_value(data.into())
                    .map_err(|e| SkillError::YamlParse(e.to_string()))
            })?;

        // Validate metadata
        metadata.validate().map_err(|msg| SkillError::InvalidSkill {
            path: path.display().to_string(),
            message: msg,
        })?;

        Ok(SkillInfo {
            name: metadata.name,
            description: metadata.description,
            location: path.to_path_buf(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[tokio::test]
    async fn test_skill_registry_initialization() {
        // Create temp dir with skill
        let temp_dir = TempDir::new().unwrap();
        let skill_dir = temp_dir.path().join("skill").join("test-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        
        let skill_md = skill_dir.join("SKILL.md");
        fs::write(&skill_md, r#"---
name: test-skill
description: A test skill
---
# Test Skill Content
"#).unwrap();

        // Note: This test would need dependency injection to test properly
        // For now, it just verifies the API compiles
        assert!(SkillRegistry::all().await.is_ok());
    }
}
