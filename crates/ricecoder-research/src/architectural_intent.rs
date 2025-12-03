//! Architectural intent tracking and analysis
//!
//! This module provides functionality to track and understand architectural decisions,
//! infer architectural styles from code structure, and parse Architecture Decision Records (ADRs).

use crate::models::{ArchitecturalIntent, ArchitecturalStyle, ArchitecturalDecision};
use crate::ResearchError;
use std::path::Path;
use chrono::Utc;

/// Tracks and understands architectural decisions and patterns
#[derive(Debug, Clone)]
pub struct ArchitecturalIntentTracker {
    /// Minimum confidence threshold for style inference (0.0 to 1.0)
    pub confidence_threshold: f32,
}

impl ArchitecturalIntentTracker {
    /// Creates a new ArchitecturalIntentTracker with default settings
    pub fn new() -> Self {
        Self {
            confidence_threshold: 0.5,
        }
    }

    /// Creates a new ArchitecturalIntentTracker with a custom confidence threshold
    pub fn with_threshold(confidence_threshold: f32) -> Self {
        Self {
            confidence_threshold: confidence_threshold.clamp(0.0, 1.0),
        }
    }

    /// Analyzes code structure to infer architectural style
    ///
    /// # Arguments
    ///
    /// * `root` - The root directory of the project
    ///
    /// # Returns
    ///
    /// The inferred architectural style, or an error if analysis fails
    pub fn infer_style(&self, root: &Path) -> Result<ArchitecturalStyle, ResearchError> {
        // Analyze directory structure and module organization
        let style = self.infer_from_structure(root)?;
        Ok(style)
    }

    /// Infers architectural style from directory structure
    fn infer_from_structure(&self, root: &Path) -> Result<ArchitecturalStyle, ResearchError> {
        // Check for common architectural patterns in directory structure
        
        // Check for layered architecture (domain, application, infrastructure, interfaces)
        if self.has_layered_structure(root)? {
            return Ok(ArchitecturalStyle::Layered);
        }

        // Check for microservices pattern (multiple service directories)
        if self.has_microservices_structure(root)? {
            return Ok(ArchitecturalStyle::Microservices);
        }

        // Check for event-driven pattern (event handlers, pub/sub)
        if self.has_event_driven_structure(root)? {
            return Ok(ArchitecturalStyle::EventDriven);
        }

        // Check for serverless pattern (functions, handlers)
        if self.has_serverless_structure(root)? {
            return Ok(ArchitecturalStyle::Serverless);
        }

        // Default to monolithic if no specific pattern detected
        Ok(ArchitecturalStyle::Monolithic)
    }

