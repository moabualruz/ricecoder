//! Data models for pattern detection

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Detected pattern information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DetectedPattern {
    /// Pattern name
    pub name: String,
    /// Pattern category
    pub category: PatternCategory,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
    /// Pattern location information
    pub locations: Vec<PatternLocation>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Pattern category classification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PatternCategory {
    /// Architectural patterns (layered, microservices, etc.)
    Architectural,
    /// Design patterns (factory, observer, etc.)
    Design,
    /// Coding conventions (naming, documentation, etc.)
    Convention,
    /// Anti-patterns (code smells, bad practices)
    AntiPattern,
}

/// Location where pattern was detected
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatternLocation {
    /// File path
    pub file: String,
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
    /// Code snippet
    pub snippet: String,
}

/// Architectural pattern types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ArchitecturalPattern {
    /// Layered architecture (presentation, business, data)
    LayeredArchitecture,
    /// Microservices architecture
    Microservices,
    /// Event-driven architecture
    EventDriven,
    /// Monolithic architecture
    Monolithic,
    /// Hexagonal architecture
    Hexagonal,
    /// Clean architecture
    Clean,
}

/// Design pattern types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DesignPattern {
    /// Factory pattern
    Factory,
    /// Observer pattern
    Observer,
    /// Repository pattern
    Repository,
    /// Strategy pattern
    Strategy,
    /// Singleton pattern
    Singleton,
    /// Builder pattern
    Builder,
}

/// Coding convention types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CodingConvention {
    /// Naming convention (camelCase, snake_case, etc.)
    NamingConvention,
    /// Documentation style
    DocumentationStyle,
    /// Import organization
    ImportOrganization,
    /// Error handling patterns
    ErrorHandling,
}

/// Pattern detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternDetectionConfig {
    /// Minimum confidence threshold (0.0 to 1.0)
    pub min_confidence: f64,
    /// Maximum number of patterns to detect
    pub max_patterns: usize,
    /// Enable architectural pattern detection
    pub detect_architectural: bool,
    /// Enable design pattern detection
    pub detect_design: bool,
    /// Enable convention analysis
    pub detect_conventions: bool,
    /// Enable anti-pattern detection
    pub detect_anti_patterns: bool,
}

impl Default for PatternDetectionConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.7,
            max_patterns: 50,
            detect_architectural: true,
            detect_design: true,
            detect_conventions: true,
            detect_anti_patterns: true,
        }
    }
}