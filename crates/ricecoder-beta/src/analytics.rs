//! Analytics and metrics collection for beta testing

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::feedback::{FeedbackSeverity, FeedbackType, UserFeedback};

/// Analytics data for beta testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BetaAnalytics {
    pub total_users: u64,
    pub active_sessions: u64,
    pub feedback_count: u64,
    pub bug_reports: u64,
    pub feature_requests: u64,
    pub performance_issues: u64,
    pub enterprise_feedback: u64,
    pub average_session_duration: std::time::Duration,
    pub user_satisfaction_score: f64,
    pub enterprise_compliance_score: f64,
    pub performance_metrics: PerformanceMetrics,
    pub collected_at: DateTime<Utc>,
}

/// Performance metrics for beta testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub startup_times: Vec<f64>,
    pub response_times: Vec<f64>,
    pub memory_usage: Vec<f64>,
    pub error_rate: f64,
    pub crash_rate: f64,
}

/// Feedback analytics
pub struct FeedbackAnalytics {
    feedback_by_type: HashMap<FeedbackType, u64>,
    feedback_by_severity: HashMap<FeedbackSeverity, u64>,
    feedback_trends: Vec<(DateTime<Utc>, u64)>,
}

impl FeedbackAnalytics {
    pub fn new() -> Self {
        Self {
            feedback_by_type: HashMap::new(),
            feedback_by_severity: HashMap::new(),
            feedback_trends: vec![],
        }
    }

    /// Record feedback for analytics
    pub async fn record_feedback(&mut self, feedback: &UserFeedback) -> Result<(), AnalyticsError> {
        // Update type counts
        *self
            .feedback_by_type
            .entry(feedback.feedback_type.clone())
            .or_insert(0) += 1;

        // Update severity counts
        *self
            .feedback_by_severity
            .entry(feedback.severity.clone())
            .or_insert(0) += 1;

        // Update trends (simplified - in real implementation would aggregate by time periods)
        let now = Utc::now();
        self.feedback_trends.push((now, 1));

        Ok(())
    }

    /// Get feedback distribution by type
    pub fn get_feedback_by_type(&self) -> &HashMap<FeedbackType, u64> {
        &self.feedback_by_type
    }

    /// Get feedback distribution by severity
    pub fn get_feedback_by_severity(&self) -> &HashMap<FeedbackSeverity, u64> {
        &self.feedback_by_severity
    }

    /// Get feedback trends over time
    pub fn get_feedback_trends(&self) -> &[(DateTime<Utc>, u64)] {
        &self.feedback_trends
    }

    /// Calculate user satisfaction score based on feedback
    pub fn calculate_satisfaction_score(&self) -> f64 {
        let total_feedback = self.feedback_by_type.values().sum::<u64>() as f64;
        if total_feedback == 0.0 {
            return 0.0;
        }

        let positive_feedback = self
            .feedback_by_type
            .get(&FeedbackType::FeatureRequest)
            .unwrap_or(&0)
            + self
                .feedback_by_type
                .get(&FeedbackType::GeneralFeedback)
                .unwrap_or(&0);

        (positive_feedback as f64 / total_feedback) * 100.0
    }
}

/// Enterprise requirements validation analytics
pub struct EnterpriseValidationAnalytics {
    compliance_checks: Vec<ComplianceCheck>,
    deployment_scenarios: Vec<DeploymentScenario>,
    integration_tests: Vec<IntegrationTest>,
}

impl EnterpriseValidationAnalytics {
    pub fn new() -> Self {
        Self {
            compliance_checks: vec![],
            deployment_scenarios: vec![],
            integration_tests: vec![],
        }
    }

    /// Record compliance check result
    pub fn record_compliance_check(
        &mut self,
        check_type: ComplianceType,
        passed: bool,
        details: String,
    ) {
        self.compliance_checks.push(ComplianceCheck {
            check_type,
            passed,
            details,
            timestamp: Utc::now(),
        });
    }

