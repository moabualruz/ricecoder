/// Core learning manager that orchestrates all learning operations
use crate::analytics_engine::AnalyticsEngine;
use crate::conflict_resolver::ConflictResolver;
use crate::decision_logger::DecisionLogger;
use crate::error::{LearningError, Result};
use crate::models::{Decision, DecisionContext, LearnedPattern, LearningConfig, Rule, RuleScope};
use crate::pattern_capturer::PatternCapturer;
use crate::pattern_validator::PatternValidator;
use crate::rule_promoter::RulePromoter;
use crate::rule_storage::RuleStorage;
use crate::rule_validator::RuleValidator;
use crate::scope_config::{ScopeConfiguration, ScopeConfigurationLoader, ScopeFilter};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Central coordinator for all learning operations
pub struct LearningManager {
    /// Configuration for the learning system
    config: Arc<RwLock<LearningConfig>>,
    /// Scope configuration with learning control flags
    scope_config: Arc<RwLock<ScopeConfiguration>>,
    /// Decision logger for capturing and retrieving decisions
    decision_logger: Arc<DecisionLogger>,
    /// Rule storage for persisting rules
    rule_storage: Arc<RuleStorage>,
    /// Rule validator for validating rules before storage
    rule_validator: Arc<RuleValidator>,
    /// Pattern capturer for extracting patterns from decisions
    pattern_capturer: Arc<PatternCapturer>,
    /// Pattern validator for validating patterns against decisions
    pattern_validator: Arc<PatternValidator>,
    /// Rule promoter for managing rule promotion workflow
    rule_promoter: Arc<RwLock<RulePromoter>>,
    /// In-memory storage for patterns
    patterns: Arc<RwLock<HashMap<String, LearnedPattern>>>,
    /// Analytics engine for tracking rule metrics
    analytics_engine: Arc<AnalyticsEngine>,
}

impl LearningManager {
    /// Create a new learning manager with the specified scope
    pub fn new(scope: RuleScope) -> Self {
        let config = LearningConfig::new(scope);
        let scope_config = ScopeConfiguration::new(scope);
        Self {
            config: Arc::new(RwLock::new(config.clone())),
            scope_config: Arc::new(RwLock::new(scope_config)),
            decision_logger: Arc::new(DecisionLogger::new()),
            rule_storage: Arc::new(RuleStorage::new(scope)),
            rule_validator: Arc::new(RuleValidator::new()),
            pattern_capturer: Arc::new(PatternCapturer::new()),
            pattern_validator: Arc::new(PatternValidator::new()),
            rule_promoter: Arc::new(RwLock::new(RulePromoter::new())),
            patterns: Arc::new(RwLock::new(HashMap::new())),
            analytics_engine: Arc::new(AnalyticsEngine::new()),
        }
    }

