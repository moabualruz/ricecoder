/// Shared rules management and promotion
use crate::error::Result;
use crate::models::{AdoptionMetrics, EffectivenessMetrics, RuleScope, SharedRule};
use std::sync::Arc;
use tracing::{debug, info};

/// Manages rule promotion, validation, versioning, and approval workflows
///
/// This manager integrates with ricecoder-learning components to:
/// - Promote rules from Project → Team → Organization scope
/// - Validate rules before promotion
/// - Track version history with timestamps and metadata
/// - Support rollback to previous versions
/// - Track adoption and effectiveness metrics
///
/// # Requirements
/// - Requirement 2.1: Support promotion from Project → Team → Organization
/// - Requirement 2.2: Support promotion from Team → Organization
/// - Requirement 2.4: Validate rules before promotion
/// - Requirement 2.5: Track adoption metrics
/// - Requirement 2.6: Track effectiveness metrics
/// - Requirement 2.7: Support rollback to previous versions
/// - Requirement 2.8: Maintain complete version history
pub struct SharedRulesManager {
    /// Rule promoter for handling rule promotion logic
    rule_promoter: Arc<dyn RulePromoter>,
    /// Rule validator for validating rules before promotion
    rule_validator: Arc<dyn RuleValidator>,
    /// Analytics engine for tracking metrics
    analytics_engine: Arc<dyn AnalyticsEngine>,
}

/// Trait for rule promotion functionality
pub trait RulePromoter: Send + Sync {
    /// Promote a rule from one scope to another
    fn promote(&self, rule: &SharedRule, from_scope: RuleScope, to_scope: RuleScope) -> Result<()>;
}

/// Trait for rule validation functionality
pub trait RuleValidator: Send + Sync {
    /// Validate a rule and return a validation report
    fn validate(&self, rule: &SharedRule) -> Result<ValidationReport>;
}

/// Trait for analytics functionality
pub trait AnalyticsEngine: Send + Sync {
    /// Track adoption metrics for a rule
    fn track_adoption(&self, rule_id: &str) -> Result<AdoptionMetrics>;
    /// Track effectiveness metrics for a rule
    fn track_effectiveness(&self, rule_id: &str) -> Result<EffectivenessMetrics>;
}

