/// Rule promotion workflow for promoting rules from project to global scope
use crate::conflict_resolver::ConflictResolver;
use crate::error::{LearningError, Result};
use crate::models::{Rule, RuleScope, RuleSource};
use chrono::Utc;
use std::collections::HashMap;

/// Metadata about a rule promotion
#[derive(Debug, Clone)]
pub struct PromotionMetadata {
    /// When the promotion was requested
    pub requested_at: chrono::DateTime<chrono::Utc>,
    /// When the promotion was completed (if approved)
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Whether the promotion was approved
    pub approved: bool,
    /// Reason for approval or rejection
    pub reason: Option<String>,
    /// Previous version of the rule (before promotion)
    pub previous_version: Option<Rule>,
}

/// Information about a rule for review
#[derive(Debug, Clone)]
pub struct RuleReview {
    /// The rule being reviewed
    pub rule: Rule,
    /// Metadata about the promotion
    pub promotion_metadata: PromotionMetadata,
    /// Conflicts detected with existing global rules
    pub conflicts: Vec<Rule>,
    /// Comparison with previous version
    pub version_changes: Option<VersionChanges>,
}

/// Changes between rule versions
#[derive(Debug, Clone)]
pub struct VersionChanges {
    /// Previous pattern
    pub previous_pattern: String,
    /// New pattern
    pub new_pattern: String,
    /// Previous action
    pub previous_action: String,
    /// New action
    pub new_action: String,
    /// Previous confidence
    pub previous_confidence: f32,
    /// New confidence
    pub new_confidence: f32,
}

/// Promotion history entry
#[derive(Debug, Clone)]
pub struct PromotionHistoryEntry {
    /// Rule ID
    pub rule_id: String,
    /// Source scope
    pub source_scope: RuleScope,
    /// Target scope
    pub target_scope: RuleScope,
    /// When the promotion occurred
    pub promoted_at: chrono::DateTime<chrono::Utc>,
    /// Whether it was approved
    pub approved: bool,
    /// Reason for approval or rejection
    pub reason: Option<String>,
}

/// Manages rule promotion from project to global scope
pub struct RulePromoter {
    /// Promotion history
    promotion_history: Vec<PromotionHistoryEntry>,
    /// Pending promotions awaiting approval
    pending_promotions: HashMap<String, RuleReview>,
}

impl RulePromoter {
    /// Create a new rule promoter
    pub fn new() -> Self {
        Self {
            promotion_history: Vec::new(),
            pending_promotions: HashMap::new(),
        }
    }

    /// Request promotion of a rule from project to global scope
    pub fn request_promotion(
        &mut self,
        rule: Rule,
        global_rules: &[Rule],
    ) -> Result<RuleReview> {
        // Validate that the rule is from project scope
        if rule.scope != RuleScope::Project {
            return Err(LearningError::RulePromotionFailed(
                format!(
                    "Can only promote rules from project scope, rule is in {} scope",
                    rule.scope
                ),
            ));
        }

        // Check for conflicts with existing global rules
        let conflicts = self.detect_conflicts(&rule, global_rules)?;

        // Create promotion metadata
        let promotion_metadata = PromotionMetadata {
            requested_at: Utc::now(),
            completed_at: None,
            approved: false,
            reason: None,
            previous_version: None,
        };

        // Create rule review
        let rule_review = RuleReview {
            rule: rule.clone(),
            promotion_metadata,
            conflicts,
            version_changes: None,
        };

        // Store in pending promotions
        self.pending_promotions
            .insert(rule.id.clone(), rule_review.clone());

        Ok(rule_review)
    }

    /// Detect conflicts between a rule and existing global rules
    fn detect_conflicts(&self, rule: &Rule, global_rules: &[Rule]) -> Result<Vec<Rule>> {
        let mut conflicts = Vec::new();

        for global_rule in global_rules {
            if ConflictResolver::detect_conflict(rule, global_rule) {
                conflicts.push(global_rule.clone());
            }
        }

        Ok(conflicts)
    }

