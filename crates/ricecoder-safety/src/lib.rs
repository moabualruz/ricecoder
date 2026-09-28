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
//! use ricecoder_safety::constraints::ValidationContext;
//! use ricecoder_safety::risk::RiskContext;
//!
//! #[tokio::main]
//! async fn main() -> ricecoder_safety::SafetyResult<()> {
//!     let validator = SafetyValidator::new();
//!     validator
//!         .add_constraint(SecurityConstraint::max_file_size(10 * 1024 * 1024))
//!         .await?;
//!     let operation = ValidationContext::new().with_file_size(1024);
//!     let _result = validator.validate_operation(&operation).await?;
//!
//!     let scorer = RiskScorer::new();
//!     let context = RiskContext::default();
//!     let _risk_score = scorer.score_action("read_file", &context)?;
//!     Ok(())
//! }
//! ```

pub mod constraints;
pub mod di;
pub mod error;
pub mod monitoring;
pub mod risk;
pub mod validation;

// Re-export commonly used types
pub use constraints::{ConstraintResult, ConstraintType, SecurityConstraint};
pub use error::{SafetyError, SafetyResult};
pub use monitoring::{AlertLevel, SafetyMetrics, SafetyMonitor};
pub use risk::{RiskFactors, RiskLevel, RiskScore, RiskScorer};
pub use validation::{ApprovalGate, ApprovalRequest, SafetyValidator, ValidationResult};
