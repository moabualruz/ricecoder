//! Swift dependency parser for Package.swift

use crate::error::ResearchError;
use crate::models::Dependency;
use std::path::Path;
use tracing::debug;

/// Parses Swift dependencies from Package.swift
#[derive(Debug)]
pub struct SwiftParser;

impl SwiftParser {
    /// Creates a new SwiftParser
    pub fn new() -> Self {
        SwiftParser
    }

    /// Parses dependencies from Package.swift
    pub fn parse(&self, root: &Path) -> Result<Vec<Dependency>, ResearchError> {
        let package_swift_path = root.join("Package.swift");

        if !package_swift_path.exists() {
            return Ok(Vec::new());
        }

        debug!("Parsing Swift dependencies from {:?}", package_swift_path);

        let content = std::fs::read_to_string(&package_swift_path)
            .map_err(|e| ResearchError::DependencyParsingFailed {
                language: "Swift".to_string(),
                path: Some(package_swift_path.clone()),
                reason: format!("Failed to read Package.swift: {}", e),
            })?;

        let mut dependencies = Vec::new();

        // Parse .package declarations
        // Pattern: .package(url: "...", from: "1.0.0")
        //          .package(url: "...", .upToNextMajor(from: "1.0.0"))
        let package_pattern = regex::Regex::new(
            r#"\.package\(url:\s*"([^"]+)"[^)]*(?:from|\.upToNextMajor|\.upToNextMinor):\s*"([^"]+)"[^)]*\)"#
        ).unwrap();

        for cap in package_pattern.captures_iter(&content) {
            let url = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let version = cap.get(2).map(|m| m.as_str()).unwrap_or("");

            // Extract package name from URL
            let name = if let Some(last_slash) = url.rfind('/') {
                let name_with_ext = &url[last_slash + 1..];
                if let Some(stripped) = name_with_ext.strip_suffix(".git") {
                    stripped
                } else {
                    name_with_ext
                }
            } else {
                url
            };

            dependencies.push(Dependency {
                name: name.to_string(),
                version: version.to_string(),
                constraints: Some(version.to_string()),
                is_dev: false,
            });
        }

        Ok(dependencies)
    }

    /// Checks if Package.swift exists
    pub fn has_manifest(&self, root: &Path) -> bool {
        root.join("Package.swift").exists()
    }
}

impl Default for SwiftParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_swift_parser_creation() {
        let parser = SwiftParser::new();
        assert!(true);
    }

    #[test]
    fn test_swift_parser_no_manifest() {
        let parser = SwiftParser::new();
        let temp_dir = TempDir::new().unwrap();
        let result = parser.parse(temp_dir.path()).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_swift_parser_simple_dependencies() {
        let parser = SwiftParser::new();
        let temp_dir = TempDir::new().unwrap();

        let package_swift = r#"
// swift-tools-version:5.5
import PackageDescription

let package = Package(
    name: "MyPackage",
    dependencies: [
        .package(url: "https://github.com/apple/swift-nio.git", from: "2.0.0"),
        .package(url: "https://github.com/vapor/vapor.git", .upToNextMajor(from: "4.0.0"))
    ]
)
"#;

        fs::write(temp_dir.path().join("Package.swift"), package_swift).unwrap();

        let deps = parser.parse(temp_dir.path()).unwrap();
        assert_eq!(deps.len(), 2);

        let nio = deps.iter().find(|d| d.name.contains("nio")).unwrap();
        assert_eq!(nio.version, "2.0.0");
    }

    #[test]
    fn test_swift_parser_has_manifest() {
        let parser = SwiftParser::new();
        let temp_dir = TempDir::new().unwrap();

        assert!(!parser.has_manifest(temp_dir.path()));

        fs::write(temp_dir.path().join("Package.swift"), "").unwrap();
        assert!(parser.has_manifest(temp_dir.path()));
    }
}
