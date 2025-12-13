use ricecoder_cli::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_handler_list_empty() {
        let handler = CustomCommandHandler::new(CustomAction::List);
        // Should not panic and should handle empty registry gracefully
        let result = handler.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_custom_handler_info_not_found() {
        let handler = CustomCommandHandler::new(CustomAction::Info("nonexistent".to_string()));
        let result = handler.execute();
        assert!(result.is_err());
    }

    #[test]
    fn test_custom_handler_run_not_found() {
        let handler =
            CustomCommandHandler::new(CustomAction::Run("nonexistent".to_string(), vec![]));
        let result = handler.execute();
        assert!(result.is_err());
    }

    #[test]
    fn test_custom_handler_search_empty() {
        let handler = CustomCommandHandler::new(CustomAction::Search("test".to_string()));
        // Should not panic and should handle empty results gracefully
        let result = handler.execute();
        assert!(result.is_ok());
    }
}