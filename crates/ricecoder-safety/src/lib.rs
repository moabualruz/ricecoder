//! RiceCoder Safety and Security Constraints
//!
//! This crate provides enterprise-grade security constraints, risk analysis,
//! and safety validation for RiceCoder operations. It ensures secure execution
//! of AI workflows while maintaining compliance with enterprise security policies.
//!
//! ## Features
//!
//! - **Security Constraints**: Configurable security policies and validation rules
//! - **Risk Analysis**: Dynamic risk scoring for operations and data
//! - **Safety Validation**: Pre-execution safety checks and approval gates
//! - **Compliance Monitoring**: Enterprise security compliance validation
//! - **Audit Integration**: Seamless integration with activity logging
//!
//! ## Architecture
//!
//! The safety system operates at multiple levels:
//!
//! - **Policy Layer**: Defines security constraints and risk thresholds
//! - **Validation Layer**: Checks operations against security policies
//! - **Risk Assessment Layer**: Analyzes and scores operational risks
//! - **Approval Layer**: Implements human-in-the-loop approval processes
//! - **Monitoring Layer**: Continuous security monitoring and alerting
//!
//! ## Usage
//!
//! ```rust
//! use ricecoder_safety::{SafetyValidator, RiskScorer, SecurityConstraint};
//!
//! // Create a safety validator
//! let validator = SafetyValidator::new();
//!
//! // Add security constraints
//! validator.add_constraint(SecurityConstraint::max_file_size(10 * 1024 * 1024));
//!
//! // Validate an operation
//! let result = validator.validate_operation(&operation).await;
//!
//! // Score risk for an action
//! let risk_score = RiskScorer::score_action(&action, &context);
//! ```

pub mod constraints;
pub mod error;
pub mod risk;
pub mod validation;
pub mod monitoring;

// Re-export commonly used types
pub use constraints::{SecurityConstraint, ConstraintType, ConstraintResult};
pub use error::{SafetyError, SafetyResult};
pub use risk::{RiskScorer, RiskLevel, RiskScore, RiskFactors};
pub use validation::{SafetyValidator, ValidationResult, ApprovalGate};
pub use monitoring::{SafetyMonitor, SafetyMetrics, AlertLevel};