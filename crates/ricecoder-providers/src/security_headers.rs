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
        headers.insert("X-Frame-Options".to_string(), "DENY".to_string());

        // Prevent MIME type sniffing
        headers.insert("X-Content-Type-Options".to_string(), "nosniff".to_string());

        // Enable XSS protection (for older browsers)
        headers.insert("X-XSS-Protection".to_string(), "1; mode=block".to_string());

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
            "X-Frame-Options" => value == "DENY" || value == "SAMEORIGIN",
            "X-Content-Type-Options" => value == "nosniff",
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
            "Strict-Transport-Security" => value.contains("max-age=") && value.contains("31536000"),
            "Content-Security-Policy" => {
                !value.contains("unsafe-inline") || value.contains("'unsafe-inline'")
            }
            _ => true,
        }
    }
}
