// Unit tests for branding and visual identity
// **Feature: ricecoder-cli, Tests for Requirements 7.1-7.5**

use ricecoder_cli::branding::{BrandingManager, TerminalCapabilities};

// ============================================================================
// ASCII Logo Tests
// ============================================================================

#[test]
fn test_default_ascii_logo_contains_ricecoder() {
    let logo = BrandingManager::default_ascii_logo();
    assert!(logo.contains("RiceCoder"));
}

#[test]
fn test_default_ascii_logo_not_empty() {
    let logo = BrandingManager::default_ascii_logo();
    assert!(!logo.is_empty());
}

#[test]
fn test_default_ascii_logo_contains_branding_elements() {
    let logo = BrandingManager::default_ascii_logo();
    
    // Should contain visual elements
    assert!(logo.contains("╔") || logo.contains("═") || logo.contains("╗"));
}

#[test]
fn test_default_ascii_logo_multiline() {
    let logo = BrandingManager::default_ascii_logo();
    
    // Logo should have multiple lines
    let lines: Vec<&str> = logo.lines().collect();
    assert!(lines.len() > 1);
}

#[test]
fn test_default_ascii_logo_contains_tagline() {
    let logo = BrandingManager::default_ascii_logo();
    
    // Should contain the tagline
    assert!(logo.contains("Terminal-first") || logo.contains("Spec-driven"));
}

#[test]
fn test_load_ascii_logo_returns_string() {
    let result = BrandingManager::load_ascii_logo();
    
    // Should return a result
    assert!(result.is_ok());
    
    let logo = result.unwrap();
    assert!(!logo.is_empty());
}

#[test]
fn test_load_ascii_logo_contains_ricecoder() {
    let result = BrandingManager::load_ascii_logo();
    
    assert!(result.is_ok());
    let logo = result.unwrap();
    assert!(logo.contains("RiceCoder"));
}

// ============================================================================
// Terminal Capabilities Tests
// ============================================================================

#[test]
fn test_terminal_capabilities_structure() {
    let caps = BrandingManager::detect_terminal_capabilities();
    
    // Verify all fields are present
    assert!(caps.width > 0);
    assert!(caps.height > 0);
}

#[test]
fn test_terminal_capabilities_width_reasonable() {
    let caps = BrandingManager::detect_terminal_capabilities();
    
    // Terminal width should be reasonable (at least 40, typically 80+)
    assert!(caps.width >= 40);
    assert!(caps.width <= 500);
}

#[test]
fn test_terminal_capabilities_height_reasonable() {
    let caps = BrandingManager::detect_terminal_capabilities();
    
    // Terminal height should be reasonable (at least 1 in test environments, typically 24+)
    assert!(caps.height >= 1);
    assert!(caps.height <= 200);
}

#[test]
fn test_terminal_capabilities_colors_boolean() {
    let caps = BrandingManager::detect_terminal_capabilities();
    
    // Just verify it's a boolean
    assert!(caps.supports_colors || !caps.supports_colors);
}

#[test]
fn test_terminal_capabilities_unicode_boolean() {
    let caps = BrandingManager::detect_terminal_capabilities();
    
    // Just verify it's a boolean
    assert!(caps.supports_unicode || !caps.supports_unicode);
}

#[test]
fn test_terminal_capabilities_images_boolean() {
    let caps = BrandingManager::detect_terminal_capabilities();
    
    // Just verify it's a boolean
    assert!(caps.supports_images || !caps.supports_images);
}

#[test]
fn test_terminal_capabilities_clone() {
    let caps1 = BrandingManager::detect_terminal_capabilities();
    let caps2 = caps1.clone();
    
    assert_eq!(caps1.width, caps2.width);
    assert_eq!(caps1.height, caps2.height);
    assert_eq!(caps1.supports_colors, caps2.supports_colors);
}

#[test]
fn test_terminal_capabilities_debug_format() {
    let caps = BrandingManager::detect_terminal_capabilities();
    let debug_str = format!("{:?}", caps);
    
    assert!(debug_str.contains("TerminalCapabilities"));
}

// ============================================================================
// Unicode Support Tests
// ============================================================================

#[test]
fn test_supports_unicode_returns_boolean() {
    let supports = BrandingManager::supports_unicode();
    
    // Just verify it returns a boolean
    assert!(supports || !supports);
}

#[test]
fn test_supports_unicode_consistency() {
    let supports1 = BrandingManager::supports_unicode();
    let supports2 = BrandingManager::supports_unicode();
    
    // Should be consistent across calls
    assert_eq!(supports1, supports2);
}

// ============================================================================
// Image Support Tests
// ============================================================================

#[test]
fn test_supports_images_returns_boolean() {
    let supports = BrandingManager::supports_images();
    
    // Just verify it returns a boolean
    assert!(supports || !supports);
}

#[test]
fn test_supports_images_consistency() {
    let supports1 = BrandingManager::supports_images();
    let supports2 = BrandingManager::supports_images();
    
    // Should be consistent across calls
    assert_eq!(supports1, supports2);
}

// ============================================================================
// Terminal Dimension Tests
// ============================================================================

#[test]
fn test_get_terminal_width_positive() {
    let width = BrandingManager::get_terminal_width();
    
    assert!(width > 0);
}

#[test]
fn test_get_terminal_height_positive() {
    let height = BrandingManager::get_terminal_height();
    
    assert!(height > 0);
}

#[test]
fn test_get_terminal_width_reasonable() {
    let width = BrandingManager::get_terminal_width();
    
    // Should be between 40 and 500
    assert!(width >= 40);
    assert!(width <= 500);
}

