//! Clipboard operations for prompt
//!
//! Handles paste operations for:
//! - Plain text (with summarization for large pastes)
//! - Images (base64 encoded)
//! - Files (including SVG as text)
//!
//! # DDD Layer: Infrastructure
//! Clipboard integration for the prompt system.

use std::path::Path;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

/// Result of a clipboard read
#[derive(Debug, Clone)]
pub enum ClipboardContent {
    /// Plain text content
    Text(String),
    /// Image content (mime type + base64 data)
    Image {
        mime: String,
        data: String,
        filename: Option<String>,
    },
    /// No content available
    Empty,
}

/// Pasted content result for prompt
#[derive(Debug, Clone)]
pub enum PastedContent {
    /// Plain text to insert directly
    PlainText(String),
    /// Summarized text (virtual text + full content)
    SummarizedText {
        virtual_text: String,
        full_text: String,
        line_count: usize,
    },
    /// Image attachment
    Image {
        mime: String,
        data: String,
        filename: Option<String>,
        virtual_text: String,
    },
    /// SVG as text content
    SvgText {
        virtual_text: String,
        content: String,
        filename: String,
    },
}

/// Configuration for paste behavior
#[derive(Debug, Clone)]
pub struct PasteConfig {
    /// Minimum lines to trigger summarization
    pub summarize_min_lines: usize,
    /// Minimum length to trigger summarization
    pub summarize_min_length: usize,
    /// Whether paste summarization is disabled
    pub disable_paste_summary: bool,
}

impl Default for PasteConfig {
    fn default() -> Self {
        Self {
            summarize_min_lines: 3,
            summarize_min_length: 150,
            disable_paste_summary: false,
        }
    }
}

/// Clipboard operations
pub struct Clipboard;

impl Clipboard {
    /// Read from system clipboard (platform-specific)
    #[cfg(feature = "clipboard")]
    pub fn read() -> ClipboardContent {
        use arboard::Clipboard as SystemClipboard;
        
        match SystemClipboard::new() {
            Ok(mut clipboard) => {
                // Try to get image first
                if let Ok(image) = clipboard.get_image() {
                    // Convert to PNG base64
                    if let Ok(png_data) = Self::image_to_png_base64(&image) {
                        return ClipboardContent::Image {
                            mime: "image/png".to_string(),
                            data: png_data,
                            filename: Some("clipboard".to_string()),
                        };
                    }
                }
                
                // Fall back to text
                if let Ok(text) = clipboard.get_text() {
                    return ClipboardContent::Text(text);
                }
                
                ClipboardContent::Empty
            }
            Err(_) => ClipboardContent::Empty,
        }
    }
    
    /// Stub for when clipboard feature is disabled
    #[cfg(not(feature = "clipboard"))]
    pub fn read() -> ClipboardContent {
        ClipboardContent::Empty
    }
    
    /// Convert arboard image to PNG base64
    #[cfg(feature = "clipboard")]
    fn image_to_png_base64(image: &arboard::ImageData) -> Result<String, std::io::Error> {
        use image::{ImageBuffer, Rgba};
        
        let img: ImageBuffer<Rgba<u8>, _> = ImageBuffer::from_raw(
            image.width as u32,
            image.height as u32,
            image.bytes.to_vec(),
        ).ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid image data"))?;
        
        let mut png_bytes = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut png_bytes);
        img.write_to(&mut cursor, image::ImageFormat::Png)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        
        Ok(BASE64.encode(&png_bytes))
    }
    
    /// Process pasted text content
    pub fn process_text(text: &str, config: &PasteConfig, _image_count: usize) -> PastedContent {
        // Normalize line endings
        let normalized = text
            .replace("\r\n", "\n")
            .replace('\r', "\n");
        let trimmed = normalized.trim();
        
        if trimmed.is_empty() {
            return PastedContent::PlainText(String::new());
        }
        
        // Check if it's a file path
        if let Some(content) = Self::try_read_file(trimmed) {
            return content;
        }
        
        // Check if we should summarize
        let line_count = trimmed.matches('\n').count() + 1;
        let should_summarize = !config.disable_paste_summary
            && (line_count >= config.summarize_min_lines || trimmed.len() > config.summarize_min_length);
        
        if should_summarize {
            PastedContent::SummarizedText {
                virtual_text: format!("[Pasted ~{} lines]", line_count),
                full_text: trimmed.to_string(),
                line_count,
            }
        } else {
            PastedContent::PlainText(normalized)
        }
    }
    
    /// Try to read content from a file path
    fn try_read_file(text: &str) -> Option<PastedContent> {
        // Strip quotes and escape sequences
        let filepath = text
            .trim_matches('\'')
            .trim_matches('"')
            .replace("\\ ", " ");
        
        // Skip URLs
        if filepath.starts_with("http://") || filepath.starts_with("https://") {
            return None;
        }
        
        let path = Path::new(&filepath);
        if !path.exists() {
            return None;
        }
        
        // Get file info
        let filename = path.file_name()?.to_string_lossy().to_string();
        let mime = Self::guess_mime_type(path);
        
        // Handle SVG as text
        if mime == "image/svg+xml" {
            if let Ok(content) = std::fs::read_to_string(path) {
                return Some(PastedContent::SvgText {
                    virtual_text: format!("[SVG: {}]", filename),
                    content,
                    filename,
                });
            }
        }
        
        // Handle images
        if mime.starts_with("image/") {
            if let Ok(bytes) = std::fs::read(path) {
                let data = BASE64.encode(&bytes);
                return Some(PastedContent::Image {
                    mime,
                    data,
                    filename: Some(filename),
                    virtual_text: String::new(), // Set by caller based on index
                });
            }
        }
        
        None
    }
    
    /// Guess MIME type from file extension
    fn guess_mime_type(path: &Path) -> String {
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();
        
        match ext.as_str() {
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "webp" => "image/webp",
            "svg" => "image/svg+xml",
            "bmp" => "image/bmp",
            "ico" => "image/x-icon",
            "pdf" => "application/pdf",
            "txt" => "text/plain",
            "json" => "application/json",
            "xml" => "application/xml",
            "html" | "htm" => "text/html",
            "css" => "text/css",
            "js" => "text/javascript",
            "ts" => "text/typescript",
            "rs" => "text/rust",
            "py" => "text/python",
            "md" => "text/markdown",
            _ => "application/octet-stream",
        }.to_string()
    }
    
    /// Create virtual text for an image
    pub fn image_virtual_text(index: usize) -> String {
        format!("[Image {}]", index + 1)
    }
    
    /// Create virtual text for summarized paste
    pub fn paste_virtual_text(line_count: usize) -> String {
        format!("[Pasted ~{} lines]", line_count)
    }
}

