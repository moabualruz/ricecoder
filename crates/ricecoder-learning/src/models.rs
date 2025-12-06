/// Core data models for the learning system
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// Scope where rules are stored and applied
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleScope {
    /// Global scope: ~/.ricecoder/rules/
    Global,
    /// Project scope: ./.ricecoder/rules/
    Project,
    /// Session-only (in-memory)
    Session,
}

impl std::fmt::Display for RuleScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuleScope::Global => write!(f, "global"),
            RuleScope::Project => write!(f, "project"),
            RuleScope::Session => write!(f, "session"),
        }
    }
}

/// Source of a rule
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleSource {
    /// Automatically captured from user decisions
    Learned,
    /// User-defined rule
    Manual,
    /// Promoted from project to global scope
    Promoted,
}

impl std::fmt::Display for RuleSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuleSource::Learned => write!(f, "learned"),
            RuleSource::Manual => write!(f, "manual"),
            RuleSource::Promoted => write!(f, "promoted"),
        }
    }
}

/// A learned rule that guides code generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    /// Unique identifier for the rule
    pub id: String,
    /// Scope where this rule is stored
    pub scope: RuleScope,
    /// Pattern that triggers this rule
    pub pattern: String,
    /// Action to take when pattern matches
    pub action: String,
    /// Source of the rule
    pub source: RuleSource,
    /// When the rule was created
    pub created_at: DateTime<Utc>,
    /// When the rule was last updated
    pub updated_at: DateTime<Utc>,
    /// Version number of the rule
    pub version: u32,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
    /// Number of times this rule has been applied
    pub usage_count: u64,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f32,
    /// Additional metadata
    pub metadata: serde_json::Value,
}

impl Rule {
    /// Create a new rule
    pub fn new(
        scope: RuleScope,
        pattern: String,
        action: String,
        source: RuleSource,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            scope,
            pattern,
            action,
            source,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            version: 1,
            confidence: 0.5,
            usage_count: 0,
            success_rate: 0.0,
            metadata: serde_json::json!({}),
        }
    }
}

/// Context in which a decision was made
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionContext {
    /// Path to the project
    pub project_path: PathBuf,
    /// Path to the file being edited
    pub file_path: PathBuf,
    /// Line number in the file
    pub line_number: u32,
    /// Type of agent making the decision
    pub agent_type: String,
}

/// A captured user decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    /// Unique identifier for the decision
    pub id: String,
    /// When the decision was made
    pub timestamp: DateTime<Utc>,
    /// Context in which the decision was made
    pub context: DecisionContext,
    /// Type of decision (e.g., "code_generation", "refactoring")
    pub decision_type: String,
    /// Input that led to the decision
    pub input: serde_json::Value,
    /// Output of the decision
    pub output: serde_json::Value,
    /// Additional metadata
    pub metadata: serde_json::Value,
}

impl Decision {
    /// Create a new decision
    pub fn new(
        context: DecisionContext,
        decision_type: String,
        input: serde_json::Value,
        output: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            context,
            decision_type,
            input,
            output,
            metadata: serde_json::json!({}),
        }
    }
}

/// Example of a pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternExample {
    /// Input that produced this example
    pub input: serde_json::Value,
    /// Output of this example
    pub output: serde_json::Value,
    /// Context of this example
    pub context: serde_json::Value,
}

/// A learned pattern extracted from repeated decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedPattern {
    /// Unique identifier for the pattern
    pub id: String,
    /// Type of pattern (e.g., "code_generation", "refactoring")
    pub pattern_type: String,
    /// Human-readable description
    pub description: String,
    /// Examples of this pattern
    pub examples: Vec<PatternExample>,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
    /// Number of times this pattern has been observed
    pub occurrences: usize,
    /// When the pattern was first identified
    pub created_at: DateTime<Utc>,
    /// When the pattern was last observed
    pub last_seen: DateTime<Utc>,
}

impl LearnedPattern {
    /// Create a new pattern
    pub fn new(pattern_type: String, description: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            pattern_type,
            description,
            examples: Vec::new(),
            confidence: 0.0,
            occurrences: 0,
            created_at: Utc::now(),
            last_seen: Utc::now(),
        }
    }
}

/// Learning system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningConfig {
    /// Scope for learning (global, project, or session)
    pub scope: RuleScope,
    /// Whether learning is enabled
    pub enabled: bool,
    /// Whether approval is required for new rules
    pub approval_required: bool,
    /// Whether to automatically promote rules
    pub auto_promote: bool,
    /// How many days to retain rules
    pub retention_days: u32,
    /// Maximum number of rules to store
    pub max_rules: usize,
}

