use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Rule review interface for managing rule promotion reviews
use crate::models::Rule;

/// Detailed comparison between two rule versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleComparison {
    /// Original rule
    pub original: Rule,
    /// Updated rule
    pub updated: Rule,
    /// Fields that changed
    pub changed_fields: Vec<String>,
    /// Detailed changes
    pub changes: ComparisonDetails,
}

/// Detailed comparison information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonDetails {
    /// Pattern changed
    pub pattern_changed: bool,
    /// Old pattern
    pub old_pattern: Option<String>,
    /// New pattern
    pub new_pattern: Option<String>,
    /// Action changed
    pub action_changed: bool,
    /// Old action
    pub old_action: Option<String>,
    /// New action
    pub new_action: Option<String>,
    /// Confidence changed
    pub confidence_changed: bool,
    /// Old confidence
    pub old_confidence: Option<f32>,
    /// New confidence
    pub new_confidence: Option<f32>,
    /// Metadata changed
    pub metadata_changed: bool,
    /// Old metadata
    pub old_metadata: Option<serde_json::Value>,
    /// New metadata
    pub new_metadata: Option<serde_json::Value>,
}

/// Review status for a rule
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewStatus {
    /// Pending review
    Pending,
    /// Approved
    Approved,
    /// Rejected
    Rejected,
    /// Needs revision
    NeedsRevision,
}

impl std::fmt::Display for ReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReviewStatus::Pending => write!(f, "pending"),
            ReviewStatus::Approved => write!(f, "approved"),
            ReviewStatus::Rejected => write!(f, "rejected"),
            ReviewStatus::NeedsRevision => write!(f, "needs_revision"),
        }
    }
}

/// Review comment on a rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewComment {
    /// Unique identifier
    pub id: String,
    /// Author of the comment
    pub author: String,
    /// Comment text
    pub text: String,
    /// When the comment was created
    pub created_at: DateTime<Utc>,
    /// Whether this is a critical comment
    pub is_critical: bool,
}

impl ReviewComment {
    /// Create a new review comment
    pub fn new(author: String, text: String, is_critical: bool) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            author,
            text,
            created_at: Utc::now(),
            is_critical,
        }
    }
}

/// Complete review information for a rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewInfo {
    /// Rule being reviewed
    pub rule: Rule,
    /// Current review status
    pub status: ReviewStatus,
    /// When the review was started
    pub started_at: DateTime<Utc>,
    /// When the review was completed
    pub completed_at: Option<DateTime<Utc>>,
    /// Reviewer name
    pub reviewer: Option<String>,
    /// Review comments
    pub comments: Vec<ReviewComment>,
    /// Comparison with previous version
    pub comparison: Option<RuleComparison>,
    /// Overall review score (0.0 to 1.0)
    pub review_score: Option<f32>,
}

impl ReviewInfo {
    /// Create a new review
    pub fn new(rule: Rule) -> Self {
        Self {
            rule,
            status: ReviewStatus::Pending,
            started_at: Utc::now(),
            completed_at: None,
            reviewer: None,
            comments: Vec::new(),
            comparison: None,
            review_score: None,
        }
    }

    /// Add a comment to the review
    pub fn add_comment(&mut self, comment: ReviewComment) {
        self.comments.push(comment);
    }

    /// Set the comparison
    pub fn set_comparison(&mut self, comparison: RuleComparison) {
        self.comparison = Some(comparison);
    }

    /// Approve the review
    pub fn approve(&mut self, reviewer: String, score: f32) {
        self.status = ReviewStatus::Approved;
        self.reviewer = Some(reviewer);
        self.completed_at = Some(Utc::now());
        self.review_score = Some(score);
    }

    /// Reject the review
    pub fn reject(&mut self, reviewer: String, score: f32) {
        self.status = ReviewStatus::Rejected;
        self.reviewer = Some(reviewer);
        self.completed_at = Some(Utc::now());
        self.review_score = Some(score);
    }

    /// Mark as needing revision
    pub fn request_revision(&mut self, reviewer: String) {
        self.status = ReviewStatus::NeedsRevision;
        self.reviewer = Some(reviewer);
    }

    /// Check if review is complete
    pub fn is_complete(&self) -> bool {
        self.status != ReviewStatus::Pending && self.completed_at.is_some()
    }

