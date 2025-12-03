//! .NET dependency parser for .csproj and packages.config

use crate::error::ResearchError;
use crate::models::Dependency;
use std::path::Path;
use tracing::debug;

/// Parses .NET dependencies from .csproj and packages.config
#[derive(Debug)]
pub struct DotNetParser;

impl DotNetParser {
    /// Creates a new DotNetParser
    pub fn new() -> Self {
        DotNetParser
    }

    /// Parses dependencies from .csproj or packages.config
    pub fn parse(&self, root: &Path) -> Result<Vec<Dependency>, ResearchError> {
        let mut dependencies = Vec::new();

        // Try .csproj files
        if let Ok(entries) = std::fs::read_dir(root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "csproj") {
                    debug!("Parsing .NET dependencies from {:?}", path);
                    if let Ok(mut deps) = self.parse_csproj(&path) {
                        dependencies.append(&mut deps);
                    }
                }
            }
        }

        // Try packages.config
        let packages_config_path = root.join("packages.config");
        if packages_config_path.exists() {
            debug!("Parsing .NET dependencies from {:?}", packages_config_path);
            if let Ok(mut deps) = self.parse_packages_config(&packages_config_path) {
                dependencies.append(&mut deps);
            }
        }

        Ok(dependencies)
    }

    /// Parses dependencies from .csproj
    fn parse_csproj(&self, path: &Path) -> Result<Vec<Dependency>, ResearchError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ResearchError::DependencyParsingFailed {
                language: ".NET".to_string(),
                path: Some(path.to_path_buf()),
                reason: format!("Failed to read .csproj: {}", e),
            })?;

        let mut dependencies = Vec::new();

        // Look for PackageReference elements
        // <PackageReference Include="PackageName" Version="1.0.0" />
        let dep_pattern = regex::Regex::new(
            r#"<PackageReference\s+Include="([^"]+)"\s+Version="([^"]+)"#
        ).unwrap();

        for cap in dep_pattern.captures_iter(&content) {
            let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let version = cap.get(2).map(|m| m.as_str()).unwrap_or("");

            dependencies.push(Dependency {
                name: name.to_string(),
                version: version.to_string(),
                constraints: Some(version.to_string()),
                is_dev: false,
            });
        }

        Ok(dependencies)
    }

    /// Parses dependencies from packages.config
    fn parse_packages_config(&self, path: &Path) -> Result<Vec<Dependency>, ResearchError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ResearchError::DependencyParsingFailed {
                language: ".NET".to_string(),
                path: Some(path.to_path_buf()),
                reason: format!("Failed to read packages.config: {}", e),
            })?;

        let mut dependencies = Vec::new();

        // Look for package elements
        // <package id="PackageName" version="1.0.0" />
        let dep_pattern = regex::Regex::new(
            r#"<package\s+id="([^"]+)"\s+version="([^"]+)"#
        ).unwrap();

        for cap in dep_pattern.captures_iter(&content) {
            let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let version = cap.get(2).map(|m| m.as_str()).unwrap_or("");

            dependencies.push(Dependency {
                name: name.to_string(),
                version: version.to_string(),
                constraints: Some(version.to_string()),
                is_dev: false,
            });
        }

        Ok(dependencies)
    }

    /// Checks if .NET manifest files exist
    pub fn has_manifest(&self, root: &Path) -> bool {
        // Check for .csproj files
        if let Ok(entries) = std::fs::read_dir(root) {
            for entry in entries.flatten() {
                if entry.path().extension().map_or(false, |ext| ext == "csproj") {
                    return true;
                }
            }
        }

        // Check for packages.config
        root.join("packages.config").exists()
    }
}

impl Default for DotNetParser {
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
    fn test_dotnet_parser_creation() {
        let parser = DotNetParser::new();
        assert!(true);
    }

    #[test]
    fn test_dotnet_parser_no_manifest() {
        let parser = DotNetParser::new();
        let temp_dir = TempDir::new().unwrap();
        let result = parser.parse(temp_dir.path()).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_dotnet_parser_csproj() {
        let parser = DotNetParser::new();
        let temp_dir = TempDir::new().unwrap();

        let csproj = r#"<?xml version="1.0"?>
<Project Sdk="Microsoft.NET.Sdk">
  <ItemGroup>
    <PackageReference Include="Newtonsoft.Json" Version="13.0.1" />
    <PackageReference Include="System.Net.Http" Version="4.3.4" />
  </ItemGroup>
</Project>"#;

        fs::write(temp_dir.path().join("test.csproj"), csproj).unwrap();

        let deps = parser.parse(temp_dir.path()).unwrap();
        assert_eq!(deps.len(), 2);

        let newtonsoft = deps.iter().find(|d| d.name == "Newtonsoft.Json").unwrap();
        assert_eq!(newtonsoft.version, "13.0.1");
    }

    #[test]
    fn test_dotnet_parser_has_manifest() {
        let parser = DotNetParser::new();
        let temp_dir = TempDir::new().unwrap();

        assert!(!parser.has_manifest(temp_dir.path()));

        fs::write(temp_dir.path().join("test.csproj"), "").unwrap();
        assert!(parser.has_manifest(temp_dir.path()));
    }
}
