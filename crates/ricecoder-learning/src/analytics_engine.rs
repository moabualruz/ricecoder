/// Analytics engine for tracking and analyzing rule metrics
use crate::error::{LearningError, Result};
use crate::models::{Rule, RuleScope};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Metrics for a single rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleMetrics {
    /// Rule ID
    pub rule_id: String,
    /// Number of times the rule has been applied
    pub usage_count: u64,
    /// Number of successful applications
    pub success_count: u64,
    /// Number of failed applications
    pub failure_count: u64,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f32,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
    /// When the rule was first applied
    pub first_applied: Option<DateTime<Utc>>,
    /// When the rule was last applied
    pub last_applied: Option<DateTime<Utc>>,
    /// Average time to apply the rule (in milliseconds)
    pub avg_application_time_ms: f64,
}

impl RuleMetrics {
    /// Create new metrics for a rule
    pub fn new(rule_id: String) -> Self {
        Self {
            rule_id,
            usage_count: 0,
            success_count: 0,
            failure_count: 0,
            success_rate: 0.0,
            confidence: 0.5,
            first_applied: None,
            last_applied: None,
            avg_application_time_ms: 0.0,
        }
    }

    /// Record a successful application
    pub fn record_success(&mut self, application_time_ms: f64) {
        self.usage_count += 1;
        self.success_count += 1;
        self.update_success_rate();
        self.update_application_time(application_time_ms);
        self.update_last_applied();
    }

    /// Record a failed application
    pub fn record_failure(&mut self, application_time_ms: f64) {
        self.usage_count += 1;
        self.failure_count += 1;
        self.update_success_rate();
        self.update_application_time(application_time_ms);
        self.update_last_applied();
    }

    /// Update success rate based on current counts
    fn update_success_rate(&mut self) {
        if self.usage_count > 0 {
            self.success_rate = self.success_count as f32 / self.usage_count as f32;
        }
    }

    /// Update average application time
    fn update_application_time(&mut self, new_time_ms: f64) {
        if self.usage_count == 1 {
            self.avg_application_time_ms = new_time_ms;
        } else {
            let total_time = self.avg_application_time_ms * (self.usage_count - 1) as f64;
            self.avg_application_time_ms = (total_time + new_time_ms) / self.usage_count as f64;
        }
    }

    /// Update last applied timestamp
    fn update_last_applied(&mut self) {
        self.last_applied = Some(Utc::now());
        if self.first_applied.is_none() {
            self.first_applied = Some(Utc::now());
        }
    }
}

/// Analytics insights about rule usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsInsights {
    /// Total number of rules tracked
    pub total_rules: usize,
    /// Total number of rule applications
    pub total_applications: u64,
    /// Average success rate across all rules
    pub avg_success_rate: f32,
    /// Average confidence score across all rules
    pub avg_confidence: f32,
    /// Most frequently used rule ID
    pub most_used_rule: Option<String>,
    /// Least frequently used rule ID
    pub least_used_rule: Option<String>,
    /// Rules with highest success rate
    pub top_performing_rules: Vec<String>,
    /// Rules with lowest success rate
    pub bottom_performing_rules: Vec<String>,
}

/// Analytics engine for tracking rule metrics and generating insights
pub struct AnalyticsEngine {
    /// Metrics for each rule
    metrics: Arc<RwLock<HashMap<String, RuleMetrics>>>,
}

impl AnalyticsEngine {
    /// Create a new analytics engine
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record a rule application
    pub async fn record_application(
        &self,
        rule_id: String,
        success: bool,
        application_time_ms: f64,
    ) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        let rule_metrics = metrics
            .entry(rule_id.clone())
            .or_insert_with(|| RuleMetrics::new(rule_id));

        if success {
            rule_metrics.record_success(application_time_ms);
        } else {
            rule_metrics.record_failure(application_time_ms);
        }

