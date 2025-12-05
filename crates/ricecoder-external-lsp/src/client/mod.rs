//! LSP client communication and protocol handling

pub mod capabilities;
pub mod connection;
pub mod protocol;

pub use capabilities::{CapabilityNegotiator, ClientCapabilities, ServerCapabilities};
pub use connection::{LspConnection, PendingRequest};
pub use protocol::{
    JsonRpcError, JsonRpcHandler, JsonRpcMessage, JsonRpcNotification, JsonRpcRequest,
    JsonRpcResponse, RequestId,
};
