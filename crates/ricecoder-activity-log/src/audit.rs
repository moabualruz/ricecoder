//! Audit trail functionality for compliance and security

use crate::error::{ActivityLogError, ActivityLogResult};
use crate::events::{ActivityEvent, EventCategory, LogLevel};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Audit trail entry for compliance tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrail {
    /// Unique audit entry ID
    pub id: Uuid,
    /// Timestamp of the audit event
    pub timestamp: DateTime<Utc>,
    /// Type of audit event
    pub event_type: AuditEventType,
    /// User or system that triggered the event
    pub actor: String,
    /// Action performed
    pub action: String,
    /// Resource affected
    pub resource: String,
    /// Outcome of the action
    pub outcome: AuditOutcome,
    /// Additional audit data
    pub audit_data: HashMap<String, serde_json::Value>,
    /// Compliance requirements met
    pub compliance_flags: Vec<ComplianceFlag>,
    /// Risk score (0-100)
    pub risk_score: Option<u8>,
    /// Session ID if applicable
    pub session_id: Option<String>,
    /// IP address or source
    pub source_ip: Option<String>,
}

/// Types of audit events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditEventType {
    /// User authentication events
    Authentication,
    /// Authorization decisions
    Authorization,
    /// Data access events
    DataAccess,
    /// Configuration changes
    Configuration,
    /// Security events
    Security,
    /// Administrative actions
    Administration,
    /// Compliance-related events
    Compliance,
    /// Custom audit event type
    Custom(String),
}

/// Outcome of an audited action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditOutcome {
    /// Action succeeded
    Success,
    /// Action failed with reason
    Failure(String),
    /// Action was denied
    Denied(String),
    /// Action was blocked by security controls
    Blocked(String),
}

/// Compliance flags for regulatory requirements
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ComplianceFlag {
    /// GDPR compliance
    GDPR,
    /// HIPAA compliance
    HIPAA,
    /// SOC 2 compliance
    SOC2,
    /// PCI DSS compliance
    PCIDSS,
    /// ISO 27001 compliance
    ISO27001,
    /// Custom compliance requirement
    Custom(String),
}

/// Compliance event for regulatory reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceEvent {
    /// Unique event ID
    pub id: Uuid,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Compliance framework
    pub framework: ComplianceFlag,
    /// Event description
    pub description: String,
    /// Severity level
    pub severity: ComplianceSeverity,
    /// Related audit trail entries
    pub related_audits: Vec<Uuid>,
    /// Remediation actions taken
    pub remediation: Option<String>,
}

/// Compliance severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ComplianceSeverity {
    /// Low severity - informational
    Low,
    /// Medium severity - requires attention
    Medium,
    /// High severity - critical issue
    High,
    /// Critical severity - immediate action required
    Critical,
}

/// Audit logger for compliance and security tracking
pub struct AuditLogger {
    trails: RwLock<Vec<AuditTrail>>,
    compliance_events: RwLock<Vec<ComplianceEvent>>,
    max_entries: usize,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new(max_entries: usize) -> Self {
        Self {
            trails: RwLock::new(Vec::new()),
            compliance_events: RwLock::new(Vec::new()),
            max_entries,
        }
    }

    /// Log an audit trail entry
    pub async fn log_audit(&self, trail: AuditTrail) -> ActivityLogResult<()> {
        let mut trails = self.trails.write().await;

        // Add new entry
        trails.push(trail);

        // Maintain max entries limit (remove oldest)
        if trails.len() > self.max_entries {
            let excess = trails.len() - self.max_entries;
            trails.drain(0..excess);
        }

        Ok(())
    }

    /// Log a compliance event
    pub async fn log_compliance(&self, event: ComplianceEvent) -> ActivityLogResult<()> {
        self.compliance_events.write().await.push(event);
        Ok(())
    }

    /// Create and log an audit trail from an activity event
    pub async fn audit_from_activity(
        &self,
        event: &ActivityEvent,
        event_type: AuditEventType,
        outcome: AuditOutcome,
    ) -> ActivityLogResult<()> {
        let trail = AuditTrail {
            id: Uuid::new_v4(),
            timestamp: event.timestamp,
            event_type,
            actor: event.actor.clone(),
            action: event.action.clone(),
            resource: event.resource.clone(),
            outcome,
            audit_data: Self::extract_audit_data(event),
            compliance_flags: Self::determine_compliance_flags(event),
            risk_score: Self::calculate_risk_score(event),
            session_id: event.session_id.clone(),
            source_ip: event.source.clone(),
        };

        self.log_audit(trail).await
    }

    /// Get audit trails with optional filtering
    pub async fn get_audit_trails(
        &self,
        actor: Option<&str>,
        action: Option<&str>,
        resource: Option<&str>,
        limit: Option<usize>,
    ) -> Vec<AuditTrail> {
        let trails = self.trails.read().await;
        let mut filtered: Vec<_> = trails
            .iter()
            .filter(|trail| {
                actor.map_or(true, |a| trail.actor.contains(a))
                    && action.map_or(true, |a| trail.action.contains(a))
                    && resource.map_or(true, |r| trail.resource.contains(r))
            })
            .cloned()
            .collect();

        // Sort by timestamp (newest first)
        filtered.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // Apply limit
        if let Some(limit) = limit {
            filtered.truncate(limit);
        }

        filtered
    }

