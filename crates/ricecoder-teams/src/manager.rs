/// Central team manager orchestrating all team operations
use crate::access::AccessControlManager;
use crate::analytics::AnalyticsDashboard;
use crate::config::TeamConfigManager;
use crate::error::{Result, TeamError};
use crate::models::{Team, TeamMember, TeamStandards};
use crate::rules::SharedRulesManager;
use crate::sync::SyncService;
use chrono::Utc;
use ricecoder_storage::PathResolver;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Central coordinator for all team operations
pub struct TeamManager {
    config_manager: Arc<TeamConfigManager>,
    rules_manager: Arc<SharedRulesManager>,
    access_control: Arc<AccessControlManager>,
    sync_service: Arc<SyncService>,
    analytics: Arc<AnalyticsDashboard>,
    /// In-memory cache of teams (team_id -> Team)
    teams_cache: Arc<RwLock<HashMap<String, Team>>>,
}

impl TeamManager {
    /// Create a new TeamManager
    pub fn new() -> Self {
        // Create SharedRulesManager with mock implementations
        // TODO: Replace with actual ricecoder-learning implementations
        let rules_manager = SharedRulesManager::new(
            Arc::new(crate::rules::mocks::MockRulePromoter),
            Arc::new(crate::rules::mocks::MockRuleValidator),
            Arc::new(crate::rules::mocks::MockAnalyticsEngine),
        );

        // Create AccessControlManager with default dependencies
        let access_control = AccessControlManager::default();

        TeamManager {
            config_manager: Arc::new(TeamConfigManager::new()),
            rules_manager: Arc::new(rules_manager),
            access_control: Arc::new(access_control),
            sync_service: Arc::new(SyncService::new()),
            analytics: Arc::new(AnalyticsDashboard::new()),
            teams_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new team with initial members and roles
    pub async fn create_team(&self, name: &str, members: Vec<TeamMember>) -> Result<Team> {
        let team_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        tracing::info!(team_id = %team_id, team_name = %name, "Creating team");

        // Create team object
        let team = Team {
            id: team_id.clone(),
            name: name.to_string(),
            organization_id: None,
            members: members.clone(),
            created_at: now,
            updated_at: now,
        };

        // Store team to persistent storage
        self.store_team(&team).await?;

        // Update cache
        let mut cache = self.teams_cache.write().await;
        cache.insert(team_id.clone(), team.clone());

        // Assign roles to members using access control
        for member in &members {
            self.access_control
                .assign_role(&team.id, &member.id, member.role)
                .await?;
        }

        // Initialize team storage and configuration
        let team_standards = TeamStandards {
            id: Uuid::new_v4().to_string(),
            team_id: team.id.clone(),
            code_review_rules: Vec::new(),
            templates: Vec::new(),
            steering_docs: Vec::new(),
            compliance_requirements: Vec::new(),
            version: 1,
            created_at: now,
            updated_at: now,
        };

        self.config_manager
            .store_standards(&team.id, team_standards)
            .await?;

        tracing::info!(team_id = %team.id, member_count = %members.len(), "Team created successfully");

        Ok(team)
    }

    /// Add a team member with assigned role
    pub async fn add_member(&self, team_id: &str, member: TeamMember) -> Result<()> {
        // Retrieve team from cache or storage
        let mut team = self.get_team(team_id).await?;

        // Check if member already exists
        if team.members.iter().any(|m| m.id == member.id) {
            return Err(TeamError::Internal(format!(
                "Member {} already exists in team",
                member.id
            )));
        }

        tracing::info!(
            team_id = %team_id,
            member_id = %member.id,
            role = %member.role.as_str(),
            "Adding team member"
        );

        // Add member to team
        team.members.push(member.clone());
        team.updated_at = Utc::now();

        // Store updated team
        self.store_team(&team).await?;

        // Update cache
        let mut cache = self.teams_cache.write().await;
        cache.insert(team_id.to_string(), team);

        // Assign role using access control
        self.access_control
            .assign_role(team_id, &member.id, member.role)
            .await?;

        tracing::info!(
            team_id = %team_id,
            member_id = %member.id,
            "Team member added successfully"
        );

        Ok(())
    }

    /// Remove a team member and revoke access
    pub async fn remove_member(&self, team_id: &str, member_id: &str) -> Result<()> {
        // Retrieve team from cache or storage
        let mut team = self.get_team(team_id).await?;

        // Check if member exists
        let initial_count = team.members.len();
        team.members.retain(|m| m.id != member_id);

        if team.members.len() == initial_count {
            return Err(TeamError::MemberNotFound(member_id.to_string()));
        }

        tracing::info!(
            team_id = %team_id,
            member_id = %member_id,
            "Removing team member"
        );

        team.updated_at = Utc::now();

        // Store updated team
        self.store_team(&team).await?;

        // Update cache
        let mut cache = self.teams_cache.write().await;
        cache.insert(team_id.to_string(), team);

        // Revoke access using access control
        self.access_control
            .revoke_access(team_id, member_id)
            .await?;

        tracing::info!(
            team_id = %team_id,
            member_id = %member_id,
            "Team member removed successfully"
        );

        Ok(())
    }

    /// Get team information and members
    pub async fn get_team(&self, team_id: &str) -> Result<Team> {
        // Check cache first
        {
            let cache = self.teams_cache.read().await;
            if let Some(team) = cache.get(team_id) {
                tracing::debug!(team_id = %team_id, "Retrieved team from cache");
                return Ok(team.clone());
            }
        }

        // Load from storage
        let team = self.load_team(team_id).await?;

        // Update cache
        let mut cache = self.teams_cache.write().await;
        cache.insert(team_id.to_string(), team.clone());

        tracing::info!(team_id = %team_id, "Retrieved team from storage");

        Ok(team)
    }

    /// Get reference to config manager
    pub fn config_manager(&self) -> Arc<TeamConfigManager> {
        self.config_manager.clone()
    }

    /// Get reference to rules manager
    pub fn rules_manager(&self) -> Arc<SharedRulesManager> {
        self.rules_manager.clone()
    }

    /// Get reference to access control manager
    pub fn access_control(&self) -> Arc<AccessControlManager> {
        self.access_control.clone()
    }

    /// Get reference to sync service
    pub fn sync_service(&self) -> Arc<SyncService> {
        self.sync_service.clone()
    }

    /// Get reference to analytics dashboard
    pub fn analytics(&self) -> Arc<AnalyticsDashboard> {
        self.analytics.clone()
    }

    // Helper functions

    /// Store team to persistent storage
    async fn store_team(&self, team: &Team) -> Result<()> {
        let storage_path = Self::resolve_team_path(&team.id)?;

        // Ensure parent directory exists
        if let Some(parent) = storage_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                TeamError::StorageError(format!("Failed to create storage directory: {}", e))
            })?;
        }

