//! Integration tests for LSP server lifecycle (startup, shutdown, resource cleanup)
//!
//! These tests verify that the LSP server properly initializes, handles requests,
//! and shuts down gracefully without resource leaks.

use ricecoder_lsp::{types::ServerState, LspServer};
use serde_json::json;

#[tokio::test]
async fn test_server_initialization() {
    // Create a new server
    let server = LspServer::new();

    // Verify initial state
    assert_eq!(server.state(), ServerState::Initializing);

    // Verify capabilities are set
    let caps = server.capabilities();
    assert!(caps.hover_provider);
    assert!(caps.code_action_provider);
    assert!(caps.diagnostic_provider);
}

#[tokio::test]
async fn test_server_state_transitions() {
    let mut server = LspServer::new();

    // Initial state should be Initializing
    assert_eq!(server.state(), ServerState::Initializing);

    // Simulate initialization
    let init_result = server
        .handle_initialize(json!({
            "capabilities": {}
        }))
        .await;
    assert!(init_result.is_ok());

    // After initialization, state should still be Initializing until initialized notification
    assert_eq!(server.state(), ServerState::Initializing);

    // Send initialized notification
    let init_notif_result = server.handle_initialized().await;
    assert!(init_notif_result.is_ok());

    // After initialized notification, state should be Initialized
    assert_eq!(server.state(), ServerState::Initialized);

    // Send shutdown request
    let shutdown_result = server.handle_shutdown().await;
    assert!(shutdown_result.is_ok());

    // After shutdown, state should be ShuttingDown
    assert_eq!(server.state(), ServerState::ShuttingDown);

    // Send exit notification
    let exit_result = server.handle_exit().await;
    assert!(exit_result.is_ok());

    // After exit, state should be ShutDown
    assert_eq!(server.state(), ServerState::ShutDown);
}

#[tokio::test]
async fn test_document_lifecycle() {
    let mut server = LspServer::new();
    server.set_state(ServerState::Initialized);

    let uri = "file://test.rs";
    let content = "fn main() {}";

    // Document should not exist initially
    assert_eq!(server.get_document(uri), None);

    // Open document
    let open_result = server
        .handle_did_open(json!({
            "textDocument": {
                "uri": uri,
                "text": content
            }
        }))
        .await;
    assert!(open_result.is_ok());

    // Document should now exist
    assert_eq!(server.get_document(uri), Some(content));

    // Update document
    let new_content = "fn main() { println!(\"hello\"); }";
    let change_result = server
        .handle_did_change(json!({
            "textDocument": { "uri": uri },
            "contentChanges": [{ "text": new_content }]
        }))
        .await;
    assert!(change_result.is_ok());

    // Document should be updated
    assert_eq!(server.get_document(uri), Some(new_content));

    // Close document
    let close_result = server
        .handle_did_close(json!({
            "textDocument": { "uri": uri }
        }))
        .await;
    assert!(close_result.is_ok());

    // Document should no longer exist
    assert_eq!(server.get_document(uri), None);
}

#[tokio::test]
async fn test_multiple_documents() {
    let mut server = LspServer::new();
    server.set_state(ServerState::Initialized);

    let uri1 = "file://test1.rs";
    let uri2 = "file://test2.rs";
    let content1 = "fn main() {}";
    let content2 = "fn helper() {}";

    // Open first document
    let open1_result = server
        .handle_did_open(json!({
            "textDocument": {
                "uri": uri1,
                "text": content1
            }
        }))
        .await;
    assert!(open1_result.is_ok());

    // Open second document
    let open2_result = server
        .handle_did_open(json!({
            "textDocument": {
                "uri": uri2,
                "text": content2
            }
        }))
        .await;
    assert!(open2_result.is_ok());

    // Both documents should exist
    assert_eq!(server.get_document(uri1), Some(content1));
    assert_eq!(server.get_document(uri2), Some(content2));

    // Close first document
    let close1_result = server
        .handle_did_close(json!({
            "textDocument": { "uri": uri1 }
        }))
        .await;
    assert!(close1_result.is_ok());

    // First document should be gone, second should remain
    assert_eq!(server.get_document(uri1), None);
    assert_eq!(server.get_document(uri2), Some(content2));

    // Close second document
    let close2_result = server
        .handle_did_close(json!({
            "textDocument": { "uri": uri2 }
        }))
        .await;
    assert!(close2_result.is_ok());

    // Both documents should be gone
    assert_eq!(server.get_document(uri1), None);
    assert_eq!(server.get_document(uri2), None);
}

