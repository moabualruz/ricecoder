mod tests {
    use super::*;

    #[test]
    fn test_menu_widget() {
        let mut menu = MenuWidget::new();
        menu.add_item(MenuItem::new("Option 1"));
        menu.add_item(MenuItem::new("Option 2"));

        assert_eq!(menu.selected, 0);
        menu.select_next();
        assert_eq!(menu.selected, 1);
        menu.select_prev();
        assert_eq!(menu.selected, 0);
    }

    #[test]
    fn test_menu_widget_with_title() {
        let menu = MenuWidget::with_title("Main Menu");
        assert_eq!(menu.title, Some("Main Menu".to_string()));
        assert!(!menu.open);
    }

    #[test]
    fn test_menu_widget_open_close() {
        let mut menu = MenuWidget::new();
        assert!(!menu.open);

        menu.open();
        assert!(menu.open);

        menu.close();
        assert!(!menu.open);

        menu.toggle();
        assert!(menu.open);
    }

    #[test]
    fn test_menu_widget_first_last() {
        let mut menu = MenuWidget::new();
        menu.add_item(MenuItem::new("Item 1"));
        menu.add_item(MenuItem::new("Item 2"));
        menu.add_item(MenuItem::new("Item 3"));
        menu.add_item(MenuItem::new("Item 4"));

        menu.select_last();
        assert_eq!(menu.selected, 3);

        menu.select_first();
        assert_eq!(menu.selected, 0);
    }

    #[test]
    fn test_menu_widget_visible_items() {
        let mut menu = MenuWidget::new();
        for i in 0..10 {
            menu.add_item(MenuItem::new(format!("Item {}", i)));
        }

        let visible = menu.visible_items(5);
        assert_eq!(visible.len(), 5);

        menu.scroll = 3;
        let visible = menu.visible_items(5);
        assert_eq!(visible.len(), 5);
        assert_eq!(visible[0].0, 3);
    }

    #[test]
    fn test_menu_widget_clear() {
        let mut menu = MenuWidget::new();
        menu.add_item(MenuItem::new("Item 1"));
        menu.add_item(MenuItem::new("Item 2"));
        menu.selected = 1;

        menu.clear();
        assert!(menu.items.is_empty());
        assert_eq!(menu.selected, 0);
        assert_eq!(menu.scroll, 0);
    }

    #[test]
    fn test_menu_widget_add_items() {
        let mut menu = MenuWidget::new();
        let items = vec![
            MenuItem::new("Item 1"),
            MenuItem::new("Item 2"),
            MenuItem::new("Item 3"),
        ];
        menu.add_items(items);

        assert_eq!(menu.item_count(), 3);
    }

    #[test]
    fn test_menu_widget_is_empty() {
        let menu = MenuWidget::new();
        assert!(menu.is_empty());

        let mut menu = MenuWidget::new();
        menu.add_item(MenuItem::new("Item"));
        assert!(!menu.is_empty());
    }

    #[test]
    fn test_list_widget() {
        let mut list = ListWidget::new();
        list.add_item("apple");
        list.add_item("banana");
        list.add_item("cherry");

        assert_eq!(list.filtered_items().len(), 3);

        list.set_filter("app");
        assert_eq!(list.filtered_items().len(), 1);
    }

    #[test]
    fn test_list_widget_selection() {
        let mut list = ListWidget::new();
        list.add_item("item1");
        list.add_item("item2");
        list.add_item("item3");

        list.select_next();
        assert_eq!(list.selected, Some(0));

        list.select_next();
        assert_eq!(list.selected, Some(1));
    }

    #[test]
    fn test_list_widget_multi_select() {
        let mut list = ListWidget::new().with_multi_select();
        list.add_item("item1");
        list.add_item("item2");
        list.add_item("item3");

        assert!(list.multi_select);

        list.select_next();
        list.toggle_selection();
        assert!(list.selected_items.contains(&0));

        list.select_next();
        list.toggle_selection();
        assert!(list.selected_items.contains(&1));

        let selected = list.get_selected_items();
        assert_eq!(selected.len(), 2);
    }

    #[test]
    fn test_list_widget_select_all() {
        let mut list = ListWidget::new().with_multi_select();
        list.add_item("item1");
        list.add_item("item2");
        list.add_item("item3");

        list.select_all();
        assert_eq!(list.selected_items.len(), 3);
    }

    #[test]
    fn test_list_widget_deselect_all() {
        let mut list = ListWidget::new().with_multi_select();
        list.add_item("item1");
        list.add_item("item2");
        list.add_item("item3");

        list.select_all();
        assert_eq!(list.selected_items.len(), 3);

        list.deselect_all();
        assert!(list.selected_items.is_empty());
    }

    #[test]
    fn test_list_widget_filter() {
        let mut list = ListWidget::new();
        list.add_item("apple");
        list.add_item("apricot");
        list.add_item("banana");
        list.add_item("blueberry");

        list.set_filter("ap");
        assert_eq!(list.filtered_items().len(), 2);

        list.set_filter("b");
        assert_eq!(list.filtered_items().len(), 2);

        list.clear_filter();
        assert_eq!(list.filtered_items().len(), 4);
    }

    #[test]
    fn test_list_widget_visible_items() {
        let mut list = ListWidget::new();
        for i in 0..10 {
            list.add_item(format!("item{}", i));
        }

        let visible = list.visible_items(5);
        assert_eq!(visible.len(), 5);

        list.scroll = 3;
        let visible = list.visible_items(5);
        assert_eq!(visible.len(), 5);
    }

    #[test]
    fn test_list_widget_add_items() {
        let mut list = ListWidget::new();
        let items = vec![
            "item1".to_string(),
            "item2".to_string(),
            "item3".to_string(),
        ];
        list.add_items(items);

        assert_eq!(list.item_count(), 3);
    }

    #[test]
    fn test_list_widget_clear() {
        let mut list = ListWidget::new();
        list.add_item("item1");
        list.add_item("item2");
        list.selected = Some(0);

        list.clear();
        assert!(list.items.is_empty());
        assert!(list.selected.is_none());
    }

    #[test]
    fn test_list_widget_is_empty() {
        let list = ListWidget::new();
        assert!(list.is_empty());

        let mut list = ListWidget::new();
        list.add_item("item");
        assert!(!list.is_empty());
    }

    #[test]
    fn test_dialog_widget() {
        let dialog = DialogWidget::new(DialogType::Input, "Title", "Message");
        assert_eq!(dialog.dialog_type, DialogType::Input);
        assert_eq!(dialog.title, "Title");
        assert!(dialog.is_pending());
    }

    #[test]
    fn test_dialog_widget_input() {
        let mut dialog = DialogWidget::new(DialogType::Input, "Title", "Message");
        dialog.insert_char('h');
        dialog.insert_char('i');

        assert_eq!(dialog.get_input(), "hi");
    }

    #[test]
    fn test_dialog_widget_cursor_movement() {
        let mut dialog = DialogWidget::new(DialogType::Input, "Title", "Message");
        dialog.insert_char('h');
        dialog.insert_char('e');
        dialog.insert_char('l');
        dialog.insert_char('l');
        dialog.insert_char('o');

        assert_eq!(dialog.cursor, 5);

        dialog.cursor_left();
        assert_eq!(dialog.cursor, 4);

        dialog.cursor_right();
        assert_eq!(dialog.cursor, 5);

        dialog.cursor_start();
        assert_eq!(dialog.cursor, 0);

        dialog.cursor_end();
        assert_eq!(dialog.cursor, 5);
    }

    #[test]
    fn test_dialog_widget_delete() {
        let mut dialog = DialogWidget::new(DialogType::Input, "Title", "Message");
        dialog.insert_char('h');
        dialog.insert_char('e');
        dialog.insert_char('l');
        dialog.insert_char('l');
        dialog.insert_char('o');

        dialog.cursor_start();
        dialog.delete();
        assert_eq!(dialog.get_input(), "ello");
    }

    #[test]
    fn test_dialog_widget_backspace() {
        let mut dialog = DialogWidget::new(DialogType::Input, "Title", "Message");
        dialog.insert_char('h');
        dialog.insert_char('e');
        dialog.insert_char('l');
        dialog.insert_char('l');
        dialog.insert_char('o');

        dialog.backspace();
        assert_eq!(dialog.get_input(), "hell");
        assert_eq!(dialog.cursor, 4);
    }

    #[test]
    fn test_dialog_widget_confirm() {
        let mut dialog = DialogWidget::new(DialogType::Input, "Title", "Message");
        dialog.insert_char('t');
        dialog.insert_char('e');
        dialog.insert_char('s');
        dialog.insert_char('t');

        dialog.confirm();
        assert!(dialog.is_confirmed());
        assert!(!dialog.is_pending());
    }

    #[test]
    fn test_dialog_widget_cancel() {
        let mut dialog = DialogWidget::new(DialogType::Input, "Title", "Message");
        dialog.cancel();
        assert!(dialog.is_cancelled());
        assert!(!dialog.is_pending());
    }

    #[test]
    fn test_dialog_widget_validation() {
        let mut dialog = DialogWidget::new(DialogType::Input, "Title", "Message")
            .with_validator(|input| !input.is_empty() && input.len() >= 3);

        dialog.insert_char('a');
        dialog.insert_char('b');
        assert!(!dialog.validate());

        dialog.insert_char('c');
        assert!(dialog.validate());
    }

    #[test]
    fn test_dialog_widget_confirm_dialog() {
        let mut dialog = DialogWidget::new(DialogType::Confirm, "Confirm", "Are you sure?");
        assert!(dialog.confirmed.is_none());

        dialog.confirm();
        assert_eq!(dialog.confirmed, Some(true));
        assert!(dialog.is_confirmed());

        let mut dialog = DialogWidget::new(DialogType::Confirm, "Confirm", "Are you sure?");
        dialog.cancel();
        assert_eq!(dialog.confirmed, Some(false));
        assert!(dialog.is_cancelled());
    }

    #[test]
    fn test_dialog_widget_clear_input() {
        let mut dialog = DialogWidget::new(DialogType::Input, "Title", "Message");
        dialog.insert_char('t');
        dialog.insert_char('e');
        dialog.insert_char('s');
        dialog.insert_char('t');

        dialog.clear_input();
        assert!(dialog.get_input().is_empty());
        assert_eq!(dialog.cursor, 0);
    }

    #[test]
    fn test_split_view_widget() {
        let mut split = SplitViewWidget::new();
        split.set_left("left content");
        split.set_right("right content");

        assert_eq!(split.left_content, "left content");
        assert_eq!(split.right_content, "right content");
        assert_eq!(split.split_ratio, 50);
    }

    #[test]
    fn test_split_view_adjust() {
        let mut split = SplitViewWidget::new();
        split.adjust_split(10);
        assert_eq!(split.split_ratio, 60);

        split.adjust_split(-20);
        assert_eq!(split.split_ratio, 40);
    }

    #[test]
    fn test_split_view_direction() {
        let split = SplitViewWidget::new();
        assert_eq!(split.direction, SplitDirection::Vertical);

        let split = SplitViewWidget::horizontal();
        assert_eq!(split.direction, SplitDirection::Horizontal);
    }

    #[test]
    fn test_split_view_panel_switching() {
        let mut split = SplitViewWidget::new();
        split.set_left("left");
        split.set_right("right");

        assert_eq!(split.active_panel, 0);
        assert_eq!(split.active_content(), "left");

        split.switch_panel();
        assert_eq!(split.active_panel, 1);
        assert_eq!(split.active_content(), "right");

        split.switch_panel();
        assert_eq!(split.active_panel, 0);
    }

    #[test]
    fn test_split_view_scrolling() {
        let mut split = SplitViewWidget::new();
        split.set_left("left content");
        split.set_right("right content");

        assert_eq!(split.left_scroll, 0);
        split.scroll_down();
        assert_eq!(split.left_scroll, 1);

        split.scroll_up();
        assert_eq!(split.left_scroll, 0);

        split.switch_panel();
        split.scroll_down();
        assert_eq!(split.right_scroll, 1);
    }

    #[test]
    fn test_tab_widget() {
        let mut tabs = TabWidget::new();
        tabs.add_tab("Tab 1");
        tabs.add_tab("Tab 2");
        tabs.add_tab("Tab 3");

        assert_eq!(tabs.active, 0);
        tabs.select_next();
        assert_eq!(tabs.active, 1);
        tabs.select_prev();
        assert_eq!(tabs.active, 0);
    }

    #[test]
    fn test_tab_widget_with_content() {
        let mut tabs = TabWidget::new();
        tabs.add_tab_with_content("Tab 1", "Content 1");
        tabs.add_tab_with_content("Tab 2", "Content 2");

        assert_eq!(tabs.active_content(), Some(&"Content 1".to_string()));

        tabs.select_next();
        assert_eq!(tabs.active_content(), Some(&"Content 2".to_string()));
    }

    #[test]
    fn test_tab_widget_select_by_index() {
        let mut tabs = TabWidget::new();
        tabs.add_tab("Tab 1");
        tabs.add_tab("Tab 2");
        tabs.add_tab("Tab 3");

        tabs.select_tab(2);
        assert_eq!(tabs.active, 2);

        tabs.select_tab(0);
        assert_eq!(tabs.active, 0);
    }

    #[test]
    fn test_tab_widget_close_tab() {
        let mut tabs = TabWidget::new();
        tabs.add_tab("Tab 1");
        tabs.add_tab("Tab 2");
        tabs.add_tab("Tab 3");

        assert_eq!(tabs.tab_count(), 3);

        tabs.close_tab(1);
        assert_eq!(tabs.tab_count(), 2);
        assert_eq!(tabs.active_tab(), Some(&"Tab 1".to_string()));
    }

    #[test]
    fn test_tab_widget_close_active_tab() {
        let mut tabs = TabWidget::new();
        tabs.add_tab("Tab 1");
        tabs.add_tab("Tab 2");
        tabs.add_tab("Tab 3");

        // Select last tab
        tabs.select_tab(2);
        assert_eq!(tabs.active, 2);
        tabs.close_active_tab();
        assert_eq!(tabs.tab_count(), 2);
        // After closing tab at index 2 (last), active should be adjusted to 1
        assert_eq!(tabs.active, 1);
    }

    #[test]
    fn test_tab_widget_set_content() {
        let mut tabs = TabWidget::new();
        tabs.add_tab("Tab 1");
        tabs.set_active_content("New content");

        assert_eq!(tabs.active_content(), Some(&"New content".to_string()));
    }

    #[test]
    fn test_tab_widget_visible_tabs() {
        let mut tabs = TabWidget::new();
        for i in 0..10 {
            tabs.add_tab(format!("Tab {}", i));
        }

        let visible = tabs.visible_tabs(5);
        assert_eq!(visible.len(), 5);

        tabs.scroll = 3;
        let visible = tabs.visible_tabs(5);
        assert_eq!(visible.len(), 5);
    }

    #[test]
    fn test_tab_widget_clear() {
        let mut tabs = TabWidget::new();
        tabs.add_tab("Tab 1");
        tabs.add_tab("Tab 2");

        tabs.clear();
        assert!(tabs.is_empty());
        assert_eq!(tabs.active, 0);
    }

    #[test]
    fn test_mode_indicator_creation() {
        let indicator = ModeIndicator::new(AppMode::Chat);
        assert_eq!(indicator.mode, AppMode::Chat);
        assert!(indicator.show_shortcut);
    }

    #[test]
    fn test_mode_indicator_display_text() {
        let indicator = ModeIndicator::new(AppMode::Chat);
        let text = indicator.display_text();
        assert!(text.contains("Chat"));
        assert!(text.contains("Ctrl+1"));
    }

    #[test]
    fn test_mode_indicator_short_text() {
        let indicator = ModeIndicator::new(AppMode::Command);
        assert_eq!(indicator.short_text(), "Command");
    }

    #[test]
    fn test_mode_indicator_set_mode() {
        let mut indicator = ModeIndicator::new(AppMode::Chat);
        indicator.set_mode(AppMode::Diff);
        assert_eq!(indicator.mode, AppMode::Diff);
    }

    #[test]
    fn test_mode_indicator_toggle_shortcut() {
        let mut indicator = ModeIndicator::new(AppMode::Chat);
        assert!(indicator.show_shortcut);
        indicator.toggle_shortcut_display();
        assert!(!indicator.show_shortcut);
        indicator.toggle_shortcut_display();
        assert!(indicator.show_shortcut);
    }

    #[test]
    fn test_mode_indicator_get_capabilities() {
        let indicator = ModeIndicator::new(AppMode::Chat);
        let caps = indicator.get_capabilities();
        assert!(caps.contains(&"QuestionAnswering"));
        assert!(caps.contains(&"FreeformChat"));
    }

    #[test]
    fn test_mode_indicator_capabilities_text() {
        let indicator = ModeIndicator::new(AppMode::Command);
        let text = indicator.capabilities_text();
        assert!(text.contains("Capabilities:"));
        assert!(text.contains("CodeGeneration"));
    }

    #[test]
    fn test_mode_indicator_toggle_capabilities() {
        let mut indicator = ModeIndicator::new(AppMode::Chat);
        assert!(!indicator.show_capabilities);
        indicator.toggle_capabilities_display();
        assert!(indicator.show_capabilities);
        indicator.toggle_capabilities_display();
        assert!(!indicator.show_capabilities);
    }

    #[test]
    fn test_mode_indicator_show_hide_capabilities() {
        let mut indicator = ModeIndicator::new(AppMode::Chat);
        assert!(!indicator.show_capabilities);

        indicator.show_capabilities_enabled();
        assert!(indicator.show_capabilities);

        indicator.hide_capabilities_enabled();
        assert!(!indicator.show_capabilities);
    }

    #[test]
    fn test_mode_selection_menu_creation() {
        let menu = ModeSelectionMenu::new();
        assert!(!menu.open);
        assert_eq!(menu.modes.len(), 4);
        assert_eq!(menu.selected_mode(), AppMode::Chat);
    }

    #[test]
    fn test_mode_selection_menu_open_close() {
        let mut menu = ModeSelectionMenu::new();
        assert!(!menu.open);

        menu.open(AppMode::Chat);
        assert!(menu.open);

        menu.close();
        assert!(!menu.open);
    }

    #[test]
    fn test_mode_selection_menu_navigation() {
        let mut menu = ModeSelectionMenu::new();
        menu.open(AppMode::Chat);

        assert_eq!(menu.selected_mode(), AppMode::Chat);
        menu.select_next();
        assert_eq!(menu.selected_mode(), AppMode::Command);
        menu.select_next();
        assert_eq!(menu.selected_mode(), AppMode::Diff);
        menu.select_prev();
        assert_eq!(menu.selected_mode(), AppMode::Command);
    }

    #[test]
    fn test_mode_selection_menu_wrap_around() {
        let mut menu = ModeSelectionMenu::new();
        menu.open(AppMode::Help);

        assert_eq!(menu.selected_mode(), AppMode::Help);
        menu.select_next();
        assert_eq!(menu.selected_mode(), AppMode::Chat);

        menu.select_prev();
        assert_eq!(menu.selected_mode(), AppMode::Help);
    }

    #[test]
    fn test_mode_selection_menu_confirm() {
        let mut menu = ModeSelectionMenu::new();
        menu.open(AppMode::Chat);
        menu.select_next();

        let selected = menu.confirm_switch();
        assert_eq!(selected, AppMode::Command);
        assert!(!menu.open);
    }

    #[test]
    fn test_mode_selection_menu_descriptions() {
        let menu = ModeSelectionMenu::new();
        let descriptions = menu.get_mode_descriptions();
        assert_eq!(descriptions.len(), 4);
        assert!(descriptions[0].1.contains("Chat"));
    }

    #[test]
    fn test_mode_selection_menu_shortcuts() {
        let menu = ModeSelectionMenu::new();
        let shortcuts = menu.get_shortcuts();
        assert_eq!(shortcuts.len(), 4);
        assert_eq!(shortcuts[0].0, "Ctrl+1");
    }

    #[test]
    fn test_vim_keybindings_creation() {
        let vim = VimKeybindings::new();
        assert!(!vim.enabled);
        assert_eq!(vim.mode, VimMode::Normal);
        assert!(vim.command_buffer.is_empty());
    }

    #[test]
    fn test_vim_keybindings_enable_disable() {
        let mut vim = VimKeybindings::new();
        assert!(!vim.enabled);

        vim.enable();
        assert!(vim.enabled);
        assert_eq!(vim.mode, VimMode::Normal);

        vim.disable();
        assert!(!vim.enabled);
    }

    #[test]
    fn test_vim_keybindings_toggle() {
        let mut vim = VimKeybindings::new();
        assert!(!vim.enabled);

        vim.toggle();
        assert!(vim.enabled);

        vim.toggle();
        assert!(!vim.enabled);
    }

    #[test]
    fn test_vim_keybindings_mode_switching() {
        let mut vim = VimKeybindings::new();
        vim.enable();

        assert_eq!(vim.mode, VimMode::Normal);

        vim.enter_insert();
        assert_eq!(vim.mode, VimMode::Insert);

        vim.enter_visual();
        assert_eq!(vim.mode, VimMode::Visual);

        vim.enter_command();
        assert_eq!(vim.mode, VimMode::Command);

        vim.enter_normal();
        assert_eq!(vim.mode, VimMode::Normal);
    }

    #[test]
    fn test_vim_keybindings_command_buffer() {
        let mut vim = VimKeybindings::new();
        vim.enable();
        vim.enter_command();

        vim.add_to_command('w');
        vim.add_to_command('q');

        assert_eq!(vim.get_command(), "wq");

        vim.clear_command();
        assert!(vim.get_command().is_empty());
    }

    #[test]
    fn test_vim_keybindings_mode_checks() {
        let mut vim = VimKeybindings::new();
        vim.enable();

        assert!(vim.is_normal());
        assert!(!vim.is_insert());
        assert!(!vim.is_visual());
        assert!(!vim.is_command());

        vim.enter_insert();
        assert!(!vim.is_normal());
        assert!(vim.is_insert());

        vim.enter_visual();
        assert!(vim.is_visual());

        vim.enter_command();
        assert!(vim.is_command());
    }

    #[test]
    fn test_vim_keybindings_disabled_mode_checks() {
        let vim = VimKeybindings::new();
        assert!(!vim.is_normal());
        assert!(!vim.is_insert());
        assert!(!vim.is_visual());
        assert!(!vim.is_command());
    }

    #[test]
    fn test_vim_keybindings_enter_normal_clears_command() {
        let mut vim = VimKeybindings::new();
        vim.enable();
        vim.enter_command();
        vim.add_to_command('w');
        vim.add_to_command('q');

        assert!(!vim.get_command().is_empty());

        vim.enter_normal();
        assert!(vim.get_command().is_empty());
    }

    #[test]
    fn test_vim_keybindings_disable_resets_mode() {
        let mut vim = VimKeybindings::new();
        vim.enable();
        vim.enter_command();
        vim.add_to_command('t');
        vim.add_to_command('e');
        vim.add_to_command('s');
        vim.add_to_command('t');

        vim.disable();
        assert_eq!(vim.mode, VimMode::Normal);
        assert!(vim.get_command().is_empty());
    }
}