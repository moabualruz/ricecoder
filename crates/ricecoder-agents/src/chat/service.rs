//! Chat Service Implementation
//!
//! Core service for managing AI agent conversations, coordinating between
//! LLM providers, context management, and tool execution.
//!
//! Ported from crustly's AgentService with adaptations for ricecoder's architecture.

use std::collections::HashMap;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;

use serde_json::Value;
use tracing::{debug, error, info, trace, warn};
use uuid::Uuid;

use ricecoder_providers::provider::Provider;

use super::context::ChatContext;
use super::error::{ChatError, Result};
use super::types::{
    ChatMessage, ContentBlock, Role, StopReason, ToolApprovalInfo, ToolDefinition, Usage,
};
use crate::tool_registry::{ToolInvoker, ToolRegistry};

/// Type alias for approval callback function
/// Returns true if approved, false if denied
pub type ApprovalCallback = Arc<
    dyn Fn(ToolApprovalInfo) -> Pin<Box<dyn Future<Output = Result<bool>> + Send>> + Send + Sync,
>;

/// Chat Service for managing AI conversations with tool execution
///
/// This service implements the core conversation loop:
/// 1. Receive user message
/// 2. Send to LLM provider
/// 3. If model requests tool use, execute tools
/// 4. Send tool results back to LLM
/// 5. Repeat until model finishes or max iterations reached
pub struct ChatService {
    /// LLM provider
    provider: Arc<dyn Provider>,

    /// Tool registry for executing tools
    tool_registry: Arc<ToolRegistry>,

    /// Maximum tool execution iterations
    max_tool_iterations: usize,

    /// Default system prompt
    default_system_prompt: Option<String>,

    /// Whether to auto-approve tool execution
    auto_approve_tools: bool,

    /// Callback for requesting tool approval from user
    approval_callback: Option<ApprovalCallback>,

    /// Working directory for tool execution
    working_directory: PathBuf,

    /// Default model to use
    default_model: Option<String>,
}

impl ChatService {
    /// Create a new chat service
    pub fn new(provider: Arc<dyn Provider>) -> Self {
        Self {
            provider,
            tool_registry: Arc::new(ToolRegistry::new()),
            max_tool_iterations: 10,
            default_system_prompt: None,
            auto_approve_tools: false,
            approval_callback: None,
            working_directory: std::env::current_dir().unwrap_or_default(),
            default_model: None,
        }
    }

    /// Set the default system prompt
    pub fn with_system_prompt(mut self, prompt: String) -> Self {
        self.default_system_prompt = Some(prompt);
        self
    }

    /// Set maximum tool iterations
    pub fn with_max_tool_iterations(mut self, max: usize) -> Self {
        self.max_tool_iterations = max;
        self
    }

    /// Set the tool registry
    pub fn with_tool_registry(mut self, registry: Arc<ToolRegistry>) -> Self {
        self.tool_registry = registry;
        self
    }

    /// Set whether to auto-approve tool execution
    pub fn with_auto_approve_tools(mut self, auto_approve: bool) -> Self {
        self.auto_approve_tools = auto_approve;
        self
    }

    /// Set the approval callback for interactive tool approval
    pub fn with_approval_callback(mut self, callback: Option<ApprovalCallback>) -> Self {
        self.approval_callback = callback;
        self
    }

    /// Set the working directory for tool execution
    pub fn with_working_directory(mut self, working_directory: PathBuf) -> Self {
        self.working_directory = working_directory;
        self
    }

    /// Set the default model
    pub fn with_default_model(mut self, model: String) -> Self {
        self.default_model = Some(model);
        self
    }

    /// Get the provider name
    pub fn provider_name(&self) -> &str {
        self.provider.name()
    }

    /// Get the provider ID
    pub fn provider_id(&self) -> &str {
        self.provider.id()
    }

