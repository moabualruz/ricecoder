//! Report generation for code generation results
//!
//! Produces generation reports with statistics including:
//! - File count and lines generated
//! - Validation results
//! - Conflict statistics
//! - Time elapsed and tokens used
//!
//! Implements Requirement 1.6: Generation report with statistics

use crate::models::{GeneratedFile, ValidationResult};
use crate::conflict_detector::FileConflictInfo;
use crate::review_engine::ReviewResult;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Statistics about code generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationStats {
    /// Number of tokens used in generation
    pub tokens_used: usize,
    /// Time elapsed during generation
    pub time_elapsed: Duration,
    /// Number of files generated
    pub files_generated: usize,
    /// Total lines of code generated
    pub lines_generated: usize,
    /// Number of conflicts detected
    pub conflicts_detected: usize,
    /// Number of conflicts resolved
    pub conflicts_resolved: usize,
}

impl Default for GenerationStats {
    fn default() -> Self {
        Self {
            tokens_used: 0,
            time_elapsed: Duration::ZERO,
            files_generated: 0,
            lines_generated: 0,
            conflicts_detected: 0,
            conflicts_resolved: 0,
        }
    }
}

/// Complete result of code generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationResult {
    /// Generated files
    pub files: Vec<GeneratedFile>,
    /// Validation results
    pub validation: ValidationResult,
    /// Review results (optional)
    pub review: Option<ReviewResult>,
    /// Detected conflicts
    pub conflicts: Vec<FileConflictInfo>,
    /// Generation statistics
    pub stats: GenerationStats,
}

impl GenerationResult {
    /// Create a new generation result
    pub fn new(
        files: Vec<GeneratedFile>,
        validation: ValidationResult,
        conflicts: Vec<FileConflictInfo>,
        stats: GenerationStats,
    ) -> Self {
        Self {
            files,
            validation,
            review: None,
            conflicts,
            stats,
        }
    }

    /// Add review results to the generation result
    pub fn with_review(mut self, review: ReviewResult) -> Self {
        self.review = Some(review);
        self
    }
}

/// A generation report with formatted statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationReport {
    /// Report title
    pub title: String,
    /// Report timestamp (ISO 8601 format)
    pub timestamp: String,
    /// Summary of generation
    pub summary: ReportSummary,
    /// File statistics
    pub file_stats: FileStatistics,
    /// Validation report
    pub validation_report: ValidationReport,
    /// Conflict report
    pub conflict_report: ConflictReport,
    /// Review report (optional)
    pub review_report: Option<ReviewReport>,
    /// Performance metrics
    pub performance: PerformanceMetrics,
}

/// Summary of generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSummary {
    /// Whether generation was successful
    pub success: bool,
    /// Overall status message
    pub status: String,
    /// Number of files generated
    pub files_generated: usize,
    /// Total lines of code generated
    pub lines_generated: usize,
}

/// File statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStatistics {
    /// Total files generated
    pub total_files: usize,
    /// Files by language
    pub files_by_language: std::collections::HashMap<String, usize>,
    /// Total lines of code
    pub total_lines: usize,
    /// Average lines per file
    pub average_lines_per_file: f64,
}

/// Validation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    /// Whether validation passed
    pub passed: bool,
    /// Number of errors found
    pub error_count: usize,
    /// Number of warnings found
    pub warning_count: usize,
    /// Errors by file
    pub errors_by_file: std::collections::HashMap<String, usize>,
    /// Warnings by file
    pub warnings_by_file: std::collections::HashMap<String, usize>,
}

/// Conflict report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictReport {
    /// Total conflicts detected
    pub total_conflicts: usize,
    /// Conflicts resolved
    pub conflicts_resolved: usize,
    /// Conflicts pending
    pub conflicts_pending: usize,
    /// Conflicts by strategy
    pub conflicts_by_strategy: std::collections::HashMap<String, usize>,
}