        Ok(())
    }

    /// Get metrics for a specific rule
    pub async fn get_rule_metrics(&self, rule_id: &str) -> Result<Option<RuleMetrics>> {
        let metrics = self.metrics.read().await;
        Ok(metrics.get(rule_id).cloned())
    }

    /// Get all metrics
    pub async fn get_all_metrics(&self) -> Result<Vec<RuleMetrics>> {
        let metrics = self.metrics.read().await;
        Ok(metrics.values().cloned().collect())
    }

    /// Update rule confidence based on validation results
    pub async fn update_confidence(&self, rule_id: &str, new_confidence: f32) -> Result<()> {
        if !(0.0..=1.0).contains(&new_confidence) {
            return Err(LearningError::AnalyticsError(
                "Confidence must be between 0.0 and 1.0".to_string(),
            ));
        }

        let mut metrics = self.metrics.write().await;
        if let Some(rule_metrics) = metrics.get_mut(rule_id) {
            rule_metrics.confidence = new_confidence;
            Ok(())
        } else {
            Err(LearningError::AnalyticsError(format!(
                "Rule {} not found in metrics",
                rule_id
            )))
        }
    }

    /// Generate analytics insights
    pub async fn generate_insights(&self) -> Result<AnalyticsInsights> {
        let metrics = self.metrics.read().await;

        if metrics.is_empty() {
            return Ok(AnalyticsInsights {
                total_rules: 0,
                total_applications: 0,
                avg_success_rate: 0.0,
                avg_confidence: 0.0,
                most_used_rule: None,
                least_used_rule: None,
                top_performing_rules: Vec::new(),
                bottom_performing_rules: Vec::new(),
            });
        }

        let total_rules = metrics.len();
        let total_applications: u64 = metrics.values().map(|m| m.usage_count).sum();
        let avg_success_rate: f32 = metrics.values().map(|m| m.success_rate).sum::<f32>()
            / total_rules as f32;
        let avg_confidence: f32 =
            metrics.values().map(|m| m.confidence).sum::<f32>() / total_rules as f32;

        // Find most and least used rules
        let most_used_rule = metrics
            .values()
            .max_by_key(|m| m.usage_count)
            .map(|m| m.rule_id.clone());
        let least_used_rule = metrics
            .values()
            .filter(|m| m.usage_count > 0)
            .min_by_key(|m| m.usage_count)
            .map(|m| m.rule_id.clone());

        // Find top and bottom performing rules
        let mut sorted_by_success: Vec<_> = metrics.values().collect();
        sorted_by_success.sort_by(|a, b| b.success_rate.partial_cmp(&a.success_rate).unwrap());

        let top_performing_rules: Vec<String> = sorted_by_success
            .iter()
            .take(5)
            .map(|m| m.rule_id.clone())
            .collect();

        let bottom_performing_rules: Vec<String> = sorted_by_success
            .iter()
            .rev()
            .take(5)
            .map(|m| m.rule_id.clone())
            .collect();

        Ok(AnalyticsInsights {
            total_rules,
            total_applications,
            avg_success_rate,
            avg_confidence,
            most_used_rule,
            least_used_rule,
            top_performing_rules,
            bottom_performing_rules,
        })
    }

    /// Clear all metrics
    pub async fn clear_metrics(&self) -> Result<()> {
        self.metrics.write().await.clear();
        Ok(())
    }

    /// Get metrics for rules in a specific scope
    pub async fn get_metrics_by_scope(
        &self,
        rules: &[Rule],
        scope: RuleScope,
    ) -> Result<Vec<RuleMetrics>> {
        let metrics = self.metrics.read().await;
        let scope_metrics: Vec<RuleMetrics> = rules
            .iter()
            .filter(|r| r.scope == scope)
            .filter_map(|r| metrics.get(&r.id).cloned())
            .collect();

        Ok(scope_metrics)
    }
}

