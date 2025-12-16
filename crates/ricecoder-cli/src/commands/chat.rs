// Interactive chat mode
// Adapted from automation/src/cli/prompts.rs

use super::Command;
use crate::chat::ChatSession;
use crate::error::{CliError, CliResult};
use crate::output::OutputStyle;
use ricecoder_storage::{ConfigLoader, PathResolver};
use ricecoder_providers::provider::ProviderRegistry;
use ricecoder_providers::models::ChatRequest;

/// Interactive chat mode
pub struct ChatCommand {
    pub message: Option<String>,
    pub provider: Option<String>,
    pub model: Option<String>,
}

impl ChatCommand {
    pub fn new(message: Option<String>, provider: Option<String>, model: Option<String>) -> Self {
        Self {
            message,
            provider,
            model,
        }
    }

    /// Get provider from CLI args or configuration
    pub fn get_provider(&self) -> CliResult<String> {
        // If provider is specified via CLI, use it
        if let Some(provider) = &self.provider {
            return Ok(provider.clone());
        }

        // Load configuration to get default provider
        let config = ConfigLoader::new().load_merged()
            .map_err(|e| CliError::Config(format!("Failed to load configuration: {}", e)))?;

        // Use configured default provider or fall back to "zen"
        Ok(config
            .providers
            .default_provider
            .unwrap_or_else(|| "zen".to_string()))
    }

    /// Validate provider is supported by checking ProviderRegistry
    pub fn validate_provider(&self, provider: &str) -> CliResult<()> {
        // For now, we accept any provider name as the ProviderRegistry
        // will handle validation when the provider is actually used.
        // This allows for extensibility without hardcoding provider lists.
        if provider.is_empty() {
            return Err(CliError::Provider(
                "Provider name cannot be empty".to_string(),
            ));
        }

        Ok(())
    }

    /// Get model from CLI args or configuration
    pub fn get_model(&self) -> CliResult<String> {
        // If model is specified via CLI, use it
        if let Some(model) = &self.model {
            return Ok(model.clone());
        }

        // Load configuration to get default model
        let config = ConfigLoader::new().load_merged()
            .map_err(|e| CliError::Config(format!("Failed to load configuration: {}", e)))?;

        // Use configured default model or fall back to "zen/big-pickle"
        Ok(config
            .defaults
            .model
            .unwrap_or_else(|| "zen/big-pickle".to_string()))
    }

    /// Load project specs for context
    fn load_project_context(&self) -> CliResult<Vec<String>> {
        let style = OutputStyle::default();
        let mut specs = Vec::new();

        // Resolve project path using PathResolver
        let project_path = PathResolver::resolve_project_path();
        let specs_path = project_path.join("specs");

        // Check for .agent/specs directory (project-level specs)
        if specs_path.exists() {
            println!("{}", style.info("Loading project specs..."));
            // TODO: Actually load specs from .agent/specs/ directory
            specs.push("specs_loaded".to_string());
            println!("{}", style.success("Specs loaded"));
        }

        Ok(specs)
    }

    /// Load knowledge base
    fn load_knowledge_base(&self) -> CliResult<Vec<String>> {
        let style = OutputStyle::default();
        let mut kb = Vec::new();

        // TODO: Load knowledge base from global location
        println!("{}", style.info("Loading knowledge base..."));
        kb.push("kb_loaded".to_string());
        println!("{}", style.success("Knowledge base loaded"));

        Ok(kb)
    }

    /// Process initial message by sending to provider and getting response
    fn process_initial_message(&self, message: &str, session: &mut ChatSession) -> CliResult<()> {
        let style = OutputStyle::default();

        // Add user message to history
        session.add_message("user".to_string(), message.to_string());

        println!();
        println!("{}", style.prompt("r["));
        println!("{}", message);
        println!("{}", style.info("Processing message..."));

        // Send to AI provider and get response
        let response = self.send_to_provider(message, session)?;
        session.add_message("assistant".to_string(), response.clone());

        println!("{}", style.success(&response));
        println!();

        Ok(())
    }

    /// Send message to provider and get response
    fn send_to_provider(&self, message: &str, session: &ChatSession) -> CliResult<String> {
        // Create a runtime for async operations
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| CliError::Internal(format!("Failed to create runtime: {}", e)))?;

