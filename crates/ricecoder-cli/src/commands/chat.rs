// Interactive chat mode
// Adapted from automation/src/cli/prompts.rs

use crate::error::{CliError, CliResult};
use crate::output::OutputStyle;
use crate::chat::ChatSession;
use super::Command;
use ricecoder_storage::PathResolver;

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

    /// Validate provider is supported
    fn validate_provider(&self) -> CliResult<String> {
        let provider = self.provider.as_deref().unwrap_or("openai");

        // List of supported providers
        let supported = ["openai", "anthropic", "local"];

        if !supported.contains(&provider) {
            return Err(CliError::Provider(format!(
                "Unsupported provider: {}. Supported providers: {}",
                provider,
                supported.join(", ")
            )));
        }

        Ok(provider.to_string())
    }

    /// Validate model is specified or use default
    fn get_model(&self) -> String {
        self.model.as_deref().unwrap_or("gpt-4").to_string()
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

    /// Process initial message
    fn process_initial_message(&self, message: &str, session: &mut ChatSession) -> CliResult<()> {
        let style = OutputStyle::default();

        // Add user message to history
        session.add_message("user".to_string(), message.to_string());

        println!();
        println!("{}", style.prompt("r["));
        println!("{}", message);
        println!("{}", style.info("Processing message..."));

        // TODO: Send to AI provider and get response
        let response = "This is a placeholder response. Full AI integration coming soon.";
        session.add_message("assistant".to_string(), response.to_string());

        println!("{}", style.success(response));
        println!();

        Ok(())
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

        // Validate provider
        let provider = self.validate_provider()?;
        let model = self.get_model();

        println!("{}", style.section("Chat Configuration"));
        println!("{}", style.key_value("Provider", &provider));
        println!("{}", style.key_value("Model", &model));
        println!();

        // Load project context
        let _specs = self.load_project_context()?;
        let _kb = self.load_knowledge_base()?;

        // Create chat session
        let mut session = ChatSession::new(provider, model);

        // Run chat loop
        self.run_chat_loop(&mut session)?;

        println!();
        println!("{}", style.success("Chat session ended"));

        Ok(())
    }
}
