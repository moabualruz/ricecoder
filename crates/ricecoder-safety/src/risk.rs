//! Risk scoring and analysis functionality

use crate::error::{SafetyError, SafetyResult};
use chrono::Timelike;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Risk level classifications
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Low risk - safe to proceed
    Low,
    /// Medium risk - requires review
    Medium,
    /// High risk - requires approval
    High,
    /// Critical risk - blocked
    Critical,
}

impl RiskLevel {
    /// Convert a score to a risk level
    pub fn from_score(score: u8) -> Self {
        match score {
            0..=25 => RiskLevel::Low,
            26..=50 => RiskLevel::Medium,
            51..=75 => RiskLevel::High,
            76..=u8::MAX => RiskLevel::Critical,
        }
    }

    /// Get the minimum score for this risk level
    pub fn min_score(&self) -> u8 {
        match self {
            RiskLevel::Low => 0,
            RiskLevel::Medium => 26,
            RiskLevel::High => 51,
            RiskLevel::Critical => 76,
        }
    }

    /// Get the maximum score for this risk level
    pub fn max_score(&self) -> u8 {
        match self {
            RiskLevel::Low => 25,
            RiskLevel::Medium => 50,
            RiskLevel::High => 75,
            RiskLevel::Critical => 100,
        }
    }
}

/// Risk score with detailed breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskScore {
    /// Overall risk score (0-100)
    pub score: u8,
    /// Risk level classification
    pub level: RiskLevel,
    /// Risk factors contributing to the score
    pub factors: RiskFactors,
    /// Confidence in the risk assessment (0.0-1.0)
    pub confidence: f64,
    /// Recommendations for risk mitigation
    pub recommendations: Vec<String>,
    /// Timestamp of the assessment
    pub assessed_at: chrono::DateTime<chrono::Utc>,
}

/// Risk factors contributing to risk score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactors {
    /// User behavior risk factors
    pub user_factors: HashMap<String, u8>,
    /// Operation risk factors
    pub operation_factors: HashMap<String, u8>,
    /// Environmental risk factors
    pub environment_factors: HashMap<String, u8>,
    /// Historical risk factors
    pub historical_factors: HashMap<String, u8>,
}

impl Default for RiskFactors {
    fn default() -> Self {
        Self {
            user_factors: HashMap::new(),
            operation_factors: HashMap::new(),
            environment_factors: HashMap::new(),
            historical_factors: HashMap::new(),
        }
    }
}

/// Risk scorer for analyzing operations and users
pub struct RiskScorer {
    /// Risk scoring rules
    rules: HashMap<String, RiskRule>,
    /// Historical risk data
    historical_data: HashMap<String, Vec<RiskScore>>,
}

impl RiskScorer {
    /// Create a new risk scorer
    pub fn new() -> Self {
        let mut scorer = Self {
            rules: HashMap::new(),
            historical_data: HashMap::new(),
        };
        scorer.initialize_default_rules();
        scorer
    }

    /// Score an action based on context
    pub fn score_action(&self, action: &str, context: &RiskContext) -> SafetyResult<RiskScore> {
        let mut total_score = 0u8;
        let mut factors = RiskFactors::default();
        let mut recommendations = Vec::new();

        // Apply relevant rules
        for rule in self.rules.values() {
            if rule.applies_to(action, context) {
                let rule_score = rule.calculate_score(context);
                total_score = total_score.saturating_add(rule_score);

                // Add factors
                match rule.category {
                    RiskCategory::User => {
                        factors.user_factors.insert(rule.name.clone(), rule_score);
                    }
                    RiskCategory::Operation => {
                        factors
                            .operation_factors
                            .insert(rule.name.clone(), rule_score);
                    }
                    RiskCategory::Environment => {
                        factors
                            .environment_factors
                            .insert(rule.name.clone(), rule_score);
                    }
                    RiskCategory::Historical => {
                        factors
                            .historical_factors
                            .insert(rule.name.clone(), rule_score);
                    }
                }

                // Add recommendations
                if rule_score > 50 {
                    recommendations.extend(rule.recommendations.clone());
                }
            }
        }

        // Cap at 100
        total_score = total_score.min(100);

        // Calculate confidence based on available data
        let confidence = self.calculate_confidence(context);

        // Add historical factors
        if let Some(user_id) = &context.user_id {
            if let Some(history) = self.historical_data.get(user_id) {
                if !history.is_empty() {
                    let avg_historical_score =
                        history.iter().map(|s| s.score as f64).sum::<f64>() / history.len() as f64;
                    let historical_factor = (avg_historical_score * 0.3) as u8;
                    total_score = total_score.saturating_add(historical_factor);
                    factors
                        .historical_factors
                        .insert("historical_average".to_string(), historical_factor);
                }
            }
        }

        let level = RiskLevel::from_score(total_score);

        Ok(RiskScore {
            score: total_score,
            level,
            factors,
            confidence,
            recommendations,
            assessed_at: chrono::Utc::now(),
        })
    }

