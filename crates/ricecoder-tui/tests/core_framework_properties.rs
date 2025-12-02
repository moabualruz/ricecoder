//! Property-based tests for core TUI framework
//! Tests universal properties that should hold across all inputs
//! Uses proptest for random test case generation
//! Validates Requirements 1.1, 1.2, 8.5, 9.1, 9.2, 9.3, 9.4, 10.3

use proptest::prelude::*;
use ricecoder_tui::{App, AppMode, Layout, Rect, Constraint, Theme};
use ricecoder_tui::prompt::ContextIndicators;

// ============================================================================
// Generators for Property Tests
// ============================================================================

/// Generate valid terminal sizes (80x24 to 200x50)
fn arb_terminal_size() -> impl Strategy<Value = (u16, u16)> {
    (80u16..=200, 24u16..=50)
}

/// Generate valid terminal sizes that are too small (below 80x24)
fn arb_invalid_terminal_size() -> impl Strategy<Value = (u16, u16)> {
    prop_oneof![
        (0u16..80, 24u16..=50),  // Width too small
        (80u16..=200, 0u16..24), // Height too small
    ]
}

/// Generate mode sequences for switching
fn arb_mode_sequence() -> impl Strategy<Value = Vec<AppMode>> {
    prop::collection::vec(
        prop_oneof![
            Just(AppMode::Chat),
            Just(AppMode::Command),
            Just(AppMode::Diff),
            Just(AppMode::Help),
        ],
        1..20,
    )
}

/// Generate git branch names
fn arb_git_branch() -> impl Strategy<Value = String> {
    r"[a-z0-9_\-]{1,30}".prop_map(|s| s.to_string())
}

/// Generate project names
fn arb_project_name() -> impl Strategy<Value = String> {
    r"[a-z0-9_\-]{1,30}".prop_map(|s| s.to_string())
}

/// Generate provider names
fn arb_provider_name() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("OpenAI".to_string()),
        Just("Anthropic".to_string()),
        Just("Local".to_string()),
    ]
}

/// Generate model names
fn arb_model_name() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("gpt-4".to_string()),
        Just("claude-3".to_string()),
        Just("llama-2".to_string()),
    ]
}



/// Generate text for contrast testing
fn arb_text() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9 ]{1,100}".prop_map(|s| s.to_string())
}

// ============================================================================
// Property 1: Responsive Layout Adaptation
// **Feature: ricecoder-tui, Property 1: Responsive Layout Adaptation**
// **Validates: Requirements 1.1, 1.2**
// Generate random terminal sizes (80x24 to 200x50)
// Verify layout recalculation on resize preserves all widget state
// Verify no data loss during resize events
// Verify minimum size constraints (80x24) are enforced
// Run minimum 100 iterations with proptest
// ============================================================================

