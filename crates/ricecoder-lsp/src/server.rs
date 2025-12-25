//! LSP Server implementation
//!
//! This module implements the core LSP server that handles initialization,
//! shutdown, and request routing.

use std::{collections::HashMap, sync::Arc};

use ricecoder_completion::CompletionEngine;
use serde_json::{json, Value};
use tracing::{debug, error, info, warn};

use crate::{
    code_actions::{CodeActionsEngine, DefaultCodeActionsEngine},
    completion::CompletionHandler,
    config::CompletionConfig,
    diagnostics::{DefaultDiagnosticsEngine, DiagnosticsEngine},
    hover::HoverProvider,
    refactoring::RefactoringHandler,
    transport::{AsyncStdioTransport, JsonRpcError, JsonRpcResponse, LspMessage},
    types::{Language, LspError, LspResult, Position, ServerState},
};

/// Server capabilities
#[derive(Debug, Clone)]
pub struct ServerCapabilities {
    /// Text document sync capability
    pub text_document_sync: u32,
    /// Hover capability
    pub hover_provider: bool,
    /// Code action capability
    pub code_action_provider: bool,
    /// Diagnostics capability
    pub diagnostic_provider: bool,
    /// Completion capability
    pub completion_provider: bool,
}

impl Default for ServerCapabilities {
    fn default() -> Self {
        Self {
            text_document_sync: 1, // Full sync
            hover_provider: true,
            code_action_provider: true,
            diagnostic_provider: true,
            completion_provider: true,
        }
    }
}

impl ServerCapabilities {
    /// Convert to JSON
    pub fn to_json(&self) -> Value {
        json!({
            "textDocumentSync": self.text_document_sync,
            "hoverProvider": self.hover_provider,
            "codeActionProvider": self.code_action_provider,
            "diagnosticProvider": self.diagnostic_provider,
            "completionProvider": {
                "resolveProvider": true,
                "triggerCharacters": [".", ":", "::", "(", "[", "{", " "]
            },
        })
    }
}

/// Client capabilities
#[derive(Debug, Clone)]
pub struct ClientCapabilities {
    /// Raw capabilities from client
    pub raw: Value,
}

impl ClientCapabilities {
    /// Create from JSON value
    pub fn from_json(value: Value) -> Self {
        Self { raw: value }
    }
}

/// LSP Server
pub struct LspServer {
    /// Server state
    state: ServerState,
    /// Server capabilities
    capabilities: ServerCapabilities,
    /// Client capabilities
    client_capabilities: Option<ClientCapabilities>,
    /// Open documents (URI -> content)
    documents: HashMap<String, String>,
    /// Transport
    transport: AsyncStdioTransport,
    /// Hover provider
    hover_provider: HoverProvider,
    /// Diagnostics engine
    diagnostics_engine: Box<dyn DiagnosticsEngine>,
    /// Code actions engine
    code_actions_engine: Box<dyn CodeActionsEngine>,
    /// Completion handler
    completion_handler: Option<CompletionHandler>,
    /// Completion configuration
    completion_config: CompletionConfig,
    /// Refactoring handler
    refactoring_handler: RefactoringHandler,
}

impl LspServer {
    /// Create a new LSP server
    pub fn new() -> Self {
        Self {
            state: ServerState::Initializing,
            capabilities: ServerCapabilities::default(),
            client_capabilities: None,
            documents: HashMap::new(),
            transport: AsyncStdioTransport::new(),
            hover_provider: HoverProvider::new(),
            diagnostics_engine: Box::new(DefaultDiagnosticsEngine::new()),
            code_actions_engine: Box::new(DefaultCodeActionsEngine::new()),
            completion_handler: None,
            completion_config: CompletionConfig::default(),
            refactoring_handler: RefactoringHandler::new(),
        }
    }

    /// Create a new LSP server with custom completion config
    pub fn with_completion_config(completion_config: CompletionConfig) -> Self {
        Self {
            state: ServerState::Initializing,
            capabilities: ServerCapabilities::default(),
            client_capabilities: None,
            documents: HashMap::new(),
            transport: AsyncStdioTransport::new(),
            hover_provider: HoverProvider::new(),
            diagnostics_engine: Box::new(DefaultDiagnosticsEngine::new()),
            code_actions_engine: Box::new(DefaultCodeActionsEngine::new()),
            completion_handler: None,
            completion_config,
            refactoring_handler: RefactoringHandler::new(),
        }
    }

