//! Edit File Use Case
//!
//! Orchestrates file editing operations: find pattern, replace, write back.
//! Supports both dry-run (preview) and actual replacement modes.

use crate::application::{AppResult, AppError, FileRepository, EventPublisher};
use crate::domain::{FilePath, EditPattern, FileEdit, DomainEvent};

/// Request for editing a file
#[derive(Debug, Clone)]
pub struct EditFileRequest {
    /// Path to the file to edit
    pub file_path: String,
    /// Pattern to find (literal or regex)
    pub pattern: String,
    /// Replacement text
    pub replacement: String,
    /// Whether the pattern is a regex
    pub is_regex: bool,
    /// Whether to preview without writing
    pub dry_run: bool,
}

/// Response from editing a file
#[derive(Debug, Clone)]
pub struct EditFileResponse {
    /// Path to the edited file
    pub file_path: String,
    /// Number of matches replaced
    pub matches_replaced: usize,
    /// Preview of changes (if dry_run)
    pub preview: Option<String>,
    /// Whether this was a dry run
    pub was_dry_run: bool,
}

/// Use case for editing files with find/replace
///
/// # Example
/// ```ignore
/// let use_case = EditFileUseCase::new(file_repo, event_publisher);
/// let request = EditFileRequest {
///     file_path: "src/main.rs".to_string(),
///     pattern: "old_name".to_string(),
///     replacement: "new_name".to_string(),
///     is_regex: false,
///     dry_run: true,
/// };
/// let response = use_case.execute(request)?;
/// ```
pub struct EditFileUseCase<F: FileRepository, E: EventPublisher> {
    file_repo: F,
    event_publisher: E,
}

impl<F: FileRepository, E: EventPublisher> EditFileUseCase<F, E> {
    /// Create a new edit file use case
    pub fn new(file_repo: F, event_publisher: E) -> Self {
        EditFileUseCase {
            file_repo,
            event_publisher,
        }
    }

