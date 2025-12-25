//! File type detection and MIME type utilities
//!
//! This module provides utilities for:
//! - Binary file detection (by extension and content)
//! - MIME type detection and mapping
//! - File extension categorization
//!
//! Used by MCP servers, TUI, and CLI for safe file handling.

use std::path::Path;

// ============================================================================
// Binary Detection
// ============================================================================

/// Known binary file extensions that should not be read as text.
const BINARY_EXTENSIONS: &[&str] = &[
    // Executables and libraries
    "exe", "dll", "so", "dylib", "bin", "o", "a", "lib",
    // Archives
    "zip", "tar", "gz", "bz2", "xz", "7z", "rar", "zst",
    // Images
    "jpg", "jpeg", "png", "gif", "bmp", "ico", "webp", "tiff", "svg",
    // Media
    "mp3", "mp4", "avi", "mov", "mkv", "flv", "wav", "ogg", "flac",
    // Documents
    "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx",
    // Other binary
    "wasm", "pyc", "class", "jar",
];

/// Text file extensions commonly encountered.
const TEXT_EXTENSIONS: &[&str] = &[
    // Code
    "rs", "py", "js", "ts", "tsx", "jsx", "go", "java", "c", "cpp", "h", "hpp",
    "rb", "php", "swift", "kt", "scala", "cs", "fs", "clj", "lua", "pl", "sh",
    "bash", "zsh", "fish", "ps1", "bat", "cmd",
    // Markup and data
    "md", "txt", "json", "yaml", "yml", "toml", "xml", "html", "htm", "css",
    "scss", "sass", "less", "csv", "ini", "cfg", "conf",
    // Documentation
    "rst", "adoc", "tex", "org",
];

/// Check if a file is binary based on extension.
///
/// Fast check that doesn't require reading file content.
pub fn is_binary_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| BINARY_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

/// Check if a file has a known text extension.
pub fn is_text_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| TEXT_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

/// Check if content is binary by looking for null bytes.
///
/// Checks the first 8KB for null bytes, which are common in binary files.
pub fn is_binary_content(content: &[u8]) -> bool {
    let check_len = content.len().min(8192);
    content[..check_len].contains(&0)
}

/// Combined check: is file binary by extension OR content?
pub fn is_binary_file(path: &Path, content: &[u8]) -> bool {
    is_binary_extension(path) || is_binary_content(content)
}

// ============================================================================
// MIME Type Detection
// ============================================================================

/// Get MIME type based on file extension.
///
/// Returns `None` for unknown extensions.
pub fn mime_from_extension(path: &Path) -> Option<String> {
    let ext = path.extension()?.to_str()?.to_lowercase();
    
    Some(match ext.as_str() {
        // Text
        "txt" => "text/plain",
        "md" | "markdown" => "text/markdown",
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "csv" => "text/csv",
        "xml" => "application/xml",
        
        // Code
        "rs" => "text/x-rust",
        "py" => "text/x-python",
        "js" => "application/javascript",
        "ts" => "application/typescript",
        "tsx" => "application/typescript-jsx",
        "jsx" => "application/javascript-jsx",
        "go" => "text/x-go",
        "java" => "text/x-java",
        "c" => "text/x-c",
        "cpp" | "cc" | "cxx" => "text/x-c++",
        "h" | "hpp" => "text/x-c-header",
        "rb" => "text/x-ruby",
        "php" => "application/x-php",
        "swift" => "text/x-swift",
        "kt" | "kts" => "text/x-kotlin",
        "scala" => "text/x-scala",
        "cs" => "text/x-csharp",
        "sh" | "bash" => "application/x-sh",
        "ps1" => "application/x-powershell",
        
        // Data
        "json" => "application/json",
        "yaml" | "yml" => "application/x-yaml",
        "toml" => "application/toml",
        
        // Images
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "bmp" => "image/bmp",
        
        // Documents
        "pdf" => "application/pdf",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xls" => "application/vnd.ms-excel",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        
        // Archives
        "zip" => "application/zip",
        "tar" => "application/x-tar",
        "gz" | "gzip" => "application/gzip",
        "7z" => "application/x-7z-compressed",
        "rar" => "application/vnd.rar",
        
        // Media
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "mp4" => "video/mp4",
        "avi" => "video/x-msvideo",
        "mkv" => "video/x-matroska",
        
        // Binary
        "exe" => "application/x-msdownload",
        "dll" => "application/x-msdownload",
        "wasm" => "application/wasm",
        
        _ => return None,
    }.to_string())
}

