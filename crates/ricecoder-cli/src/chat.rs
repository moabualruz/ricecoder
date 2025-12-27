// Interactive chat mode

use std::sync::Arc;

use ricecoder_providers::provider::Provider;
use tokio::sync::mpsc;

use crate::{
    error::{CliError, CliResult},
    output::OutputStyle,
};

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

/// Command from readline thread
enum ReadlineCommand {
    Line(String),
    Exit,
    Error(String),
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

    /// Get the provider instance for this session
    pub fn get_provider_instance(&self) -> Option<Arc<dyn Provider>> {
        self.provider_instance.clone()
    }

    /// Start interactive chat mode (async version)
    ///
    /// Uses spawn_blocking for readline to avoid blocking the async runtime.
    pub async fn start_async(&mut self) -> CliResult<()> {
        use rustyline::DefaultEditor;

        let style = OutputStyle::default();
        println!("{}", style.header("Chat Mode"));
        println!("{}", style.key_value("Provider", &self.provider));
        println!("{}", style.key_value("Model", &self.model));
        println!("{}", style.info("Type 'exit' to quit"));
        println!();

        // Create channel for readline communication
        let (tx, mut rx) = mpsc::channel::<ReadlineCommand>(1);

        // Spawn blocking task for readline
        let readline_handle = tokio::task::spawn_blocking(move || {
            let mut rl = match DefaultEditor::new() {
                Ok(rl) => rl,
                Err(e) => {
                    let _ = tx.blocking_send(ReadlineCommand::Error(e.to_string()));
                    return;
                }
            };

            loop {
                match rl.readline("r[ > ") {
                    Ok(line) => {
                        if tx.blocking_send(ReadlineCommand::Line(line)).is_err() {
                            break;
                        }
                    }
                    Err(_) => {
                        let _ = tx.blocking_send(ReadlineCommand::Exit);
                        break;
                    }
                }
            }
        });

        // Process commands from readline
        while let Some(cmd) = rx.recv().await {
            match cmd {
                ReadlineCommand::Line(line) => {
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

                        // Send to provider and get response (async, no new runtime)
                        match self.send_message_to_provider_async(&line).await {
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
                ReadlineCommand::Exit => {
                    println!("{}", style.success("Goodbye!"));
                    break;
                }
                ReadlineCommand::Error(e) => {
                    return Err(CliError::Internal(e));
                }
            }
        }

        // Clean up readline task
        readline_handle.abort();

        Ok(())
    }

    /// Start interactive chat mode (sync wrapper for backward compatibility)
    ///
    /// NOTE: This creates a new runtime. Prefer `start_async()` when already in async context.
    pub fn start(&mut self) -> CliResult<()> {
        // Use Handle::current() if we're already in a runtime, otherwise create one
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                // We're in an async context - use block_on with the current handle
                // This is safe because start_async uses spawn_blocking for readline
                tokio::task::block_in_place(|| handle.block_on(self.start_async()))
            }
            Err(_) => {
                // No runtime - create a new multi-threaded one
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .map_err(|e| CliError::Internal(format!("Failed to create runtime: {}", e)))?;
                rt.block_on(self.start_async())
            }
        }
    }

    /// Simulate streaming by typing characters one at a time
    fn simulate_streaming(text: &str) -> CliResult<()> {
        use std::{thread, time::Duration};

        for ch in text.chars() {
            print!("{}", ch);
            std::io::Write::flush(&mut std::io::stdout())
                .map_err(|e| CliError::Internal(e.to_string()))?;
            thread::sleep(Duration::from_millis(10));
        }
        Ok(())
    }

    /// Async version of send_message_to_provider
    pub async fn send_message_to_provider_async(&self, _message: &str) -> CliResult<String> {
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
