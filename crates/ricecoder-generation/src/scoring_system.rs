//! Scoring system for code quality and spec compliance
//!
//! Provides detailed scoring mechanisms for evaluating code quality metrics,
//! spec compliance, and generating actionable feedback based on scores.

use crate::error::GenerationError;
use crate::models::GeneratedFile;
use ricecoder_specs::models::Spec;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Detailed score breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreBreakdown {
    /// Documentation score component
    pub documentation: ScoreComponent,
    /// Error handling score component
    pub error_handling: ScoreComponent,
    /// Code style score component
    pub style: ScoreComponent,
    /// Test coverage score component
    pub coverage: ScoreComponent,
    /// Complexity score component
    pub complexity: ScoreComponent,
    /// Naming conventions score component
    pub naming: ScoreComponent,
}

/// A single score component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreComponent {
    /// Component name
    pub name: String,
    /// Score value (0.0 to 1.0)
    pub score: f32,
    /// Weight in overall calculation (0.0 to 1.0)
    pub weight: f32,
    /// Feedback message
    pub feedback: String,
    /// Specific issues found
    pub issues: Vec<String>,
}

impl ScoreComponent {
    /// Creates a new score component
    pub fn new(name: &str, score: f32, weight: f32) -> Self {
        Self {
            name: name.to_string(),
            score: score.clamp(0.0, 1.0),
            weight: weight.clamp(0.0, 1.0),
            feedback: String::new(),
            issues: Vec::new(),
        }
    }

    /// Adds feedback to the component
    pub fn with_feedback(mut self, feedback: &str) -> Self {
        self.feedback = feedback.to_string();
        self
    }

    /// Adds an issue to the component
    pub fn add_issue(&mut self, issue: String) {
        self.issues.push(issue);
    }
}

/// Spec compliance score breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceScore {
    /// Overall compliance score (0.0 to 1.0)
    pub overall: f32,
    /// Requirement coverage score
    pub requirement_coverage: f32,
    /// Acceptance criteria coverage score
    pub criteria_coverage: f32,
    /// Constraint adherence score
    pub constraint_adherence: f32,
    /// Detailed breakdown by requirement
    pub requirement_scores: HashMap<String, f32>,
}

impl Default for ComplianceScore {
    fn default() -> Self {
        Self {
            overall: 0.0,
            requirement_coverage: 0.0,
            criteria_coverage: 0.0,
            constraint_adherence: 0.0,
            requirement_scores: HashMap::new(),
        }
    }
}

/// Actionable feedback based on scores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringFeedback {
    /// Overall assessment
    pub assessment: String,
    /// Strengths identified
    pub strengths: Vec<String>,
    /// Areas for improvement
    pub improvements: Vec<String>,
    /// Critical issues that must be addressed
    pub critical_issues: Vec<String>,
    /// Recommended next steps
    pub next_steps: Vec<String>,
}

/// Scoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringConfig {
    /// Weight for documentation score
    pub documentation_weight: f32,
    /// Weight for error handling score
    pub error_handling_weight: f32,
    /// Weight for style score
    pub style_weight: f32,
    /// Weight for coverage score
    pub coverage_weight: f32,
    /// Weight for complexity score
    pub complexity_weight: f32,
    /// Weight for naming conventions score
    pub naming_weight: f32,
    /// Threshold for critical issues
    pub critical_threshold: f32,
    /// Threshold for warnings
    pub warning_threshold: f32,
}

impl Default for ScoringConfig {
    fn default() -> Self {
        Self {
            documentation_weight: 0.20,
            error_handling_weight: 0.20,
            style_weight: 0.15,
            coverage_weight: 0.20,
            complexity_weight: 0.15,
            naming_weight: 0.10,
            critical_threshold: 0.3,
            warning_threshold: 0.6,
        }
    }
}

/// Scoring system for code quality and compliance
#[derive(Debug, Clone)]
pub struct ScoringSystem {
    config: ScoringConfig,
}

