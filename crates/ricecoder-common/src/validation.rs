//! Validation traits and common validators
//!
//! Provides a unified validation interface to replace duplicate validation
//! functions scattered across the codebase.

use std::ops::RangeInclusive;
use thiserror::Error;

/// Validation error with context
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Invalid value for {field}: {message}")]
    InvalidValue { field: String, message: String },

    #[error("Value out of range for {field}: expected {expected}, got {actual}")]
    OutOfRange {
        field: String,
        expected: String,
        actual: String,
    },

    #[error("Required field missing: {field}")]
    Required { field: String },

    #[error("Format error for {field}: {message}")]
    Format { field: String, message: String },

    #[error("Multiple validation errors: {0:?}")]
    Multiple(Vec<ValidationError>),
}

/// Trait for types that can be validated
pub trait Validatable {
    /// Validate the instance, returning Ok(()) if valid or a ValidationError if invalid
    fn validate(&self) -> Result<(), ValidationError>;

    /// Check if the instance is valid without returning the error details
    fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }
}

/// Trait for validators that can check values
pub trait Validator<T> {
    /// Validate a value
    fn validate(&self, value: &T) -> Result<(), ValidationError>;
}

/// Port number validator (common pattern found in multiple crates)
pub struct PortValidator;

impl Validator<u16> for PortValidator {
    fn validate(&self, value: &u16) -> Result<(), ValidationError> {
        if *value == 0 {
            return Err(ValidationError::InvalidValue {
                field: "port".to_string(),
                message: "Port cannot be 0".to_string(),
            });
        }
        Ok(())
    }
}

/// Timeout validator with configurable range
pub struct TimeoutValidator {
    range: RangeInclusive<u64>,
}

impl TimeoutValidator {
    pub fn new(min: u64, max: u64) -> Self {
        Self { range: min..=max }
    }

    pub fn default_range() -> Self {
        Self::new(1, 3600) // 1 second to 1 hour
    }
}

impl Validator<u64> for TimeoutValidator {
    fn validate(&self, value: &u64) -> Result<(), ValidationError> {
        if !self.range.contains(value) {
            return Err(ValidationError::OutOfRange {
                field: "timeout".to_string(),
                expected: format!("{:?}", self.range),
                actual: value.to_string(),
            });
        }
        Ok(())
    }
}

/// Non-empty string validator
pub struct NonEmptyStringValidator {
    field_name: String,
}

impl NonEmptyStringValidator {
    pub fn new(field_name: impl Into<String>) -> Self {
        Self {
            field_name: field_name.into(),
        }
    }
}

impl Validator<String> for NonEmptyStringValidator {
    fn validate(&self, value: &String) -> Result<(), ValidationError> {
        if value.trim().is_empty() {
            return Err(ValidationError::Required {
                field: self.field_name.clone(),
            });
        }
        Ok(())
    }
}

impl Validator<&str> for NonEmptyStringValidator {
    fn validate(&self, value: &&str) -> Result<(), ValidationError> {
        if value.trim().is_empty() {
            return Err(ValidationError::Required {
                field: self.field_name.clone(),
            });
        }
        Ok(())
    }
}

/// URL validator
pub struct UrlValidator {
    field_name: String,
}

impl UrlValidator {
    pub fn new(field_name: impl Into<String>) -> Self {
        Self {
            field_name: field_name.into(),
        }
    }
}

impl Validator<String> for UrlValidator {
    fn validate(&self, value: &String) -> Result<(), ValidationError> {
        if !value.starts_with("http://") && !value.starts_with("https://") {
            return Err(ValidationError::Format {
                field: self.field_name.clone(),
                message: "URL must start with http:// or https://".to_string(),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_validator() {
        let validator = PortValidator;
        assert!(validator.validate(&80).is_ok());
        assert!(validator.validate(&443).is_ok());
        assert!(validator.validate(&0).is_err());
    }

    #[test]
    fn test_timeout_validator() {
        let validator = TimeoutValidator::default_range();
        assert!(validator.validate(&60).is_ok());
        assert!(validator.validate(&0).is_err());
        assert!(validator.validate(&3601).is_err());
    }

    #[test]
    fn test_non_empty_string_validator() {
        let validator = NonEmptyStringValidator::new("name");
        assert!(validator.validate(&"hello".to_string()).is_ok());
        assert!(validator.validate(&"".to_string()).is_err());
        assert!(validator.validate(&"   ".to_string()).is_err());
    }
}