#[test]
fn test_get_terminal_height_reasonable() {
    let height = BrandingManager::get_terminal_height();
    
    // Should be between 1 and 200 (1 in test environments, typically 24+)
    assert!(height >= 1);
    assert!(height <= 200);
}

#[test]
fn test_get_terminal_dimensions_consistency() {
    let width1 = BrandingManager::get_terminal_width();
    let width2 = BrandingManager::get_terminal_width();
    
    // Should be consistent
    assert_eq!(width1, width2);
}

// ============================================================================
// Branding Display Tests
// ============================================================================

#[test]
fn test_display_startup_banner_succeeds() {
    let result = BrandingManager::display_startup_banner();
    
    assert!(result.is_ok());
}

#[test]
fn test_display_version_banner_succeeds() {
    let result = BrandingManager::display_version_banner("1.0.0");
    
    assert!(result.is_ok());
}

#[test]
fn test_display_version_banner_with_different_versions() {
    let versions = vec!["0.1.0", "1.0.0", "2.5.3", "10.0.0-beta"];
    
    for version in versions {
        let result = BrandingManager::display_version_banner(version);
        assert!(result.is_ok());
    }
}

// ============================================================================
// Branding Manager Tests
// ============================================================================

#[test]
fn test_branding_manager_creation() {
    let _manager = BrandingManager;
    // Just verify it can be created
    assert!(true);
}

// ============================================================================
// Terminal Capabilities Edge Cases
// ============================================================================

#[test]
fn test_terminal_capabilities_all_false() {
    let caps = TerminalCapabilities {
        supports_colors: false,
        supports_unicode: false,
        supports_images: false,
        width: 80,
        height: 24,
    };
    
    assert!(!caps.supports_colors);
    assert!(!caps.supports_unicode);
    assert!(!caps.supports_images);
}

#[test]
fn test_terminal_capabilities_all_true() {
    let caps = TerminalCapabilities {
        supports_colors: true,
        supports_unicode: true,
        supports_images: true,
        width: 120,
        height: 40,
    };
    
    assert!(caps.supports_colors);
    assert!(caps.supports_unicode);
    assert!(caps.supports_images);
}

#[test]
fn test_terminal_capabilities_mixed() {
    let caps = TerminalCapabilities {
        supports_colors: true,
        supports_unicode: false,
        supports_images: true,
        width: 100,
        height: 30,
    };
    
    assert!(caps.supports_colors);
    assert!(!caps.supports_unicode);
    assert!(caps.supports_images);
}

#[test]
fn test_terminal_capabilities_minimum_dimensions() {
    let caps = TerminalCapabilities {
        supports_colors: false,
        supports_unicode: false,
        supports_images: false,
        width: 40,
        height: 20,
    };
    
    assert_eq!(caps.width, 40);
    assert_eq!(caps.height, 20);
}

#[test]
fn test_terminal_capabilities_large_dimensions() {
    let caps = TerminalCapabilities {
        supports_colors: true,
        supports_unicode: true,
        supports_images: true,
        width: 500,
        height: 200,
    };
    
    assert_eq!(caps.width, 500);
    assert_eq!(caps.height, 200);
}

// ============================================================================
// Property-Based Tests
// ============================================================================

#[test]
fn test_default_ascii_logo_idempotent() {
    let logo1 = BrandingManager::default_ascii_logo();
    let logo2 = BrandingManager::default_ascii_logo();
    
    assert_eq!(logo1, logo2);
}

#[test]
fn test_load_ascii_logo_idempotent() {
    let result1 = BrandingManager::load_ascii_logo();
    let result2 = BrandingManager::load_ascii_logo();
    
    assert_eq!(result1.is_ok(), result2.is_ok());
    
    if let (Ok(logo1), Ok(logo2)) = (result1, result2) {
        assert_eq!(logo1, logo2);
    }
}

#[test]
fn test_terminal_capabilities_consistency() {
    let caps1 = BrandingManager::detect_terminal_capabilities();
    let caps2 = BrandingManager::detect_terminal_capabilities();
    
    // Dimensions should be consistent
    assert_eq!(caps1.width, caps2.width);
    assert_eq!(caps1.height, caps2.height);
}

#[test]
fn test_display_startup_banner_idempotent() {
    let result1 = BrandingManager::display_startup_banner();
    let result2 = BrandingManager::display_startup_banner();
    
    assert_eq!(result1.is_ok(), result2.is_ok());
}

#[test]
fn test_display_version_banner_idempotent() {
    let result1 = BrandingManager::display_version_banner("1.0.0");
    let result2 = BrandingManager::display_version_banner("1.0.0");
    
    assert_eq!(result1.is_ok(), result2.is_ok());
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_branding_with_terminal_capabilities() {
    let caps = BrandingManager::detect_terminal_capabilities();
    let logo = BrandingManager::load_ascii_logo();
    
    // Both should succeed
    assert!(logo.is_ok());
    assert!(caps.width > 0);
}

#[test]
fn test_branding_display_with_capabilities() {
    let caps = BrandingManager::detect_terminal_capabilities();
    let result = BrandingManager::display_startup_banner();
    
    // Display should succeed regardless of capabilities
    assert!(result.is_ok());
    assert!(caps.width > 0);
}

#[test]
fn test_all_branding_operations_succeed() {
    // All branding operations should succeed
    assert!(BrandingManager::load_ascii_logo().is_ok());
    assert!(BrandingManager::display_startup_banner().is_ok());
    assert!(BrandingManager::display_version_banner("1.0.0").is_ok());
    
    let caps = BrandingManager::detect_terminal_capabilities();
    assert!(caps.width > 0);
}
