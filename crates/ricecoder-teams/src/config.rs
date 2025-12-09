/// Team configuration management
use crate::error::{Result, TeamError};
use crate::models::{MergedStandards, StandardsOverride, TeamStandards};
use chrono::Utc;
use ricecoder_storage::PathResolver;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Change history entry for tracking modifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeHistoryEntry {
    pub timestamp: chrono::DateTime<Utc>,
    pub version: u32,
    pub description: String,
    pub changed_by: String,
}

/// Manages team-level configuration storage and inheritance
pub struct TeamConfigManager {
    /// Cache for standards to improve performance
    standards_cache: Arc<RwLock<HashMap<String, TeamStandards>>>,
    /// Change history for tracking modifications
    change_history: Arc<RwLock<HashMap<String, Vec<ChangeHistoryEntry>>>>,
}

impl TeamConfigManager {
    /// Create a new TeamConfigManager
    pub fn new() -> Self {
        TeamConfigManager {
            standards_cache: Arc::new(RwLock::new(HashMap::new())),
            change_history: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Store standards for a team using ricecoder-storage in YAML format
    pub async fn store_standards(&self, team_id: &str, standards: TeamStandards) -> Result<()> {
        // Resolve the storage path for team standards
        let storage_path = Self::resolve_team_standards_path(team_id)?;

        // Ensure parent directory exists
        if let Some(parent) = storage_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                TeamError::StorageError(format!("Failed to create storage directory: {}", e))
            })?;
        }

        // Serialize standards to YAML
        let yaml_content = serde_yaml::to_string(&standards).map_err(TeamError::YamlError)?;

        // Write to file
        std::fs::write(&storage_path, yaml_content).map_err(|e| {
            TeamError::StorageError(format!("Failed to write standards file: {}", e))
        })?;

        // Update cache
        let mut cache = self.standards_cache.write().await;
        cache.insert(team_id.to_string(), standards);

        tracing::info!(
            team_id = %team_id,
            path = ?storage_path,
            "Team standards stored successfully"
        );

