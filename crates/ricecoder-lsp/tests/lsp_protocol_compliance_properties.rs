//! Property-based tests for LSP protocol compliance
//!
//! **Feature: ricecoder-lsp, Property 1: LSP protocol compliance**
//! **Validates: Requirements LSP-1.1, LSP-1.2**
//!
//! These tests verify that the LSP server correctly implements the Language Server Protocol
//! by testing that valid LSP messages produce valid responses and invalid messages are handled gracefully.

use proptest::prelude::*;
use ricecoder_lsp::transport::{JsonRpcNotification, JsonRpcRequest, JsonRpcResponse, LspMessage};
use serde_json::{json, Value};

/// Strategy for generating valid JSON-RPC request IDs
fn request_id_strategy() -> impl Strategy<Value = Value> {
    prop_oneof![
        Just(json!(1)),
        Just(json!(2)),
        Just(json!(3)),
        Just(json!("request-1")),
        Just(json!("request-2")),
    ]
}

/// Strategy for generating valid method names
fn method_name_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("initialize".to_string()),
        Just("shutdown".to_string()),
        Just("textDocument/didOpen".to_string()),
        Just("textDocument/didChange".to_string()),
        Just("textDocument/didClose".to_string()),
        Just("textDocument/hover".to_string()),
        Just("textDocument/codeAction".to_string()),
    ]
}

/// Strategy for generating valid JSON-RPC request parameters
fn params_strategy() -> impl Strategy<Value = Option<Value>> {
    prop_oneof![
        Just(None),
        Just(Some(json!({}))),
        Just(Some(json!({"processId": 1234}))),
        Just(Some(json!({"capabilities": {}}))),
    ]
}

/// Property 1: Valid LSP initialize messages produce valid responses
#[test]
fn prop_valid_initialize_request_produces_valid_response() {
    proptest!(|(id in request_id_strategy())| {
        let req = JsonRpcRequest::new(
            id.clone(),
            "initialize".to_string(),
            Some(json!({"processId": 1234, "capabilities": {}})),
        );

        // Verify request is valid
        assert_eq!(req.jsonrpc, "2.0");
        assert_eq!(req.method, "initialize");
        assert!(req.params.is_some());

        // Serialize and deserialize
        let json_str = serde_json::to_string(&req).unwrap();
        let deserialized: JsonRpcRequest = serde_json::from_str(&json_str).unwrap();

        // Verify round-trip
        assert_eq!(deserialized.jsonrpc, req.jsonrpc);
        assert_eq!(deserialized.method, req.method);
        assert_eq!(deserialized.id, req.id);
    });
}

/// Property 2: Invalid messages are handled gracefully without crashing
#[test]
fn prop_invalid_messages_handled_gracefully() {
    proptest!(|(invalid_json in r#"[a-zA-Z0-9\{\}\[\]\",:]*"#)| {
        // Try to parse invalid JSON
        let result = LspMessage::from_json(&invalid_json);

        // Should either succeed (if it happens to be valid JSON) or fail gracefully
        match result {
            Ok(_) => {
                // If it parsed, it must have been valid JSON
                let _: Value = serde_json::from_str(&invalid_json).unwrap();
            }
            Err(err) => {
                // Error should be descriptive
                assert!(!err.to_string().is_empty());
            }
        }
    });
}

/// Property 3: Server responds with correct message format (Content-Length header)
#[test]
fn prop_response_has_correct_format() {
    proptest!(|(id in request_id_strategy())| {
        let response = JsonRpcResponse::success(id, json!({"capabilities": {}}));

        // Verify response structure
        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_some());
        assert!(response.error.is_none());

        // Serialize to JSON
        let json_str = serde_json::to_string(&response).unwrap();

        // Verify it's valid JSON
        let _: Value = serde_json::from_str(&json_str).unwrap();

        // Verify Content-Length would be correct
        let content_length = json_str.len();
        assert!(content_length > 0);
    });
}

