use ricecoder_tui::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scrollview_creation() {
        let view = ScrollViewWidget::new("Test");
        assert_eq!(view.message_count(), 0);
        assert_eq!(view.scroll_offset(), 0);
        assert!(view.selected().is_none());
    }

    #[test]
    fn test_add_message() {
        let mut view = ScrollViewWidget::new("Test");
        view.add_message("Message 1");
        view.add_message("Message 2");

        assert_eq!(view.message_count(), 2);
    }

    #[test]
    fn test_clear() {
        let mut view = ScrollViewWidget::new("Test");
        view.add_message("Message 1");
        view.add_message("Message 2");
        view.select(0);

        view.clear();
        assert_eq!(view.message_count(), 0);
        assert!(view.selected().is_none());
    }

    #[test]
    fn test_scroll_up() {
        let mut view = ScrollViewWidget::new("Test");
        for i in 0..10 {
            view.add_message(format!("Message {}", i));
        }

        view.set_scroll_offset(5);
        view.scroll_up();
        assert_eq!(view.scroll_offset(), 4);

        view.scroll_to_top();
        view.scroll_up();
        assert_eq!(view.scroll_offset(), 0);
    }

    #[test]
    fn test_scroll_down() {
        let mut view = ScrollViewWidget::new("Test");
        for i in 0..10 {
            view.add_message(format!("Message {}", i));
        }

        view.scroll_down(5);
        assert_eq!(view.scroll_offset(), 1);

        view.scroll_to_bottom(5);
        assert_eq!(view.scroll_offset(), 5);
    }

    #[test]
    fn test_selection() {
        let mut view = ScrollViewWidget::new("Test");
        view.add_message("Message 1");
        view.add_message("Message 2");
        view.add_message("Message 3");

        view.select(1);
        assert_eq!(view.selected(), Some(1));
        assert_eq!(view.selected_message(), Some("Message 2"));

        view.deselect();
        assert!(view.selected().is_none());
    }

    #[test]
    fn test_select_next() {
        let mut view = ScrollViewWidget::new("Test");
        view.add_message("Message 1");
        view.add_message("Message 2");
        view.add_message("Message 3");

        assert!(view.selected().is_none());

        view.select_next();
        assert_eq!(view.selected(), Some(0));

        view.select_next();
        assert_eq!(view.selected(), Some(1));

        view.select_next();
        assert_eq!(view.selected(), Some(2));

        view.select_next();
        assert_eq!(view.selected(), Some(2)); // Stay at last
    }

    #[test]
    fn test_select_prev() {
        let mut view = ScrollViewWidget::new("Test");
        view.add_message("Message 1");
        view.add_message("Message 2");
        view.add_message("Message 3");

        view.select(2);
        view.select_prev();
        assert_eq!(view.selected(), Some(1));

        view.select_prev();
        assert_eq!(view.selected(), Some(0));

        view.select_prev();
        assert!(view.selected().is_none());
    }

    #[test]
    fn test_visible_messages() {
        let mut view = ScrollViewWidget::new("Test");
        for i in 0..10 {
            view.add_message(format!("Message {}", i));
        }

        let visible = view.visible_messages(5);
        assert_eq!(visible.len(), 5);
        assert_eq!(visible[0], "Message 0");

        view.set_scroll_offset(5);
        let visible = view.visible_messages(5);
        assert_eq!(visible.len(), 5);
        assert_eq!(visible[0], "Message 5");
    }

    #[test]
    fn test_scroll_percentage() {
        let mut view = ScrollViewWidget::new("Test");
        for i in 0..10 {
            view.add_message(format!("Message {}", i));
        }

        assert_eq!(view.scroll_percentage(5), 0);

        view.set_scroll_offset(5);
        assert_eq!(view.scroll_percentage(5), 100);

        view.set_scroll_offset(2);
        let percentage = view.scroll_percentage(5);
        assert!(percentage > 0 && percentage < 100);
    }

    #[test]
    fn test_is_at_top_bottom() {
        let mut view = ScrollViewWidget::new("Test");
        for i in 0..10 {
            view.add_message(format!("Message {}", i));
        }

        assert!(view.is_at_top());
        assert!(!view.is_at_bottom(5));

        view.scroll_to_bottom(5);
        assert!(!view.is_at_top());
        assert!(view.is_at_bottom(5));
    }

    #[test]
    fn test_total_height() {
        let mut view = ScrollViewWidget::new("Test");
        assert_eq!(view.total_height(), 0);

        for i in 0..10 {
            view.add_message(format!("Message {}", i));
        }
        assert_eq!(view.total_height(), 10);
    }

    #[test]
    fn test_set_title() {
        let mut view = ScrollViewWidget::new("Original");
        assert_eq!(view.title, "Original");

        view.set_title("New Title");
        assert_eq!(view.title, "New Title");
    }

    #[test]
    fn test_set_show_borders() {
        let mut view = ScrollViewWidget::new("Test");
        assert!(view.show_borders);

        view.set_show_borders(false);
        assert!(!view.show_borders);
    }

    #[test]
    fn test_scroll_page_functions() {
        let mut view = ScrollViewWidget::new("Test");
        for i in 0..20 {
            view.add_message(format!("Message {}", i));
        }

        // Test page up
        view.set_scroll_offset(10);
        view.scroll_page_up(5);
        assert_eq!(view.scroll_offset(), 5);

        // Test page down
        view.scroll_page_down(5, 10);
        assert_eq!(view.scroll_offset(), 10);

        // Test half page scroll
        view.scroll_half_page_up(2);
        assert_eq!(view.scroll_offset(), 8);

        view.scroll_half_page_down(2, 10);
        assert_eq!(view.scroll_offset(), 10);
    }

    #[test]
    fn test_scroll_position_percentage() {
        let mut view = ScrollViewWidget::new("Test");
        for i in 0..10 {
            view.add_message(format!("Message {}", i));
        }

        assert_eq!(view.scroll_position_percentage(5), 0);

        view.set_scroll_offset(5);
        assert_eq!(view.scroll_position_percentage(5), 100);

        view.set_scroll_offset(2);
        let percentage = view.scroll_position_percentage(5);
        assert!(percentage > 0 && percentage < 100);
    }

    #[test]
    fn test_scroll_bar_info() {
        let mut view = ScrollViewWidget::new("Test");
        for i in 0..10 {
            view.add_message(format!("Message {}", i));
        }

        let info = view.scroll_bar_info(5);
        assert_eq!(info.position, 0);
        assert_eq!(info.total_content_height, 10);
        assert_eq!(info.visible_height, 5);
    }

    #[test]
    fn test_auto_scroll() {
        let mut view = ScrollViewWidget::new("Test");
        for i in 0..10 {
            view.add_message(format!("Message {}", i));
        }

        // Scroll to bottom to enable auto-scroll
        view.scroll_to_bottom(5);
        assert!(view.is_auto_scroll(5));

        // Scroll up, should disable auto-scroll
        view.set_scroll_offset(3);
        assert!(!view.is_auto_scroll(5));

        // Enable auto-scroll again
        view.set_auto_scroll(true, 5);
        assert!(view.is_auto_scroll(5));
    }

    #[test]
    fn test_mouse_wheel() {
        let mut view = ScrollViewWidget::new("Test");
        for i in 0..10 {
            view.add_message(format!("Message {}", i));
        }

        // Scroll down with mouse wheel
        view.handle_mouse_wheel(-3, 5);
        assert_eq!(view.scroll_offset(), 3);

        // Scroll up with mouse wheel
        view.handle_mouse_wheel(1, 5);
        assert_eq!(view.scroll_offset(), 2);
    }

    #[test]
    fn test_scroll_state() {
        let mut view = ScrollViewWidget::new("Test");
        // Add more messages so we can actually scroll
        for i in 0..10 {
            view.add_message(format!("Message {}", i));
        }

        view.set_scroll_offset(2);
        let state = view.scroll_state(5); // Visible height of 5
        assert_eq!(state.position(), 2);

        // Restore state
        let mut new_view = ScrollViewWidget::new("Test2");
        for i in 0..10 {
            new_view.add_message(format!("Message {}", i));
        }
        new_view.restore_scroll_state(&state);
        assert_eq!(new_view.scroll_offset(), 2);
    }
}