    /// Create a new learning manager with custom configuration
    pub fn with_config(config: LearningConfig) -> Result<Self> {
        config.validate()?;
        let scope = config.scope;
        let scope_config = ScopeConfiguration::new(scope);
        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            scope_config: Arc::new(RwLock::new(scope_config)),
            decision_logger: Arc::new(DecisionLogger::new()),
            rule_storage: Arc::new(RuleStorage::new(scope)),
            rule_validator: Arc::new(RuleValidator::new()),
            pattern_capturer: Arc::new(PatternCapturer::new()),
            pattern_validator: Arc::new(PatternValidator::new()),
            rule_promoter: Arc::new(RwLock::new(RulePromoter::new())),
            patterns: Arc::new(RwLock::new(HashMap::new())),
            analytics_engine: Arc::new(AnalyticsEngine::new()),
        })
    }

    /// Get the current configuration
    pub async fn get_config(&self) -> LearningConfig {
        self.config.read().await.clone()
    }

    /// Update the configuration
    pub async fn set_config(&self, config: LearningConfig) -> Result<()> {
        config.validate()?;
        *self.config.write().await = config;
        Ok(())
    }

    /// Get the current learning scope
    pub async fn get_scope(&self) -> RuleScope {
        self.config.read().await.scope
    }

    /// Check if learning is enabled
    pub async fn is_enabled(&self) -> bool {
        self.config.read().await.enabled
    }

    /// Enable or disable learning
    pub async fn set_enabled(&self, enabled: bool) {
        let mut config = self.config.write().await;
        config.enabled = enabled;
    }

    // ============================================================================
    // Scope Configuration Methods (Task 8.1, 8.2, 8.3)
    // ============================================================================

    /// Load scope configuration from project/user/defaults hierarchy
    pub async fn load_scope_configuration(&self) -> Result<ScopeConfiguration> {
        let scope = self.get_scope().await;
        let config = ScopeConfigurationLoader::load_configuration(scope).await?;
        *self.scope_config.write().await = config.clone();
        Ok(config)
    }

    /// Get the current scope configuration
    pub async fn get_scope_configuration(&self) -> ScopeConfiguration {
        self.scope_config.read().await.clone()
    }

    /// Update the scope configuration
    pub async fn set_scope_configuration(&self, config: ScopeConfiguration) -> Result<()> {
        config.validate()?;
        *self.scope_config.write().await = config;
        Ok(())
    }

    /// Check if learning is enabled for the current scope
    pub async fn is_scope_learning_enabled(&self) -> bool {
        self.scope_config.read().await.learning_enabled
    }

    /// Enable or disable learning for the current scope
    pub async fn set_scope_learning_enabled(&self, enabled: bool) {
        let mut config = self.scope_config.write().await;
        config.learning_enabled = enabled;
    }

    /// Check if project-only learning is enabled
    pub async fn is_project_only_learning(&self) -> bool {
        self.scope_config.read().await.project_only
    }

    /// Enable or disable project-only learning
    pub async fn set_project_only_learning(&self, project_only: bool) {
        let mut config = self.scope_config.write().await;
        config.project_only = project_only;
    }

    /// Check if approval is required for new rules
    pub async fn is_approval_required(&self) -> bool {
        self.scope_config.read().await.approval_required
    }

    /// Enable or disable approval requirement for new rules
    pub async fn set_approval_required(&self, required: bool) {
        let mut config = self.scope_config.write().await;
        config.approval_required = required;
    }

    /// Get the maximum number of rules for the current scope
    pub async fn get_max_rules(&self) -> usize {
        self.scope_config.read().await.max_rules
    }

    /// Set the maximum number of rules for the current scope
    pub async fn set_max_rules(&self, max_rules: usize) -> Result<()> {
        if max_rules == 0 {
            return Err(LearningError::ConfigurationError(
                "max_rules must be greater than 0".to_string(),
            ));
        }
        let mut config = self.scope_config.write().await;
        config.max_rules = max_rules;
        Ok(())
    }

    /// Get the retention period in days for the current scope
    pub async fn get_retention_days(&self) -> u32 {
        self.scope_config.read().await.retention_days
    }

    /// Set the retention period in days for the current scope
    pub async fn set_retention_days(&self, retention_days: u32) -> Result<()> {
        if retention_days == 0 {
            return Err(LearningError::ConfigurationError(
                "retention_days must be greater than 0".to_string(),
            ));
        }
        let mut config = self.scope_config.write().await;
        config.retention_days = retention_days;
        Ok(())
    }

    /// Save scope configuration to project-level config file
    pub async fn save_scope_configuration_to_project(&self) -> Result<()> {
        let config = self.scope_config.read().await.clone();
        ScopeConfigurationLoader::save_project_config(&config).await
    }

    /// Save scope configuration to user-level config file
    pub async fn save_scope_configuration_to_user(&self) -> Result<()> {
        let config = self.scope_config.read().await.clone();
        ScopeConfigurationLoader::save_user_config(&config).await
    }

    /// Get rules filtered by current scope
    pub async fn get_rules_for_scope(&self) -> Result<Vec<Rule>> {
        let scope = self.get_scope().await;
        let all_rules = self.get_rules().await?;
        Ok(ScopeFilter::get_rules_with_precedence(&all_rules, scope))
    }

    /// Get rules filtered by specific scope
    pub async fn get_rules_by_scope(&self, scope: RuleScope) -> Result<Vec<Rule>> {
        let all_rules = self.get_rules().await?;
        Ok(ScopeFilter::filter_by_scope(&all_rules, scope))
    }

    /// Check if rules from different scopes interfere
    pub async fn check_scope_interference(
        &self,
        scope1: RuleScope,
        scope2: RuleScope,
    ) -> Result<bool> {
        let all_rules = self.get_rules().await?;
        let rules1 = ScopeFilter::filter_by_scope(&all_rules, scope1);
        let rules2 = ScopeFilter::filter_by_scope(&all_rules, scope2);
        Ok(ScopeFilter::check_scope_interference(&rules1, &rules2))
    }

    /// Capture a decision with full metadata
    pub async fn capture_decision(&self, decision: Decision) -> Result<String> {
        if !self.is_enabled().await {
            return Err(LearningError::DecisionCaptureFailed(
                "Learning is disabled".to_string(),
            ));
        }

        self.decision_logger.log_decision(decision).await
    }

    /// Get all captured decisions
    pub async fn get_decisions(&self) -> Vec<Decision> {
        self.decision_logger.get_history().await
    }

    /// Get decisions by type
    pub async fn get_decisions_by_type(&self, decision_type: &str) -> Vec<Decision> {
        self.decision_logger.get_history_by_type(decision_type).await
    }

    /// Get decisions by context
    pub async fn get_decisions_by_context(&self, context: &DecisionContext) -> Vec<Decision> {
        self.decision_logger.get_history_by_context(context).await
    }

    /// Get a specific decision by ID
    pub async fn get_decision(&self, decision_id: &str) -> Result<Decision> {
        self.decision_logger.get_decision(decision_id).await
    }

    /// Replay decisions for validation
    pub async fn replay_decisions(&self) -> Vec<Decision> {
        self.decision_logger.replay_decisions().await
    }

    /// Replay decisions for a specific context
    pub async fn replay_decisions_for_context(&self, context: &DecisionContext) -> Vec<Decision> {
        self.decision_logger.replay_decisions_for_context(context).await
    }

    /// Get the number of captured decisions
    pub async fn decision_count(&self) -> usize {
        self.decision_logger.decision_count().await
    }

    /// Clear all decisions
    pub async fn clear_decisions(&self) {
        self.decision_logger.clear().await;
    }

    /// Get decision statistics
    pub async fn get_decision_statistics(&self) -> crate::decision_logger::DecisionStatistics {
        self.decision_logger.get_statistics().await
    }

    /// Store a rule (with validation)
    pub async fn store_rule(&self, rule: Rule) -> Result<String> {
        // Validate the rule before storage
        self.rule_validator.validate(&rule)?;

        // Check for conflicts with existing rules
        let existing_rules = self.rule_storage.list_rules().await?;
        self.rule_validator.check_conflicts(&rule, &existing_rules)?;

        // Store the validated rule
        self.rule_storage.store_rule(rule).await
    }

    /// Get a rule by ID
    pub async fn get_rule(&self, rule_id: &str) -> Result<Rule> {
        self.rule_storage.get_rule(rule_id).await
    }

    /// Get all rules
    pub async fn get_rules(&self) -> Result<Vec<Rule>> {
        self.rule_storage.list_rules().await
    }

    /// Delete a rule
    pub async fn delete_rule(&self, rule_id: &str) -> Result<()> {
        self.rule_storage.delete_rule(rule_id).await
    }

    /// Update a rule (with validation)
    pub async fn update_rule(&self, rule: Rule) -> Result<String> {
        // Validate the rule before update
        self.rule_validator.validate(&rule)?;

        // Store the validated rule
        self.rule_storage.update_rule(rule).await
    }

    /// Validate a rule without storing it
    pub fn validate_rule(&self, rule: &Rule) -> Result<()> {
        self.rule_validator.validate(rule)
    }

    /// Get a detailed validation report for a rule
    pub fn validate_rule_with_report(&self, rule: &Rule) -> crate::rule_validator::ValidationReport {
        self.rule_validator.validate_with_report(rule)
    }

    /// Check if a rule conflicts with existing rules
    pub async fn check_rule_conflicts(&self, rule: &Rule) -> Result<()> {
        let existing_rules = self.rule_storage.list_rules().await?;
        self.rule_validator.check_conflicts(rule, &existing_rules)
    }

    /// Get rules by pattern
    pub async fn get_rules_by_pattern(&self, pattern: &str) -> Result<Vec<Rule>> {
        self.rule_storage.get_rules_by_pattern(pattern).await
    }

    /// Get rules by source
    pub async fn get_rules_by_source(&self, source: crate::models::RuleSource) -> Result<Vec<Rule>> {
        self.rule_storage.get_rules_by_source(source).await
    }

    /// Get rules by confidence
    pub async fn get_rules_by_confidence(&self, min_confidence: f32) -> Result<Vec<Rule>> {
        self.rule_storage.get_rules_by_confidence(min_confidence).await
    }

    /// Get rules sorted by usage
    pub async fn get_rules_by_usage(&self) -> Result<Vec<Rule>> {
        self.rule_storage.get_rules_by_usage().await
    }

    /// Get rules by usage count
    pub async fn get_rules_by_usage_count(&self, min_usage: u64) -> Result<Vec<Rule>> {
        self.rule_storage.get_rules_by_usage_count(min_usage).await
    }

    /// Get rules by success rate
    pub async fn get_rules_by_success_rate(&self, min_success_rate: f32) -> Result<Vec<Rule>> {
        self.rule_storage.get_rules_by_success_rate(min_success_rate).await
    }

    /// Get rule count
    pub async fn rule_count(&self) -> Result<usize> {
        self.rule_storage.rule_count().await
    }

    /// Clear all rules
    pub async fn clear_rules(&self) -> Result<()> {
        self.rule_storage.clear_all().await
    }

    /// Load all rules into cache
    pub async fn load_rules(&self) -> Result<()> {
        self.rule_storage.load_all().await
    }

    /// Detect if two rules conflict
    pub fn detect_rule_conflict(rule1: &Rule, rule2: &Rule) -> bool {
        ConflictResolver::detect_conflict(rule1, rule2)
    }

    /// Find all conflicts in the current rules
    pub async fn find_rule_conflicts(&self) -> Result<Vec<(Rule, Rule)>> {
        let rules = self.get_rules().await?;
        Ok(ConflictResolver::find_conflicts(&rules))
    }

    /// Check if a rule conflicts with existing rules before storage
    pub async fn check_rule_conflicts_before_storage(&self, rule: &Rule) -> Result<()> {
        let existing_rules = self.get_rules().await?;
        ConflictResolver::check_conflicts(rule, &existing_rules)
    }

    /// Apply scope precedence to get the highest priority rule for a pattern
    pub async fn get_rule_by_pattern_with_precedence(&self, pattern: &str) -> Result<Option<Rule>> {
        let rules = self.get_rules().await?;
        Ok(ConflictResolver::get_highest_priority_rule(&rules, pattern))
    }

    /// Get all rules for a pattern, applying scope precedence
    pub async fn get_rules_by_pattern_with_precedence(&self, pattern: &str) -> Result<Vec<Rule>> {
        let rules = self.get_rules().await?;
        Ok(ConflictResolver::get_rules_by_pattern_with_precedence(&rules, pattern))
    }

    /// Resolve conflicts in all rules by applying scope precedence
    pub async fn resolve_all_conflicts(&self) -> Result<Vec<Rule>> {
        let rules = self.get_rules().await?;
        ConflictResolver::resolve_conflicts(&rules)
    }

    /// Check for conflicts between project and global rules
    pub async fn check_cross_scope_conflicts(
        &self,
        project_rules: &[Rule],
        global_rules: &[Rule],
    ) -> Vec<(Rule, Rule)> {
        ConflictResolver::check_cross_scope_conflicts(project_rules, global_rules)
    }

    /// Log conflict resolution decision
    pub fn log_conflict_resolution(selected_rule: &Rule, conflicting_rules: &[Rule]) -> String {
        ConflictResolver::log_conflict_resolution(selected_rule, conflicting_rules)
    }

    /// Store a pattern
    pub async fn store_pattern(&self, pattern: LearnedPattern) -> Result<String> {
        let pattern_id = pattern.id.clone();
        let mut patterns = self.patterns.write().await;
        patterns.insert(pattern_id.clone(), pattern);

        Ok(pattern_id)
    }

    /// Get a pattern
    pub async fn get_pattern(&self, pattern_id: &str) -> Result<LearnedPattern> {
        let patterns = self.patterns.read().await;
        patterns
            .get(pattern_id)
            .cloned()
            .ok_or_else(|| LearningError::PatternNotFound(pattern_id.to_string()))
    }

    /// Get all patterns
    pub async fn get_patterns(&self) -> Vec<LearnedPattern> {
        self.patterns
            .read()
            .await
            .values()
            .cloned()
            .collect()
    }

    /// Get patterns by type
    pub async fn get_patterns_by_type(&self, pattern_type: &str) -> Vec<LearnedPattern> {
        self.patterns
            .read()
            .await
            .values()
            .filter(|p| p.pattern_type == pattern_type)
            .cloned()
            .collect()
    }

    /// Delete a pattern
    pub async fn delete_pattern(&self, pattern_id: &str) -> Result<()> {
        let mut patterns = self.patterns.write().await;
        patterns
            .remove(pattern_id)
            .ok_or_else(|| LearningError::PatternNotFound(pattern_id.to_string()))?;
        Ok(())
    }

    /// Get the storage path for a given scope
    pub fn get_scope_path(&self, scope: RuleScope) -> Result<PathBuf> {
        match scope {
            RuleScope::Global => {
                let home = dirs::home_dir()
                    .ok_or_else(|| LearningError::PathResolutionFailed("Home directory not found".to_string()))?;
                Ok(home.join(".ricecoder").join("rules"))
            }
            RuleScope::Project => Ok(PathBuf::from(".ricecoder/rules")),
            RuleScope::Session => Err(LearningError::PathResolutionFailed(
                "Session scope has no persistent path".to_string(),
            )),
        }
    }

    /// Extract patterns from decision history
    pub async fn extract_patterns(&self) -> Result<Vec<LearnedPattern>> {
        let decisions = self.get_decisions().await;
        let patterns = self.pattern_capturer.extract_patterns(&decisions)?;
        Ok(patterns)
    }

    /// Extract patterns from decision history with detailed analysis
    pub async fn extract_patterns_with_analysis(
        &self,
    ) -> Result<Vec<(LearnedPattern, crate::pattern_capturer::PatternAnalysis)>> {
        let decisions = self.get_decisions().await;
        let results = self.pattern_capturer.extract_patterns_with_analysis(&decisions)?;
        Ok(results)
    }

    /// Validate a pattern against historical decisions
    pub async fn validate_pattern(&self, pattern: &LearnedPattern) -> Result<f32> {
        let decisions = self.get_decisions().await;
        let validation_score = self.pattern_capturer.validate_pattern(pattern, &decisions)?;
        Ok(validation_score)
    }

    /// Update pattern confidence based on validation results
    pub async fn update_pattern_confidence(
        &self,
        pattern_id: &str,
        validation_score: f32,
    ) -> Result<()> {
        let mut patterns = self.patterns.write().await;
        if let Some(pattern) = patterns.get_mut(pattern_id) {
            self.pattern_capturer.update_confidence(pattern, validation_score)?;
            Ok(())
        } else {
            Err(LearningError::PatternNotFound(pattern_id.to_string()))
        }
    }

    /// Validate a pattern using the pattern validator
    pub async fn validate_pattern_comprehensive(
        &self,
        pattern: &LearnedPattern,
    ) -> Result<crate::pattern_validator::ValidationResult> {
        let decisions = self.get_decisions().await;
        let validation_result = self.pattern_validator.validate_pattern(pattern, &decisions)?;
        Ok(validation_result)
    }

    /// Validate multiple patterns
    pub async fn validate_patterns(
        &self,
        patterns: &[LearnedPattern],
    ) -> Result<Vec<crate::pattern_validator::ValidationResult>> {
        let decisions = self.get_decisions().await;
        let validation_results = self.pattern_validator.validate_patterns(patterns, &decisions)?;
        Ok(validation_results)
    }

    /// Get validation statistics for all stored patterns
    pub async fn get_pattern_validation_statistics(
        &self,
    ) -> Result<crate::pattern_validator::ValidationStatistics> {
        let patterns: Vec<_> = self.patterns.read().await.values().cloned().collect();
        let validation_results = self.validate_patterns(&patterns).await?;
        let stats = self.pattern_validator.get_validation_statistics(&validation_results);
        Ok(stats)
    }

    /// Validate and update pattern confidence based on validation results
    pub async fn validate_and_update_pattern(
        &self,
        pattern_id: &str,
    ) -> Result<crate::pattern_validator::ValidationResult> {
        let pattern = self.get_pattern(pattern_id).await?;
        let validation_result = self.validate_pattern_comprehensive(&pattern).await?;

        // Update pattern confidence based on validation recommendation
        self.update_pattern_confidence(pattern_id, validation_result.confidence_recommendation)
            .await?;

        Ok(validation_result)
    }

    /// Capture patterns from recent decisions and store them
    pub async fn capture_and_store_patterns(&self) -> Result<Vec<String>> {
        let patterns = self.extract_patterns().await?;
        let mut pattern_ids = Vec::new();

        for pattern in patterns {
            let pattern_id = pattern.id.clone();
            self.store_pattern(pattern).await?;
            pattern_ids.push(pattern_id);
        }

        Ok(pattern_ids)
    }

    /// Get patterns by confidence threshold
    pub async fn get_patterns_by_confidence(&self, min_confidence: f32) -> Vec<LearnedPattern> {
        self.patterns
            .read()
            .await
            .values()
            .filter(|p| p.confidence >= min_confidence)
            .cloned()
            .collect()
    }

    /// Get patterns sorted by confidence
    pub async fn get_patterns_by_confidence_sorted(&self) -> Vec<LearnedPattern> {
        let mut patterns: Vec<_> = self.patterns.read().await.values().cloned().collect();
        patterns.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
        patterns
    }

    /// Get patterns sorted by occurrences
    pub async fn get_patterns_by_occurrences(&self) -> Vec<LearnedPattern> {
        let mut patterns: Vec<_> = self.patterns.read().await.values().cloned().collect();
        patterns.sort_by(|a, b| b.occurrences.cmp(&a.occurrences));
        patterns
    }

    /// Get pattern count
    pub async fn pattern_count(&self) -> usize {
        self.patterns.read().await.len()
    }

    /// Clear all patterns
    pub async fn clear_patterns(&self) {
        self.patterns.write().await.clear();
    }

    /// Request promotion of a rule from project to global scope
    pub async fn request_rule_promotion(&self, rule: Rule) -> Result<crate::rule_promoter::RuleReview> {
        // Get global rules for conflict checking
        let global_rules = self.rule_storage.list_rules().await?;

        // Request promotion
        let mut promoter = self.rule_promoter.write().await;
        promoter.request_promotion(rule, &global_rules)
    }

    /// Get a pending promotion for review
    pub async fn get_pending_promotion(&self, rule_id: &str) -> Result<crate::rule_promoter::RuleReview> {
        let promoter = self.rule_promoter.read().await;
        promoter.get_pending_promotion(rule_id)
    }

    /// Get all pending promotions
    pub async fn get_pending_promotions(&self) -> Vec<crate::rule_promoter::RuleReview> {
        let promoter = self.rule_promoter.read().await;
        promoter.get_pending_promotions()
    }

    /// Get the number of pending promotions
    pub async fn pending_promotion_count(&self) -> usize {
        let promoter = self.rule_promoter.read().await;
        promoter.pending_promotion_count()
    }

    /// Approve a pending promotion and store the promoted rule
    pub async fn approve_promotion(
        &self,
        rule_id: &str,
        reason: Option<String>,
    ) -> Result<Rule> {
        // Approve the promotion
        let mut promoter = self.rule_promoter.write().await;
        let promoted_rule = promoter.approve_promotion(rule_id, reason)?;

        // Validate the promoted rule
        promoter.validate_promotion(&promoted_rule, &self.rule_storage.list_rules().await?)?;

        // Store the promoted rule in the appropriate scope storage
        // If the promoted rule is Global-scoped, use a Global-scoped storage
        if promoted_rule.scope == RuleScope::Global {
            let global_storage = RuleStorage::new(RuleScope::Global);
            global_storage.store_rule(promoted_rule.clone()).await?;
        } else {
            self.rule_storage.store_rule(promoted_rule.clone()).await?;
        }

        Ok(promoted_rule)
    }

    /// Reject a pending promotion
    pub async fn reject_promotion(
        &self,
        rule_id: &str,
        reason: Option<String>,
    ) -> Result<()> {
        let mut promoter = self.rule_promoter.write().await;
        promoter.reject_promotion(rule_id, reason)
    }

    /// Get promotion history
    pub async fn get_promotion_history(&self) -> Vec<crate::rule_promoter::PromotionHistoryEntry> {
        let promoter = self.rule_promoter.read().await;
        promoter.get_promotion_history()
    }

    /// Get promotion history for a specific rule
    pub async fn get_promotion_history_for_rule(
        &self,
        rule_id: &str,
    ) -> Vec<crate::rule_promoter::PromotionHistoryEntry> {
        let promoter = self.rule_promoter.read().await;
        promoter.get_promotion_history_for_rule(rule_id)
    }

    /// Get approved promotions from history
    pub async fn get_approved_promotions(&self) -> Vec<crate::rule_promoter::PromotionHistoryEntry> {
        let promoter = self.rule_promoter.read().await;
        promoter.get_approved_promotions()
    }

    /// Get rejected promotions from history
    pub async fn get_rejected_promotions(&self) -> Vec<crate::rule_promoter::PromotionHistoryEntry> {
        let promoter = self.rule_promoter.read().await;
        promoter.get_rejected_promotions()
    }

    /// Clear all pending promotions
    pub async fn clear_pending_promotions(&self) {
        let mut promoter = self.rule_promoter.write().await;
        promoter.clear_pending_promotions();
    }

    /// Clear promotion history
    pub async fn clear_promotion_history(&self) {
        let mut promoter = self.rule_promoter.write().await;
        promoter.clear_promotion_history();
    }

    // ============================================================================
    // Rule Application Methods (Task 9)
    // ============================================================================

    /// Apply a single rule to a generation context
    pub async fn apply_rule_to_context(
        &self,
        rule: &Rule,
        context: &crate::rule_application::GenerationContext,
    ) -> crate::rule_application::RuleApplicationResult {
        crate::rule_application::RuleApplicationEngine::apply_rule(rule, context)
    }

    /// Apply multiple rules to a generation context
    pub async fn apply_rules_to_context(
        &self,
        rules: &[Rule],
        context: &crate::rule_application::GenerationContext,
    ) -> Vec<crate::rule_application::RuleApplicationResult> {
        crate::rule_application::RuleApplicationEngine::apply_rules(rules, context)
    }

    /// Apply rules with precedence to get the best matching rule
    pub async fn apply_rules_with_precedence(
        &self,
        rules: &[Rule],
        context: &crate::rule_application::GenerationContext,
    ) -> Option<crate::rule_application::RuleApplicationResult> {
        crate::rule_application::RuleApplicationEngine::apply_rules_with_precedence(rules, context)
    }

    /// Chain multiple rules together for sequential application
    pub async fn chain_rules(
        &self,
        rules: &[Rule],
        context: &crate::rule_application::GenerationContext,
    ) -> Result<Vec<crate::rule_application::RuleApplicationResult>> {
        crate::rule_application::RuleApplicationEngine::chain_rules(rules, context)
    }

    /// Compose multiple rules into a single action
    pub async fn compose_rules(
        &self,
        rules: &[Rule],
        context: &crate::rule_application::GenerationContext,
    ) -> Result<Option<String>> {
        crate::rule_application::RuleApplicationEngine::compose_rules(rules, context)
    }

    /// Validate that a rule can be applied to a context
    pub async fn validate_rule_application(
        &self,
        rule: &Rule,
        context: &crate::rule_application::GenerationContext,
    ) -> Result<()> {
        crate::rule_application::RuleApplicationEngine::validate_rule_application(rule, context)
    }

    /// Get all matching rules for a context
    pub async fn get_matching_rules(
        &self,
        context: &crate::rule_application::GenerationContext,
    ) -> Result<Vec<Rule>> {
        let rules = self.get_rules().await?;
        Ok(crate::rule_application::RuleApplicationEngine::get_matching_rules(&rules, context))
    }

    /// Get matching rules sorted by confidence
    pub async fn get_matching_rules_sorted(
        &self,
        context: &crate::rule_application::GenerationContext,
    ) -> Result<Vec<Rule>> {
        let rules = self.get_rules().await?;
        Ok(crate::rule_application::RuleApplicationEngine::get_matching_rules_sorted(&rules, context))
    }

    /// Get matching rules sorted by usage
    pub async fn get_matching_rules_by_usage(
        &self,
        context: &crate::rule_application::GenerationContext,
    ) -> Result<Vec<Rule>> {
        let rules = self.get_rules().await?;
        Ok(crate::rule_application::RuleApplicationEngine::get_matching_rules_by_usage(&rules, context))
    }

    /// Get matching rules sorted by success rate
    pub async fn get_matching_rules_by_success(
        &self,
        context: &crate::rule_application::GenerationContext,
    ) -> Result<Vec<Rule>> {
        let rules = self.get_rules().await?;
        Ok(crate::rule_application::RuleApplicationEngine::get_matching_rules_by_success(&rules, context))
    }

    /// Apply learned rules to guide code generation
    pub async fn apply_learned_rules_to_generation(
        &self,
        context: &crate::rule_application::GenerationContext,
    ) -> Result<Option<String>> {
        // Get all rules for the current scope
        let rules = self.get_rules_for_scope().await?;

        // Apply rules with precedence to get the best matching rule
        if let Some(result) = crate::rule_application::RuleApplicationEngine::apply_rules_with_precedence(&rules, context) {
            Ok(result.action)
        } else {
            Ok(None)
        }
    }

    /// Apply learned rules and get all matching actions
    pub async fn apply_learned_rules_get_all(
        &self,
        context: &crate::rule_application::GenerationContext,
    ) -> Result<Vec<String>> {
        // Get all rules for the current scope
        let rules = self.get_rules_for_scope().await?;

        // Apply all rules and collect matching actions
        let results = crate::rule_application::RuleApplicationEngine::apply_rules(&rules, context);
        let actions: Vec<String> = results
            .iter()
            .filter(|r| r.matched)
            .filter_map(|r| r.action.clone())
            .collect();

        Ok(actions)
    }

    /// Apply learned rules with context-based matching
    pub async fn apply_learned_rules_with_context(
        &self,
        generation_type: String,
        language: String,
        input: String,
    ) -> Result<Option<String>> {
        let context = crate::rule_application::GenerationContext::new(
            generation_type,
            language,
            input,
        );

        self.apply_learned_rules_to_generation(&context).await
    }

    // ============================================================================
    // Analytics Methods (Task 10)
    // ============================================================================

    /// Record a rule application in analytics
    pub async fn record_rule_application(
        &self,
        rule_id: String,
        success: bool,
        application_time_ms: f64,
    ) -> Result<()> {
        self.analytics_engine
            .record_application(rule_id, success, application_time_ms)
            .await
    }

    /// Get metrics for a specific rule
    pub async fn get_rule_metrics(&self, rule_id: &str) -> Result<Option<crate::analytics_engine::RuleMetrics>> {
        self.analytics_engine.get_rule_metrics(rule_id).await
    }

    /// Get all rule metrics
    pub async fn get_all_rule_metrics(&self) -> Result<Vec<crate::analytics_engine::RuleMetrics>> {
        self.analytics_engine.get_all_metrics().await
    }

    /// Update rule confidence based on validation results
    pub async fn update_rule_confidence(&self, rule_id: &str, new_confidence: f32) -> Result<()> {
        self.analytics_engine.update_confidence(rule_id, new_confidence).await
    }

    /// Generate analytics insights
    pub async fn generate_analytics_insights(&self) -> Result<crate::analytics_engine::AnalyticsInsights> {
        self.analytics_engine.generate_insights().await
    }

    /// Clear all analytics metrics
    pub async fn clear_analytics_metrics(&self) -> Result<()> {
        self.analytics_engine.clear_metrics().await
    }

    /// Get metrics for rules in a specific scope
    pub async fn get_metrics_for_scope(
        &self,
        scope: RuleScope,
    ) -> Result<Vec<crate::analytics_engine::RuleMetrics>> {
        let rules = self.get_rules().await?;
        self.analytics_engine.get_metrics_by_scope(&rules, scope).await
    }

    /// Export rules with metrics to JSON
    pub async fn export_rules_with_metrics(&self, description: Option<String>) -> Result<String> {
        let rules = self.get_rules().await?;
        crate::rule_exchange::RuleExporter::export_to_json(rules, description)
    }

    /// Export rules with metrics to file
    pub async fn export_rules_to_file(
        &self,
        path: &std::path::Path,
        description: Option<String>,
    ) -> Result<()> {
        let rules = self.get_rules().await?;
        crate::rule_exchange::RuleExporter::export_to_file(rules, path, description)
    }

    /// Import rules from JSON with validation
    pub async fn import_rules_from_json(&self, json: &str) -> Result<Vec<Rule>> {
        crate::rule_exchange::RuleImporter::import_from_json(json)
    }

    /// Import rules from file with validation
    pub async fn import_rules_from_file(&self, path: &std::path::Path) -> Result<Vec<Rule>> {
        crate::rule_exchange::RuleImporter::import_from_file(path)
    }

    /// Import and validate rules, returning both valid and invalid rules
    pub async fn import_and_validate_rules(
        &self,
        json: &str,
    ) -> Result<(Vec<Rule>, Vec<String>)> {
        crate::rule_exchange::RuleImporter::import_and_validate(json)
    }

    /// Store imported rules
    pub async fn store_imported_rules(&self, rules: Vec<Rule>) -> Result<Vec<String>> {
        let mut stored_ids = Vec::new();
        for rule in rules {
            let id = self.store_rule(rule).await?;
            stored_ids.push(id);
        }
        Ok(stored_ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{DecisionContext, RuleSource};

    #[tokio::test]
    async fn test_learning_manager_creation() {
        let manager = LearningManager::new(RuleScope::Global);
        assert_eq!(manager.get_scope().await, RuleScope::Global);
        assert!(manager.is_enabled().await);
    }

    #[tokio::test]
    async fn test_learning_manager_with_config() {
        let config = LearningConfig::new(RuleScope::Project);
        let manager = LearningManager::with_config(config).expect("Failed to create manager");
        assert_eq!(manager.get_scope().await, RuleScope::Project);
    }

    #[tokio::test]
    async fn test_capture_decision() {
        let manager = LearningManager::new(RuleScope::Session);

        let context = DecisionContext {
            project_path: PathBuf::from("/project"),
            file_path: PathBuf::from("/project/src/main.rs"),
            line_number: 10,
            agent_type: "test_agent".to_string(),
        };

        let decision = Decision::new(
            context,
            "test_type".to_string(),
            serde_json::json!({}),
            serde_json::json!({}),
        );

        let decision_id = decision.id.clone();
        let result = manager.capture_decision(decision).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), decision_id);

        let decisions = manager.get_decisions().await;
        assert_eq!(decisions.len(), 1);
    }

    #[tokio::test]
    async fn test_capture_decision_disabled() {
        let manager = LearningManager::new(RuleScope::Session);
        manager.set_enabled(false).await;

        let context = DecisionContext {
            project_path: PathBuf::from("/project"),
            file_path: PathBuf::from("/project/src/main.rs"),
            line_number: 10,
            agent_type: "test_agent".to_string(),
        };

        let decision = Decision::new(
            context,
            "test_type".to_string(),
            serde_json::json!({}),
            serde_json::json!({}),
        );

        let result = manager.capture_decision(decision).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_decisions_by_type() {
        let manager = LearningManager::new(RuleScope::Session);

        let context = DecisionContext {
            project_path: PathBuf::from("/project"),
            file_path: PathBuf::from("/project/src/main.rs"),
            line_number: 10,
            agent_type: "test_agent".to_string(),
        };

        let decision1 = Decision::new(
            context.clone(),
            "type_a".to_string(),
            serde_json::json!({}),
            serde_json::json!({}),
        );

        let decision2 = Decision::new(
            context.clone(),
            "type_b".to_string(),
            serde_json::json!({}),
            serde_json::json!({}),
        );

        let decision3 = Decision::new(
            context,
            "type_a".to_string(),
            serde_json::json!({}),
            serde_json::json!({}),
        );

        manager.capture_decision(decision1).await.unwrap();
        manager.capture_decision(decision2).await.unwrap();
        manager.capture_decision(decision3).await.unwrap();

        let type_a_decisions = manager.get_decisions_by_type("type_a").await;
        assert_eq!(type_a_decisions.len(), 2);

        let type_b_decisions = manager.get_decisions_by_type("type_b").await;
        assert_eq!(type_b_decisions.len(), 1);
    }

    #[tokio::test]
    async fn test_store_rule() {
        let manager = LearningManager::new(RuleScope::Session);

        let rule = Rule::new(
            RuleScope::Session,
            "pattern".to_string(),
            "action".to_string(),
            RuleSource::Learned,
        );

        let rule_id = rule.id.clone();
        let result = manager.store_rule(rule).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), rule_id);

        let retrieved = manager.get_rule(&rule_id).await;
        assert!(retrieved.is_ok());
    }

    #[tokio::test]
    async fn test_store_pattern() {
        let manager = LearningManager::new(RuleScope::Session);

        let pattern = LearnedPattern::new(
            "code_generation".to_string(),
            "Test pattern".to_string(),
        );

        let pattern_id = pattern.id.clone();
        let result = manager.store_pattern(pattern).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), pattern_id);

        let retrieved = manager.get_pattern(&pattern_id).await;
        assert!(retrieved.is_ok());
    }

    #[tokio::test]
    async fn test_get_patterns_by_type() {
        let manager = LearningManager::new(RuleScope::Session);

        let pattern1 = LearnedPattern::new(
            "code_generation".to_string(),
            "Pattern 1".to_string(),
        );

        let pattern2 = LearnedPattern::new(
            "refactoring".to_string(),
            "Pattern 2".to_string(),
        );

        let pattern3 = LearnedPattern::new(
            "code_generation".to_string(),
            "Pattern 3".to_string(),
        );

        manager.store_pattern(pattern1).await.unwrap();
        manager.store_pattern(pattern2).await.unwrap();
        manager.store_pattern(pattern3).await.unwrap();

        let code_gen_patterns = manager.get_patterns_by_type("code_generation").await;
        assert_eq!(code_gen_patterns.len(), 2);

        let refactoring_patterns = manager.get_patterns_by_type("refactoring").await;
        assert_eq!(refactoring_patterns.len(), 1);
    }

    #[tokio::test]
    async fn test_get_scope_path() {
        let manager = LearningManager::new(RuleScope::Global);

        let global_path = manager.get_scope_path(RuleScope::Global);
        assert!(global_path.is_ok());
        let path = global_path.unwrap();
        assert!(path.to_string_lossy().contains(".ricecoder"));

        let project_path = manager.get_scope_path(RuleScope::Project);
        assert!(project_path.is_ok());
        assert_eq!(project_path.unwrap(), PathBuf::from(".ricecoder/rules"));

        let session_path = manager.get_scope_path(RuleScope::Session);
        assert!(session_path.is_err());
    }

    #[tokio::test]
    async fn test_enable_disable_learning() {
        let manager = LearningManager::new(RuleScope::Session);

        assert!(manager.is_enabled().await);

        manager.set_enabled(false).await;
        assert!(!manager.is_enabled().await);

        manager.set_enabled(true).await;
        assert!(manager.is_enabled().await);
    }

    #[tokio::test]
    async fn test_clear_decisions() {
        let manager = LearningManager::new(RuleScope::Session);

        let context = DecisionContext {
            project_path: PathBuf::from("/project"),
            file_path: PathBuf::from("/project/src/main.rs"),
            line_number: 10,
            agent_type: "test_agent".to_string(),
        };

        let decision = Decision::new(
            context,
            "test_type".to_string(),
            serde_json::json!({}),
            serde_json::json!({}),
        );

        manager.capture_decision(decision).await.unwrap();
        assert_eq!(manager.get_decisions().await.len(), 1);

        manager.clear_decisions().await;
        assert_eq!(manager.get_decisions().await.len(), 0);
    }

    #[tokio::test]
    async fn test_get_decisions_by_context() {
        let manager = LearningManager::new(RuleScope::Session);

        let context1 = DecisionContext {
            project_path: PathBuf::from("/project1"),
            file_path: PathBuf::from("/project1/src/main.rs"),
            line_number: 10,
            agent_type: "agent1".to_string(),
        };

        let context2 = DecisionContext {
            project_path: PathBuf::from("/project2"),
            file_path: PathBuf::from("/project2/src/main.rs"),
            line_number: 20,
            agent_type: "agent2".to_string(),
        };

        let decision1 = Decision::new(
            context1.clone(),
            "type_a".to_string(),
            serde_json::json!({}),
            serde_json::json!({}),
        );

        let decision2 = Decision::new(
            context2.clone(),
            "type_b".to_string(),
            serde_json::json!({}),
            serde_json::json!({}),
        );

        let decision3 = Decision::new(
            context1.clone(),
            "type_a".to_string(),
            serde_json::json!({}),
            serde_json::json!({}),
        );

        manager.capture_decision(decision1).await.unwrap();
        manager.capture_decision(decision2).await.unwrap();
        manager.capture_decision(decision3).await.unwrap();

        let context1_decisions = manager.get_decisions_by_context(&context1).await;
        assert_eq!(context1_decisions.len(), 2);

        let context2_decisions = manager.get_decisions_by_context(&context2).await;
        assert_eq!(context2_decisions.len(), 1);
    }

    #[tokio::test]
    async fn test_get_decision() {
        let manager = LearningManager::new(RuleScope::Session);

        let context = DecisionContext {
            project_path: PathBuf::from("/project"),
            file_path: PathBuf::from("/project/src/main.rs"),
            line_number: 10,
            agent_type: "test_agent".to_string(),
        };

        let decision = Decision::new(
            context,
            "test_type".to_string(),
            serde_json::json!({}),
            serde_json::json!({}),
        );

        let decision_id = decision.id.clone();
        manager.capture_decision(decision).await.unwrap();

        let retrieved = manager.get_decision(&decision_id).await;
        assert!(retrieved.is_ok());
        assert_eq!(retrieved.unwrap().id, decision_id);
    }

    #[tokio::test]
    async fn test_replay_decisions() {
        let manager = LearningManager::new(RuleScope::Session);

        let context = DecisionContext {
            project_path: PathBuf::from("/project"),
            file_path: PathBuf::from("/project/src/main.rs"),
            line_number: 10,
            agent_type: "test_agent".to_string(),
        };

        let decision1 = Decision::new(
            context.clone(),
            "type_a".to_string(),
            serde_json::json!({}),
            serde_json::json!({}),
        );

        let decision2 = Decision::new(
            context.clone(),
            "type_b".to_string(),
            serde_json::json!({}),
            serde_json::json!({}),
        );

        manager.capture_decision(decision1).await.unwrap();
        manager.capture_decision(decision2).await.unwrap();

        let replayed = manager.replay_decisions().await;
        assert_eq!(replayed.len(), 2);
        assert_eq!(replayed[0].decision_type, "type_a");
        assert_eq!(replayed[1].decision_type, "type_b");
    }

    #[tokio::test]
    async fn test_replay_decisions_for_context() {
        let manager = LearningManager::new(RuleScope::Session);

        let context = DecisionContext {
            project_path: PathBuf::from("/project"),
            file_path: PathBuf::from("/project/src/main.rs"),
            line_number: 10,
            agent_type: "test_agent".to_string(),
        };

        let decision1 = Decision::new(
            context.clone(),
            "type_a".to_string(),
            serde_json::json!({}),
            serde_json::json!({}),
        );

        let decision2 = Decision::new(
            context.clone(),
            "type_b".to_string(),
            serde_json::json!({}),
            serde_json::json!({}),
        );

        manager.capture_decision(decision1).await.unwrap();
        manager.capture_decision(decision2).await.unwrap();

        let replayed = manager.replay_decisions_for_context(&context).await;
        assert_eq!(replayed.len(), 2);
    }

    #[tokio::test]
    async fn test_decision_count() {
        let manager = LearningManager::new(RuleScope::Session);

        assert_eq!(manager.decision_count().await, 0);

        let context = DecisionContext {
            project_path: PathBuf::from("/project"),
            file_path: PathBuf::from("/project/src/main.rs"),
            line_number: 10,
            agent_type: "test_agent".to_string(),
        };

        let decision = Decision::new(
            context,
            "test_type".to_string(),
            serde_json::json!({}),
            serde_json::json!({}),
        );

        manager.capture_decision(decision).await.unwrap();
        assert_eq!(manager.decision_count().await, 1);
    }

    #[tokio::test]
    async fn test_get_decision_statistics() {
        let manager = LearningManager::new(RuleScope::Session);

        let context = DecisionContext {
            project_path: PathBuf::from("/project"),
            file_path: PathBuf::from("/project/src/main.rs"),
            line_number: 10,
            agent_type: "agent1".to_string(),
        };

        let decision1 = Decision::new(
            context.clone(),
            "type_a".to_string(),
            serde_json::json!({}),
            serde_json::json!({}),
        );

        let decision2 = Decision::new(
            context.clone(),
            "type_b".to_string(),
            serde_json::json!({}),
            serde_json::json!({}),
        );

        let decision3 = Decision::new(
            context,
            "type_a".to_string(),
            serde_json::json!({}),
            serde_json::json!({}),
        );

        manager.capture_decision(decision1).await.unwrap();
        manager.capture_decision(decision2).await.unwrap();
        manager.capture_decision(decision3).await.unwrap();

        let stats = manager.get_decision_statistics().await;

        assert_eq!(stats.total_decisions, 3);
        assert_eq!(stats.decision_types.get("type_a"), Some(&2));
        assert_eq!(stats.decision_types.get("type_b"), Some(&1));
        assert_eq!(stats.agent_types.get("agent1"), Some(&3));
    }

    #[tokio::test]
    async fn test_extract_patterns() {
        let manager = LearningManager::new(RuleScope::Session);

        let context = DecisionContext {
            project_path: PathBuf::from("/project"),
            file_path: PathBuf::from("/project/src/main.rs"),
            line_number: 10,
            agent_type: "test_agent".to_string(),
        };

        let decision1 = Decision::new(
            context.clone(),
            "code_generation".to_string(),
            serde_json::json!({"input": "test"}),
            serde_json::json!({"output": "result"}),
        );

        let decision2 = Decision::new(
            context,
            "code_generation".to_string(),
            serde_json::json!({"input": "test"}),
            serde_json::json!({"output": "result"}),
        );

        manager.capture_decision(decision1).await.unwrap();
        manager.capture_decision(decision2).await.unwrap();

        let patterns = manager.extract_patterns().await.unwrap();
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].pattern_type, "code_generation");
    }

    #[tokio::test]
    async fn test_capture_and_store_patterns() {
        let manager = LearningManager::new(RuleScope::Session);

        let context = DecisionContext {
            project_path: PathBuf::from("/project"),
            file_path: PathBuf::from("/project/src/main.rs"),
            line_number: 10,
            agent_type: "test_agent".to_string(),
        };

        let decision1 = Decision::new(
            context.clone(),
            "code_generation".to_string(),
            serde_json::json!({"input": "test"}),
            serde_json::json!({"output": "result"}),
        );

        let decision2 = Decision::new(
            context,
            "code_generation".to_string(),
            serde_json::json!({"input": "test"}),
            serde_json::json!({"output": "result"}),
        );

        manager.capture_decision(decision1).await.unwrap();
        manager.capture_decision(decision2).await.unwrap();

        let pattern_ids = manager.capture_and_store_patterns().await.unwrap();
        assert_eq!(pattern_ids.len(), 1);

        let patterns = manager.get_patterns().await;
        assert_eq!(patterns.len(), 1);
    }

    #[tokio::test]
    async fn test_validate_pattern() {
        let manager = LearningManager::new(RuleScope::Session);

        let context = DecisionContext {
            project_path: PathBuf::from("/project"),
            file_path: PathBuf::from("/project/src/main.rs"),
            line_number: 10,
            agent_type: "test_agent".to_string(),
        };

        let decision1 = Decision::new(
            context.clone(),
            "code_generation".to_string(),
            serde_json::json!({"input": "test"}),
            serde_json::json!({"output": "result"}),
        );

        let decision2 = Decision::new(
            context,
            "code_generation".to_string(),
            serde_json::json!({"input": "test"}),
            serde_json::json!({"output": "result"}),
        );

        manager.capture_decision(decision1).await.unwrap();
        manager.capture_decision(decision2).await.unwrap();

        let patterns = manager.extract_patterns().await.unwrap();
        assert_eq!(patterns.len(), 1);

        let validation_score = manager.validate_pattern(&patterns[0]).await.unwrap();
        assert!(validation_score >= 0.0);
        assert!(validation_score <= 1.0);
    }

    #[tokio::test]
    async fn test_update_pattern_confidence() {
        let manager = LearningManager::new(RuleScope::Session);

        let context = DecisionContext {
            project_path: PathBuf::from("/project"),
            file_path: PathBuf::from("/project/src/main.rs"),
            line_number: 10,
            agent_type: "test_agent".to_string(),
        };

        let decision1 = Decision::new(
            context.clone(),
            "code_generation".to_string(),
            serde_json::json!({"input": "test"}),
            serde_json::json!({"output": "result"}),
        );

        let decision2 = Decision::new(
            context,
            "code_generation".to_string(),
            serde_json::json!({"input": "test"}),
            serde_json::json!({"output": "result"}),
        );

        manager.capture_decision(decision1).await.unwrap();
        manager.capture_decision(decision2).await.unwrap();

        let patterns = manager.extract_patterns().await.unwrap();
        assert!(!patterns.is_empty(), "Should extract at least one pattern");

        let pattern_id = patterns[0].id.clone();

        // Store the pattern first
        manager.store_pattern(patterns[0].clone()).await.unwrap();

        let result = manager.update_pattern_confidence(&pattern_id, 0.9).await;
        assert!(result.is_ok());

        let updated_pattern = manager.get_pattern(&pattern_id).await.unwrap();
        assert!(updated_pattern.confidence > 0.0);
    }

    #[tokio::test]
    async fn test_get_patterns_by_confidence() {
        let manager = LearningManager::new(RuleScope::Session);

        let mut pattern1 = LearnedPattern::new(
            "code_generation".to_string(),
            "Pattern 1".to_string(),
        );
        pattern1.confidence = 0.8;

        let mut pattern2 = LearnedPattern::new(
            "refactoring".to_string(),
            "Pattern 2".to_string(),
        );
        pattern2.confidence = 0.3;

        manager.store_pattern(pattern1).await.unwrap();
        manager.store_pattern(pattern2).await.unwrap();

        let high_confidence = manager.get_patterns_by_confidence(0.5).await;
        assert_eq!(high_confidence.len(), 1);
        assert!(high_confidence[0].confidence >= 0.5);
    }

    #[tokio::test]
    async fn test_get_patterns_by_confidence_sorted() {
        let manager = LearningManager::new(RuleScope::Session);

        let mut pattern1 = LearnedPattern::new(
            "code_generation".to_string(),
            "Pattern 1".to_string(),
        );
        pattern1.confidence = 0.5;

        let mut pattern2 = LearnedPattern::new(
            "refactoring".to_string(),
            "Pattern 2".to_string(),
        );
        pattern2.confidence = 0.9;

        let mut pattern3 = LearnedPattern::new(
            "analysis".to_string(),
            "Pattern 3".to_string(),
        );
        pattern3.confidence = 0.7;

        manager.store_pattern(pattern1).await.unwrap();
        manager.store_pattern(pattern2).await.unwrap();
        manager.store_pattern(pattern3).await.unwrap();

        let sorted = manager.get_patterns_by_confidence_sorted().await;
        assert_eq!(sorted.len(), 3);
        assert!(sorted[0].confidence >= sorted[1].confidence);
        assert!(sorted[1].confidence >= sorted[2].confidence);
    }

    #[tokio::test]
    async fn test_get_patterns_by_occurrences() {
        let manager = LearningManager::new(RuleScope::Session);

        let mut pattern1 = LearnedPattern::new(
            "code_generation".to_string(),
            "Pattern 1".to_string(),
        );
        pattern1.occurrences = 5;

        let mut pattern2 = LearnedPattern::new(
            "refactoring".to_string(),
            "Pattern 2".to_string(),
        );
        pattern2.occurrences = 10;

        let mut pattern3 = LearnedPattern::new(
            "analysis".to_string(),
            "Pattern 3".to_string(),
        );
        pattern3.occurrences = 3;

        manager.store_pattern(pattern1).await.unwrap();
        manager.store_pattern(pattern2).await.unwrap();
        manager.store_pattern(pattern3).await.unwrap();

        let sorted = manager.get_patterns_by_occurrences().await;
        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0].occurrences, 10);
        assert_eq!(sorted[1].occurrences, 5);
        assert_eq!(sorted[2].occurrences, 3);
    }

    #[tokio::test]
    async fn test_pattern_count() {
        let manager = LearningManager::new(RuleScope::Session);

        assert_eq!(manager.pattern_count().await, 0);

        let pattern1 = LearnedPattern::new(
            "code_generation".to_string(),
            "Pattern 1".to_string(),
        );

        let pattern2 = LearnedPattern::new(
            "refactoring".to_string(),
            "Pattern 2".to_string(),
        );

        manager.store_pattern(pattern1).await.unwrap();
        manager.store_pattern(pattern2).await.unwrap();

        assert_eq!(manager.pattern_count().await, 2);
    }

    #[tokio::test]
    async fn test_clear_patterns() {
        let manager = LearningManager::new(RuleScope::Session);

        let pattern = LearnedPattern::new(
            "code_generation".to_string(),
            "Pattern".to_string(),
        );

        manager.store_pattern(pattern).await.unwrap();
        assert_eq!(manager.pattern_count().await, 1);

        manager.clear_patterns().await;
        assert_eq!(manager.pattern_count().await, 0);
    }

    #[tokio::test]
    async fn test_request_rule_promotion() {
        let manager = LearningManager::new(RuleScope::Session);

        let rule = Rule::new(
            RuleScope::Project,
            "pattern".to_string(),
            "action".to_string(),
            crate::models::RuleSource::Learned,
        );

        let result = manager.request_rule_promotion(rule).await;
        assert!(result.is_ok());

        let _review = result.unwrap();
        assert_eq!(manager.pending_promotion_count().await, 1);
    }

    #[tokio::test]
    async fn test_get_pending_promotions() {
        let manager = LearningManager::new(RuleScope::Session);

        let rule1 = Rule::new(
            RuleScope::Project,
            "pattern1".to_string(),
            "action1".to_string(),
            crate::models::RuleSource::Learned,
        );

        let rule2 = Rule::new(
            RuleScope::Project,
            "pattern2".to_string(),
            "action2".to_string(),
            crate::models::RuleSource::Learned,
        );

        manager.request_rule_promotion(rule1).await.unwrap();
        manager.request_rule_promotion(rule2).await.unwrap();

        let pending = manager.get_pending_promotions().await;
        assert_eq!(pending.len(), 2);
    }

    #[tokio::test]
    async fn test_approve_promotion() {
        let manager = LearningManager::new(RuleScope::Session);

        let rule = Rule::new(
            RuleScope::Project,
            "pattern".to_string(),
            "action".to_string(),
            crate::models::RuleSource::Learned,
        );

        let rule_id = rule.id.clone();
        manager.request_rule_promotion(rule).await.unwrap();

        let result = manager.approve_promotion(&rule_id, Some("Looks good".to_string())).await;
        assert!(result.is_ok());

        let promoted_rule = result.unwrap();
        assert_eq!(promoted_rule.scope, RuleScope::Global);
        assert_eq!(promoted_rule.source, crate::models::RuleSource::Promoted);

        assert_eq!(manager.pending_promotion_count().await, 0);
    }

    #[tokio::test]
    async fn test_reject_promotion() {
        let manager = LearningManager::new(RuleScope::Session);

        let rule = Rule::new(
            RuleScope::Project,
            "pattern".to_string(),
            "action".to_string(),
            crate::models::RuleSource::Learned,
        );

        let rule_id = rule.id.clone();
        manager.request_rule_promotion(rule).await.unwrap();

        let result = manager.reject_promotion(&rule_id, Some("Not ready".to_string())).await;
        assert!(result.is_ok());

        assert_eq!(manager.pending_promotion_count().await, 0);
    }

    #[tokio::test]
    async fn test_promotion_history() {
        let manager = LearningManager::new(RuleScope::Session);

        let rule = Rule::new(
            RuleScope::Project,
            "pattern".to_string(),
            "action".to_string(),
            crate::models::RuleSource::Learned,
        );

        let rule_id = rule.id.clone();
        manager.request_rule_promotion(rule).await.unwrap();
        manager.approve_promotion(&rule_id, None).await.unwrap();

        let history = manager.get_promotion_history().await;
        assert_eq!(history.len(), 1);
        assert!(history[0].approved);

        let rule_history = manager.get_promotion_history_for_rule(&rule_id).await;
        assert_eq!(rule_history.len(), 1);
    }

    #[tokio::test]
    async fn test_get_approved_promotions() {
        let manager = LearningManager::new(RuleScope::Session);

        let rule1 = Rule::new(
            RuleScope::Project,
            "pattern1".to_string(),
            "action1".to_string(),
            crate::models::RuleSource::Learned,
        );

        let rule2 = Rule::new(
            RuleScope::Project,
            "pattern2".to_string(),
            "action2".to_string(),
            crate::models::RuleSource::Learned,
        );

        let rule1_id = rule1.id.clone();
        let rule2_id = rule2.id.clone();

        manager.request_rule_promotion(rule1).await.unwrap();
        manager.request_rule_promotion(rule2).await.unwrap();

        manager.approve_promotion(&rule1_id, None).await.unwrap();
        manager.reject_promotion(&rule2_id, None).await.unwrap();

        let approved = manager.get_approved_promotions().await;
        assert_eq!(approved.len(), 1);
        assert!(approved[0].approved);

        let rejected = manager.get_rejected_promotions().await;
        assert_eq!(rejected.len(), 1);
        assert!(!rejected[0].approved);
    }

    #[tokio::test]
    async fn test_clear_pending_promotions() {
        let manager = LearningManager::new(RuleScope::Session);

        let rule = Rule::new(
            RuleScope::Project,
            "pattern".to_string(),
            "action".to_string(),
            crate::models::RuleSource::Learned,
        );

        manager.request_rule_promotion(rule).await.unwrap();
        assert_eq!(manager.pending_promotion_count().await, 1);

        manager.clear_pending_promotions().await;
        assert_eq!(manager.pending_promotion_count().await, 0);
    }

    #[tokio::test]
    async fn test_clear_promotion_history() {
        let manager = LearningManager::new(RuleScope::Session);

        let rule = Rule::new(
            RuleScope::Project,
            "pattern".to_string(),
            "action".to_string(),
            crate::models::RuleSource::Learned,
        );

        let rule_id = rule.id.clone();
        manager.request_rule_promotion(rule).await.unwrap();
        manager.approve_promotion(&rule_id, None).await.unwrap();

        assert_eq!(manager.get_promotion_history().await.len(), 1);

        manager.clear_promotion_history().await;
        assert_eq!(manager.get_promotion_history().await.len(), 0);
    }
}