    /// Register a completion engine
    pub fn register_completion_engine(&mut self, engine: Arc<dyn CompletionEngine>) {
        self.completion_handler = Some(CompletionHandler::new(engine));
        info!("Completion engine registered");
    }

    /// Get completion configuration
    pub fn completion_config(&self) -> &CompletionConfig {
        &self.completion_config
    }

    /// Set completion configuration
    pub fn set_completion_config(&mut self, config: CompletionConfig) {
        self.completion_config = config;
        debug!("Completion configuration updated");
    }

    /// Check if completion is enabled
    pub fn is_completion_enabled(&self) -> bool {
        self.completion_config.enabled && self.completion_handler.is_some()
    }

    /// Get the refactoring handler
    pub fn refactoring_handler(&self) -> &RefactoringHandler {
        &self.refactoring_handler
    }

    /// Get mutable refactoring handler
    pub fn refactoring_handler_mut(&mut self) -> &mut RefactoringHandler {
        &mut self.refactoring_handler
    }

    /// Enable or disable refactoring
    pub fn set_refactoring_enabled(&mut self, enabled: bool) {
        self.refactoring_handler.set_enabled(enabled);
    }

    /// Check if refactoring is enabled
    pub fn is_refactoring_enabled(&self) -> bool {
        self.refactoring_handler.is_enabled()
    }

    /// Get the current server state
    pub fn state(&self) -> ServerState {
        self.state
    }

    /// Set the server state (for testing)
    pub fn set_state(&mut self, state: ServerState) {
        self.state = state;
    }

    /// Get server capabilities
    pub fn capabilities(&self) -> &ServerCapabilities {
        &self.capabilities
    }

    /// Get client capabilities
    pub fn client_capabilities(&self) -> Option<&ClientCapabilities> {
        self.client_capabilities.as_ref()
    }

    /// Get a document by URI
    pub fn get_document(&self, uri: &str) -> Option<&str> {
        self.documents.get(uri).map(|s| s.as_str())
    }

    /// Set a document
    pub fn set_document(&mut self, uri: String, content: String) {
        self.documents.insert(uri, content);
    }

    /// Remove a document
    pub fn remove_document(&mut self, uri: &str) {
        self.documents.remove(uri);
    }

    /// Handle initialize request
    pub async fn handle_initialize(&mut self, params: Value) -> LspResult<Value> {
        if self.state != ServerState::Initializing {
            return Err(LspError::InvalidRequest(
                "Server is not in initializing state".to_string(),
            ));
        }

        info!("Initializing LSP server");

        // Extract client capabilities
        if let Some(capabilities) = params.get("capabilities") {
            self.client_capabilities = Some(ClientCapabilities::from_json(capabilities.clone()));
            debug!("Client capabilities received");
        }

        // Return server capabilities
        info!("LSP server initialization complete");
        Ok(json!({
            "capabilities": self.capabilities.to_json(),
            "serverInfo": {
                "name": "lsp-server",
                "version": "0.1.0"
            }
        }))
    }

    /// Handle initialized notification
    pub async fn handle_initialized(&mut self) -> LspResult<()> {
        if self.state != ServerState::Initializing {
            return Err(LspError::InvalidRequest(
                "Server is not in initializing state".to_string(),
            ));
        }

        self.state = ServerState::Initialized;
        Ok(())
    }

    /// Handle shutdown request
    pub async fn handle_shutdown(&mut self) -> LspResult<Value> {
        if self.state != ServerState::Initialized {
            return Err(LspError::InvalidRequest(
                "Server is not initialized".to_string(),
            ));
        }

        info!("Shutdown request received");
        self.state = ServerState::ShuttingDown;
        info!("Server state changed to ShuttingDown");
        Ok(json!(null))
    }

