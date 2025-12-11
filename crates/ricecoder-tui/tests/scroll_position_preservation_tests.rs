//! Property-based tests for scroll position preservation in ScrollViewWidget
//!
//! These tests verify that scroll positions are preserved correctly during
//! terminal resize operations, as specified in the requirements.

use proptest::prelude::*;
use ricecoder_tui::scrollview_widget::{ScrollViewWidget, ScrollState};

/// **Feature: ricecoder-tui-improvement, Property 10: Scroll Position Preservation**
/// **Validates: Requirements 8.4**
///
/// *For any* scroll position and terminal resize, the relative scroll position 
/// should be preserved within a reasonable tolerance.
#[test]
fn scroll_position_preservation() {
    proptest!(ProptestConfig::with_cases(100), |(
        content_height in 100usize..1000,
        scroll_pos in 0usize..100,
        old_height in 20u16..50,
        new_height in 20u16..50
    )| {
        prop_assume!(scroll_pos < content_height);
        
        let mut scroll = ScrollState::new(content_height, old_height as usize);
        scroll.set_position(scroll_pos);
        
        let relative_pos = scroll_pos as f64 / content_height as f64;
        
        scroll.handle_resize(new_height as usize);
        
        let new_relative = scroll.position() as f64 / content_height as f64;
        
        prop_assert!((relative_pos - new_relative).abs() < 0.1);
    });
}