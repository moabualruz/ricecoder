//! Property-based tests for interactive components
//! Tests universal properties that should hold across all inputs
//! Uses proptest for random test case generation
//! Validates Requirements 4.1, 4.2, 4.3, 4.4, 4.5, 10.2

use proptest::prelude::*;
use ricecoder_tui::components::{
    MenuWidget, MenuItem, ListWidget, DialogWidget, DialogType,
    SplitViewWidget, TabWidget, VimKeybindings,
};

// ============================================================================
// Generators for Property Tests
// ============================================================================

/// Generate a valid menu item
fn arb_menu_item() -> impl Strategy<Value = MenuItem> {
    (
        "[a-zA-Z0-9 ]{1,20}",
        prop_oneof![
            Just(None),
            "[a-zA-Z0-9 ]{1,30}".prop_map(Some),
        ],
    )
        .prop_map(|(label, desc)| {
            let item = MenuItem::new(label);
            if let Some(d) = desc {
                item.with_description(d)
            } else {
                item
            }
        })
}

/// Generate a sequence of menu items
fn arb_menu_items() -> impl Strategy<Value = Vec<MenuItem>> {
    prop::collection::vec(arb_menu_item(), 1..10)
}

/// Generate a valid list item
fn arb_list_item() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 ]{1,20}"
}

/// Generate a sequence of list items
fn arb_list_items() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(arb_list_item(), 1..10)
}

/// Generate a keyboard navigation action
#[derive(Debug, Clone, Copy)]
enum NavigationAction {
    Next,
    Previous,
    First,
    Last,
}

/// Generate a sequence of navigation actions
fn arb_navigation_actions() -> impl Strategy<Value = Vec<NavigationAction>> {
    prop::collection::vec(
        prop_oneof![
            Just(NavigationAction::Next),
            Just(NavigationAction::Previous),
            Just(NavigationAction::First),
            Just(NavigationAction::Last),
        ],
        1..20,
    )
}

/// Generate a dialog input character
fn arb_dialog_char() -> impl Strategy<Value = char> {
    prop_oneof![
        (b'a'..=b'z').prop_map(|c| c as char),
        (b'A'..=b'Z').prop_map(|c| c as char),
        (b'0'..=b'9').prop_map(|c| c as char),
        Just(' '),
    ]
}

/// Generate a sequence of dialog input characters
fn arb_dialog_input() -> impl Strategy<Value = Vec<char>> {
    prop::collection::vec(arb_dialog_char(), 0..20)
}

/// Generate a tab title
fn arb_tab_title() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 ]{1,15}"
}

/// Generate a sequence of tab titles
fn arb_tab_titles() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(arb_tab_title(), 1..10)
}

// ============================================================================
// Property 6: Keyboard Navigation Completeness
// **Feature: ricecoder-tui, Property 6: Keyboard Navigation Completeness**
// **Validates: Requirements 4.1, 4.2, 4.3, 4.4, 4.5, 10.2**
// Generate random component sequences (menus, lists, dialogs, tabs)
// Verify all focusable elements are reachable via keyboard navigation
// Verify Tab key cycles through all focusable elements
// Verify arrow keys navigate within components
// Verify Enter/Escape keys work as expected
// Run minimum 100 iterations with proptest
// ============================================================================