#[tokio::test]
async fn test_error_handling_invalid_state() {
    let server = LspServer::new();

    // Server is in Initializing state, so requests should fail
    let hover_result = server
        .handle_hover(json!({
            "textDocument": { "uri": "file://test.rs" },
            "position": { "line": 0, "character": 0 }
        }))
        .await;
    assert!(hover_result.is_err());

    let diagnostics_result = server
        .handle_diagnostics(json!({
            "textDocument": { "uri": "file://test.rs" }
        }))
        .await;
    assert!(diagnostics_result.is_err());
}

#[tokio::test]
async fn test_error_handling_missing_parameters() {
    let mut server = LspServer::new();
    server.set_state(ServerState::Initialized);

    // Missing textDocument parameter
    let hover_result = server.handle_hover(json!({})).await;
    assert!(hover_result.is_err());

    // Missing position parameter
    let hover_result = server
        .handle_hover(json!({
            "textDocument": { "uri": "file://test.rs" }
        }))
        .await;
    assert!(hover_result.is_err());

    // Missing uri parameter
    let hover_result = server
        .handle_hover(json!({
            "textDocument": {},
            "position": { "line": 0, "character": 0 }
        }))
        .await;
    assert!(hover_result.is_err());
}

#[tokio::test]
async fn test_error_handling_document_not_found() {
    let mut server = LspServer::new();
    server.set_state(ServerState::Initialized);

    // Try to get hover for non-existent document
    let hover_result = server
        .handle_hover(json!({
            "textDocument": { "uri": "file://nonexistent.rs" },
            "position": { "line": 0, "character": 0 }
        }))
        .await;
    assert!(hover_result.is_err());

    // Try to get diagnostics for non-existent document
    let diagnostics_result = server
        .handle_diagnostics(json!({
            "textDocument": { "uri": "file://nonexistent.rs" }
        }))
        .await;
    assert!(diagnostics_result.is_err());
}

#[tokio::test]
async fn test_resource_cleanup_on_shutdown() {
    let mut server = LspServer::new();
    server.set_state(ServerState::Initialized);

    // Open multiple documents
    for i in 0..5 {
        let uri = format!("file://test{}.rs", i);
        let content = format!("fn main() {{ // file {} }}", i);

        let open_result = server
            .handle_did_open(json!({
                "textDocument": {
                    "uri": uri,
                    "text": content
                }
            }))
            .await;
        assert!(open_result.is_ok());
    }

    // Verify all documents are open
    for i in 0..5 {
        let uri = format!("file://test{}.rs", i);
        assert!(server.get_document(&uri).is_some());
    }

    // Shutdown should succeed
    let shutdown_result = server.handle_shutdown().await;
    assert!(shutdown_result.is_ok());
    assert_eq!(server.state(), ServerState::ShuttingDown);

    // Exit should succeed
    let exit_result = server.handle_exit().await;
    assert!(exit_result.is_ok());
    assert_eq!(server.state(), ServerState::ShutDown);
}

