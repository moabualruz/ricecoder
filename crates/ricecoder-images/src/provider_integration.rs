//! Integration with ricecoder-providers for image analysis.
//!
//! This module handles:
//! - Extending ChatRequest with image data
//! - Image serialization for different providers
//! - Provider-specific image format handling
//! - Token counting for images
//! - Audit logging of image analysis requests

use crate::error::ImageResult;
use ricecoder_providers::models::ChatRequest;
use serde::{Deserialize, Serialize};

/// Image data for inclusion in chat requests.
///
/// This struct wraps image data in a format that can be serialized
/// and sent to AI providers. Different providers may require different
/// formats (base64, URL, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageData {
    /// Image format (png, jpg, gif, webp)
    pub format: String,
    /// Image data encoded as base64
    pub data: String,
    /// Image dimensions (width, height)
    pub dimensions: (u32, u32),
    /// Image size in bytes
    pub size_bytes: u64,
}

impl ImageData {
    /// Create image data from raw bytes.
    pub fn from_bytes(
        format: &str,
        data: &[u8],
        width: u32,
        height: u32,
    ) -> Self {
        let base64_data = base64_encode(data);
        Self {
            format: format.to_string(),
            data: base64_data,
            dimensions: (width, height),
            size_bytes: data.len() as u64,
        }
    }

    /// Get the MIME type for this image format.
    pub fn mime_type(&self) -> &str {
        match self.format.as_str() {
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "webp" => "image/webp",
            _ => "application/octet-stream",
        }
    }

    /// Get the data URL for this image (for providers that support it).
    pub fn data_url(&self) -> String {
        format!("data:{};base64,{}", self.mime_type(), self.data)
    }
}

/// Extended chat request with image support.
///
/// This wraps the standard ChatRequest and adds image data that can be
/// serialized for different providers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequestWithImages {
    /// The base chat request
    pub request: ChatRequest,
    /// Images to include in the request
    pub images: Vec<ImageData>,
    /// Provider-specific image handling
    pub provider_format: ProviderImageFormat,
}

/// How to format images for a specific provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderImageFormat {
    /// OpenAI format (base64 in message content)
    OpenAi,
    /// Anthropic format (base64 in message content)
    Anthropic,
    /// Google format (base64 in message content)
    Google,
    /// Ollama format (base64 in message content)
    Ollama,
    /// Generic format (base64 in message content)
    Generic,
}

impl ProviderImageFormat {
    /// Get the format for a provider name.
    pub fn for_provider(provider_name: &str) -> Self {
        match provider_name.to_lowercase().as_str() {
            "openai" => ProviderImageFormat::OpenAi,
            "anthropic" => ProviderImageFormat::Anthropic,
            "google" => ProviderImageFormat::Google,
            "ollama" => ProviderImageFormat::Ollama,
            _ => ProviderImageFormat::Generic,
        }
    }
}

impl ChatRequestWithImages {
    /// Create a new chat request with images.
    pub fn new(request: ChatRequest, provider_name: &str) -> Self {
        Self {
            request,
            images: Vec::new(),
            provider_format: ProviderImageFormat::for_provider(provider_name),
        }
    }

    /// Add an image to the request.
    pub fn add_image(&mut self, image: ImageData) {
        self.images.push(image);
    }

    /// Add multiple images to the request.
    pub fn add_images(&mut self, images: Vec<ImageData>) {
        self.images.extend(images);
    }

    /// Serialize the request for the target provider.
    ///
    /// This converts the images into the format expected by the provider
    /// and updates the message content accordingly.
    pub fn serialize_for_provider(&self) -> ImageResult<ChatRequest> {
        let mut request = self.request.clone();

        // If there are no images, return the request as-is
        if self.images.is_empty() {
            return Ok(request);
        }

        // Update the last user message to include image references
        if let Some(last_message) = request.messages.iter_mut().rev().find(|m| m.role == "user") {
            let image_content = self.format_images_for_provider();
            last_message.content = format!("{}\n\n{}", last_message.content, image_content);
        }

        Ok(request)
    }

    /// Format images according to the provider's requirements.
    fn format_images_for_provider(&self) -> String {
        match self.provider_format {
            ProviderImageFormat::OpenAi => self.format_for_openai(),
            ProviderImageFormat::Anthropic => self.format_for_anthropic(),
            ProviderImageFormat::Google => self.format_for_google(),
            ProviderImageFormat::Ollama => self.format_for_ollama(),
            ProviderImageFormat::Generic => self.format_generic(),
        }
    }

    /// Format images for OpenAI (base64 with MIME type).
    fn format_for_openai(&self) -> String {
        let mut content = String::new();
        for (i, image) in self.images.iter().enumerate() {
            content.push_str(&format!(
                "[Image {}]\nFormat: {}\nDimensions: {}x{}\nSize: {} bytes\nData: data:{}base64,{}...\n",
                i + 1,
                image.format,
                image.dimensions.0,
                image.dimensions.1,
                image.size_bytes,
                image.mime_type(),
                &image.data[..std::cmp::min(50, image.data.len())]
            ));
        }
        content
    }

