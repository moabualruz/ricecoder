//! Ruby dependency parser for Gemfile

use crate::error::ResearchError;
use crate::models::Dependency;
use std::path::Path;
use tracing::debug;

/// Parses Ruby dependencies from Gemfile
#[derive(Debug)]
pub struct RubyParser;

impl RubyParser {
    /// Creates a new RubyParser
    pub fn new() -> Self {
        RubyParser
    }

    /// Parses dependencies from Gemfile
    pub fn parse(&self, root: &Path) -> Result<Vec<Dependency>, ResearchError> {
        let gemfile_path = root.join("Gemfile");

        if !gemfile_path.exists() {
            return Ok(Vec::new());
        }

        debug!("Parsing Ruby dependencies from {:?}", gemfile_path);

        let content = std::fs::read_to_string(&gemfile_path)
            .map_err(|e| ResearchError::DependencyParsingFailed {
                language: "Ruby".to_string(),
                path: Some(gemfile_path.clone()),
                reason: format!("Failed to read Gemfile: {}", e),
            })?;

        let mut dependencies = Vec::new();

        // Parse gem declarations
        // Patterns: gem 'name', 'version'
        //          gem 'name', '~> 1.0'
        //          gem 'name'
        let gem_pattern = regex::Regex::new(
            r#"gem\s+['"]([a-zA-Z0-9_\-\.]+)['"](?:\s*,\s*['"]([^'"]+)['"])?"#
        ).unwrap();

        for cap in gem_pattern.captures_iter(&content) {
            let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let version = cap.get(2).map(|m| m.as_str()).unwrap_or("*");

            dependencies.push(Dependency {
                name: name.to_string(),
                version: version.to_string(),
                constraints: Some(version.to_string()),
                is_dev: false,
            });
        }

        Ok(dependencies)
    }

    /// Checks if Gemfile exists
    pub fn has_manifest(&self, root: &Path) -> bool {
        root.join("Gemfile").exists()
    }
}

impl Default for RubyParser {
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
    fn test_ruby_parser_creation() {
        let parser = RubyParser::new();
        assert!(true);
    }

    #[test]
    fn test_ruby_parser_no_manifest() {
        let parser = RubyParser::new();
        let temp_dir = TempDir::new().unwrap();
        let result = parser.parse(temp_dir.path()).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_ruby_parser_simple_dependencies() {
        let parser = RubyParser::new();
        let temp_dir = TempDir::new().unwrap();

        let gemfile = r#"
source 'https://rubygems.org'

gem 'rails', '~> 7.0'
gem 'pg', '~> 1.1'
gem 'puma', '~> 5.0'

group :development, :test do
  gem 'rspec-rails', '~> 5.0'
end
"#;

        fs::write(temp_dir.path().join("Gemfile"), gemfile).unwrap();

        let deps = parser.parse(temp_dir.path()).unwrap();
        assert!(deps.len() >= 3);

        let rails = deps.iter().find(|d| d.name == "rails").unwrap();
        assert_eq!(rails.version, "~> 7.0");
    }

    #[test]
    fn test_ruby_parser_has_manifest() {
        let parser = RubyParser::new();
        let temp_dir = TempDir::new().unwrap();

        assert!(!parser.has_manifest(temp_dir.path()));

        fs::write(temp_dir.path().join("Gemfile"), "").unwrap();
        assert!(parser.has_manifest(temp_dir.path()));
    }
}