impl ScoringSystem {
    /// Creates a new scoring system with default configuration
    pub fn new() -> Self {
        Self {
            config: ScoringConfig::default(),
        }
    }

    /// Creates a new scoring system with custom configuration
    pub fn with_config(config: ScoringConfig) -> Self {
        Self { config }
    }

    /// Scores code quality with detailed breakdown
    pub fn score_quality(
        &self,
        files: &[GeneratedFile],
    ) -> Result<ScoreBreakdown, GenerationError> {
        let documentation = self.score_documentation(files)?;
        let error_handling = self.score_error_handling(files)?;
        let style = self.score_style(files)?;
        let coverage = self.score_coverage(files)?;
        let complexity = self.score_complexity(files)?;
        let naming = self.score_naming(files)?;

        Ok(ScoreBreakdown {
            documentation,
            error_handling,
            style,
            coverage,
            complexity,
            naming,
        })
    }

    /// Scores spec compliance
    pub fn score_compliance(
        &self,
        files: &[GeneratedFile],
        spec: &Spec,
    ) -> Result<ComplianceScore, GenerationError> {
        let combined_content = files.iter().map(|f| f.content.as_str()).collect::<Vec<_>>().join("\n");

        let mut requirement_scores = HashMap::new();
        let mut total_coverage = 0.0;
        let mut criteria_met = 0;
        let mut total_criteria = 0;

        for requirement in &spec.requirements {
            let mut req_coverage = 0.0;

            // Check if requirement is addressed
            if combined_content.contains(&requirement.id) || combined_content.contains(&requirement.user_story) {
                req_coverage = 0.5;
            }

            // Check acceptance criteria
            let mut criteria_count = 0;
            for criterion in &requirement.acceptance_criteria {
                total_criteria += 1;
                let criterion_text = format!("{} {}", criterion.when, criterion.then);
                if combined_content.contains(&criterion_text) {
                    criteria_met += 1;
                    criteria_count += 1;
                }
            }

            if !requirement.acceptance_criteria.is_empty() {
                let criteria_ratio = criteria_count as f32 / requirement.acceptance_criteria.len() as f32;
                req_coverage = (req_coverage + criteria_ratio) / 2.0;
            }

            requirement_scores.insert(requirement.id.clone(), req_coverage);
            total_coverage += req_coverage;
        }

        let requirement_coverage = if !spec.requirements.is_empty() {
            total_coverage / spec.requirements.len() as f32
        } else {
            0.0
        };

        let criteria_coverage = if total_criteria > 0 {
            criteria_met as f32 / total_criteria as f32
        } else {
            0.0
        };

        let constraint_adherence = self.score_constraint_adherence(files)?;

        let overall = (requirement_coverage * 0.4) + (criteria_coverage * 0.4) + (constraint_adherence * 0.2);

        Ok(ComplianceScore {
            overall,
            requirement_coverage,
            criteria_coverage,
            constraint_adherence,
            requirement_scores,
        })
    }

