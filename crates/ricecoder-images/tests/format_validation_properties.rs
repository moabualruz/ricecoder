//! Property-based tests for image format validation.
//!
//! **Feature: ricecoder-images, Property 1: Format Validation**
//! **Validates: Requirements 1.3, 4.1, 4.2, 4.3, 4.4, 4.5**

use proptest::prelude::*;
use ricecoder_images::formats::ImageFormat;
use tempfile::NamedTempFile;

/// Strategy for generating valid PNG file headers
fn png_header_strategy() -> impl Strategy<Value = Vec<u8>> {
    Just(vec![0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a])
}

/// Strategy for generating valid JPEG file headers
fn jpeg_header_strategy() -> impl Strategy<Value = Vec<u8>> {
    Just(vec![0xff, 0xd8, 0xff, 0xe0])
}

/// Strategy for generating valid GIF file headers
fn gif_header_strategy() -> impl Strategy<Value = Vec<u8>> {
    Just(b"GIF89a".to_vec())
}

/// Strategy for generating valid WebP file headers
fn webp_header_strategy() -> impl Strategy<Value = Vec<u8>> {
    let mut header = b"RIFF".to_vec();
    header.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
    header.extend_from_slice(b"WEBP");
    Just(header)
}

/// Strategy for generating random invalid file headers
fn invalid_header_strategy() -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(any::<u8>(), 4..16).prop_filter("Filter out valid headers", |bytes| {
        // Exclude valid headers
        !bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47]) // PNG
                && !bytes.starts_with(&[0xFF, 0xD8, 0xFF]) // JPEG
                && !bytes.starts_with(b"GIF") // GIF
                && !(bytes.len() >= 12
                    && bytes.starts_with(b"RIFF")
                    && bytes[8..12] == *b"WEBP") // WebP
    })
}

/// Property 1: Format Validation - Supported formats are accepted
///
/// For any supported image format (PNG, JPG, GIF, WebP), the system SHALL accept it.
#[test]
fn prop_supported_formats_accepted() {
    proptest!(|(
        header in prop_oneof![
            png_header_strategy(),
            jpeg_header_strategy(),
            gif_header_strategy(),
            webp_header_strategy(),
        ]
    )| {
        let result = ImageFormat::detect_from_bytes(&header);
        prop_assert!(result.is_ok(), "Supported format should be accepted");

        let format = result.unwrap();
        prop_assert!(
            matches!(format, ImageFormat::Png | ImageFormat::Jpeg | ImageFormat::Gif | ImageFormat::WebP),
            "Format should be one of the supported types"
        );
    });
}

/// Property 1: Format Validation - Unsupported formats are rejected
///
/// For any unsupported image format, the system SHALL reject it with a clear error.
#[test]
fn prop_unsupported_formats_rejected() {
    proptest!(|(header in invalid_header_strategy())| {
        let result = ImageFormat::detect_from_bytes(&header);
        prop_assert!(result.is_err(), "Unsupported format should be rejected");

        let error = result.unwrap_err();
        let error_msg = error.to_string();
        prop_assert!(
            error_msg.contains("Unable to detect") || error_msg.contains("Invalid"),
            "Error message should indicate format detection failure"
        );
    });
}