proptest! {
    #[test]
    fn prop_test_menu_navigation_completeness(items in arb_menu_items()) {
        // For any menu with items, all items should be reachable via keyboard navigation
        let mut menu = MenuWidget::new();
        for item in &items {
            menu.add_item(item.clone());
        }

        // Verify we can reach every item by navigating forward
        for i in 0..items.len() {
            assert_eq!(menu.selected, i, "Should be at item {}", i);
            if i < items.len() - 1 {
                menu.select_next();
            }
        }

        // Verify we can reach every item by navigating backward
        for i in (0..items.len()).rev() {
            assert_eq!(menu.selected, i, "Should be at item {}", i);
            if i > 0 {
                menu.select_prev();
            }
        }
    }

    #[test]
    fn prop_test_menu_navigation_first_last(items in arb_menu_items()) {
        // For any menu, first and last navigation should work
        let mut menu = MenuWidget::new();
        for item in &items {
            menu.add_item(item.clone());
        }

        // Jump to last
        menu.select_last();
        assert_eq!(menu.selected, items.len() - 1, "Should be at last item");

        // Jump to first
        menu.select_first();
        assert_eq!(menu.selected, 0, "Should be at first item");

        // Jump to last again
        menu.select_last();
        assert_eq!(menu.selected, items.len() - 1, "Should be at last item again");
    }

    #[test]
    fn prop_test_menu_navigation_wrapping(items in arb_menu_items()) {
        // For any menu, navigation should handle boundaries correctly
        let mut menu = MenuWidget::new();
        for item in &items {
            menu.add_item(item.clone());
        }

        // At first item, previous should not go negative
        menu.select_first();
        menu.select_prev();
        assert_eq!(menu.selected, 0, "Should stay at first item");

        // At last item, next should not go beyond
        menu.select_last();
        menu.select_next();
        assert_eq!(menu.selected, items.len() - 1, "Should stay at last item");
    }

    #[test]
    fn prop_test_menu_navigation_sequence(
        items in arb_menu_items(),
        actions in arb_navigation_actions()
    ) {
        // For any menu and navigation sequence, selection should remain valid
        let mut menu = MenuWidget::new();
        for item in &items {
            menu.add_item(item.clone());
        }

        for action in actions {
            match action {
                NavigationAction::Next => menu.select_next(),
                NavigationAction::Previous => menu.select_prev(),
                NavigationAction::First => menu.select_first(),
                NavigationAction::Last => menu.select_last(),
            }

            // Selection should always be within bounds
            assert!(
                menu.selected < items.len(),
                "Selection {} should be within bounds [0, {})",
                menu.selected,
                items.len()
            );
        }
    }

    #[test]
    fn prop_test_list_navigation_completeness(items in arb_list_items()) {
        // For any list with items, all items should be reachable via keyboard navigation
        let mut list = ListWidget::new();
        for item in &items {
            list.add_item(item.clone());
        }

        // Verify we can reach every item by navigating forward
        for i in 0..items.len() {
            // First call to select_next initializes selection
            if i == 0 {
                list.select_next();
            }
            
            if let Some(selected) = list.selected {
                assert_eq!(selected, i, "Should be at item {}", i);
            }
            
            if i < items.len() - 1 {
                list.select_next();
            }
        }

        // Verify we can reach every item by navigating backward
        for i in (0..items.len()).rev() {
            if let Some(selected) = list.selected {
                assert_eq!(selected, i, "Should be at item {}", i);
            }
            if i > 0 {
                list.select_prev();
            }
        }
    }

    #[test]
    fn prop_test_list_navigation_with_filter(items in arb_list_items()) {
        // For any list, navigation should work correctly with filters
        let mut list = ListWidget::new();
        for item in &items {
            list.add_item(item.clone());
        }

        // Apply a filter that matches at least one item
        if !items.is_empty() {
            let first_char = items[0].chars().next().unwrap_or('a');
            list.set_filter(first_char.to_string());

            // Navigate through filtered items
            {
                let filtered = list.filtered_items();
                if !filtered.is_empty() {
                    // Check if we can navigate
                    let has_items = !filtered.is_empty();
                    drop(filtered); // Drop the borrow before calling select_next
                    
                    if has_items {
                        list.select_next();
                        // Selection should be one of the filtered items
                        if let Some(selected) = list.selected {
                            let filtered = list.filtered_items();
                            assert!(
                                filtered.iter().any(|(idx, _)| *idx == selected),
                                "Selected item should be in filtered results"
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn prop_test_list_multi_select_navigation(items in arb_list_items()) {
        // For any list in multi-select mode, navigation should work correctly
        let mut list = ListWidget::new().with_multi_select();
        for item in &items {
            list.add_item(item.clone());
        }

        // Navigate and select items
        for _ in 0..items.len().min(3) {
            list.select_next();
            list.toggle_selection();
        }

        // Verify selections are valid
        for selected_idx in &list.selected_items {
            assert!(
                *selected_idx < items.len(),
                "Selected index {} should be within bounds",
                selected_idx
            );
        }
    }

    #[test]
    fn prop_test_dialog_input_navigation(input_chars in arb_dialog_input()) {
        // For any dialog input, cursor navigation should work correctly
        let mut dialog = DialogWidget::new(DialogType::Input, "Test", "Enter text");

        // Insert characters
        for ch in &input_chars {
            dialog.insert_char(*ch);
        }

        // Verify cursor is at the end
        assert_eq!(dialog.cursor, input_chars.len(), "Cursor should be at end");

        // Navigate to start
        dialog.cursor_start();
        assert_eq!(dialog.cursor, 0, "Cursor should be at start");

        // Navigate to end
        dialog.cursor_end();
        assert_eq!(dialog.cursor, input_chars.len(), "Cursor should be at end");

        // Navigate left and right
        if input_chars.len() > 0 {
            dialog.cursor_left();
            assert_eq!(dialog.cursor, input_chars.len() - 1, "Cursor should move left");

            dialog.cursor_right();
            assert_eq!(dialog.cursor, input_chars.len(), "Cursor should move right");
        }
    }

    #[test]
    fn prop_test_dialog_cursor_bounds(input_chars in arb_dialog_input()) {
        // For any dialog input, cursor should stay within bounds
        let mut dialog = DialogWidget::new(DialogType::Input, "Test", "Enter text");

        for ch in &input_chars {
            dialog.insert_char(*ch);
        }

        // Try to move left from start
        dialog.cursor_start();
        dialog.cursor_left();
        assert_eq!(dialog.cursor, 0, "Cursor should not go below 0");

        // Try to move right from end
        dialog.cursor_end();
        dialog.cursor_right();
        assert_eq!(dialog.cursor, input_chars.len(), "Cursor should not go beyond length");
    }

    #[test]
    fn prop_test_tab_navigation_completeness(titles in arb_tab_titles()) {
        // For any tab widget with tabs, all tabs should be reachable via keyboard navigation
        let mut tabs = TabWidget::new();
        for title in &titles {
            tabs.add_tab(title.clone());
        }

        // Verify we can reach every tab by navigating forward
        for i in 0..titles.len() {
            assert_eq!(tabs.active, i, "Should be at tab {}", i);
            if i < titles.len() - 1 {
                tabs.select_next();
            }
        }

        // Verify we can reach every tab by navigating backward
        for i in (0..titles.len()).rev() {
            assert_eq!(tabs.active, i, "Should be at tab {}", i);
            if i > 0 {
                tabs.select_prev();
            }
        }
    }

    #[test]
    fn prop_test_tab_navigation_direct_selection(titles in arb_tab_titles()) {
        // For any tab widget, direct selection should work
        let mut tabs = TabWidget::new();
        for title in &titles {
            tabs.add_tab(title.clone());
        }

        // Select each tab directly
        for i in 0..titles.len() {
            tabs.select_tab(i);
            assert_eq!(tabs.active, i, "Should be at tab {}", i);
        }
    }

    #[test]
    fn prop_test_tab_navigation_boundaries(titles in arb_tab_titles()) {
        // For any tab widget, navigation should handle boundaries correctly
        let mut tabs = TabWidget::new();
        for title in &titles {
            tabs.add_tab(title.clone());
        }

        // At first tab, previous should not go negative
        tabs.select_tab(0);
        tabs.select_prev();
        assert_eq!(tabs.active, 0, "Should stay at first tab");

        // At last tab, next should not go beyond
        tabs.select_tab(titles.len() - 1);
        tabs.select_next();
        assert_eq!(tabs.active, titles.len() - 1, "Should stay at last tab");
    }

    #[test]
    fn prop_test_split_view_panel_switching(
        left_content in "[a-zA-Z0-9 ]{1,30}",
        right_content in "[a-zA-Z0-9 ]{1,30}"
    ) {
        // For any split view, panel switching should work correctly
        let mut split = SplitViewWidget::new();
        split.set_left(&left_content);
        split.set_right(&right_content);

        // Verify initial panel
        assert_eq!(split.active_panel, 0, "Should start at left panel");
        assert_eq!(split.active_content(), &left_content, "Should show left content");

        // Switch to right panel
        split.switch_panel();
        assert_eq!(split.active_panel, 1, "Should be at right panel");
        assert_eq!(split.active_content(), &right_content, "Should show right content");

        // Switch back to left panel
        split.switch_panel();
        assert_eq!(split.active_panel, 0, "Should be back at left panel");
        assert_eq!(split.active_content(), &left_content, "Should show left content again");
    }

    #[test]
    fn prop_test_vim_mode_navigation(_actions in arb_navigation_actions()) {
        // For any vim mode configuration, mode switching should work correctly
        let mut vim = VimKeybindings::new();
        vim.enable();

        // Verify initial state
        assert!(vim.is_normal(), "Should start in normal mode");

        // Switch through modes
        vim.enter_insert();
        assert!(vim.is_insert(), "Should be in insert mode");

        vim.enter_visual();
        assert!(vim.is_visual(), "Should be in visual mode");

        vim.enter_command();
        assert!(vim.is_command(), "Should be in command mode");

        vim.enter_normal();
        assert!(vim.is_normal(), "Should be back in normal mode");
    }

    #[test]
    fn prop_test_vim_mode_disabled_navigation(_actions in arb_navigation_actions()) {
        // For any vim mode when disabled, mode checks should return false
        let vim = VimKeybindings::new();

        assert!(!vim.is_normal(), "Should not be in normal mode when disabled");
        assert!(!vim.is_insert(), "Should not be in insert mode when disabled");
        assert!(!vim.is_visual(), "Should not be in visual mode when disabled");
        assert!(!vim.is_command(), "Should not be in command mode when disabled");
    }

    #[test]
    fn prop_test_dialog_confirm_cancel_navigation(input_chars in arb_dialog_input()) {
        // For any dialog, confirm and cancel should work correctly
        let mut dialog = DialogWidget::new(DialogType::Input, "Test", "Enter text");

        for ch in &input_chars {
            dialog.insert_char(*ch);
        }

        // Confirm should set result
        dialog.confirm();
        assert!(dialog.is_confirmed(), "Should be confirmed");
        assert!(!dialog.is_pending(), "Should not be pending");

        // Create new dialog and cancel
        let mut dialog = DialogWidget::new(DialogType::Input, "Test", "Enter text");
        dialog.cancel();
        assert!(dialog.is_cancelled(), "Should be cancelled");
        assert!(!dialog.is_pending(), "Should not be pending");
    }

    #[test]
    fn prop_test_menu_selected_item_consistency(items in arb_menu_items()) {
        // For any menu, selected_item should match selected index
        let mut menu = MenuWidget::new();
        for item in &items {
            menu.add_item(item.clone());
        }

        for i in 0..items.len() {
            menu.selected = i;
            let selected = menu.selected_item();
            assert!(selected.is_some(), "Should have selected item at index {}", i);
            assert_eq!(selected.unwrap().label, items[i].label, "Selected item should match");
        }
    }

    #[test]
    fn prop_test_list_selected_item_consistency(items in arb_list_items()) {
        // For any list, selected_item should match selected index
        let mut list = ListWidget::new();
        for item in &items {
            list.add_item(item.clone());
        }

        for i in 0..items.len() {
            list.selected = Some(i);
            let selected = list.selected_item();
            assert!(selected.is_some(), "Should have selected item at index {}", i);
            assert_eq!(selected.unwrap(), &items[i], "Selected item should match");
        }
    }

    #[test]
    fn prop_test_tab_active_content_consistency(titles in arb_tab_titles()) {
        // For any tab widget, active_content should match active index
        let mut tabs = TabWidget::new();
        for (i, title) in titles.iter().enumerate() {
            tabs.add_tab_with_content(title.clone(), format!("Content {}", i));
        }

        for i in 0..titles.len() {
            tabs.select_tab(i);
            let content = tabs.active_content();
            assert!(content.is_some(), "Should have content at tab {}", i);
            assert_eq!(content.unwrap(), &format!("Content {}", i), "Content should match");
        }
    }

    #[test]
    fn prop_test_dialog_validation_consistency(input_chars in arb_dialog_input()) {
        // For any dialog with validator, validation should be consistent
        let mut dialog = DialogWidget::new(DialogType::Input, "Test", "Enter text")
            .with_validator(|input| !input.is_empty() && input.len() >= 2);

        for ch in &input_chars {
            dialog.insert_char(*ch);
        }

        // Validate multiple times should give same result
        let first_result = dialog.validate();
        let second_result = dialog.validate();
        assert_eq!(first_result, second_result, "Validation should be consistent");

        // Result should match input length
        if input_chars.len() >= 2 {
            assert!(first_result, "Should validate for input length >= 2");
        } else if input_chars.len() > 0 {
            assert!(!first_result, "Should not validate for input length < 2");
        }
    }
}

// ============================================================================
// Additional Property Tests for Robustness
// ============================================================================

proptest! {
    #[test]
    fn prop_test_menu_item_count_consistency(items in arb_menu_items()) {
        // For any menu, item_count should match items.len()
        let mut menu = MenuWidget::new();
        for item in &items {
            menu.add_item(item.clone());
        }

        assert_eq!(menu.item_count(), items.len(), "Item count should match");
    }

    #[test]
    fn prop_test_list_item_count_consistency(items in arb_list_items()) {
        // For any list, item_count should match items.len()
        let mut list = ListWidget::new();
        for item in &items {
            list.add_item(item.clone());
        }

        assert_eq!(list.item_count(), items.len(), "Item count should match");
    }

    #[test]
    fn prop_test_tab_count_consistency(titles in arb_tab_titles()) {
        // For any tab widget, tab_count should match tabs.len()
        let mut tabs = TabWidget::new();
        for title in &titles {
            tabs.add_tab(title.clone());
        }

        assert_eq!(tabs.tab_count(), titles.len(), "Tab count should match");
    }

    #[test]
    fn prop_test_menu_clear_resets_state(items in arb_menu_items()) {
        // For any menu, clear should reset all state
        let mut menu = MenuWidget::new();
        for item in &items {
            menu.add_item(item.clone());
        }

        menu.selected = items.len() - 1;
        menu.scroll = 5;
        menu.clear();

        assert!(menu.is_empty(), "Menu should be empty after clear");
        assert_eq!(menu.selected, 0, "Selected should be reset");
        assert_eq!(menu.scroll, 0, "Scroll should be reset");
    }

    #[test]
    fn prop_test_list_clear_resets_state(items in arb_list_items()) {
        // For any list, clear should reset all state
        let mut list = ListWidget::new();
        for item in &items {
            list.add_item(item.clone());
        }

        list.selected = Some(items.len() - 1);
        list.scroll = 5;
        list.clear();

        assert!(list.is_empty(), "List should be empty after clear");
        assert!(list.selected.is_none(), "Selected should be reset");
        assert_eq!(list.scroll, 0, "Scroll should be reset");
    }

    #[test]
    fn prop_test_tab_clear_resets_state(titles in arb_tab_titles()) {
        // For any tab widget, clear should reset all state
        let mut tabs = TabWidget::new();
        for title in &titles {
            tabs.add_tab(title.clone());
        }

        tabs.active = titles.len() - 1;
        tabs.scroll = 5;
        tabs.clear();

        assert!(tabs.is_empty(), "Tabs should be empty after clear");
        assert_eq!(tabs.active, 0, "Active should be reset");
        assert_eq!(tabs.scroll, 0, "Scroll should be reset");
    }

    #[test]
    fn prop_test_dialog_clear_input_resets_state(input_chars in arb_dialog_input()) {
        // For any dialog, clear_input should reset input state
        let mut dialog = DialogWidget::new(DialogType::Input, "Test", "Enter text");

        for ch in &input_chars {
            dialog.insert_char(*ch);
        }

        dialog.clear_input();

        assert!(dialog.get_input().is_empty(), "Input should be empty");
        assert_eq!(dialog.cursor, 0, "Cursor should be at start");
        assert!(dialog.error_message.is_none(), "Error message should be cleared");
    }

    #[test]
    fn prop_test_split_view_content_independence(
        left_content in "[a-zA-Z0-9 ]{1,30}",
        right_content in "[a-zA-Z0-9 ]{1,30}"
    ) {
        // For any split view, left and right content should be independent
        let mut split = SplitViewWidget::new();
        split.set_left(&left_content);
        split.set_right(&right_content);

        // Modify left content
        split.set_left("new left");
        assert_eq!(split.left_content, "new left", "Left content should be updated");
        assert_eq!(split.right_content, right_content, "Right content should be unchanged");

        // Modify right content
        split.set_right("new right");
        assert_eq!(split.left_content, "new left", "Left content should be unchanged");
        assert_eq!(split.right_content, "new right", "Right content should be updated");
    }

    #[test]
    fn prop_test_vim_toggle_idempotent(_actions in arb_navigation_actions()) {
        // For any vim configuration, toggling should be idempotent
        let mut vim = VimKeybindings::new();

        let initial_state = vim.enabled;

        vim.toggle();
        vim.toggle();

        assert_eq!(vim.enabled, initial_state, "Toggle should be idempotent");
    }
}
