//! Search functionality for help content

use crate::{HelpCategory, HelpContent, HelpItem, Result};

/// Search state and functionality for help content
#[derive(Debug, Clone)]
pub struct HelpSearch {
    query: String,
    results: Vec<SearchResult>,
    selected_index: usize,
    case_sensitive: bool,
}

/// A search result with context
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub category_name: String,
    pub item_title: String,
    pub item_content: String,
    pub match_type: MatchType,
    pub match_positions: Vec<(usize, usize)>, // (start, end) positions of matches
}

/// Type of match found
#[derive(Debug, Clone, PartialEq)]
pub enum MatchType {
    Title,
    Content,
    Keyword,
}

impl HelpSearch {
    /// Create a new search instance
    pub fn new() -> Self {
        Self {
            query: String::new(),
            results: Vec::new(),
            selected_index: 0,
            case_sensitive: false,
        }
    }
    
    /// Set case sensitivity
    pub fn set_case_sensitive(&mut self, case_sensitive: bool) {
        self.case_sensitive = case_sensitive;
    }
    
    /// Get current query
    pub fn query(&self) -> &str {
        &self.query
    }
    
    /// Get search results
    pub fn results(&self) -> &[SearchResult] {
        &self.results
    }
    
    /// Get selected result index
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }
    
    /// Get selected result
    pub fn selected_result(&self) -> Option<&SearchResult> {
        self.results.get(self.selected_index)
    }
    
    /// Move selection up
    pub fn select_previous(&mut self) {
        if !self.results.is_empty() && self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }
    
    /// Move selection down
    pub fn select_next(&mut self) {
        if self.selected_index + 1 < self.results.len() {
            self.selected_index += 1;
        }
    }
    
    /// Clear search
    pub fn clear(&mut self) {
        self.query.clear();
        self.results.clear();
        self.selected_index = 0;
    }
    
    /// Update search query and perform search
    pub fn search(&mut self, content: &HelpContent, query: &str) -> Result<()> {
        self.query = query.to_string();
        self.results.clear();
        self.selected_index = 0;
        
        if query.trim().is_empty() {
            return Ok(());
        }
        
        let search_query = if self.case_sensitive {
            query.to_string()
        } else {
            query.to_lowercase()
        };
        
        // Search through all categories and items
        for category in &content.categories {
            for item in &category.items {
                if let Some(result) = self.search_item(category, item, &search_query) {
                    self.results.push(result);
                }
            }
        }
        
        // Sort results by relevance (title matches first, then content, then keywords)
        self.results.sort_by(|a, b| {
            use std::cmp::Ordering;
            match (a.match_type.clone(), b.match_type.clone()) {
                (MatchType::Title, MatchType::Title) => Ordering::Equal,
                (MatchType::Title, _) => Ordering::Less,
                (_, MatchType::Title) => Ordering::Greater,
                (MatchType::Content, MatchType::Content) => Ordering::Equal,
                (MatchType::Content, MatchType::Keyword) => Ordering::Less,
                (MatchType::Keyword, MatchType::Content) => Ordering::Greater,
                (MatchType::Keyword, MatchType::Keyword) => Ordering::Equal,
            }
        });
        
        Ok(())
    }
    
    /// Search within a single item
    fn search_item(
        &self,
        category: &HelpCategory,
        item: &HelpItem,
        query: &str,
    ) -> Option<SearchResult> {
        let title_text = if self.case_sensitive {
            item.title.clone()
        } else {
            item.title.to_lowercase()
        };
        
        let content_text = if self.case_sensitive {
            item.content.clone()
        } else {
            item.content.to_lowercase()
        };
        
        // Check title match
        if let Some(positions) = self.find_matches(&title_text, query) {
            return Some(SearchResult {
                category_name: category.name.clone(),
                item_title: item.title.clone(),
                item_content: item.content.clone(),
                match_type: MatchType::Title,
                match_positions: positions,
            });
        }
        
        // Check content match
        if let Some(positions) = self.find_matches(&content_text, query) {
            return Some(SearchResult {
                category_name: category.name.clone(),
                item_title: item.title.clone(),
                item_content: item.content.clone(),
                match_type: MatchType::Content,
                match_positions: positions,
            });
        }
        
        // Check keyword matches
        for keyword in &item.keywords {
            let keyword_text = if self.case_sensitive {
                keyword.clone()
            } else {
                keyword.to_lowercase()
            };
            
            if let Some(positions) = self.find_matches(&keyword_text, query) {
                return Some(SearchResult {
                    category_name: category.name.clone(),
                    item_title: item.title.clone(),
                    item_content: item.content.clone(),
                    match_type: MatchType::Keyword,
                    match_positions: positions,
                });
            }
        }
        
        None
    }
    
    /// Find all match positions in text
    fn find_matches(&self, text: &str, query: &str) -> Option<Vec<(usize, usize)>> {
        let mut positions = Vec::new();
        let mut start = 0;
        
        while let Some(pos) = text[start..].find(query) {
            let absolute_pos = start + pos;
            positions.push((absolute_pos, absolute_pos + query.len()));
            start = absolute_pos + 1;
        }
        
        if positions.is_empty() {
            None
        } else {
            Some(positions)
        }
    }
    
    /// Get highlighted text with match positions
    pub fn highlight_matches(&self, text: &str, positions: &[(usize, usize)]) -> Vec<TextSegment> {
        if positions.is_empty() {
            return vec![TextSegment::Normal(text.to_string())];
        }
        
        let mut segments = Vec::new();
        let mut last_end = 0;
        
        for &(start, end) in positions {
            // Add text before match
            if start > last_end {
                segments.push(TextSegment::Normal(text[last_end..start].to_string()));
            }
            
            // Add highlighted match
            segments.push(TextSegment::Highlighted(text[start..end].to_string()));
            last_end = end;
        }
        
        // Add remaining text
        if last_end < text.len() {
            segments.push(TextSegment::Normal(text[last_end..].to_string()));
        }
        
        segments
    }
}

