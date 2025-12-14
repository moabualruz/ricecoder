//! MCP Transport Protocols
//!
//! Implements the transport layer for MCP communication including:
//! - stdio: Standard input/output streams
//! - HTTP: RESTful HTTP transport
//! - SSE: Server-Sent Events for real-time communication

use async_trait::async_trait;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader as AsyncBufReader};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::error::{Error, Result};

/// Core MCP message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum MCPMessage {
    #[serde(rename = "request")]
    Request(MCPRequest),
    #[serde(rename = "response")]
    Response(MCPResponse),
    #[serde(rename = "notification")]
    Notification(MCPNotification),
    #[serde(rename = "error")]
    Error(MCPError),
}

/// MCP request message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPRequest {
    pub id: String,
    pub method: String,
    pub params: serde_json::Value,
}

/// MCP response message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPResponse {
    pub id: String,
    pub result: serde_json::Value,
}

/// MCP notification message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPNotification {
    pub method: String,
    pub params: serde_json::Value,
}

/// MCP error message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPError {
    pub id: Option<String>,
    pub error: MCPErrorData,
}

/// MCP error data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPErrorData {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// Transport trait for MCP communication
#[async_trait]
pub trait MCPTransport: Send + Sync {
    /// Send a message
    async fn send(&self, message: &MCPMessage) -> Result<()>;

    /// Receive a message
    async fn receive(&self) -> Result<MCPMessage>;

    /// Check if transport is connected
    async fn is_connected(&self) -> bool;

    /// Close the transport
    async fn close(&self) -> Result<()>;
}

/// Stdio transport for MCP communication
pub struct StdioTransport {
    child: std::sync::Mutex<Option<Child>>,
    stdin: std::sync::Mutex<Option<std::process::ChildStdin>>,
    stdout_reader: std::sync::Mutex<Option<AsyncBufReader<tokio::process::ChildStdout>>>,
}

impl StdioTransport {
    /// Create a new stdio transport by spawning a process
    pub fn new(command: &str, args: &[&str]) -> Result<Self> {
        info!("Starting MCP server process: {} {:?}", command, args);

        let mut child = Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| Error::ConnectionError(format!("Failed to spawn process: {}", e)))?;

        let stdin = child.stdin.take();
        let stdout = child.stdout.take();

        let stdout_reader = stdout.map(|s| AsyncBufReader::new(s));

        Ok(Self {
            child: std::sync::Mutex::new(Some(child)),
            stdin: std::sync::Mutex::new(stdin),
            stdout_reader: std::sync::Mutex::new(stdout_reader),
        })
    }
}

#[async_trait]
impl MCPTransport for StdioTransport {
    async fn send(&self, message: &MCPMessage) -> Result<()> {
        let mut stdin_guard = self.stdin.lock().unwrap();
        if let Some(ref mut stdin) = *stdin_guard {
            let json = serde_json::to_string(message)
                .map_err(|e| Error::SerializationError(e))?;

            // Add newline for message framing
            let framed_message = format!("{}\n", json);

            // For synchronous stdin, we need to use a different approach
            use std::io::Write;
            stdin.write_all(framed_message.as_bytes())
                .map_err(|e| Error::ConnectionError(format!("Failed to write to stdin: {}", e)))?;
            stdin.flush()
                .map_err(|e| Error::ConnectionError(format!("Failed to flush stdin: {}", e)))?;

            debug!("Sent MCP message via stdio: {}", json);
            Ok(())
        } else {
            Err(Error::ConnectionError("Stdin not available".to_string()))
        }
    }

    async fn receive(&self) -> Result<MCPMessage> {
        let mut reader_guard = self.stdout_reader.lock().unwrap();
        if let Some(ref mut reader) = *reader_guard {
            let mut line = String::new();
            let bytes_read = reader.read_line(&mut line).await
                .map_err(|e| Error::ConnectionError(format!("Failed to read from stdout: {}", e)))?;

            if bytes_read == 0 {
                return Err(Error::ConnectionError("EOF reached".to_string()));
            }

            let message: MCPMessage = serde_json::from_str(line.trim())
                .map_err(|e| Error::SerializationError(e))?;

            debug!("Received MCP message via stdio: {:?}", message);
            Ok(message)
        } else {
            Err(Error::ConnectionError("Stdout not available".to_string()))
        }
    }