    /// Add a custom risk rule
    pub fn add_rule(&mut self, rule: RiskRule) {
        self.rules.insert(rule.name.clone(), rule);
    }

    /// Record a risk assessment for historical analysis
    pub fn record_assessment(&mut self, user_id: &str, score: RiskScore) {
        self.historical_data
            .entry(user_id.to_string())
            .or_insert_with(Vec::new)
            .push(score);

        // Keep only last 100 assessments per user
        if let Some(history) = self.historical_data.get_mut(user_id) {
            if history.len() > 100 {
                history.remove(0);
            }
        }
    }

    /// Get risk statistics for a user
    pub fn get_user_risk_stats(&self, user_id: &str) -> Option<RiskStats> {
        self.historical_data.get(user_id).map(|history| {
            let scores: Vec<u8> = history.iter().map(|s| s.score).collect();
            let avg_score = scores.iter().map(|&s| s as f64).sum::<f64>() / scores.len() as f64;
            let max_score = *scores.iter().max().unwrap_or(&0);
            let min_score = *scores.iter().min().unwrap_or(&0);

            let level_distribution = history.iter().fold(HashMap::new(), |mut acc, score| {
                *acc.entry(score.level).or_insert(0) += 1;
                acc
            });

            RiskStats {
                user_id: user_id.to_string(),
                assessment_count: history.len(),
                average_score: avg_score,
                max_score,
                min_score,
                level_distribution,
                last_assessment: history.last().map(|s| s.assessed_at),
            }
        })
    }

    /// Initialize default risk scoring rules
    fn initialize_default_rules(&mut self) {
        // User-based rules
        self.add_rule(RiskRule {
            name: "new_user".to_string(),
            category: RiskCategory::User,
            condition: Box::new(|ctx| ctx.account_age_days.map_or(true, |age| age < 7)),
            score_calculator: Box::new(|_| 30),
            recommendations: vec![
                "New user - consider additional verification".to_string(),
                "Monitor first few operations closely".to_string(),
            ],
        });

        self.add_rule(RiskRule {
            name: "high_frequency_actions".to_string(),
            category: RiskCategory::User,
            condition: Box::new(|ctx| ctx.actions_per_hour.map_or(false, |rate| rate > 100)),
            score_calculator: Box::new(|ctx| {
                let rate = ctx.actions_per_hour.unwrap_or(0);
                ((rate as f64 / 100.0).min(3.0) * 25.0) as u8
            }),
            recommendations: vec![
                "High action frequency detected".to_string(),
                "Consider rate limiting".to_string(),
            ],
        });

        // Operation-based rules
        self.add_rule(RiskRule {
            name: "file_deletion".to_string(),
            category: RiskCategory::Operation,
            condition: Box::new(|ctx| {
                ctx.operation_type
                    .as_ref()
                    .map_or(false, |op| op.contains("delete"))
            }),
            score_calculator: Box::new(|_| 40),
            recommendations: vec![
                "File deletion operation - ensure proper authorization".to_string(),
                "Consider backup verification".to_string(),
            ],
        });

        self.add_rule(RiskRule {
            name: "large_file_operation".to_string(),
            category: RiskCategory::Operation,
            condition: Box::new(|ctx| ctx.file_size_mb.map_or(false, |size| size > 100.0)),
            score_calculator: Box::new(|ctx| {
                let size = ctx.file_size_mb.unwrap_or(0.0);
                ((size / 100.0).min(5.0) * 15.0) as u8
            }),
            recommendations: vec![
                "Large file operation detected".to_string(),
                "Verify sufficient resources".to_string(),
            ],
        });

        // Environment-based rules
        self.add_rule(RiskRule {
            name: "unusual_time".to_string(),
            category: RiskCategory::Environment,
            condition: Box::new(|ctx| {
                ctx.timestamp.map_or(false, |ts| {
                    let hour = ts.hour();
                    hour < 6 || hour > 22 // Outside normal business hours
                })
            }),
            score_calculator: Box::new(|_| 20),
            recommendations: vec![
                "Operation outside normal hours".to_string(),
                "Verify legitimacy of request".to_string(),
            ],
        });

        self.add_rule(RiskRule {
            name: "unknown_location".to_string(),
            category: RiskCategory::Environment,
            condition: Box::new(|ctx| ctx.is_unusual_location),
            score_calculator: Box::new(|_| 35),
            recommendations: vec![
                "Unusual location detected".to_string(),
                "Consider additional authentication".to_string(),
            ],
        });
    }