    /// Send a message and get a response (simple, no tools)
    ///
    /// This is the basic chat completion without tool execution.
    pub async fn send_message(
        &self,
        context: &mut ChatContext,
        user_message: String,
        model: Option<String>,
    ) -> Result<ChatResponse> {
        // Add user message to context
        let user_msg = ChatMessage::user(user_message);
        context.add_message(user_msg);

        // Build request
        let model_name = model
            .or_else(|| self.default_model.clone())
            .unwrap_or_else(|| {
                self.provider
                    .models()
                    .first()
                    .map(|m| m.id.clone())
                    .unwrap_or_else(|| "default".to_string())
            });

        let mut messages = context.to_simple_messages();

        // Add system prompt if available
        if let Some(system) = &self.default_system_prompt {
            messages.insert(
                0,
                ricecoder_providers::models::Message {
                    role: "system".to_string(),
                    content: system.clone(),
                },
            );
        }

        let request = ricecoder_providers::models::ChatRequest {
            model: model_name.clone(),
            messages,
            temperature: Some(0.7),
            max_tokens: Some(4096),
            stream: false,
        };

        // Send to provider
        let response = self.provider.chat(request).await?;

        // Add assistant response to context
        let assistant_msg = ChatMessage::assistant(&response.content);
        context.add_message(assistant_msg);

        // Track usage
        let usage = Usage::new(
            response.usage.prompt_tokens,
            response.usage.completion_tokens,
        );
        context.add_usage(usage.clone());

        Ok(ChatResponse {
            content: response.content,
            model: response.model,
            usage,
            stop_reason: StopReason::EndTurn,
            tool_calls: Vec::new(),
        })
    }

    /// Send a message with automatic tool execution
    ///
    /// This method implements the full tool execution loop:
    /// 1. Send message to LLM
    /// 2. If LLM requests tool use, execute the tool
    /// 3. Send tool results back to LLM
    /// 4. Repeat until LLM finishes or max iterations reached
    pub async fn send_message_with_tools(
        &self,
        context: &mut ChatContext,
        user_message: String,
        model: Option<String>,
    ) -> Result<ChatResponse> {
        // Add user message to context
        let user_msg = ChatMessage::user(user_message);
        context.add_message(user_msg);

        let model_name = model
            .or_else(|| self.default_model.clone())
            .unwrap_or_else(|| {
                self.provider
                    .models()
                    .first()
                    .map(|m| m.id.clone())
                    .unwrap_or_else(|| "default".to_string())
            });

        // Tool execution loop
        let mut iteration = 0;
        let mut total_usage = Usage::default();
        let mut all_tool_calls: Vec<ToolCall> = Vec::new();
        let mut recent_tool_signatures: Vec<String> = Vec::new();

        while iteration < self.max_tool_iterations {
            iteration += 1;
            debug!("Tool execution iteration {}/{}", iteration, self.max_tool_iterations);

            // Build messages for provider
            let mut messages = context.to_simple_messages();

            // Add system prompt if available
            if let Some(system) = &self.default_system_prompt {
                messages.insert(
                    0,
                    ricecoder_providers::models::Message {
                        role: "system".to_string(),
                        content: system.clone(),
                    },
                );
            }

            let request = ricecoder_providers::models::ChatRequest {
                model: model_name.clone(),
                messages,
                temperature: Some(0.7),
                max_tokens: Some(4096),
                stream: false,
            };

            // Send to provider
            let response = self.provider.chat(request).await?;

            // Track usage
            total_usage.input_tokens += response.usage.prompt_tokens;
            total_usage.output_tokens += response.usage.completion_tokens;

            // Parse response for tool calls
            // For now, we look for tool calls in the response content
            // This is a simplified approach - full implementation would use
            // provider-specific tool calling APIs
            let tool_calls = self.parse_tool_calls(&response.content);

            if tool_calls.is_empty() {
                // No tool calls - we're done
                debug!("No tool calls found, completing conversation");

                // Add final assistant response
                let assistant_msg = ChatMessage::assistant(&response.content);
                context.add_message(assistant_msg);
                context.add_usage(total_usage.clone());

                return Ok(ChatResponse {
                    content: response.content,
                    model: response.model,
                    usage: total_usage,
                    stop_reason: StopReason::EndTurn,
                    tool_calls: all_tool_calls,
                });
            }

            // Check for tool loops
            let current_signature = self.compute_tool_signature(&tool_calls);
            if self.detect_tool_loop(&mut recent_tool_signatures, &current_signature) {
                warn!("Detected tool loop, breaking out");
                
                let assistant_msg = ChatMessage::assistant(&response.content);
                context.add_message(assistant_msg);
                context.add_usage(total_usage.clone());

                return Ok(ChatResponse {
                    content: response.content,
                    model: response.model,
                    usage: total_usage,
                    stop_reason: StopReason::EndTurn,
                    tool_calls: all_tool_calls,
                });
            }

            // Execute tools
            let mut tool_results: Vec<ContentBlock> = Vec::new();

            for tool_call in &tool_calls {
                info!("Executing tool '{}' (iteration {}/{})", 
                    tool_call.name, iteration, self.max_tool_iterations);

                // Check if approval is needed
                let needs_approval = !self.auto_approve_tools;

                if needs_approval {
                    if let Some(ref callback) = self.approval_callback {
                        let approval_info = ToolApprovalInfo {
                            tool_name: tool_call.name.clone(),
                            tool_description: format!("Execute tool: {}", tool_call.name),
                            tool_input: tool_call.input.clone(),
                            capabilities: vec!["execute".to_string()],
                        };

                        match callback(approval_info).await {
                            Ok(approved) => {
                                if !approved {
                                    warn!("User denied approval for tool '{}'", tool_call.name);
                                    tool_results.push(ContentBlock::tool_result(
                                        &tool_call.id,
                                        "User denied permission to execute this tool",
                                        true,
                                    ));
                                    continue;
                                }
                            }
                            Err(e) => {
                                error!("Approval callback error: {}", e);
                                tool_results.push(ContentBlock::tool_result(
                                    &tool_call.id,
                                    format!("Approval request failed: {}", e),
                                    true,
                                ));
                                continue;
                            }
                        }
                    }
                }

                // Execute the tool
                match self.tool_registry.invoke_tool(&tool_call.name, tool_call.input.clone()).await {
                    Ok(result) => {
                        let result_str = serde_json::to_string_pretty(&result)
                            .unwrap_or_else(|_| result.to_string());
                        tool_results.push(ContentBlock::tool_result(
                            &tool_call.id,
                            result_str,
                            false,
                        ));
                        all_tool_calls.push(tool_call.clone());
                    }
                    Err(e) => {
                        error!("Tool execution failed: {}", e);
                        tool_results.push(ContentBlock::tool_result(
                            &tool_call.id,
                            format!("Tool execution error: {}", e),
                            true,
                        ));
                    }
                }
            }

            // Add assistant message with tool calls to context
            let assistant_content: Vec<ContentBlock> = std::iter::once(ContentBlock::text(&response.content))
                .chain(tool_calls.iter().map(|tc| {
                    ContentBlock::tool_use(&tc.id, &tc.name, tc.input.clone())
                }))
                .collect();
            context.add_message(ChatMessage::with_content(Role::Assistant, assistant_content));

            // Add tool results to context
            context.add_message(ChatMessage::with_content(Role::User, tool_results));
        }

        Err(ChatError::MaxIterationsExceeded(self.max_tool_iterations))
    }

