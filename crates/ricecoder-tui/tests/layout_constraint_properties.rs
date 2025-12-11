//! Property-based tests for layout constraint satisfaction
//! Tests universal properties that should hold for layout calculations
//! Uses proptest for random test case generation
//! **Feature: ricecoder-tui-improvement, Property 6: Layout Constraint Satisfaction**
//! **Validates: Requirements 2.1, 2.2**

use proptest::prelude::*;
use ricecoder_tui::Layout;
use ricecoder_tui::layout::{LayoutConfig, DegradationLevel};

// ============================================================================
// Generators for Property Tests
// ============================================================================

/// Generate valid terminal sizes (minimum 40x10 to 200x50)
fn arb_terminal_size() -> impl Strategy<Value = (u16, u16)> {
    (40u16..=200, 10u16..=50)
}

/// Generate valid terminal sizes above minimum requirements (80x24 to 200x50)
fn arb_good_terminal_size() -> impl Strategy<Value = (u16, u16)> {
    (80u16..=200, 24u16..=50)
}

/// Generate layout configuration with reasonable values
fn arb_layout_config() -> impl Strategy<Value = LayoutConfig> {
    (
        0u16..=10,  // banner_height
        0u16..=40,  // sidebar_width
        any::<bool>(), // sidebar_enabled
        1u16..=5,   // input_height
        40u16..=100, // min_width
        10u16..=30,  // min_height
        10u16..=30,  // min_chat_width
        any::<bool>(), // auto_hide_sidebar
        any::<bool>(), // auto_reduce_banner
    ).prop_map(|(banner_height, sidebar_width, sidebar_enabled, input_height, min_width, min_height, min_chat_width, auto_hide_sidebar, auto_reduce_banner)| {
        LayoutConfig {
            banner_height,
            sidebar_width,
            sidebar_enabled,
            input_height,
            min_width,
            min_height,
            min_chat_width,
            auto_hide_sidebar,
            auto_reduce_banner,
        }
    })
}

/// Generate sequences of terminal size changes for resize testing
fn arb_resize_sequence() -> impl Strategy<Value = Vec<(u16, u16)>> {
    prop::collection::vec(arb_terminal_size(), 1..10)
}

// ============================================================================
// Property 6: Layout Constraint Satisfaction
// **Feature: ricecoder-tui-improvement, Property 6: Layout Constraint Satisfaction**
// **Validates: Requirements 2.1, 2.2**
// For any terminal size above minimum, the layout manager should produce 
// non-overlapping areas that fill the available space.
// ============================================================================

