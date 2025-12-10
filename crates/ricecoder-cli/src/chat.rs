// Interactive chat mode

use crate::error::{CliError, CliResult};
use crate::output::OutputStyle;
use rustyline::DefaultEditor;
use std::sync::Arc;
use ricecoder_providers::provider::Provider;

/// Chat session manager
pub struct ChatSession {
    pub provider: String,
    pub model: String,
    pub history: Vec<ChatMessage>,
    provider_instance: Option<Arc<dyn Provider>>,
}

/// A single chat message
#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatSession {
    /// Create a new chat session
    pub fn new(provider: String, model: String) -> Self {
        Self {
            provider,
            model,
            history: Vec::new(),
            provider_instance: None,
        }
    }

    /// Set the provider instance for this session
    pub fn set_provider(&mut self, provider: Arc<dyn Provider>) {
        self.provider_instance = Some(provider);
    }

    /// Start interactive chat mode
    pub fn start(&mut self) -> CliResult<()> {
        let mut rl =
            DefaultEditor::new().map_err(|e| crate::error::CliError::Internal(e.to_string()))?;

        let style = OutputStyle::default();
        println!("{}", style.header("Chat Mode"));
        println!("{}", style.key_value("Provider", &self.provider));
        println!("{}", style.key_value("Model", &self.model));
        println!("{}", style.info("Type 'exit' to quit"));
        println!();

        loop {
            let readline = rl.readline("r[ > ");
            match readline {
                Ok(line) => {
                    if line.trim() == "exit" {
                        println!("{}", style.success("Goodbye!"));
                        break;
                    }

                    if !line.trim().is_empty() {
                        // Add user message to history
                        self.history.push(ChatMessage {
                            role: "user".to_string(),
                            content: line.clone(),
                        });

                        // Send to provider and get response
                        match self.send_message_to_provider(&line) {
                            Ok(response) => {
                                // Add assistant response to history
                                self.history.push(ChatMessage {
                                    role: "assistant".to_string(),
                                    content: response.clone(),
                                });

                                // Display response
                                println!();
                                println!("{}", style.success(&response));
                                println!();
                            }
                            Err(e) => {
                                println!("{}", style.error(&format!("Error: {}", e)));
                                println!();
                            }
                        }
                    }
                }
                Err(_) => {
                    println!("{}", style.success("Goodbye!"));
                    break;
                }
            }
        }

        Ok(())
    }

    /// Send a message to the provider and get response
    fn send_message_to_provider(&self, message: &str) -> CliResult<String> {
        // Create a runtime for async operations
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| CliError::Internal(format!("Failed to create runtime: {}", e)))?;

        rt.block_on(self.send_message_to_provider_async(message))
    }

    /// Simulate streaming by typing characters one at a time
    fn simulate_streaming(text: &str) -> CliResult<()> {
        use std::thread;
        use std::time::Duration;
        
        for ch in text.chars() {
            print!("{}", ch);
            std::io::Write::flush(&mut std::io::stdout())
                .map_err(|e| CliError::Internal(e.to_string()))?;
            thread::sleep(Duration::from_millis(10));
        }
        Ok(())
    }

    /// Async version of send_message_to_provider
    async fn send_message_to_provider_async(&self, _message: &str) -> CliResult<String> {
        use futures::stream::StreamExt;

        // Get provider instance
        let provider = self
            .provider_instance
            .as_ref()
            .ok_or_else(|| CliError::Provider("Provider not initialized".to_string()))?;

        // Create chat request with conversation history
        let mut messages = Vec::new();
        for msg in &self.history {
            messages.push(ricecoder_providers::models::Message {
                role: msg.role.clone(),
                content: msg.content.clone(),
            });
        }

        let request = ricecoder_providers::models::ChatRequest {
            model: self.model.clone(),
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

                // Simulate streaming by typing character by character
                Self::simulate_streaming(&response.content)?;
                Ok(response.content)
            }
        }
    }

    /// Add a message to history
    pub fn add_message(&mut self, role: String, content: String) {
        self.history.push(ChatMessage { role, content });
    }

    /// Get chat history
    pub fn get_history(&self) -> &[ChatMessage] {
        &self.history
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_session_creation() {
        let session = ChatSession::new("openai".to_string(), "gpt-4".to_string());
        assert_eq!(session.provider, "openai");
        assert_eq!(session.model, "gpt-4");
        assert!(session.history.is_empty());
    }

    #[test]
    fn test_add_message() {
        let mut session = ChatSession::new("openai".to_string(), "gpt-4".to_string());
        session.add_message("user".to_string(), "Hello".to_string());
        assert_eq!(session.history.len(), 1);
        assert_eq!(session.history[0].role, "user");
        assert_eq!(session.history[0].content, "Hello");
    }

    #[test]
    fn test_get_history() {
        let mut session = ChatSession::new("openai".to_string(), "gpt-4".to_string());
        session.add_message("user".to_string(), "Hello".to_string());
        session.add_message("assistant".to_string(), "Hi there!".to_string());

        let history = session.get_history();
        assert_eq!(history.len(), 2);
    }
}
