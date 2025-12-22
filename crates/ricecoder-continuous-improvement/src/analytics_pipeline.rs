//! Feature usage analytics and prioritization pipeline

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use ricecoder_monitoring::analytics::{AnalyticsEngine, FeatureAdoptionMetrics, UsageStats};
use tokio::{sync::mpsc, time};

use crate::types::*;

/// Analytics pipeline for feature usage analysis and prioritization
pub struct AnalyticsPipeline {
    config: AnalyticsPipelineConfig,
    analytics_engine: Arc<Mutex<AnalyticsEngine>>,
    shutdown_tx: Option<mpsc::Sender<()>>,
    prioritization_task: Option<tokio::task::JoinHandle<()>>,
}

impl AnalyticsPipeline {
    /// Create a new analytics pipeline
    pub fn new(config: AnalyticsPipelineConfig) -> Self {
        let analytics_config = ricecoder_monitoring::analytics::AnalyticsConfig {
            enabled: config.enabled,
            tracking_id: None,
            event_buffer_size: 1000,
            flush_interval: config.collection_interval,
        };

        Self {
            config,
            analytics_engine: Arc::new(Mutex::new(AnalyticsEngine::new(analytics_config))),
            shutdown_tx: None,
            prioritization_task: None,
        }
    }

    /// Start the analytics pipeline
    pub async fn start(&mut self) -> Result<(), ContinuousImprovementError> {
        if !self.config.enabled {
            return Ok(());
        }

        tracing::info!("Starting analytics pipeline");

        self.analytics_engine
            .lock()
            .unwrap()
            .start()
            .await
            .map_err(|e| ContinuousImprovementError::AnalyticsError(e.to_string()))?;

        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        let prioritization_interval = self.config.prioritization_interval;
        let analytics_engine = Arc::clone(&self.analytics_engine);
        let threshold = self.config.feature_adoption_threshold;

        let task = tokio::spawn(async move {
            let mut interval = time::interval(prioritization_interval.to_std().unwrap());

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if let Err(e) = Self::perform_prioritization_analysis(&analytics_engine, threshold).await {
                            tracing::error!("Analytics prioritization failed: {}", e);
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        tracing::info!("Analytics pipeline prioritization task shutting down");
                        break;
                    }
                }
            }
        });

        self.prioritization_task = Some(task);
        tracing::info!("Analytics pipeline started");
        Ok(())
    }

    /// Stop the analytics pipeline
    pub async fn stop(&mut self) -> Result<(), ContinuousImprovementError> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }

        if let Some(task) = self.prioritization_task.take() {
            let _ = task.await;
        }

        self.analytics_engine
            .lock()
            .unwrap()
            .stop()
            .await
            .map_err(|e| ContinuousImprovementError::AnalyticsError(e.to_string()))?;

        tracing::info!("Analytics pipeline stopped");
        Ok(())
    }

    /// Track feature usage
    pub fn track_feature_usage(
        &self,
        user_id: Option<String>,
        feature: &str,
        properties: HashMap<String, serde_json::Value>,
    ) {
        self.analytics_engine
            .lock()
            .unwrap()
            .track_action(user_id, feature, properties);
    }

    /// Get analytics insights
    pub async fn get_insights(&self) -> Result<AnalyticsInsights, ContinuousImprovementError> {
        let usage_stats = self.analytics_engine.lock().unwrap().get_usage_stats(None);

        // Get feature usage (simplified mapping)
        let feature_usage = usage_stats
            .events_by_type
            .iter()
            .map(|(k, v)| (k.clone(), *v as f64))
            .collect();

        // Calculate user engagement (simplified)
        let total_events = usage_stats.total_events as f64;
        let user_engagement = if total_events > 0.0 {
            (usage_stats.unique_users as f64 / total_events) * 100.0
        } else {
            0.0
        };

        // Get adoption rates (simplified - would analyze over time)
        let adoption_rates = usage_stats
            .events_by_type
            .iter()
            .map(|(k, v)| (k.clone(), (*v as f64 / total_events) * 100.0))
            .collect();

        // Get performance metrics (simplified - would integrate with performance monitoring)
        let performance_metrics = HashMap::from([
            ("response_time_avg".to_string(), 150.0),
            ("error_rate".to_string(), 2.5),
            ("uptime".to_string(), 99.9),
        ]);

        Ok(AnalyticsInsights {
            feature_usage,
            user_engagement,
            adoption_rates,
            performance_metrics,
        })
    }

    /// Get feature priorities
    pub async fn get_feature_priorities(
        &self,
    ) -> Result<Vec<FeaturePriority>, ContinuousImprovementError> {
        let usage_stats = self.analytics_engine.lock().unwrap().get_usage_stats(None);

        let mut priorities = Vec::new();

        for (feature_name, usage_count) in &usage_stats.events_by_type {
            let usage_score = *usage_count as f64;
            let feedback_score = 5.0; // Simplified - would integrate with feedback
            let issue_score = 3.0; // Simplified - would integrate with issues
            let enterprise_score = 7.0; // Simplified - would analyze enterprise usage

            let overall_score = (usage_score * 0.4)
                + (feedback_score * 0.3)
                + (issue_score * 0.2)
                + (enterprise_score * 0.1);

            let current_priority = if overall_score > 80.0 {
                Priority::Critical
            } else if overall_score > 60.0 {
                Priority::High
            } else if overall_score > 40.0 {
                Priority::Medium
            } else {
                Priority::Low
            };

            priorities.push(FeaturePriority {
                feature_name: feature_name.clone(),
                current_priority,
                usage_score,
                feedback_score,
                issue_score,
                enterprise_score,
                overall_score,
                trend: PriorityTrend::Stable, // Simplified
            });
        }

        // Sort by overall score descending
        priorities.sort_by(|a, b| b.overall_score.partial_cmp(&a.overall_score).unwrap());

        Ok(priorities)
    }

    /// Health check
    pub async fn health_check(&self) -> ComponentHealth {
        // Simple health check - in real implementation would check actual status
        ComponentHealth::Healthy
    }

    /// Perform prioritization analysis
    async fn perform_prioritization_analysis(
        analytics_engine: &Mutex<AnalyticsEngine>,
        threshold: f64,
    ) -> Result<(), ContinuousImprovementError> {
        tracing::info!("Performing analytics prioritization analysis");

        let priorities = Self::calculate_priorities(analytics_engine, threshold).await?;

        tracing::info!(
            "Prioritization analysis complete - {} features analyzed, {} high priority",
            priorities.len(),
            priorities
                .iter()
                .filter(|p| matches!(p.current_priority, Priority::High | Priority::Critical))
                .count()
        );

        Ok(())
    }

    /// Calculate feature priorities
    async fn calculate_priorities(
        analytics_engine: &Mutex<AnalyticsEngine>,
        threshold: f64,
    ) -> Result<Vec<FeaturePriority>, ContinuousImprovementError> {
        let usage_stats = analytics_engine.lock().unwrap().get_usage_stats(None);

        let mut priorities = Vec::new();

        for (feature_name, usage_count) in &usage_stats.events_by_type {
            if *usage_count as f64 >= threshold {
                let usage_score = *usage_count as f64;
                let feedback_score = 5.0; // Would integrate with feedback analytics
                let issue_score = 3.0; // Would integrate with issue tracking
                let enterprise_score = 7.0; // Would analyze enterprise usage patterns

                let overall_score = (usage_score * 0.4)
                    + (feedback_score * 0.3)
                    + (issue_score * 0.2)
                    + (enterprise_score * 0.1);

                let current_priority = if overall_score > 80.0 {
                    Priority::Critical
                } else if overall_score > 60.0 {
                    Priority::High
                } else if overall_score > 40.0 {
                    Priority::Medium
                } else {
                    Priority::Low
                };

                priorities.push(FeaturePriority {
                    feature_name: feature_name.clone(),
                    current_priority,
                    usage_score,
                    feedback_score,
                    issue_score,
                    enterprise_score,
                    overall_score,
                    trend: PriorityTrend::Stable,
                });
            }
        }

        Ok(priorities)
    }
}