    /// Handle exit notification
    pub async fn handle_exit(&mut self) -> LspResult<()> {
        if self.state != ServerState::ShuttingDown {
            return Err(LspError::InvalidRequest(
                "Server is not shutting down".to_string(),
            ));
        }

        info!("Exit notification received");
        self.state = ServerState::ShutDown;
        info!("Server state changed to ShutDown");
        Ok(())
    }

    /// Handle did_open notification
    pub async fn handle_did_open(&mut self, params: Value) -> LspResult<()> {
        if self.state != ServerState::Initialized {
            return Err(LspError::InvalidRequest(
                "Server is not initialized".to_string(),
            ));
        }

        let text_document = params
            .get("textDocument")
            .ok_or_else(|| LspError::InvalidParams("Missing textDocument".to_string()))?;

        let uri = text_document
            .get("uri")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LspError::InvalidParams("Missing uri".to_string()))?;

        let text = text_document
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LspError::InvalidParams("Missing text".to_string()))?;

        debug!("Document opened: uri={}, size={} bytes", uri, text.len());
        self.set_document(uri.to_string(), text.to_string());
        Ok(())
    }

    /// Handle did_change notification
    pub async fn handle_did_change(&mut self, params: Value) -> LspResult<()> {
        if self.state != ServerState::Initialized {
            return Err(LspError::InvalidRequest(
                "Server is not initialized".to_string(),
            ));
        }

        let text_document = params
            .get("textDocument")
            .ok_or_else(|| LspError::InvalidParams("Missing textDocument".to_string()))?;

        let uri = text_document
            .get("uri")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LspError::InvalidParams("Missing uri".to_string()))?;

        let content_changes = params
            .get("contentChanges")
            .and_then(|v| v.as_array())
            .ok_or_else(|| LspError::InvalidParams("Missing contentChanges".to_string()))?;

        // For full document sync, take the last change
        if let Some(last_change) = content_changes.last() {
            if let Some(text) = last_change.get("text").and_then(|v| v.as_str()) {
                self.set_document(uri.to_string(), text.to_string());
            }
        }

        Ok(())
    }

    /// Handle did_close notification
    pub async fn handle_did_close(&mut self, params: Value) -> LspResult<()> {
        if self.state != ServerState::Initialized {
            return Err(LspError::InvalidRequest(
                "Server is not initialized".to_string(),
            ));
        }

        let text_document = params
            .get("textDocument")
            .ok_or_else(|| LspError::InvalidParams("Missing textDocument".to_string()))?;

        let uri = text_document
            .get("uri")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LspError::InvalidParams("Missing uri".to_string()))?;

        debug!("Document closed: uri={}", uri);
        self.remove_document(uri);
        Ok(())
    }

    /// Handle hover request
    pub async fn handle_hover(&self, params: Value) -> LspResult<Value> {
        if self.state != ServerState::Initialized {
            return Err(LspError::InvalidRequest(
                "Server is not initialized".to_string(),
            ));
        }

        let text_document = params
            .get("textDocument")
            .ok_or_else(|| LspError::InvalidParams("Missing textDocument".to_string()))?;

        let uri = text_document
            .get("uri")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LspError::InvalidParams("Missing uri".to_string()))?;

        let position = params
            .get("position")
            .ok_or_else(|| LspError::InvalidParams("Missing position".to_string()))?;

        let line = position
            .get("line")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| LspError::InvalidParams("Missing line".to_string()))?
            as u32;

        let character = position
            .get("character")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| LspError::InvalidParams("Missing character".to_string()))?
            as u32;

        // Get document content
        let code = self
            .get_document(uri)
            .ok_or_else(|| LspError::InvalidParams(format!("Document not found: {}", uri)))?;

        // Get hover information
        let hover_info = self
            .hover_provider
            .get_hover_info(code, Position::new(line, character));

        // Convert to JSON response
        match hover_info {
            Some(info) => Ok(json!({
                "contents": {
                    "kind": match info.contents.kind {
                        crate::types::MarkupKind::PlainText => "plaintext",
                        crate::types::MarkupKind::Markdown => "markdown",
                    },
                    "value": info.contents.value,
                },
                "range": info.range.map(|r| {
                    json!({
                        "start": {
                            "line": r.start.line,
                            "character": r.start.character,
                        },
                        "end": {
                            "line": r.end.line,
                            "character": r.end.character,
                        },
                    })
                }),
            })),
            None => Ok(json!(null)),
        }
    }

    /// Handle diagnostics request
    pub async fn handle_diagnostics(&self, params: Value) -> LspResult<Value> {
        if self.state != ServerState::Initialized {
            return Err(LspError::InvalidRequest(
                "Server is not initialized".to_string(),
            ));
        }

        let text_document = params
            .get("textDocument")
            .ok_or_else(|| LspError::InvalidParams("Missing textDocument".to_string()))?;

        let uri = text_document
            .get("uri")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LspError::InvalidParams("Missing uri".to_string()))?;

        // Get document content
        let code = self
            .get_document(uri)
            .ok_or_else(|| LspError::InvalidParams(format!("Document not found: {}", uri)))?;

        // Detect language from URI
        let language = self.detect_language(uri);

        // Generate diagnostics
        let diagnostics = self
            .diagnostics_engine
            .generate_diagnostics(code, language)
            .map_err(|e| LspError::InternalError(e.to_string()))?;

        // Convert to JSON response
        let diagnostics_json: Vec<Value> = diagnostics
            .iter()
            .map(|diag| {
                json!({
                    "range": {
                        "start": {
                            "line": diag.range.start.line,
                            "character": diag.range.start.character,
                        },
                        "end": {
                            "line": diag.range.end.line,
                            "character": diag.range.end.character,
                        },
                    },
                    "severity": match diag.severity {
                        crate::types::DiagnosticSeverity::Error => 1,
                        crate::types::DiagnosticSeverity::Warning => 2,
                        crate::types::DiagnosticSeverity::Information => 3,
                        crate::types::DiagnosticSeverity::Hint => 4,
                    },
                    "message": diag.message,
                    "code": diag.code,
                    "source": diag.source,
                })
            })
            .collect();

        Ok(json!(diagnostics_json))
    }

    /// Handle completion request
    pub async fn handle_completion(&self, params: Value) -> LspResult<Value> {
        if self.state != ServerState::Initialized {
            return Err(LspError::InvalidRequest(
                "Server is not initialized".to_string(),
            ));
        }

        if !self.is_completion_enabled() {
            return Ok(json!([]));
        }

        let text_document = params
            .get("textDocument")
            .ok_or_else(|| LspError::InvalidParams("Missing textDocument".to_string()))?;

        let uri = text_document
            .get("uri")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LspError::InvalidParams("Missing uri".to_string()))?;

        let position = params
            .get("position")
            .ok_or_else(|| LspError::InvalidParams("Missing position".to_string()))?;

        let line = position
            .get("line")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| LspError::InvalidParams("Missing line".to_string()))?
            as u32;

        let character = position
            .get("character")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| LspError::InvalidParams("Missing character".to_string()))?
            as u32;

        // Get document content
        let code = self
            .get_document(uri)
            .ok_or_else(|| LspError::InvalidParams(format!("Document not found: {}", uri)))?;

        // Detect language from URI
        let language = self.detect_language(uri);

        // Get completion handler
        let handler = self.completion_handler.as_ref().ok_or_else(|| {
            LspError::InternalError("Completion engine not initialized".to_string())
        })?;

        // Generate completions
        let completions = handler
            .handle_completion(code, Position::new(line, character), language.as_str())
            .await?;

        Ok(json!(completions))
    }

    /// Handle completionItem/resolve request
    pub async fn handle_completion_resolve(&self, params: Value) -> LspResult<Value> {
        if self.state != ServerState::Initialized {
            return Err(LspError::InvalidRequest(
                "Server is not initialized".to_string(),
            ));
        }

        if !self.is_completion_enabled() {
            return Ok(params);
        }

        let handler = self.completion_handler.as_ref().ok_or_else(|| {
            LspError::InternalError("Completion engine not initialized".to_string())
        })?;

        handler.handle_completion_resolve(&params).await
    }

    /// Handle code action request
    pub async fn handle_code_action(&self, params: Value) -> LspResult<Value> {
        if self.state != ServerState::Initialized {
            return Err(LspError::InvalidRequest(
                "Server is not initialized".to_string(),
            ));
        }

        let text_document = params
            .get("textDocument")
            .ok_or_else(|| LspError::InvalidParams("Missing textDocument".to_string()))?;

        let uri = text_document
            .get("uri")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LspError::InvalidParams("Missing uri".to_string()))?;

        let context = params
            .get("context")
            .ok_or_else(|| LspError::InvalidParams("Missing context".to_string()))?;

        let diagnostics_array = context
            .get("diagnostics")
            .and_then(|v| v.as_array())
            .ok_or_else(|| LspError::InvalidParams("Missing diagnostics in context".to_string()))?;

        // Get document content
        let code = self
            .get_document(uri)
            .ok_or_else(|| LspError::InvalidParams(format!("Document not found: {}", uri)))?;

        let mut actions = Vec::new();

        // Generate code actions for each diagnostic
        for diag_json in diagnostics_array {
            // Parse diagnostic from JSON
            if let Ok(diagnostic) =
                serde_json::from_value::<crate::types::Diagnostic>(diag_json.clone())
            {
                match self
                    .code_actions_engine
                    .suggest_code_actions(&diagnostic, code)
                {
                    Ok(suggested_actions) => {
                        for action in suggested_actions {
                            actions.push(json!({
                                "title": action.title,
                                "kind": match action.kind {
                                    crate::types::CodeActionKind::QuickFix => "quickfix",
                                    crate::types::CodeActionKind::Refactor => "refactor",
                                    crate::types::CodeActionKind::RefactorExtract => "refactor.extract",
                                    crate::types::CodeActionKind::RefactorInline => "refactor.inline",
                                    crate::types::CodeActionKind::RefactorRewrite => "refactor.rewrite",
                                    crate::types::CodeActionKind::Source => "source",
                                    crate::types::CodeActionKind::SourceOrganizeImports => "source.organizeImports",
                                    crate::types::CodeActionKind::SourceFixAll => "source.fixAll",
                                },
                                "edit": {
                                    "changes": action.edit.changes.iter().map(|(uri, edits)| {
                                        (uri.clone(), edits.iter().map(|edit| {
                                            json!({
                                                "range": {
                                                    "start": {
                                                        "line": edit.range.start.line,
                                                        "character": edit.range.start.character,
                                                    },
                                                    "end": {
                                                        "line": edit.range.end.line,
                                                        "character": edit.range.end.character,
                                                    },
                                                },
                                                "newText": edit.new_text,
                                            })
                                        }).collect::<Vec<_>>())
                                    }).collect::<std::collections::HashMap<_, _>>(),
                                },
                            }));
                        }
                    }
                    Err(e) => {
                        // Log error but continue processing other diagnostics
                        eprintln!("Error generating code actions: {}", e);
                    }
                }
            }
        }

        Ok(json!(actions))
    }

    /// Detect language from file URI
    fn detect_language(&self, uri: &str) -> Language {
        // Extract file extension from URI
        if let Some(last_dot) = uri.rfind('.') {
            let ext = &uri[last_dot + 1..];
            Language::from_extension(ext)
        } else {
            Language::Unknown
        }
    }

    /// Process a message
    async fn process_message(&mut self, message: LspMessage) -> LspResult<Option<Value>> {
        match message {
            LspMessage::Request(req) => match req.method.as_str() {
                "initialize" => {
                    let params = req.params.unwrap_or(json!({}));
                    self.handle_initialize(params).await
                }
                "shutdown" => self.handle_shutdown().await,
                "textDocument/hover" => {
                    let params = req.params.unwrap_or(json!({}));
                    self.handle_hover(params).await
                }
                "textDocument/diagnostics" => {
                    let params = req.params.unwrap_or(json!({}));
                    self.handle_diagnostics(params).await
                }
                "textDocument/codeAction" => {
                    let params = req.params.unwrap_or(json!({}));
                    self.handle_code_action(params).await
                }
                "textDocument/completion" => {
                    let params = req.params.unwrap_or(json!({}));
                    self.handle_completion(params).await
                }
                "completionItem/resolve" => {
                    let params = req.params.unwrap_or(json!({}));
                    self.handle_completion_resolve(params).await
                }
                _ => Err(LspError::MethodNotFound(req.method)),
            }
            .map(Some),
            LspMessage::Notification(notif) => {
                match notif.method.as_str() {
                    "initialized" => self.handle_initialized().await,
                    "textDocument/didOpen" => {
                        let params = notif.params.unwrap_or(json!({}));
                        self.handle_did_open(params).await
                    }
                    "textDocument/didChange" => {
                        let params = notif.params.unwrap_or(json!({}));
                        self.handle_did_change(params).await
                    }
                    "textDocument/didClose" => {
                        let params = notif.params.unwrap_or(json!({}));
                        self.handle_did_close(params).await
                    }
                    "exit" => self.handle_exit().await,
                    _ => {
                        // Unknown notification, just ignore it
                        Ok(())
                    }
                }
                .map(|_| None)
            }
            LspMessage::Response(_) => {
                // Ignore responses from client
                Ok(None)
            }
        }
    }

    /// Run the server
    pub async fn run(&mut self) -> LspResult<()> {
        info!("LSP server started");

        loop {
            // Read message
            match self.transport.read_message().await {
                Ok(message) => {
                    // Log incoming message
                    match &message {
                        LspMessage::Request(req) => {
                            debug!("Received request: method={}, id={:?}", req.method, req.id);
                        }
                        LspMessage::Notification(notif) => {
                            debug!("Received notification: method={}", notif.method);
                        }
                        LspMessage::Response(_) => {
                            debug!("Received response");
                        }
                    }

                    // Process message
                    match self.process_message(message.clone()).await {
                        Ok(Some(result)) => {
                            // Send response
                            if let LspMessage::Request(req) = message {
                                debug!("Sending response for request: id={:?}", req.id);
                                let response = JsonRpcResponse::success(req.id, result);
                                let response_msg = LspMessage::Response(response);
                                if let Err(e) = self.transport.write_message(&response_msg).await {
                                    error!("Failed to send response: {}", e);
                                }
                            }
                        }
                        Ok(None) => {
                            // Notification processed, no response needed
                            if let LspMessage::Notification(notif) = message {
                                debug!("Notification processed: method={}", notif.method);
                            }
                        }
                        Err(err) => {
                            // Send error response
                            if let LspMessage::Request(req) = message {
                                warn!("Error processing request: {}", err);
                                let error = match err {
                                    LspError::MethodNotFound(method) => {
                                        error!("Method not found: {}", method);
                                        JsonRpcError::method_not_found(method)
                                    }
                                    LspError::InvalidParams(msg) => {
                                        warn!("Invalid parameters: {}", msg);
                                        JsonRpcError::invalid_params(msg)
                                    }
                                    LspError::ParseError(msg) => {
                                        error!("Parse error: {}", msg);
                                        JsonRpcError::parse_error(msg)
                                    }
                                    LspError::TimeoutError(msg) => {
                                        warn!("Timeout error: {}", msg);
                                        JsonRpcError::internal_error(msg)
                                    }
                                    _ => {
                                        error!("Internal error: {}", err);
                                        JsonRpcError::internal_error(err.to_string())
                                    }
                                };
                                let response = JsonRpcResponse::error(req.id, error);
                                let response_msg = LspMessage::Response(response);
                                if let Err(e) = self.transport.write_message(&response_msg).await {
                                    error!("Failed to send error response: {}", e);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to read message: {}", e);
                    // Continue processing, don't break on read errors
                }
            }

            // Check if server should exit
            if self.state == ServerState::ShutDown {
                info!("LSP server shutting down");
                break;
            }
        }

        info!("LSP server stopped");
        Ok(())
    }
}

impl Default for LspServer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let server = LspServer::new();
        assert_eq!(server.state(), ServerState::Initializing);
    }

    #[test]
    fn test_server_capabilities() {
        let server = LspServer::new();
        let caps = server.capabilities();
        assert!(caps.hover_provider);
        assert!(caps.code_action_provider);
    }

    #[test]
    fn test_document_management() {
        let mut server = LspServer::new();
        server.set_document("file://test.rs".to_string(), "fn main() {}".to_string());
        assert_eq!(server.get_document("file://test.rs"), Some("fn main() {}"));

        server.remove_document("file://test.rs");
        assert_eq!(server.get_document("file://test.rs"), None);
    }

    #[test]
    fn test_server_capabilities_json() {
        let caps = ServerCapabilities::default();
        let json = caps.to_json();
        assert!(json.get("textDocumentSync").is_some());
        assert!(json.get("hoverProvider").is_some());
    }

    #[test]
    fn test_error_handling_invalid_request() {
        let server = LspServer::new();
        // Server should be in Initializing state, so requests should fail
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(server.handle_hover(json!({})));
        assert!(result.is_err());
    }

    #[test]
    fn test_error_handling_missing_params() {
        let mut server = LspServer::new();
        server.state = ServerState::Initialized;

        // Missing textDocument parameter
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(server.handle_hover(json!({})));
        assert!(result.is_err());
    }

    #[test]
    fn test_language_detection_rust() {
        let server = LspServer::new();
        let lang = server.detect_language("file://test.rs");
        assert_eq!(lang, Language::Rust);
    }

    #[test]
    fn test_language_detection_typescript() {
        let server = LspServer::new();
        let lang = server.detect_language("file://test.ts");
        assert_eq!(lang, Language::TypeScript);
    }

    #[test]
    fn test_language_detection_python() {
        let server = LspServer::new();
        let lang = server.detect_language("file://test.py");
        assert_eq!(lang, Language::Python);
    }

    #[test]
    fn test_language_detection_unknown() {
        let server = LspServer::new();
        let lang = server.detect_language("file://test.unknown");
        assert_eq!(lang, Language::Unknown);
    }

    #[test]
    fn test_error_handling_document_not_found() {
        let mut server = LspServer::new();
        server.state = ServerState::Initialized;

        // Try to get hover for non-existent document
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(server.handle_hover(json!({
                "textDocument": { "uri": "file://nonexistent.rs" },
                "position": { "line": 0, "character": 0 }
            })));
        assert!(result.is_err());
    }

    #[test]
    fn test_diagnostics_handler_integration() {
        let mut server = LspServer::new();
        server.state = ServerState::Initialized;

        // Add a document
        server.set_document("file://test.rs".to_string(), "fn main() {}".to_string());

        // Request diagnostics
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(server.handle_diagnostics(json!({
                "textDocument": { "uri": "file://test.rs" }
            })));

        // Should succeed and return a JSON array
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.is_array());
    }

    #[test]
    fn test_code_action_handler_integration() {
        let mut server = LspServer::new();
        server.state = ServerState::Initialized;

        // Add a document
        server.set_document("file://test.rs".to_string(), "fn main() {}".to_string());

        // Request code actions
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(server.handle_code_action(json!({
                "textDocument": { "uri": "file://test.rs" },
                "context": { "diagnostics": [] }
            })));

        // Should succeed and return a JSON array
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.is_array());
    }

    #[test]
    fn test_hover_handler_integration() {
        let mut server = LspServer::new();
        server.state = ServerState::Initialized;

        // Add a document
        server.set_document("file://test.rs".to_string(), "fn main() {}".to_string());

        // Request hover information
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(server.handle_hover(json!({
                "textDocument": { "uri": "file://test.rs" },
                "position": { "line": 0, "character": 0 }
            })));

        // Should succeed (may return null if no symbol at position)
        assert!(result.is_ok());
    }

    #[test]
    fn test_diagnostics_with_language_detection() {
        let mut server = LspServer::new();
        server.state = ServerState::Initialized;

        // Add Rust document
        server.set_document("file://test.rs".to_string(), "fn main() {}".to_string());

        // Request diagnostics for Rust file
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(server.handle_diagnostics(json!({
                "textDocument": { "uri": "file://test.rs" }
            })));

        assert!(result.is_ok());

        // Add Python document
        server.set_document("file://test.py".to_string(), "def main(): pass".to_string());

        // Request diagnostics for Python file
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(server.handle_diagnostics(json!({
                "textDocument": { "uri": "file://test.py" }
            })));

        assert!(result.is_ok());
    }
}