    /// Approve a pending promotion
    pub fn approve_promotion(
        &mut self,
        rule_id: &str,
        reason: Option<String>,
    ) -> Result<Rule> {
        let mut rule_review = self
            .pending_promotions
            .remove(rule_id)
            .ok_or_else(|| {
                LearningError::RulePromotionFailed(format!(
                    "No pending promotion found for rule '{}'",
                    rule_id
                ))
            })?;

        // Update the rule to be in global scope and mark as promoted
        let mut promoted_rule = rule_review.rule.clone();
        promoted_rule.scope = RuleScope::Global;
        promoted_rule.source = RuleSource::Promoted;
        promoted_rule.version += 1;
        promoted_rule.updated_at = Utc::now();

        // Update promotion metadata
        rule_review.promotion_metadata.approved = true;
        rule_review.promotion_metadata.completed_at = Some(Utc::now());
        rule_review.promotion_metadata.reason = reason.clone();

        // Record in promotion history
        self.promotion_history.push(PromotionHistoryEntry {
            rule_id: promoted_rule.id.clone(),
            source_scope: RuleScope::Project,
            target_scope: RuleScope::Global,
            promoted_at: Utc::now(),
            approved: true,
            reason,
        });

        Ok(promoted_rule)
    }

    /// Reject a pending promotion
    pub fn reject_promotion(
        &mut self,
        rule_id: &str,
        reason: Option<String>,
    ) -> Result<()> {
        let mut rule_review = self
            .pending_promotions
            .remove(rule_id)
            .ok_or_else(|| {
                LearningError::RulePromotionFailed(format!(
                    "No pending promotion found for rule '{}'",
                    rule_id
                ))
            })?;

        // Update promotion metadata
        rule_review.promotion_metadata.approved = false;
        rule_review.promotion_metadata.completed_at = Some(Utc::now());
        rule_review.promotion_metadata.reason = reason.clone();

        // Record in promotion history
        self.promotion_history.push(PromotionHistoryEntry {
            rule_id: rule_review.rule.id.clone(),
            source_scope: RuleScope::Project,
            target_scope: RuleScope::Global,
            promoted_at: Utc::now(),
            approved: false,
            reason,
        });

        Ok(())
    }

    /// Get a pending promotion for review
    pub fn get_pending_promotion(&self, rule_id: &str) -> Result<RuleReview> {
        self.pending_promotions
            .get(rule_id)
            .cloned()
            .ok_or_else(|| {
                LearningError::RulePromotionFailed(format!(
                    "No pending promotion found for rule '{}'",
                    rule_id
                ))
            })
    }

    /// Get all pending promotions
    pub fn get_pending_promotions(&self) -> Vec<RuleReview> {
        self.pending_promotions.values().cloned().collect()
    }

    /// Get the number of pending promotions
    pub fn pending_promotion_count(&self) -> usize {
        self.pending_promotions.len()
    }

    /// Get promotion history
    pub fn get_promotion_history(&self) -> Vec<PromotionHistoryEntry> {
        self.promotion_history.clone()
    }

    /// Get promotion history for a specific rule
    pub fn get_promotion_history_for_rule(&self, rule_id: &str) -> Vec<PromotionHistoryEntry> {
        self.promotion_history
            .iter()
            .filter(|entry| entry.rule_id == rule_id)
            .cloned()
            .collect()
    }

    /// Get promotion history for a specific scope
    pub fn get_promotion_history_for_scope(
        &self,
        source_scope: RuleScope,
        target_scope: RuleScope,
    ) -> Vec<PromotionHistoryEntry> {
        self.promotion_history
            .iter()
            .filter(|entry| entry.source_scope == source_scope && entry.target_scope == target_scope)
            .cloned()
            .collect()
    }

