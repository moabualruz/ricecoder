//! Enterprise compliance and audit logging

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::IndustryResult;

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Unique entry ID
    pub id: Uuid,
    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,
    /// User or system that performed the action
    pub actor: String,
    /// Action performed
    pub action: String,
    /// Resource affected
    pub resource: String,
    /// Action result (success/failure)
    pub result: AuditResult,
    /// Additional context data
    pub context: HashMap<String, serde_json::Value>,
    /// IP address or source identifier
    pub source: String,
    /// Session ID if applicable
    pub session_id: Option<String>,
}

/// Audit result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditResult {
    /// Action succeeded
    Success,
    /// Action failed with error message
    Failure(String),
    /// Action was denied
    Denied(String),
}

/// Security validation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRule {
    /// Rule ID
    pub id: String,
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: String,
    /// Severity level
    pub severity: SecuritySeverity,
    /// Rule logic (could be extended to support complex expressions)
    pub condition: SecurityCondition,
    /// Actions to take when rule is violated
    pub actions: Vec<SecurityAction>,
}

/// Security severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum SecuritySeverity {
    /// Low severity - informational
    Low,
    /// Medium severity - requires attention
    Medium,
    /// High severity - critical security issue
    High,
    /// Critical severity - immediate action required
    Critical,
}

/// Security condition for validation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityCondition {
    /// Check if a field matches a pattern
    PatternMatch { field: String, pattern: String },
    /// Check if a value is within a range
    RangeCheck {
        field: String,
        min: Option<f64>,
        max: Option<f64>,
    },
    /// Check if a field is required
    RequiredField { field: String },
    /// Check if a value is in an allowed list
    AllowedValues { field: String, values: Vec<String> },
    /// Custom validation function (placeholder for extensibility)
    Custom {
        validator: String,
        params: HashMap<String, serde_json::Value>,
    },
}

/// Actions to take when security rules are violated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityAction {
    /// Log the violation
    Log,
    /// Block the operation
    Block,
    /// Send alert to administrators
    Alert,
    /// Require additional authentication
    RequireAuth,
}

/// Compliance check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceCheck {
    /// Check ID
    pub id: String,
    /// Check name
    pub name: String,
    /// Check result
    pub result: ComplianceResult,
    /// Details about the check
    pub details: String,
    /// Timestamp of the check
    pub timestamp: DateTime<Utc>,
}

/// Compliance result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceResult {
    /// Check passed
    Passed,
    /// Check failed with reason
    Failed(String),
    /// Check was skipped
    Skipped(String),
}

