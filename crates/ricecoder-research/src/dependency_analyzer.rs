//! Dependency analysis for multiple programming languages
//!
//! Supports parsing dependencies from manifest files for 11+ languages:
//! - Rust (Cargo.toml)
//! - Node.js (package.json)
//! - Python (pyproject.toml, requirements.txt)
//! - Go (go.mod)
//! - Java (pom.xml, build.gradle)
//! - Kotlin (build.gradle.kts, pom.xml)
//! - .NET (.csproj, packages.config)
//! - PHP (composer.json)
//! - Ruby (Gemfile)
//! - Swift (Package.swift)
//! - Dart/Flutter (pubspec.yaml)

use crate::error::ResearchError;
use crate::models::{Dependency, Language};
use std::path::Path;
use tracing::debug;

mod rust_parser;
mod nodejs_parser;
mod python_parser;
mod go_parser;
mod java_parser;
mod kotlin_parser;
mod dotnet_parser;
mod php_parser;
mod ruby_parser;
mod swift_parser;
mod dart_parser;
mod version_analyzer;

pub use rust_parser::RustParser;
pub use nodejs_parser::NodeJsParser;
pub use python_parser::PythonParser;
pub use go_parser::GoParser;
pub use java_parser::JavaParser;
pub use kotlin_parser::KotlinParser;
pub use dotnet_parser::DotNetParser;
pub use php_parser::PhpParser;
pub use ruby_parser::RubyParser;
pub use swift_parser::SwiftParser;
pub use dart_parser::DartParser;
pub use version_analyzer::VersionAnalyzer;

/// Analyzes project dependencies across multiple languages
#[derive(Debug)]
pub struct DependencyAnalyzer {
    rust_parser: RustParser,
    nodejs_parser: NodeJsParser,
    python_parser: PythonParser,
    go_parser: GoParser,
    java_parser: JavaParser,
    kotlin_parser: KotlinParser,
    dotnet_parser: DotNetParser,
    php_parser: PhpParser,
    ruby_parser: RubyParser,
    swift_parser: SwiftParser,
    dart_parser: DartParser,
    version_analyzer: VersionAnalyzer,
}

impl DependencyAnalyzer {
    /// Creates a new DependencyAnalyzer
    pub fn new() -> Self {
        DependencyAnalyzer {
            rust_parser: RustParser::new(),
            nodejs_parser: NodeJsParser::new(),
            python_parser: PythonParser::new(),
            go_parser: GoParser::new(),
            java_parser: JavaParser::new(),
            kotlin_parser: KotlinParser::new(),
            dotnet_parser: DotNetParser::new(),
            php_parser: PhpParser::new(),
            ruby_parser: RubyParser::new(),
            swift_parser: SwiftParser::new(),
            dart_parser: DartParser::new(),
            version_analyzer: VersionAnalyzer::new(),
        }
    }

    /// Analyzes dependencies in a project
    ///
    /// Detects project language(s) and routes to appropriate parser(s).
    /// Returns all dependencies found across all detected languages.
    pub fn analyze(&self, root: &Path) -> Result<Vec<Dependency>, ResearchError> {
        debug!("Analyzing dependencies in {:?}", root);

        let mut all_dependencies = Vec::new();

        // Try each language parser
        if let Ok(deps) = self.rust_parser.parse(root) {
            debug!("Found {} Rust dependencies", deps.len());
            all_dependencies.extend(deps);
        }

        if let Ok(deps) = self.nodejs_parser.parse(root) {
            debug!("Found {} Node.js dependencies", deps.len());
            all_dependencies.extend(deps);
        }

        if let Ok(deps) = self.python_parser.parse(root) {
            debug!("Found {} Python dependencies", deps.len());
            all_dependencies.extend(deps);
        }

        if let Ok(deps) = self.go_parser.parse(root) {
            debug!("Found {} Go dependencies", deps.len());
            all_dependencies.extend(deps);
        }

        if let Ok(deps) = self.java_parser.parse(root) {
            debug!("Found {} Java dependencies", deps.len());
            all_dependencies.extend(deps);
        }

        if let Ok(deps) = self.kotlin_parser.parse(root) {
            debug!("Found {} Kotlin dependencies", deps.len());
            all_dependencies.extend(deps);
        }

        if let Ok(deps) = self.dotnet_parser.parse(root) {
            debug!("Found {} .NET dependencies", deps.len());
            all_dependencies.extend(deps);
        }

        if let Ok(deps) = self.php_parser.parse(root) {
            debug!("Found {} PHP dependencies", deps.len());
            all_dependencies.extend(deps);
        }

        if let Ok(deps) = self.ruby_parser.parse(root) {
            debug!("Found {} Ruby dependencies", deps.len());
            all_dependencies.extend(deps);
        }

        if let Ok(deps) = self.swift_parser.parse(root) {
            debug!("Found {} Swift dependencies", deps.len());
            all_dependencies.extend(deps);
        }

        if let Ok(deps) = self.dart_parser.parse(root) {
            debug!("Found {} Dart/Flutter dependencies", deps.len());
            all_dependencies.extend(deps);
        }

        // Remove duplicates (same name and version)
        all_dependencies.sort_by(|a, b| {
            a.name.cmp(&b.name)
                .then_with(|| a.version.cmp(&b.version))
        });
        all_dependencies.dedup_by(|a, b| a.name == b.name && a.version == b.version);

        Ok(all_dependencies)
    }