    async fn is_connected(&self) -> bool {
        let child_guard = self.child.lock().unwrap();
        child_guard.as_ref().map_or(false, |c| c.try_wait().unwrap_or(None).is_none())
    }

    async fn close(&self) -> Result<()> {
        let mut child_guard = self.child.lock().unwrap();
        if let Some(child) = child_guard.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        Ok(())
    }
}

impl Drop for StdioTransport {
    fn drop(&mut self) {
        if let Ok(mut child_guard) = self.child.lock() {
            if let Some(child) = child_guard.take() {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }
}

/// HTTP transport for MCP communication
pub struct HTTPTransport {
    base_url: String,
    client: reqwest::Client,
}

impl HTTPTransport {
    /// Create a new HTTP transport
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Create with custom client
    pub fn with_client(base_url: &str, client: reqwest::Client) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client,
        }
    }
}

#[async_trait]
impl MCPTransport for HTTPTransport {
    async fn send(&self, message: &MCPMessage) -> Result<()> {
        match message {
            MCPMessage::Request(request) => {
                let url = format!("{}/{}", self.base_url, request.method);
                let response = self.client
                    .post(&url)
                    .json(&request.params)
                    .send()
                    .await
                    .map_err(|e| Error::ConnectionError(format!("HTTP request failed: {}", e)))?;

                if !response.status().is_success() {
                    return Err(Error::ServerError(format!("HTTP {}: {}", response.status(), response.text().await.unwrap_or_default())));
                }

                debug!("Sent MCP request via HTTP: {} to {}", request.method, url);
                Ok(())
            }
            MCPMessage::Notification(notification) => {
                let url = format!("{}/notify/{}", self.base_url, notification.method);
                let response = self.client
                    .post(&url)
                    .json(&notification.params)
                    .send()
                    .await
                    .map_err(|e| Error::ConnectionError(format!("HTTP notification failed: {}", e)))?;

                if !response.status().is_success() {
                    warn!("HTTP notification failed with status: {}", response.status());
                }

                debug!("Sent MCP notification via HTTP: {}", notification.method);
                Ok(())
            }
            _ => Err(Error::ValidationError("HTTP transport only supports requests and notifications".to_string())),
        }
    }

    async fn receive(&self) -> Result<MCPMessage> {
        // HTTP transport is request-response, so receiving is not directly supported
        // In a real implementation, this might poll for responses or use webhooks
        Err(Error::ValidationError("HTTP transport does not support receiving messages".to_string()))
    }

    async fn is_connected(&self) -> bool {
        // Simple connectivity check
        match self.client.get(&format!("{}/health", self.base_url)).send().await {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }

    async fn close(&self) -> Result<()> {
        // HTTP client doesn't need explicit closing
        Ok(())
    }
}

/// SSE (Server-Sent Events) transport for MCP communication
pub struct SSETransport {
    url: String,
    client: reqwest::Client,
    event_receiver: std::sync::Mutex<Option<mpsc::Receiver<String>>>,
    _handle: std::sync::Mutex<Option<tokio::task::JoinHandle<()>>>,
}

impl SSETransport {
    /// Create a new SSE transport
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            client: reqwest::Client::new(),
            event_receiver: None,
            _handle: None,
        }
    }

    /// Start listening for SSE events
    pub async fn connect(&mut self) -> Result<()> {
        let (tx, rx) = mpsc::channel(100);
        *self.event_receiver.lock().unwrap() = Some(rx);

        let url = self.url.clone();
        let client = self.client.clone();

        let handle = tokio::spawn(async move {
            loop {
                match Self::listen_for_events(&url, &client, &tx).await {
                    Ok(_) => break,
                    Err(e) => {
                        error!("SSE connection error: {}", e);
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    }
                }
            }
        });

        *self._handle.lock().unwrap() = Some(handle);
        Ok(())
    }

    async fn listen_for_events(
        url: &str,
        client: &reqwest::Client,
        tx: &mpsc::Sender<String>,
    ) -> Result<()> {
        let response = client
            .get(url)
            .header("Accept", "text/event-stream")
            .send()
            .await
            .map_err(|e| Error::ConnectionError(format!("SSE connection failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::ConnectionError(format!("SSE connection failed with status: {}", response.status())));
        }

        let mut stream = response.bytes_stream();
        let mut buffer = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| Error::ConnectionError(format!("SSE stream error: {}", e)))?;
            let text = String::from_utf8_lossy(&chunk);

            buffer.push_str(&text);

            // Process complete events
            while let Some(event_end) = buffer.find("\n\n") {
                let event = buffer[..event_end].to_string();
                buffer = buffer[event_end + 2..].to_string();

                if let Some(data_line) = event.lines().find(|line| line.starts_with("data: ")) {
                    let data = data_line.trim_start_matches("data: ");
                    let _ = tx.send(data.to_string()).await;
                }
            }
        }

        Ok(())
    }
}