impl Default for AnalyticsEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_metrics_creation() {
        let metrics = RuleMetrics::new("rule_1".to_string());
        assert_eq!(metrics.rule_id, "rule_1");
        assert_eq!(metrics.usage_count, 0);
        assert_eq!(metrics.success_count, 0);
        assert_eq!(metrics.failure_count, 0);
        assert_eq!(metrics.success_rate, 0.0);
        assert_eq!(metrics.confidence, 0.5);
    }

    #[test]
    fn test_rule_metrics_record_success() {
        let mut metrics = RuleMetrics::new("rule_1".to_string());
        metrics.record_success(10.0);

        assert_eq!(metrics.usage_count, 1);
        assert_eq!(metrics.success_count, 1);
        assert_eq!(metrics.failure_count, 0);
        assert_eq!(metrics.success_rate, 1.0);
        assert_eq!(metrics.avg_application_time_ms, 10.0);
        assert!(metrics.first_applied.is_some());
        assert!(metrics.last_applied.is_some());
    }

    #[test]
    fn test_rule_metrics_record_failure() {
        let mut metrics = RuleMetrics::new("rule_1".to_string());
        metrics.record_failure(5.0);

        assert_eq!(metrics.usage_count, 1);
        assert_eq!(metrics.success_count, 0);
        assert_eq!(metrics.failure_count, 1);
        assert_eq!(metrics.success_rate, 0.0);
    }

    #[test]
    fn test_rule_metrics_mixed_results() {
        let mut metrics = RuleMetrics::new("rule_1".to_string());
        metrics.record_success(10.0);
        metrics.record_success(12.0);
        metrics.record_failure(8.0);

        assert_eq!(metrics.usage_count, 3);
        assert_eq!(metrics.success_count, 2);
        assert_eq!(metrics.failure_count, 1);
        assert!((metrics.success_rate - 2.0 / 3.0).abs() < 0.01);
    }

    #[test]
    fn test_rule_metrics_average_time() {
        let mut metrics = RuleMetrics::new("rule_1".to_string());
        metrics.record_success(10.0);
        metrics.record_success(20.0);
        metrics.record_success(30.0);

        assert!((metrics.avg_application_time_ms - 20.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_analytics_engine_record_application() {
        let engine = AnalyticsEngine::new();
        engine
            .record_application("rule_1".to_string(), true, 10.0)
            .await
            .unwrap();

        let metrics = engine.get_rule_metrics("rule_1").await.unwrap();
        assert!(metrics.is_some());
        let metrics = metrics.unwrap();
        assert_eq!(metrics.usage_count, 1);
        assert_eq!(metrics.success_count, 1);
    }

    #[tokio::test]
    async fn test_analytics_engine_get_all_metrics() {
        let engine = AnalyticsEngine::new();
        engine
            .record_application("rule_1".to_string(), true, 10.0)
            .await
            .unwrap();
        engine
            .record_application("rule_2".to_string(), false, 5.0)
            .await
            .unwrap();

        let all_metrics = engine.get_all_metrics().await.unwrap();
        assert_eq!(all_metrics.len(), 2);
    }

    #[tokio::test]
    async fn test_analytics_engine_update_confidence() {
        let engine = AnalyticsEngine::new();
        engine
            .record_application("rule_1".to_string(), true, 10.0)
            .await
            .unwrap();

        engine
            .update_confidence("rule_1", 0.8)
            .await
            .unwrap();

        let metrics = engine.get_rule_metrics("rule_1").await.unwrap().unwrap();
        assert_eq!(metrics.confidence, 0.8);
    }

    #[tokio::test]
    async fn test_analytics_engine_invalid_confidence() {
        let engine = AnalyticsEngine::new();
        engine
            .record_application("rule_1".to_string(), true, 10.0)
            .await
            .unwrap();

        let result = engine.update_confidence("rule_1", 1.5).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_analytics_engine_generate_insights() {
        let engine = AnalyticsEngine::new();
        engine
            .record_application("rule_1".to_string(), true, 10.0)
            .await
            .unwrap();
        engine
            .record_application("rule_1".to_string(), true, 12.0)
            .await
            .unwrap();
        engine
            .record_application("rule_2".to_string(), false, 5.0)
            .await
            .unwrap();

        let insights = engine.generate_insights().await.unwrap();
        assert_eq!(insights.total_rules, 2);
        assert_eq!(insights.total_applications, 3);
        assert!(insights.most_used_rule.is_some());
    }

    #[tokio::test]
    async fn test_analytics_engine_clear_metrics() {
        let engine = AnalyticsEngine::new();
        engine
            .record_application("rule_1".to_string(), true, 10.0)
            .await
            .unwrap();

        engine.clear_metrics().await.unwrap();

        let all_metrics = engine.get_all_metrics().await.unwrap();
        assert_eq!(all_metrics.len(), 0);
    }

    #[tokio::test]
    async fn test_analytics_engine_empty_insights() {
        let engine = AnalyticsEngine::new();
        let insights = engine.generate_insights().await.unwrap();

        assert_eq!(insights.total_rules, 0);
        assert_eq!(insights.total_applications, 0);
        assert_eq!(insights.avg_success_rate, 0.0);
        assert_eq!(insights.avg_confidence, 0.0);
        assert!(insights.most_used_rule.is_none());
    }
}
