//! LSP capability negotiation

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Client capabilities for LSP initialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCapabilities {
    /// Text document capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_document: Option<TextDocumentClientCapabilities>,
    /// Workspace capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace: Option<WorkspaceClientCapabilities>,
    /// General capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub general: Option<GeneralClientCapabilities>,
}

/// Text document client capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDocumentClientCapabilities {
    /// Synchronization capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub synchronization: Option<SynchronizationCapability>,
    /// Completion capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion: Option<CompletionCapability>,
    /// Hover capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hover: Option<HoverCapability>,
    /// Diagnostic capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publish_diagnostics: Option<PublishDiagnosticsCapability>,
}

/// Synchronization capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynchronizationCapability {
    /// Whether the client supports incremental synchronization
    #[serde(skip_serializing_if = "Option::is_none")]
    pub did_save: Option<bool>,
    /// Whether the client supports full document synchronization
    #[serde(skip_serializing_if = "Option::is_none")]
    pub will_save: Option<bool>,
}

/// Completion capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionCapability {
    /// Whether the client supports completion item snippets
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_item: Option<CompletionItemCapability>,
}

/// Completion item capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionItemCapability {
    /// Whether the client supports snippet syntax
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snippet_support: Option<bool>,
}

/// Hover capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoverCapability {
    /// Whether the client supports markdown content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_format: Option<Vec<String>>,
}

/// Publish diagnostics capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishDiagnosticsCapability {
    /// Whether the client supports related information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_information: Option<bool>,
}

/// Workspace client capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceClientCapabilities {
    /// Whether the client supports workspace folders
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_folders: Option<bool>,
}

/// General client capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralClientCapabilities {
    /// Whether the client supports regular expressions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regular_expressions: Option<RegularExpressionCapability>,
}

/// Regular expression capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegularExpressionCapability {
    /// The regex engine used
    pub engine: String,
}

/// Server capabilities from LSP initialization response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    /// Completion provider capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_provider: Option<Value>,
    /// Hover provider capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hover_provider: Option<Value>,
    /// Definition provider capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub definition_provider: Option<Value>,
    /// References provider capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub references_provider: Option<Value>,
    /// Document symbol provider capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_symbol_provider: Option<Value>,
    /// Workspace symbol provider capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_symbol_provider: Option<Value>,
    /// Code action provider capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_action_provider: Option<Value>,
    /// Text document sync capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_document_sync: Option<Value>,
    /// Diagnostic provider capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostic_provider: Option<Value>,
}

/// Handles LSP capability negotiation
pub struct CapabilityNegotiator;

impl CapabilityNegotiator {
    /// Create a new capability negotiator
    pub fn new() -> Self {
        Self
    }

    /// Create default client capabilities for ricecoder
    pub fn default_client_capabilities() -> ClientCapabilities {
        ClientCapabilities {
            text_document: Some(TextDocumentClientCapabilities {
                synchronization: Some(SynchronizationCapability {
                    did_save: Some(true),
                    will_save: Some(false),
                }),
                completion: Some(CompletionCapability {
                    completion_item: Some(CompletionItemCapability {
                        snippet_support: Some(true),
                    }),
                }),
                hover: Some(HoverCapability {
                    content_format: Some(vec!["markdown".to_string(), "plaintext".to_string()]),
                }),
                publish_diagnostics: Some(PublishDiagnosticsCapability {
                    related_information: Some(true),
                }),
            }),
            workspace: Some(WorkspaceClientCapabilities {
                workspace_folders: Some(true),
            }),
            general: Some(GeneralClientCapabilities {
                regular_expressions: Some(RegularExpressionCapability {
                    engine: "ECMAScript".to_string(),
                }),
            }),
        }
    }

    /// Create initialization request parameters
    pub fn create_initialize_params(
        process_id: Option<u32>,
        root_path: Option<String>,
        root_uri: Option<String>,
    ) -> Value {
        let mut params = json!({
            "processId": process_id,
            "capabilities": Self::default_client_capabilities(),
        });

        if let Some(root_path) = root_path {
            params["rootPath"] = json!(root_path);
        }

        if let Some(root_uri) = root_uri {
            params["rootUri"] = json!(root_uri);
        }

        params
    }

