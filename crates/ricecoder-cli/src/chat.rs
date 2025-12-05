// Interactive chat mode

use crate::error::CliResult;
use rustyline::DefaultEditor;

/// Chat session manager
pub struct ChatSession {
    pub provider: String,
    pub model: String,
    pub history: Vec<ChatMessage>,
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
        }
    }

    /// Start interactive chat mode
    pub fn start(&mut self) -> CliResult<()> {
        let mut rl =
            DefaultEditor::new().map_err(|e| crate::error::CliError::Internal(e.to_string()))?;

        println!("Entering chat mode. Type 'exit' to quit.");
        println!("Provider: {}, Model: {}", self.provider, self.model);

        loop {
            let readline = rl.readline("r[ > ");
            match readline {
                Ok(line) => {
                    if line.trim() == "exit" {
                        println!("Goodbye!");
                        break;
                    }

                    if !line.trim().is_empty() {
                        // Add to history
                        self.history.push(ChatMessage {
                            role: "user".to_string(),
                            content: line.clone(),
                        });

                        // Process message
                        println!("Processing: {}", line);
                    }
                }
                Err(_) => {
                    println!("Goodbye!");
                    break;
                }
            }
        }

        Ok(())
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