/// OSC 52 clipboard operations (terminal clipboard)
pub struct Osc52Clipboard;

impl Osc52Clipboard {
    /// Write to clipboard using OSC 52
    pub fn write(text: &str) -> String {
        let encoded = BASE64.encode(text);
        format!("\x1b]52;c;{}\x07", encoded)
    }
    
    /// Request clipboard content using OSC 52
    pub fn request() -> &'static str {
        "\x1b]52;c;?\x07"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_paste_config_default() {
        let config = PasteConfig::default();
        assert_eq!(config.summarize_min_lines, 3);
        assert_eq!(config.summarize_min_length, 150);
        assert!(!config.disable_paste_summary);
    }
    
    #[test]
    fn test_process_text_plain() {
        let config = PasteConfig::default();
        let result = Clipboard::process_text("hello world", &config, 0);
        
        match result {
            PastedContent::PlainText(text) => assert_eq!(text, "hello world"),
            _ => panic!("Expected PlainText"),
        }
    }
    
    #[test]
    fn test_process_text_summarized() {
        let config = PasteConfig::default();
        let text = "line1\nline2\nline3\nline4";
        let result = Clipboard::process_text(text, &config, 0);
        
        match result {
            PastedContent::SummarizedText { virtual_text, line_count, .. } => {
                assert_eq!(line_count, 4);
                assert!(virtual_text.contains("4"));
            }
            _ => panic!("Expected SummarizedText"),
        }
    }
    
    #[test]
    fn test_process_text_disabled_summary() {
        let config = PasteConfig {
            disable_paste_summary: true,
            ..Default::default()
        };
        let text = "line1\nline2\nline3\nline4";
        let result = Clipboard::process_text(text, &config, 0);
        
        match result {
            PastedContent::PlainText(_) => {}
            _ => panic!("Expected PlainText when summary disabled"),
        }
    }
    
    #[test]
    fn test_guess_mime_type() {
        assert_eq!(Clipboard::guess_mime_type(Path::new("test.png")), "image/png");
        assert_eq!(Clipboard::guess_mime_type(Path::new("test.jpg")), "image/jpeg");
        assert_eq!(Clipboard::guess_mime_type(Path::new("test.svg")), "image/svg+xml");
        assert_eq!(Clipboard::guess_mime_type(Path::new("test.rs")), "text/rust");
    }
    
    #[test]
    fn test_image_virtual_text() {
        assert_eq!(Clipboard::image_virtual_text(0), "[Image 1]");
        assert_eq!(Clipboard::image_virtual_text(2), "[Image 3]");
    }
    
    #[test]
    fn test_osc52_write() {
        let output = Osc52Clipboard::write("hello");
        assert!(output.starts_with("\x1b]52;c;"));
        assert!(output.ends_with("\x07"));
    }
    
    #[test]
    fn test_line_ending_normalization() {
        let config = PasteConfig {
            disable_paste_summary: true,
            ..Default::default()
        };
        
        // Test CRLF
        let result = Clipboard::process_text("line1\r\nline2", &config, 0);
        match result {
            PastedContent::PlainText(text) => assert_eq!(text, "line1\nline2"),
            _ => panic!("Expected PlainText"),
        }
        
        // Test CR only
        let result = Clipboard::process_text("line1\rline2", &config, 0);
        match result {
            PastedContent::PlainText(text) => assert_eq!(text, "line1\nline2"),
            _ => panic!("Expected PlainText"),
        }
    }
}
