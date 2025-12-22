//! User feedback collection and analysis pipeline

use std::sync::{Arc, Mutex};

use ricecoder_beta::{
    analytics::FeedbackAnalytics,
    feedback::{FeedbackCollector, FeedbackSeverity, FeedbackType},
};
use tokio::{sync::mpsc, time};

use crate::types::*;

/// Feedback pipeline for collecting and analyzing user feedback
pub struct FeedbackPipeline {
    config: FeedbackPipelineConfig,
    collector: Arc<Mutex<FeedbackCollector>>,
    analytics: Arc<Mutex<FeedbackAnalytics>>,
    shutdown_tx: Option<mpsc::Sender<()>>,
    analysis_task: Option<tokio::task::JoinHandle<()>>,
}

impl FeedbackPipeline {
    /// Create a new feedback pipeline
    pub fn new(config: FeedbackPipelineConfig) -> Self {
        Self {
            config,
            collector: Arc::new(Mutex::new(FeedbackCollector::new())),
            analytics: Arc::new(Mutex::new(FeedbackAnalytics::new())),
            shutdown_tx: None,
            analysis_task: None,
        }
    }

    /// Start the feedback pipeline
    pub async fn start(&mut self) -> Result<(), ContinuousImprovementError> {
        if !self.config.enabled {
            return Ok(());
        }

        tracing::info!("Starting feedback pipeline");

        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        let analysis_interval = self.config.analysis_interval;
        let analytics = Arc::clone(&self.analytics);

        let task = tokio::spawn(async move {
            let mut interval = time::interval(analysis_interval.to_std().unwrap());

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if let Err(e) = Self::perform_feedback_analysis(&analytics).await {
                            tracing::error!("Feedback analysis failed: {}", e);
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        tracing::info!("Feedback pipeline analysis task shutting down");
                        break;
                    }
                }
            }
        });

        self.analysis_task = Some(task);
        tracing::info!("Feedback pipeline started");
        Ok(())
    }

    /// Stop the feedback pipeline
    pub async fn stop(&mut self) -> Result<(), ContinuousImprovementError> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }

        if let Some(task) = self.analysis_task.take() {
            let _ = task.await;
        }

        tracing::info!("Feedback pipeline stopped");
        Ok(())
    }

    /// Collect feedback
    pub async fn collect_feedback(
        &self,
        feedback_type: FeedbackType,
        severity: FeedbackSeverity,
        title: String,
        description: String,
        user_id: Option<String>,
        enterprise_category: Option<ricecoder_beta::feedback::EnterpriseCategory>,
        tags: Vec<String>,
        metadata: std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<(), ContinuousImprovementError> {
        // In a real implementation, this would get user/session/project context
        let feedback = self
            .collector
            .lock()
            .unwrap()
            .collect_feedback(
                None, // user
                None, // session
                None, // project
                feedback_type,
                severity,
                title,
                description,
                enterprise_category,
                tags,
                metadata,
            )
            .await
            .map_err(|e| ContinuousImprovementError::FeedbackError(e.to_string()))?;

        // Record in analytics
        self.analytics
            .lock()
            .unwrap()
            .record_feedback(&feedback)
            .await
            .map_err(|e| ContinuousImprovementError::FeedbackError(e.to_string()))?;

        tracing::info!("Feedback collected: {}", feedback.title);
        Ok(())
    }

    /// Get feedback insights
    pub async fn get_insights(&self) -> Result<FeedbackInsights, ContinuousImprovementError> {
        let satisfaction_score = self
            .analytics
            .lock()
            .unwrap()
            .calculate_satisfaction_score();

        // Get top pain points (simplified - would analyze feedback content)
        let top_pain_points = vec![
            "Performance issues".to_string(),
            "UI complexity".to_string(),
            "Missing enterprise features".to_string(),
        ];

        // Get feature requests (simplified)
        let feature_requests = vec![
            "Advanced session management".to_string(),
            "Better MCP integration".to_string(),
            "Enterprise compliance tools".to_string(),
        ];

        // Get enterprise feedback (simplified)
        let enterprise_feedback = vec![
            "Need better audit logging".to_string(),
            "Compliance reporting improvements".to_string(),
            "Multi-tenant support".to_string(),
        ];

        Ok(FeedbackInsights {
            satisfaction_score,
            top_pain_points,
            feature_requests,
            enterprise_feedback,
        })
    }

    /// Health check
    pub async fn health_check(&self) -> ComponentHealth {
        // Simple health check - in real implementation would check actual status
        ComponentHealth::Healthy
    }

    /// Perform feedback analysis
    async fn perform_feedback_analysis(
        analytics: &Mutex<FeedbackAnalytics>,
    ) -> Result<(), ContinuousImprovementError> {
        tracing::info!("Performing feedback analysis");

        let analytics = analytics.lock().unwrap();
        let satisfaction = analytics.calculate_satisfaction_score();
        let feedback_by_type = analytics.get_feedback_by_type();
        let feedback_by_severity = analytics.get_feedback_by_severity();

        tracing::info!(
            "Feedback analysis complete - Satisfaction: {:.1}%, Types: {}, Severities: {}",
            satisfaction,
            feedback_by_type.len(),
            feedback_by_severity.len()
        );

        Ok(())
    }
}
