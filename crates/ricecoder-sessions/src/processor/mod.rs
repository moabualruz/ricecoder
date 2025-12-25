//! SessionProcessor for AI stream processing
//!
//! Processes AI model response streams and manages tool execution lifecycle.

mod stream;

pub use stream::{
    FinishReason, ProcessResult, SessionProcessor, StreamEvent, ToolState,
};

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_util::sync::CancellationToken;
    
    #[tokio::test]
    async fn test_processor_creation() {
        let cancel = CancellationToken::new();
        let processor = SessionProcessor::new(
            "session-123".to_string(),
            "message-456".to_string(),
            cancel,
        );
        
        assert_eq!(processor.session_id(), "session-123");
        assert_eq!(processor.message_id(), "message-456");
    }
    
    #[tokio::test]
    async fn test_doom_loop_detection() {
        let cancel = CancellationToken::new();
        let mut processor = SessionProcessor::new(
            "session-123".to_string(),
            "message-456".to_string(),
            cancel,
        );
        
        let input = serde_json::json!({"query": "test"});
        
        // Not a doom loop yet
        assert!(!processor.is_doom_loop("test_tool", &input));
        
        // Add same call 3 times
        for _ in 0..3 {
            processor.record_tool_call("test_tool", &input);
        }
        
        // Now it's a doom loop
        assert!(processor.is_doom_loop("test_tool", &input));
    }
    
    #[tokio::test]
    async fn test_cancellation_check() {
        let cancel = CancellationToken::new();
        let processor = SessionProcessor::new(
            "session-123".to_string(),
            "message-456".to_string(),
            cancel.clone(),
        );
        
        // Not cancelled yet
        assert!(!processor.is_cancelled());
        
        // Cancel
        cancel.cancel();
        
        // Now cancelled
        assert!(processor.is_cancelled());
    }
}