    /// Get critical comments
    pub fn get_critical_comments(&self) -> Vec<&ReviewComment> {
        self.comments.iter().filter(|c| c.is_critical).collect()
    }

    /// Get all comments
    pub fn get_comments(&self) -> &[ReviewComment] {
        &self.comments
    }

    /// Get comment count
    pub fn comment_count(&self) -> usize {
        self.comments.len()
    }

    /// Get critical comment count
    pub fn critical_comment_count(&self) -> usize {
        self.comments.iter().filter(|c| c.is_critical).count()
    }
}

/// Rule review manager
pub struct RuleReviewManager {
    /// Active reviews
    reviews: std::collections::HashMap<String, ReviewInfo>,
}

impl RuleReviewManager {
    /// Create a new review manager
    pub fn new() -> Self {
        Self {
            reviews: std::collections::HashMap::new(),
        }
    }

    /// Start a new review
    pub fn start_review(&mut self, rule: Rule) -> String {
        let review = ReviewInfo::new(rule.clone());
        let rule_id = rule.id.clone();
        self.reviews.insert(rule_id.clone(), review);
        rule_id
    }

    /// Get a review
    pub fn get_review(&self, rule_id: &str) -> Option<&ReviewInfo> {
        self.reviews.get(rule_id)
    }

    /// Get a mutable review
    pub fn get_review_mut(&mut self, rule_id: &str) -> Option<&mut ReviewInfo> {
        self.reviews.get_mut(rule_id)
    }

    /// Add a comment to a review
    pub fn add_comment(
        &mut self,
        rule_id: &str,
        author: String,
        text: String,
        is_critical: bool,
    ) -> crate::error::Result<()> {
        let review = self.reviews.get_mut(rule_id).ok_or_else(|| {
            crate::error::LearningError::RulePromotionFailed(format!(
                "Review not found for rule '{}'",
                rule_id
            ))
        })?;

        let comment = ReviewComment::new(author, text, is_critical);
        review.add_comment(comment);
        Ok(())
    }

    /// Approve a review
    pub fn approve_review(
        &mut self,
        rule_id: &str,
        reviewer: String,
        score: f32,
    ) -> crate::error::Result<()> {
        if !(0.0..=1.0).contains(&score) {
            return Err(crate::error::LearningError::RulePromotionFailed(
                "Review score must be between 0.0 and 1.0".to_string(),
            ));
        }

        let review = self.reviews.get_mut(rule_id).ok_or_else(|| {
            crate::error::LearningError::RulePromotionFailed(format!(
                "Review not found for rule '{}'",
                rule_id
            ))
        })?;

        review.approve(reviewer, score);
        Ok(())
    }

    /// Reject a review
    pub fn reject_review(
        &mut self,
        rule_id: &str,
        reviewer: String,
        score: f32,
    ) -> crate::error::Result<()> {
        if !(0.0..=1.0).contains(&score) {
            return Err(crate::error::LearningError::RulePromotionFailed(
                "Review score must be between 0.0 and 1.0".to_string(),
            ));
        }

        let review = self.reviews.get_mut(rule_id).ok_or_else(|| {
            crate::error::LearningError::RulePromotionFailed(format!(
                "Review not found for rule '{}'",
                rule_id
            ))
        })?;

        review.reject(reviewer, score);
        Ok(())
    }

    /// Request revision
    pub fn request_revision(
        &mut self,
        rule_id: &str,
        reviewer: String,
    ) -> crate::error::Result<()> {
        let review = self.reviews.get_mut(rule_id).ok_or_else(|| {
            crate::error::LearningError::RulePromotionFailed(format!(
                "Review not found for rule '{}'",
                rule_id
            ))
        })?;

        review.request_revision(reviewer);
        Ok(())
    }

    /// Get all reviews
    pub fn get_all_reviews(&self) -> Vec<&ReviewInfo> {
        self.reviews.values().collect()
    }

    /// Get reviews by status
    pub fn get_reviews_by_status(&self, status: ReviewStatus) -> Vec<&ReviewInfo> {
        self.reviews
            .values()
            .filter(|r| r.status == status)
            .collect()
    }

    /// Get pending reviews
    pub fn get_pending_reviews(&self) -> Vec<&ReviewInfo> {
        self.get_reviews_by_status(ReviewStatus::Pending)
    }

