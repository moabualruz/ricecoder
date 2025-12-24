//! Domain Aggregate Roots
//!
//! Aggregates represent clusters of domain objects treated as a single unit.
//! They enforce business invariants and emit domain events when they change.

use crate::domain::value_objects::{FilePath, EditPattern};
use crate::domain::events::DomainEvent;
use crate::domain::errors::{DomainError, DomainResult};

/// File edit aggregate representing a validated edit operation
#[derive(Debug, Clone)]
pub struct FileEdit {
    file_path: FilePath,
    old_pattern: EditPattern,
    new_content: String,
    dry_run: bool,
    events: Vec<DomainEvent>,
}

impl FileEdit {
    pub fn new(file_path: FilePath, old_pattern: EditPattern, new_content: String, dry_run: bool) -> DomainResult<Self> {
        if new_content.trim().is_empty() {
            return Err(DomainError::InvalidFileEdit("New content cannot be empty".to_string()));
        }
        
        let mut edit = FileEdit { file_path, old_pattern, new_content, dry_run, events: Vec::new() };
        edit.add_event(DomainEvent::FileEditValidated {
            file_path: edit.file_path.as_path().to_string_lossy().to_string(),
            pattern: edit.old_pattern.pattern().to_string(),
            is_regex: edit.old_pattern.is_regex(),
            dry_run: edit.dry_run,
        });
        Ok(edit)
    }
    
    pub fn file_path(&self) -> &FilePath { &self.file_path }
    pub fn old_pattern(&self) -> &EditPattern { &self.old_pattern }
    pub fn new_content(&self) -> &str { &self.new_content }
    pub fn is_dry_run(&self) -> bool { self.dry_run }
    pub fn events(&self) -> &[DomainEvent] { &self.events }
    pub fn take_events(&mut self) -> Vec<DomainEvent> { std::mem::take(&mut self.events) }
    
    pub fn validate_pattern_exists(&mut self, file_content: &str) -> DomainResult<()> {
        if !file_content.contains(self.old_pattern.pattern()) {
            return Err(DomainError::InvalidFileEdit(format!("Pattern '{}' not found", self.old_pattern.pattern())));
        }
        Ok(())
    }
    
    pub fn mark_executed(&mut self, matches_replaced: usize) {
        self.add_event(DomainEvent::FileEditExecuted {
            file_path: self.file_path.as_path().to_string_lossy().to_string(),
            pattern: self.old_pattern.pattern().to_string(),
            replacement: self.new_content.clone(),
            matches_replaced,
            was_dry_run: self.dry_run,
        });
    }
    
    fn add_event(&mut self, event: DomainEvent) { self.events.push(event); }
}

/// Search result aggregate with matches from a single file
#[derive(Debug, Clone)]
pub struct SearchResult {
    file_path: FilePath,
    matches: Vec<SearchMatch>,
    events: Vec<DomainEvent>,
}

/// A single search match
#[derive(Debug, Clone)]
pub struct SearchMatch {
    line_number: usize,
    column_start: usize,
    matched_text: String,
}

impl SearchMatch {
    pub fn new(line_number: usize, column_start: usize, matched_text: String) -> Self {
        SearchMatch { line_number, column_start, matched_text }
    }
    pub fn line_number(&self) -> usize { self.line_number }
    pub fn column_start(&self) -> usize { self.column_start }
    pub fn matched_text(&self) -> &str { &self.matched_text }
}

impl SearchResult {
    pub fn new(file_path: FilePath, matches: Vec<SearchMatch>) -> Self {
        let total_matches = matches.len();
        let mut result = SearchResult { file_path, matches, events: Vec::new() };
        result.add_event(DomainEvent::SearchExecuted {
            file_path: result.file_path.as_path().to_string_lossy().to_string(),
            matches_found: total_matches,
        });
        result
    }
    
    pub fn file_path(&self) -> &FilePath { &self.file_path }
    pub fn matches(&self) -> &[SearchMatch] { &self.matches }
    pub fn total_matches(&self) -> usize { self.matches.len() }
    pub fn has_matches(&self) -> bool { !self.matches.is_empty() }
    pub fn events(&self) -> &[DomainEvent] { &self.events }
    pub fn take_events(&mut self) -> Vec<DomainEvent> { std::mem::take(&mut self.events) }
    fn add_event(&mut self, event: DomainEvent) { self.events.push(event); }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_edit_creation() {
        let file_path = FilePath::new("src/main.rs").unwrap();
        let pattern = EditPattern::literal("hello").unwrap();
        let edit = FileEdit::new(file_path, pattern, "hi".to_string(), false).unwrap();
        assert_eq!(edit.new_content(), "hi");
        assert_eq!(edit.events().len(), 1);
    }
    
    #[test]
    fn test_search_result_creation() {
        let file_path = FilePath::new("src/main.rs").unwrap();
        let result = SearchResult::new(file_path, vec![SearchMatch::new(1, 0, "test".to_string())]);
        assert_eq!(result.total_matches(), 1);
    }
}