    /// Get compliance events
    pub async fn get_compliance_events(
        &self,
        framework: Option<&ComplianceFlag>,
    ) -> Vec<ComplianceEvent> {
        let events = self.compliance_events.read().await;
        let mut filtered: Vec<_> = if let Some(framework) = framework {
            events
                .iter()
                .filter(|event| &event.framework == framework)
                .cloned()
                .collect()
        } else {
            events.clone()
        };

        // Sort by timestamp (newest first)
        filtered.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        filtered
    }

    /// Generate compliance report
    pub async fn generate_compliance_report(&self, framework: &ComplianceFlag) -> ComplianceReport {
        let trails = self.trails.read().await;
        let events = self.compliance_events.read().await;

        let relevant_trails: Vec<_> = trails
            .iter()
            .filter(|trail| trail.compliance_flags.contains(framework))
            .collect();

        let relevant_events: Vec<_> = events
            .iter()
            .filter(|event| &event.framework == framework)
            .collect();

        let total_violations = relevant_events
            .iter()
            .filter(|event| {
                matches!(
                    event.severity,
                    ComplianceSeverity::High | ComplianceSeverity::Critical
                )
            })
            .count();

        let compliance_score = if relevant_trails.is_empty() {
            100.0
        } else {
            let compliant_actions = relevant_trails
                .iter()
                .filter(|trail| matches!(trail.outcome, AuditOutcome::Success))
                .count();
            (compliant_actions as f64 / relevant_trails.len() as f64) * 100.0
        };

        ComplianceReport {
            framework: framework.clone(),
            generated_at: Utc::now(),
            total_audit_trails: relevant_trails.len(),
            total_compliance_events: relevant_events.len(),
            compliance_score,
            violations_count: total_violations,
            recommendations: Self::generate_recommendations(framework, compliance_score),
        }
    }

    /// Extract audit data from activity event
    fn extract_audit_data(event: &ActivityEvent) -> HashMap<String, serde_json::Value> {
        let mut audit_data = HashMap::new();

        // Add basic event data
        audit_data.insert("level".to_string(), serde_json::json!(event.level));
        audit_data.insert("category".to_string(), serde_json::json!(event.category));
        audit_data.insert("details".to_string(), event.details.clone());

        // Add optional fields
        if let Some(duration) = event.duration_ms {
            audit_data.insert("duration_ms".to_string(), serde_json::json!(duration));
        }

        if let Some(user_agent) = &event.user_agent {
            audit_data.insert("user_agent".to_string(), serde_json::json!(user_agent));
        }

        // Add metadata
        for (key, value) in &event.metadata {
            audit_data.insert(format!("meta_{}", key), value.clone());
        }

        audit_data
    }

    /// Determine compliance flags based on event
    fn determine_compliance_flags(event: &ActivityEvent) -> Vec<ComplianceFlag> {
        let mut flags = Vec::new();

        match event.category {
            EventCategory::Security => {
                flags.push(ComplianceFlag::GDPR);
                flags.push(ComplianceFlag::HIPAA);
                flags.push(ComplianceFlag::SOC2);
            }
            EventCategory::UserInterface => {
                flags.push(ComplianceFlag::GDPR); // User data handling
            }
            EventCategory::AIProvider => {
                flags.push(ComplianceFlag::GDPR); // AI data processing
            }
            _ => {}
        }

        // Add flags based on risk score
        if let Some(risk) = Self::calculate_risk_score(event) {
            if risk > 70 {
                flags.push(ComplianceFlag::ISO27001);
            }
        }

        flags
    }

    /// Calculate risk score for an event
    fn calculate_risk_score(event: &ActivityEvent) -> Option<u8> {
        let mut score = 0u8;

        // Base score from log level
        score += match event.level {
            LogLevel::Debug => 10,
            LogLevel::Info => 20,
            LogLevel::Warn => 40,
            LogLevel::Error => 60,
            LogLevel::Critical => 80,
        };

        // Additional score from category
        score += match event.category {
            EventCategory::Security => 30,
            EventCategory::AIProvider => 20,
            EventCategory::Network => 15,
            EventCategory::FileSystem => 10,
            _ => 0,
        };

        // Additional score from action patterns
        if event.action.contains("delete") || event.action.contains("remove") {
            score += 20;
        }

        if event.action.contains("admin") || event.action.contains("sudo") {
            score += 25;
        }

        Some(score.min(100))
    }

    /// Generate compliance recommendations
    fn generate_recommendations(framework: &ComplianceFlag, score: f64) -> Vec<String> {
        let mut recommendations = Vec::new();

        if score < 80.0 {
            match framework {
                ComplianceFlag::GDPR => {
                    recommendations.push("Implement stricter data access controls".to_string());
                    recommendations.push("Add data processing consent tracking".to_string());
                }
                ComplianceFlag::HIPAA => {
                    recommendations.push("Enhance audit logging for PHI access".to_string());
                    recommendations.push("Implement data encryption at rest".to_string());
                }
                ComplianceFlag::SOC2 => {
                    recommendations.push("Strengthen access control mechanisms".to_string());
                    recommendations.push("Improve security monitoring".to_string());
                }
                _ => {
                    recommendations.push("Review and strengthen security controls".to_string());
                }
            }
        }

        recommendations
    }
}

/// Compliance report for regulatory frameworks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    /// Compliance framework
    pub framework: ComplianceFlag,
    /// Report generation timestamp
    pub generated_at: DateTime<Utc>,
    /// Total audit trails reviewed
    pub total_audit_trails: usize,
    /// Total compliance events
    pub total_compliance_events: usize,
    /// Compliance score (0-100)
    pub compliance_score: f64,
    /// Number of violations found
    pub violations_count: usize,
    /// Recommendations for improvement
    pub recommendations: Vec<String>,
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new(10000) // Default to 10k entries
    }
}