impl Default for LearningConfig {
    fn default() -> Self {
        Self {
            scope: RuleScope::Global,
            enabled: true,
            approval_required: false,
            auto_promote: false,
            retention_days: 365,
            max_rules: 10000,
        }
    }
}

impl LearningConfig {
    /// Create a new configuration with default values
    pub fn new(scope: RuleScope) -> Self {
        Self {
            scope,
            ..Default::default()
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> crate::error::Result<()> {
        if self.retention_days == 0 {
            return Err(crate::error::LearningError::ConfigurationError(
                "retention_days must be greater than 0".to_string(),
            ));
        }

        if self.max_rules == 0 {
            return Err(crate::error::LearningError::ConfigurationError(
                "max_rules must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_creation() {
        let rule = Rule::new(
            RuleScope::Global,
            "pattern".to_string(),
            "action".to_string(),
            RuleSource::Learned,
        );

        assert_eq!(rule.scope, RuleScope::Global);
        assert_eq!(rule.pattern, "pattern");
        assert_eq!(rule.action, "action");
        assert_eq!(rule.source, RuleSource::Learned);
        assert_eq!(rule.version, 1);
        assert_eq!(rule.confidence, 0.5);
        assert_eq!(rule.usage_count, 0);
    }

    #[test]
    fn test_rule_serialization() {
        let rule = Rule::new(
            RuleScope::Project,
            "test_pattern".to_string(),
            "test_action".to_string(),
            RuleSource::Manual,
        );

        let json = serde_json::to_string(&rule).expect("Failed to serialize");
        let deserialized: Rule = serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(rule.id, deserialized.id);
        assert_eq!(rule.scope, deserialized.scope);
        assert_eq!(rule.pattern, deserialized.pattern);
    }

    #[test]
    fn test_decision_creation() {
        let context = DecisionContext {
            project_path: PathBuf::from("/project"),
            file_path: PathBuf::from("/project/src/main.rs"),
            line_number: 42,
            agent_type: "code_generator".to_string(),
        };

        let decision = Decision::new(
            context.clone(),
            "code_generation".to_string(),
            serde_json::json!({"input": "test"}),
            serde_json::json!({"output": "result"}),
        );

        assert_eq!(decision.decision_type, "code_generation");
        assert_eq!(decision.context.line_number, 42);
    }

    #[test]
    fn test_decision_serialization() {
        let context = DecisionContext {
            project_path: PathBuf::from("/project"),
            file_path: PathBuf::from("/project/src/main.rs"),
            line_number: 10,
            agent_type: "test_agent".to_string(),
        };

        let decision = Decision::new(
            context,
            "test_type".to_string(),
            serde_json::json!({}),
            serde_json::json!({}),
        );

        let json = serde_json::to_string(&decision).expect("Failed to serialize");
        let deserialized: Decision = serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(decision.id, deserialized.id);
        assert_eq!(decision.decision_type, deserialized.decision_type);
    }

    #[test]
    fn test_pattern_creation() {
        let pattern = LearnedPattern::new(
            "code_generation".to_string(),
            "Test pattern".to_string(),
        );

        assert_eq!(pattern.pattern_type, "code_generation");
        assert_eq!(pattern.description, "Test pattern");
        assert_eq!(pattern.occurrences, 0);
        assert_eq!(pattern.confidence, 0.0);
    }

    #[test]
    fn test_pattern_serialization() {
        let pattern = LearnedPattern::new(
            "refactoring".to_string(),
            "Refactoring pattern".to_string(),
        );

        let json = serde_json::to_string(&pattern).expect("Failed to serialize");
        let deserialized: LearnedPattern =
            serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(pattern.id, deserialized.id);
        assert_eq!(pattern.pattern_type, deserialized.pattern_type);
    }

    #[test]
    fn test_learning_config_default() {
        let config = LearningConfig::default();

        assert_eq!(config.scope, RuleScope::Global);
        assert!(config.enabled);
        assert!(!config.approval_required);
        assert!(!config.auto_promote);
        assert_eq!(config.retention_days, 365);
        assert_eq!(config.max_rules, 10000);
    }

    #[test]
    fn test_learning_config_validation() {
        let mut config = LearningConfig::default();
        assert!(config.validate().is_ok());

        config.retention_days = 0;
        assert!(config.validate().is_err());

        config.retention_days = 365;
        config.max_rules = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_rule_scope_display() {
        assert_eq!(RuleScope::Global.to_string(), "global");
        assert_eq!(RuleScope::Project.to_string(), "project");
        assert_eq!(RuleScope::Session.to_string(), "session");
    }

    #[test]
    fn test_rule_source_display() {
        assert_eq!(RuleSource::Learned.to_string(), "learned");
        assert_eq!(RuleSource::Manual.to_string(), "manual");
        assert_eq!(RuleSource::Promoted.to_string(), "promoted");
    }
}
