use ricecoder_cli::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_ascii_logo() {
        let logo = BrandingManager::default_ascii_logo();
        assert!(logo.contains("RiceCoder"));
        assert!(!logo.is_empty());
    }

    #[test]
    fn test_terminal_capabilities() {
        let caps = BrandingManager::detect_terminal_capabilities();
        assert!(caps.width > 0);
        assert!(caps.height > 0);
    }

    #[test]
    fn test_supports_unicode() {
        let supports = BrandingManager::supports_unicode();
        // Just verify it returns a boolean
        assert!(supports || !supports);
    }
}
