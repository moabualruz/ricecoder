//! Java dependency parser for pom.xml and build.gradle

use crate::error::ResearchError;
use crate::models::Dependency;
use std::path::Path;
use tracing::debug;

/// Parses Java dependencies from pom.xml and build.gradle
#[derive(Debug)]
pub struct JavaParser;

impl JavaParser {
    /// Creates a new JavaParser
    pub fn new() -> Self {
        JavaParser
    }

    /// Parses dependencies from pom.xml or build.gradle
    pub fn parse(&self, root: &Path) -> Result<Vec<Dependency>, ResearchError> {
        let mut dependencies = Vec::new();

        // Try pom.xml first
        let pom_path = root.join("pom.xml");
        if pom_path.exists() {
            debug!("Parsing Java dependencies from {:?}", pom_path);
            if let Ok(mut deps) = self.parse_pom(&pom_path) {
                dependencies.append(&mut deps);
            }
        }

        // Try build.gradle
        let gradle_path = root.join("build.gradle");
        if gradle_path.exists() {
            debug!("Parsing Java dependencies from {:?}", gradle_path);
            if let Ok(mut deps) = self.parse_gradle(&gradle_path) {
                dependencies.append(&mut deps);
            }
        }

        Ok(dependencies)
    }

    /// Parses dependencies from pom.xml
    fn parse_pom(&self, path: &Path) -> Result<Vec<Dependency>, ResearchError> {
        let content =
            std::fs::read_to_string(path).map_err(|e| ResearchError::DependencyParsingFailed {
                language: "Java".to_string(),
                path: Some(path.to_path_buf()),
                reason: format!("Failed to read pom.xml: {}", e),
            })?;

        let mut dependencies = Vec::new();

        // Simple regex-based parsing for dependencies
        // Look for <dependency> blocks
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

    /// Parses dependencies from build.gradle
    fn parse_gradle(&self, path: &Path) -> Result<Vec<Dependency>, ResearchError> {
        let content =
            std::fs::read_to_string(path).map_err(|e| ResearchError::DependencyParsingFailed {
                language: "Java".to_string(),
                path: Some(path.to_path_buf()),
                reason: format!("Failed to read build.gradle: {}", e),
            })?;

        let mut dependencies = Vec::new();

        // Parse dependencies block
        // Look for patterns like: implementation 'group:artifact:version'
        let dep_pattern = regex::Regex::new(
            r#"(?:implementation|testImplementation|api|testApi|compileOnly|testCompileOnly)\s+['"]([^:]+):([^:]+):([^'"]+)['"]"#
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

    /// Checks if Java manifest files exist
    pub fn has_manifest(&self, root: &Path) -> bool {
        root.join("pom.xml").exists() || root.join("build.gradle").exists()
    }
}

impl Default for JavaParser {
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
    fn test_java_parser_creation() {
        let parser = JavaParser::new();
        assert!(true);
    }

    #[test]
    fn test_java_parser_no_manifest() {
        let parser = JavaParser::new();
        let temp_dir = TempDir::new().unwrap();
        let result = parser.parse(temp_dir.path()).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_java_parser_pom_xml() {
        let parser = JavaParser::new();
        let temp_dir = TempDir::new().unwrap();

        let pom = r#"<?xml version="1.0"?>
<project>
    <dependencies>
        <dependency>
            <groupId>junit</groupId>
            <artifactId>junit</artifactId>
            <version>4.13.2</version>
            <scope>test</scope>
        </dependency>
        <dependency>
            <groupId>org.springframework</groupId>
            <artifactId>spring-core</artifactId>
            <version>5.3.0</version>
        </dependency>
    </dependencies>
</project>"#;

        fs::write(temp_dir.path().join("pom.xml"), pom).unwrap();

        let deps = parser.parse(temp_dir.path()).unwrap();
        assert_eq!(deps.len(), 2);

        let junit = deps.iter().find(|d| d.name.contains("junit")).unwrap();
        assert!(junit.is_dev);
    }

    #[test]
    fn test_java_parser_has_manifest() {
        let parser = JavaParser::new();
        let temp_dir = TempDir::new().unwrap();

        assert!(!parser.has_manifest(temp_dir.path()));

        fs::write(temp_dir.path().join("pom.xml"), "").unwrap();
        assert!(parser.has_manifest(temp_dir.path()));
    }
}
