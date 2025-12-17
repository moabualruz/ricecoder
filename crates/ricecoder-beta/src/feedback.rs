//! User feedback collection and management for beta testing

use crate::analytics::FeedbackAnalytics;
use chrono::{DateTime, Utc};
use ricecoder_domain::entities::{Project, Session, User};
use ricecoder_domain::value_objects::{ProjectId, SessionId};
use ricecoder_security::audit::AuditEvent;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Types of feedback that can be collected
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum FeedbackType {
    BugReport,
    FeatureRequest,
    PerformanceIssue,
    UsabilityIssue,
    EnterpriseIntegration,
    ComplianceConcern,
    GeneralFeedback,
}

/// Severity levels for feedback
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum FeedbackSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Enterprise-specific feedback categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnterpriseCategory {
    AuditLogging,
    AccessControl,
    ComplianceReporting,
    DataEncryption,
    MultiTenantSupport,
    DeploymentIntegration,
    PerformanceRequirements,
    SecurityFeatures,
}

/// User feedback structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFeedback {
    pub id: Uuid,
    pub user_id: Option<String>,
    pub session_id: Option<SessionId>,
    pub project_id: Option<ProjectId>,
    pub feedback_type: FeedbackType,
    pub severity: FeedbackSeverity,
    pub title: String,
    pub description: String,
    pub enterprise_category: Option<EnterpriseCategory>,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub attachments: Vec<FeedbackAttachment>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: FeedbackStatus,
}

/// Status of feedback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeedbackStatus {
    Open,
    InReview,
    Addressed,
    Closed,
    Rejected,
}

/// Feedback attachments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackAttachment {
    pub id: Uuid,
    pub filename: String,
    pub content_type: String,
    pub size: usize,
    pub data: Vec<u8>,
}

/// Feedback collection service
pub struct FeedbackCollector {
    analytics: FeedbackAnalytics,
}

impl FeedbackCollector {
    pub fn new() -> Self {
        Self {
            analytics: FeedbackAnalytics::new(),
        }
    }

    /// Collect feedback from a user
    pub async fn collect_feedback(
        &mut self,
        user: Option<&User>,
        session: Option<&Session>,
        project: Option<&Project>,
        feedback_type: FeedbackType,
        severity: FeedbackSeverity,
        title: String,
        description: String,
        enterprise_category: Option<EnterpriseCategory>,
        tags: Vec<String>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Result<UserFeedback, FeedbackError> {
        let feedback = UserFeedback {
            id: Uuid::new_v4(),
            user_id: user.map(|u| u.id.clone()),
            session_id: session.map(|s| s.id),
            project_id: project.map(|p| p.id),
            feedback_type,
            severity,
            title,
            description,
            enterprise_category,
            tags,
            metadata,
            attachments: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            status: FeedbackStatus::Open,
        };

        // Record analytics
        self.analytics.record_feedback(&feedback).await?;

        // Create audit event
        if let Some(user) = user {
            let audit_event = AuditEvent::new(
                "feedback_submitted".to_string(),
                Some(user.id.clone()),
                serde_json::to_value(&feedback).unwrap_or_default(),
            );
            // In real implementation, this would be sent to audit logging service
        }

        Ok(feedback)
    }

    /// Add attachment to feedback
    pub fn add_attachment(
        &mut self,
        feedback: &mut UserFeedback,
        filename: String,
        content_type: String,
        data: Vec<u8>,
    ) -> Result<(), FeedbackError> {
        let attachment = FeedbackAttachment {
            id: Uuid::new_v4(),
            filename,
            content_type,
            size: data.len(),
            data,
        };

        feedback.attachments.push(attachment);
        feedback.updated_at = Utc::now();

        Ok(())
    }

    /// Update feedback status
    pub fn update_status(
        &mut self,
        feedback: &mut UserFeedback,
        status: FeedbackStatus,
    ) -> Result<(), FeedbackError> {
        feedback.status = status;
        feedback.updated_at = Utc::now();

        Ok(())
    }

    /// Get feedback analytics
    pub fn get_analytics(&self) -> &FeedbackAnalytics {
        &self.analytics
    }
}

/// Feedback collection errors
#[derive(Debug, thiserror::Error)]
pub enum FeedbackError {
    #[error("Invalid feedback data: {0}")]
    InvalidData(String),

    #[error("Analytics error: {0}")]
    AnalyticsError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Attachment too large: {0} bytes")]
    AttachmentTooLarge(usize),
}

impl From<crate::analytics::AnalyticsError> for FeedbackError {
    fn from(err: crate::analytics::AnalyticsError) -> Self {
        FeedbackError::AnalyticsError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_collect_feedback() {
        let mut collector = FeedbackCollector::new();

        let feedback = collector
            .collect_feedback(
                None,
                None,
                None,
                FeedbackType::GeneralFeedback,
                FeedbackSeverity::Medium,
                "Test feedback".to_string(),
                "This is a test feedback".to_string(),
                None,
                vec!["test".to_string()],
                HashMap::new(),
            )
            .await
            .unwrap();

        assert_eq!(feedback.title, "Test feedback");
        assert_eq!(feedback.severity, FeedbackSeverity::Medium);
        assert_eq!(feedback.status, FeedbackStatus::Open);
    }
}