    /// Get approved promotions from history
    pub fn get_approved_promotions(&self) -> Vec<PromotionHistoryEntry> {
        self.promotion_history
            .iter()
            .filter(|entry| entry.approved)
            .cloned()
            .collect()
    }

    /// Get rejected promotions from history
    pub fn get_rejected_promotions(&self) -> Vec<PromotionHistoryEntry> {
        self.promotion_history
            .iter()
            .filter(|entry| !entry.approved)
            .cloned()
            .collect()
    }

    /// Validate a promoted rule against global rules
    pub fn validate_promotion(
        &self,
        promoted_rule: &Rule,
        global_rules: &[Rule],
    ) -> Result<()> {
        // Check that the rule is in global scope
        if promoted_rule.scope != RuleScope::Global {
            return Err(LearningError::RulePromotionFailed(
                "Promoted rule must be in global scope".to_string(),
            ));
        }

        // Check that the rule source is Promoted
        if promoted_rule.source != RuleSource::Promoted {
            return Err(LearningError::RulePromotionFailed(
                "Promoted rule must have source 'Promoted'".to_string(),
            ));
        }

        // Check for conflicts with existing global rules
        for global_rule in global_rules {
            if global_rule.id != promoted_rule.id
                && ConflictResolver::detect_conflict(promoted_rule, global_rule)
            {
                return Err(LearningError::RulePromotionFailed(
                    format!(
                        "Promoted rule conflicts with existing global rule '{}': both match pattern '{}' but have different actions",
                        global_rule.id, promoted_rule.pattern
                    ),
                ));
            }
        }

        Ok(())
    }

    /// Compare two versions of a rule
    pub fn compare_versions(previous: &Rule, current: &Rule) -> VersionChanges {
        VersionChanges {
            previous_pattern: previous.pattern.clone(),
            new_pattern: current.pattern.clone(),
            previous_action: previous.action.clone(),
            new_action: current.action.clone(),
            previous_confidence: previous.confidence,
            new_confidence: current.confidence,
        }
    }

    /// Create a rule review with version comparison
    pub fn create_review_with_comparison(
        rule: Rule,
        previous_version: Option<Rule>,
        global_rules: &[Rule],
    ) -> Result<RuleReview> {
        let conflicts = ConflictResolver::find_conflicts(&[rule.clone()])
            .into_iter()
            .filter(|(r1, r2)| {
                global_rules.iter().any(|gr| {
                    (gr.id == r1.id || gr.id == r2.id)
                        && (gr.id != rule.id)
                })
            })
            .flat_map(|(r1, r2)| vec![r1, r2])
            .collect::<Vec<_>>();

        let version_changes = previous_version.as_ref().map(|prev| {
            Self::compare_versions(prev, &rule)
        });

        let promotion_metadata = PromotionMetadata {
            requested_at: Utc::now(),
            completed_at: None,
            approved: false,
            reason: None,
            previous_version,
        };

        Ok(RuleReview {
            rule,
            promotion_metadata,
            conflicts,
            version_changes,
        })
    }

    /// Clear all pending promotions
    pub fn clear_pending_promotions(&mut self) {
        self.pending_promotions.clear();
    }

    /// Clear promotion history
    pub fn clear_promotion_history(&mut self) {
        self.promotion_history.clear();
    }
}