/// Text segment for rendering with highlighting
#[derive(Debug, Clone)]
pub enum TextSegment {
    Normal(String),
    Highlighted(String),
}

impl Default for HelpSearch {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HelpContent;

    #[test]
    fn test_search_creation() {
        let search = HelpSearch::new();
        assert_eq!(search.query(), "");
        assert_eq!(search.results().len(), 0);
        assert_eq!(search.selected_index(), 0);
    }

    #[test]
    fn test_search_basic() {
        let mut search = HelpSearch::new();
        let content = HelpContent::default_ricecoder_help();
        
        search.search(&content, "help").unwrap();
        
        assert!(!search.results().is_empty());
        assert_eq!(search.query(), "help");
    }

    #[test]
    fn test_search_case_insensitive() {
        let mut search = HelpSearch::new();
        let content = HelpContent::default_ricecoder_help();
        
        search.search(&content, "HELP").unwrap();
        
        assert!(!search.results().is_empty());
    }

    #[test]
    fn test_search_case_sensitive() {
        let mut search = HelpSearch::new();
        search.set_case_sensitive(true);
        let content = HelpContent::default_ricecoder_help();
        
        search.search(&content, "help").unwrap();
        let case_insensitive_count = search.results().len();
        
        search.search(&content, "HELP").unwrap();
        let case_sensitive_count = search.results().len();
        
        // Case sensitive should find fewer results
        assert!(case_sensitive_count <= case_insensitive_count);
    }

    #[test]
    fn test_search_navigation() {
        let mut search = HelpSearch::new();
        let content = HelpContent::default_ricecoder_help();
        
        search.search(&content, "help").unwrap();
        
        if search.results().len() > 1 {
            assert_eq!(search.selected_index(), 0);
            
            search.select_next();
            assert_eq!(search.selected_index(), 1);
            
            search.select_previous();
            assert_eq!(search.selected_index(), 0);
        }
    }

    #[test]
    fn test_search_clear() {
        let mut search = HelpSearch::new();
        let content = HelpContent::default_ricecoder_help();
        
        search.search(&content, "help").unwrap();
        assert!(!search.results().is_empty());
        
        search.clear();
        assert_eq!(search.query(), "");
        assert_eq!(search.results().len(), 0);
        assert_eq!(search.selected_index(), 0);
    }

    #[test]
    fn test_find_matches() {
        let search = HelpSearch::new();
        let positions = search.find_matches("hello world hello", "hello").unwrap();
        
        assert_eq!(positions.len(), 2);
        assert_eq!(positions[0], (0, 5));
        assert_eq!(positions[1], (12, 17));
    }

    #[test]
    fn test_highlight_matches() {
        let search = HelpSearch::new();
        let positions = vec![(0, 5), (12, 17)];
        let segments = search.highlight_matches("hello world hello", &positions);
        
        assert_eq!(segments.len(), 3); // highlighted, normal, highlighted
        match &segments[0] {
            TextSegment::Highlighted(text) => assert_eq!(text, "hello"),
            _ => panic!("Expected highlighted segment"),
        }
        match &segments[1] {
            TextSegment::Normal(text) => assert_eq!(text, " world "),
            _ => panic!("Expected normal segment"),
        }
        match &segments[2] {
            TextSegment::Highlighted(text) => assert_eq!(text, "hello"),
            _ => panic!("Expected highlighted segment"),
        }
    }
}