use ricecoder_tui::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_popup_type_name() {
        assert_eq!(PopupType::Confirmation.name(), "Confirmation");
        assert_eq!(PopupType::Information.name(), "Information");
        assert_eq!(PopupType::Error.name(), "Error");
    }

    #[test]
    fn test_popup_button_creation() {
        let button = PopupButton::new("OK", "ok");
        assert_eq!(button.label, "OK");
        assert_eq!(button.id, "ok");
        assert!(!button.is_default);
    }

    #[test]
    fn test_popup_button_default() {
        let button = PopupButton::new("OK", "ok").with_default(true);
        assert!(button.is_default);
    }

    #[test]
    fn test_popup_widget_creation() {
        let popup = PopupWidget::new(PopupType::Information, "Title", "Message");
        assert_eq!(popup.title(), "Title");
        assert_eq!(popup.message(), "Message");
        assert!(!popup.is_visible());
    }

    #[test]
    fn test_popup_widget_confirmation() {
        let popup = PopupWidget::confirmation("Confirm", "Are you sure?");
        assert_eq!(popup.popup_type(), PopupType::Confirmation);
        assert_eq!(popup.buttons().len(), 2);
    }

    #[test]
    fn test_popup_widget_information() {
        let popup = PopupWidget::information("Info", "Information message");
        assert_eq!(popup.popup_type(), PopupType::Information);
        assert_eq!(popup.buttons().len(), 1);
    }

    #[test]
    fn test_popup_widget_error() {
        let popup = PopupWidget::error("Error", "An error occurred");
        assert_eq!(popup.popup_type(), PopupType::Error);
        assert_eq!(popup.buttons().len(), 1);
    }

    #[test]
    fn test_popup_widget_input() {
        let popup = PopupWidget::input("Input", "Enter text");
        assert_eq!(popup.popup_type(), PopupType::Input);
        assert_eq!(popup.buttons().len(), 2);
    }

    #[test]
    fn test_popup_widget_add_button() {
        let mut popup = PopupWidget::new(PopupType::Information, "Title", "Message");
        popup.add_button(PopupButton::new("Button 1", "btn1"));
        popup.add_button(PopupButton::new("Button 2", "btn2"));

        assert_eq!(popup.buttons().len(), 2);
    }

    #[test]
    fn test_popup_widget_button_selection() {
        let mut popup = PopupWidget::confirmation("Confirm", "Are you sure?");
        assert_eq!(popup.selected_button_id(), Some("yes"));

        popup.select_next_button();
        assert_eq!(popup.selected_button_id(), Some("no"));

        popup.select_prev_button();
        assert_eq!(popup.selected_button_id(), Some("yes"));
    }

    #[test]
    fn test_popup_widget_select_button_by_id() {
        let mut popup = PopupWidget::confirmation("Confirm", "Are you sure?");
        assert!(popup.select_button_by_id("no"));
        assert_eq!(popup.selected_button_id(), Some("no"));

        assert!(!popup.select_button_by_id("invalid"));
    }

    #[test]
    fn test_popup_widget_visibility() {
        let mut popup = PopupWidget::new(PopupType::Information, "Title", "Message");
        assert!(!popup.is_visible());

        popup.show();
        assert!(popup.is_visible());

        popup.hide();
        assert!(!popup.is_visible());
    }

    #[test]
    fn test_popup_widget_input_text() {
        let mut popup = PopupWidget::input("Input", "Enter text");
        assert_eq!(popup.input_text(), "");

        popup.set_input_text("Hello");
        assert_eq!(popup.input_text(), "Hello");

        popup.append_input('!');
        assert_eq!(popup.input_text(), "Hello!");

        popup.backspace_input();
        assert_eq!(popup.input_text(), "Hello");

        popup.clear_input();
        assert_eq!(popup.input_text(), "");
    }

    #[test]
    fn test_popup_widget_dimensions() {
        let mut popup = PopupWidget::new(PopupType::Information, "Title", "Message");
        popup.set_max_width(100);
        popup.set_max_height(50);

        assert_eq!(popup.max_width(), 100);
        assert_eq!(popup.max_height(), 50);
    }

    #[test]
    fn test_popup_widget_format_display() {
        let popup = PopupWidget::confirmation("Confirm", "Are you sure?");
        let display = popup.format_display();
        assert!(display.contains("Confirm"));
        assert!(display.contains("Are you sure?"));
        assert!(display.contains("Yes"));
        assert!(display.contains("No"));
    }

    #[test]
    fn test_popup_widget_message_update() {
        let mut popup = PopupWidget::new(PopupType::Information, "Title", "Original message");
        assert_eq!(popup.message(), "Original message");

        popup.set_message("Updated message");
        assert_eq!(popup.message(), "Updated message");
    }
}