/// Review report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewReport {
    /// Overall quality score (0-100)
    pub quality_score: f64,
    /// Number of suggestions
    pub suggestion_count: usize,
    /// Number of issues found
    pub issue_count: usize,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Time elapsed in seconds
    pub time_elapsed_seconds: f64,
    /// Tokens used
    pub tokens_used: usize,
    /// Files per second
    pub files_per_second: f64,
    /// Lines per second
    pub lines_per_second: f64,
}

/// Generates reports from generation results
pub struct ReportGenerator;

impl ReportGenerator {
    /// Generate a report from generation results
    pub fn generate(result: &GenerationResult) -> GenerationReport {
        let timestamp = chrono::Local::now().to_rfc3339();
        
        // Calculate file statistics
        let file_stats = Self::calculate_file_stats(&result.files);
        
        // Calculate validation report
        let validation_report = Self::calculate_validation_report(&result.validation);
        
        // Calculate conflict report
        let conflict_report = Self::calculate_conflict_report(&result.conflicts);
        
        // Calculate performance metrics
        let performance = Self::calculate_performance(&result.stats);
        
        // Calculate review report if available
        let review_report = result.review.as_ref().map(|review| ReviewReport {
            quality_score: (review.overall_score * 100.0) as f64,
            suggestion_count: review.suggestions.len(),
            issue_count: review.issues.len(),
        });
        
        // Determine overall success
        let success = result.validation.valid && result.conflicts.is_empty();
        let status = if success {
            "Generation completed successfully".to_string()
        } else if !result.validation.valid {
            format!(
                "Generation completed with {} validation errors",
                result.validation.errors.len()
            )
        } else {
            format!(
                "Generation completed with {} conflicts detected",
                result.conflicts.len()
            )
        };
        
        GenerationReport {
            title: "Code Generation Report".to_string(),
            timestamp,
            summary: ReportSummary {
                success,
                status,
                files_generated: result.files.len(),
                lines_generated: file_stats.total_lines,
            },
            file_stats,
            validation_report,
            conflict_report,
            review_report,
            performance,
        }
    }

    /// Generate a report as formatted text
    pub fn generate_text(result: &GenerationResult) -> String {
        let report = Self::generate(result);
        Self::format_report(&report)
    }

    /// Generate a report as JSON
    pub fn generate_json(result: &GenerationResult) -> Result<String, serde_json::Error> {
        let report = Self::generate(result);
        serde_json::to_string_pretty(&report)
    }

    fn calculate_file_stats(files: &[GeneratedFile]) -> FileStatistics {
        let mut files_by_language = std::collections::HashMap::new();
        let mut total_lines = 0;

        for file in files {
            let line_count = file.content.lines().count();
            total_lines += line_count;
            
            *files_by_language.entry(file.language.clone()).or_insert(0) += 1;
        }

        let average_lines_per_file = if files.is_empty() {
            0.0
        } else {
            total_lines as f64 / files.len() as f64
        };

        FileStatistics {
            total_files: files.len(),
            files_by_language,
            total_lines,
            average_lines_per_file,
        }
    }

    fn calculate_validation_report(validation: &ValidationResult) -> ValidationReport {
        let mut errors_by_file = std::collections::HashMap::new();
        let mut warnings_by_file = std::collections::HashMap::new();

        for error in &validation.errors {
            *errors_by_file.entry(error.file.clone()).or_insert(0) += 1;
        }

        for warning in &validation.warnings {
            *warnings_by_file.entry(warning.file.clone()).or_insert(0) += 1;
        }

        ValidationReport {
            passed: validation.valid,
            error_count: validation.errors.len(),
            warning_count: validation.warnings.len(),
            errors_by_file,
            warnings_by_file,
        }
    }