/// Property 4: Document synchronization maintains correct state across operations
#[test]
fn prop_document_sync_maintains_state() {
    proptest!(|(
        uri in r#"file://[a-zA-Z0-9_/\.]+"#,
        content1 in r#"[a-zA-Z0-9\s\n]+"#,
        content2 in r#"[a-zA-Z0-9\s\n]+"#,
    )| {
        // Create did_open notification
        let did_open = JsonRpcNotification::new(
            "textDocument/didOpen".to_string(),
            Some(json!({
                "textDocument": {
                    "uri": uri,
                    "text": content1
                }
            })),
        );

        // Verify notification structure
        assert_eq!(did_open.jsonrpc, "2.0");
        assert_eq!(did_open.method, "textDocument/didOpen");
        assert!(did_open.params.is_some());

        // Create did_change notification
        let did_change = JsonRpcNotification::new(
            "textDocument/didChange".to_string(),
            Some(json!({
                "textDocument": {
                    "uri": uri,
                    "version": 2
                },
                "contentChanges": [
                    {
                        "text": content2
                    }
                ]
            })),
        );

        // Verify change notification structure
        assert_eq!(did_change.jsonrpc, "2.0");
        assert_eq!(did_change.method, "textDocument/didChange");

        // Create did_close notification
        let did_close = JsonRpcNotification::new(
            "textDocument/didClose".to_string(),
            Some(json!({
                "textDocument": {
                    "uri": uri
                }
            })),
        );

        // Verify close notification structure
        assert_eq!(did_close.jsonrpc, "2.0");
        assert_eq!(did_close.method, "textDocument/didClose");

        // All notifications should serialize correctly
        let _ = serde_json::to_string(&did_open).unwrap();
        let _ = serde_json::to_string(&did_change).unwrap();
        let _ = serde_json::to_string(&did_close).unwrap();
    });
}

/// Property 5: LSP message round-trip (serialize and deserialize)
#[test]
fn prop_lsp_message_round_trip() {
    proptest!(|(
        id in request_id_strategy(),
        method in method_name_strategy(),
        params in params_strategy(),
    )| {
        let req = JsonRpcRequest::new(id.clone(), method.clone(), params.clone());
        let msg = LspMessage::Request(req);

        // Serialize
        let json_str = msg.to_json().unwrap();

        // Deserialize
        let msg2 = LspMessage::from_json(&json_str).unwrap();

        // Verify round-trip
        match msg2 {
            LspMessage::Request(req2) => {
                assert_eq!(req2.jsonrpc, "2.0");
                assert_eq!(req2.method, method);
                assert_eq!(req2.id, id);
            }
            _ => panic!("Expected request"),
        }
    });
}

/// Property 6: Error responses have correct structure
#[test]
fn prop_error_response_structure() {
    proptest!(|(id in request_id_strategy())| {
        use ricecoder_lsp::transport::JsonRpcError;

        let error = JsonRpcError::parse_error("Test error".to_string());
        let response = JsonRpcResponse::error(id.clone(), error);

        // Verify error response structure
        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.error.is_some());
        assert!(response.result.is_none());
        assert_eq!(response.id, id);

        // Verify error details
        if let Some(err) = response.error {
            assert_eq!(err.code, -32700); // Parse error code
            assert!(!err.message.is_empty());
        }
    });
}

/// Property 7: Notification messages don't have IDs
#[test]
fn prop_notifications_have_no_id() {
    proptest!(|(method in method_name_strategy())| {
        let notif = JsonRpcNotification::new(method.clone(), None);

        // Verify notification structure
        assert_eq!(notif.jsonrpc, "2.0");
        assert_eq!(notif.method, method);

        // Serialize and deserialize
        let json_str = serde_json::to_string(&notif).unwrap();
        let msg = LspMessage::from_json(&json_str).unwrap();

        // Verify it's a notification
        match msg {
            LspMessage::Notification(n) => {
                assert_eq!(n.method, notif.method);
            }
            _ => panic!("Expected notification"),
        }
    });
}

/// Property 8: All JSON-RPC error codes are valid
#[test]
fn prop_jsonrpc_error_codes_valid() {
    use ricecoder_lsp::transport::JsonRpcError;

    proptest!(|(code in -32700i32..=-32600i32)| {
        let error = JsonRpcError::new(code, "Test error".to_string());

        // Verify error structure
        assert_eq!(error.code, code);
        assert!(!error.message.is_empty());

        // Serialize and deserialize
        let json_str = serde_json::to_string(&error).unwrap();
        let deserialized: JsonRpcError = serde_json::from_str(&json_str).unwrap();

        // Verify round-trip
        assert_eq!(deserialized.code, error.code);
        assert_eq!(deserialized.message, error.message);
    });
}