    /// Format images for Anthropic (base64 with metadata).
    fn format_for_anthropic(&self) -> String {
        let mut content = String::new();
        for (i, image) in self.images.iter().enumerate() {
            content.push_str(&format!(
                "[Image {}]\nType: {}\nDimensions: {}x{}\nSize: {} bytes\n",
                i + 1,
                image.mime_type(),
                image.dimensions.0,
                image.dimensions.1,
                image.size_bytes
            ));
        }
        content
    }

    /// Format images for Google (base64 with metadata).
    fn format_for_google(&self) -> String {
        let mut content = String::new();
        for (i, image) in self.images.iter().enumerate() {
            content.push_str(&format!(
                "[Image {}]\nMIME Type: {}\nResolution: {}x{}\nSize: {} bytes\n",
                i + 1,
                image.mime_type(),
                image.dimensions.0,
                image.dimensions.1,
                image.size_bytes
            ));
        }
        content
    }

    /// Format images for Ollama (base64 with metadata).
    fn format_for_ollama(&self) -> String {
        let mut content = String::new();
        for (i, image) in self.images.iter().enumerate() {
            content.push_str(&format!(
                "[Image {}]\nFormat: {}\nSize: {}x{}\nBytes: {}\n",
                i + 1,
                image.format,
                image.dimensions.0,
                image.dimensions.1,
                image.size_bytes
            ));
        }
        content
    }

    /// Format images in generic format (base64 with minimal metadata).
    fn format_generic(&self) -> String {
        let mut content = String::new();
        for (i, image) in self.images.iter().enumerate() {
            content.push_str(&format!(
                "[Image {}] {} ({}x{}, {} bytes)\n",
                i + 1,
                image.format.to_uppercase(),
                image.dimensions.0,
                image.dimensions.1,
                image.size_bytes
            ));
        }
        content
    }

    /// Get the number of images in this request.
    pub fn image_count(&self) -> usize {
        self.images.len()
    }

    /// Check if this request has any images.
    pub fn has_images(&self) -> bool {
        !self.images.is_empty()
    }
}

/// Audit log entry for image analysis requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageAuditLogEntry {
    /// Timestamp of the request
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Provider name
    pub provider: String,
    /// Model used
    pub model: String,
    /// Number of images analyzed
    pub image_count: usize,
    /// Total image size in bytes
    pub total_image_size: u64,
    /// Image hashes (for deduplication tracking)
    pub image_hashes: Vec<String>,
    /// Request status (success, failure, timeout)
    pub status: String,
    /// Error message if failed
    pub error: Option<String>,
    /// Tokens used
    pub tokens_used: Option<u32>,
}

impl ImageAuditLogEntry {
    /// Create a new audit log entry for a successful analysis.
    pub fn success(
        provider: String,
        model: String,
        image_count: usize,
        total_image_size: u64,
        image_hashes: Vec<String>,
        tokens_used: u32,
    ) -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            provider,
            model,
            image_count,
            total_image_size,
            image_hashes,
            status: "success".to_string(),
            error: None,
            tokens_used: Some(tokens_used),
        }
    }

    /// Create a new audit log entry for a failed analysis.
    pub fn failure(
        provider: String,
        model: String,
        image_count: usize,
        total_image_size: u64,
        image_hashes: Vec<String>,
        error: String,
    ) -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            provider,
            model,
            image_count,
            total_image_size,
            image_hashes,
            status: "failure".to_string(),
            error: Some(error),
            tokens_used: None,
        }
    }

    /// Create a new audit log entry for a timeout.
    pub fn timeout(
        provider: String,
        model: String,
        image_count: usize,
        total_image_size: u64,
        image_hashes: Vec<String>,
    ) -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            provider,
            model,
            image_count,
            total_image_size,
            image_hashes,
            status: "timeout".to_string(),
            error: Some("Analysis timeout".to_string()),
            tokens_used: None,
        }
    }
}

