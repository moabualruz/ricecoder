//! SOC 2 compliance reporting entities

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::BenchmarkResult;

/// SOC 2 compliance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub id: String,
    pub generated_at: DateTime<Utc>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub compliance_status: ComplianceStatus,
    pub findings: Vec<ComplianceFinding>,
    pub recommendations: Vec<String>,
    pub audit_trail_count: usize,
    pub alert_count: usize,
    pub benchmark_results: Vec<BenchmarkResult>,
}

impl ComplianceReport {
    /// Create a new compliance report
    pub fn new(period_start: DateTime<Utc>, period_end: DateTime<Utc>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            generated_at: Utc::now(),
            period_start,
            period_end,
            compliance_status: ComplianceStatus::Compliant,
            findings: Vec::new(),
            recommendations: Vec::new(),
            audit_trail_count: 0,
            alert_count: 0,
            benchmark_results: Vec::new(),
        }
    }

    /// Add a finding
    pub fn add_finding(&mut self, finding: ComplianceFinding) {
        let is_critical =
            finding.severity == FindingSeverity::High || finding.severity == FindingSeverity::Critical;
        self.findings.push(finding);
        if is_critical {
            self.compliance_status = ComplianceStatus::NonCompliant;
        }
    }

    /// Add recommendation
    pub fn add_recommendation(&mut self, recommendation: String) {
        self.recommendations.push(recommendation);
    }
}

/// Compliance status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplianceStatus {
    Compliant,
    NonCompliant,
    UnderReview,
}

/// Compliance finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFinding {
    pub id: String,
    pub title: String,
    pub description: String,
    pub severity: FindingSeverity,
    pub soc2_principle: Soc2Principle,
    pub remediation: String,
}

impl ComplianceFinding {
    /// Create a new finding
    pub fn new(
        title: String,
        description: String,
        severity: FindingSeverity,
        principle: Soc2Principle,
        remediation: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            description,
            severity,
            soc2_principle: principle,
            remediation,
        }
    }
}

/// Finding severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FindingSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// SOC 2 principles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Soc2Principle {
    Security,
    Availability,
    ProcessingIntegrity,
    Confidentiality,
    Privacy,
}
