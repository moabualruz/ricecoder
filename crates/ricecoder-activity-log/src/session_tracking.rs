//! Session activity tracking and monitoring

use crate::error::{ActivityLogError, ActivityLogResult};
use crate::events::{ActivityEvent, EventCategory, LogLevel};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Session activity record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionActivity {
    /// Session ID
    pub session_id: String,
    /// User ID associated with the session
    pub user_id: String,
    /// Session start time
    pub started_at: DateTime<Utc>,
    /// Last activity timestamp
    pub last_activity: DateTime<Utc>,
    /// Total number of events in this session
    pub event_count: u64,
    /// Events by category
    pub events_by_category: HashMap<EventCategory, u64>,
    /// Events by level
    pub events_by_level: HashMap<LogLevel, u64>,
    /// Most active resource
    pub top_resource: Option<String>,
    /// Session duration in seconds
    pub duration_seconds: u64,
    /// Average events per minute
    pub events_per_minute: f64,
    /// Risk score for this session
    pub risk_score: u8,
}

/// Session tracker for monitoring user sessions
pub struct SessionTracker {
    activities: RwLock<HashMap<String, SessionActivity>>,
    max_sessions: usize,
}

impl SessionTracker {
    /// Create a new session tracker
    pub fn new(max_sessions: usize) -> Self {
        Self {
            activities: RwLock::new(HashMap::new()),
            max_sessions,
        }
    }

    /// Start tracking a new session
    pub async fn start_session(
        &self,
        session_id: String,
        user_id: String,
    ) -> ActivityLogResult<()> {
        let mut activities = self.activities.write().await;

        // Check session limit
        if activities.len() >= self.max_sessions {
            // Remove oldest inactive session
            if let Some(oldest_session) = activities
                .iter()
                .min_by_key(|(_, activity)| activity.last_activity)
                .map(|(id, _)| id.clone())
            {
                activities.remove(&oldest_session);
            }
        }

        let activity = SessionActivity {
            session_id: session_id.clone(),
            user_id,
            started_at: Utc::now(),
            last_activity: Utc::now(),
            event_count: 0,
            events_by_category: HashMap::new(),
            events_by_level: HashMap::new(),
            top_resource: None,
            duration_seconds: 0,
            events_per_minute: 0.0,
            risk_score: 0,
        };

        activities.insert(session_id, activity);
        Ok(())
    }

    /// End tracking a session
    pub async fn end_session(
        &self,
        session_id: &str,
    ) -> ActivityLogResult<Option<SessionActivity>> {
        let mut activities = self.activities.write().await;
        Ok(activities.remove(session_id))
    }

    /// Record activity for a session
    pub async fn record_activity(
        &self,
        session_id: &str,
        event: &ActivityEvent,
    ) -> ActivityLogResult<()> {
        let mut activities = self.activities.write().await;

        if let Some(activity) = activities.get_mut(session_id) {
            activity.last_activity = event.timestamp;
            activity.event_count += 1;

            // Update category counts
            *activity
                .events_by_category
                .entry(event.category.clone())
                .or_insert(0) += 1;

            // Update level counts
            *activity.events_by_level.entry(event.level).or_insert(0) += 1;

            // Update top resource
            Self::update_top_resource(activity, &event.resource);

            // Update duration and rate
            activity.duration_seconds = (Utc::now() - activity.started_at).num_seconds() as u64;
            if activity.duration_seconds > 0 {
                activity.events_per_minute =
                    (activity.event_count as f64 * 60.0) / activity.duration_seconds as f64;
            }

            // Update risk score
            activity.risk_score = Self::calculate_session_risk(activity);
        }

        Ok(())
    }

    /// Get session activity
    pub async fn get_session_activity(&self, session_id: &str) -> Option<SessionActivity> {
        self.activities.read().await.get(session_id).cloned()
    }