    /// Checks if the project has a layered architecture structure
    fn has_layered_structure(&self, root: &Path) -> Result<bool, ResearchError> {
        // Look for common layered architecture directories
        let layered_dirs = ["domain", "application", "infrastructure", "interfaces"];
        
        for dir in &layered_dirs {
            let path = root.join(dir);
            if path.exists() && path.is_dir() {
                return Ok(true);
            }
        }

        // Also check for src subdirectories with layer names
        let src_path = root.join("src");
        if src_path.exists() && src_path.is_dir() {
            for dir in &layered_dirs {
                let path = src_path.join(dir);
                if path.exists() && path.is_dir() {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Checks if the project has a microservices structure
    fn has_microservices_structure(&self, root: &Path) -> Result<bool, ResearchError> {
        // Look for multiple service directories
        let services_path = root.join("services");
        if services_path.exists() && services_path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&services_path) {
                let service_count = entries.filter_map(|e| e.ok()).filter(|e| {
                    e.path().is_dir()
                }).count();
                
                if service_count >= 2 {
                    return Ok(true);
                }
            }
        }

        // Check for microservices in crates (Rust workspace)
        let crates_path = root.join("crates");
        if crates_path.exists() && crates_path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&crates_path) {
                let crate_count = entries.filter_map(|e| e.ok()).filter(|e| {
                    e.path().is_dir()
                }).count();
                
                if crate_count >= 2 {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Checks if the project has an event-driven structure
    fn has_event_driven_structure(&self, root: &Path) -> Result<bool, ResearchError> {
        // Look for event-related directories
        let event_dirs = ["events", "handlers", "subscribers", "listeners"];
        
        for dir in &event_dirs {
            let path = root.join(dir);
            if path.exists() && path.is_dir() {
                return Ok(true);
            }
        }

        // Check in src directory
        let src_path = root.join("src");
        if src_path.exists() && src_path.is_dir() {
            for dir in &event_dirs {
                let path = src_path.join(dir);
                if path.exists() && path.is_dir() {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Checks if the project has a serverless structure
    fn has_serverless_structure(&self, root: &Path) -> Result<bool, ResearchError> {
        // Look for serverless-specific directories
        let serverless_dirs = ["functions", "handlers", "lambdas"];
        
        for dir in &serverless_dirs {
            let path = root.join(dir);
            if path.exists() && path.is_dir() {
                return Ok(true);
            }
        }

        // Check for serverless.yml or serverless.yaml
        if root.join("serverless.yml").exists() || root.join("serverless.yaml").exists() {
            return Ok(true);
        }

        Ok(false)
    }

    /// Parses Architecture Decision Record (ADR) files
    ///
    /// # Arguments
    ///
    /// * `root` - The root directory to search for ADR files
    ///
    /// # Returns
    ///
    /// A vector of parsed architectural decisions, or an error if parsing fails
    pub fn parse_adrs(&self, root: &Path) -> Result<Vec<ArchitecturalDecision>, ResearchError> {
        let mut decisions = Vec::new();

        // Look for ADR files in common locations
        let adr_dirs = vec![
            root.join("docs/adr"),
            root.join("docs/decisions"),
            root.join("adr"),
            root.join("decisions"),
            root.join("architecture/decisions"),
        ];

        for adr_dir in adr_dirs {
            if adr_dir.exists() && adr_dir.is_dir() {
                if let Ok(entries) = std::fs::read_dir(&adr_dir) {
                    for entry in entries.filter_map(|e| e.ok()) {
                        let path = entry.path();
                        if path.is_file() && (path.extension().is_some_and(|ext| ext == "md")) {
                            if let Ok(decision) = self.parse_adr_file(&path) {
                                decisions.push(decision);
                            }
                        }
                    }
                }
            }
        }

        Ok(decisions)
    }

    /// Parses a single ADR file
    fn parse_adr_file(&self, path: &Path) -> Result<ArchitecturalDecision, ResearchError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ResearchError::IoError {
                reason: format!("Failed to read ADR file: {}", e),
            })?;

        // Extract ADR metadata from filename and content
        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        // Parse ADR ID from filename (e.g., "0001-use-rust.md" -> "0001")
        let id = filename.split('-').next().unwrap_or("unknown").to_string();

        // Extract title from filename
        let title = filename
            .strip_prefix(&format!("{}-", id))
            .and_then(|s| s.strip_suffix(".md"))
            .unwrap_or(filename)
            .replace('-', " ");

        // Parse sections from content
        let (context, decision, consequences) = self.parse_adr_sections(&content);

        Ok(ArchitecturalDecision {
            id,
            title,
            context,
            decision,
            consequences,
            date: Utc::now(),
        })
    }

    /// Parses ADR sections from markdown content
    fn parse_adr_sections(&self, content: &str) -> (String, String, Vec<String>) {
        let mut context = String::new();
        let mut decision = String::new();
        let mut consequences = Vec::new();

        let lines: Vec<&str> = content.lines().collect();
        let mut current_section = "";

        for line in lines {
            let lower = line.to_lowercase();
            
            if lower.contains("## context") || lower.contains("# context") {
                current_section = "context";
            } else if lower.contains("## decision") || lower.contains("# decision") {
                current_section = "decision";
            } else if lower.contains("## consequences") || lower.contains("# consequences") {
                current_section = "consequences";
            } else if line.starts_with('#') {
                current_section = "";
            } else if !line.trim().is_empty() {
                match current_section {
                    "context" => {
                        context.push_str(line);
                        context.push('\n');
                    }
                    "decision" => {
                        decision.push_str(line);
                        decision.push('\n');
                    }
                    "consequences" => {
                        if line.trim().starts_with('-') || line.trim().starts_with('*') {
                            consequences.push(line.trim_start_matches(['-', '*']).trim().to_string());
                        }
                    }
                    _ => {}
                }
            }
        }

        (context.trim().to_string(), decision.trim().to_string(), consequences)
    }

    /// Builds complete architectural intent from analysis
    ///
    /// # Arguments
    ///
    /// * `root` - The root directory of the project
    ///
    /// # Returns
    ///
    /// The complete architectural intent, or an error if analysis fails
    pub fn build_intent(&self, root: &Path) -> Result<ArchitecturalIntent, ResearchError> {
        let style = self.infer_style(root)?;
        let decisions = self.parse_adrs(root)?;
        
        // Extract principles and constraints from decisions
        let principles = self.extract_principles(&decisions);
        let constraints = self.extract_constraints(&decisions);

        Ok(ArchitecturalIntent {
            style,
            principles,
            constraints,
            decisions,
        })
    }

    /// Extracts architectural principles from decisions
    fn extract_principles(&self, decisions: &[ArchitecturalDecision]) -> Vec<String> {
        let mut principles = Vec::new();

        for decision in decisions {
            // Extract principles from decision context and decision text
            if decision.context.to_lowercase().contains("principle") {
                principles.push(decision.context.clone());
            }
            if decision.decision.to_lowercase().contains("principle") {
                principles.push(decision.decision.clone());
            }
        }

        principles.sort();
        principles.dedup();
        principles
    }

    /// Extracts architectural constraints from decisions
    fn extract_constraints(&self, decisions: &[ArchitecturalDecision]) -> Vec<String> {
        let mut constraints = Vec::new();

        for decision in decisions {
            // Extract constraints from consequences
            for consequence in &decision.consequences {
                if consequence.to_lowercase().contains("constraint") 
                    || consequence.to_lowercase().contains("must")
                    || consequence.to_lowercase().contains("require") {
                    constraints.push(consequence.clone());
                }
            }
        }

        constraints.sort();
        constraints.dedup();
        constraints
    }
}

impl Default for ArchitecturalIntentTracker {
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
    fn test_architectural_intent_tracker_creation() {
        let tracker = ArchitecturalIntentTracker::new();
        assert_eq!(tracker.confidence_threshold, 0.5);
    }

    #[test]
    fn test_architectural_intent_tracker_with_threshold() {
        let tracker = ArchitecturalIntentTracker::with_threshold(0.7);
        assert_eq!(tracker.confidence_threshold, 0.7);
    }

    #[test]
    fn test_threshold_clamping() {
        let tracker_low = ArchitecturalIntentTracker::with_threshold(-0.5);
        assert_eq!(tracker_low.confidence_threshold, 0.0);

        let tracker_high = ArchitecturalIntentTracker::with_threshold(1.5);
        assert_eq!(tracker_high.confidence_threshold, 1.0);
    }

    #[test]
    fn test_detect_layered_structure() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        // Create layered directories
        fs::create_dir(root.join("domain"))?;
        fs::create_dir(root.join("application"))?;
        fs::create_dir(root.join("infrastructure"))?;

        let tracker = ArchitecturalIntentTracker::new();
        let has_layered = tracker.has_layered_structure(root)?;
        assert!(has_layered);

        Ok(())
    }

    #[test]
    fn test_detect_microservices_structure() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        // Create services directories
        let services = root.join("services");
        fs::create_dir(&services)?;
        fs::create_dir(services.join("service1"))?;
        fs::create_dir(services.join("service2"))?;

        let tracker = ArchitecturalIntentTracker::new();
        let has_microservices = tracker.has_microservices_structure(root)?;
        assert!(has_microservices);

        Ok(())
    }

    #[test]
    fn test_detect_event_driven_structure() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        // Create event-driven directories
        fs::create_dir(root.join("events"))?;
        fs::create_dir(root.join("handlers"))?;

        let tracker = ArchitecturalIntentTracker::new();
        let has_event_driven = tracker.has_event_driven_structure(root)?;
        assert!(has_event_driven);

        Ok(())
    }

    #[test]
    fn test_parse_adr_sections() {
        let tracker = ArchitecturalIntentTracker::new();
        let content = r#"
# ADR-001: Use Rust

## Context
We need a performant language.

## Decision
We will use Rust for core components.

## Consequences
- Steep learning curve
- Better performance
"#;

        let (context, decision, consequences) = tracker.parse_adr_sections(content);
        
        assert!(context.contains("performant"));
        assert!(decision.contains("Rust"));
        assert_eq!(consequences.len(), 2);
    }

    #[test]
    fn test_extract_principles() {
        let decisions = vec![
            ArchitecturalDecision {
                id: "001".to_string(),
                title: "Test Decision".to_string(),
                context: "Following the principle of separation of concerns".to_string(),
                decision: "We will use layered architecture".to_string(),
                consequences: vec![],
                date: Utc::now(),
            },
        ];

        let tracker = ArchitecturalIntentTracker::new();
        let principles = tracker.extract_principles(&decisions);
        
        assert!(!principles.is_empty());
    }

    #[test]
    fn test_extract_constraints() {
        let decisions = vec![
            ArchitecturalDecision {
                id: "001".to_string(),
                title: "Test Decision".to_string(),
                context: "".to_string(),
                decision: "".to_string(),
                consequences: vec![
                    "Must use HTTPS for all communication".to_string(),
                    "Constraint: Maximum response time 100ms".to_string(),
                ],
                date: Utc::now(),
            },
        ];

        let tracker = ArchitecturalIntentTracker::new();
        let constraints = tracker.extract_constraints(&decisions);
        
        assert_eq!(constraints.len(), 2);
    }

    #[test]
    fn test_infer_style_layered() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        // Create layered architecture
        fs::create_dir(root.join("domain"))?;
        fs::create_dir(root.join("application"))?;

        let tracker = ArchitecturalIntentTracker::new();
        let style = tracker.infer_style(root)?;
        
        assert_eq!(style, ArchitecturalStyle::Layered);
        Ok(())
    }

    #[test]
    fn test_infer_style_microservices() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        // Create microservices structure
        let services = root.join("services");
        fs::create_dir(&services)?;
        fs::create_dir(services.join("auth-service"))?;
        fs::create_dir(services.join("api-service"))?;

        let tracker = ArchitecturalIntentTracker::new();
        let style = tracker.infer_style(root)?;
        
        assert_eq!(style, ArchitecturalStyle::Microservices);
        Ok(())
    }

    #[test]
    fn test_infer_style_event_driven() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        // Create event-driven structure
        fs::create_dir(root.join("events"))?;
        fs::create_dir(root.join("handlers"))?;

        let tracker = ArchitecturalIntentTracker::new();
        let style = tracker.infer_style(root)?;
        
        assert_eq!(style, ArchitecturalStyle::EventDriven);
        Ok(())
    }