    /// Parse tool calls from response content
    ///
    /// This is a simplified parser that looks for JSON tool call blocks.
    /// Full implementation would use provider-specific tool calling APIs.
    fn parse_tool_calls(&self, content: &str) -> Vec<ToolCall> {
        let mut calls = Vec::new();

        // Look for tool call patterns like:
        // <tool_call>{"name": "...", "input": {...}}</tool_call>
        // or function_call blocks
        
        // Simple regex-free parsing for common patterns
        if let Some(start) = content.find("<tool_call>") {
            if let Some(end) = content.find("</tool_call>") {
                let json_str = &content[start + 11..end];
                if let Ok(parsed) = serde_json::from_str::<ToolCallJson>(json_str) {
                    calls.push(ToolCall {
                        id: Uuid::new_v4().to_string(),
                        name: parsed.name,
                        input: parsed.input.unwrap_or(Value::Object(Default::default())),
                    });
                }
            }
        }

        // Also check for Anthropic-style tool use markers
        // This is a heuristic - real implementation would use proper parsing
        if content.contains("I'll use the") || content.contains("Let me use") {
            // Try to extract tool name and construct a call
            // This would need provider-specific handling
        }

        calls
    }

    /// Compute a signature for tool calls to detect loops
    fn compute_tool_signature(&self, calls: &[ToolCall]) -> String {
        calls
            .iter()
            .map(|c| {
                // Include tool name and key arguments for signature
                let args_summary = match &c.input {
                    Value::Object(map) => {
                        map.iter()
                            .filter_map(|(k, v)| {
                                match v {
                                    Value::String(s) => Some(format!("{}:{}", k, s)),
                                    _ => None,
                                }
                            })
                            .collect::<Vec<_>>()
                            .join(",")
                    }
                    _ => String::new(),
                };
                format!("{}:{}", c.name, args_summary)
            })
            .collect::<Vec<_>>()
            .join("|")
    }