    /// Calculate confidence in risk assessment
    fn calculate_confidence(&self, context: &RiskContext) -> f64 {
        let mut confidence = 0.5; // Base confidence

        // Increase confidence based on available data
        if context.user_id.is_some() {
            confidence += 0.1;
        }
        if context.account_age_days.is_some() {
            confidence += 0.1;
        }
        if context.actions_per_hour.is_some() {
            confidence += 0.1;
        }
        if context.file_size_mb.is_some() {
            confidence += 0.05;
        }
        if context.is_unusual_location {
            confidence += 0.1;
        }

        // Historical data increases confidence
        if let Some(user_id) = &context.user_id {
            if let Some(history) = self.historical_data.get(user_id) {
                confidence += (history.len() as f64 * 0.01).min(0.2);
            }
        }

        confidence.min(1.0)
    }
}

/// Risk rule definition
pub struct RiskRule {
    /// Rule name
    pub name: String,
    /// Rule category
    pub category: RiskCategory,
    /// Condition function
    pub condition: Box<dyn Fn(&RiskContext) -> bool + Send + Sync>,
    /// Score calculator function
    pub score_calculator: Box<dyn Fn(&RiskContext) -> u8 + Send + Sync>,
    /// Recommendations for high-risk scenarios
    pub recommendations: Vec<String>,
}

impl RiskRule {
    /// Check if this rule applies to the given action and context
    pub fn applies_to(&self, _action: &str, context: &RiskContext) -> bool {
        (self.condition)(context)
    }

    /// Calculate the risk score for this rule
    pub fn calculate_score(&self, context: &RiskContext) -> u8 {
        (self.score_calculator)(context)
    }
}

/// Risk categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RiskCategory {
    /// User behavior related
    User,
    /// Operation characteristics
    Operation,
    /// Environmental factors
    Environment,
    /// Historical patterns
    Historical,
}

/// Context for risk assessment
#[derive(Debug, Clone)]
pub struct RiskContext {
    /// User ID
    pub user_id: Option<String>,
    /// Account age in days
    pub account_age_days: Option<u32>,
    /// Actions per hour by this user
    pub actions_per_hour: Option<u32>,
    /// Operation type
    pub operation_type: Option<String>,
    /// File size in MB
    pub file_size_mb: Option<f64>,
    /// Timestamp of operation
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
    /// Whether the location is unusual
    pub is_unusual_location: bool,
    /// Additional context data
    pub additional_data: HashMap<String, serde_json::Value>,
}

impl Default for RiskContext {
    fn default() -> Self {
        Self {
            user_id: None,
            account_age_days: None,
            actions_per_hour: None,
            operation_type: None,
            file_size_mb: None,
            timestamp: Some(chrono::Utc::now()),
            is_unusual_location: false,
            additional_data: HashMap::new(),
        }
    }
}

/// Risk statistics for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskStats {
    /// User ID
    pub user_id: String,
    /// Number of risk assessments
    pub assessment_count: usize,
    /// Average risk score
    pub average_score: f64,
    /// Maximum risk score
    pub max_score: u8,
    /// Minimum risk score
    pub min_score: u8,
    /// Distribution of risk levels
    pub level_distribution: HashMap<RiskLevel, usize>,
    /// Last assessment timestamp
    pub last_assessment: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for RiskScorer {
    fn default() -> Self {
        Self::new()
    }
}