    /// Get all active sessions
    pub async fn get_active_sessions(&self) -> Vec<SessionActivity> {
        self.activities.read().await.values().cloned().collect()
    }

    /// Get sessions by user
    pub async fn get_sessions_by_user(&self, user_id: &str) -> Vec<SessionActivity> {
        self.activities
            .read()
            .await
            .values()
            .filter(|activity| activity.user_id == user_id)
            .cloned()
            .collect()
    }

    /// Get session statistics
    pub async fn get_session_stats(&self) -> SessionStats {
        let activities = self.activities.read().await;

        let total_sessions = activities.len();
        let active_sessions = activities
            .values()
            .filter(|activity| {
                // Consider active if activity in last 30 minutes
                let thirty_minutes_ago = Utc::now() - Duration::minutes(30);
                activity.last_activity > thirty_minutes_ago
            })
            .count();

        let total_events: u64 = activities.values().map(|a| a.event_count).sum();
        let avg_events_per_session = if total_sessions > 0 {
            total_events as f64 / total_sessions as f64
        } else {
            0.0
        };

        let high_risk_sessions = activities
            .values()
            .filter(|activity| activity.risk_score > 70)
            .count();

        SessionStats {
            total_sessions,
            active_sessions,
            total_events,
            avg_events_per_session,
            high_risk_sessions,
        }
    }

    /// Clean up inactive sessions
    pub async fn cleanup_inactive_sessions(
        &self,
        max_age_minutes: i64,
    ) -> ActivityLogResult<usize> {
        let mut activities = self.activities.write().await;
        let cutoff_time = Utc::now() - Duration::minutes(max_age_minutes);

        let initial_count = activities.len();
        activities.retain(|_, activity| activity.last_activity > cutoff_time);

        Ok(initial_count - activities.len())
    }

    /// Update the top resource for a session
    fn update_top_resource(activity: &mut SessionActivity, resource: &str) {
        let mut resource_counts: HashMap<String, u64> = HashMap::new();

        // This is a simplified implementation - in practice, you'd track counts
        // For now, just set the current resource as top if it's different
        let mut resource_counts: HashMap<String, u64> = HashMap::new();
        *resource_counts.entry(resource.to_string()).or_insert(0) += 1;

        if let Some(current_top) = &activity.top_resource {
            if current_top != resource {
                activity.top_resource = Some(resource.to_string());
            }
        } else {
            activity.top_resource = Some(resource.to_string());
        }
    }

    /// Calculate risk score for a session
    fn calculate_session_risk(activity: &SessionActivity) -> u8 {
        let mut score = 0u8;

        // High event rate increases risk
        if activity.events_per_minute > 100.0 {
            score += 30;
        } else if activity.events_per_minute > 50.0 {
            score += 15;
        }

        // Many error/critical events increase risk
        let error_count = activity.events_by_level.get(&LogLevel::Error).unwrap_or(&0)
            + activity
                .events_by_level
                .get(&LogLevel::Critical)
                .unwrap_or(&0);
        if error_count > 10 {
            score += 25;
        } else if error_count > 5 {
            score += 10;
        }

        // Security events always increase risk
        let security_events = activity
            .events_by_category
            .get(&EventCategory::Security)
            .unwrap_or(&0);
        score += (*security_events as u8).min(20);

        // Long duration with high activity might indicate automated behavior
        if activity.duration_seconds > 3600 && activity.events_per_minute > 20.0 {
            score += 15;
        }

        score.min(100)
    }
}

/// Session statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    /// Total number of tracked sessions
    pub total_sessions: usize,
    /// Number of currently active sessions
    pub active_sessions: usize,
    /// Total events across all sessions
    pub total_events: u64,
    /// Average events per session
    pub avg_events_per_session: f64,
    /// Number of high-risk sessions
    pub high_risk_sessions: usize,
}

impl Default for SessionTracker {
    fn default() -> Self {
        Self::new(1000) // Default to 1000 concurrent sessions
    }
}
