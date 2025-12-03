//! Review engine for generated code
//!
//! Provides code review capabilities that check generated code against spec requirements,
//! measure code quality metrics, and provide actionable feedback and suggestions.

use crate::error::GenerationError;
use crate::models::GeneratedFile;
use ricecoder_specs::models::Spec;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Result of code review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResult {
    /// Overall quality score (0.0 to 1.0)
    pub quality_score: f32,
    /// Spec compliance score (0.0 to 1.0)
    pub compliance_score: f32,
    /// Overall review score (0.0 to 1.0)
    pub overall_score: f32,
    /// Code quality metrics
    pub quality_metrics: CodeQualityMetrics,
    /// Spec compliance details
    pub compliance_details: ComplianceDetails,
    /// Suggestions for improvement
    pub suggestions: Vec<Suggestion>,
    /// Issues found during review
    pub issues: Vec<ReviewIssue>,
    /// Summary of the review
    pub summary: String,
}

/// Code quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQualityMetrics {
    /// Average cyclomatic complexity
    pub avg_complexity: f32,
    /// Estimated test coverage percentage
    pub estimated_coverage: f32,
    /// Code style score (0.0 to 1.0)
    pub style_score: f32,
    /// Documentation score (0.0 to 1.0)
    pub documentation_score: f32,
    /// Error handling score (0.0 to 1.0)
    pub error_handling_score: f32,
    /// Total lines of code
    pub total_lines: usize,
    /// Lines of comments
    pub comment_lines: usize,
    /// Number of functions/methods
    pub function_count: usize,
    /// Number of public functions/methods
    pub public_function_count: usize,
}

impl Default for CodeQualityMetrics {
    fn default() -> Self {
        Self {
            avg_complexity: 0.0,
            estimated_coverage: 0.0,
            style_score: 1.0,
            documentation_score: 0.0,
            error_handling_score: 0.0,
            total_lines: 0,
            comment_lines: 0,
            function_count: 0,
            public_function_count: 0,
        }
    }
}

/// Spec compliance details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceDetails {
    /// Total requirements in spec
    pub total_requirements: usize,
    /// Requirements addressed by generated code
    pub addressed_requirements: usize,
    /// Acceptance criteria coverage
    pub criteria_coverage: f32,
    /// Requirements not addressed
    pub unaddressed_requirements: Vec<String>,
    /// Acceptance criteria not met
    pub unmet_criteria: Vec<String>,
}

impl Default for ComplianceDetails {
    fn default() -> Self {
        Self {
            total_requirements: 0,
            addressed_requirements: 0,
            criteria_coverage: 0.0,
            unaddressed_requirements: Vec::new(),
            unmet_criteria: Vec::new(),
        }
    }
}

/// A suggestion for improvement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    /// Suggestion category
    pub category: SuggestionCategory,
    /// File path this suggestion applies to
    pub file: Option<String>,
    /// Line number this suggestion applies to
    pub line: Option<usize>,
    /// Suggestion message
    pub message: String,
    /// Suggested fix or action
    pub action: String,
    /// Priority of this suggestion (1-5, 5 being highest)
    pub priority: u8,
}

/// Categories of suggestions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SuggestionCategory {
    /// Code quality improvement
    CodeQuality,
    /// Documentation improvement
    Documentation,
    /// Error handling improvement
    ErrorHandling,
    /// Testing improvement
    Testing,
    /// Performance improvement
    Performance,
    /// Security improvement
    Security,
    /// Spec compliance improvement
    SpecCompliance,
}

/// An issue found during review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewIssue {
    /// Issue severity
    pub severity: IssueSeverity,
    /// File path where issue was found
    pub file: String,
    /// Line number where issue was found
    pub line: Option<usize>,
    /// Issue message
    pub message: String,
    /// Issue code (e.g., "REVIEW-001")
    pub code: String,
}

/// Severity levels for review issues
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueSeverity {
    /// Critical issue that must be fixed
    Critical,
    /// Major issue that should be fixed
    Major,
    /// Minor issue that could be improved
    Minor,
    /// Informational issue
    Info,
}

/// Configuration for code review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewConfig {
    /// Whether to check code quality metrics
    pub check_quality: bool,
    /// Whether to check spec compliance
    pub check_compliance: bool,
    /// Whether to generate suggestions
    pub generate_suggestions: bool,
    /// Minimum quality score threshold (0.0 to 1.0)
    pub min_quality_score: f32,
    /// Minimum compliance score threshold (0.0 to 1.0)
    pub min_compliance_score: f32,
}

impl Default for ReviewConfig {
    fn default() -> Self {
        Self {
            check_quality: true,
            check_compliance: true,
            generate_suggestions: true,
            min_quality_score: 0.6,
            min_compliance_score: 0.8,
        }
    }
}