proptest! {
    #[test]
    fn prop_layout_areas_no_overlap(
        (width, height) in arb_good_terminal_size(),
        config in arb_layout_config()
    ) {
        // **Feature: ricecoder-tui-improvement, Property 6: Layout Constraint Satisfaction**
        // **Validates: Requirements 2.1, 2.2**
        
        // For any terminal size above minimum, areas should not overlap
        let layout = Layout::new(width, height);
        let areas = layout.calculate_areas(&config);
        
        // Validate that areas don't overlap
        assert!(layout.validate_areas(&areas).is_ok(), 
            "Layout areas should not overlap for size {}x{}", width, height);
    }

    #[test]
    fn prop_layout_areas_fill_terminal(
        (width, height) in arb_good_terminal_size(),
        config in arb_layout_config()
    ) {
        // **Feature: ricecoder-tui-improvement, Property 6: Layout Constraint Satisfaction**
        // **Validates: Requirements 2.1, 2.2**
        
        // For any terminal size, areas should collectively fill the terminal
        let layout = Layout::new(width, height);
        let areas = layout.calculate_areas(&config);
        
        // All areas should fit within terminal bounds
        
        if let Some(banner) = areas.banner {
            assert!(banner.x + banner.width <= width, "Banner should fit within terminal width");
            assert!(banner.y + banner.height <= height, "Banner should fit within terminal height");
        }
        
        if let Some(sidebar) = areas.sidebar {
            assert!(sidebar.x + sidebar.width <= width, "Sidebar should fit within terminal width");
            assert!(sidebar.y + sidebar.height <= height, "Sidebar should fit within terminal height");
        }
        
        assert!(areas.chat.x + areas.chat.width <= width, "Chat should fit within terminal width");
        assert!(areas.chat.y + areas.chat.height <= height, "Chat should fit within terminal height");
        
        assert!(areas.input.x + areas.input.width <= width, "Input should fit within terminal width");
        assert!(areas.input.y + areas.input.height <= height, "Input should fit within terminal height");
        
        assert!(areas.status.x + areas.status.width <= width, "Status should fit within terminal width");
        assert!(areas.status.y + areas.status.height <= height, "Status should fit within terminal height");
    }

    #[test]
    fn prop_layout_areas_minimum_sizes(
        (width, height) in arb_good_terminal_size(),
        config in arb_layout_config()
    ) {
        // **Feature: ricecoder-tui-improvement, Property 6: Layout Constraint Satisfaction**
        // **Validates: Requirements 2.1, 2.2**
        
        // For any terminal size, areas should have minimum viable sizes
        let layout = Layout::new(width, height);
        let areas = layout.calculate_areas(&config);
        
        // Chat area should have minimum width if sidebar is present
        if areas.sidebar.is_some() {
            assert!(areas.chat.width >= config.min_chat_width, 
                "Chat area should have minimum width when sidebar is present");
        }
        
        // Input area should have at least 1 line
        assert!(areas.input.height >= 1, "Input area should have at least 1 line");
        
        // Status bar should be exactly 1 line
        assert_eq!(areas.status.height, 1, "Status bar should be exactly 1 line");
        
        // All areas should have non-zero width
        if let Some(banner) = areas.banner {
            assert!(banner.width > 0, "Banner should have non-zero width");
        }
        if let Some(sidebar) = areas.sidebar {
            assert!(sidebar.width > 0, "Sidebar should have non-zero width");
        }
        assert!(areas.chat.width > 0, "Chat should have non-zero width");
        assert!(areas.input.width > 0, "Input should have non-zero width");
        assert!(areas.status.width > 0, "Status should have non-zero width");
    }

    #[test]
    fn prop_layout_graceful_degradation(
        (width, height) in arb_terminal_size(),
        config in arb_layout_config()
    ) {
        // **Feature: ricecoder-tui-improvement, Property 6: Layout Constraint Satisfaction**
        // **Validates: Requirements 2.1, 2.2**
        
        // For any terminal size, layout should degrade gracefully
        let layout = Layout::new(width, height);
        let degradation = layout.degradation_level(&config);
        let areas = layout.calculate_areas(&config);
        
        match degradation {
            DegradationLevel::TooSmall => {
                // Should still produce valid areas, even if minimal
                assert!(layout.validate_areas(&areas).is_ok(), 
                    "Even too-small terminals should produce valid areas");
            }
            DegradationLevel::Minimal => {
                // Should have no banner, no sidebar, minimal input
                assert!(areas.banner.is_none(), "Minimal layout should have no banner");
                assert!(areas.sidebar.is_none(), "Minimal layout should have no sidebar");
                assert!(areas.input.height <= 2, "Minimal layout should have small input");
            }
            DegradationLevel::ReduceBanner => {
                // Should have reduced banner height if present and configured
                if config.banner_height > 0 {
                    if let Some(banner) = areas.banner {
                        assert!(banner.height <= config.banner_height, 
                            "Reduced banner should be smaller than or equal to configured");
                    }
                } else {
                    // If banner_height is 0, there should be no banner
                    assert!(areas.banner.is_none(), "No banner should be present when banner_height is 0");
                }
            }
            DegradationLevel::HideSidebar => {
                // Should have no sidebar
                assert!(areas.sidebar.is_none(), "Narrow layout should hide sidebar");
            }
            DegradationLevel::Full => {
                // Should have all areas if configured
                if config.banner_height > 0 {
                    assert!(areas.banner.is_some(), "Full layout should have banner if configured");
                }
                if config.sidebar_enabled && config.sidebar_width > 0 {
                    // Sidebar might still be hidden if terminal is too narrow for min chat width
                    if width >= config.sidebar_width + config.min_chat_width {
                        assert!(areas.sidebar.is_some(), "Full layout should have sidebar if space allows");
                    }
                }
            }
        }
    }

    #[test]
    fn prop_layout_resize_performance(
        resize_sequence in arb_resize_sequence(),
        config in arb_layout_config()
    ) {
        // **Feature: ricecoder-tui-improvement, Property 6: Layout Constraint Satisfaction**
        // **Validates: Requirements 2.1, 2.2**
        
        // For any sequence of resizes, each should complete within 16ms (Requirement 2.2)
        let mut layout = Layout::new(80, 24);
        
        for (width, height) in resize_sequence {
            let (_areas, _scroll) = layout.handle_resize(width, height, &config);
            
            // Should meet performance requirement
            assert!(layout.meets_resize_performance_requirement(), 
                "Resize to {}x{} should complete within 16ms", width, height);
        }
    }

    #[test]
    fn prop_layout_areas_distinct_positions(
        (width, height) in arb_good_terminal_size(),
        config in arb_layout_config()
    ) {
        // **Feature: ricecoder-tui-improvement, Property 6: Layout Constraint Satisfaction**
        // **Validates: Requirements 2.1, 2.2**
        
        // For any terminal size, areas should have distinct, logical positions
        let layout = Layout::new(width, height);
        let areas = layout.calculate_areas(&config);
        
        // Banner should be at top if present
        if let Some(banner) = areas.banner {
            assert_eq!(banner.y, 0, "Banner should be at top");
            assert_eq!(banner.x, 0, "Banner should start at left edge");
            assert_eq!(banner.width, width, "Banner should span full width");
        }
        
        // Status should be at bottom
        assert_eq!(areas.status.y + areas.status.height, height, 
            "Status should be at bottom of terminal");
        assert_eq!(areas.status.x, 0, "Status should start at left edge");
        assert_eq!(areas.status.width, width, "Status should span full width");
        
        // Input should be above status
        assert!(areas.input.y + areas.input.height <= areas.status.y, 
            "Input should be above status");
        assert_eq!(areas.input.x, 0, "Input should start at left edge");
        assert_eq!(areas.input.width, width, "Input should span full width");
        
        // Sidebar should be at left if present
        if let Some(sidebar) = areas.sidebar {
            assert_eq!(sidebar.x, 0, "Sidebar should be at left edge");
            // Chat should be to the right of sidebar
            assert_eq!(areas.chat.x, sidebar.width, "Chat should be right of sidebar");
        } else {
            // Chat should be at left edge if no sidebar
            assert_eq!(areas.chat.x, 0, "Chat should be at left edge when no sidebar");
        }
    }

    #[test]
    fn prop_layout_areas_height_consistency(
        (width, height) in arb_good_terminal_size(),
        config in arb_layout_config()
    ) {
        // **Feature: ricecoder-tui-improvement, Property 6: Layout Constraint Satisfaction**
        // **Validates: Requirements 2.1, 2.2**
        
        // For any terminal size, vertical areas should add up correctly
        let layout = Layout::new(width, height);
        let areas = layout.calculate_areas(&config);
        
        let mut total_height = 0u16;
        
        // Add banner height if present
        if let Some(banner) = areas.banner {
            total_height += banner.height;
        }
        
        // Add main content height (chat area height)
        total_height += areas.chat.height;
        
        // Add input height
        total_height += areas.input.height;
        
        // Add status height
        total_height += areas.status.height;
        
        // Total should equal terminal height
        assert_eq!(total_height, height, 
            "Sum of area heights should equal terminal height");
    }

    #[test]
    fn prop_layout_scroll_adjustment_consistency(
        (width1, height1) in arb_good_terminal_size(),
        (width2, height2) in arb_good_terminal_size(),
        config in arb_layout_config()
    ) {
        // **Feature: ricecoder-tui-improvement, Property 6: Layout Constraint Satisfaction**
        // **Validates: Requirements 2.1, 2.2**
        
        // For any resize, scroll adjustment should be consistent with height change
        let mut layout = Layout::new(width1, height1);
        
        // Set initial areas
        let initial_areas = layout.calculate_areas(&config);
        layout.previous_areas = Some(initial_areas);
        
        // Resize and check scroll adjustment
        let (new_areas, scroll_adjustment) = layout.handle_resize(width2, height2, &config);
        
        if let Some(adjustment) = scroll_adjustment {
            let height_change = new_areas.chat.height as i32 - initial_areas.chat.height as i32;
            assert_eq!(adjustment.height_delta, height_change, 
                "Scroll adjustment should match chat area height change");
            assert!(adjustment.preserve_bottom, 
                "Scroll adjustment should preserve bottom position");
        }
    }

    #[test]
    fn prop_layout_minimum_requirements_enforced(
        (width, height) in arb_terminal_size(),
        config in arb_layout_config()
    ) {
        // **Feature: ricecoder-tui-improvement, Property 6: Layout Constraint Satisfaction**
        // **Validates: Requirements 2.1, 2.2**
        
        // For any terminal size, minimum requirements should be enforced
        let layout = Layout::new(width, height);
        
        // Check if meets minimum requirements
        let meets_min = layout.meets_minimum(&config);
        let is_usable = layout.is_usable();
        
        if width >= config.min_width && height >= config.min_height {
            assert!(meets_min, "Should meet minimum requirements when size is adequate");
        } else {
            assert!(!meets_min, "Should not meet minimum requirements when size is inadequate");
        }
        
        if width >= 40 && height >= 10 {
            assert!(is_usable, "Should be usable when size is at least 40x10");
        } else {
            assert!(!is_usable, "Should not be usable when size is below 40x10");
        }
    }

    #[test]
    fn prop_layout_constraint_satisfaction_idempotent(
        (width, height) in arb_good_terminal_size(),
        config in arb_layout_config()
    ) {
        // **Feature: ricecoder-tui-improvement, Property 6: Layout Constraint Satisfaction**
        // **Validates: Requirements 2.1, 2.2**
        
        // For any terminal size, calculating areas multiple times should be idempotent
        let layout = Layout::new(width, height);
        
        let areas1 = layout.calculate_areas(&config);
        let areas2 = layout.calculate_areas(&config);
        
        // Areas should be identical
        assert_eq!(areas1.banner, areas2.banner, "Banner area should be consistent");
        assert_eq!(areas1.sidebar, areas2.sidebar, "Sidebar area should be consistent");
        assert_eq!(areas1.chat, areas2.chat, "Chat area should be consistent");
        assert_eq!(areas1.input, areas2.input, "Input area should be consistent");
        assert_eq!(areas1.status, areas2.status, "Status area should be consistent");
    }
}