//! Common types for the continuous improvement pipeline

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, TimeDelta};
use std::collections::HashMap;
use ricecoder_monitoring::types::ComplianceStatus;

/// Configuration for the continuous improvement pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContinuousImprovementConfig {
    pub feedback_config: FeedbackPipelineConfig,
    pub analytics_config: AnalyticsPipelineConfig,
    pub issue_detection_config: IssueDetectionPipelineConfig,
    pub security_config: SecurityMonitoringConfig,
    pub roadmap_config: RoadmapPlanningConfig,
}

impl Default for ContinuousImprovementConfig {
    fn default() -> Self {
        Self {
            feedback_config: FeedbackPipelineConfig::default(),
            analytics_config: AnalyticsPipelineConfig::default(),
            issue_detection_config: IssueDetectionPipelineConfig::default(),
            security_config: SecurityMonitoringConfig::default(),
            roadmap_config: RoadmapPlanningConfig::default(),
        }
    }
}

/// Feedback pipeline configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackPipelineConfig {
    pub enabled: bool,
    pub collection_interval: TimeDelta,
    pub analysis_interval: TimeDelta,
    pub enterprise_focus: bool,
}

impl Default for FeedbackPipelineConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            collection_interval: TimeDelta::seconds(300), // 5 minutes
            analysis_interval: TimeDelta::seconds(3600), // 1 hour
            enterprise_focus: true,
        }
    }
}

/// Analytics pipeline configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsPipelineConfig {
    pub enabled: bool,
    pub collection_interval: TimeDelta,
    pub prioritization_interval: TimeDelta,
    pub feature_adoption_threshold: f64,
}

impl Default for AnalyticsPipelineConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            collection_interval: TimeDelta::seconds(600), // 10 minutes
            prioritization_interval: TimeDelta::seconds(7200), // 2 hours
            feature_adoption_threshold: 10.0, // 10% adoption rate
        }
    }
}

/// Issue detection pipeline configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueDetectionPipelineConfig {
    pub enabled: bool,
    pub detection_interval: TimeDelta,
    pub escalation_thresholds: EscalationThresholds,
    pub enterprise_escalation: bool,
}

impl Default for IssueDetectionPipelineConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            detection_interval: TimeDelta::seconds(180), // 3 minutes
            escalation_thresholds: EscalationThresholds::default(),
            enterprise_escalation: true,
        }
    }
}

/// Escalation thresholds for issue detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationThresholds {
    pub error_rate_threshold: f64,
    pub performance_degradation_threshold: f64,
    pub security_incident_threshold: u32,
}

impl Default for EscalationThresholds {
    fn default() -> Self {
        Self {
            error_rate_threshold: 5.0, // 5% error rate
            performance_degradation_threshold: 20.0, // 20% degradation
            security_incident_threshold: 3, // 3 security incidents
        }
    }
}

/// Security monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityMonitoringConfig {
    pub enabled: bool,
    pub monitoring_interval: TimeDelta,
    pub compliance_check_interval: TimeDelta,
    pub update_check_interval: TimeDelta,
    pub standards: Vec<String>,
}

impl Default for SecurityMonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            monitoring_interval: TimeDelta::seconds(900), // 15 minutes
            compliance_check_interval: TimeDelta::seconds(86400), // Daily
            update_check_interval: TimeDelta::seconds(3600), // Hourly
            standards: vec!["SOC2".to_string(), "GDPR".to_string(), "HIPAA".to_string()],
        }
    }
}

/// Roadmap planning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadmapPlanningConfig {
    pub enabled: bool,
    pub planning_interval: TimeDelta,
    pub prioritization_weights: PrioritizationWeights,
    pub enterprise_focus: bool,
}

impl Default for RoadmapPlanningConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            planning_interval: TimeDelta::seconds(604800), // Weekly
            prioritization_weights: PrioritizationWeights::default(),
            enterprise_focus: true,
        }
    }
}