    /// Generates actionable feedback based on scores
    pub fn generate_feedback(
        &self,
        quality_breakdown: &ScoreBreakdown,
        compliance_score: &ComplianceScore,
    ) -> ScoringFeedback {
        let mut strengths = Vec::new();
        let mut improvements = Vec::new();
        let mut critical_issues = Vec::new();
        let mut next_steps = Vec::new();

        // Analyze quality components
        if quality_breakdown.documentation.score > 0.8 {
            strengths.push("Excellent documentation coverage".to_string());
        } else if quality_breakdown.documentation.score < self.config.critical_threshold {
            critical_issues.push("Documentation is severely lacking".to_string());
        } else if quality_breakdown.documentation.score < self.config.warning_threshold {
            improvements.push("Improve documentation coverage".to_string());
        }

        if quality_breakdown.error_handling.score > 0.8 {
            strengths.push("Strong error handling implementation".to_string());
        } else if quality_breakdown.error_handling.score < self.config.critical_threshold {
            critical_issues.push("Error handling is insufficient".to_string());
        } else if quality_breakdown.error_handling.score < self.config.warning_threshold {
            improvements.push("Enhance error handling".to_string());
        }

        if quality_breakdown.style.score > 0.8 {
            strengths.push("Code style is consistent and clean".to_string());
        } else if quality_breakdown.style.score < self.config.warning_threshold {
            improvements.push("Improve code style consistency".to_string());
        }

        if quality_breakdown.coverage.score > 0.7 {
            strengths.push("Good test coverage".to_string());
        } else if quality_breakdown.coverage.score < self.config.warning_threshold {
            improvements.push("Increase test coverage".to_string());
        }

        if quality_breakdown.complexity.score > 0.7 {
            strengths.push("Functions have reasonable complexity".to_string());
        } else if quality_breakdown.complexity.score < self.config.warning_threshold {
            improvements.push("Reduce function complexity".to_string());
        }

        if quality_breakdown.naming.score > 0.8 {
            strengths.push("Naming conventions are well-followed".to_string());
        } else if quality_breakdown.naming.score < self.config.warning_threshold {
            improvements.push("Improve naming consistency".to_string());
        }

        // Analyze compliance
        if compliance_score.overall > 0.9 {
            strengths.push("Excellent spec compliance".to_string());
        } else if compliance_score.overall < self.config.critical_threshold {
            critical_issues.push("Spec compliance is critically low".to_string());
        } else if compliance_score.overall < self.config.warning_threshold {
            improvements.push("Improve spec compliance".to_string());
        }

        // Generate assessment
        let assessment = if critical_issues.is_empty() {
            if improvements.is_empty() {
                "Code quality and compliance are excellent".to_string()
            } else {
                "Code quality is good with some areas for improvement".to_string()
            }
        } else {
            "Code quality and compliance need significant improvement".to_string()
        };

        // Generate next steps
        if !critical_issues.is_empty() {
            next_steps.push("Address critical issues immediately".to_string());
        }
        if !improvements.is_empty() {
            next_steps.push("Implement suggested improvements".to_string());
        }
        if compliance_score.overall < 0.8 {
            next_steps.push("Review spec requirements and ensure all are addressed".to_string());
        }
        if quality_breakdown.coverage.score < 0.6 {
            next_steps.push("Add more comprehensive tests".to_string());
        }

        ScoringFeedback {
            assessment,
            strengths,
            improvements,
            critical_issues,
            next_steps,
        }
    }

    /// Scores documentation
    fn score_documentation(&self, files: &[GeneratedFile]) -> Result<ScoreComponent, GenerationError> {
        let mut component = ScoreComponent::new("Documentation", 0.0, self.config.documentation_weight);

        let mut total_lines = 0;
        let mut doc_lines = 0;
        let mut public_items = 0;
        let mut documented_items = 0;

        for file in files {
            let lines: Vec<&str> = file.content.lines().collect();
            total_lines += lines.len();

            for (idx, line) in lines.iter().enumerate() {
                let trimmed = line.trim();

                // Count documentation lines
                if trimmed.starts_with("///") || trimmed.starts_with("//!") || trimmed.starts_with("/**") {
                    doc_lines += 1;
                }

                // Count public items
                if trimmed.starts_with("pub ") {
                    public_items += 1;
                    // Check if documented
                    if idx > 0 && (lines[idx - 1].trim().starts_with("///") || lines[idx - 1].trim().starts_with("//!")) {
                        documented_items += 1;
                    }
                }
            }
        }

        let doc_ratio = if total_lines > 0 {
            doc_lines as f32 / total_lines as f32
        } else {
            0.0
        };

        let public_doc_ratio = if public_items > 0 {
            documented_items as f32 / public_items as f32
        } else {
            1.0
        };

        component.score = (doc_ratio * 0.5 + public_doc_ratio * 0.5).clamp(0.0, 1.0);

        if component.score > 0.8 {
            component.feedback = "Documentation is comprehensive and well-maintained".to_string();
        } else if component.score > 0.6 {
            component.feedback = "Documentation is adequate but could be improved".to_string();
            component.add_issue("Some public items lack documentation".to_string());
        } else {
            component.feedback = "Documentation coverage is insufficient".to_string();
            component.add_issue(format!("Only {:.0}% of public items are documented", public_doc_ratio * 100.0));
        }

        Ok(component)
    }