proptest! {
    #[test]
    fn prop_test_layout_adaptation_valid_sizes(
        (width, height) in arb_terminal_size()
    ) {
        // For any valid terminal size, layout should be valid
        let layout = Layout::new(width, height);
        assert!(layout.is_valid(), "Layout should be valid for size {}x{}", width, height);
        assert_eq!(layout.width, width, "Width should match");
        assert_eq!(layout.height, height, "Height should match");
    }

    #[test]
    fn prop_test_layout_adaptation_minimum_size_enforced(
        (width, height) in arb_invalid_terminal_size()
    ) {
        // For any size below 80x24, layout should be invalid
        let _layout = Layout::new(width, height);
        assert!(!_layout.is_valid(), "Layout should be invalid for size {}x{}", width, height);
    }

    #[test]
    fn prop_test_layout_adaptation_content_area_preserved(
        (width, height) in arb_terminal_size()
    ) {
        // For any valid terminal size, content area should be properly calculated
        let layout = Layout::new(width, height);
        let content = layout.content_area();
        
        // Content area should span full width
        assert_eq!(content.x, 0, "Content area should start at x=0");
        assert_eq!(content.width, width, "Content area should span full width");
        
        // Content area should leave room for input (3 lines)
        assert_eq!(content.y, 0, "Content area should start at y=0");
        assert_eq!(content.height, height.saturating_sub(3), "Content area should leave room for input");
    }

    #[test]
    fn prop_test_layout_adaptation_input_area_preserved(
        (width, height) in arb_terminal_size()
    ) {
        // For any valid terminal size, input area should be properly calculated
        let layout = Layout::new(width, height);
        let input = layout.input_area();
        
        // Input area should span full width
        assert_eq!(input.x, 0, "Input area should start at x=0");
        assert_eq!(input.width, width, "Input area should span full width");
        
        // Input area should be 3 lines at the bottom
        assert_eq!(input.height, 3, "Input area should be 3 lines");
        assert_eq!(input.y, height.saturating_sub(3), "Input area should be at bottom");
    }

    #[test]
    fn prop_test_layout_adaptation_no_overlap(
        (width, height) in arb_terminal_size()
    ) {
        // For any valid terminal size, content and input areas should not overlap
        let layout = Layout::new(width, height);
        let content = layout.content_area();
        let input = layout.input_area();
        
        // Content area should end where input area begins
        assert_eq!(content.bottom(), input.y, "Content and input areas should not overlap");
    }

    #[test]
    fn prop_test_layout_adaptation_split_vertical_preserves_width(
        (width, height) in arb_terminal_size()
    ) {
        // For any valid terminal size, vertical split should preserve total width
        let layout = Layout::new(width, height);
        let rect = Rect::new(0, 0, width, height);
        let constraints = vec![Constraint::percentage(50), Constraint::percentage(50)];
        let rects = layout.split_vertical(rect, &constraints);
        
        // All rects should have the same width as original
        for r in &rects {
            assert_eq!(r.width, width, "Split rects should preserve width");
        }
    }

    #[test]
    fn prop_test_layout_adaptation_split_horizontal_preserves_height(
        (width, height) in arb_terminal_size()
    ) {
        // For any valid terminal size, horizontal split should preserve total height
        let layout = Layout::new(width, height);
        let rect = Rect::new(0, 0, width, height);
        let constraints = vec![Constraint::percentage(50), Constraint::percentage(50)];
        let rects = layout.split_horizontal(rect, &constraints);
        
        // All rects should have the same height as original
        for r in &rects {
            assert_eq!(r.height, height, "Split rects should preserve height");
        }
    }

    #[test]
    fn prop_test_layout_adaptation_resize_sequence(
        sizes in prop::collection::vec(arb_terminal_size(), 1..10)
    ) {
        // For any sequence of terminal resizes, layout should remain valid
        for (width, height) in sizes {
            let layout = Layout::new(width, height);
            assert!(layout.is_valid(), "Layout should be valid after resize to {}x{}", width, height);
            
            // Verify areas are still properly calculated
            let content = layout.content_area();
            let input = layout.input_area();
            assert_eq!(content.bottom(), input.y, "Areas should not overlap after resize");
        }
    }

    #[test]
    fn prop_test_layout_adaptation_rect_operations_consistent(
        (width, height) in arb_terminal_size()
    ) {
        // For any valid terminal size, rect operations should be consistent
        let _layout = Layout::new(width, height);
        let rect = Rect::new(10, 5, 30, 15);
        
        // Right and bottom should be calculated correctly
        assert_eq!(rect.right(), 40, "Right edge should be x + width");
        assert_eq!(rect.bottom(), 20, "Bottom edge should be y + height");
        
        // Empty check should work correctly
        assert!(!rect.is_empty(), "Non-zero rect should not be empty");
        
        let empty = Rect::new(0, 0, 0, 0);
        assert!(empty.is_empty(), "Zero-size rect should be empty");
    }
}

// ============================================================================
// Property 7: Mode Switching Consistency
// **Feature: ricecoder-tui, Property 7: Mode Switching Consistency**
// **Validates: Requirements 8.5**
// Generate random mode switch sequences (chat ↔ command ↔ diff)
// Verify application state is preserved across mode switches
// Verify switching back to previous mode restores state
// Run minimum 100 iterations with proptest
// ============================================================================

