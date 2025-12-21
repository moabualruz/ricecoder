use ricecoder_providers::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_security_headers_builder_default() {
        let builder = SecurityHeadersBuilder::new();
        let headers = builder.build();

        assert!(headers.contains_key("X-Frame-Options"));
        assert!(headers.contains_key("X-Content-Type-Options"));
        assert!(headers.contains_key("Referrer-Policy"));
        assert!(headers.contains_key("Strict-Transport-Security"));
    }

    #[test]
    fn test_security_headers_builder_add_header() {
        let mut builder = SecurityHeadersBuilder::new();
        builder.add_header("Custom-Header", "custom-value");

        let headers = builder.build();
        assert_eq!(
            headers.get("Custom-Header"),
            Some(&"custom-value".to_string())
        );
    }

    #[test]
    fn test_security_headers_builder_remove_header() {
        let mut builder = SecurityHeadersBuilder::new();
        builder.remove_header("X-Frame-Options");

        let headers = builder.build();
        assert!(!headers.contains_key("X-Frame-Options"));
    }

    #[test]
    fn test_security_headers_builder_get_header() {
        let builder = SecurityHeadersBuilder::new();
        assert_eq!(builder.get_header("X-Frame-Options"), Some("DENY"));
    }

    #[test]
    fn test_security_headers_builder_has_header() {
        let builder = SecurityHeadersBuilder::new();
        assert!(builder.has_header("X-Frame-Options"));
        assert!(!builder.has_header("Non-Existent-Header"));
    }

    #[test]
    fn test_security_headers_validator_validate() {
        let mut headers = HashMap::new();
        headers.insert("X-Frame-Options".to_string(), "DENY".to_string());
        headers.insert("X-Content-Type-Options".to_string(), "nosniff".to_string());
        headers.insert(
            "Referrer-Policy".to_string(),
            "strict-origin-when-cross-origin".to_string(),
        );
        headers.insert(
            "Strict-Transport-Security".to_string(),
            "max-age=31536000".to_string(),
        );

        let result = SecurityHeadersValidator::validate(&headers);
        assert!(result.is_ok());
    }

    #[test]
    fn test_security_headers_validator_missing_headers() {
        let headers = HashMap::new();
        let result = SecurityHeadersValidator::validate(&headers);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_security_headers_validator_is_secure_header() {
        assert!(SecurityHeadersValidator::is_secure_header(
            "X-Frame-Options",
            "DENY"
        ));
        assert!(SecurityHeadersValidator::is_secure_header(
            "X-Frame-Options",
            "SAMEORIGIN"
        ));
        assert!(!SecurityHeadersValidator::is_secure_header(
            "X-Frame-Options",
            "ALLOW-FROM"
        ));

        assert!(SecurityHeadersValidator::is_secure_header(
            "X-Content-Type-Options",
            "nosniff"
        ));
        assert!(!SecurityHeadersValidator::is_secure_header(
            "X-Content-Type-Options",
            "sniff"
        ));

        assert!(SecurityHeadersValidator::is_secure_header(
            "Referrer-Policy",
            "no-referrer"
        ));
        assert!(SecurityHeadersValidator::is_secure_header(
            "Referrer-Policy",
            "strict-origin-when-cross-origin"
        ));
    }

    #[test]
    fn test_security_headers_builder_default_values() {
        let builder = SecurityHeadersBuilder::new();
        assert_eq!(builder.get_header("X-Frame-Options"), Some("DENY"));
        assert_eq!(
            builder.get_header("X-Content-Type-Options"),
            Some("nosniff")
        );
        assert!(builder
            .get_header("Strict-Transport-Security")
            .unwrap()
            .contains("31536000"));
    }
}