    /// Scores error handling
    fn score_error_handling(&self, files: &[GeneratedFile]) -> Result<ScoreComponent, GenerationError> {
        let mut component = ScoreComponent::new("Error Handling", 0.0, self.config.error_handling_weight);

        let mut total_lines = 0;
        let mut error_lines = 0;

        for file in files {
            let content = &file.content;
            let language = &file.language;

            total_lines += content.lines().count();

            let error_patterns = match language.to_lowercase().as_str() {
                "rust" => vec!["Result<", "?", "unwrap", "expect", "match"],
                "typescript" | "javascript" => vec!["try", "catch", "throw", "Error"],
                "python" => vec!["try", "except", "raise"],
                "go" => vec!["if err != nil", "error"],
                "java" => vec!["try", "catch", "throw", "Exception"],
                _ => vec![],
            };

            for pattern in error_patterns {
                error_lines += content.matches(pattern).count();
            }
        }

        component.score = if total_lines > 0 {
            (error_lines as f32 / total_lines as f32).clamp(0.0, 1.0)
        } else {
            0.0
        };

        if component.score > 0.8 {
            component.feedback = "Error handling is comprehensive".to_string();
        } else if component.score > 0.5 {
            component.feedback = "Error handling is present but could be more thorough".to_string();
            component.add_issue("Some operations may lack proper error handling".to_string());
        } else {
            component.feedback = "Error handling is minimal or missing".to_string();
            component.add_issue("Add error handling for fallible operations".to_string());
        }

        Ok(component)
    }

    /// Scores code style
    fn score_style(&self, files: &[GeneratedFile]) -> Result<ScoreComponent, GenerationError> {
        let mut component = ScoreComponent::new("Style", 1.0, self.config.style_weight);

        for file in files {
            let lines: Vec<&str> = file.content.lines().collect();

            // Check for trailing whitespace
            let trailing_ws = lines.iter().filter(|l| l.ends_with(' ') || l.ends_with('\t')).count();
            if trailing_ws > 0 {
                component.score -= 0.05;
                component.add_issue(format!("{} lines have trailing whitespace", trailing_ws));
            }

            // Check for inconsistent indentation
            let mut indent_styles = std::collections::HashMap::new();
            for line in &lines {
                if line.starts_with(' ') {
                    let spaces = line.len() - line.trim_start().len();
                    *indent_styles.entry(spaces % 4).or_insert(0) += 1;
                }
            }

            if indent_styles.len() > 2 {
                component.score -= 0.1;
                component.add_issue("Inconsistent indentation detected".to_string());
            }

            // Check for long lines
            let long_lines = lines.iter().filter(|l| l.len() > 100).count();
            if long_lines > 0 {
                component.score -= 0.05;
                component.add_issue(format!("{} lines exceed 100 characters", long_lines));
            }
        }

        component.score = component.score.clamp(0.0, 1.0);

        if component.score > 0.9 {
            component.feedback = "Code style is excellent".to_string();
        } else if component.score > 0.7 {
            component.feedback = "Code style is generally good".to_string();
        } else {
            component.feedback = "Code style needs improvement".to_string();
        }

        Ok(component)
    }

    /// Scores test coverage
    fn score_coverage(&self, files: &[GeneratedFile]) -> Result<ScoreComponent, GenerationError> {
        let mut component = ScoreComponent::new("Coverage", 0.0, self.config.coverage_weight);

        let has_tests = files.iter().any(|f| {
            f.path.contains("test") || f.path.contains("spec") || f.path.ends_with("_test.rs")
        });

        if has_tests {
            component.score = 0.6;
            component.feedback = "Test files are present".to_string();
        } else {
            component.score = 0.2;
            component.feedback = "No test files detected".to_string();
            component.add_issue("Add unit tests for public functions".to_string());
        }

        Ok(component)
    }

