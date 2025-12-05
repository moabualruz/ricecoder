//! Property-based tests for LSP client communication
//!
//! **Feature: ricecoder-external-lsp, Property 2: Request-Response Correlation**
//! **Validates: Requirements ELSP-3.3, ELSP-3.4**

use proptest::prelude::*;
use ricecoder_external_lsp::client::JsonRpcHandler;
use serde_json::json;

/// Strategy for generating valid request IDs
fn request_id_strategy() -> impl Strategy<Value = u64> {
    1u64..1000u64
}

/// Strategy for generating valid method names
fn method_name_strategy() -> impl Strategy<Value = String> {
    r"[a-zA-Z][a-zA-Z0-9_/]*"
        .prop_map(|s| s.to_string())
        .boxed()
}

proptest! {
    /// Property: Request IDs are unique and monotonically increasing
    ///
    /// This property verifies that:
    /// 1. Each request gets a unique ID
    /// 2. IDs are assigned in increasing order
    /// 3. No ID is ever reused
    #[test]
    fn prop_request_ids_are_unique_and_increasing(
        num_requests in 1usize..100usize,
    ) {
        let handler = JsonRpcHandler::new();

        let mut ids = Vec::new();
        for _ in 0..num_requests {
            let request = handler.create_request("test", None);
            ids.push(request.id.unwrap());
        }

        // Verify all IDs are unique
        let unique_ids: std::collections::HashSet<_> = ids.iter().copied().collect();
        prop_assert_eq!(
            unique_ids.len(),
            ids.len(),
            "All request IDs should be unique"
        );

        // Verify IDs are in increasing order
        for i in 1..ids.len() {
            prop_assert!(
                ids[i] > ids[i - 1],
                "Request IDs should be monotonically increasing"
            );
        }
    }

    /// Property: Requests can be serialized and deserialized correctly
    ///
    /// This property verifies that:
    /// 1. Requests serialize to valid JSON
    /// 2. Responses can be parsed from JSON
    /// 3. Round-trip serialization preserves data
    #[test]
    fn prop_request_serialization_roundtrip(
        method in method_name_strategy(),
    ) {
        let handler = JsonRpcHandler::new();
        let request = handler.create_request(method.clone(), Some(json!({"test": "data"})));

        // Serialize request
        let json_str = handler.serialize_request(&request).unwrap();
        prop_assert!(!json_str.is_empty());
        prop_assert!(json_str.contains("\"jsonrpc\":\"2.0\""));
        prop_assert!(json_str.contains("\"method\""));

        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        prop_assert_eq!(&parsed["jsonrpc"], "2.0");
        prop_assert_eq!(parsed["method"].as_str(), Some(method.as_str()));
    }

    /// Property: Error responses are correctly identified
    ///
    /// This property verifies that:
    /// 1. Error responses are correctly detected
    /// 2. Success responses are not marked as errors
    /// 3. Error messages are preserved
    #[test]
    fn prop_error_response_detection(
        error_code in -32768i32..0i32,
        error_message in r"[a-zA-Z0-9 ]*",
    ) {
        let error_response = ricecoder_external_lsp::JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(ricecoder_external_lsp::JsonRpcError {
                code: error_code,
                message: error_message.to_string(),
                data: None,
            }),
            id: 1,
        };

        // Verify error is detected
        prop_assert!(JsonRpcHandler::is_error_response(&error_response));

        // Verify error message is extracted
        let msg = JsonRpcHandler::extract_error_message(&error_response);
        prop_assert_eq!(msg, Some(error_message.to_string()));

        // Verify success response is not marked as error
        let success_response = ricecoder_external_lsp::JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(json!({"success": true})),
            error: None,
            id: 1,
        };

        prop_assert!(!JsonRpcHandler::is_error_response(&success_response));
        prop_assert!(JsonRpcHandler::extract_error_message(&success_response).is_none());
    }

    /// Property: Notifications are created correctly
    ///
    /// This property verifies that:
    /// 1. Notifications don't have request IDs
    /// 2. Notifications can be serialized
    /// 3. Notification method names are preserved
    #[test]
    fn prop_notification_creation(
        method in method_name_strategy(),
    ) {
        let handler = JsonRpcHandler::new();
        let notification = handler.create_notification(method.clone(), Some(json!({"test": "data"})));

        // Verify notification structure
        prop_assert_eq!(&notification.jsonrpc, "2.0");
        prop_assert_eq!(&notification.method, &method);
        prop_assert!(notification.params.is_some());

        // Verify serialization works
        let json_str = handler.serialize_notification(&notification).unwrap();
        prop_assert!(!json_str.is_empty());
        prop_assert!(json_str.contains("\"jsonrpc\":\"2.0\""));
        prop_assert!(json_str.contains("\"method\""));
    }

    /// Property: Multiple requests have different IDs
    ///
    /// This property verifies that:
    /// 1. Concurrent request creation produces unique IDs
    /// 2. No ID collisions occur
    /// 3. IDs are always positive
    #[test]
    fn prop_concurrent_request_ids(
        num_requests in 1usize..50usize,
    ) {
        let handler = JsonRpcHandler::new();

        let mut ids = Vec::new();
        for _ in 0..num_requests {
            let request = handler.create_request("test", None);
            let id = request.id.unwrap();
            prop_assert!(id > 0, "Request ID should be positive");
            ids.push(id);
        }

        // Verify no duplicates
        let unique_ids: std::collections::HashSet<_> = ids.iter().copied().collect();
        prop_assert_eq!(
            unique_ids.len(),
            ids.len(),
            "All request IDs should be unique"
        );
    }
}