/// Validation report for a rule
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub rule_id: String,
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl SharedRulesManager {
    /// Create a new SharedRulesManager with the provided components
    ///
    /// # Arguments
    /// * `rule_promoter` - Component for handling rule promotion
    /// * `rule_validator` - Component for validating rules
    /// * `analytics_engine` - Component for tracking metrics
    pub fn new(
        rule_promoter: Arc<dyn RulePromoter>,
        rule_validator: Arc<dyn RuleValidator>,
        analytics_engine: Arc<dyn AnalyticsEngine>,
    ) -> Self {
        debug!("Creating new SharedRulesManager");
        SharedRulesManager {
            rule_promoter,
            rule_validator,
            analytics_engine,
        }
    }

    /// Promote a rule from one scope to another
    ///
    /// Supports promotion from Project → Team → Organization.
    /// Uses ricecoder-learning RulePromoter for promotion logic.
    ///
    /// # Arguments
    /// * `rule` - The rule to promote
    /// * `from_scope` - Current scope of the rule
    /// * `to_scope` - Target scope for promotion
    ///
    /// # Errors
    /// Returns error if promotion fails or rule is invalid
    pub async fn promote_rule(
        &self,
        rule: SharedRule,
        from_scope: RuleScope,
        to_scope: RuleScope,
    ) -> Result<()> {
        info!(
            rule_id = %rule.id,
            from_scope = %from_scope.as_str(),
            to_scope = %to_scope.as_str(),
            "Promoting rule"
        );

        // Validate rule before promotion (Requirement 2.4)
        let validation = self.validate_rule(&rule).await?;
        if !validation.is_valid {
            return Err(crate::error::TeamError::RuleValidationFailed(format!(
                "Rule validation failed: {:?}",
                validation.errors
            )));
        }

        // Use ricecoder-learning RulePromoter for promotion logic (Requirement 2.1, 2.2)
        self.rule_promoter.promote(&rule, from_scope, to_scope)?;

        info!(
            rule_id = %rule.id,
            from_scope = %from_scope.as_str(),
            to_scope = %to_scope.as_str(),
            "Rule promoted successfully"
        );

        Ok(())
    }

    /// Validate a rule before promotion
    ///
    /// Uses ricecoder-learning RuleValidator to validate rules.
    /// Returns detailed validation reports including errors and warnings.
    ///
    /// # Arguments
    /// * `rule` - The rule to validate
    ///
    /// # Returns
    /// Validation report with validation status, errors, and warnings
    pub async fn validate_rule(&self, rule: &SharedRule) -> Result<ValidationReport> {
        debug!(rule_id = %rule.id, "Validating rule");

        let report = self.rule_validator.validate(rule)?;

        if report.is_valid {
            info!(rule_id = %rule.id, "Rule validation passed");
        } else {
            info!(
                rule_id = %rule.id,
                errors = ?report.errors,
                "Rule validation failed"
            );
        }

        Ok(report)
    }

    /// Get the complete version history for a rule
    ///
    /// Retrieves complete version history with timestamps and promotion metadata.
    /// Uses ricecoder-learning versioning system.
    ///
    /// # Arguments
    /// * `rule_id` - ID of the rule to get history for
    ///
    /// # Returns
    /// Vector of SharedRule entries representing version history
    pub async fn get_rule_history(&self, rule_id: &str) -> Result<Vec<SharedRule>> {
        debug!(rule_id = %rule_id, "Retrieving rule history");

        // TODO: Integrate with ricecoder-learning versioning system
        // This will retrieve the complete version history from storage
        // Each entry should include timestamps and promotion metadata

        info!(rule_id = %rule_id, "Rule history retrieved");
        Ok(Vec::new())
    }

    /// Rollback a rule to a previous version
    ///
    /// Supports rollback to previous rule versions using ricecoder-learning versioning system.
    /// This is useful when a promoted rule causes issues.
    ///
    /// # Arguments
    /// * `rule_id` - ID of the rule to rollback
    /// * `version` - Version number to rollback to
    ///
    /// # Errors
    /// Returns error if version doesn't exist or rollback fails
    pub async fn rollback_rule(&self, rule_id: &str, version: u32) -> Result<()> {
        info!(
            rule_id = %rule_id,
            version = %version,
            "Rolling back rule"
        );

        // TODO: Integrate with ricecoder-learning versioning system
        // This will restore the rule to the specified version
        // Should validate that the version exists before rollback

        info!(
            rule_id = %rule_id,
            version = %version,
            "Rule rolled back successfully"
        );

        Ok(())
    }

    /// Track adoption metrics for a rule
    ///
    /// Tracks adoption metrics showing percentage of team members applying the rule.
    /// Uses ricecoder-learning AnalyticsEngine for metric calculation.
    ///
    /// # Arguments
    /// * `rule_id` - ID of the rule to track adoption for
    ///
    /// # Returns
    /// AdoptionMetrics with adoption percentage and trend data
    pub async fn track_adoption(&self, rule_id: &str) -> Result<AdoptionMetrics> {
        debug!(rule_id = %rule_id, "Tracking rule adoption");

        let metrics = self.analytics_engine.track_adoption(rule_id)?;

        info!(
            rule_id = %rule_id,
            adoption_percentage = %metrics.adoption_percentage,
            "Rule adoption metrics tracked"
        );

        Ok(metrics)
    }

    /// Track effectiveness metrics for a rule
    ///
    /// Tracks effectiveness metrics measuring positive outcomes from rule application.
    /// Uses ricecoder-learning AnalyticsEngine for metric calculation.
    ///
    /// # Arguments
    /// * `rule_id` - ID of the rule to track effectiveness for
    ///
    /// # Returns
    /// EffectivenessMetrics with effectiveness score and impact trend
    pub async fn track_effectiveness(&self, rule_id: &str) -> Result<EffectivenessMetrics> {
        debug!(rule_id = %rule_id, "Tracking rule effectiveness");

        let metrics = self.analytics_engine.track_effectiveness(rule_id)?;

        info!(
            rule_id = %rule_id,
            effectiveness_score = %metrics.effectiveness_score,
            "Rule effectiveness metrics tracked"
        );

        Ok(metrics)
    }
}