    /// Scores complexity
    fn score_complexity(&self, files: &[GeneratedFile]) -> Result<ScoreComponent, GenerationError> {
        let mut component = ScoreComponent::new("Complexity", 1.0, self.config.complexity_weight);

        for file in files {
            let content = &file.content;

            // Estimate complexity by counting nested structures
            let mut max_nesting: i32 = 0;
            let mut current_nesting: i32 = 0;

            for ch in content.chars() {
                match ch {
                    '{' | '[' | '(' => {
                        current_nesting += 1;
                        max_nesting = max_nesting.max(current_nesting);
                    }
                    '}' | ']' | ')' => {
                        current_nesting = current_nesting.saturating_sub(1);
                    }
                    _ => {}
                }
            }

            if max_nesting > 5 {
                component.score -= 0.1;
                component.add_issue(format!("High nesting depth detected: {}", max_nesting));
            }
        }

        component.score = component.score.clamp(0.0, 1.0);

        if component.score > 0.8 {
            component.feedback = "Functions have reasonable complexity".to_string();
        } else {
            component.feedback = "Some functions may be too complex".to_string();
        }

        Ok(component)
    }

    /// Scores naming conventions
    fn score_naming(&self, files: &[GeneratedFile]) -> Result<ScoreComponent, GenerationError> {
        let mut component = ScoreComponent::new("Naming", 1.0, self.config.naming_weight);

        for file in files {
            let language = &file.language;
            let content = &file.content;

            // Check naming conventions based on language
            match language.to_lowercase().as_str() {
                "rust" => {
                    // Check for snake_case functions
                    let snake_case_violations = content.matches("fn [A-Z]").count();
                    if snake_case_violations > 0 {
                        component.score -= 0.1;
                        component.add_issue("Rust functions should use snake_case".to_string());
                    }
                }
                "typescript" | "javascript" => {
                    // Check for camelCase variables
                    let violations = content.matches("const [a-z]_[a-z]").count();
                    if violations > 0 {
                        component.score -= 0.1;
                        component.add_issue("TypeScript variables should use camelCase".to_string());
                    }
                }
                "python" => {
                    // Check for snake_case functions
                    let violations = content.matches("def [A-Z]").count();
                    if violations > 0 {
                        component.score -= 0.1;
                        component.add_issue("Python functions should use snake_case".to_string());
                    }
                }
                _ => {}
            }
        }

        component.score = component.score.clamp(0.0, 1.0);

        if component.score > 0.9 {
            component.feedback = "Naming conventions are well-followed".to_string();
        } else if component.score > 0.7 {
            component.feedback = "Naming conventions are mostly followed".to_string();
        } else {
            component.feedback = "Naming conventions need improvement".to_string();
        }

        Ok(component)
    }

    /// Scores constraint adherence
    fn score_constraint_adherence(&self, _files: &[GeneratedFile]) -> Result<f32, GenerationError> {
        // Simplified constraint adherence scoring
        // In a full implementation, this would check against specific constraints
        Ok(0.8)
    }
}

impl Default for ScoringSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scoring_system_creation() {
        let system = ScoringSystem::new();
        assert_eq!(system.config.documentation_weight, 0.20);
    }

    #[test]
    fn test_score_component_creation() {
        let component = ScoreComponent::new("Test", 0.8, 0.5);
        assert_eq!(component.name, "Test");
        assert_eq!(component.score, 0.8);
        assert_eq!(component.weight, 0.5);
    }

    #[test]
    fn test_score_component_clamping() {
        let component = ScoreComponent::new("Test", 1.5, 1.5);
        assert_eq!(component.score, 1.0);
        assert_eq!(component.weight, 1.0);
    }

    #[test]
    fn test_compliance_score_default() {
        let score = ComplianceScore::default();
        assert_eq!(score.overall, 0.0);
        assert_eq!(score.requirement_coverage, 0.0);
    }
}
