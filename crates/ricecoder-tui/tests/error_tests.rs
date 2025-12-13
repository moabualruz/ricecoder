use ricecoder_tui::*;

mod tests {
    use super::*;

    #[test]
    fn test_tui_error_constructors() {
        assert!(matches!(TuiError::config("test"), TuiError::Config { .. }));
        assert!(matches!(TuiError::theme("test"), TuiError::Theme { .. }));
        assert!(matches!(TuiError::render("test"), TuiError::Render { .. }));
        assert!(matches!(TuiError::widget("test"), TuiError::Widget { .. }));
        assert!(matches!(TuiError::event("test"), TuiError::Event { .. }));
        assert!(matches!(TuiError::command("test"), TuiError::Command { .. }));
        assert!(matches!(TuiError::vcs("test"), TuiError::Vcs { .. }));
        assert!(matches!(TuiError::lsp("test"), TuiError::Lsp { .. }));
        assert!(matches!(TuiError::terminal("test"), TuiError::Terminal { .. }));
        assert!(matches!(TuiError::image("test"), TuiError::Image { .. }));
        assert!(matches!(TuiError::markdown("test"), TuiError::Markdown { .. }));
        assert!(matches!(TuiError::toml("test"), TuiError::Toml { .. }));
        assert!(matches!(TuiError::task("test"), TuiError::Task { .. }));
        assert!(matches!(TuiError::performance("test"), TuiError::Performance { .. }));
        assert!(matches!(TuiError::accessibility("test"), TuiError::Accessibility { .. }));
        assert!(matches!(TuiError::security("test"), TuiError::Security { .. }));
        assert!(matches!(TuiError::network("test"), TuiError::Network { .. }));
        assert!(matches!(TuiError::validation("field", "msg"), TuiError::Validation { .. }));
        assert!(matches!(TuiError::state("test"), TuiError::State { .. }));
        assert!(matches!(TuiError::init("test"), TuiError::Init { .. }));
        assert!(matches!(TuiError::shutdown("test"), TuiError::Shutdown { .. }));
        assert!(matches!(TuiError::timeout("op", 1000), TuiError::Timeout { .. }));
        assert!(matches!(TuiError::cancelled("op"), TuiError::Cancelled { .. }));
        assert!(matches!(TuiError::resource_exhausted("res"), TuiError::ResourceExhausted { .. }));
        assert!(matches!(TuiError::version("test"), TuiError::Version { .. }));
        assert!(matches!(TuiError::plugin("test"), TuiError::Plugin { .. }));
        assert!(matches!(TuiError::internal("test"), TuiError::Internal { .. }));
    }

    #[test]
    fn test_error_conversion() {
        // Test ProviderError conversion
        let provider_err = ricecoder_providers::ProviderError::NotFound("test".to_string());
        let tui_err: TuiError = provider_err.into();
        assert!(matches!(tui_err, TuiError::Provider(_)));

        // Test StorageError conversion
        let storage_err = ricecoder_storage::StorageError::internal("test");
        let tui_err: TuiError = storage_err.into();
        assert!(matches!(tui_err, TuiError::Storage(_)));

        // Test KeybindError conversion
        let keybind_err = ricecoder_keybinds::error::EngineError::NotInitialized;
        let tui_err: TuiError = keybind_err.into();
        assert!(matches!(tui_err, TuiError::Keybind(_)));
    }

    #[test]
    fn test_session_error_display() {
        let err = SessionError::NotFound { id: "test".to_string() };
        assert!(err.to_string().contains("Session not found: test"));
    }

    #[test]
    fn test_tool_error_display() {
        let err = ToolError::ExecutionFailed {
            name: "test_tool".to_string(),
            message: "failed".to_string(),
        };
        assert!(err.to_string().contains("Tool execution failed: test_tool - failed"));
    }
}