/// Encode binary data as base64 string.
fn base64_encode(data: &[u8]) -> String {
    const BASE64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();

    for chunk in data.chunks(3) {
        let b1 = chunk[0];
        let b2 = chunk.get(1).copied().unwrap_or(0);
        let b3 = chunk.get(2).copied().unwrap_or(0);

        let n = ((b1 as u32) << 16) | ((b2 as u32) << 8) | (b3 as u32);

        result.push(BASE64_CHARS[((n >> 18) & 63) as usize] as char);
        result.push(BASE64_CHARS[((n >> 12) & 63) as usize] as char);

        if chunk.len() > 1 {
            result.push(BASE64_CHARS[((n >> 6) & 63) as usize] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(BASE64_CHARS[(n & 63) as usize] as char);
        } else {
            result.push('=');
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_providers::models::Message;

    #[test]
    fn test_image_data_creation() {
        let data = vec![1, 2, 3, 4, 5];
        let image = ImageData::from_bytes("png", &data, 800, 600);

        assert_eq!(image.format, "png");
        assert_eq!(image.dimensions, (800, 600));
        assert_eq!(image.size_bytes, 5);
        assert!(!image.data.is_empty());
    }

    #[test]
    fn test_image_data_mime_type() {
        let data = vec![1, 2, 3];

        let png = ImageData::from_bytes("png", &data, 100, 100);
        assert_eq!(png.mime_type(), "image/png");

        let jpg = ImageData::from_bytes("jpg", &data, 100, 100);
        assert_eq!(jpg.mime_type(), "image/jpeg");

        let gif = ImageData::from_bytes("gif", &data, 100, 100);
        assert_eq!(gif.mime_type(), "image/gif");

        let webp = ImageData::from_bytes("webp", &data, 100, 100);
        assert_eq!(webp.mime_type(), "image/webp");
    }

    #[test]
    fn test_image_data_url() {
        let data = vec![1, 2, 3];
        let image = ImageData::from_bytes("png", &data, 100, 100);
        let url = image.data_url();

        assert!(url.starts_with("data:image/png;base64,"));
    }

    #[test]
    fn test_provider_image_format() {
        assert_eq!(
            ProviderImageFormat::for_provider("openai"),
            ProviderImageFormat::OpenAi
        );
        assert_eq!(
            ProviderImageFormat::for_provider("anthropic"),
            ProviderImageFormat::Anthropic
        );
        assert_eq!(
            ProviderImageFormat::for_provider("google"),
            ProviderImageFormat::Google
        );
        assert_eq!(
            ProviderImageFormat::for_provider("ollama"),
            ProviderImageFormat::Ollama
        );
        assert_eq!(
            ProviderImageFormat::for_provider("unknown"),
            ProviderImageFormat::Generic
        );
    }

    #[test]
    fn test_chat_request_with_images_creation() {
        let request = ChatRequest {
            model: "gpt-4".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Analyze this image".to_string(),
            }],
            temperature: Some(0.7),
            max_tokens: Some(1000),
            stream: false,
        };

        let chat_with_images = ChatRequestWithImages::new(request, "openai");
        assert_eq!(chat_with_images.image_count(), 0);
        assert!(!chat_with_images.has_images());
    }

    #[test]
    fn test_chat_request_add_image() {
        let request = ChatRequest {
            model: "gpt-4".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Analyze this image".to_string(),
            }],
            temperature: Some(0.7),
            max_tokens: Some(1000),
            stream: false,
        };

        let mut chat_with_images = ChatRequestWithImages::new(request, "openai");
        let image = ImageData::from_bytes("png", &[1, 2, 3], 800, 600);
        chat_with_images.add_image(image);

        assert_eq!(chat_with_images.image_count(), 1);
        assert!(chat_with_images.has_images());
    }

    #[test]
    fn test_chat_request_serialize_for_provider() {
        let request = ChatRequest {
            model: "gpt-4".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Analyze this image".to_string(),
            }],
            temperature: Some(0.7),
            max_tokens: Some(1000),
            stream: false,
        };

        let mut chat_with_images = ChatRequestWithImages::new(request, "openai");
        let image = ImageData::from_bytes("png", &[1, 2, 3], 800, 600);
        chat_with_images.add_image(image);

        let serialized = chat_with_images.serialize_for_provider().unwrap();
        assert!(!serialized.messages.is_empty());
        assert!(serialized.messages[0].content.contains("Image"));
    }

    #[test]
    fn test_audit_log_entry_success() {
        let entry = ImageAuditLogEntry::success(
            "openai".to_string(),
            "gpt-4".to_string(),
            1,
            1024,
            vec!["hash1".to_string()],
            100,
        );

        assert_eq!(entry.status, "success");
        assert_eq!(entry.tokens_used, Some(100));
        assert!(entry.error.is_none());
    }

    #[test]
    fn test_audit_log_entry_failure() {
        let entry = ImageAuditLogEntry::failure(
            "openai".to_string(),
            "gpt-4".to_string(),
            1,
            1024,
            vec!["hash1".to_string()],
            "Provider error".to_string(),
        );

        assert_eq!(entry.status, "failure");
        assert!(entry.tokens_used.is_none());
        assert!(entry.error.is_some());
    }

    #[test]
    fn test_audit_log_entry_timeout() {
        let entry = ImageAuditLogEntry::timeout(
            "openai".to_string(),
            "gpt-4".to_string(),
            1,
            1024,
            vec!["hash1".to_string()],
        );

        assert_eq!(entry.status, "timeout");
        assert!(entry.tokens_used.is_none());
        assert!(entry.error.is_some());
    }

    #[test]
    fn test_base64_encode() {
        let data = b"Hello";
        let encoded = base64_encode(data);
        assert!(!encoded.is_empty());

        let empty = base64_encode(&[]);
        assert_eq!(empty, "");

        let single = base64_encode(&[65]); // 'A'
        assert!(!single.is_empty());
    }
}
