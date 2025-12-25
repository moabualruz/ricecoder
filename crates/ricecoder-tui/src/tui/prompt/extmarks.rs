//! Extmark (inline annotation) management for prompt
//!
//! Extmarks are inline annotations that replace text ranges with virtual text.
//! Used for:
//! - File attachments: `[Image 1]`, `[File: name.pdf]`
//! - Agent mentions: `@build`, `@plan`
//! - Pasted text: `[Pasted ~50 lines]`
//!
//! # DDD Layer: Infrastructure
//! Manages visual annotations in the prompt input.

use std::collections::HashMap;
use ratatui::style::{Color, Style};

/// Style identifier for extmark types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExtmarkStyle {
    /// File attachment style
    File,
    /// Agent mention style
    Agent,
    /// Pasted text style
    Paste,
}

impl ExtmarkStyle {
    /// Get default color for style
    pub fn default_color(&self) -> Color {
        match self {
            Self::File => Color::Cyan,
            Self::Agent => Color::Magenta,
            Self::Paste => Color::Yellow,
        }
    }
    
    /// Get ratatui style
    pub fn to_style(&self) -> Style {
        Style::default().fg(self.default_color())
    }
}

/// A single extmark (inline annotation)
#[derive(Debug, Clone)]
pub struct Extmark {
    /// Unique identifier
    pub id: u32,
    /// Start position in text (byte offset)
    pub start: usize,
    /// End position in text (byte offset)
    pub end: usize,
    /// Virtual text to display
    pub virtual_text: String,
    /// Style for rendering
    pub style: ExtmarkStyle,
    /// Type ID for grouping
    pub type_id: u32,
}

impl Extmark {
    /// Create a new extmark
    pub fn new(id: u32, start: usize, end: usize, virtual_text: impl Into<String>, style: ExtmarkStyle) -> Self {
        Self {
            id,
            start,
            end,
            virtual_text: virtual_text.into(),
            style,
            type_id: 0,
        }
    }
    
    /// Set type ID
    pub fn with_type_id(mut self, type_id: u32) -> Self {
        self.type_id = type_id;
        self
    }
    
    /// Get the length of the extmark range
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }
    
    /// Check if extmark is empty
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
    
    /// Check if a position is within this extmark
    pub fn contains(&self, pos: usize) -> bool {
        pos >= self.start && pos < self.end
    }
    
    /// Shift positions by offset
    pub fn shift(&mut self, offset: isize) {
        if offset >= 0 {
            self.start = self.start.saturating_add(offset as usize);
            self.end = self.end.saturating_add(offset as usize);
        } else {
            let abs_offset = (-offset) as usize;
            self.start = self.start.saturating_sub(abs_offset);
            self.end = self.end.saturating_sub(abs_offset);
        }
    }
}

/// Manager for extmarks
#[derive(Debug, Default)]
pub struct ExtmarkManager {
    /// All extmarks by ID
    extmarks: HashMap<u32, Extmark>,
    /// Next ID to assign
    next_id: u32,
    /// Registered type IDs
    type_ids: HashMap<String, u32>,
    /// Next type ID
    next_type_id: u32,
}

impl ExtmarkManager {
    /// Create a new extmark manager
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Register a new type and get its ID
    pub fn register_type(&mut self, name: &str) -> u32 {
        if let Some(&id) = self.type_ids.get(name) {
            return id;
        }
        let id = self.next_type_id;
        self.next_type_id += 1;
        self.type_ids.insert(name.to_string(), id);
        id
    }
    
    /// Create a new extmark
    pub fn create(&mut self, start: usize, end: usize, virtual_text: impl Into<String>, style: ExtmarkStyle, type_id: u32) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        
        let extmark = Extmark::new(id, start, end, virtual_text, style).with_type_id(type_id);
        self.extmarks.insert(id, extmark);
        id
    }
    
    /// Get extmark by ID
    pub fn get(&self, id: u32) -> Option<&Extmark> {
        self.extmarks.get(&id)
    }
    
    /// Get mutable extmark by ID
    pub fn get_mut(&mut self, id: u32) -> Option<&mut Extmark> {
        self.extmarks.get_mut(&id)
    }
    
    /// Remove extmark by ID
    pub fn remove(&mut self, id: u32) -> Option<Extmark> {
        self.extmarks.remove(&id)
    }
    
    /// Clear all extmarks
    pub fn clear(&mut self) {
        self.extmarks.clear();
    }
    
    /// Get all extmarks
    pub fn all(&self) -> impl Iterator<Item = &Extmark> {
        self.extmarks.values()
    }
    
    /// Get all extmarks for a type ID
    pub fn get_by_type(&self, type_id: u32) -> Vec<&Extmark> {
        self.extmarks.values().filter(|e| e.type_id == type_id).collect()
    }
    
    /// Get all extmarks sorted by start position (descending)
    pub fn all_sorted_desc(&self) -> Vec<&Extmark> {
        let mut extmarks: Vec<_> = self.extmarks.values().collect();
        extmarks.sort_by(|a, b| b.start.cmp(&a.start));
        extmarks
    }
    
    /// Get extmark at position
    pub fn at_position(&self, pos: usize) -> Option<&Extmark> {
        self.extmarks.values().find(|e| e.contains(pos))
    }
    
    /// Update extmark positions after text change
    pub fn on_text_change(&mut self, change_start: usize, old_len: usize, new_len: usize) {
        let offset = new_len as isize - old_len as isize;
        
        // Remove extmarks that are within the changed region
        let to_remove: Vec<_> = self.extmarks
            .iter()
            .filter(|(_, e)| {
                // Remove if the extmark is completely within the deleted region
                e.start >= change_start && e.end <= change_start + old_len
            })
            .map(|(&id, _)| id)
            .collect();
        
        for id in to_remove {
            self.extmarks.remove(&id);
        }
        
        // Shift remaining extmarks
        for extmark in self.extmarks.values_mut() {
            if extmark.start >= change_start + old_len {
                // Extmark is after the change, shift it
                extmark.shift(offset);
            } else if extmark.end > change_start {
                // Extmark overlaps with change, adjust end
                if offset >= 0 {
                    extmark.end = extmark.end.saturating_add(offset as usize);
                } else {
                    extmark.end = extmark.end.saturating_sub((-offset) as usize);
                }
            }
        }
    }
    
    /// Sync extmarks with prompt parts (rebuild extmark to part mapping)
    pub fn sync_with_parts<F>(&self, mut callback: F)
    where
        F: FnMut(u32, usize, usize), // extmark_id, start, end
    {
        for extmark in self.extmarks.values() {
            callback(extmark.id, extmark.start, extmark.end);
        }
    }
}