        // Serialize team to YAML
        let yaml_content = serde_yaml::to_string(team).map_err(TeamError::YamlError)?;

        // Write to file
        std::fs::write(&storage_path, yaml_content)
            .map_err(|e| TeamError::StorageError(format!("Failed to write team file: {}", e)))?;

        tracing::debug!(team_id = %team.id, path = ?storage_path, "Team stored successfully");

        Ok(())
    }

    /// Load team from persistent storage
    async fn load_team(&self, team_id: &str) -> Result<Team> {
        let storage_path = Self::resolve_team_path(team_id)?;

        if !storage_path.exists() {
            return Err(TeamError::TeamNotFound(format!(
                "Team not found: {}",
                team_id
            )));
        }

        let yaml_content = std::fs::read_to_string(&storage_path)
            .map_err(|e| TeamError::StorageError(format!("Failed to read team file: {}", e)))?;

        let team: Team = serde_yaml::from_str(&yaml_content).map_err(TeamError::YamlError)?;

        tracing::debug!(team_id = %team_id, path = ?storage_path, "Team loaded successfully");

        Ok(team)
    }

    /// Resolve the storage path for a team
    fn resolve_team_path(team_id: &str) -> Result<PathBuf> {
        let global_path = PathResolver::resolve_global_path()
            .map_err(|e| TeamError::StorageError(e.to_string()))?;

        let team_path = global_path.join("teams").join(team_id).join("team.yaml");

        Ok(team_path)
    }
}

impl Default for TeamManager {
    fn default() -> Self {
        Self::new()
    }
}
