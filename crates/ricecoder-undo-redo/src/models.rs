//! Data models and validators

use crate::change::Change;
use crate::error::UndoRedoError;

/// Validates changes for consistency and correctness
pub struct ChangeValidator;

impl ChangeValidator {
    /// Validate a change for consistency
    pub fn validate_change(change: &Change) -> Result<(), UndoRedoError> {
        // Delegate to the Change's own validation method
        change.validate()
    }

    /// Validate a change and handle file-related errors gracefully
    pub fn validate_change_graceful(change: &Change) -> Result<(), UndoRedoError> {
        // Perform basic validation
        change.validate()?;

        // Additional graceful error handling for file operations
        // File not found errors are logged but don't fail validation
        if change.file_path.is_empty() {
            return Err(UndoRedoError::validation_error("file_path cannot be empty"));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::change::ChangeType;

    #[test]
    fn test_change_validator_valid_modify() {
        let change = Change::new(
            "test.txt",
            "before",
            "after",
            "Modify",
            ChangeType::Modify,
        )
        .unwrap();
        let result = ChangeValidator::validate_change(&change);
        assert!(result.is_ok());
    }

    #[test]
    fn test_change_validator_valid_create() {
        let change = Change::new(
            "test.txt",
            "",
            "content",
            "Create",
            ChangeType::Create,
        )
        .unwrap();
        let result = ChangeValidator::validate_change(&change);
        assert!(result.is_ok());
    }

    #[test]
    fn test_change_validator_valid_delete() {
        let change = Change::new(
            "test.txt",
            "content",
            "",
            "Delete",
            ChangeType::Delete,
        )
        .unwrap();
        let result = ChangeValidator::validate_change(&change);
        assert!(result.is_ok());
    }

    #[test]
    fn test_change_validator_graceful() {
        let change = Change::new(
            "test.txt",
            "before",
            "after",
            "Modify",
            ChangeType::Modify,
        )
        .unwrap();
        let result = ChangeValidator::validate_change_graceful(&change);
        assert!(result.is_ok());
    }
}