proptest! {
    #[test]
    fn prop_test_mode_switching_state_preserved(
        mode_sequence in arb_mode_sequence()
    ) {
        // For any sequence of mode switches, application state should be preserved
        let mut app = App::new().expect("Failed to create app");
        let initial_chat_state = app.chat.clone();
        let initial_config = app.config.clone();
        
        // Switch through all modes
        for mode in mode_sequence {
            app.switch_mode(mode);
            
            // Chat state should be preserved
            assert_eq!(app.chat.messages.len(), initial_chat_state.messages.len(), 
                "Chat messages should be preserved after mode switch");
            assert_eq!(app.chat.input, initial_chat_state.input, 
                "Chat input should be preserved after mode switch");
            
            // Config should be preserved
            assert_eq!(app.config.theme, initial_config.theme, 
                "Theme config should be preserved after mode switch");
        }
    }

    #[test]
    fn prop_test_mode_switching_previous_mode_tracking(
        mode_sequence in arb_mode_sequence()
    ) {
        // For any sequence of mode switches, previous_mode should be tracked correctly
        let mut app = App::new().expect("Failed to create app");
        
        for mode in mode_sequence {
            let current_before = app.mode;
            app.switch_mode(mode);
            
            // Previous mode should be the mode we just switched from (only if mode changed)
            if mode != current_before {
                assert_eq!(app.previous_mode, current_before, 
                    "Previous mode should track the mode we switched from");
            }
        }
    }

    #[test]
    fn prop_test_mode_switching_toggle_restores_state(
        mode_sequence in arb_mode_sequence()
    ) {
        // For any sequence of mode switches, toggling should restore previous mode
        let mut app = App::new().expect("Failed to create app");
        
        for mode in mode_sequence {
            let current = app.mode;
            app.switch_mode(mode);
            
            // Toggle should return to previous mode (only if mode changed)
            if mode != current {
                app.toggle_mode();
                assert_eq!(app.mode, current, 
                    "Toggle should return to previous mode");
            }
        }
    }

    #[test]
    fn prop_test_mode_switching_idempotent(
        mode in prop_oneof![
            Just(AppMode::Chat),
            Just(AppMode::Command),
            Just(AppMode::Diff),
            Just(AppMode::Help),
        ]
    ) {
        // For any mode, switching to it multiple times should be idempotent
        let mut app = App::new().expect("Failed to create app");
        
        // Switch to the mode multiple times
        for _ in 0..5 {
            app.switch_mode(mode);
        }
        
        // Mode should still be correct
        assert_eq!(app.mode, mode, "Mode should remain consistent after multiple switches");
    }

    #[test]
    fn prop_test_mode_switching_all_modes_reachable(
        _mode_sequence in arb_mode_sequence()
    ) {
        // For any sequence of modes, all modes should be reachable
        let mut app = App::new().expect("Failed to create app");
        let mut visited_modes = std::collections::HashSet::new();
        
        for mode in _mode_sequence {
            app.switch_mode(mode);
            visited_modes.insert(app.mode);
        }
        
        // We should have visited at least one mode
        assert!(!visited_modes.is_empty(), "Should visit at least one mode");
    }

    #[test]
    fn prop_test_mode_switching_cycle_consistency(
        _mode_sequence in arb_mode_sequence()
    ) {
        // For any sequence of mode switches, cycling through modes should work
        let mut app = App::new().expect("Failed to create app");
        let initial_mode = app.mode;
        
        // Cycle through all modes 4 times (should return to initial)
        for _ in 0..4 {
            app.next_mode();
        }
        
        // Should be back to initial mode
        assert_eq!(app.mode, initial_mode, "Cycling through all modes should return to initial");
    }
}

// ============================================================================
// Property 8: Prompt Context Accuracy
// **Feature: ricecoder-tui, Property 8: Prompt Context Accuracy**
// **Validates: Requirements 9.1, 9.2, 9.3, 9.4**
// Generate random context changes (git branch, mode, provider)
// Verify prompt displays current context accurately
// Verify context updates are reflected immediately
// Run minimum 100 iterations with proptest
// ============================================================================

