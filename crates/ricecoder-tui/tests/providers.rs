use ricecoder_tui::providers::ProviderFactory;

#[tokio::test]
async fn test_provider_factory_creation() {
    let factory = ProviderFactory::new();

    // Test that providers can be created
    let theme_provider = factory.theme_provider();
    let config_provider = factory.config_provider();
    let session_provider = factory.session_provider();
    let command_provider = factory.command_provider();
    let help_provider = factory.help_provider();
    let keybind_provider = factory.keybind_provider();

    // Test basic functionality (these are mock implementations)
    let theme = theme_provider.get_current_theme().await.unwrap();
    assert_eq!(theme.name, "dark");

    let config = config_provider.get_config().await.unwrap();
    assert_eq!(config.ui.theme, "dark");

    let sessions = session_provider.get_active_sessions().await.unwrap();
    assert!(sessions.is_empty());

    let result = command_provider.execute_command(ricecoder_commands::Command::Save).await.unwrap();
    assert!(result.success);

    let help = help_provider.get_help_content("test").await.unwrap();
    assert!(help.is_none());

    let keybind = keybind_provider.get_keybinding("test").await.unwrap();
    assert!(keybind.is_none());
}

#[tokio::test]
async fn test_provider_error_handling() {
    let factory = ProviderFactory::new();
    let theme_provider = factory.theme_provider();

    // Test that providers handle errors gracefully
    // (Mock implementations currently don't error, but this tests the interface)
    let result = theme_provider.get_current_theme().await;
    assert!(result.is_ok());
}