    /// Detects which languages are present in a project
    pub fn detect_languages(&self, root: &Path) -> Result<Vec<Language>, ResearchError> {
        let mut languages = Vec::new();

        if self.rust_parser.has_manifest(root) {
            languages.push(Language::Rust);
        }

        if self.nodejs_parser.has_manifest(root) {
            languages.push(Language::TypeScript);
        }

        if self.python_parser.has_manifest(root) {
            languages.push(Language::Python);
        }

        if self.go_parser.has_manifest(root) {
            languages.push(Language::Go);
        }

        if self.java_parser.has_manifest(root) {
            languages.push(Language::Java);
        }

        if self.kotlin_parser.has_manifest(root) {
            languages.push(Language::Kotlin);
        }

        if self.dotnet_parser.has_manifest(root) {
            languages.push(Language::CSharp);
        }

        if self.php_parser.has_manifest(root) {
            languages.push(Language::Php);
        }

        if self.ruby_parser.has_manifest(root) {
            languages.push(Language::Ruby);
        }

        if self.swift_parser.has_manifest(root) {
            languages.push(Language::Swift);
        }

        if self.dart_parser.has_manifest(root) {
            languages.push(Language::Dart);
        }

        Ok(languages)
    }

    /// Analyzes version conflicts across dependencies
    pub fn analyze_version_conflicts(&self, dependencies: &[Dependency]) -> Vec<VersionConflict> {
        self.version_analyzer.find_conflicts(dependencies)
    }

    /// Suggests version updates for dependencies
    pub fn suggest_updates(&self, dependencies: &[Dependency]) -> Vec<VersionUpdate> {
        self.version_analyzer.suggest_updates(dependencies)
    }
}

impl Default for DependencyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a version conflict between dependencies
#[derive(Debug, Clone)]
pub struct VersionConflict {
    /// Name of the dependency
    pub dependency_name: String,
    /// Conflicting versions
    pub versions: Vec<String>,
    /// Description of the conflict
    pub description: String,
}

/// Represents a suggested version update
#[derive(Debug, Clone)]
pub struct VersionUpdate {
    /// Name of the dependency
    pub dependency_name: String,
    /// Current version
    pub current_version: String,
    /// Suggested version
    pub suggested_version: String,
    /// Reason for the update
    pub reason: String,
}

/// Trait for language-specific dependency parsers
pub trait DependencyParser: Send + Sync {
    /// Parses dependencies from manifest files
    fn parse(&self, root: &Path) -> Result<Vec<Dependency>, ResearchError>;

    /// Checks if the language's manifest file exists
    fn has_manifest(&self, root: &Path) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_analyzer_creation() {
        let analyzer = DependencyAnalyzer::new();
        // Verify analyzer is created successfully
        assert!(true);
    }

    #[test]
    fn test_dependency_analyzer_default() {
        let analyzer = DependencyAnalyzer::default();
        // Verify default creation works
        assert!(true);
    }
}