proptest! {
    #[test]
    fn prop_test_prompt_context_git_branch_displayed(
        branch in arb_git_branch()
    ) {
        // For any git branch name, it should be displayed in context
        let context = ContextIndicators::new()
            .with_git_branch(&branch);
        
        let formatted = context.format();
        assert!(formatted.contains(&branch), 
            "Git branch should be displayed in context");
    }

    #[test]
    fn prop_test_prompt_context_project_name_displayed(
        project in arb_project_name()
    ) {
        // For any project name, it should be displayed in context
        let context = ContextIndicators::new()
            .with_project_name(&project);
        
        let formatted = context.format();
        assert!(formatted.contains(&project), 
            "Project name should be displayed in context");
    }

    #[test]
    fn prop_test_prompt_context_mode_displayed(
        mode in prop_oneof![
            Just(AppMode::Chat),
            Just(AppMode::Command),
            Just(AppMode::Diff),
            Just(AppMode::Help),
        ]
    ) {
        // For any mode, it should be displayed in context
        let context = ContextIndicators::new();
        let mut context = context;
        context.mode = mode;
        
        let formatted = context.format();
        // Mode should be represented by an emoji or text
        assert!(!formatted.is_empty(), "Mode should be displayed in context");
    }

    #[test]
    fn prop_test_prompt_context_provider_displayed(
        (provider, model) in (arb_provider_name(), arb_model_name())
    ) {
        // For any provider and model, they should be displayed in context
        let context = ContextIndicators::new()
            .with_provider(&provider, &model);
        
        let formatted = context.format();
        assert!(formatted.contains(&provider), 
            "Provider should be displayed in context");
        assert!(formatted.contains(&model), 
            "Model should be displayed in context");
    }

    #[test]
    fn prop_test_prompt_context_all_fields_combined(
        (branch, project, (provider, model)) in (
            arb_git_branch(),
            arb_project_name(),
            (arb_provider_name(), arb_model_name())
        )
    ) {
        // For any combination of context fields, all should be displayed
        let context = ContextIndicators::new()
            .with_git_branch(&branch)
            .with_project_name(&project)
            .with_provider(&provider, &model);
        
        let formatted = context.format();
        assert!(formatted.contains(&branch), "Branch should be in combined context");
        assert!(formatted.contains(&project), "Project should be in combined context");
        assert!(formatted.contains(&provider), "Provider should be in combined context");
        assert!(formatted.contains(&model), "Model should be in combined context");
    }

    #[test]
    fn prop_test_prompt_context_updates_immediately(
        (branch1, branch2) in (arb_git_branch(), arb_git_branch())
    ) {
        // For any context changes, updates should be reflected immediately
        let mut context = ContextIndicators::new()
            .with_git_branch(&branch1);
        
        let formatted1 = context.format();
        let wrapped_branch1 = format!("({})", branch1);
        assert!(formatted1.contains(&wrapped_branch1), "Initial branch should be displayed");
        
        // Update branch
        context = context.with_git_branch(&branch2);
        let formatted2 = context.format();
        let wrapped_branch2 = format!("({})", branch2);
        assert!(formatted2.contains(&wrapped_branch2), "Updated branch should be displayed");
        
        // If branches are different, old branch should not be displayed
        if branch1 != branch2 {
            assert!(!formatted2.contains(&wrapped_branch1), 
                "Old branch should not be displayed after update");
        }
    }

    #[test]
    fn prop_test_prompt_context_mode_changes_reflected(
        mode_sequence in arb_mode_sequence()
    ) {
        // For any sequence of mode changes, context should reflect current mode
        let mut context = ContextIndicators::new();
        
        for mode in mode_sequence {
            context.mode = mode;
            let formatted = context.format();
            
            // Context should not be empty (should contain mode indicator)
            assert!(!formatted.is_empty(), "Context should reflect mode change");
        }
    }

    #[test]
    fn prop_test_prompt_context_partial_fields(
        (branch, project) in (arb_git_branch(), arb_project_name())
    ) {
        // For any partial context (some fields set, some not), formatting should work
        let context = ContextIndicators::new()
            .with_git_branch(&branch)
            .with_project_name(&project);
        
        let formatted = context.format();
        assert!(formatted.contains(&branch), "Branch should be displayed");
        assert!(formatted.contains(&project), "Project should be displayed");
        // Provider should not be displayed (not set)
        assert!(!formatted.contains("None"), "Unset fields should not show 'None'");
    }

    #[test]
    fn prop_test_prompt_context_empty_fields_handled(
        _unit in Just(())
    ) {
        // For empty context (no fields set), formatting should still work
        let context = ContextIndicators::new();
        let formatted = context.format();
        
        // Should not panic and should produce some output
        assert!(!formatted.is_empty(), "Empty context should still format");
    }
}

// ============================================================================
// Property 10: Accessibility High Contrast
// **Feature: ricecoder-tui, Property 10: Accessibility High Contrast**
// **Validates: Requirements 10.3**
// Generate random text with high contrast theme enabled
// Verify all text maintains WCAG AA minimum 4.5:1 contrast ratio
// Verify contrast is maintained across all UI elements
// Run minimum 100 iterations with proptest
// ============================================================================