    /// Detect tool execution loops
    ///
    /// Returns true if a loop is detected (same signature appearing multiple times)
    fn detect_tool_loop(&self, recent: &mut Vec<String>, current: &String) -> bool {
        recent.push(current.clone());

        // Keep last 15 iterations
        if recent.len() > 15 {
            recent.remove(0);
        }

        // Check for repeated patterns
        let threshold = 3; // Same signature 3 times = loop
        if recent.len() >= threshold {
            let last_n = &recent[recent.len() - threshold..];
            if last_n.iter().all(|s| s == current) {
                return true;
            }
        }

        false
    }
}

/// Response from the chat service
#[derive(Debug, Clone)]
pub struct ChatResponse {
    /// Response content
    pub content: String,

    /// Model used
    pub model: String,

    /// Token usage
    pub usage: Usage,

    /// Stop reason
    pub stop_reason: StopReason,

    /// Tool calls executed
    pub tool_calls: Vec<ToolCall>,
}

/// A tool call executed during the conversation
#[derive(Debug, Clone)]
pub struct ToolCall {
    /// Unique ID for this tool call
    pub id: String,
    /// Tool name
    pub name: String,
    /// Tool input parameters
    pub input: Value,
}

/// JSON structure for parsing tool calls
#[derive(Debug, serde::Deserialize)]
struct ToolCallJson {
    name: String,
    #[serde(default)]
    input: Option<Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock provider for testing
    struct MockProvider {
        response: String,
    }

    #[async_trait::async_trait]
    impl Provider for MockProvider {
        fn id(&self) -> &str {
            "mock"
        }

        fn name(&self) -> &str {
            "Mock Provider"
        }

        fn models(&self) -> Vec<ricecoder_providers::models::ModelInfo> {
            vec![ricecoder_providers::models::ModelInfo {
                id: "mock-model".to_string(),
                name: "Mock Model".to_string(),
                provider: "mock".to_string(),
                context_window: 4096,
                capabilities: vec![],
                pricing: None,
                is_free: true,
            }]
        }

        async fn chat(
            &self,
            _request: ricecoder_providers::models::ChatRequest,
        ) -> std::result::Result<ricecoder_providers::models::ChatResponse, ricecoder_providers::error::ProviderError> {
            Ok(ricecoder_providers::models::ChatResponse {
                content: self.response.clone(),
                model: "mock-model".to_string(),
                usage: ricecoder_providers::models::TokenUsage {
                    prompt_tokens: 10,
                    completion_tokens: 20,
                    total_tokens: 30,
                },
                finish_reason: ricecoder_providers::models::FinishReason::Stop,
            })
        }

        async fn chat_stream(
            &self,
            _request: ricecoder_providers::models::ChatRequest,
        ) -> std::result::Result<ricecoder_providers::provider::ChatStream, ricecoder_providers::error::ProviderError> {
            unimplemented!()
        }

        fn count_tokens(&self, content: &str, _model: &str) -> std::result::Result<usize, ricecoder_providers::error::ProviderError> {
            Ok(content.len() / 4)
        }

        async fn health_check(&self) -> std::result::Result<bool, ricecoder_providers::error::ProviderError> {
            Ok(true)
        }
    }

    #[tokio::test]
    async fn test_chat_service_creation() {
        let provider = Arc::new(MockProvider {
            response: "Hello!".to_string(),
        });
        let service = ChatService::new(provider);
        assert_eq!(service.max_tool_iterations, 10);
    }

    #[tokio::test]
    async fn test_send_message() {
        let provider = Arc::new(MockProvider {
            response: "Hello! How can I help?".to_string(),
        });
        let service = ChatService::new(provider);

        let mut context = ChatContext::new(Uuid::new_v4(), 4096);
        let response = service
            .send_message(&mut context, "Hi there".to_string(), None)
            .await
            .unwrap();

        assert_eq!(response.content, "Hello! How can I help?");
        assert_eq!(context.messages.len(), 2); // user + assistant
    }

    #[tokio::test]
    async fn test_tool_loop_detection() {
        let provider = Arc::new(MockProvider {
            response: "Test".to_string(),
        });
        let service = ChatService::new(provider);

        let mut signatures = Vec::new();
        let sig = "read:path:test.txt".to_string();

        // First two times - no loop
        assert!(!service.detect_tool_loop(&mut signatures, &sig));
        assert!(!service.detect_tool_loop(&mut signatures, &sig));

        // Third time - loop detected
        assert!(service.detect_tool_loop(&mut signatures, &sig));
    }
}