/// Review engine for generated code
#[derive(Debug, Clone)]
pub struct ReviewEngine {
    config: ReviewConfig,
}

impl ReviewEngine {
    /// Creates a new ReviewEngine with default configuration
    pub fn new() -> Self {
        Self {
            config: ReviewConfig::default(),
        }
    }

    /// Creates a new ReviewEngine with custom configuration
    pub fn with_config(config: ReviewConfig) -> Self {
        Self { config }
    }

    /// Reviews generated code against spec requirements
    ///
    /// # Arguments
    ///
    /// * `files` - Generated code files to review
    /// * `spec` - Specification to review against
    ///
    /// # Returns
    ///
    /// A review result with scores, metrics, and suggestions
    ///
    /// # Errors
    ///
    /// Returns an error if review cannot be completed
    pub fn review(
        &self,
        files: &[GeneratedFile],
        spec: &Spec,
    ) -> Result<ReviewResult, GenerationError> {
        // Calculate code quality metrics
        let quality_metrics = if self.config.check_quality {
            self.calculate_quality_metrics(files)?
        } else {
            CodeQualityMetrics::default()
        };

        // Check spec compliance
        let compliance_details = if self.config.check_compliance {
            self.check_compliance(files, spec)?
        } else {
            ComplianceDetails::default()
        };

        // Generate suggestions
        let suggestions = if self.config.generate_suggestions {
            self.generate_suggestions(files, spec, &quality_metrics, &compliance_details)?
        } else {
            Vec::new()
        };

        // Calculate scores
        let quality_score = self.calculate_quality_score(&quality_metrics);
        let compliance_score = compliance_details.criteria_coverage;
        let overall_score = (quality_score * 0.4) + (compliance_score * 0.6);

        // Find issues
        let issues = self.find_issues(files, spec)?;

        // Generate summary
        let summary = self.generate_summary(
            &quality_metrics,
            &compliance_details,
            quality_score,
            compliance_score,
        );

        Ok(ReviewResult {
            quality_score,
            compliance_score,
            overall_score,
            quality_metrics,
            compliance_details,
            suggestions,
            issues,
            summary,
        })
    }

    /// Calculates code quality metrics for generated files
    fn calculate_quality_metrics(
        &self,
        files: &[GeneratedFile],
    ) -> Result<CodeQualityMetrics, GenerationError> {
        let mut metrics = CodeQualityMetrics::default();

        for file in files {
            let lines: Vec<&str> = file.content.lines().collect();
            metrics.total_lines += lines.len();

            // Count comment lines
            for line in &lines {
                let trimmed = line.trim();
                if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*") {
                    metrics.comment_lines += 1;
                }
            }

            // Estimate function count and complexity
            let function_count = self.count_functions(&file.content, &file.language);
            metrics.function_count += function_count;

            // Estimate public function count
            let public_count = self.count_public_functions(&file.content, &file.language);
            metrics.public_function_count += public_count;
        }

        // Calculate documentation score
        if metrics.total_lines > 0 {
            metrics.documentation_score = (metrics.comment_lines as f32 / metrics.total_lines as f32).min(1.0);
        }

        // Estimate coverage based on test presence
        metrics.estimated_coverage = self.estimate_coverage(files);

        // Calculate style score
        metrics.style_score = self.calculate_style_score(files)?;

        // Calculate error handling score
        metrics.error_handling_score = self.calculate_error_handling_score(files)?;

        // Estimate average complexity
        if metrics.function_count > 0 {
            metrics.avg_complexity = 1.5; // Simplified estimate
        }

