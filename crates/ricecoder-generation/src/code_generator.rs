//! AI-based code generation with streaming support
//!
//! Provides CodeGenerator for calling AI providers with built prompts,
//! handling streaming responses, parsing generated code, and extracting
//! multiple files from single responses.

use std::time::Duration;

use ricecoder_providers::{
    models::{ChatRequest, Message},
    provider::Provider,
};
use tokio::time::sleep;

use crate::{error::GenerationError, models::GeneratedFile, prompt_builder::GeneratedPrompt};

/// Configuration for code generation
#[derive(Debug, Clone)]
pub struct CodeGeneratorConfig {
    /// Maximum number of retries on transient failures
    pub max_retries: usize,
    /// Initial backoff duration for retries
    pub initial_backoff: Duration,
    /// Maximum backoff duration for retries
    pub max_backoff: Duration,
    /// Backoff multiplier for exponential backoff
    pub backoff_multiplier: f64,
}

impl Default for CodeGeneratorConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(10),
            backoff_multiplier: 2.0,
        }
    }
}

/// Generates code using AI providers
#[derive(Debug, Clone)]
pub struct CodeGenerator {
    /// Configuration for code generation
    config: CodeGeneratorConfig,
}

impl CodeGenerator {
    /// Creates a new CodeGenerator with default configuration
    pub fn new() -> Self {
        Self {
            config: CodeGeneratorConfig::default(),
        }
    }

    /// Creates a new CodeGenerator with custom configuration
    pub fn with_config(config: CodeGeneratorConfig) -> Self {
        Self { config }
    }

    /// Generates code from a prompt using the provided provider
    ///
    /// # Arguments
    /// * `provider` - The AI provider to use for generation
    /// * `prompt` - The generated prompt containing system and user messages
    /// * `model` - The model to use for generation
    /// * `temperature` - Temperature for sampling (0.0 to 2.0)
    /// * `max_tokens` - Maximum tokens to generate
    ///
    /// # Returns
    /// A vector of generated files
    ///
    /// # Errors
    /// Returns `GenerationError` if generation fails after all retries
    pub async fn generate(
        &self,
        provider: &dyn Provider,
        prompt: &GeneratedPrompt,
        model: &str,
        temperature: f32,
        max_tokens: usize,
    ) -> Result<Vec<GeneratedFile>, GenerationError> {
        let mut backoff = self.config.initial_backoff;
        let mut last_error = None;

        for attempt in 0..=self.config.max_retries {
            match self
                .generate_internal(provider, prompt, model, temperature, max_tokens)
                .await
            {
                Ok(files) => return Ok(files),
                Err(e) => {
                    last_error = Some(e);

                    // Don't retry on the last attempt
                    if attempt < self.config.max_retries {
                        sleep(backoff).await;
                        backoff = Duration::from_secs_f64(
                            (backoff.as_secs_f64() * self.config.backoff_multiplier)
                                .min(self.config.max_backoff.as_secs_f64()),
                        );
                    }
                }
            }
        }

        Err(last_error
            .unwrap_or_else(|| GenerationError::GenerationFailed("Unknown error".to_string())))
    }

    /// Internal implementation of code generation
    async fn generate_internal(
        &self,
        provider: &dyn Provider,
        prompt: &GeneratedPrompt,
        model: &str,
        temperature: f32,
        max_tokens: usize,
    ) -> Result<Vec<GeneratedFile>, GenerationError> {
        // Build the chat request
        let mut messages = vec![Message {
            role: "system".to_string(),
            content: prompt.system_prompt.clone(),
        }];

        messages.push(Message {
            role: "user".to_string(),
            content: prompt.user_prompt.clone(),
        });

        let request = ChatRequest {
            model: model.to_string(),
            messages,
            temperature: Some(temperature),
            max_tokens: Some(max_tokens),
            stream: false,
        };

        // Call the provider
        let response = provider
            .chat(request)
            .await
            .map_err(|e| GenerationError::GenerationFailed(e.to_string()))?;

        // Parse the response into files
        self.parse_generated_code(&response.content)
    }

    /// Parses generated code into individual files
    ///
    /// Supports multiple file formats:
    /// - Markdown code blocks with file paths: ```rust\n// file: src/main.rs\n...```
    /// - JSON format with file list
    /// - Plain code (single file)
    fn parse_generated_code(&self, content: &str) -> Result<Vec<GeneratedFile>, GenerationError> {
        let mut files = Vec::new();

        // Try to parse as markdown code blocks first
        if let Ok(parsed_files) = self.parse_markdown_blocks(content) {
            if !parsed_files.is_empty() {
                return Ok(parsed_files);
            }
        }

        // Try to parse as JSON
        if let Ok(parsed_files) = self.parse_json_files(content) {
            if !parsed_files.is_empty() {
                return Ok(parsed_files);
            }
        }

        // Fall back to treating entire content as a single file
        files.push(GeneratedFile {
            path: "generated.rs".to_string(),
            content: content.to_string(),
            language: "rust".to_string(),
        });

        Ok(files)
    }

