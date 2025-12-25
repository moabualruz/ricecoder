//! Stream processing types and implementation
//!
//! Handles AI model stream events and manages tool execution state.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;

/// Maximum number of recent tool calls to track for doom loop detection
const DOOM_LOOP_WINDOW: usize = 10;

/// Threshold for identical calls to trigger doom loop detection
const DOOM_LOOP_THRESHOLD: usize = 3;

/// Stream event types from AI provider
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum StreamEvent {
    /// Start of stream
    Start,
    
    /// Text delta from assistant
    TextDelta { text: String },
    
    /// Start of reasoning section
    ReasoningStart,
    
    /// Reasoning content delta
    ReasoningDelta { text: String },
    
    /// End of reasoning section
    ReasoningEnd,
    
    /// Start of tool call input
    ToolCallStart { id: String, name: String },
    
    /// Tool call input data
    ToolCallInput { id: String, input: String },
    
    /// Tool execution result
    ToolResult { id: String, output: String },
    
    /// Tool execution error
    ToolError { id: String, error: String },
    
    /// Stream finished
    Finish { reason: FinishReason },
    
    /// Stream error
    Error { error: String },
}

/// Reason for stream completion
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FinishReason {
    /// Normal completion
    Stop,
    
    /// Context length limit reached
    Length,
    
    /// Tool call required
    ToolCall,
    
    /// Content filtered
    ContentFilter,
}

/// Tool execution state
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "kebab-case")]
pub enum ToolState {
    /// Waiting to execute
    Pending {
        input: Value,
    },
    
    /// Currently executing
    Running {
        input: Value,
        start_time: u64, // Unix timestamp in milliseconds
    },
    
    /// Successfully completed
    Completed {
        input: Value,
        output: String,
        duration_ms: u64,
    },
    
    /// Failed with error
    Error {
        input: Value,
        error: String,
        duration_ms: u64,
    },
}

/// Result of processing a stream event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ProcessResult {
    /// Continue processing
    Continue,
    
    /// Tool call needs to be executed
    ToolCallRequired {
        id: String,
        name: String,
        input: Value,
    },
    
    /// Stream finished
    Finished {
        reason: FinishReason,
    },
    
    /// Processing cancelled
    Cancelled,
    
    /// Error occurred
    Error {
        error: String,
    },
}

/// Tool call record for doom loop detection
#[derive(Debug, Clone)]
struct ToolCallRecord {
    tool: String,
    input: Value,
    timestamp: Instant,
}

/// Session processor for handling AI streams
pub struct SessionProcessor {
    session_id: String,
    message_id: String,
    cancel: CancellationToken,
    tool_states: HashMap<String, ToolState>,
    recent_calls: VecDeque<ToolCallRecord>,
    /// Token usage tracking
    input_tokens: usize,
    output_tokens: usize,
    /// Retry tracking
    retry_count: usize,
    max_retries: usize,
    /// Snapshot ID for this processing step
    snapshot_id: Option<String>,
}

impl SessionProcessor {
    /// Create a new session processor
    pub fn new(
        session_id: String,
        message_id: String,
        cancel: CancellationToken,
    ) -> Self {
        Self {
            session_id,
            message_id,
            cancel,
            tool_states: HashMap::new(),
            recent_calls: VecDeque::with_capacity(DOOM_LOOP_WINDOW),
            input_tokens: 0,
            output_tokens: 0,
            retry_count: 0,
            max_retries: 3,
            snapshot_id: None,
        }
    }
    
    /// Create a new session processor with retry configuration
    pub fn with_retries(
        session_id: String,
        message_id: String,
        cancel: CancellationToken,
        max_retries: usize,
    ) -> Self {
        Self {
            session_id,
            message_id,
            cancel,
            tool_states: HashMap::new(),
            recent_calls: VecDeque::with_capacity(DOOM_LOOP_WINDOW),
            input_tokens: 0,
            output_tokens: 0,
            retry_count: 0,
            max_retries,
            snapshot_id: None,
        }
    }
    
