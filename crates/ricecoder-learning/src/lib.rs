/// RiceCoder Learning System
///
/// This crate provides the learning system for RiceCoder, which captures user decisions
/// and converts them into reusable rules. The system supports multiple learning scopes
/// (global, project, session) and enables rule promotion across scopes.
pub mod analytics_engine;
pub mod confidence_score_property;
pub mod conflict_resolver;
pub mod decision_capture_property;
pub mod decision_logger;
pub mod drift_detector;
pub mod error;
pub mod intent_tracker;
pub mod intent_tracking_integration;
pub mod manager;
pub mod models;
pub mod pattern_capturer;
pub mod pattern_extraction_property;
pub mod pattern_validation_integration;
pub mod pattern_validator;
pub mod rule_application;
pub mod rule_exchange;
pub mod rule_persistence_property;
pub mod rule_promoter;
pub mod rule_promotion_safety_property;
pub mod rule_review;
pub mod rule_storage;
pub mod rule_validation_property;
pub mod rule_validator;
pub mod scope_config;
pub mod scope_isolation_property;
pub mod scope_precedence_property;

// Re-export public types
pub use analytics_engine::{AnalyticsEngine, AnalyticsInsights, RuleMetrics};
pub use conflict_resolver::ConflictResolver;
pub use decision_logger::{DecisionLogger, DecisionStatistics};
pub use drift_detector::{DriftDetectionConfig, DriftDetector, DriftStatistics};
pub use error::{LearningError, Result};
pub use intent_tracker::{
    ArchitecturalDecision, ArchitecturalEvolution, ArchitecturalSummary, DriftDetection,
    IntentTracker,
};
pub use intent_tracking_integration::{ArchitecturalReport, IntentTrackingIntegration};
pub use manager::LearningManager;
pub use models::{
    Decision, DecisionContext, LearnedPattern, LearningConfig, PatternExample, Rule, RuleScope,
    RuleSource,
};
pub use pattern_capturer::{PatternAnalysis, PatternCapturer};
pub use pattern_validator::{PatternValidator, ValidationResult, ValidationStatistics};
pub use rule_application::{GenerationContext, RuleApplicationEngine, RuleApplicationResult};
pub use rule_exchange::{ExportMetadata, RuleExport, RuleExporter, RuleImporter};
pub use rule_promoter::{
    PromotionHistoryEntry, PromotionMetadata, RulePromoter, RuleReview, VersionChanges,
};
pub use rule_review::{
    compare_rules, ReviewComment, ReviewInfo, ReviewStatus, RuleComparison, RuleReviewManager,
};
pub use rule_storage::RuleStorage;
pub use rule_validator::{RuleValidator, ValidationReport};
pub use scope_config::{ScopeConfiguration, ScopeConfigurationLoader, ScopeFilter};
