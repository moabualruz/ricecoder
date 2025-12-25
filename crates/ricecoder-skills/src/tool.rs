//! Skill tool implementation (Gaps G-17-02, G-17-10, G-17-11, G-17-12)
//!
//! Provides the MCP tool interface for loading skills with permission enforcement.
//! Matches OpenCode SkillTool behavior.

use crate::errors::SkillError;
use crate::models::SkillInfo;
use crate::permissions::{SkillPermission, SkillPermissionAction, SkillPermissionChecker};
use crate::registry::SkillRegistry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Skill tool input parameters (OpenCode parameters schema)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillToolInput {
    /// Skill identifier from available_skills
    pub name: String,
}

/// Skill tool output (OpenCode ToolResult format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillToolOutput {
    /// Output title (Gap G-17-10)
    pub title: String,
    
    /// Formatted skill content (Gap G-17-10)
    pub output: String,
    
    /// Metadata about the loaded skill
    pub metadata: SkillToolMetadata,
}

/// Skill tool metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillToolMetadata {
    pub name: String,
    pub dir: String,
}

/// Skill tool provider (Gap G-17-02)
pub struct SkillToolProvider {
    permission_checker: SkillPermissionChecker,
}

impl Default for SkillToolProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SkillToolProvider {
    /// Create a new skill tool provider
    pub fn new() -> Self {
        Self {
            permission_checker: SkillPermissionChecker::new(),
        }
    }

    /// Get tool description with available skills (Gap G-17-11)
    /// Filters skills by agent permissions if provided
    pub async fn get_description(
        &self,
        agent_permissions: Option<&SkillPermission>,
    ) -> Result<String, SkillError> {
        let skills = SkillRegistry::all().await?;
        
        // Filter accessible skills (Gap G-17-11, G-17-12)
        let accessible_skills: Vec<SkillInfo> = if let Some(perms) = agent_permissions {
            skills
                .into_iter()
                .filter(|skill| {
                    let action = perms.check(&skill.name);
                    action != SkillPermissionAction::Deny
                })
                .collect()
        } else {
            skills
        };

        // Build description with XML-like skill list (Gap G-17-11 - OpenCode format)
        let mut desc = vec![
            "Load a skill to get detailed instructions for a specific task.".to_string(),
            "Skills provide specialized knowledge and step-by-step guidance.".to_string(),
            "Use this when a task matches an available skill's description.".to_string(),
            "<available_skills>".to_string(),
        ];

        for skill in accessible_skills {
            desc.push(format!("  <skill>"));
            desc.push(format!("    <name>{}</name>", skill.name));
            desc.push(format!("    <description>{}</description>", skill.description));
            desc.push(format!("  </skill>"));
        }

        desc.push("</available_skills>".to_string());

        Ok(desc.join("\n"))
    }

    /// Execute skill tool (Gap G-17-02, G-17-03, G-17-04, G-17-10)
    pub async fn execute(
        &self,
        input: SkillToolInput,
        agent_name: &str,
        agent_permissions: &SkillPermission,
    ) -> Result<SkillToolOutput, SkillError> {
        // Load skill from registry
        let skill = SkillRegistry::get(&input.name)
            .await?
            .ok_or_else(|| {
                let all_skills = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        SkillRegistry::all().await
                            .unwrap_or_default()
                            .iter()
                            .map(|s| s.name.clone())
                            .collect::<Vec<_>>()
                    })
                });
                SkillError::not_found(&input.name, &all_skills)
            })?;

        // Check permissions (Gap G-17-03, G-17-04)
        let action = self.permission_checker.check_with_cache(&input.name, agent_permissions);

        match action {
            SkillPermissionAction::Deny => {
                return Err(SkillError::permission_denied(&skill.name, agent_name));
            }
            SkillPermissionAction::Ask => {
                // TODO: Integrate with RiceCoder approval system
                // For now, log and continue (would normally prompt user)
                tracing::warn!(
                    "Skill '{}' requires approval for agent '{}' (auto-approving for now)",
                    skill.name,
                    agent_name
                );
                self.permission_checker.approve(&input.name);
            }
            SkillPermissionAction::Allow => {
                // Proceed without prompting
            }
        }

        // Load skill content
        let content = self.load_skill_content(&skill.location)?;
        let dir = skill.location
            .parent()
            .and_then(|p| p.to_str())
            .unwrap_or("")
            .to_string();

        // Format output (Gap G-17-10 - matches OpenCode format exactly)
        let output = format!(
            "## Skill: {}\n\n**Base directory**: {}\n\n{}",
            skill.name,
            dir,
            content.trim()
        );

        Ok(SkillToolOutput {
            title: format!("Loaded skill: {}", skill.name),
            output,
            metadata: SkillToolMetadata {
                name: skill.name,
                dir,
            },
        })
    }

    /// Load and parse skill markdown content
    fn load_skill_content(&self, path: &Path) -> Result<String, SkillError> {
        let content = std::fs::read_to_string(path)?;
        
        // Parse frontmatter to extract content only
        let matter = gray_matter::Matter::<gray_matter::engine::YAML>::new();
        let parsed = matter.parse(&content);
        
        Ok(parsed.content)
    }

    /// Get the permission checker (for session management)
    pub fn permission_checker(&self) -> &SkillPermissionChecker {
        &self.permission_checker
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_skill_tool_description() {
        let provider = SkillToolProvider::new();
        
        // With no permissions filter
        let desc = provider.get_description(None).await;
        assert!(desc.is_ok());
        
        let desc_text = desc.unwrap();
        assert!(desc_text.contains("<available_skills>"));
        assert!(desc_text.contains("</available_skills>"));
    }

    #[tokio::test]
    async fn test_skill_tool_description_with_permissions() {
        let provider = SkillToolProvider::new();
        
        let mut rules = HashMap::new();
        rules.insert("*".to_string(), SkillPermissionAction::Allow);
        rules.insert("dangerous-*".to_string(), SkillPermissionAction::Deny);
        let perms = SkillPermission::new(rules);
        
        let desc = provider.get_description(Some(&perms)).await;
        assert!(desc.is_ok());
    }

    #[test]
    fn test_load_skill_content() {
        use tempfile::NamedTempFile;
        use std::io::Write;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(
            temp_file,
            "---\nname: test\ndescription: Test skill\n---\n# Skill Content\nThis is the content."
        )
        .unwrap();

        let provider = SkillToolProvider::new();
        let content = provider.load_skill_content(temp_file.path());
        
        assert!(content.is_ok());
        let content_str = content.unwrap();
        assert!(content_str.contains("# Skill Content"));
        assert!(!content_str.contains("---"));
    }
}