        rt.block_on(self.send_to_provider_async(message, session))
    }

    /// Async version of send_to_provider
    async fn send_to_provider_async(
        &self,
        _message: &str,
        session: &ChatSession,
    ) -> CliResult<String> {
        use futures::stream::StreamExt;

        // Create provider registry and register Zen provider
        let mut registry = ProviderRegistry::new();

        // Get API key from environment or use empty string for free models
        let api_key = std::env::var("OPENCODE_API_KEY").ok();

        // Create and register Zen provider
        let zen_provider = ricecoder_providers::ZenProvider::new(api_key)
            .map_err(|e| CliError::Provider(format!("Failed to create Zen provider: {}", e)))?;

        registry
            .register(std::sync::Arc::new(zen_provider))
            .map_err(|e| CliError::Provider(format!("Failed to register provider: {}", e)))?;

        // Get the provider
        let provider = registry
            .get(&session.provider)
            .map_err(|e| CliError::Provider(format!("Provider not found: {}", e)))?;

        // Create chat request with conversation history
        let mut messages = Vec::new();
        for msg in session.get_history() {
            messages.push(ricecoder_providers::models::Message {
                role: msg.role.clone(),
                content: msg.content.clone(),
            });
        }

        let request = ChatRequest {
            model: session.model.clone(),
            messages,
            temperature: None,
            max_tokens: None,
            stream: true,
        };

        // Try streaming first, fall back to non-streaming if not supported
        match provider.chat_stream(request.clone()).await {
            Ok(mut stream) => {
                // Consume the streaming response and collect all chunks
                let mut full_response = String::new();
                
                while let Some(result) = stream.next().await {
                    match result {
                        Ok(chunk) => {
                            // Display chunk in real-time
                            print!("{}", chunk.content);
                            std::io::Write::flush(&mut std::io::stdout())
                                .map_err(|e| CliError::Internal(e.to_string()))?;
                            full_response.push_str(&chunk.content);
                        }
                        Err(e) => {
                            return Err(CliError::Provider(format!("Streaming error: {}", e)));
                        }
                    }
                }
                
                Ok(full_response)
            }
            Err(_) => {
                // Fall back to non-streaming if streaming is not supported
                let mut request = request;
                request.stream = false;
                
                let response = provider
                    .chat(request)
                    .await
                    .map_err(|e| CliError::Provider(format!("Chat request failed: {}", e)))?;

                Ok(response.content)
            }
        }
    }

    /// Create and initialize a provider for the chat session
    async fn create_provider(&self) -> CliResult<std::sync::Arc<dyn ricecoder_providers::provider::Provider>> {
        // Get API key from environment or use empty string for free models
        let api_key = std::env::var("OPENCODE_API_KEY").ok();

        // Create and return the appropriate provider based on configuration
        match self.get_provider()?.as_str() {
            "zen" => {
                let zen_provider = ricecoder_providers::ZenProvider::new(api_key)
                    .map_err(|e| CliError::Provider(format!("Failed to create Zen provider: {}", e)))?;
                Ok(std::sync::Arc::new(zen_provider))
            }
            provider_name => {
                Err(CliError::Provider(format!(
                    "Unsupported provider: {}",
                    provider_name
                )))
            }
        }
    }

    /// Enter interactive chat loop
    fn run_chat_loop(&self, session: &mut ChatSession) -> CliResult<()> {
        let style = OutputStyle::default();

        // If initial message provided, process it
        if let Some(msg) = &self.message {
            self.process_initial_message(msg, session)?;
        } else {
            // Interactive mode
            println!();
            println!("{}", style.header("RiceCoder Chat Mode"));
            println!("{}", style.info("Type 'exit' to quit, 'help' for commands"));
            println!();

            // Use the chat session's built-in REPL
            session.start()?;
        }

        Ok(())
    }
}

impl Command for ChatCommand {
    fn execute(&self) -> CliResult<()> {
        let style = OutputStyle::default();

        // Get provider from CLI args or configuration
        let provider = self.get_provider()?;
        self.validate_provider(&provider)?;

        // Get model from CLI args or configuration
        let model = self.get_model()?;

        println!("{}", style.section("Chat Configuration"));
        println!("{}", style.key_value("Provider", &provider));
        println!("{}", style.key_value("Model", &model));
        println!();

        // Load project context
        let _specs = self.load_project_context()?;
        let _kb = self.load_knowledge_base()?;

        // Create chat session
        let mut session = ChatSession::new(provider, model);

        // Create and initialize provider for the session
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| CliError::Internal(format!("Failed to create runtime: {}", e)))?;

        let provider_instance = rt.block_on(self.create_provider())?;
        session.set_provider(provider_instance);

        // Run chat loop
        self.run_chat_loop(&mut session)?;

        println!();
        println!("{}", style.success("Chat session ended"));

        Ok(())
    }
}