        Ok(metrics)
    }

    /// Counts functions in code
    fn count_functions(&self, content: &str, language: &str) -> usize {
        match language.to_lowercase().as_str() {
            "rust" => content.matches("fn ").count(),
            "typescript" | "javascript" => {
                content.matches("function ").count() + content.matches("=>").count()
            }
            "python" => content.matches("def ").count(),
            "go" => content.matches("func ").count(),
            "java" => content.matches("public ").count() + content.matches("private ").count(),
            _ => 0,
        }
    }

    /// Counts public functions in code
    fn count_public_functions(&self, content: &str, language: &str) -> usize {
        match language.to_lowercase().as_str() {
            "rust" => content.matches("pub fn ").count(),
            "typescript" | "javascript" => content.matches("export ").count(),
            "python" => {
                // In Python, functions not starting with _ are public
                content.lines().filter(|l| l.trim().starts_with("def ") && !l.contains("_")).count()
            }
            "go" => {
                // In Go, functions starting with uppercase are public
                content.matches("func (").count() + content.matches("func [A-Z]").count()
            }
            "java" => content.matches("public ").count(),
            _ => 0,
        }
    }

    /// Estimates test coverage based on test file presence
    fn estimate_coverage(&self, files: &[GeneratedFile]) -> f32 {
        let has_tests = files.iter().any(|f| {
            f.path.contains("test") || f.path.contains("spec") || f.path.ends_with("_test.rs")
        });

        if has_tests {
            0.6 // Estimate 60% coverage if tests are present
        } else {
            0.2 // Estimate 20% coverage if no tests
        }
    }

    /// Calculates style score
    fn calculate_style_score(&self, files: &[GeneratedFile]) -> Result<f32, GenerationError> {
        let mut score: f32 = 1.0;

        for file in files {
            // Check for consistent indentation
            let lines: Vec<&str> = file.content.lines().collect();
            let mut indent_styles = HashMap::new();

            for line in &lines {
                if line.starts_with(' ') {
                    let spaces = line.len() - line.trim_start().len();
                    *indent_styles.entry(spaces % 4).or_insert(0) += 1;
                }
            }

            // If multiple indentation styles, reduce score
            if indent_styles.len() > 1 {
                score -= 0.1;
            }

            // Check for trailing whitespace
            let trailing_ws = lines.iter().filter(|l| l.ends_with(' ') || l.ends_with('\t')).count();
            if trailing_ws > 0 {
                score -= 0.05;
            }
        }

        Ok(score.max(0.0))
    }

    /// Calculates error handling score
    fn calculate_error_handling_score(&self, files: &[GeneratedFile]) -> Result<f32, GenerationError> {
        let mut total_score = 0.0;
        let mut file_count = 0;

        for file in files {
            let content = &file.content;
            let language = &file.language;

            let error_patterns = match language.to_lowercase().as_str() {
                "rust" => vec!["Result<", "?", "unwrap", "expect"],
                "typescript" | "javascript" => vec!["try", "catch", "throw", "Error"],
                "python" => vec!["try", "except", "raise"],
                "go" => vec!["if err != nil", "error"],
                "java" => vec!["try", "catch", "throw", "Exception"],
                _ => vec![],
            };

            let error_count = error_patterns.iter().map(|p| content.matches(p).count()).sum::<usize>();
            let lines = content.lines().count();

            let score = if lines > 0 {
                (error_count as f32 / lines as f32).min(1.0)
            } else {
                0.0
            };

            total_score += score;
            file_count += 1;
        }

        if file_count > 0 {
            Ok(total_score / file_count as f32)
        } else {
            Ok(0.0)
        }
    }

    /// Checks spec compliance
    fn check_compliance(
        &self,
        files: &[GeneratedFile],
        spec: &Spec,
    ) -> Result<ComplianceDetails, GenerationError> {
        let mut details = ComplianceDetails {
            total_requirements: spec.requirements.len(),
            ..Default::default()
        };

        // Check if requirements are addressed in generated code
        let combined_content = files.iter().map(|f| f.content.as_str()).collect::<Vec<_>>().join("\n");

        for requirement in &spec.requirements {
            let mut requirement_addressed = false;

            // Check if requirement ID or keywords appear in code
            if combined_content.contains(&requirement.id) || combined_content.contains(&requirement.user_story) {
                requirement_addressed = true;
                details.addressed_requirements += 1;
            }

            if !requirement_addressed {
                details.unaddressed_requirements.push(requirement.id.clone());
            }

            // Check acceptance criteria
            for criterion in &requirement.acceptance_criteria {
                let criterion_text = format!("{} {}", criterion.when, criterion.then);
                if !combined_content.contains(&criterion_text) {
                    details.unmet_criteria.push(criterion_text);
                }
            }
        }

        // Calculate coverage
        if details.total_requirements > 0 {
            details.criteria_coverage = (details.addressed_requirements as f32 / details.total_requirements as f32).min(1.0);
        }

        Ok(details)
    }

    /// Generates suggestions for improvement
    fn generate_suggestions(
        &self,
        _files: &[GeneratedFile],
        _spec: &Spec,
        metrics: &CodeQualityMetrics,
        compliance: &ComplianceDetails,
    ) -> Result<Vec<Suggestion>, GenerationError> {
        let mut suggestions = Vec::new();

        // Suggest improvements based on metrics
        if metrics.documentation_score < 0.5 {
            suggestions.push(Suggestion {
                category: SuggestionCategory::Documentation,
                file: None,
                line: None,
                message: "Code documentation is below recommended level".to_string(),
                action: "Add doc comments to public functions and types".to_string(),
                priority: 4,
            });
        }

        if metrics.error_handling_score < 0.5 {
            suggestions.push(Suggestion {
                category: SuggestionCategory::ErrorHandling,
                file: None,
                line: None,
                message: "Error handling coverage is low".to_string(),
                action: "Add error handling for fallible operations".to_string(),
                priority: 4,
            });
        }

        if metrics.estimated_coverage < 0.5 {
            suggestions.push(Suggestion {
                category: SuggestionCategory::Testing,
                file: None,
                line: None,
                message: "Test coverage is estimated to be low".to_string(),
                action: "Add unit tests for public functions".to_string(),
                priority: 3,
            });
        }

        // Suggest improvements based on compliance
        if compliance.criteria_coverage < 0.8 {
            suggestions.push(Suggestion {
                category: SuggestionCategory::SpecCompliance,
                file: None,
                line: None,
                message: format!("Only {:.0}% of spec requirements are addressed", compliance.criteria_coverage * 100.0),
                action: "Review unaddressed requirements and implement missing functionality".to_string(),
                priority: 5,
            });
        }

        // Suggest code quality improvements
        if metrics.avg_complexity > 5.0 {
            suggestions.push(Suggestion {
                category: SuggestionCategory::CodeQuality,
                file: None,
                line: None,
                message: "Average function complexity is high".to_string(),
                action: "Consider breaking down complex functions into smaller, more focused functions".to_string(),
                priority: 3,
            });
        }

        Ok(suggestions)
    }

    /// Finds issues in generated code
    fn find_issues(
        &self,
        files: &[GeneratedFile],
        _spec: &Spec,
    ) -> Result<Vec<ReviewIssue>, GenerationError> {
        let mut issues = Vec::new();

        // Check for missing public documentation
        for file in files {
            let lines: Vec<&str> = file.content.lines().collect();
            for (idx, line) in lines.iter().enumerate() {
                let trimmed = line.trim();

                // Check for public functions without doc comments
                if trimmed.starts_with("pub fn ") || trimmed.starts_with("pub struct ") || trimmed.starts_with("pub enum ") {
                    // Check if previous line is a doc comment
                    if idx == 0 || !lines[idx - 1].trim().starts_with("///") {
                        issues.push(ReviewIssue {
                            severity: IssueSeverity::Minor,
                            file: file.path.clone(),
                            line: Some(idx + 1),
                            message: "Public item missing documentation comment".to_string(),
                            code: "REVIEW-001".to_string(),
                        });
                    }
                }
            }
        }

        Ok(issues)
    }

    /// Calculates overall quality score
    fn calculate_quality_score(&self, metrics: &CodeQualityMetrics) -> f32 {
        let weights = [
            (metrics.documentation_score, 0.25),
            (metrics.error_handling_score, 0.25),
            (metrics.style_score, 0.25),
            (metrics.estimated_coverage, 0.25),
        ];

        weights.iter().map(|(score, weight)| score * weight).sum()
    }

    /// Generates a summary of the review
    fn generate_summary(
        &self,
        metrics: &CodeQualityMetrics,
        compliance: &ComplianceDetails,
        quality_score: f32,
        compliance_score: f32,
    ) -> String {
        format!(
            "Code Review Summary:\n\
             - Quality Score: {:.1}%\n\
             - Compliance Score: {:.1}%\n\
             - Total Lines: {}\n\
             - Functions: {}\n\
             - Public Functions: {}\n\
             - Documentation Coverage: {:.1}%\n\
             - Estimated Test Coverage: {:.1}%\n\
             - Requirements Addressed: {}/{}\n\
             - Unmet Criteria: {}",
            quality_score * 100.0,
            compliance_score * 100.0,
            metrics.total_lines,
            metrics.function_count,
            metrics.public_function_count,
            metrics.documentation_score * 100.0,
            metrics.estimated_coverage * 100.0,
            compliance.addressed_requirements,
            compliance.total_requirements,
            compliance.unmet_criteria.len()
        )
    }
}

impl Default for ReviewEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_review_engine_creation() {
        let engine = ReviewEngine::new();
        assert_eq!(engine.config.check_quality, true);
        assert_eq!(engine.config.check_compliance, true);
    }

    #[test]
    fn test_count_functions_rust() {
        let engine = ReviewEngine::new();
        let code = "fn foo() {}\nfn bar() {}";
        assert_eq!(engine.count_functions(code, "rust"), 2);
    }

    #[test]
    fn test_count_public_functions_rust() {
        let engine = ReviewEngine::new();
        let code = "pub fn foo() {}\nfn bar() {}";
        assert_eq!(engine.count_public_functions(code, "rust"), 1);
    }

    #[test]
    fn test_calculate_quality_score() {
        let engine = ReviewEngine::new();
        let metrics = CodeQualityMetrics {
            documentation_score: 0.8,
            error_handling_score: 0.7,
            style_score: 0.9,
            estimated_coverage: 0.6,
            ..Default::default()
        };
        let score = engine.calculate_quality_score(&metrics);
        assert!(score > 0.0 && score <= 1.0);
    }
}