    /// Parses markdown code blocks with file paths
    fn parse_markdown_blocks(&self, content: &str) -> Result<Vec<GeneratedFile>, GenerationError> {
        let mut files = Vec::new();
        let mut current_file: Option<GeneratedFile> = None;
        let mut in_code_block = false;
        let mut code_buffer = String::new();

        for line in content.lines() {
            if line.starts_with("```") {
                if in_code_block {
                    // End of code block
                    if let Some(mut file) = current_file.take() {
                        file.content = code_buffer.trim().to_string();
                        files.push(file);
                    }
                    code_buffer.clear();
                    in_code_block = false;
                } else {
                    // Start of code block
                    let header = line.trim_start_matches("```").trim();

                    // Extract language and optional file path
                    let parts: Vec<&str> = header.split_whitespace().collect();
                    if !parts.is_empty() {
                        let language = parts[0].to_string();

                        // Check for file path in comment
                        let file_path = if parts.len() > 1 && parts[1] == "file:" {
                            parts.get(2).map(|s| s.to_string())
                        } else {
                            None
                        };

                        current_file = Some(GeneratedFile {
                            path: file_path.unwrap_or_else(|| format!("generated.{}", language)),
                            content: String::new(),
                            language,
                        });
                    }

                    in_code_block = true;
                }
            } else if in_code_block {
                code_buffer.push_str(line);
                code_buffer.push('\n');
            }
        }

        Ok(files)
    }

    /// Parses JSON format with file list
    fn parse_json_files(&self, content: &str) -> Result<Vec<GeneratedFile>, GenerationError> {
        // Try to find JSON structure in the content
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(content) {
            if let Some(files_array) = json.get("files").and_then(|v| v.as_array()) {
                let mut files = Vec::new();

                for file_obj in files_array {
                    if let (Some(path), Some(file_content), Some(language)) = (
                        file_obj.get("path").and_then(|v| v.as_str()),
                        file_obj.get("content").and_then(|v| v.as_str()),
                        file_obj.get("language").and_then(|v| v.as_str()),
                    ) {
                        files.push(GeneratedFile {
                            path: path.to_string(),
                            content: file_content.to_string(),
                            language: language.to_string(),
                        });
                    }
                }

                if !files.is_empty() {
                    return Ok(files);
                }
            }
        }

        Ok(Vec::new())
    }

    /// Generates code with streaming support
    ///
    /// # Arguments
    /// * `provider` - The AI provider to use for generation
    /// * `prompt` - The generated prompt containing system and user messages
    /// * `model` - The model to use for generation
    /// * `temperature` - Temperature for sampling (0.0 to 2.0)
    /// * `max_tokens` - Maximum tokens to generate
    ///
    /// # Returns
    /// A stream of generated content chunks
    ///
    /// # Errors
    /// Returns `GenerationError` if streaming fails
    pub async fn generate_streaming(
        &self,
        provider: &dyn Provider,
        prompt: &GeneratedPrompt,
        model: &str,
        temperature: f32,
        max_tokens: usize,
    ) -> Result<String, GenerationError> {
        // Build the chat request
        let mut messages = vec![Message {
            role: "system".to_string(),
            content: prompt.system_prompt.clone(),
        }];

        messages.push(Message {
            role: "user".to_string(),
            content: prompt.user_prompt.clone(),
        });

        let request = ChatRequest {
            model: model.to_string(),
            messages,
            temperature: Some(temperature),
            max_tokens: Some(max_tokens),
            stream: true,
        };

        // Call the provider with streaming
        let mut stream = provider
            .chat_stream(request)
            .await
            .map_err(|e| GenerationError::GenerationFailed(e.to_string()))?;

        let mut full_content = String::new();

        // Collect all streamed responses
        use futures::StreamExt;
        while let Some(result) = stream.next().await {
            match result {
                Ok(response) => {
                    full_content.push_str(&response.content);
                }
                Err(e) => {
                    return Err(GenerationError::GenerationFailed(e.to_string()));
                }
            }
        }

        Ok(full_content)
    }
}

impl Default for CodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_markdown_blocks_single_file() {
        let generator = CodeGenerator::new();
        let content = r#"```rust
pub fn hello() {
    println!("Hello, world!");
}
```"#;

        let files = generator.parse_markdown_blocks(content).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].language, "rust");
        assert!(files[0].content.contains("hello"));
    }

    #[test]
    fn test_parse_markdown_blocks_multiple_files() {
        let generator = CodeGenerator::new();
        let content = r#"```rust file: src/main.rs
pub fn main() {}
```

```typescript file: src/index.ts
export function main() {}
```"#;

        let files = generator.parse_markdown_blocks(content).unwrap();
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].path, "src/main.rs");
        assert_eq!(files[1].path, "src/index.ts");
    }

    #[test]
    fn test_parse_json_files() {
        let generator = CodeGenerator::new();
        let content = r#"{
  "files": [
    {
      "path": "src/main.rs",
      "language": "rust",
      "content": "pub fn main() {}"
    }
  ]
}"#;

        let files = generator.parse_json_files(content).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "src/main.rs");
    }

    #[test]
    fn test_parse_generated_code_fallback() {
        let generator = CodeGenerator::new();
        let content = "pub fn hello() {}";

        let files = generator.parse_generated_code(content).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "generated.rs");
    }
}
