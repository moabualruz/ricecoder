//! Domain Events
//!
//! Simple domain events that capture when aggregates change state.
//! Event publishing is handled by the application layer.

/// Domain events emitted by aggregates when they change state
#[derive(Debug, Clone, PartialEq)]
pub enum DomainEvent {
    /// A file edit was validated and is ready for execution
    FileEditValidated {
        file_path: String,
        pattern: String,
        is_regex: bool,
        dry_run: bool,
    },
    
    /// A file edit was successfully executed
    FileEditExecuted {
        file_path: String,
        pattern: String,
        replacement: String,
        matches_replaced: usize,
        was_dry_run: bool,
    },
    
    /// A search operation was executed
    SearchExecuted {
        file_path: String,
        matches_found: usize,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_event_creation() {
        let event = DomainEvent::FileEditValidated {
            file_path: "test.rs".to_string(),
            pattern: "hello".to_string(),
            is_regex: false,
            dry_run: true,
        };
        
        if let DomainEvent::FileEditValidated { pattern, is_regex, dry_run, .. } = event {
            assert_eq!(pattern, "hello");
            assert!(!is_regex);
            assert!(dry_run);
        } else {
            panic!("Expected FileEditValidated event");
        }
    }
    
    #[test]
    fn test_domain_event_types() {
        let events = vec![
            DomainEvent::FileEditValidated {
                file_path: "test.rs".to_string(),
                pattern: "hello".to_string(),
                is_regex: false,
                dry_run: true,
            },
            DomainEvent::FileEditExecuted {
                file_path: "test.rs".to_string(),
                pattern: "hello".to_string(),
                replacement: "hi".to_string(),
                matches_replaced: 1,
                was_dry_run: false,
            },
            DomainEvent::SearchExecuted {
                file_path: "test.rs".to_string(),
                matches_found: 1,
            },
        ];
        
        assert_eq!(events.len(), 3);
    }
}