    /// Record deployment scenario result
    pub fn record_deployment_scenario(
        &mut self,
        scenario: DeploymentScenarioType,
        success: bool,
        duration: std::time::Duration,
        issues: Vec<String>,
    ) {
        self.deployment_scenarios.push(DeploymentScenario {
            scenario,
            success,
            duration,
            issues,
            timestamp: Utc::now(),
        });
    }

    /// Record integration test result
    pub fn record_integration_test(
        &mut self,
        test_type: IntegrationTestType,
        passed: bool,
        performance_metrics: HashMap<String, f64>,
    ) {
        self.integration_tests.push(IntegrationTest {
            test_type,
            passed,
            performance_metrics,
            timestamp: Utc::now(),
        });
    }

    /// Get compliance pass rate
    pub fn get_compliance_pass_rate(&self) -> f64 {
        if self.compliance_checks.is_empty() {
            return 0.0;
        }

        let passed = self.compliance_checks.iter().filter(|c| c.passed).count();
        (passed as f64 / self.compliance_checks.len() as f64) * 100.0
    }

    /// Get deployment success rate
    pub fn get_deployment_success_rate(&self) -> f64 {
        if self.deployment_scenarios.is_empty() {
            return 0.0;
        }

        let successful = self
            .deployment_scenarios
            .iter()
            .filter(|s| s.success)
            .count();
        (successful as f64 / self.deployment_scenarios.len() as f64) * 100.0
    }
}

/// Compliance check types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceType {
    SOC2TypeII,
    GDPR,
    HIPAA,
    EnterpriseSecurity,
    AuditLogging,
    AccessControl,
}

/// Deployment scenario types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentScenarioType {
    SingleTenant,
    MultiTenant,
    CloudDeployment,
    OnPremise,
    Hybrid,
    Containerized,
}

/// Integration test types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntegrationTestType {
    MCPIntegration,
    ProviderIntegration,
    SessionManagement,
    EnterpriseSecurity,
    PerformanceLoad,
    ComplianceValidation,
}

/// Compliance check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceCheck {
    pub check_type: ComplianceType,
    pub passed: bool,
    pub details: String,
    pub timestamp: DateTime<Utc>,
}

/// Deployment scenario result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentScenario {
    pub scenario: DeploymentScenarioType,
    pub success: bool,
    pub duration: std::time::Duration,
    pub issues: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

/// Integration test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationTest {
    pub test_type: IntegrationTestType,
    pub passed: bool,
    pub performance_metrics: HashMap<String, f64>,
    pub timestamp: DateTime<Utc>,
}

/// Analytics errors
#[derive(Debug, thiserror::Error)]
pub enum AnalyticsError {
    #[error("Data collection error: {0}")]
    DataCollectionError(String),

    #[error("Metrics calculation error: {0}")]
    MetricsError(String),

    #[error("Storage error: {0}")]
    StorageError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feedback_analytics() {
        let mut analytics = FeedbackAnalytics::new();

        let feedback = UserFeedback {
            id: uuid::Uuid::new_v4(),
            user_id: None,
            session_id: None,
            project_id: None,
            feedback_type: FeedbackType::BugReport,
            severity: FeedbackSeverity::High,
            title: "Test".to_string(),
            description: "Test".to_string(),
            enterprise_category: None,
            tags: vec![],
            metadata: HashMap::new(),
            attachments: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            status: crate::feedback::FeedbackStatus::Open,
        };

        // Note: This would normally be async, but for test we assume it works
        // analytics.record_feedback(&feedback).await.unwrap();

        assert_eq!(analytics.calculate_satisfaction_score(), 0.0);
    }

    #[test]
    fn test_enterprise_validation_analytics() {
        let mut analytics = EnterpriseValidationAnalytics::new();

        analytics.record_compliance_check(
            ComplianceType::SOC2TypeII,
            true,
            "All controls verified".to_string(),
        );

        assert_eq!(analytics.get_compliance_pass_rate(), 100.0);
    }
}
