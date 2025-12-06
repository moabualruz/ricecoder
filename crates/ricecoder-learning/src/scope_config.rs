/// Scope configuration and isolation
use crate::error::{LearningError, Result};
use crate::models::RuleScope;
use ricecoder_storage::manager::PathResolver;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

/// Scope configuration with learning control flags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeConfiguration {
    /// The scope this configuration applies to
    pub scope: RuleScope,
    /// Whether learning is enabled for this scope
    pub learning_enabled: bool,
    /// Whether to restrict learning to this scope only (project-only learning)
    pub project_only: bool,
    /// Whether approval is required for new rules
    pub approval_required: bool,
    /// Maximum number of rules to store in this scope
    pub max_rules: usize,
    /// Retention period in days
    pub retention_days: u32,
}

impl Default for ScopeConfiguration {
    fn default() -> Self {
        Self {
            scope: RuleScope::Global,
            learning_enabled: true,
            project_only: false,
            approval_required: false,
            max_rules: 10000,
            retention_days: 365,
        }
    }
}

impl ScopeConfiguration {
    /// Create a new scope configuration
    pub fn new(scope: RuleScope) -> Self {
        Self {
            scope,
            ..Default::default()
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        if self.max_rules == 0 {
            return Err(LearningError::ConfigurationError(
                "max_rules must be greater than 0".to_string(),
            ));
        }

        if self.retention_days == 0 {
            return Err(LearningError::ConfigurationError(
                "retention_days must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }
}

/// Scope configuration loader that handles project/user/default hierarchy
pub struct ScopeConfigurationLoader;

impl ScopeConfigurationLoader {
    /// Load configuration from project/user/defaults hierarchy
    pub async fn load_configuration(scope: RuleScope) -> Result<ScopeConfiguration> {
        // Try to load from project config first
        if let Ok(config) = Self::load_project_config(scope).await {
            return Ok(config);
        }

        // Fall back to user config
        if let Ok(config) = Self::load_user_config(scope).await {
            return Ok(config);
        }

        // Fall back to defaults
        Ok(ScopeConfiguration::new(scope))
    }

    /// Load configuration from project-level config file
    async fn load_project_config(scope: RuleScope) -> Result<ScopeConfiguration> {
        let config_path = Self::get_project_config_path(scope)?;

        if !config_path.exists() {
            return Err(LearningError::ConfigurationError(
                "Project config not found".to_string(),
            ));
        }

        let content = fs::read_to_string(&config_path)
            .await
            .map_err(|e| LearningError::ConfigurationError(format!("Failed to read project config: {}", e)))?;

        let config: ScopeConfiguration = serde_yaml::from_str(&content)
            .map_err(|e| LearningError::ConfigurationError(format!("Failed to parse project config: {}", e)))?;

        config.validate()?;
        Ok(config)
    }

    /// Load configuration from user-level config file
    async fn load_user_config(scope: RuleScope) -> Result<ScopeConfiguration> {
        let config_path = Self::get_user_config_path(scope)?;

        if !config_path.exists() {
            return Err(LearningError::ConfigurationError(
                "User config not found".to_string(),
            ));
        }

        let content = fs::read_to_string(&config_path)
            .await
            .map_err(|e| LearningError::ConfigurationError(format!("Failed to read user config: {}", e)))?;

        let config: ScopeConfiguration = serde_yaml::from_str(&content)
            .map_err(|e| LearningError::ConfigurationError(format!("Failed to parse user config: {}", e)))?;

        config.validate()?;
        Ok(config)
    }

    /// Get the project-level config file path
    fn get_project_config_path(scope: RuleScope) -> Result<PathBuf> {
        match scope {
            RuleScope::Global => Ok(PathBuf::from(".ricecoder/learning-global.yaml")),
            RuleScope::Project => Ok(PathBuf::from(".ricecoder/learning-project.yaml")),
            RuleScope::Session => Ok(PathBuf::from(".ricecoder/learning-session.yaml")),
        }
    }

    /// Get the user-level config file path
    fn get_user_config_path(scope: RuleScope) -> Result<PathBuf> {
        let home = PathResolver::resolve_global_path()?;
        let filename = match scope {
            RuleScope::Global => "learning-global.yaml",
            RuleScope::Project => "learning-project.yaml",
            RuleScope::Session => "learning-session.yaml",
        };
        Ok(home.join(filename))
    }

    /// Save configuration to project-level config file
    pub async fn save_project_config(config: &ScopeConfiguration) -> Result<()> {
        config.validate()?;

        let config_path = Self::get_project_config_path(config.scope)?;

        // Ensure directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| LearningError::ConfigurationError(format!("Failed to create config directory: {}", e)))?;
        }

        let yaml = serde_yaml::to_string(config)
            .map_err(|e| LearningError::ConfigurationError(format!("Failed to serialize config: {}", e)))?;

        fs::write(&config_path, yaml)
            .await
            .map_err(|e| LearningError::ConfigurationError(format!("Failed to write config file: {}", e)))?;

        Ok(())
    }

    /// Save configuration to user-level config file
    pub async fn save_user_config(config: &ScopeConfiguration) -> Result<()> {
        config.validate()?;

        let config_path = Self::get_user_config_path(config.scope)?;

        // Ensure directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| LearningError::ConfigurationError(format!("Failed to create config directory: {}", e)))?;
        }

        let yaml = serde_yaml::to_string(config)
            .map_err(|e| LearningError::ConfigurationError(format!("Failed to serialize config: {}", e)))?;

        fs::write(&config_path, yaml)
            .await
            .map_err(|e| LearningError::ConfigurationError(format!("Failed to write config file: {}", e)))?;

        Ok(())
    }
}

/// Scope filter for filtering rules by scope
pub struct ScopeFilter;

impl ScopeFilter {
    /// Filter rules by scope
    pub fn filter_by_scope(rules: &[crate::models::Rule], scope: RuleScope) -> Vec<crate::models::Rule> {
        rules.iter().filter(|r| r.scope == scope).cloned().collect()
    }

    /// Filter rules by multiple scopes
    pub fn filter_by_scopes(rules: &[crate::models::Rule], scopes: &[RuleScope]) -> Vec<crate::models::Rule> {
        rules
            .iter()
            .filter(|r| scopes.contains(&r.scope))
            .cloned()
            .collect()
    }

    /// Check if rules from different scopes interfere
    pub fn check_scope_interference(
        rules1: &[crate::models::Rule],
        rules2: &[crate::models::Rule],
    ) -> bool {
        // Rules interfere if they have the same pattern but different actions
        for rule1 in rules1 {
            for rule2 in rules2 {
                if rule1.pattern == rule2.pattern && rule1.action != rule2.action {
                    return true;
                }
            }
        }
        false
    }

    /// Get rules for a specific scope with precedence
    pub fn get_rules_with_precedence(
        rules: &[crate::models::Rule],
        scope: RuleScope,
    ) -> Vec<crate::models::Rule> {
        // For project scope, include both project and session rules
        // For global scope, include only global rules
        // For session scope, include only session rules
        match scope {
            RuleScope::Project => {
                Self::filter_by_scopes(rules, &[RuleScope::Project, RuleScope::Session])
            }
            RuleScope::Global => Self::filter_by_scope(rules, RuleScope::Global),
            RuleScope::Session => Self::filter_by_scope(rules, RuleScope::Session),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_configuration_creation() {
        let config = ScopeConfiguration::new(RuleScope::Global);
        assert_eq!(config.scope, RuleScope::Global);
        assert!(config.learning_enabled);
        assert!(!config.project_only);
        assert!(!config.approval_required);
    }

    #[test]
    fn test_scope_configuration_validation() {
        let mut config = ScopeConfiguration::new(RuleScope::Project);
        assert!(config.validate().is_ok());

        config.max_rules = 0;
        assert!(config.validate().is_err());

        config.max_rules = 100;
        config.retention_days = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_scope_configuration_default() {
        let config = ScopeConfiguration::default();
        assert_eq!(config.scope, RuleScope::Global);
        assert!(config.learning_enabled);
        assert_eq!(config.max_rules, 10000);
        assert_eq!(config.retention_days, 365);
    }

    #[test]
    fn test_scope_filter_by_scope() {
        let rules = vec![
            crate::models::Rule::new(
                RuleScope::Global,
                "pattern1".to_string(),
                "action1".to_string(),
                crate::models::RuleSource::Learned,
            ),
            crate::models::Rule::new(
                RuleScope::Project,
                "pattern2".to_string(),
                "action2".to_string(),
                crate::models::RuleSource::Learned,
            ),
            crate::models::Rule::new(
                RuleScope::Global,
                "pattern3".to_string(),
                "action3".to_string(),
                crate::models::RuleSource::Learned,
            ),
        ];

        let global_rules = ScopeFilter::filter_by_scope(&rules, RuleScope::Global);
        assert_eq!(global_rules.len(), 2);

        let project_rules = ScopeFilter::filter_by_scope(&rules, RuleScope::Project);
        assert_eq!(project_rules.len(), 1);
    }

    #[test]
    fn test_scope_filter_by_scopes() {
        let rules = vec![
            crate::models::Rule::new(
                RuleScope::Global,
                "pattern1".to_string(),
                "action1".to_string(),
                crate::models::RuleSource::Learned,
            ),
            crate::models::Rule::new(
                RuleScope::Project,
                "pattern2".to_string(),
                "action2".to_string(),
                crate::models::RuleSource::Learned,
            ),
            crate::models::Rule::new(
                RuleScope::Session,
                "pattern3".to_string(),
                "action3".to_string(),
                crate::models::RuleSource::Learned,
            ),
        ];

        let filtered = ScopeFilter::filter_by_scopes(&rules, &[RuleScope::Project, RuleScope::Session]);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_scope_interference_detection() {
        let rules1 = vec![crate::models::Rule::new(
            RuleScope::Global,
            "pattern".to_string(),
            "action1".to_string(),
            crate::models::RuleSource::Learned,
        )];

        let rules2 = vec![crate::models::Rule::new(
            RuleScope::Project,
            "pattern".to_string(),
            "action2".to_string(),
            crate::models::RuleSource::Learned,
        )];

        assert!(ScopeFilter::check_scope_interference(&rules1, &rules2));
    }

    #[test]
    fn test_scope_interference_no_conflict() {
        let rules1 = vec![crate::models::Rule::new(
            RuleScope::Global,
            "pattern1".to_string(),
            "action1".to_string(),
            crate::models::RuleSource::Learned,
        )];

        let rules2 = vec![crate::models::Rule::new(
            RuleScope::Project,
            "pattern2".to_string(),
            "action2".to_string(),
            crate::models::RuleSource::Learned,
        )];

        assert!(!ScopeFilter::check_scope_interference(&rules1, &rules2));
    }

    #[test]
    fn test_get_rules_with_precedence_project() {
        let rules = vec![
            crate::models::Rule::new(
                RuleScope::Global,
                "pattern1".to_string(),
                "action1".to_string(),
                crate::models::RuleSource::Learned,
            ),
            crate::models::Rule::new(
                RuleScope::Project,
                "pattern2".to_string(),
                "action2".to_string(),
                crate::models::RuleSource::Learned,
            ),
            crate::models::Rule::new(
                RuleScope::Session,
                "pattern3".to_string(),
                "action3".to_string(),
                crate::models::RuleSource::Learned,
            ),
        ];

        let filtered = ScopeFilter::get_rules_with_precedence(&rules, RuleScope::Project);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|r| r.scope == RuleScope::Project || r.scope == RuleScope::Session));
    }

    #[test]
    fn test_get_rules_with_precedence_global() {
        let rules = vec![
            crate::models::Rule::new(
                RuleScope::Global,
                "pattern1".to_string(),
                "action1".to_string(),
                crate::models::RuleSource::Learned,
            ),
            crate::models::Rule::new(
                RuleScope::Project,
                "pattern2".to_string(),
                "action2".to_string(),
                crate::models::RuleSource::Learned,
            ),
            crate::models::Rule::new(
                RuleScope::Session,
                "pattern3".to_string(),
                "action3".to_string(),
                crate::models::RuleSource::Learned,
            ),
        ];

        let filtered = ScopeFilter::get_rules_with_precedence(&rules, RuleScope::Global);
        assert_eq!(filtered.len(), 1);
        assert!(filtered.iter().all(|r| r.scope == RuleScope::Global));
    }

    #[test]
    fn test_scope_configuration_loading_defaults() {
        let config = ScopeConfiguration::new(RuleScope::Project);
        assert_eq!(config.scope, RuleScope::Project);
        assert!(config.learning_enabled);
        assert!(!config.project_only);
        assert!(!config.approval_required);
        assert_eq!(config.max_rules, 10000);
        assert_eq!(config.retention_days, 365);
    }

    #[test]
    fn test_scope_configuration_project_only_flag() {
        let mut config = ScopeConfiguration::new(RuleScope::Project);
        assert!(!config.project_only);

        config.project_only = true;
        assert!(config.project_only);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_scope_configuration_approval_required_flag() {
        let mut config = ScopeConfiguration::new(RuleScope::Global);
        assert!(!config.approval_required);

        config.approval_required = true;
        assert!(config.approval_required);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_scope_configuration_learning_enabled_flag() {
        let mut config = ScopeConfiguration::new(RuleScope::Session);
        assert!(config.learning_enabled);

        config.learning_enabled = false;
        assert!(!config.learning_enabled);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_scope_configuration_max_rules_validation() {
        let mut config = ScopeConfiguration::new(RuleScope::Global);
        config.max_rules = 100;
        assert!(config.validate().is_ok());

        config.max_rules = 0;
        assert!(config.validate().is_err());

        config.max_rules = 1;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_scope_configuration_retention_days_validation() {
        let mut config = ScopeConfiguration::new(RuleScope::Global);
        config.retention_days = 30;
        assert!(config.validate().is_ok());

        config.retention_days = 0;
        assert!(config.validate().is_err());

        config.retention_days = 1;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_scope_filter_empty_rules() {
        let rules: Vec<crate::models::Rule> = Vec::new();

        let global_rules = ScopeFilter::filter_by_scope(&rules, RuleScope::Global);
        assert_eq!(global_rules.len(), 0);

        let project_rules = ScopeFilter::filter_by_scope(&rules, RuleScope::Project);
        assert_eq!(project_rules.len(), 0);

        let session_rules = ScopeFilter::filter_by_scope(&rules, RuleScope::Session);
        assert_eq!(session_rules.len(), 0);
    }

    #[test]
    fn test_scope_filter_single_scope() {
        let rules = vec![
            crate::models::Rule::new(
                RuleScope::Global,
                "pattern1".to_string(),
                "action1".to_string(),
                crate::models::RuleSource::Learned,
            ),
            crate::models::Rule::new(
                RuleScope::Global,
                "pattern2".to_string(),
                "action2".to_string(),
                crate::models::RuleSource::Learned,
            ),
        ];

        let global_rules = ScopeFilter::filter_by_scope(&rules, RuleScope::Global);
        assert_eq!(global_rules.len(), 2);

        let project_rules = ScopeFilter::filter_by_scope(&rules, RuleScope::Project);
        assert_eq!(project_rules.len(), 0);
    }

    #[test]
    fn test_scope_filter_multiple_scopes_combination() {
        let rules = vec![
            crate::models::Rule::new(
                RuleScope::Global,
                "pattern1".to_string(),
                "action1".to_string(),
                crate::models::RuleSource::Learned,
            ),
            crate::models::Rule::new(
                RuleScope::Project,
                "pattern2".to_string(),
                "action2".to_string(),
                crate::models::RuleSource::Learned,
            ),
            crate::models::Rule::new(
                RuleScope::Session,
                "pattern3".to_string(),
                "action3".to_string(),
                crate::models::RuleSource::Learned,
            ),
            crate::models::Rule::new(
                RuleScope::Project,
                "pattern4".to_string(),
                "action4".to_string(),
                crate::models::RuleSource::Learned,
            ),
        ];

        let filtered = ScopeFilter::filter_by_scopes(&rules, &[RuleScope::Project, RuleScope::Session]);
        assert_eq!(filtered.len(), 3);
        assert!(filtered.iter().all(|r| r.scope == RuleScope::Project || r.scope == RuleScope::Session));
    }

    #[test]
    fn test_scope_interference_same_pattern_different_action() {
        let rules1 = vec![crate::models::Rule::new(
            RuleScope::Global,
            "pattern".to_string(),
            "action1".to_string(),
            crate::models::RuleSource::Learned,
        )];

        let rules2 = vec![crate::models::Rule::new(
            RuleScope::Project,
            "pattern".to_string(),
            "action2".to_string(),
            crate::models::RuleSource::Learned,
        )];

        assert!(ScopeFilter::check_scope_interference(&rules1, &rules2));
    }

    #[test]
    fn test_scope_interference_same_pattern_same_action() {
        let rules1 = vec![crate::models::Rule::new(
            RuleScope::Global,
            "pattern".to_string(),
            "action".to_string(),
            crate::models::RuleSource::Learned,
        )];

        let rules2 = vec![crate::models::Rule::new(
            RuleScope::Project,
            "pattern".to_string(),
            "action".to_string(),
            crate::models::RuleSource::Learned,
        )];

        assert!(!ScopeFilter::check_scope_interference(&rules1, &rules2));
    }

    #[test]
    fn test_scope_interference_multiple_rules() {
        let rules1 = vec![
            crate::models::Rule::new(
                RuleScope::Global,
                "pattern1".to_string(),
                "action1".to_string(),
                crate::models::RuleSource::Learned,
            ),
            crate::models::Rule::new(
                RuleScope::Global,
                "pattern2".to_string(),
                "action2".to_string(),
                crate::models::RuleSource::Learned,
            ),
        ];

        let rules2 = vec![
            crate::models::Rule::new(
                RuleScope::Project,
                "pattern1".to_string(),
                "action_different".to_string(),
                crate::models::RuleSource::Learned,
            ),
            crate::models::Rule::new(
                RuleScope::Project,
                "pattern3".to_string(),
                "action3".to_string(),
                crate::models::RuleSource::Learned,
            ),
        ];

        assert!(ScopeFilter::check_scope_interference(&rules1, &rules2));
    }

    #[test]
    fn test_scope_precedence_project_includes_session() {
        let rules = vec![
            crate::models::Rule::new(
                RuleScope::Global,
                "pattern1".to_string(),
                "action1".to_string(),
                crate::models::RuleSource::Learned,
            ),
            crate::models::Rule::new(
                RuleScope::Project,
                "pattern2".to_string(),
                "action2".to_string(),
                crate::models::RuleSource::Learned,
            ),
            crate::models::Rule::new(
                RuleScope::Session,
                "pattern3".to_string(),
                "action3".to_string(),
                crate::models::RuleSource::Learned,
            ),
        ];

        let project_rules = ScopeFilter::get_rules_with_precedence(&rules, RuleScope::Project);
        assert_eq!(project_rules.len(), 2);
        assert!(project_rules.iter().any(|r| r.scope == RuleScope::Project));
        assert!(project_rules.iter().any(|r| r.scope == RuleScope::Session));
        assert!(!project_rules.iter().any(|r| r.scope == RuleScope::Global));
    }

    #[test]
    fn test_scope_precedence_session_only_session() {
        let rules = vec![
            crate::models::Rule::new(
                RuleScope::Global,
                "pattern1".to_string(),
                "action1".to_string(),
                crate::models::RuleSource::Learned,
            ),
            crate::models::Rule::new(
                RuleScope::Project,
                "pattern2".to_string(),
                "action2".to_string(),
                crate::models::RuleSource::Learned,
            ),
            crate::models::Rule::new(
                RuleScope::Session,
                "pattern3".to_string(),
                "action3".to_string(),
                crate::models::RuleSource::Learned,
            ),
        ];

        let session_rules = ScopeFilter::get_rules_with_precedence(&rules, RuleScope::Session);
        assert_eq!(session_rules.len(), 1);
        assert!(session_rules.iter().all(|r| r.scope == RuleScope::Session));
    }
}