    /// Execute the edit file use case
    pub fn execute(&self, request: EditFileRequest) -> AppResult<EditFileResponse> {
        // 1. Validate inputs and create domain objects
        let file_path = FilePath::new(&request.file_path)
            .map_err(|e| AppError::Validation { message: e.to_string() })?;
        
        let pattern = EditPattern::new(&request.pattern, request.is_regex)
            .map_err(|e| AppError::Validation { message: e.to_string() })?;
        
        // 2. Read file content
        let content = self.file_repo.read(&file_path)?;
        
        // 3. Create FileEdit aggregate and validate
        let mut file_edit = FileEdit::new(
            file_path.clone(),
            pattern.clone(),
            request.replacement.clone(),
            request.dry_run,
        ).map_err(|e| AppError::Validation { message: e.to_string() })?;
        
        // 4. Validate pattern exists in file
        // For regex patterns, we check using regex; for literal patterns, use domain validation
        if request.is_regex {
            match regex::Regex::new(&request.pattern) {
                Ok(re) if !re.is_match(&content) => {
                    return Err(AppError::Validation { 
                        message: format!("Pattern '{}' not found", request.pattern) 
                    });
                }
                Err(e) => {
                    return Err(AppError::Validation { 
                        message: format!("Invalid regex: {}", e) 
                    });
                }
                _ => {} // Pattern matches
            }
        } else {
            file_edit.validate_pattern_exists(&content)
                .map_err(|e| AppError::Validation { message: e.to_string() })?;
        }
        
        // 5. Publish validation event
        self.event_publisher.publish(&DomainEvent::FileEditValidated {
            file_path: request.file_path.clone(),
            pattern: request.pattern.clone(),
            is_regex: request.is_regex,
            dry_run: request.dry_run,
        });
        
        // 6. Execute replacement
        let (new_content, match_count) = if request.is_regex {
            // Regex replacement
            match regex::Regex::new(&request.pattern) {
                Ok(re) => {
                    let new = re.replace_all(&content, &request.replacement).to_string();
                    let count = re.find_iter(&content).count();
                    (new, count)
                }
                Err(_) => (content.clone(), 0),
            }
        } else {
            // Literal replacement
            let count = content.matches(&request.pattern).count();
            let new = content.replace(&request.pattern, &request.replacement);
            (new, count)
        };
        
        // 7. Mark executed in domain
        file_edit.mark_executed(match_count);
        
        // 8. Write if not dry run
        if !request.dry_run && match_count > 0 {
            self.file_repo.write(&file_path, &new_content)?;
        }
        
        // 9. Publish execution event
        self.event_publisher.publish(&DomainEvent::FileEditExecuted {
            file_path: request.file_path.clone(),
            pattern: request.pattern.clone(),
            replacement: request.replacement.clone(),
            matches_replaced: match_count,
            was_dry_run: request.dry_run,
        });
        
        // 10. Return response
        Ok(EditFileResponse {
            file_path: request.file_path,
            matches_replaced: match_count,
            preview: if request.dry_run { Some(new_content) } else { None },
            was_dry_run: request.dry_run,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;

    // Test doubles
    struct TestFileRepo {
        files: RefCell<HashMap<String, String>>,
    }

    impl TestFileRepo {
        fn new() -> Self {
            TestFileRepo { files: RefCell::new(HashMap::new()) }
        }
        
        fn with_file(self, path: &str, content: &str) -> Self {
            self.files.borrow_mut().insert(path.to_string(), content.to_string());
            self
        }
        
        fn get_content(&self, path: &str) -> Option<String> {
            self.files.borrow().get(path).cloned()
        }
    }

    impl FileRepository for TestFileRepo {
        fn read(&self, path: &FilePath) -> AppResult<String> {
            let path_str = path.as_path().to_string_lossy().to_string();
            self.files.borrow().get(&path_str).cloned()
                .ok_or_else(|| AppError::Io {
                    operation: crate::application::IoOperation::Read,
                    path: path_str,
                    source: std::io::Error::new(std::io::ErrorKind::NotFound, "not found"),
                })
        }
        
        fn write(&self, path: &FilePath, content: &str) -> AppResult<()> {
            let path_str = path.as_path().to_string_lossy().to_string();
            self.files.borrow_mut().insert(path_str, content.to_string());
            Ok(())
        }
        
        fn exists(&self, path: &FilePath) -> bool {
            let path_str = path.as_path().to_string_lossy().to_string();
            self.files.borrow().contains_key(&path_str)
        }
        
        fn delete(&self, path: &FilePath) -> AppResult<()> {
            let path_str = path.as_path().to_string_lossy().to_string();
            self.files.borrow_mut().remove(&path_str);
            Ok(())
        }
        
        fn ensure_parent_dirs(&self, _path: &FilePath) -> AppResult<()> {
            Ok(())
        }
    }

    struct TestEventPublisher {
        events: RefCell<Vec<DomainEvent>>,
    }

    impl TestEventPublisher {
        fn new() -> Self {
            TestEventPublisher { events: RefCell::new(Vec::new()) }
        }
        
        fn event_count(&self) -> usize {
            self.events.borrow().len()
        }
    }

    impl EventPublisher for TestEventPublisher {
        fn publish(&self, event: &DomainEvent) {
            self.events.borrow_mut().push(event.clone());
        }
    }

    #[test]
    fn test_edit_file_dry_run() {
        let file_repo = TestFileRepo::new()
            .with_file("test.rs", "fn hello() { println!(\"hello\"); }");
        let event_pub = TestEventPublisher::new();
        
        let use_case = EditFileUseCase::new(file_repo, event_pub);
        
        let request = EditFileRequest {
            file_path: "test.rs".to_string(),
            pattern: "hello".to_string(),
            replacement: "world".to_string(),
            is_regex: false,
            dry_run: true,
        };
        
        let response = use_case.execute(request).unwrap();
        
        assert!(response.was_dry_run);
        assert_eq!(response.matches_replaced, 2); // "hello" appears twice
        assert!(response.preview.is_some());
        assert!(response.preview.unwrap().contains("world"));
    }

    #[test]
    fn test_edit_file_actual_write() {
        let file_repo = TestFileRepo::new()
            .with_file("test.rs", "fn old_name() {}");
        let event_pub = TestEventPublisher::new();
        
        let use_case = EditFileUseCase::new(file_repo, event_pub);
        
        let request = EditFileRequest {
            file_path: "test.rs".to_string(),
            pattern: "old_name".to_string(),
            replacement: "new_name".to_string(),
            is_regex: false,
            dry_run: false,
        };
        
        let response = use_case.execute(request).unwrap();
        
        assert!(!response.was_dry_run);
        assert_eq!(response.matches_replaced, 1);
        assert!(response.preview.is_none());
        
        // Verify file was actually written
        let updated = use_case.file_repo.get_content("test.rs").unwrap();
        assert!(updated.contains("new_name"));
        assert!(!updated.contains("old_name"));
    }

    #[test]
    fn test_edit_file_publishes_events() {
        let file_repo = TestFileRepo::new()
            .with_file("test.rs", "old content here");
        let event_pub = TestEventPublisher::new();
        
        let use_case = EditFileUseCase::new(file_repo, event_pub);
        
        let request = EditFileRequest {
            file_path: "test.rs".to_string(),
            pattern: "old".to_string(),
            replacement: "new".to_string(),
            is_regex: false,
            dry_run: false,
        };
        
        use_case.execute(request).unwrap();
        
        // Should publish validation and execution events
        assert_eq!(use_case.event_publisher.event_count(), 2);
    }

    #[test]
    fn test_edit_file_not_found() {
        let file_repo = TestFileRepo::new();
        let event_pub = TestEventPublisher::new();
        
        let use_case = EditFileUseCase::new(file_repo, event_pub);
        
        let request = EditFileRequest {
            file_path: "missing.rs".to_string(),
            pattern: "old".to_string(),
            replacement: "new".to_string(),
            is_regex: false,
            dry_run: false,
        };
        
        let result = use_case.execute(request);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_edit_file_invalid_path() {
        let file_repo = TestFileRepo::new();
        let event_pub = TestEventPublisher::new();
        
        let use_case = EditFileUseCase::new(file_repo, event_pub);
        
        let request = EditFileRequest {
            file_path: "".to_string(), // Invalid empty path
            pattern: "old".to_string(),
            replacement: "new".to_string(),
            is_regex: false,
            dry_run: false,
        };
        
        let result = use_case.execute(request);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Validation { .. }));
    }

    #[test]
    fn test_edit_file_regex_pattern() {
        let file_repo = TestFileRepo::new()
            .with_file("test.rs", "fn foo1() {} fn foo2() {} fn bar() {}");
        let event_pub = TestEventPublisher::new();
        
        let use_case = EditFileUseCase::new(file_repo, event_pub);
        
        let request = EditFileRequest {
            file_path: "test.rs".to_string(),
            pattern: r"foo\d".to_string(),
            replacement: "baz".to_string(),
            is_regex: true,
            dry_run: true,
        };
        
        let response = use_case.execute(request).unwrap();
        
        assert_eq!(response.matches_replaced, 2); // foo1, foo2
        let preview = response.preview.unwrap();
        assert!(preview.contains("baz"));
        assert!(!preview.contains("foo1"));
        assert!(!preview.contains("foo2"));
        assert!(preview.contains("bar")); // bar unchanged
    }
}