    /// Process a stream event
    pub async fn process_event(&mut self, event: StreamEvent) -> ProcessResult {
        // Check cancellation
        if self.cancel.is_cancelled() {
            return ProcessResult::Cancelled;
        }
        
        match event {
            StreamEvent::Start => ProcessResult::Continue,
            
            StreamEvent::TextDelta { .. } => {
                // Text deltas are handled by the caller for streaming updates
                ProcessResult::Continue
            }
            
            StreamEvent::ReasoningStart => ProcessResult::Continue,
            
            StreamEvent::ReasoningDelta { .. } => {
                // Reasoning deltas are handled by the caller for streaming updates
                ProcessResult::Continue
            }
            
            StreamEvent::ReasoningEnd => ProcessResult::Continue,
            
            StreamEvent::ToolCallStart { id, name } => {
                // Initialize tool state as pending
                self.tool_states.insert(
                    id.clone(),
                    ToolState::Pending {
                        input: Value::Null,
                    },
                );
                ProcessResult::Continue
            }
            
            StreamEvent::ToolCallInput { id, input } => {
                // Parse input JSON
                match serde_json::from_str::<Value>(&input) {
                    Ok(input_value) => {
                        // Get tool name from existing state
                        if let Some(ToolState::Pending { .. }) = self.tool_states.get(&id) {
                            // Update to running state
                            let start_time = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_millis() as u64;
                            
                            self.tool_states.insert(
                                id.clone(),
                                ToolState::Running {
                                    input: input_value.clone(),
                                    start_time,
                                },
                            );
                            
                            // Extract tool name - this is a simplification
                            // In production, track tool names separately
                            let tool_name = "unknown".to_string();
                            
                            // Check for doom loop
                            if self.is_doom_loop(&tool_name, &input_value) {
                                return ProcessResult::Error {
                                    error: format!(
                                        "Doom loop detected: {} consecutive identical calls to {}",
                                        DOOM_LOOP_THRESHOLD, tool_name
                                    ),
                                };
                            }
                            
                            // Record call
                            self.record_tool_call(&tool_name, &input_value);
                            
                            ProcessResult::ToolCallRequired {
                                id,
                                name: tool_name,
                                input: input_value,
                            }
                        } else {
                            ProcessResult::Error {
                                error: format!("Tool call {} not initialized", id),
                            }
                        }
                    }
                    Err(e) => ProcessResult::Error {
                        error: format!("Failed to parse tool input: {}", e),
                    },
                }
            }
            
            StreamEvent::ToolResult { id, output } => {
                // Update tool state to completed
                if let Some(state) = self.tool_states.get(&id) {
                    let (input, start_time) = match state {
                        ToolState::Running { input, start_time } => (input.clone(), *start_time),
                        _ => {
                            return ProcessResult::Error {
                                error: format!("Tool {} not in running state", id),
                            };
                        }
                    };
                    
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64;
                    
                    let duration_ms = now.saturating_sub(start_time);
                    
                    self.tool_states.insert(
                        id,
                        ToolState::Completed {
                            input,
                            output,
                            duration_ms,
                        },
                    );
                }
                ProcessResult::Continue
            }
            
            StreamEvent::ToolError { id, error } => {
                // Update tool state to error
                if let Some(state) = self.tool_states.get(&id) {
                    let (input, start_time) = match state {
                        ToolState::Running { input, start_time } => (input.clone(), *start_time),
                        _ => {
                            return ProcessResult::Error {
                                error: format!("Tool {} not in running state", id),
                            };
                        }
                    };
                    
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64;
                    
                    let duration_ms = now.saturating_sub(start_time);
                    
                    self.tool_states.insert(
                        id,
                        ToolState::Error {
                            input,
                            error,
                            duration_ms,
                        },
                    );
                }
                ProcessResult::Continue
            }
            
            StreamEvent::Finish { reason } => ProcessResult::Finished { reason },
            
            StreamEvent::Error { error } => ProcessResult::Error { error },
        }
    }
    
    /// Check if a tool call pattern indicates a doom loop
    pub fn is_doom_loop(&self, tool_name: &str, input: &Value) -> bool {
        let recent: Vec<_> = self
            .recent_calls
            .iter()
            .rev()
            .take(DOOM_LOOP_THRESHOLD)
            .collect();
        
        if recent.len() < DOOM_LOOP_THRESHOLD {
            return false;
        }
        
        // Check if all recent calls match the current pattern
        recent.iter().all(|record| {
            record.tool == tool_name && record.input == *input
        })
    }
    