    /// Get approved reviews
    pub fn get_approved_reviews(&self) -> Vec<&ReviewInfo> {
        self.get_reviews_by_status(ReviewStatus::Approved)
    }

    /// Get rejected reviews
    pub fn get_rejected_reviews(&self) -> Vec<&ReviewInfo> {
        self.get_reviews_by_status(ReviewStatus::Rejected)
    }

    /// Get reviews needing revision
    pub fn get_reviews_needing_revision(&self) -> Vec<&ReviewInfo> {
        self.get_reviews_by_status(ReviewStatus::NeedsRevision)
    }

    /// Remove a review
    pub fn remove_review(&mut self, rule_id: &str) -> Option<ReviewInfo> {
        self.reviews.remove(rule_id)
    }

    /// Clear all reviews
    pub fn clear_reviews(&mut self) {
        self.reviews.clear();
    }

    /// Get review count
    pub fn review_count(&self) -> usize {
        self.reviews.len()
    }

    /// Get pending review count
    pub fn pending_review_count(&self) -> usize {
        self.get_pending_reviews().len()
    }
}

impl Default for RuleReviewManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Compare two rules
pub fn compare_rules(original: &Rule, updated: &Rule) -> RuleComparison {
    let mut changed_fields = Vec::new();
    let mut details = ComparisonDetails {
        pattern_changed: false,
        old_pattern: None,
        new_pattern: None,
        action_changed: false,
        old_action: None,
        new_action: None,
        confidence_changed: false,
        old_confidence: None,
        new_confidence: None,
        metadata_changed: false,
        old_metadata: None,
        new_metadata: None,
    };

    if original.pattern != updated.pattern {
        changed_fields.push("pattern".to_string());
        details.pattern_changed = true;
        details.old_pattern = Some(original.pattern.clone());
        details.new_pattern = Some(updated.pattern.clone());
    }

    if original.action != updated.action {
        changed_fields.push("action".to_string());
        details.action_changed = true;
        details.old_action = Some(original.action.clone());
        details.new_action = Some(updated.action.clone());
    }

    if (original.confidence - updated.confidence).abs() > f32::EPSILON {
        changed_fields.push("confidence".to_string());
        details.confidence_changed = true;
        details.old_confidence = Some(original.confidence);
        details.new_confidence = Some(updated.confidence);
    }

    if original.metadata != updated.metadata {
        changed_fields.push("metadata".to_string());
        details.metadata_changed = true;
        details.old_metadata = Some(original.metadata.clone());
        details.new_metadata = Some(updated.metadata.clone());
    }

    RuleComparison {
        original: original.clone(),
        updated: updated.clone(),
        changed_fields,
        changes: details,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Rule, RuleScope, RuleSource};

    #[test]
    fn test_review_info_creation() {
        let rule = Rule::new(
            RuleScope::Project,
            "pattern".to_string(),
            "action".to_string(),
            RuleSource::Learned,
        );

        let review = ReviewInfo::new(rule.clone());
        assert_eq!(review.status, ReviewStatus::Pending);
        assert_eq!(review.rule.id, rule.id);
        assert!(!review.is_complete());
    }

    #[test]
    fn test_add_comment() {
        let rule = Rule::new(
            RuleScope::Project,
            "pattern".to_string(),
            "action".to_string(),
            RuleSource::Learned,
        );

        let mut review = ReviewInfo::new(rule);
        let comment = ReviewComment::new("reviewer".to_string(), "Looks good".to_string(), false);

        review.add_comment(comment);
        assert_eq!(review.comment_count(), 1);
    }

    #[test]
    fn test_approve_review() {
        let rule = Rule::new(
            RuleScope::Project,
            "pattern".to_string(),
            "action".to_string(),
            RuleSource::Learned,
        );

        let mut review = ReviewInfo::new(rule);
        review.approve("reviewer".to_string(), 0.9);

        assert_eq!(review.status, ReviewStatus::Approved);
        assert_eq!(review.reviewer, Some("reviewer".to_string()));
        assert_eq!(review.review_score, Some(0.9));
        assert!(review.is_complete());
    }

    #[test]
    fn test_reject_review() {
        let rule = Rule::new(
            RuleScope::Project,
            "pattern".to_string(),
            "action".to_string(),
            RuleSource::Learned,
        );

        let mut review = ReviewInfo::new(rule);
        review.reject("reviewer".to_string(), 0.2);

        assert_eq!(review.status, ReviewStatus::Rejected);
        assert_eq!(review.reviewer, Some("reviewer".to_string()));
        assert_eq!(review.review_score, Some(0.2));
        assert!(review.is_complete());
    }

    #[test]
    fn test_request_revision() {
        let rule = Rule::new(
            RuleScope::Project,
            "pattern".to_string(),
            "action".to_string(),
            RuleSource::Learned,
        );

        let mut review = ReviewInfo::new(rule);
        review.request_revision("reviewer".to_string());

        assert_eq!(review.status, ReviewStatus::NeedsRevision);
        assert_eq!(review.reviewer, Some("reviewer".to_string()));
    }

    #[test]
    fn test_critical_comments() {
        let rule = Rule::new(
            RuleScope::Project,
            "pattern".to_string(),
            "action".to_string(),
            RuleSource::Learned,
        );

        let mut review = ReviewInfo::new(rule);

        let comment1 =
            ReviewComment::new("reviewer1".to_string(), "Critical issue".to_string(), true);
        let comment2 = ReviewComment::new(
            "reviewer2".to_string(),
            "Minor suggestion".to_string(),
            false,
        );

        review.add_comment(comment1);
        review.add_comment(comment2);

        assert_eq!(review.comment_count(), 2);
        assert_eq!(review.critical_comment_count(), 1);
        assert_eq!(review.get_critical_comments().len(), 1);
    }

    #[test]
    fn test_review_manager_creation() {
        let manager = RuleReviewManager::new();
        assert_eq!(manager.review_count(), 0);
    }

    #[test]
    fn test_start_review() {
        let mut manager = RuleReviewManager::new();

        let rule = Rule::new(
            RuleScope::Project,
            "pattern".to_string(),
            "action".to_string(),
            RuleSource::Learned,
        );

        let rule_id = rule.id.clone();
        manager.start_review(rule);

        assert_eq!(manager.review_count(), 1);
        assert!(manager.get_review(&rule_id).is_some());
    }

    #[test]
    fn test_approve_review_manager() {
        let mut manager = RuleReviewManager::new();

        let rule = Rule::new(
            RuleScope::Project,
            "pattern".to_string(),
            "action".to_string(),
            RuleSource::Learned,
        );

        let rule_id = rule.id.clone();
        manager.start_review(rule);
        manager
            .approve_review(&rule_id, "reviewer".to_string(), 0.9)
            .unwrap();

        let review = manager.get_review(&rule_id).unwrap();
        assert_eq!(review.status, ReviewStatus::Approved);
    }

    #[test]
    fn test_get_reviews_by_status() {
        let mut manager = RuleReviewManager::new();

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
        let _rule2_id = rule2.id.clone();

        manager.start_review(rule1);
        manager.start_review(rule2);

        manager
            .approve_review(&rule1_id, "reviewer".to_string(), 0.9)
            .unwrap();

        let pending = manager.get_pending_reviews();
        assert_eq!(pending.len(), 1);

        let approved = manager.get_approved_reviews();
        assert_eq!(approved.len(), 1);
    }

    #[test]
    fn test_compare_rules() {
        let mut original = Rule::new(
            RuleScope::Project,
            "old_pattern".to_string(),
            "old_action".to_string(),
            RuleSource::Learned,
        );
        original.confidence = 0.5;

        let mut updated = Rule::new(
            RuleScope::Project,
            "new_pattern".to_string(),
            "new_action".to_string(),
            RuleSource::Learned,
        );
        updated.confidence = 0.8;

        let comparison = compare_rules(&original, &updated);

        assert!(comparison.changes.pattern_changed);
        assert!(comparison.changes.action_changed);
        assert!(comparison.changes.confidence_changed);
        assert_eq!(comparison.changed_fields.len(), 3);
    }

    #[test]
    fn test_invalid_review_score() {
        let mut manager = RuleReviewManager::new();

        let rule = Rule::new(
            RuleScope::Project,
            "pattern".to_string(),
            "action".to_string(),
            RuleSource::Learned,
        );

        let rule_id = rule.id.clone();
        manager.start_review(rule);

        let result = manager.approve_review(&rule_id, "reviewer".to_string(), 1.5);
        assert!(result.is_err());
    }
}