#[async_trait]
impl MCPTransport for SSETransport {
    async fn send(&self, message: &MCPMessage) -> Result<()> {
        // SSE is typically receive-only, but we could implement sending via a separate HTTP endpoint
        Err(Error::ValidationError("SSE transport does not support sending messages".to_string()))
    }

    async fn receive(&self) -> Result<MCPMessage> {
        let mut receiver_guard = self.event_receiver.lock().unwrap();
        if let Some(ref mut receiver) = *receiver_guard {
            match receiver.recv().await {
                Some(data) => {
                    let message: MCPMessage = serde_json::from_str(&data)
                        .map_err(|e| Error::SerializationError(e))?;
                    debug!("Received MCP message via SSE: {:?}", message);
                    Ok(message)
                }
                None => Err(Error::ConnectionError("SSE receiver closed".to_string())),
            }
        } else {
            Err(Error::ConnectionError("SSE transport not connected".to_string()))
        }
    }

    async fn is_connected(&self) -> bool {
        self.event_receiver.lock().unwrap().is_some()
    }

    async fn close(&self) -> Result<()> {
        *self.event_receiver.lock().unwrap() = None;
        if let Some(handle) = self._handle.lock().unwrap().take() {
            handle.abort();
        }
        Ok(())
    }
}

/// Transport factory for creating transports
pub struct TransportFactory;

impl TransportFactory {
    /// Create a transport based on configuration
    pub fn create(config: &TransportConfig) -> Result<Box<dyn MCPTransport>> {
        match config.transport_type {
            TransportType::Stdio => {
                let stdio_config = config.stdio_config.as_ref()
                    .ok_or_else(|| Error::ConfigError("Stdio config required".to_string()))?;
                let args: Vec<&str> = stdio_config.args.iter().map(|s| s.as_str()).collect();
                let transport = StdioTransport::new(&stdio_config.command, &args)?;
                Ok(Box::new(transport))
            }
            TransportType::HTTP => {
                let http_config = config.http_config.as_ref()
                    .ok_or_else(|| Error::ConfigError("HTTP config required".to_string()))?;
                let transport = HTTPTransport::new(&http_config.base_url);
                Ok(Box::new(transport))
            }
            TransportType::SSE => {
                let sse_config = config.sse_config.as_ref()
                    .ok_or_else(|| Error::ConfigError("SSE config required".to_string()))?;
                let mut transport = SSETransport::new(&sse_config.url);
                // Note: connect() should be called separately for SSE
                Ok(Box::new(transport))
            }
        }
    }
}

/// Transport configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportConfig {
    pub transport_type: TransportType,
    pub stdio_config: Option<StdioConfig>,
    pub http_config: Option<HTTPConfig>,
    pub sse_config: Option<SSEConfig>,
}

/// Transport types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransportType {
    #[serde(rename = "stdio")]
    Stdio,
    #[serde(rename = "http")]
    HTTP,
    #[serde(rename = "sse")]
    SSE,
}

/// Stdio transport configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StdioConfig {
    pub command: String,
    pub args: Vec<String>,
}

/// HTTP transport configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HTTPConfig {
    pub base_url: String,
    pub timeout_ms: Option<u64>,
}

/// SSE transport configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SSEConfig {
    pub url: String,
    pub reconnect_interval_ms: Option<u64>,
}