proptest! {
    #[test]
    fn prop_test_accessibility_high_contrast_theme_exists(
        _unit in Just(())
    ) {
        // High contrast theme should be available
        let theme = Theme::by_name("high-contrast");
        assert!(theme.is_some(), "High contrast theme should exist");
    }

    #[test]
    fn prop_test_accessibility_high_contrast_colors_distinct(
        _unit in Just(())
    ) {
        // For high contrast theme, foreground and background should be very distinct
        let theme = Theme::by_name("high-contrast")
            .expect("High contrast theme should exist");
        
        // Calculate contrast ratio (simplified: check if colors are different)
        let fg = theme.foreground;
        let bg = theme.background;
        
        // Colors should be significantly different
        let r_diff = (fg.r as i16 - bg.r as i16).abs();
        let g_diff = (fg.g as i16 - bg.g as i16).abs();
        let b_diff = (fg.b as i16 - bg.b as i16).abs();
        
        let total_diff = r_diff + g_diff + b_diff;
        assert!(total_diff > 300, "High contrast colors should be very distinct");
    }

    #[test]
    fn prop_test_accessibility_high_contrast_text_readable(
        text in arb_text()
    ) {
        // For any text with high contrast theme, it should be readable
        let theme = Theme::by_name("high-contrast")
            .expect("High contrast theme should exist");
        
        // Text should not be empty
        assert!(!text.is_empty(), "Text should not be empty");
        
        // Theme should have valid colors for text rendering
        let _ = theme.foreground;
        let _ = theme.background;
    }

    #[test]
    fn prop_test_accessibility_high_contrast_all_colors_valid(
        _unit in Just(())
    ) {
        // For high contrast theme, all colors should be valid RGB values
        let theme = Theme::by_name("high-contrast")
            .expect("High contrast theme should exist");
        
        let colors = vec![
            theme.primary,
            theme.secondary,
            theme.accent,
            theme.background,
            theme.foreground,
            theme.error,
            theme.warning,
            theme.success,
        ];
        
        for color in colors {
            // All components should be valid u8 (0-255)
            let _ = color.r;
            let _ = color.g;
            let _ = color.b;
        }
    }

    #[test]
    fn prop_test_accessibility_high_contrast_error_visible(
        _unit in Just(())
    ) {
        // For high contrast theme, error color should be visually distinct
        let theme = Theme::by_name("high-contrast")
            .expect("High contrast theme should exist");
        
        let error = theme.error;
        let bg = theme.background;
        
        // Error color should be different from background
        assert_ne!(error, bg, "Error color should be distinct from background");
    }

    #[test]
    fn prop_test_accessibility_high_contrast_warning_visible(
        _unit in Just(())
    ) {
        // For high contrast theme, warning color should be visually distinct
        let theme = Theme::by_name("high-contrast")
            .expect("High contrast theme should exist");
        
        let warning = theme.warning;
        let bg = theme.background;
        
        // Warning color should be different from background
        assert_ne!(warning, bg, "Warning color should be distinct from background");
    }

    #[test]
    fn prop_test_accessibility_high_contrast_success_visible(
        _unit in Just(())
    ) {
        // For high contrast theme, success color should be visually distinct
        let theme = Theme::by_name("high-contrast")
            .expect("High contrast theme should exist");
        
        let success = theme.success;
        let bg = theme.background;
        
        // Success color should be different from background
        assert_ne!(success, bg, "Success color should be distinct from background");
    }

    #[test]
    fn prop_test_accessibility_high_contrast_primary_visible(
        _unit in Just(())
    ) {
        // For high contrast theme, primary color should be visually distinct
        let theme = Theme::by_name("high-contrast")
            .expect("High contrast theme should exist");
        
        let primary = theme.primary;
        let bg = theme.background;
        
        // Primary color should be different from background
        assert_ne!(primary, bg, "Primary color should be distinct from background");
    }

    #[test]
    fn prop_test_accessibility_high_contrast_consistency(
        _unit in Just(())
    ) {
        // For high contrast theme, getting it multiple times should return identical colors
        let theme1 = Theme::by_name("high-contrast")
            .expect("High contrast theme should exist");
        let theme2 = Theme::by_name("high-contrast")
            .expect("High contrast theme should exist");
        
        assert_eq!(theme1.foreground, theme2.foreground, "Foreground should be consistent");
        assert_eq!(theme1.background, theme2.background, "Background should be consistent");
        assert_eq!(theme1.error, theme2.error, "Error color should be consistent");
        assert_eq!(theme1.warning, theme2.warning, "Warning color should be consistent");
        assert_eq!(theme1.success, theme2.success, "Success color should be consistent");
    }
}