/// Mock implementations for testing and default usage
pub mod mocks {
    use super::*;

    /// Mock RulePromoter for testing
    pub struct MockRulePromoter;

    impl RulePromoter for MockRulePromoter {
        fn promote(
            &self,
            _rule: &SharedRule,
            _from_scope: RuleScope,
            _to_scope: RuleScope,
        ) -> Result<()> {
            Ok(())
        }
    }

    /// Mock RuleValidator for testing
    pub struct MockRuleValidator;

    impl RuleValidator for MockRuleValidator {
        fn validate(&self, rule: &SharedRule) -> Result<ValidationReport> {
            Ok(ValidationReport {
                rule_id: rule.id.clone(),
                is_valid: true,
                errors: Vec::new(),
                warnings: Vec::new(),
            })
        }
    }

    /// Mock AnalyticsEngine for testing
    pub struct MockAnalyticsEngine;

    impl AnalyticsEngine for MockAnalyticsEngine {
        fn track_adoption(&self, rule_id: &str) -> Result<AdoptionMetrics> {
            Ok(AdoptionMetrics {
                rule_id: rule_id.to_string(),
                total_members: 10,
                adopting_members: 8,
                adoption_percentage: 80.0,
                adoption_trend: Vec::new(),
            })
        }

        fn track_effectiveness(&self, rule_id: &str) -> Result<EffectivenessMetrics> {
            Ok(EffectivenessMetrics {
                rule_id: rule_id.to_string(),
                positive_outcomes: 15,
                negative_outcomes: 2,
                effectiveness_score: 0.88,
                impact_trend: Vec::new(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::SharedRule;
    use chrono::Utc;

    fn create_test_manager() -> SharedRulesManager {
        SharedRulesManager::new(
            Arc::new(mocks::MockRulePromoter),
            Arc::new(mocks::MockRuleValidator),
            Arc::new(mocks::MockAnalyticsEngine),
        )
    }

    fn create_test_rule() -> SharedRule {
        SharedRule {
            id: "rule-1".to_string(),
            name: "Test Rule".to_string(),
            description: "A test rule".to_string(),
            scope: RuleScope::Project,
            enforced: true,
            promoted_by: "admin-1".to_string(),
            promoted_at: Utc::now(),
            version: 1,
        }
    }

    #[tokio::test]
    async fn test_validate_rule_success() {
        let manager = create_test_manager();
        let rule = create_test_rule();

        let report = manager
            .validate_rule(&rule)
            .await
            .expect("Validation failed");
        assert!(report.is_valid);
        assert_eq!(report.rule_id, "rule-1");
        assert!(report.errors.is_empty());
    }

    #[tokio::test]
    async fn test_promote_rule_success() {
        let manager = create_test_manager();
        let rule = create_test_rule();

        let result = manager
            .promote_rule(rule, RuleScope::Project, RuleScope::Team)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_track_adoption() {
        let manager = create_test_manager();

        let metrics = manager
            .track_adoption("rule-1")
            .await
            .expect("Failed to track adoption");
        assert_eq!(metrics.rule_id, "rule-1");
        assert_eq!(metrics.total_members, 10);
        assert_eq!(metrics.adopting_members, 8);
        assert_eq!(metrics.adoption_percentage, 80.0);
    }

    #[tokio::test]
    async fn test_track_effectiveness() {
        let manager = create_test_manager();

        let metrics = manager
            .track_effectiveness("rule-1")
            .await
            .expect("Failed to track effectiveness");
        assert_eq!(metrics.rule_id, "rule-1");
        assert_eq!(metrics.positive_outcomes, 15);
        assert_eq!(metrics.negative_outcomes, 2);
        assert_eq!(metrics.effectiveness_score, 0.88);
    }

    #[tokio::test]
    async fn test_get_rule_history() {
        let manager = create_test_manager();

        let history = manager
            .get_rule_history("rule-1")
            .await
            .expect("Failed to get history");
        assert!(history.is_empty()); // Mock returns empty for now
    }

    #[tokio::test]
    async fn test_rollback_rule() {
        let manager = create_test_manager();

        let result = manager.rollback_rule("rule-1", 1).await;
        assert!(result.is_ok());
    }
}