/// Property 1: Format Validation - File size validation
///
/// For any image file that exceeds 10 MB, the system SHALL reject it.
#[test]
fn test_file_size_validation_exceeds_limit() {
    // Create a temporary file with size > 10 MB
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let size_bytes = 11 * 1024 * 1024; // 11 MB

    // Write PNG header followed by zeros to reach the desired size
    let mut content = vec![0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
    content.resize(size_bytes as usize, 0);

    std::io::Write::write_all(&mut temp_file, &content).expect("Failed to write to temp file");

    let path = temp_file.path().to_path_buf();

    // Validate with 10 MB limit
    let result = ImageFormat::validate_file(&path, 10);
    assert!(result.is_err(), "File exceeding 10 MB should be rejected");

    let error = result.unwrap_err();
    let error_msg = error.to_string();
    assert!(
        error_msg.contains("too large") || error_msg.contains("exceeds"),
        "Error message should indicate file size violation"
    );
}

/// Property 1: Format Validation - Small files are accepted
///
/// For any valid image file under 10 MB, the system SHALL accept it.
#[test]
fn prop_small_files_accepted() {
    proptest!(|(
        header in prop_oneof![
            png_header_strategy(),
            jpeg_header_strategy(),
            gif_header_strategy(),
            webp_header_strategy(),
        ],
        extra_bytes in 0usize..1000usize
    )| {
        // Create a temporary file with valid header and extra bytes
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let mut content = header;
        content.resize(content.len() + extra_bytes, 0);

        std::io::Write::write_all(&mut temp_file, &content)
            .expect("Failed to write to temp file");

        let path = temp_file.path().to_path_buf();

        // Validate with 10 MB limit
        let result = ImageFormat::validate_file(&path, 10);
        prop_assert!(result.is_ok(), "Small valid file should be accepted");
    });
}

/// Property 1: Format Validation - Format detection consistency
///
/// For any valid image file, detecting the format multiple times SHALL produce the same result.
#[test]
fn prop_format_detection_consistency() {
    proptest!(|(
        header in prop_oneof![
            png_header_strategy(),
            jpeg_header_strategy(),
            gif_header_strategy(),
            webp_header_strategy(),
        ]
    )| {
        let result1 = ImageFormat::detect_from_bytes(&header);
        let result2 = ImageFormat::detect_from_bytes(&header);

        // Both should succeed
        prop_assert!(result1.is_ok(), "First detection should succeed");
        prop_assert!(result2.is_ok(), "Second detection should succeed");

        // Both should produce the same format
        let format1 = result1.unwrap();
        let format2 = result2.unwrap();
        prop_assert_eq!(format1, format2, "Format detection should be consistent");
    });
}

/// Property 1: Format Validation - Format string representation
///
/// For any detected format, the string representation SHALL be one of the supported formats.
#[test]
fn prop_format_string_representation() {
    proptest!(|(
        header in prop_oneof![
            png_header_strategy(),
            jpeg_header_strategy(),
            gif_header_strategy(),
            webp_header_strategy(),
        ]
    )| {
        let format = ImageFormat::detect_from_bytes(&header).expect("Should detect format");
        let format_str = format.as_str();

        prop_assert!(
            matches!(format_str, "png" | "jpg" | "gif" | "webp"),
            "Format string should be one of the supported formats"
        );
    });
}

/// Property 1: Format Validation - Empty files are rejected
///
/// For any empty or too-small file, the system SHALL reject it.
#[test]
fn prop_empty_files_rejected() {
    proptest!(|(size in 0usize..4usize)| {
        let bytes = vec![0u8; size];
        let result = ImageFormat::detect_from_bytes(&bytes);
        prop_assert!(result.is_err(), "Empty or too-small file should be rejected");
    });
}

/// Property 1: Format Validation - Supported formats list
///
/// The system SHALL provide a list of supported formats that includes PNG, JPG, GIF, and WebP.
#[test]
fn test_supported_formats_list() {
    let config = ricecoder_images::ImageConfig::default();
    let formats = config.supported_formats_string();

    assert!(
        formats.contains("png"),
        "PNG should be in supported formats"
    );
    assert!(
        formats.contains("jpg"),
        "JPG should be in supported formats"
    );
    assert!(
        formats.contains("gif"),
        "GIF should be in supported formats"
    );
    assert!(
        formats.contains("webp"),
        "WebP should be in supported formats"
    );
}

/// Property 1: Format Validation - Format support check
///
/// For any format in the supported list, the system SHALL recognize it as supported.
#[test]
fn prop_format_support_check() {
    proptest!(|(format in r"(png|jpg|jpeg|gif|webp)")| {
        let config = ricecoder_images::ImageConfig::default();
        prop_assert!(
            config.is_format_supported(&format),
            "Format {} should be recognized as supported",
            format
        );
    });
}