/// Get file category based on MIME type or extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileCategory {
    /// Source code files
    Code,
    /// Markup and data files (JSON, YAML, XML, etc.)
    Data,
    /// Documentation (Markdown, RST, etc.)
    Documentation,
    /// Images
    Image,
    /// Audio/Video
    Media,
    /// Archives (zip, tar, etc.)
    Archive,
    /// Executables and libraries
    Executable,
    /// Documents (PDF, Office, etc.)
    Document,
    /// Plain text
    Text,
    /// Unknown or binary
    Unknown,
}

/// Categorize a file based on extension.
pub fn categorize_file(path: &Path) -> FileCategory {
    let ext = match path.extension().and_then(|e| e.to_str()) {
        Some(e) => e.to_lowercase(),
        None => return FileCategory::Unknown,
    };
    
    match ext.as_str() {
        // Code
        "rs" | "py" | "js" | "ts" | "tsx" | "jsx" | "go" | "java" | "c" | "cpp" |
        "h" | "hpp" | "rb" | "php" | "swift" | "kt" | "scala" | "cs" | "fs" |
        "clj" | "lua" | "pl" | "sh" | "bash" | "ps1" | "bat" => FileCategory::Code,
        
        // Data
        "json" | "yaml" | "yml" | "toml" | "xml" | "csv" | "ini" | "cfg" => FileCategory::Data,
        
        // Documentation
        "md" | "markdown" | "rst" | "adoc" | "tex" | "org" => FileCategory::Documentation,
        
        // Images
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "ico" | "bmp" | "tiff" => FileCategory::Image,
        
        // Media
        "mp3" | "mp4" | "avi" | "mov" | "mkv" | "wav" | "ogg" | "flac" => FileCategory::Media,
        
        // Archives
        "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" => FileCategory::Archive,
        
        // Executables
        "exe" | "dll" | "so" | "dylib" | "bin" | "wasm" => FileCategory::Executable,
        
        // Documents
        "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" => FileCategory::Document,
        
        // Text
        "txt" | "html" | "htm" | "css" | "scss" | "sass" | "less" => FileCategory::Text,
        
        _ => FileCategory::Unknown,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_is_binary_extension() {
        assert!(is_binary_extension(Path::new("test.exe")));
        assert!(is_binary_extension(Path::new("test.png")));
        assert!(is_binary_extension(Path::new("test.PDF"))); // case insensitive
        assert!(!is_binary_extension(Path::new("test.rs")));
        assert!(!is_binary_extension(Path::new("test.txt")));
        assert!(!is_binary_extension(Path::new("no_extension")));
    }

    #[test]
    fn test_is_text_extension() {
        assert!(is_text_extension(Path::new("test.rs")));
        assert!(is_text_extension(Path::new("test.py")));
        assert!(is_text_extension(Path::new("test.MD"))); // case insensitive
        assert!(!is_text_extension(Path::new("test.exe")));
        assert!(!is_text_extension(Path::new("test.png")));
    }

    #[test]
    fn test_is_binary_content() {
        assert!(!is_binary_content(b"Hello, World!"));
        assert!(!is_binary_content(b"fn main() {\n    println!(\"hi\");\n}"));
        assert!(is_binary_content(b"Hello\x00World")); // null byte
        assert!(is_binary_content(&[0u8; 100])); // all nulls
    }

    #[test]
    fn test_is_binary_file() {
        let path = PathBuf::from("test.exe");
        assert!(is_binary_file(&path, b"anything")); // extension check

        let path = PathBuf::from("test.txt");
        assert!(!is_binary_file(&path, b"Hello, World!")); // both pass
        assert!(is_binary_file(&path, b"Hello\x00World")); // content check
    }

    #[test]
    fn test_mime_from_extension() {
        assert_eq!(mime_from_extension(Path::new("test.rs")), Some("text/x-rust".to_string()));
        assert_eq!(mime_from_extension(Path::new("test.json")), Some("application/json".to_string()));
        assert_eq!(mime_from_extension(Path::new("test.png")), Some("image/png".to_string()));
        assert_eq!(mime_from_extension(Path::new("test.unknown")), None);
    }

    #[test]
    fn test_categorize_file() {
        assert_eq!(categorize_file(Path::new("main.rs")), FileCategory::Code);
        assert_eq!(categorize_file(Path::new("config.json")), FileCategory::Data);
        assert_eq!(categorize_file(Path::new("README.md")), FileCategory::Documentation);
        assert_eq!(categorize_file(Path::new("logo.png")), FileCategory::Image);
        assert_eq!(categorize_file(Path::new("app.exe")), FileCategory::Executable);
        assert_eq!(categorize_file(Path::new("unknown.xyz")), FileCategory::Unknown);
    }
}