/// Audit logger for enterprise compliance
pub struct AuditLogger {
    entries: RwLock<Vec<AuditEntry>>,
    max_entries: usize,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: RwLock::new(Vec::new()),
            max_entries,
        }
    }

    /// Log an audit entry
    pub async fn log(&self, entry: AuditEntry) -> IndustryResult<()> {
        let mut entries = self.entries.write().await;

        // Add new entry
        entries.push(entry);

        // Maintain max entries limit (remove oldest)
        if entries.len() > self.max_entries {
            let excess = entries.len() - self.max_entries;
            entries.drain(0..excess);
        }

        Ok(())
    }

    /// Log a successful action
    pub async fn log_success(
        &self,
        actor: String,
        action: String,
        resource: String,
        context: HashMap<String, serde_json::Value>,
        source: String,
        session_id: Option<String>,
    ) -> IndustryResult<()> {
        let entry = AuditEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            actor,
            action,
            resource,
            result: AuditResult::Success,
            context,
            source,
            session_id,
        };

        self.log(entry).await
    }

    /// Log a failed action
    pub async fn log_failure(
        &self,
        actor: String,
        action: String,
        resource: String,
        error: String,
        context: HashMap<String, serde_json::Value>,
        source: String,
        session_id: Option<String>,
    ) -> IndustryResult<()> {
        let entry = AuditEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            actor,
            action,
            resource,
            result: AuditResult::Failure(error),
            context,
            source,
            session_id,
        };

        self.log(entry).await
    }

    /// Get audit entries with optional filtering
    pub async fn get_entries(
        &self,
        actor: Option<&str>,
        action: Option<&str>,
        resource: Option<&str>,
        limit: Option<usize>,
    ) -> Vec<AuditEntry> {
        let entries = self.entries.read().await;
        let mut filtered: Vec<_> = entries
            .iter()
            .filter(|entry| {
                actor.map_or(true, |a| entry.actor.contains(a))
                    && action.map_or(true, |a| entry.action.contains(a))
                    && resource.map_or(true, |r| entry.resource.contains(r))
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

    /// Get audit statistics
    pub async fn get_stats(&self) -> AuditStats {
        let entries = self.entries.read().await;
        let total_entries = entries.len();

        let mut success_count = 0;
        let mut failure_count = 0;
        let mut denied_count = 0;

        for entry in entries.iter() {
            match &entry.result {
                AuditResult::Success => success_count += 1,
                AuditResult::Failure(_) => failure_count += 1,
                AuditResult::Denied(_) => denied_count += 1,
            }
        }

        AuditStats {
            total_entries,
            success_count,
            failure_count,
            denied_count,
            success_rate: if total_entries > 0 {
                success_count as f64 / total_entries as f64
            } else {
                0.0
            },
        }
    }
}

/// Audit statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStats {
    /// Total number of audit entries
    pub total_entries: usize,
    /// Number of successful actions
    pub success_count: usize,
    /// Number of failed actions
    pub failure_count: usize,
    /// Number of denied actions
    pub denied_count: usize,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
}

/// Security validator for enterprise compliance
pub struct SecurityValidator {
    rules: RwLock<Vec<SecurityRule>>,
}

impl SecurityValidator {
    /// Create a new security validator
    pub fn new() -> Self {
        Self {
            rules: RwLock::new(Vec::new()),
        }
    }

    /// Add a security rule
    pub async fn add_rule(&self, rule: SecurityRule) -> IndustryResult<()> {
        self.rules.write().await.push(rule);
        Ok(())
    }

    /// Validate data against all security rules
    pub async fn validate(
        &self,
        data: &serde_json::Value,
    ) -> IndustryResult<Vec<SecurityViolation>> {
        let rules = self.rules.read().await;
        let mut violations = Vec::new();

        for rule in rules.iter() {
            if let Some(violation) = self.check_rule(rule, data).await? {
                violations.push(violation);
            }
        }

        Ok(violations)
    }

    /// Check a single security rule
    async fn check_rule(
        &self,
        rule: &SecurityRule,
        data: &serde_json::Value,
    ) -> IndustryResult<Option<SecurityViolation>> {
        let violated = match &rule.condition {
            SecurityCondition::RequiredField { field } => !Self::has_field(data, field),
            SecurityCondition::PatternMatch { field, pattern } => {
                if let Some(value) = Self::get_field_value(data, field) {
                    if let Some(str_value) = value.as_str() {
                        !regex::Regex::new(pattern)?.is_match(str_value)
                    } else {
                        true // Not a string, so it doesn't match
                    }
                } else {
                    false // Field not present, rule doesn't apply
                }
            }
            SecurityCondition::RangeCheck { field, min, max } => {
                if let Some(value) = Self::get_field_value(data, field) {
                    if let Some(num_value) = value.as_f64() {
                        let min_violation = min.map_or(false, |min_val| num_value < min_val);
                        let max_violation = max.map_or(false, |max_val| num_value > max_val);
                        min_violation || max_violation
                    } else {
                        true // Not a number, so it violates
                    }
                } else {
                    false // Field not present, rule doesn't apply
                }
            }
            SecurityCondition::AllowedValues { field, values } => {
                if let Some(value) = Self::get_field_value(data, field) {
                    if let Some(str_value) = value.as_str() {
                        !values.contains(&str_value.to_string())
                    } else {
                        true // Not a string, so it violates
                    }
                } else {
                    false // Field not present, rule doesn't apply
                }
            }
            SecurityCondition::Custom { .. } => {
                // Placeholder for custom validation logic
                false
            }
        };

        if violated {
            Ok(Some(SecurityViolation {
                rule_id: rule.id.clone(),
                rule_name: rule.name.clone(),
                severity: rule.severity.clone(),
                description: rule.description.clone(),
                actions: rule.actions.clone(),
            }))
        } else {
            Ok(None)
        }
    }

    /// Check if a field exists in the JSON data
    fn has_field(data: &serde_json::Value, field_path: &str) -> bool {
        Self::get_field_value(data, field_path).is_some()
    }

    /// Get a field value from JSON data using dot notation
    fn get_field_value<'a>(
        data: &'a serde_json::Value,
        field_path: &str,
    ) -> Option<&'a serde_json::Value> {
        let mut current = data;
        for part in field_path.split('.') {
            match current.get(part) {
                Some(value) => current = value,
                None => return None,
            }
        }
        Some(current)
    }
}