impl Default for RulePromoter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Rule;

    #[test]
    fn test_rule_promoter_creation() {
        let promoter = RulePromoter::new();
        assert_eq!(promoter.pending_promotion_count(), 0);
        assert_eq!(promoter.get_promotion_history().len(), 0);
    }

    #[test]
    fn test_request_promotion() {
        let mut promoter = RulePromoter::new();

        let rule = Rule::new(
            RuleScope::Project,
            "pattern".to_string(),
            "action".to_string(),
            RuleSource::Learned,
        );

        let rule_id = rule.id.clone();
        let result = promoter.request_promotion(rule, &[]);
        assert!(result.is_ok());

        let review = result.unwrap();
        assert_eq!(review.rule.id, rule_id);
        assert!(!review.promotion_metadata.approved);
        assert_eq!(promoter.pending_promotion_count(), 1);
    }

    #[test]
    fn test_request_promotion_wrong_scope() {
        let mut promoter = RulePromoter::new();

        let rule = Rule::new(
            RuleScope::Global,
            "pattern".to_string(),
            "action".to_string(),
            RuleSource::Learned,
        );

        let result = promoter.request_promotion(rule, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_approve_promotion() {
        let mut promoter = RulePromoter::new();

        let rule = Rule::new(
            RuleScope::Project,
            "pattern".to_string(),
            "action".to_string(),
            RuleSource::Learned,
        );

        let rule_id = rule.id.clone();
        promoter.request_promotion(rule, &[]).unwrap();

        let result = promoter.approve_promotion(&rule_id, Some("Looks good".to_string()));
        assert!(result.is_ok());

        let promoted_rule = result.unwrap();
        assert_eq!(promoted_rule.scope, RuleScope::Global);
        assert_eq!(promoted_rule.source, RuleSource::Promoted);
        assert_eq!(promoted_rule.version, 2);

        assert_eq!(promoter.pending_promotion_count(), 0);
        assert_eq!(promoter.get_promotion_history().len(), 1);
    }

    #[test]
    fn test_reject_promotion() {
        let mut promoter = RulePromoter::new();

        let rule = Rule::new(
            RuleScope::Project,
            "pattern".to_string(),
            "action".to_string(),
            RuleSource::Learned,
        );

        let rule_id = rule.id.clone();
        promoter.request_promotion(rule, &[]).unwrap();

        let result = promoter.reject_promotion(&rule_id, Some("Not ready".to_string()));
        assert!(result.is_ok());

        assert_eq!(promoter.pending_promotion_count(), 0);
        assert_eq!(promoter.get_promotion_history().len(), 1);

        let history = promoter.get_promotion_history();
        assert!(!history[0].approved);
    }

    #[test]
    fn test_get_pending_promotion() {
        let mut promoter = RulePromoter::new();

        let rule = Rule::new(
            RuleScope::Project,
            "pattern".to_string(),
            "action".to_string(),
            RuleSource::Learned,
        );

        let rule_id = rule.id.clone();
        promoter.request_promotion(rule, &[]).unwrap();

        let result = promoter.get_pending_promotion(&rule_id);
        assert!(result.is_ok());

        let review = result.unwrap();
        assert_eq!(review.rule.id, rule_id);
    }

    #[test]
    fn test_get_pending_promotions() {
        let mut promoter = RulePromoter::new();

        let rule1 = Rule::new(
            RuleScope::Project,
            "pattern1".to_string(),
            "action1".to_string(),
            RuleSource::Learned,
        );

        let rule2 = Rule::new(
            RuleScope::Project,
            "pattern2".to_string(),
            "action2".to_string(),
            RuleSource::Learned,
        );

        promoter.request_promotion(rule1, &[]).unwrap();
        promoter.request_promotion(rule2, &[]).unwrap();

        let pending = promoter.get_pending_promotions();
        assert_eq!(pending.len(), 2);
    }

    #[test]
    fn test_detect_conflicts() {
        let promoter = RulePromoter::new();

        let project_rule = Rule::new(
            RuleScope::Project,
            "pattern".to_string(),
            "action1".to_string(),
            RuleSource::Learned,
        );

        let global_rule = Rule::new(
            RuleScope::Global,
            "pattern".to_string(),
            "action2".to_string(),
            RuleSource::Learned,
        );

        let conflicts = promoter.detect_conflicts(&project_rule, &[global_rule]).unwrap();
        assert_eq!(conflicts.len(), 1);
    }

    #[test]
    fn test_validate_promotion() {
        let promoter = RulePromoter::new();

        let mut promoted_rule = Rule::new(
            RuleScope::Global,
            "pattern".to_string(),
            "action".to_string(),
            RuleSource::Promoted,
        );

        let result = promoter.validate_promotion(&promoted_rule, &[]);
        assert!(result.is_ok());

        // Test with wrong scope
        promoted_rule.scope = RuleScope::Project;
        let result = promoter.validate_promotion(&promoted_rule, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_compare_versions() {
        let mut previous = Rule::new(
            RuleScope::Project,
            "old_pattern".to_string(),
            "old_action".to_string(),
            RuleSource::Learned,
        );
        previous.confidence = 0.5;

        let mut current = Rule::new(
            RuleScope::Project,
            "new_pattern".to_string(),
            "new_action".to_string(),
            RuleSource::Learned,
        );
        current.confidence = 0.8;

        let changes = RulePromoter::compare_versions(&previous, &current);
        assert_eq!(changes.previous_pattern, "old_pattern");
        assert_eq!(changes.new_pattern, "new_pattern");
        assert_eq!(changes.previous_confidence, 0.5);
        assert_eq!(changes.new_confidence, 0.8);
    }

    #[test]
    fn test_promotion_history() {
        let mut promoter = RulePromoter::new();

        let rule = Rule::new(
            RuleScope::Project,
            "pattern".to_string(),
            "action".to_string(),
            RuleSource::Learned,
        );

        let rule_id = rule.id.clone();
        promoter.request_promotion(rule, &[]).unwrap();
        promoter.approve_promotion(&rule_id, None).unwrap();

        let history = promoter.get_promotion_history();
        assert_eq!(history.len(), 1);
        assert!(history[0].approved);

        let rule_history = promoter.get_promotion_history_for_rule(&rule_id);
        assert_eq!(rule_history.len(), 1);
    }

    #[test]
    fn test_get_approved_promotions() {
        let mut promoter = RulePromoter::new();

        let rule1 = Rule::new(
            RuleScope::Project,
            "pattern1".to_string(),
            "action1".to_string(),
            RuleSource::Learned,
        );

        let rule2 = Rule::new(
            RuleScope::Project,
            "pattern2".to_string(),
            "action2".to_string(),
            RuleSource::Learned,
        );

        let rule1_id = rule1.id.clone();
        let rule2_id = rule2.id.clone();

        promoter.request_promotion(rule1, &[]).unwrap();
        promoter.request_promotion(rule2, &[]).unwrap();

        promoter.approve_promotion(&rule1_id, None).unwrap();
        promoter.reject_promotion(&rule2_id, None).unwrap();

        let approved = promoter.get_approved_promotions();
        assert_eq!(approved.len(), 1);
        assert!(approved[0].approved);

        let rejected = promoter.get_rejected_promotions();
        assert_eq!(rejected.len(), 1);
        assert!(!rejected[0].approved);
    }

    #[test]
    fn test_clear_pending_promotions() {
        let mut promoter = RulePromoter::new();

        let rule = Rule::new(
            RuleScope::Project,
            "pattern".to_string(),
            "action".to_string(),
            RuleSource::Learned,
        );

        promoter.request_promotion(rule, &[]).unwrap();
        assert_eq!(promoter.pending_promotion_count(), 1);

        promoter.clear_pending_promotions();
        assert_eq!(promoter.pending_promotion_count(), 0);
    }

    #[test]
    fn test_clear_promotion_history() {
        let mut promoter = RulePromoter::new();

        let rule = Rule::new(
            RuleScope::Project,
            "pattern".to_string(),
            "action".to_string(),
            RuleSource::Learned,
        );

        let rule_id = rule.id.clone();
        promoter.request_promotion(rule, &[]).unwrap();
        promoter.approve_promotion(&rule_id, None).unwrap();

        assert_eq!(promoter.get_promotion_history().len(), 1);

        promoter.clear_promotion_history();
        assert_eq!(promoter.get_promotion_history().len(), 0);
    }
}
