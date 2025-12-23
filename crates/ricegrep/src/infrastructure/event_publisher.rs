//! Tracing-based EventPublisher Implementation
//!
//! Implements the `EventPublisher` trait using the `tracing` crate for logging.

use crate::application::EventPublisher;
use crate::domain::DomainEvent;

/// Tracing-based implementation of `EventPublisher`
///
/// Publishes domain events by logging them using the `tracing` crate.
/// This provides observability into domain operations without requiring
/// a full event store implementation.
///
/// # Event Levels
/// - `FileEditValidated` → INFO
/// - `FileEditExecuted` → INFO  
/// - `SearchExecuted` → DEBUG (high volume)
#[derive(Debug, Clone, Default)]
pub struct TracingEventPublisher {
    /// Whether to include detailed event data in logs
    verbose: bool,
}

impl TracingEventPublisher {
    /// Create a new tracing event publisher
    pub fn new() -> Self {
        TracingEventPublisher { verbose: false }
    }
    
    /// Create a verbose publisher that logs full event details
    pub fn verbose() -> Self {
        TracingEventPublisher { verbose: true }
    }
}

impl EventPublisher for TracingEventPublisher {
    fn publish(&self, event: &DomainEvent) {
        match event {
            DomainEvent::FileEditValidated { file_path, pattern, is_regex, dry_run } => {
                if self.verbose {
                    tracing::info!(
                        event = "FileEditValidated",
                        file_path = %file_path,
                        pattern = %pattern,
                        is_regex = %is_regex,
                        dry_run = %dry_run,
                        "File edit validated"
                    );
                } else {
                    tracing::info!(
                        event = "FileEditValidated",
                        file_path = %file_path,
                        "File edit validated"
                    );
                }
            }
            DomainEvent::FileEditExecuted { file_path, pattern, replacement, matches_replaced, was_dry_run } => {
                tracing::info!(
                    event = "FileEditExecuted",
                    file_path = %file_path,
                    pattern = %pattern,
                    replacement = %replacement,
                    matches_replaced = %matches_replaced,
                    was_dry_run = %was_dry_run,
                    "File edit executed"
                );
            }
            DomainEvent::SearchExecuted { file_path, matches_found } => {
                tracing::debug!(
                    event = "SearchExecuted",
                    file_path = %file_path,
                    matches_found = %matches_found,
                    "Search executed"
                );
            }
        }
    }
}

/// In-memory event publisher for testing
///
/// Collects all published events in a vector for assertions.
#[derive(Debug, Default)]
pub struct InMemoryEventPublisher {
    events: std::sync::Mutex<Vec<DomainEvent>>,
}

impl InMemoryEventPublisher {
    /// Create a new in-memory event publisher
    pub fn new() -> Self {
        InMemoryEventPublisher {
            events: std::sync::Mutex::new(Vec::new()),
        }
    }
    
    /// Get the number of published events
    pub fn event_count(&self) -> usize {
        self.events.lock().map(|e| e.len()).unwrap_or(0)
    }
    
    /// Get all published events
    pub fn events(&self) -> Vec<DomainEvent> {
        self.events.lock().map(|e| e.clone()).unwrap_or_default()
    }
    
    /// Clear all events
    pub fn clear(&self) {
        if let Ok(mut events) = self.events.lock() {
            events.clear();
        }
    }
}

impl EventPublisher for InMemoryEventPublisher {
    fn publish(&self, event: &DomainEvent) {
        if let Ok(mut events) = self.events.lock() {
            events.push(event.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracing_publisher_creation() {
        let publisher = TracingEventPublisher::new();
        assert!(!publisher.verbose);
        
        let verbose = TracingEventPublisher::verbose();
        assert!(verbose.verbose);
    }

    #[test]
    fn test_tracing_publisher_publish() {
        // This test just ensures no panics - actual logging is tested via tracing-test
        let publisher = TracingEventPublisher::new();
        
        publisher.publish(&DomainEvent::FileEditValidated {
            file_path: "test.rs".to_string(),
            pattern: "hello".to_string(),
            is_regex: false,
            dry_run: true,
        });
        
        publisher.publish(&DomainEvent::FileEditExecuted {
            file_path: "test.rs".to_string(),
            pattern: "hello".to_string(),
            replacement: "world".to_string(),
            matches_replaced: 5,
            was_dry_run: false,
        });
        
        publisher.publish(&DomainEvent::SearchExecuted {
            file_path: "test.rs".to_string(),
            matches_found: 10,
        });
    }

    #[test]
    fn test_in_memory_publisher() {
        let publisher = InMemoryEventPublisher::new();
        
        assert_eq!(publisher.event_count(), 0);
        
        publisher.publish(&DomainEvent::FileEditExecuted {
            file_path: "test.rs".to_string(),
            pattern: "old".to_string(),
            replacement: "new".to_string(),
            matches_replaced: 1,
            was_dry_run: false,
        });
        
        assert_eq!(publisher.event_count(), 1);
        
        let events = publisher.events();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], DomainEvent::FileEditExecuted { .. }));
    }

    #[test]
    fn test_in_memory_publisher_clear() {
        let publisher = InMemoryEventPublisher::new();
        
        publisher.publish(&DomainEvent::SearchExecuted {
            file_path: "test.rs".to_string(),
            matches_found: 5,
        });
        
        assert_eq!(publisher.event_count(), 1);
        
        publisher.clear();
        
        assert_eq!(publisher.event_count(), 0);
    }

    #[test]
    fn test_in_memory_publisher_batch() {
        let publisher = InMemoryEventPublisher::new();
        
        let events = vec![
            DomainEvent::FileEditValidated {
                file_path: "a.rs".to_string(),
                pattern: "foo".to_string(),
                is_regex: false,
                dry_run: false,
            },
            DomainEvent::FileEditExecuted {
                file_path: "a.rs".to_string(),
                pattern: "foo".to_string(),
                replacement: "bar".to_string(),
                matches_replaced: 1,
                was_dry_run: false,
            },
            DomainEvent::SearchExecuted {
                file_path: "b.rs".to_string(),
                matches_found: 3,
            },
        ];
        
        publisher.publish_batch(&events);
        
        assert_eq!(publisher.event_count(), 3);
    }
}