/// Security violation detected during validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityViolation {
    /// Rule ID that was violated
    pub rule_id: String,
    /// Rule name
    pub rule_name: String,
    /// Severity of the violation
    pub severity: SecuritySeverity,
    /// Description of the violation
    pub description: String,
    /// Actions to take
    pub actions: Vec<SecurityAction>,
}

/// Compliance manager for enterprise requirements
pub struct ComplianceManager {
    audit_logger: AuditLogger,
    pub security_validator: SecurityValidator,
    checks: RwLock<Vec<ComplianceCheck>>,
}

impl ComplianceManager {
    /// Create a new compliance manager
    pub fn new(audit_logger: AuditLogger, security_validator: SecurityValidator) -> Self {
        Self {
            audit_logger,
            security_validator,
            checks: RwLock::new(Vec::new()),
        }
    }

    /// Run a compliance check
    pub async fn run_check(&self, check: ComplianceCheck) -> IndustryResult<()> {
        self.checks.write().await.push(check);
        Ok(())
    }

    /// Get compliance check results
    pub async fn get_check_results(&self, limit: Option<usize>) -> Vec<ComplianceCheck> {
        let checks = self.checks.read().await;
        let mut results: Vec<_> = checks.iter().cloned().collect();

        // Sort by timestamp (newest first)
        results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // Apply limit
        if let Some(limit) = limit {
            results.truncate(limit);
        }

        results
    }

    /// Get compliance summary
    pub async fn get_compliance_summary(&self) -> ComplianceSummary {
        let checks = self.checks.read().await;
        let total_checks = checks.len();

        let mut passed = 0;
        let mut failed = 0;
        let mut skipped = 0;

        for check in checks.iter() {
            match &check.result {
                ComplianceResult::Passed => passed += 1,
                ComplianceResult::Failed(_) => failed += 1,
                ComplianceResult::Skipped(_) => skipped += 1,
            }
        }

        ComplianceSummary {
            total_checks,
            passed_checks: passed,
            failed_checks: failed,
            skipped_checks: skipped,
            compliance_rate: if total_checks > 0 {
                passed as f64 / total_checks as f64
            } else {
                0.0
            },
        }
    }

    /// Validate data and log security events
    pub async fn validate_and_log(
        &self,
        actor: String,
        action: String,
        resource: String,
        data: &serde_json::Value,
        source: String,
        session_id: Option<String>,
    ) -> IndustryResult<Vec<SecurityViolation>> {
        let violations = self.security_validator.validate(data).await?;

        if violations.is_empty() {
            // Log successful validation
            self.audit_logger
                .log_success(
                    actor,
                    action,
                    resource,
                    HashMap::from([("validation".to_string(), serde_json::json!("passed"))]),
                    source,
                    session_id,
                )
                .await?;
        } else {
            // Log validation failure
            let violation_details: Vec<_> = violations
                .iter()
                .map(|v| {
                    serde_json::json!({
                        "rule_id": v.rule_id,
                        "severity": format!("{:?}", v.severity),
                        "description": v.description
                    })
                })
                .collect();

            self.audit_logger
                .log_failure(
                    actor,
                    action,
                    resource,
                    "Security validation failed".to_string(),
                    HashMap::from([(
                        "violations".to_string(),
                        serde_json::json!(violation_details),
                    )]),
                    source,
                    session_id,
                )
                .await?;
        }

        Ok(violations)
    }
}

/// Compliance summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceSummary {
    /// Total number of compliance checks
    pub total_checks: usize,
    /// Number of passed checks
    pub passed_checks: usize,
    /// Number of failed checks
    pub failed_checks: usize,
    /// Number of skipped checks
    pub skipped_checks: usize,
    /// Overall compliance rate (0.0 to 1.0)
    pub compliance_rate: f64,
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new(10000) // Default to 10k entries
    }
}

impl Default for SecurityValidator {
    fn default() -> Self {
        Self::new()
    }
}
