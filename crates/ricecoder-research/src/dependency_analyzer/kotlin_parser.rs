//! Kotlin dependency parser for build.gradle.kts and pom.xml

use crate::error::ResearchError;
use crate::models::Dependency;
use std::path::Path;
use tracing::debug;

/// Parses Kotlin dependencies from build.gradle.kts and pom.xml
#[derive(Debug)]
pub struct KotlinParser;

impl KotlinParser {
    /// Creates a new KotlinParser
    pub fn new() -> Self {
        KotlinParser
    }

    /// Parses dependencies from build.gradle.kts or pom.xml
    pub fn parse(&self, root: &Path) -> Result<Vec<Dependency>, ResearchError> {
        let mut dependencies = Vec::new();

        // Try build.gradle.kts first
        let gradle_kts_path = root.join("build.gradle.kts");
        if gradle_kts_path.exists() {
            debug!("Parsing Kotlin dependencies from {:?}", gradle_kts_path);
            if let Ok(mut deps) = self.parse_gradle_kts(&gradle_kts_path) {
                dependencies.append(&mut deps);
            }
        }

        // Try pom.xml
        let pom_path = root.join("pom.xml");
        if pom_path.exists() {
            debug!("Parsing Kotlin dependencies from {:?}", pom_path);
            if let Ok(mut deps) = self.parse_pom(&pom_path) {
                dependencies.append(&mut deps);
            }
        }

        Ok(dependencies)
    }

    /// Parses dependencies from build.gradle.kts
    fn parse_gradle_kts(&self, path: &Path) -> Result<Vec<Dependency>, ResearchError> {
        let content =
            std::fs::read_to_string(path).map_err(|e| ResearchError::DependencyParsingFailed {
                language: "Kotlin".to_string(),
                path: Some(path.to_path_buf()),
                reason: format!("Failed to read build.gradle.kts: {}", e),
            })?;

        let mut dependencies = Vec::new();

        // Parse dependencies block
        // Look for patterns like: implementation("group:artifact:version")
        let dep_pattern = regex::Regex::new(
            r#"(?:implementation|testImplementation|api|testApi|compileOnly|testCompileOnly)\s*\(\s*["\']([^:]+):([^:]+):([^"\']+)["\']\s*\)"#
        ).unwrap();

        for cap in dep_pattern.captures_iter(&content) {
            let group_id = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let artifact_id = cap.get(2).map(|m| m.as_str()).unwrap_or("");
            let version = cap.get(3).map(|m| m.as_str()).unwrap_or("");

            let name = format!("{}:{}", group_id, artifact_id);

            dependencies.push(Dependency {
                name,
                version: version.to_string(),
                constraints: Some(version.to_string()),
                is_dev: false,
            });
        }

        Ok(dependencies)
    }

    /// Parses dependencies from pom.xml
    fn parse_pom(&self, path: &Path) -> Result<Vec<Dependency>, ResearchError> {
        let content =
            std::fs::read_to_string(path).map_err(|e| ResearchError::DependencyParsingFailed {
                language: "Kotlin".to_string(),
                path: Some(path.to_path_buf()),
                reason: format!("Failed to read pom.xml: {}", e),
            })?;

        let mut dependencies = Vec::new();

        // Simple regex-based parsing for dependencies
        let dep_pattern = regex::Regex::new(
            r"<dependency>\s*<groupId>([^<]+)</groupId>\s*<artifactId>([^<]+)</artifactId>\s*<version>([^<]+)</version>(?:\s*<scope>([^<]+)</scope>)?"
        ).unwrap();

        for cap in dep_pattern.captures_iter(&content) {
            let group_id = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let artifact_id = cap.get(2).map(|m| m.as_str()).unwrap_or("");
            let version = cap.get(3).map(|m| m.as_str()).unwrap_or("");
            let scope = cap.get(4).map(|m| m.as_str()).unwrap_or("compile");

            let name = format!("{}:{}", group_id, artifact_id);
            let is_dev = scope == "test";

            dependencies.push(Dependency {
                name,
                version: version.to_string(),
                constraints: Some(version.to_string()),
                is_dev,
            });
        }

        Ok(dependencies)
    }

    /// Checks if Kotlin manifest files exist
    pub fn has_manifest(&self, root: &Path) -> bool {
        root.join("build.gradle.kts").exists() || root.join("pom.xml").exists()
    }
}

impl Default for KotlinParser {
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
    fn test_kotlin_parser_creation() {
        let parser = KotlinParser::new();
        assert!(true);
    }

    #[test]
    fn test_kotlin_parser_no_manifest() {
        let parser = KotlinParser::new();
        let temp_dir = TempDir::new().unwrap();
        let result = parser.parse(temp_dir.path()).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_kotlin_parser_gradle_kts() {
        let parser = KotlinParser::new();
        let temp_dir = TempDir::new().unwrap();

        let gradle_kts = r#"
dependencies {
    implementation("org.jetbrains.kotlin:kotlin-stdlib:1.9.0")
    testImplementation("junit:junit:4.13.2")
}
"#;

        fs::write(temp_dir.path().join("build.gradle.kts"), gradle_kts).unwrap();

        let deps = parser.parse(temp_dir.path()).unwrap();
        assert_eq!(deps.len(), 2);
    }

    #[test]
    fn test_kotlin_parser_has_manifest() {
        let parser = KotlinParser::new();
        let temp_dir = TempDir::new().unwrap();

        assert!(!parser.has_manifest(temp_dir.path()));

        fs::write(temp_dir.path().join("build.gradle.kts"), "").unwrap();
        assert!(parser.has_manifest(temp_dir.path()));
    }
}