    /// Record a tool call for doom loop detection
    pub fn record_tool_call(&mut self, tool_name: &str, input: &Value) {
        self.recent_calls.push_back(ToolCallRecord {
            tool: tool_name.to_string(),
            input: input.clone(),
            timestamp: Instant::now(),
        });
        
        // Keep only recent calls
        while self.recent_calls.len() > DOOM_LOOP_WINDOW {
            self.recent_calls.pop_front();
        }
    }
    
    /// Check if processing has been cancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancel.is_cancelled()
    }
    
    /// Get session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }
    
    /// Get message ID
    pub fn message_id(&self) -> &str {
        &self.message_id
    }
    
    /// Get tool state
    pub fn tool_state(&self, id: &str) -> Option<&ToolState> {
        self.tool_states.get(id)
    }
    
    /// Get all tool states
    pub fn tool_states(&self) -> &HashMap<String, ToolState> {
        &self.tool_states
    }
    
    // === GAP 4: Processor enhancements ===
    
    /// Record token usage for this processing step
    pub fn record_tokens(&mut self, input: usize, output: usize) {
        self.input_tokens += input;
        self.output_tokens += output;
    }
    
    /// Get total token usage for this processing step
    pub fn token_usage(&self) -> (usize, usize) {
        (self.input_tokens, self.output_tokens)
    }
    
    /// Set the snapshot ID for this processing step
    pub fn set_snapshot(&mut self, snapshot_id: String) {
        self.snapshot_id = Some(snapshot_id);
    }
    
    /// Get the snapshot ID for this processing step
    pub fn snapshot_id(&self) -> Option<&str> {
        self.snapshot_id.as_deref()
    }
    
    /// Check if retry is available
    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries
    }
    
    /// Increment retry count
    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }
    
    /// Get current retry count
    pub fn retry_count(&self) -> usize {
        self.retry_count
    }
    
    /// Get exponential backoff delay for current retry
    pub fn backoff_delay(&self) -> Duration {
        // Exponential backoff: 2^retry * 100ms
        let base_ms = 100u64;
        let multiplier = 2u64.pow(self.retry_count as u32);
        Duration::from_millis(base_ms * multiplier)
    }
    
    /// Reset processor state for retry
    pub fn reset_for_retry(&mut self) {
        self.tool_states.clear();
        self.recent_calls.clear();
        // Keep token counts for total tracking
        // Keep snapshot_id for rollback capability
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_text_delta_processing() {
        let cancel = CancellationToken::new();
        let mut processor = SessionProcessor::new(
            "session-1".to_string(),
            "message-1".to_string(),
            cancel,
        );
        
        let event = StreamEvent::TextDelta {
            text: "Hello".to_string(),
        };
        
        let result = processor.process_event(event).await;
        assert!(matches!(result, ProcessResult::Continue));
    }
    
    #[tokio::test]
    async fn test_tool_call_lifecycle() {
        let cancel = CancellationToken::new();
        let mut processor = SessionProcessor::new(
            "session-1".to_string(),
            "message-1".to_string(),
            cancel,
        );
        
        // Start tool call
        let start_event = StreamEvent::ToolCallStart {
            id: "tool-1".to_string(),
            name: "test_tool".to_string(),
        };
        processor.process_event(start_event).await;
        
        // Provide input
        let input_event = StreamEvent::ToolCallInput {
            id: "tool-1".to_string(),
            input: r#"{"query": "test"}"#.to_string(),
        };
        let result = processor.process_event(input_event).await;
        
        assert!(matches!(result, ProcessResult::ToolCallRequired { .. }));
        
        // Return result
        let result_event = StreamEvent::ToolResult {
            id: "tool-1".to_string(),
            output: "Success".to_string(),
        };
        processor.process_event(result_event).await;
        
        // Check final state
        let state = processor.tool_state("tool-1");
        assert!(matches!(state, Some(ToolState::Completed { .. })));
    }
    
    #[tokio::test]
    async fn test_finish_processing() {
        let cancel = CancellationToken::new();
        let mut processor = SessionProcessor::new(
            "session-1".to_string(),
            "message-1".to_string(),
            cancel,
        );
        
        let event = StreamEvent::Finish {
            reason: FinishReason::Stop,
        };
        
        let result = processor.process_event(event).await;
        assert!(matches!(
            result,
            ProcessResult::Finished {
                reason: FinishReason::Stop
            }
        ));
    }
}
