//! Continuous improvement pipeline for RiceCoder
//!
//! This crate orchestrates user feedback collection, feature usage analytics,
//! automated issue detection, and continuous security monitoring to drive
//! product improvement and roadmap planning.

pub mod analytics_pipeline;
pub mod config;
pub mod feedback_pipeline;
pub mod issue_detection_pipeline;
pub mod roadmap_planning;
pub mod security_monitoring_pipeline;
pub mod types;

pub use analytics_pipeline::AnalyticsPipeline;
pub use config::*;
pub use feedback_pipeline::FeedbackPipeline;
pub use issue_detection_pipeline::IssueDetectionPipeline;
pub use roadmap_planning::RoadmapPlanner;
pub use security_monitoring_pipeline::SecurityMonitoringPipeline;
pub use types::*;

/// Main continuous improvement orchestrator
pub struct ContinuousImprovementPipeline {
    feedback_pipeline: FeedbackPipeline,
    analytics_pipeline: AnalyticsPipeline,
    issue_detection_pipeline: IssueDetectionPipeline,
    security_monitoring_pipeline: SecurityMonitoringPipeline,
    roadmap_planner: RoadmapPlanner,
    config: ContinuousImprovementConfig,
}

impl ContinuousImprovementPipeline {
    /// Create a new continuous improvement pipeline
    pub fn new(config: ContinuousImprovementConfig) -> Self {
        Self {
            feedback_pipeline: FeedbackPipeline::new(config.feedback_config.clone()),
            analytics_pipeline: AnalyticsPipeline::new(config.analytics_config.clone()),
            issue_detection_pipeline: IssueDetectionPipeline::new(
                config.issue_detection_config.clone(),
            ),
            security_monitoring_pipeline: SecurityMonitoringPipeline::new(
                config.security_config.clone(),
            ),
            roadmap_planner: RoadmapPlanner::new(config.roadmap_config.clone()),
            config: config.clone(),
        }
    }

    /// Start the continuous improvement pipeline
    pub async fn start(&mut self) -> Result<(), ContinuousImprovementError> {
        tracing::info!("Starting continuous improvement pipeline");

        // Start all pipelines
        self.feedback_pipeline.start().await?;
        self.analytics_pipeline.start().await?;
        self.issue_detection_pipeline.start().await?;
        self.security_monitoring_pipeline.start().await?;
        self.roadmap_planner.start().await?;

        tracing::info!("Continuous improvement pipeline started successfully");
        Ok(())
    }

    /// Stop the continuous improvement pipeline
    pub async fn stop(&mut self) -> Result<(), ContinuousImprovementError> {
        tracing::info!("Stopping continuous improvement pipeline");

        // Stop all pipelines in reverse order
        self.roadmap_planner.stop().await?;
        self.security_monitoring_pipeline.stop().await?;
        self.issue_detection_pipeline.stop().await?;
        self.analytics_pipeline.stop().await?;
        self.feedback_pipeline.stop().await?;

        tracing::info!("Continuous improvement pipeline stopped");
        Ok(())
    }

    /// Generate improvement recommendations
    pub async fn generate_recommendations(
        &self,
    ) -> Result<ImprovementRecommendations, ContinuousImprovementError> {
        // Collect data from all pipelines
        let feedback_insights = self.feedback_pipeline.get_insights().await?;
        let analytics_insights = self.analytics_pipeline.get_insights().await?;
        let issue_insights = self.issue_detection_pipeline.get_insights().await?;
        let security_insights = self.security_monitoring_pipeline.get_insights().await?;

        // Generate roadmap recommendations
        self.roadmap_planner
            .generate_recommendations(
                &feedback_insights,
                &analytics_insights,
                &issue_insights,
                &security_insights,
            )
            .await
    }

    /// Get pipeline health status
    pub async fn health_check(&self) -> PipelineHealth {
        PipelineHealth {
            feedback_pipeline: self.feedback_pipeline.health_check().await,
            analytics_pipeline: self.analytics_pipeline.health_check().await,
            issue_detection_pipeline: self.issue_detection_pipeline.health_check().await,
            security_monitoring_pipeline: self.security_monitoring_pipeline.health_check().await,
            roadmap_planner: self.roadmap_planner.health_check().await,
        }
    }
}

/// Pipeline health status
#[derive(Debug, Clone)]
pub struct PipelineHealth {
    pub feedback_pipeline: ComponentHealth,
    pub analytics_pipeline: ComponentHealth,
    pub issue_detection_pipeline: ComponentHealth,
    pub security_monitoring_pipeline: ComponentHealth,
    pub roadmap_planner: ComponentHealth,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pipeline_creation() {
        let config = ContinuousImprovementConfig::default();
        let pipeline = ContinuousImprovementPipeline::new(config);

        // Test that pipeline can be created
        assert!(true);
    }
}