/// Render extmarks in text
pub fn render_with_extmarks(text: &str, extmarks: &[&Extmark]) -> Vec<(String, Option<ExtmarkStyle>)> {
    let mut result = Vec::new();
    let mut last_end = 0;
    
    // Sort by start position
    let mut sorted: Vec<_> = extmarks.to_vec();
    sorted.sort_by(|a, b| a.start.cmp(&b.start));
    
    for extmark in sorted {
        // Add text before extmark
        if extmark.start > last_end {
            let plain = &text[last_end..extmark.start.min(text.len())];
            if !plain.is_empty() {
                result.push((plain.to_string(), None));
            }
        }
        
        // Add virtual text
        result.push((extmark.virtual_text.clone(), Some(extmark.style)));
        
        last_end = extmark.end;
    }
    
    // Add remaining text
    if last_end < text.len() {
        result.push((text[last_end..].to_string(), None));
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extmark_new() {
        let extmark = Extmark::new(1, 0, 10, "[Image 1]", ExtmarkStyle::File);
        assert_eq!(extmark.id, 1);
        assert_eq!(extmark.start, 0);
        assert_eq!(extmark.end, 10);
        assert_eq!(extmark.virtual_text, "[Image 1]");
    }
    
    #[test]
    fn test_extmark_contains() {
        let extmark = Extmark::new(1, 5, 15, "test", ExtmarkStyle::Paste);
        assert!(!extmark.contains(4));
        assert!(extmark.contains(5));
        assert!(extmark.contains(10));
        assert!(!extmark.contains(15));
    }
    
    #[test]
    fn test_extmark_shift() {
        let mut extmark = Extmark::new(1, 10, 20, "test", ExtmarkStyle::Agent);
        extmark.shift(5);
        assert_eq!(extmark.start, 15);
        assert_eq!(extmark.end, 25);
        
        extmark.shift(-10);
        assert_eq!(extmark.start, 5);
        assert_eq!(extmark.end, 15);
    }
    
    #[test]
    fn test_manager_create() {
        let mut manager = ExtmarkManager::new();
        let type_id = manager.register_type("prompt-part");
        let id = manager.create(0, 10, "[Image 1]", ExtmarkStyle::File, type_id);
        
        let extmark = manager.get(id).unwrap();
        assert_eq!(extmark.virtual_text, "[Image 1]");
        assert_eq!(extmark.type_id, type_id);
    }
    
    #[test]
    fn test_manager_clear() {
        let mut manager = ExtmarkManager::new();
        manager.create(0, 5, "a", ExtmarkStyle::File, 0);
        manager.create(5, 10, "b", ExtmarkStyle::Agent, 0);
        assert_eq!(manager.all().count(), 2);
        
        manager.clear();
        assert_eq!(manager.all().count(), 0);
    }
    
    #[test]
    fn test_render_with_extmarks() {
        let text = "hello world test";
        let extmarks = vec![
            Extmark::new(1, 0, 5, "[IMG]", ExtmarkStyle::File),
            Extmark::new(2, 6, 11, "@build", ExtmarkStyle::Agent),
        ];
        let refs: Vec<_> = extmarks.iter().collect();
        
        let result = render_with_extmarks(text, &refs);
        assert_eq!(result.len(), 4);
        assert_eq!(result[0].0, "[IMG]");
        assert_eq!(result[1].0, " ");
        assert_eq!(result[2].0, "@build");
        assert_eq!(result[3].0, " test");
    }
    
    #[test]
    fn test_on_text_change_insert() {
        let mut manager = ExtmarkManager::new();
        manager.create(10, 20, "test", ExtmarkStyle::Paste, 0);
        
        // Insert 5 characters at position 5
        manager.on_text_change(5, 0, 5);
        
        let extmark = manager.all().next().unwrap();
        assert_eq!(extmark.start, 15);
        assert_eq!(extmark.end, 25);
    }
    
    #[test]
    fn test_on_text_change_delete() {
        let mut manager = ExtmarkManager::new();
        manager.create(10, 20, "test", ExtmarkStyle::Paste, 0);
        
        // Delete 5 characters at position 5
        manager.on_text_change(5, 5, 0);
        
        let extmark = manager.all().next().unwrap();
        assert_eq!(extmark.start, 5);
        assert_eq!(extmark.end, 15);
    }
}