    #[test]
    fn test_infer_style_serverless() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        // Create serverless.yml
        fs::write(root.join("serverless.yml"), "service: test")?;

        let tracker = ArchitecturalIntentTracker::new();
        let style = tracker.infer_style(root)?;
        
        assert_eq!(style, ArchitecturalStyle::Serverless);
        Ok(())
    }

    #[test]
    fn test_infer_style_monolithic() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        // Create a simple structure with no specific pattern
        fs::create_dir(root.join("src"))?;

        let tracker = ArchitecturalIntentTracker::new();
        let style = tracker.infer_style(root)?;
        
        assert_eq!(style, ArchitecturalStyle::Monolithic);
        Ok(())
    }

    #[test]
    fn test_build_intent() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        // Create layered structure
        fs::create_dir(root.join("domain"))?;
        fs::create_dir(root.join("application"))?;

        let tracker = ArchitecturalIntentTracker::new();
        let intent = tracker.build_intent(root)?;
        
        assert_eq!(intent.style, ArchitecturalStyle::Layered);
        assert!(intent.decisions.is_empty()); // No ADR files
        Ok(())
    }

    #[test]
    fn test_parse_adr_file() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        // Create ADR directory and file
        let adr_dir = root.join("docs/adr");
        fs::create_dir_all(&adr_dir)?;

        let adr_content = r#"# ADR-001: Use Rust

