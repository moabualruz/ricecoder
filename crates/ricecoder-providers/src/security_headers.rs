//! Security headers for HTTP responses
//!
//! This module provides utilities for adding security headers to HTTP responses
//! to prevent common web vulnerabilities.

use std::collections::HashMap;

/// Security headers builder
pub struct SecurityHeadersBuilder {
    headers: HashMap<String, String>,
}

impl SecurityHeadersBuilder {
    /// Create a new security headers builder with default headers
    pub fn new() -> Self {
        let mut headers = HashMap::new();

        // Prevent clickjacking
        headers.insert(
            "X-Frame-Options".to_string(),
            "DENY".to_string(),
        );

        // Prevent MIME type sniffing
        headers.insert(
            "X-Content-Type-Options".to_string(),
            "nosniff".to_string(),
        );

        // Enable XSS protection (for older browsers)
        headers.insert(
            "X-XSS-Protection".to_string(),
            "1; mode=block".to_string(),
        );

        // Referrer policy
        headers.insert(
            "Referrer-Policy".to_string(),
            "strict-origin-when-cross-origin".to_string(),
        );

        // Permissions policy (formerly Feature-Policy)
        headers.insert(
            "Permissions-Policy".to_string(),
            "geolocation=(), microphone=(), camera=()".to_string(),
        );

        // Strict Transport Security (HSTS)
        headers.insert(
            "Strict-Transport-Security".to_string(),
            "max-age=31536000; includeSubDomains".to_string(),
        );

        // Content Security Policy (CSP)
        headers.insert(
            "Content-Security-Policy".to_string(),
            "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:".to_string(),
        );

        Self { headers }
    }

    /// Add a custom header
    pub fn add_header(&mut self, name: &str, value: &str) -> &mut Self {
        self.headers.insert(name.to_string(), value.to_string());
        self
    }

    /// Remove a header
    pub fn remove_header(&mut self, name: &str) -> &mut Self {
        self.headers.remove(name);
        self
    }

    /// Get all headers
    pub fn build(&self) -> HashMap<String, String> {
        self.headers.clone()
    }

    /// Get a specific header
    pub fn get_header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).map(|s| s.as_str())
    }

    /// Check if a header is set
    pub fn has_header(&self, name: &str) -> bool {
        self.headers.contains_key(name)
    }
}

impl Default for SecurityHeadersBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Validate security headers
pub struct SecurityHeadersValidator;

impl SecurityHeadersValidator {
    /// Check if required security headers are present
    pub fn validate(headers: &HashMap<String, String>) -> Result<(), Vec<String>> {
        let mut missing = Vec::new();

        let required_headers = vec![
            "X-Frame-Options",
            "X-Content-Type-Options",
            "Referrer-Policy",
            "Strict-Transport-Security",
        ];

        for header in required_headers {
            if !headers.contains_key(header) {
                missing.push(format!("Missing required header: {}", header));
            }
        }

        if missing.is_empty() {
            Ok(())
        } else {
            Err(missing)
        }
    }

    /// Check if a header value is secure
    pub fn is_secure_header(name: &str, value: &str) -> bool {
        match name {
            "X-Frame-Options" => {
                value == "DENY" || value == "SAMEORIGIN"
            }
            "X-Content-Type-Options" => {
                value == "nosniff"
            }
            "Referrer-Policy" => {
                matches!(
                    value,
                    "no-referrer"
                        | "no-referrer-when-downgrade"
                        | "same-origin"
                        | "origin"
                        | "strict-origin"
                        | "origin-when-cross-origin"
                        | "strict-origin-when-cross-origin"
                        | "unsafe-url"
                )
            }
            "Strict-Transport-Security" => {
                value.contains("max-age=") && value.contains("31536000")
            }
            "Content-Security-Policy" => {
                !value.contains("unsafe-inline") || value.contains("'unsafe-inline'")
            }
            _ => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(headers.get("Custom-Header"), Some(&"custom-value".to_string()));
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
        headers.insert("Referrer-Policy".to_string(), "strict-origin-when-cross-origin".to_string());
        headers.insert("Strict-Transport-Security".to_string(), "max-age=31536000".to_string());

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
        assert!(SecurityHeadersValidator::is_secure_header("X-Frame-Options", "DENY"));
        assert!(SecurityHeadersValidator::is_secure_header("X-Frame-Options", "SAMEORIGIN"));
        assert!(!SecurityHeadersValidator::is_secure_header("X-Frame-Options", "ALLOW-FROM"));

        assert!(SecurityHeadersValidator::is_secure_header("X-Content-Type-Options", "nosniff"));
        assert!(!SecurityHeadersValidator::is_secure_header("X-Content-Type-Options", "sniff"));

        assert!(SecurityHeadersValidator::is_secure_header("Referrer-Policy", "no-referrer"));
        assert!(SecurityHeadersValidator::is_secure_header("Referrer-Policy", "strict-origin-when-cross-origin"));
    }

    #[test]
    fn test_security_headers_builder_default_values() {
        let builder = SecurityHeadersBuilder::new();
        assert_eq!(builder.get_header("X-Frame-Options"), Some("DENY"));
        assert_eq!(builder.get_header("X-Content-Type-Options"), Some("nosniff"));
        assert!(builder.get_header("Strict-Transport-Security").unwrap().contains("31536000"));
    }
}
