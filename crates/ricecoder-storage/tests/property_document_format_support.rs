//! Property-based tests for document format support
//!
//! **Feature: ricecoder-storage, Property 4: Document Format Support**
//! **Validates: Requirements 2.3, 2.4**

use proptest::prelude::*;
use ricecoder_storage::config::{Document, DocumentLoader};
use ricecoder_storage::types::DocumentFormat;
use tempfile::TempDir;

/// Strategy for generating valid document content
fn document_content_strategy() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9\n\r\t !@#$%^&*()_+=\-\[\]{};:',.<>?/\\|`~]*".prop_map(|s| s.to_string())
}

proptest! {
    /// Property: YAML documents can be stored and retrieved
    ///
    /// For any valid document content, storing a YAML document and retrieving it
    /// should produce an equivalent document.
    #[test]
    fn prop_yaml_document_roundtrip(content in document_content_strategy()) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("document.yaml");

        let original = Document {
            content: content.clone(),
            format: DocumentFormat::Yaml,
        };

        DocumentLoader::save_to_file(&original, &file_path)
            .expect("Failed to save document");

        let loaded = DocumentLoader::load_from_file(&file_path)
            .expect("Failed to load document");

        prop_assert_eq!(original, loaded);
    }

    /// Property: Markdown documents can be stored and retrieved
    ///
    /// For any valid document content, storing a Markdown document and retrieving it
    /// should produce an equivalent document.
    #[test]
    fn prop_markdown_document_roundtrip(content in document_content_strategy()) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("document.md");

        let original = Document {
            content: content.clone(),
            format: DocumentFormat::Markdown,
        };

        DocumentLoader::save_to_file(&original, &file_path)
            .expect("Failed to save document");

        let loaded = DocumentLoader::load_from_file(&file_path)
            .expect("Failed to load document");

        prop_assert_eq!(original, loaded);
    }

    /// Property: Format detection works for YAML files
    ///
    /// For any YAML file, the format should be correctly detected as YAML.
    #[test]
    fn prop_yaml_format_detection(content in document_content_strategy()) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("document.yaml");

        let doc = Document {
            content: content.clone(),
            format: DocumentFormat::Yaml,
        };

        DocumentLoader::save_to_file(&doc, &file_path)
            .expect("Failed to save document");

        let detected_format = DocumentLoader::detect_format(&file_path)
            .expect("Failed to detect format");

        prop_assert_eq!(detected_format, DocumentFormat::Yaml);
    }

    /// Property: Format detection works for Markdown files
    ///
    /// For any Markdown file, the format should be correctly detected as Markdown.
    #[test]
    fn prop_markdown_format_detection(content in document_content_strategy()) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("document.md");

        let doc = Document {
            content: content.clone(),
            format: DocumentFormat::Markdown,
        };

        DocumentLoader::save_to_file(&doc, &file_path)
            .expect("Failed to save document");

        let detected_format = DocumentLoader::detect_format(&file_path)
            .expect("Failed to detect format");

        prop_assert_eq!(detected_format, DocumentFormat::Markdown);
    }

    /// Property: Document content is preserved exactly
    ///
    /// For any document, the content should be preserved exactly without modification.
    #[test]
    fn prop_document_content_preservation(content in document_content_strategy()) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("document.yaml");

        let original = Document {
            content: content.clone(),
            format: DocumentFormat::Yaml,
        };

        DocumentLoader::save_to_file(&original, &file_path)
            .expect("Failed to save document");

        let loaded = DocumentLoader::load_from_file(&file_path)
            .expect("Failed to load document");

        prop_assert_eq!(loaded.content, content);
    }

    /// Property: Both YAML and Markdown formats preserve content
    ///
    /// For any document content, both YAML and Markdown formats should preserve
    /// the content exactly.
    #[test]
    fn prop_both_formats_preserve_content(content in document_content_strategy()) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let yaml_path = temp_dir.path().join("document.yaml");
        let md_path = temp_dir.path().join("document.md");

        let yaml_doc = Document {
            content: content.clone(),
            format: DocumentFormat::Yaml,
        };

        let md_doc = Document {
            content: content.clone(),
            format: DocumentFormat::Markdown,
        };

        DocumentLoader::save_to_file(&yaml_doc, &yaml_path)
            .expect("Failed to save YAML document");
        DocumentLoader::save_to_file(&md_doc, &md_path)
            .expect("Failed to save Markdown document");

        let loaded_yaml = DocumentLoader::load_from_file(&yaml_path)
            .expect("Failed to load YAML document");
        let loaded_md = DocumentLoader::load_from_file(&md_path)
            .expect("Failed to load Markdown document");

        prop_assert_eq!(loaded_yaml.content, content.clone());
        prop_assert_eq!(loaded_md.content, content);
    }
}