#[tokio::test]
async fn test_logging_lifecycle_events() {
    // This test verifies that logging is properly initialized
    // The actual log output is captured by the test framework

    let mut server = LspServer::new();

    // Initialize
    let init_result = server
        .handle_initialize(json!({
            "capabilities": {}
        }))
        .await;
    assert!(init_result.is_ok());

    // Initialized
    let init_notif_result = server.handle_initialized().await;
    assert!(init_notif_result.is_ok());

    // Shutdown
    let shutdown_result = server.handle_shutdown().await;
    assert!(shutdown_result.is_ok());

    // Exit
    let exit_result = server.handle_exit().await;
    assert!(exit_result.is_ok());
}

#[tokio::test]
async fn test_server_capabilities_response() {
    let mut server = LspServer::new();

    // Initialize and get capabilities
    let init_result = server
        .handle_initialize(json!({
            "capabilities": {}
        }))
        .await;
    assert!(init_result.is_ok());

    let response = init_result.unwrap();

    // Verify response structure
    assert!(response.get("capabilities").is_some());
    assert!(response.get("serverInfo").is_some());

    // Verify server info
    let server_info = response.get("serverInfo").unwrap();
    assert_eq!(
        server_info.get("name").and_then(|v| v.as_str()),
        Some("ricecoder-lsp")
    );
    assert_eq!(
        server_info.get("version").and_then(|v| v.as_str()),
        Some("0.1.0")
    );

    // Verify capabilities
    let capabilities = response.get("capabilities").unwrap();
    assert!(capabilities.get("textDocumentSync").is_some());
    assert!(capabilities.get("hoverProvider").is_some());
    assert!(capabilities.get("codeActionProvider").is_some());
    assert!(capabilities.get("diagnosticProvider").is_some());
}

#[tokio::test]
async fn test_client_capabilities_tracking() {
    let mut server = LspServer::new();

    // Initially no client capabilities
    assert!(server.client_capabilities().is_none());

    // Initialize with client capabilities
    let client_caps = json!({
        "textDocument": {
            "hover": {
                "contentFormat": ["markdown", "plaintext"]
            }
        }
    });

    let init_result = server
        .handle_initialize(json!({
            "capabilities": client_caps.clone()
        }))
        .await;
    assert!(init_result.is_ok());

    // Client capabilities should now be tracked
    assert!(server.client_capabilities().is_some());
    let tracked_caps = server.client_capabilities().unwrap();
    assert_eq!(tracked_caps.raw, client_caps);
}

#[tokio::test]
async fn test_invalid_shutdown_sequence() {
    let mut server = LspServer::new();

    // Try to shutdown without initializing
    let shutdown_result = server.handle_shutdown().await;
    assert!(shutdown_result.is_err());

    // Initialize
    let init_result = server
        .handle_initialize(json!({
            "capabilities": {}
        }))
        .await;
    assert!(init_result.is_ok());

    // Try to shutdown before initialized notification
    let shutdown_result = server.handle_shutdown().await;
    assert!(shutdown_result.is_err());

    // Send initialized notification
    let init_notif_result = server.handle_initialized().await;
    assert!(init_notif_result.is_ok());

    // Now shutdown should succeed
    let shutdown_result = server.handle_shutdown().await;
    assert!(shutdown_result.is_ok());

    // Try to shutdown again
    let shutdown_result = server.handle_shutdown().await;
    assert!(shutdown_result.is_err());
}

#[tokio::test]
async fn test_invalid_exit_sequence() {
    let mut server = LspServer::new();

    // Try to exit without shutdown
    let exit_result = server.handle_exit().await;
    assert!(exit_result.is_err());

    // Initialize and shutdown
    let init_result = server
        .handle_initialize(json!({
            "capabilities": {}
        }))
        .await;
    assert!(init_result.is_ok());

    let init_notif_result = server.handle_initialized().await;
    assert!(init_notif_result.is_ok());

    let shutdown_result = server.handle_shutdown().await;
    assert!(shutdown_result.is_ok());

    // Now exit should succeed
    let exit_result = server.handle_exit().await;
    assert!(exit_result.is_ok());

    // Try to exit again
    let exit_result = server.handle_exit().await;
    assert!(exit_result.is_err());
}