        Ok(())
    }

    /// Retrieve standards for a team using PathResolver with caching
    pub async fn get_standards(&self, team_id: &str) -> Result<TeamStandards> {
        // Check cache first
        {
            let cache = self.standards_cache.read().await;
            if let Some(standards) = cache.get(team_id) {
                tracing::debug!(team_id = %team_id, "Retrieved standards from cache");
                return Ok(standards.clone());
            }
        }

        // Load from storage
        let storage_path = Self::resolve_team_standards_path(team_id)?;

        if !storage_path.exists() {
            return Err(TeamError::TeamNotFound(format!(
                "Standards not found for team: {}",
                team_id
            )));
        }

        let yaml_content = std::fs::read_to_string(&storage_path).map_err(|e| {
            TeamError::StorageError(format!("Failed to read standards file: {}", e))
        })?;

        let standards: TeamStandards =
            serde_yaml::from_str(&yaml_content).map_err(TeamError::YamlError)?;

        // Update cache
        let mut cache = self.standards_cache.write().await;
        cache.insert(team_id.to_string(), standards.clone());

        tracing::info!(team_id = %team_id, "Retrieved team standards from storage");

        Ok(standards)
    }

    /// Apply hierarchy: Organization → Team → Project using ConfigMerger
    pub async fn apply_hierarchy(
        &self,
        org_id: &str,
        team_id: &str,
        project_id: &str,
    ) -> Result<MergedStandards> {
        // Load standards from each level
        let org_standards = self.get_standards(org_id).await.ok();
        let team_standards = self.get_standards(team_id).await.ok();
        let project_standards = self.get_standards(project_id).await.ok();

        // Merge standards with hierarchy: Organization → Team → Project
        // Project-level standards override team-level, which override organization-level
        let final_standards = Self::merge_standards_hierarchy(
            org_standards.clone(),
            team_standards.clone(),
            project_standards.clone(),
        )?;

        tracing::info!(
            org_id = %org_id,
            team_id = %team_id,
            project_id = %project_id,
            "Standards hierarchy applied successfully"
        );

        Ok(MergedStandards {
            organization_standards: org_standards,
            team_standards,
            project_standards,
            final_standards,
        })
    }

    /// Override standards at project level with validation
    pub async fn override_standards(
        &self,
        project_id: &str,
        overrides: StandardsOverride,
    ) -> Result<()> {
        // Load current project standards
        let mut project_standards = self.get_standards(project_id).await?;

        // Validate overrides
        Self::validate_overrides(&project_standards, &overrides)?;

        // Apply overrides
        for override_id in &overrides.overridden_standards {
            // Remove overridden rules
            project_standards
                .code_review_rules
                .retain(|r| &r.id != override_id);
        }

        // Update version
        project_standards.version += 1;
        project_standards.updated_at = Utc::now();

        // Store updated standards
        self.store_standards(project_id, project_standards).await?;

        // Track the change
        self.track_changes(
            project_id,
            &format!("Applied {} overrides", overrides.overridden_standards.len()),
        )
        .await?;

        tracing::info!(
            project_id = %project_id,
            override_count = %overrides.overridden_standards.len(),
            "Standards overrides applied successfully"
        );

        Ok(())
    }

    /// Track changes to standards with timestamps and version identifiers
    pub async fn track_changes(&self, team_id: &str, change_description: &str) -> Result<()> {
        let entry = ChangeHistoryEntry {
            timestamp: Utc::now(),
            version: 1, // TODO: Get actual version from standards
            description: change_description.to_string(),
            changed_by: "system".to_string(), // TODO: Get actual user
        };

        let mut history = self.change_history.write().await;
        history
            .entry(team_id.to_string())
            .or_insert_with(Vec::new)
            .push(entry);

        tracing::info!(
            team_id = %team_id,
            change = %change_description,
            "Standards change tracked successfully"
        );

        Ok(())
    }

    /// Get change history for a team
    pub async fn get_change_history(&self, team_id: &str) -> Result<Vec<ChangeHistoryEntry>> {
        let history = self.change_history.read().await;
        Ok(history.get(team_id).cloned().unwrap_or_default())
    }

    // Helper functions

    /// Resolve the storage path for team standards
    fn resolve_team_standards_path(team_id: &str) -> Result<PathBuf> {
        let global_path = PathResolver::resolve_global_path()
            .map_err(|e| TeamError::StorageError(e.to_string()))?;

        let standards_path = global_path
            .join("teams")
            .join(team_id)
            .join("standards.yaml");

        Ok(standards_path)
    }

    /// Merge standards from hierarchy with project overrides taking precedence
    pub fn merge_standards_hierarchy(
        org_standards: Option<TeamStandards>,
        team_standards: Option<TeamStandards>,
        project_standards: Option<TeamStandards>,
    ) -> Result<TeamStandards> {
        // Start with organization standards as base
        let mut merged = org_standards.unwrap_or_else(|| TeamStandards {
            id: "merged".to_string(),
            team_id: "merged".to_string(),
            code_review_rules: Vec::new(),
            templates: Vec::new(),
            steering_docs: Vec::new(),
            compliance_requirements: Vec::new(),
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });

        // Merge team standards (override organization)
        if let Some(team) = team_standards {
            merged.code_review_rules.extend(team.code_review_rules);
            merged.templates.extend(team.templates);
            merged.steering_docs.extend(team.steering_docs);
            merged
                .compliance_requirements
                .extend(team.compliance_requirements);
            merged.version = team.version;
            merged.updated_at = team.updated_at;
        }

        // Merge project standards (override team and organization)
        if let Some(project) = project_standards {
            merged.code_review_rules.extend(project.code_review_rules);
            merged.templates.extend(project.templates);
            merged.steering_docs.extend(project.steering_docs);
            merged
                .compliance_requirements
                .extend(project.compliance_requirements);
            merged.version = project.version;
            merged.updated_at = project.updated_at;
        }

        merged.updated_at = Utc::now();

        Ok(merged)
    }

    /// Validate overrides before applying
    fn validate_overrides(standards: &TeamStandards, overrides: &StandardsOverride) -> Result<()> {
        for override_id in &overrides.overridden_standards {
            let exists = standards
                .code_review_rules
                .iter()
                .any(|r| &r.id == override_id);

            if !exists {
                return Err(TeamError::ConfigError(format!(
                    "Override target not found: {}",
                    override_id
                )));
            }
        }

        Ok(())
    }
}

impl Default for TeamConfigManager {
    fn default() -> Self {
        Self::new()
    }
}