    fn calculate_conflict_report(conflicts: &[FileConflictInfo]) -> ConflictReport {
        let mut conflicts_by_strategy = std::collections::HashMap::new();

        // All conflicts are initially pending (no strategy assigned yet)
        if !conflicts.is_empty() {
            conflicts_by_strategy.insert("Pending".to_string(), conflicts.len());
        }

        ConflictReport {
            total_conflicts: conflicts.len(),
            conflicts_resolved: 0, // Will be updated when conflicts are resolved
            conflicts_pending: conflicts.len(),
            conflicts_by_strategy,
        }
    }

    fn calculate_performance(stats: &GenerationStats) -> PerformanceMetrics {
        let time_elapsed_seconds = stats.time_elapsed.as_secs_f64();
        let files_per_second = if time_elapsed_seconds > 0.0 {
            stats.files_generated as f64 / time_elapsed_seconds
        } else {
            0.0
        };
        let lines_per_second = if time_elapsed_seconds > 0.0 {
            stats.lines_generated as f64 / time_elapsed_seconds
        } else {
            0.0
        };

        PerformanceMetrics {
            time_elapsed_seconds,
            tokens_used: stats.tokens_used,
            files_per_second,
            lines_per_second,
        }
    }

    fn format_report(report: &GenerationReport) -> String {
        let mut output = String::new();

        output.push_str("╔════════════════════════════════════════════════════════════╗\n");
        output.push_str(&format!("║ {}                                    ║\n", report.title));
        output.push_str("╚════════════════════════════════════════════════════════════╝\n\n");

        output.push_str(&format!("Timestamp: {}\n\n", report.timestamp));

        // Summary
        output.push_str("SUMMARY\n");
        output.push_str("───────────────────────────────────────────────────────────────\n");
        output.push_str(&format!("Status: {}\n", report.summary.status));
        output.push_str(&format!("Files Generated: {}\n", report.summary.files_generated));
        output.push_str(&format!("Lines Generated: {}\n\n", report.summary.lines_generated));

        // File Statistics
        output.push_str("FILE STATISTICS\n");
        output.push_str("───────────────────────────────────────────────────────────────\n");
        output.push_str(&format!("Total Files: {}\n", report.file_stats.total_files));
        output.push_str(&format!("Total Lines: {}\n", report.file_stats.total_lines));
        output.push_str(&format!(
            "Average Lines per File: {:.2}\n",
            report.file_stats.average_lines_per_file
        ));
        
        if !report.file_stats.files_by_language.is_empty() {
            output.push_str("\nFiles by Language:\n");
            for (lang, count) in &report.file_stats.files_by_language {
                output.push_str(&format!("  {}: {}\n", lang, count));
            }
        }
        output.push('\n');

        // Validation Report
        output.push_str("VALIDATION RESULTS\n");
        output.push_str("───────────────────────────────────────────────────────────────\n");
        output.push_str(&format!(
            "Status: {}\n",
            if report.validation_report.passed {
                "PASSED"
            } else {
                "FAILED"
            }
        ));
        output.push_str(&format!("Errors: {}\n", report.validation_report.error_count));
        output.push_str(&format!("Warnings: {}\n\n", report.validation_report.warning_count));

        // Conflict Report
        output.push_str("CONFLICT DETECTION\n");
        output.push_str("───────────────────────────────────────────────────────────────\n");
        output.push_str(&format!(
            "Total Conflicts: {}\n",
            report.conflict_report.total_conflicts
        ));
        output.push_str(&format!(
            "Conflicts Resolved: {}\n",
            report.conflict_report.conflicts_resolved
        ));
        output.push_str(&format!(
            "Conflicts Pending: {}\n\n",
            report.conflict_report.conflicts_pending
        ));

        // Performance Metrics
        output.push_str("PERFORMANCE METRICS\n");
        output.push_str("───────────────────────────────────────────────────────────────\n");
        output.push_str(&format!(
            "Time Elapsed: {:.2}s\n",
            report.performance.time_elapsed_seconds
        ));
        output.push_str(&format!("Tokens Used: {}\n", report.performance.tokens_used));
        output.push_str(&format!(
            "Files per Second: {:.2}\n",
            report.performance.files_per_second
        ));
        output.push_str(&format!(
            "Lines per Second: {:.2}\n\n",
            report.performance.lines_per_second
        ));

        // Review Report
        if let Some(review) = &report.review_report {
            output.push_str("CODE REVIEW\n");
            output.push_str("───────────────────────────────────────────────────────────────\n");
            output.push_str(&format!("Quality Score: {:.1}/100\n", review.quality_score));
            output.push_str(&format!("Suggestions: {}\n", review.suggestion_count));
            output.push_str(&format!("Issues: {}\n\n", review.issue_count));
        }

        output.push_str("═══════════════════════════════════════════════════════════════\n");

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generation_stats_default() {
        let stats = GenerationStats::default();
        assert_eq!(stats.tokens_used, 0);
        assert_eq!(stats.files_generated, 0);
        assert_eq!(stats.lines_generated, 0);
        assert_eq!(stats.conflicts_detected, 0);
        assert_eq!(stats.conflicts_resolved, 0);
    }

    #[test]
    fn test_generation_result_creation() {
        let files = vec![GeneratedFile {
            path: "test.rs".to_string(),
            content: "fn main() {}".to_string(),
            language: "rust".to_string(),
        }];
        let validation = ValidationResult::default();
        let conflicts = vec![];
        let stats = GenerationStats {
            files_generated: 1,
            lines_generated: 1,
            ..Default::default()
        };

        let result = GenerationResult::new(files, validation, conflicts, stats);
        assert_eq!(result.files.len(), 1);
        assert!(result.validation.valid);
        assert!(result.conflicts.is_empty());
    }

    #[test]
    fn test_report_generation() {
        let files = vec![
            GeneratedFile {
                path: "test1.rs".to_string(),
                content: "fn main() {\n    println!(\"Hello\");\n}".to_string(),
                language: "rust".to_string(),
            },
            GeneratedFile {
                path: "test2.rs".to_string(),
                content: "fn helper() {}".to_string(),
                language: "rust".to_string(),
            },
        ];
        let validation = ValidationResult::default();
        let conflicts = vec![];
        let stats = GenerationStats {
            files_generated: 2,
            lines_generated: 4,
            tokens_used: 100,
            time_elapsed: Duration::from_secs(1),
            ..Default::default()
        };

        let result = GenerationResult::new(files, validation, conflicts, stats);
        let report = ReportGenerator::generate(&result);

        assert_eq!(report.summary.files_generated, 2);
        assert_eq!(report.summary.lines_generated, 4);
        assert_eq!(report.file_stats.total_files, 2);
        assert_eq!(report.performance.tokens_used, 100);
    }

    #[test]
    fn test_report_text_generation() {
        let files = vec![GeneratedFile {
            path: "test.rs".to_string(),
            content: "fn main() {}".to_string(),
            language: "rust".to_string(),
        }];
        let validation = ValidationResult::default();
        let conflicts = vec![];
        let stats = GenerationStats::default();

        let result = GenerationResult::new(files, validation, conflicts, stats);
        let text = ReportGenerator::generate_text(&result);

        assert!(text.contains("Code Generation Report"));
        assert!(text.contains("SUMMARY"));
        assert!(text.contains("FILE STATISTICS"));
        assert!(text.contains("VALIDATION RESULTS"));
    }

    #[test]
    fn test_report_json_generation() {
        let files = vec![GeneratedFile {
            path: "test.rs".to_string(),
            content: "fn main() {}".to_string(),
            language: "rust".to_string(),
        }];
        let validation = ValidationResult::default();
        let conflicts = vec![];
        let stats = GenerationStats::default();

        let result = GenerationResult::new(files, validation, conflicts, stats);
        let json = ReportGenerator::generate_json(&result);

        assert!(json.is_ok());
        let json_str = json.unwrap();
        assert!(json_str.contains("\"title\""));
        assert!(json_str.contains("\"timestamp\""));
    }
}