/// Weights for feature prioritization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrioritizationWeights {
    pub user_feedback_weight: f64,
    pub usage_analytics_weight: f64,
    pub issue_impact_weight: f64,
    pub security_importance_weight: f64,
    pub enterprise_value_weight: f64,
}

impl Default for PrioritizationWeights {
    fn default() -> Self {
        Self {
            user_feedback_weight: 0.25,
            usage_analytics_weight: 0.20,
            issue_impact_weight: 0.25,
            security_importance_weight: 0.15,
            enterprise_value_weight: 0.15,
        }
    }
}

/// Improvement recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementRecommendations {
    pub recommendations: Vec<ImprovementRecommendation>,
    pub priorities: Vec<FeaturePriority>,
    pub roadmap_items: Vec<RoadmapItem>,
    pub generated_at: DateTime<Utc>,
}

/// Individual improvement recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementRecommendation {
    pub id: String,
    pub title: String,
    pub description: String,
    pub category: RecommendationCategory,
    pub priority: Priority,
    pub effort_estimate: EffortLevel,
    pub impact_score: f64,
    pub rationale: String,
    pub supporting_data: HashMap<String, serde_json::Value>,
}

/// Recommendation categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationCategory {
    FeatureEnhancement,
    BugFix,
    PerformanceImprovement,
    SecurityEnhancement,
    UserExperience,
    EnterpriseIntegration,
    ComplianceImprovement,
}

/// Priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

/// Effort levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffortLevel {
    Small,
    Medium,
    Large,
    ExtraLarge,
    High,
}

/// Feature priority analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturePriority {
    pub feature_name: String,
    pub current_priority: Priority,
    pub usage_score: f64,
    pub feedback_score: f64,
    pub issue_score: f64,
    pub enterprise_score: f64,
    pub overall_score: f64,
    pub trend: PriorityTrend,
}

/// Priority trend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PriorityTrend {
    Increasing,
    Stable,
    Decreasing,
}

/// Roadmap item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadmapItem {
    pub id: String,
    pub title: String,
    pub description: String,
    pub category: RoadmapCategory,
    pub priority: Priority,
    pub estimated_completion: DateTime<Utc>,
    pub dependencies: Vec<String>,
    pub stakeholders: Vec<String>,
}

/// Roadmap categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoadmapCategory {
    Feature,
    Enhancement,
    Security,
    Compliance,
    Infrastructure,
    Enterprise,
}

/// Pipeline insights from different sources
#[derive(Debug, Clone)]
pub struct FeedbackInsights {
    pub satisfaction_score: f64,
    pub top_pain_points: Vec<String>,
    pub feature_requests: Vec<String>,
    pub enterprise_feedback: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AnalyticsInsights {
    pub feature_usage: HashMap<String, f64>,
    pub user_engagement: f64,
    pub adoption_rates: HashMap<String, f64>,
    pub performance_metrics: HashMap<String, f64>,
}

#[derive(Debug, Clone)]
pub struct IssueInsights {
    pub critical_issues: Vec<String>,
    pub error_rates: HashMap<String, f64>,
    pub performance_issues: Vec<String>,
    pub security_incidents: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SecurityInsights {
    pub compliance_status: HashMap<String, ComplianceStatus>,
    pub security_vulnerabilities: Vec<String>,
    pub update_status: HashMap<String, String>,
    pub audit_findings: Vec<String>,
}

/// Component health status
#[derive(Debug, Clone)]
pub enum ComponentHealth {
    Healthy,
    Degraded(String),
    Unhealthy(String),
}

/// Continuous improvement errors
#[derive(Debug, thiserror::Error)]
pub enum ContinuousImprovementError {
    #[error("Feedback pipeline error: {0}")]
    FeedbackError(String),

    #[error("Analytics pipeline error: {0}")]
    AnalyticsError(String),

    #[error("Issue detection pipeline error: {0}")]
    IssueDetectionError(String),

    #[error("Security monitoring pipeline error: {0}")]
    SecurityMonitoringError(String),

    #[error("Roadmap planning error: {0}")]
    RoadmapPlanningError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}