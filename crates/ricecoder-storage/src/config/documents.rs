//! Document format support for Governance and specs
//!
//! This module provides loading and saving of documents in YAML and Markdown formats.

use std::path::Path;

use crate::{
    error::{StorageError, StorageResult},
    types::DocumentFormat,
};

/// Document content
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    /// Document content
    pub content: String,
    /// Document format
    pub format: DocumentFormat,
}

/// Document loader for YAML and Markdown formats
pub struct DocumentLoader;

impl DocumentLoader {
    /// Load a document from a file
    ///
    /// Automatically detects format based on file extension.
    /// Supports YAML (.yaml, .yml) and Markdown (.md, .markdown) formats.
    pub fn load_from_file(path: &Path) -> StorageResult<Document> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            StorageError::io_error(path.to_path_buf(), crate::error::IoOperation::Read, e)
        })?;

        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| {
                StorageError::parse_error(path.to_path_buf(), "unknown", "File has no extension")
            })?;

        let format = DocumentFormat::from_extension(extension).ok_or_else(|| {
            StorageError::parse_error(
                path.to_path_buf(),
                "unknown",
                format!("Unsupported document format: {}", extension),
            )
        })?;

        Ok(Document { content, format })
    }

    /// Load a document from a string with specified format
    pub fn load_from_string(content: String, format: DocumentFormat) -> Document {
        Document { content, format }
    }

    /// Save a document to a file
    pub fn save_to_file(document: &Document, path: &Path) -> StorageResult<()> {
        std::fs::write(path, &document.content).map_err(|e| {
            StorageError::io_error(path.to_path_buf(), crate::error::IoOperation::Write, e)
        })
    }

    /// Get the file extension for a document format
    pub fn extension_for_format(format: DocumentFormat) -> &'static str {
        format.extension()
    }

    /// Detect format from file extension
    pub fn detect_format<P: AsRef<Path>>(path: P) -> StorageResult<DocumentFormat> {
        let path = path.as_ref();
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| {
                StorageError::parse_error(path.to_path_buf(), "unknown", "File has no extension")
            })?;

        DocumentFormat::from_extension(extension).ok_or_else(|| {
            StorageError::parse_error(
                path.to_path_buf(),
                "unknown",
                format!("Unsupported document format: {}", extension),
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_yaml_document() {
        let yaml_content = "# Governance Document\nkey: value\n";
        let doc = DocumentLoader::load_from_string(yaml_content.to_string(), DocumentFormat::Yaml);
        assert_eq!(doc.content, yaml_content);
        assert_eq!(doc.format, DocumentFormat::Yaml);
    }

    #[test]
    fn test_load_markdown_document() {
        let md_content = "# Governance Document\n\nThis is a markdown document.\n";
        let doc =
            DocumentLoader::load_from_string(md_content.to_string(), DocumentFormat::Markdown);
        assert_eq!(doc.content, md_content);
        assert_eq!(doc.format, DocumentFormat::Markdown);
    }

    #[test]
    fn test_save_and_load_yaml_document() {
        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("Governance.yaml");
        let original = Document {
            content: "# Governance\nkey: value\n".to_string(),
            format: DocumentFormat::Yaml,
        };

        DocumentLoader::save_to_file(&original, &file_path).expect("Failed to save document");

        let loaded = DocumentLoader::load_from_file(&file_path).expect("Failed to load document");

        assert_eq!(original, loaded);
    }

    #[test]
    fn test_save_and_load_markdown_document() {
        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("Governance.md");
        let original = Document {
            content: "# Governance\n\nThis is markdown.\n".to_string(),
            format: DocumentFormat::Markdown,
        };

        DocumentLoader::save_to_file(&original, &file_path).expect("Failed to save document");

        let loaded = DocumentLoader::load_from_file(&file_path).expect("Failed to load document");

        assert_eq!(original, loaded);
    }

    #[test]
    fn test_detect_yaml_format() {
        let format = DocumentLoader::detect_format("test.yaml").expect("Failed to detect format");
        assert_eq!(format, DocumentFormat::Yaml);

        let format = DocumentLoader::detect_format("test.yml").expect("Failed to detect format");
        assert_eq!(format, DocumentFormat::Yaml);
    }

    #[test]
    fn test_detect_markdown_format() {
        let format = DocumentLoader::detect_format("test.md").expect("Failed to detect format");
        assert_eq!(format, DocumentFormat::Markdown);

        let format =
            DocumentLoader::detect_format("test.markdown").expect("Failed to detect format");
        assert_eq!(format, DocumentFormat::Markdown);
    }

    #[test]
    fn test_extension_for_format() {
        assert_eq!(
            DocumentLoader::extension_for_format(DocumentFormat::Yaml),
            "yaml"
        );
        assert_eq!(
            DocumentLoader::extension_for_format(DocumentFormat::Markdown),
            "md"
        );
    }
}