## Context
We need a performant language for core components.

## Decision
We will use Rust for all core infrastructure.

## Consequences
- Steep learning curve for team
- Better performance and memory safety
- Longer compilation times
"#;

        fs::write(adr_dir.join("0001-use-rust.md"), adr_content)?;

        let tracker = ArchitecturalIntentTracker::new();
        let decisions = tracker.parse_adrs(root)?;
        
        assert_eq!(decisions.len(), 1);
        assert_eq!(decisions[0].id, "0001");
        // Title is extracted from filename, replacing hyphens with spaces
        assert_eq!(decisions[0].title, "use rust");
        assert_eq!(decisions[0].consequences.len(), 3);
        Ok(())
    }

    #[test]
    fn test_parse_multiple_adrs() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        // Create ADR directory with multiple files
        let adr_dir = root.join("adr");
        fs::create_dir(&adr_dir)?;

        fs::write(adr_dir.join("0001-use-rust.md"), "# ADR-001\n## Context\nTest\n## Decision\nUse Rust\n## Consequences\n- Good")?;
        fs::write(adr_dir.join("0002-use-postgres.md"), "# ADR-002\n## Context\nDatabase\n## Decision\nUse PostgreSQL\n## Consequences\n- Reliable")?;

        let tracker = ArchitecturalIntentTracker::new();
        let decisions = tracker.parse_adrs(root)?;
        
        assert_eq!(decisions.len(), 2);
        Ok(())
    }

    #[test]
    fn test_default_tracker() {
        let tracker = ArchitecturalIntentTracker::default();
        assert_eq!(tracker.confidence_threshold, 0.5);
    }

    #[test]
    fn test_parse_adr_sections_with_markdown_variations() {
        let tracker = ArchitecturalIntentTracker::new();
        
        // Test with different markdown heading levels
        let content = r#"
# ADR-001

# Context
This is the context section.

# Decision
This is the decision section.

# Consequences
- Consequence 1
- Consequence 2
"#;

        let (context, decision, consequences) = tracker.parse_adr_sections(content);
        
        assert!(context.contains("context section"));
        assert!(decision.contains("decision section"));
        assert_eq!(consequences.len(), 2);
    }

    #[test]
    fn test_parse_adr_sections_empty_content() {
        let tracker = ArchitecturalIntentTracker::new();
        let content = "";

        let (context, decision, consequences) = tracker.parse_adr_sections(content);
        
        assert!(context.is_empty());
        assert!(decision.is_empty());
        assert!(consequences.is_empty());
    }

    #[test]
    fn test_extract_principles_deduplication() {
        let decisions = vec![
            ArchitecturalDecision {
                id: "001".to_string(),
                title: "Test".to_string(),
                context: "Following the principle of separation of concerns".to_string(),
                decision: "".to_string(),
                consequences: vec![],
                date: Utc::now(),
            },
            ArchitecturalDecision {
                id: "002".to_string(),
                title: "Test".to_string(),
                context: "Following the principle of separation of concerns".to_string(),
                decision: "".to_string(),
                consequences: vec![],
                date: Utc::now(),
            },
        ];

        let tracker = ArchitecturalIntentTracker::new();
        let principles = tracker.extract_principles(&decisions);
        
        // Should be deduplicated
        assert_eq!(principles.len(), 1);
    }

    #[test]
    fn test_extract_constraints_deduplication() {
        let decisions = vec![
            ArchitecturalDecision {
                id: "001".to_string(),
                title: "Test".to_string(),
                context: "".to_string(),
                decision: "".to_string(),
                consequences: vec![
                    "Must use HTTPS".to_string(),
                    "Must use HTTPS".to_string(),
                ],
                date: Utc::now(),
            },
        ];

        let tracker = ArchitecturalIntentTracker::new();
        let constraints = tracker.extract_constraints(&decisions);
        
        // Should be deduplicated
        assert_eq!(constraints.len(), 1);
    }
}