    /// Check if server supports a capability
    pub fn supports_capability(capabilities: &ServerCapabilities, capability: &str) -> bool {
        match capability {
            "completion" => capabilities.completion_provider.is_some(),
            "hover" => capabilities.hover_provider.is_some(),
            "definition" => capabilities.definition_provider.is_some(),
            "references" => capabilities.references_provider.is_some(),
            "documentSymbol" => capabilities.document_symbol_provider.is_some(),
            "workspaceSymbol" => capabilities.workspace_symbol_provider.is_some(),
            "codeAction" => capabilities.code_action_provider.is_some(),
            "diagnostics" => capabilities.diagnostic_provider.is_some(),
            _ => false,
        }
    }

    /// Get list of supported capabilities
    pub fn get_supported_capabilities(capabilities: &ServerCapabilities) -> Vec<String> {
        let mut supported = Vec::new();

        if capabilities.completion_provider.is_some() {
            supported.push("completion".to_string());
        }
        if capabilities.hover_provider.is_some() {
            supported.push("hover".to_string());
        }
        if capabilities.definition_provider.is_some() {
            supported.push("definition".to_string());
        }
        if capabilities.references_provider.is_some() {
            supported.push("references".to_string());
        }
        if capabilities.document_symbol_provider.is_some() {
            supported.push("documentSymbol".to_string());
        }
        if capabilities.workspace_symbol_provider.is_some() {
            supported.push("workspaceSymbol".to_string());
        }
        if capabilities.code_action_provider.is_some() {
            supported.push("codeAction".to_string());
        }
        if capabilities.diagnostic_provider.is_some() {
            supported.push("diagnostics".to_string());
        }

        supported
    }
}

impl Default for CapabilityNegotiator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_client_capabilities() {
        let caps = CapabilityNegotiator::default_client_capabilities();

        assert!(caps.text_document.is_some());
        assert!(caps.workspace.is_some());
        assert!(caps.general.is_some());

        let text_doc = caps.text_document.unwrap();
        assert!(text_doc.synchronization.is_some());
        assert!(text_doc.completion.is_some());
        assert!(text_doc.hover.is_some());
    }

    #[test]
    fn test_create_initialize_params() {
        let params = CapabilityNegotiator::create_initialize_params(
            Some(1234),
            Some("/path/to/project".to_string()),
            Some("file:///path/to/project".to_string()),
        );

        assert_eq!(params["processId"], 1234);
        assert_eq!(params["rootPath"], "/path/to/project");
        assert_eq!(params["rootUri"], "file:///path/to/project");
        assert!(params["capabilities"].is_object());
    }

    #[test]
    fn test_supports_capability() {
        let caps = ServerCapabilities {
            completion_provider: Some(json!({})),
            hover_provider: Some(json!({})),
            definition_provider: None,
            references_provider: None,
            document_symbol_provider: None,
            workspace_symbol_provider: None,
            code_action_provider: None,
            diagnostic_provider: None,
            text_document_sync: None,
        };

        assert!(CapabilityNegotiator::supports_capability(
            &caps,
            "completion"
        ));
        assert!(CapabilityNegotiator::supports_capability(&caps, "hover"));
        assert!(!CapabilityNegotiator::supports_capability(
            &caps,
            "definition"
        ));
    }

    #[test]
    fn test_get_supported_capabilities() {
        let caps = ServerCapabilities {
            completion_provider: Some(json!({})),
            hover_provider: Some(json!({})),
            definition_provider: Some(json!({})),
            references_provider: None,
            document_symbol_provider: None,
            workspace_symbol_provider: None,
            code_action_provider: None,
            diagnostic_provider: None,
            text_document_sync: None,
        };

        let supported = CapabilityNegotiator::get_supported_capabilities(&caps);

        assert!(supported.contains(&"completion".to_string()));
        assert!(supported.contains(&"hover".to_string()));
        assert!(supported.contains(&"definition".to_string()));
        assert_eq!(supported.len(), 3);
    }
}
