//! Conflict detection and resolution for domain agent recommendations

use serde::{Deserialize, Serialize};

use crate::domain::{error::DomainResult, models::Recommendation};

/// Type of conflict between recommendations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictType {
    /// Recommendations are incompatible
    Incompatible,
    /// Recommendations contradict each other
    Contradictory,
    /// Recommendations require specific sequencing
    RequiresSequencing,
}

/// A conflict between two recommendations
///
/// This struct represents a conflict detected between recommendations
/// from different domain agents.
///
/// # Examples
///
/// ```ignore
/// use ricecoder_agents::domain::Conflict;
///
/// let conflict = Conflict {
///     recommendation_a: rec_a,
///     recommendation_b: rec_b,
///     conflict_type: ConflictType::Incompatible,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    /// First recommendation
    pub recommendation_a: Recommendation,
    /// Second recommendation
    pub recommendation_b: Recommendation,
    /// Type of conflict
    pub conflict_type: ConflictType,
}

/// Report of detected conflicts
///
/// This struct contains a report of all conflicts detected between
/// recommendations from different domain agents.
///
/// # Examples
///
/// ```ignore
/// use ricecoder_agents::domain::ConflictReport;
///
/// let report = ConflictReport {
///     conflicting_recommendations: vec![],
///     analysis: "No conflicts detected".to_string(),
///     suggested_resolution: "Proceed with recommendations".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictReport {
    /// Conflicting recommendations
    pub conflicting_recommendations: Vec<Conflict>,
    /// Analysis of conflicts
    pub analysis: String,
    /// Suggested resolution
    pub suggested_resolution: String,
}

/// Detects conflicts between domain agent recommendations
///
/// This struct analyzes recommendations from different domain agents
/// and detects conflicts between them.
///
/// # Examples
///
/// ```ignore
/// use ricecoder_agents::domain::ConflictDetector;
///
/// let detector = ConflictDetector::new();
/// let conflicts = detector.detect_conflicts(recommendations)?;
/// ```
#[derive(Debug, Clone)]
pub struct ConflictDetector;

impl ConflictDetector {
    /// Create a new conflict detector
    pub fn new() -> Self {
        Self
    }

    /// Detect conflicts between recommendations
    ///
    /// # Arguments
    ///
    /// * `recommendations` - Recommendations to analyze
    ///
    /// # Returns
    ///
    /// Returns a vector of detected conflicts
    pub fn detect_conflicts(
        &self,
        recommendations: Vec<Recommendation>,
    ) -> DomainResult<Vec<Conflict>> {
        let mut conflicts = Vec::new();

        // Check each pair of recommendations for conflicts
        for i in 0..recommendations.len() {
            for j in (i + 1)..recommendations.len() {
                if let Some(conflict) =
                    self.check_conflict(&recommendations[i], &recommendations[j])
                {
                    conflicts.push(conflict);
                }
            }
        }

        Ok(conflicts)
    }

    /// Check if two recommendations conflict
    fn check_conflict(&self, rec_a: &Recommendation, rec_b: &Recommendation) -> Option<Conflict> {
        // Check for incompatible technologies
        if self.are_technologies_incompatible(&rec_a.technologies, &rec_b.technologies) {
            return Some(Conflict {
                recommendation_a: rec_a.clone(),
                recommendation_b: rec_b.clone(),
                conflict_type: ConflictType::Incompatible,
            });
        }

        // Check for contradictory recommendations
        if self.are_recommendations_contradictory(rec_a, rec_b) {
            return Some(Conflict {
                recommendation_a: rec_a.clone(),
                recommendation_b: rec_b.clone(),
                conflict_type: ConflictType::Contradictory,
            });
        }

        None
    }

    /// Check if technologies are incompatible
    fn are_technologies_incompatible(&self, tech_a: &[String], tech_b: &[String]) -> bool {
        // Define incompatible technology pairs
        let incompatible_pairs = vec![
            ("React", "Angular"),
            ("React", "Vue"),
            ("Angular", "Vue"),
            ("Webpack", "Vite"),
            ("npm", "yarn"),
            ("PostgreSQL", "MongoDB"),
            ("REST", "GraphQL"),
            ("Microservices", "Monolithic"),
        ];

        for tech_a_item in tech_a {
            for tech_b_item in tech_b {
                for (incompat_a, incompat_b) in &incompatible_pairs {
                    if (tech_a_item.contains(incompat_a) && tech_b_item.contains(incompat_b))
                        || (tech_a_item.contains(incompat_b) && tech_b_item.contains(incompat_a))
                    {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Check if recommendations are contradictory
    fn are_recommendations_contradictory(
        &self,
        rec_a: &Recommendation,
        rec_b: &Recommendation,
    ) -> bool {
        // Check if recommendations are from different domains
        if rec_a.domain == rec_b.domain {
            return false;
        }

        // Check for contradictory content patterns
        let contradictory_patterns = vec![
            ("must", "must not"),
            ("should", "should not"),
            ("required", "not required"),
        ];

        let content_a = rec_a.content.to_lowercase();
        let content_b = rec_b.content.to_lowercase();

        for (pattern_a, pattern_b) in contradictory_patterns {
            if (content_a.contains(pattern_a) && content_b.contains(pattern_b))
                || (content_a.contains(pattern_b) && content_b.contains(pattern_a))
            {
                return true;
            }
        }

        false
    }

    /// Generate a conflict report
    ///
    /// # Arguments
    ///
    /// * `conflicts` - Detected conflicts
    ///
    /// # Returns
    ///
    /// Returns a conflict report
    pub fn generate_report(&self, conflicts: Vec<Conflict>) -> DomainResult<ConflictReport> {
        let analysis = if conflicts.is_empty() {
            "No conflicts detected between domain recommendations".to_string()
        } else {
            format!(
                "Detected {} conflict(s) between domain recommendations",
                conflicts.len()
            )
        };

        let suggested_resolution = if conflicts.is_empty() {
            "Proceed with all recommendations".to_string()
        } else {
            "Review conflicting recommendations and prioritize based on project requirements"
                .to_string()
        };

        Ok(ConflictReport {
            conflicting_recommendations: conflicts,
            analysis,
            suggested_resolution,
        })
    }
}

impl Default for ConflictDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl ConflictDetector {
    /// Format a conflict for user presentation
    ///
    /// # Arguments
    ///
    /// * `conflict` - The conflict to format
    ///
    /// # Returns
    ///
    /// Returns a formatted string representation of the conflict
    pub fn format_conflict(&self, conflict: &Conflict) -> String {
        let conflict_type_str = match conflict.conflict_type {
            ConflictType::Incompatible => "Incompatible",
            ConflictType::Contradictory => "Contradictory",
            ConflictType::RequiresSequencing => "Requires Sequencing",
        };

        format!(
            "Conflict Type: {}\n\
             Domain A: {}\n\
             Category A: {}\n\
             Content A: {}\n\
             Rationale A: {}\n\
             \n\
             Domain B: {}\n\
             Category B: {}\n\
             Content B: {}\n\
             Rationale B: {}",
            conflict_type_str,
            conflict.recommendation_a.domain,
            conflict.recommendation_a.category,
            conflict.recommendation_a.content,
            conflict.recommendation_a.rationale,
            conflict.recommendation_b.domain,
            conflict.recommendation_b.category,
            conflict.recommendation_b.content,
            conflict.recommendation_b.rationale,
        )
    }

    /// Format a conflict report for user presentation
    ///
    /// # Arguments
    ///
    /// * `report` - The conflict report to format
    ///
    /// # Returns
    ///
    /// Returns a formatted string representation of the report
    pub fn format_report(&self, report: &ConflictReport) -> String {
        let mut output = String::new();

        output.push_str("=== CONFLICT REPORT ===\n\n");
        output.push_str(&format!("Analysis: {}\n\n", report.analysis));

        if report.conflicting_recommendations.is_empty() {
            output.push_str("No conflicts detected.\n");
        } else {
            output.push_str(&format!(
                "Conflicts Found: {}\n\n",
                report.conflicting_recommendations.len()
            ));

            for (index, conflict) in report.conflicting_recommendations.iter().enumerate() {
                output.push_str(&format!("--- Conflict {} ---\n", index + 1));
                output.push_str(&self.format_conflict(conflict));
                output.push_str("\n\n");
            }
        }

        output.push_str(&format!(
            "Suggested Resolution: {}\n",
            report.suggested_resolution
        ));

        output
    }

    /// Suggest resolution strategies for a conflict
    ///
    /// # Arguments
    ///
    /// * `conflict` - The conflict to analyze
    ///
    /// # Returns
    ///
    /// Returns a vector of suggested resolution strategies
    pub fn suggest_resolutions(&self, conflict: &Conflict) -> Vec<String> {
        let mut suggestions = Vec::new();

        match conflict.conflict_type {
            ConflictType::Incompatible => {
                suggestions.push(format!(
                    "Choose one technology: {} or {}",
                    conflict.recommendation_a.technologies.join(", "),
                    conflict.recommendation_b.technologies.join(", ")
                ));
                suggestions.push(
                    "Evaluate pros and cons of each option based on project requirements"
                        .to_string(),
                );
                suggestions.push("Consider team expertise and ecosystem maturity".to_string());
            }
            ConflictType::Contradictory => {
                suggestions.push("Review the rationale for each recommendation".to_string());
                suggestions.push(
                    "Identify the underlying requirements that led to each recommendation"
                        .to_string(),
                );
                suggestions
                    .push("Prioritize recommendations based on project constraints".to_string());
            }
            ConflictType::RequiresSequencing => {
                suggestions.push("Determine the correct order of operations".to_string());
                suggestions.push("Ensure dependencies are satisfied before proceeding".to_string());
                suggestions
                    .push("Document the sequencing requirements for future reference".to_string());
            }
        }

        suggestions
    }

    /// Generate a detailed conflict analysis
    ///
    /// # Arguments
    ///
    /// * `conflict` - The conflict to analyze
    ///
    /// # Returns
    ///
    /// Returns a detailed analysis string
    pub fn analyze_conflict(&self, conflict: &Conflict) -> String {
        let mut analysis = String::new();

        analysis.push_str("=== CONFLICT ANALYSIS ===\n\n");

        analysis.push_str(&format!("Conflict Type: "));
        match conflict.conflict_type {
            ConflictType::Incompatible => {
                analysis.push_str("Incompatible\n");
                analysis.push_str("These recommendations use incompatible technologies that cannot be used together.\n");
            }
            ConflictType::Contradictory => {
                analysis.push_str("Contradictory\n");
                analysis.push_str(
                    "These recommendations contradict each other and cannot both be implemented.\n",
                );
            }
            ConflictType::RequiresSequencing => {
                analysis.push_str("Requires Sequencing\n");
                analysis
                    .push_str("These recommendations must be implemented in a specific order.\n");
            }
        }

        analysis.push_str("\n--- Recommendation A ---\n");
        analysis.push_str(&format!("Domain: {}\n", conflict.recommendation_a.domain));
        analysis.push_str(&format!(
            "Category: {}\n",
            conflict.recommendation_a.category
        ));
        analysis.push_str(&format!("Content: {}\n", conflict.recommendation_a.content));
        analysis.push_str(&format!(
            "Rationale: {}\n",
            conflict.recommendation_a.rationale
        ));
        analysis.push_str(&format!(
            "Technologies: {}\n",
            conflict.recommendation_a.technologies.join(", ")
        ));

        analysis.push_str("\n--- Recommendation B ---\n");
        analysis.push_str(&format!("Domain: {}\n", conflict.recommendation_b.domain));
        analysis.push_str(&format!(
            "Category: {}\n",
            conflict.recommendation_b.category
        ));
        analysis.push_str(&format!("Content: {}\n", conflict.recommendation_b.content));
        analysis.push_str(&format!(
            "Rationale: {}\n",
            conflict.recommendation_b.rationale
        ));
        analysis.push_str(&format!(
            "Technologies: {}\n",
            conflict.recommendation_b.technologies.join(", ")
        ));

        analysis.push_str("\n--- Suggested Resolutions ---\n");
        let resolutions = self.suggest_resolutions(conflict);
        for (index, resolution) in resolutions.iter().enumerate() {
            analysis.push_str(&format!("{}. {}\n", index + 1, resolution));
        }

        analysis
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_recommendation(domain: &str, technologies: Vec<&str>) -> Recommendation {
        Recommendation {
            domain: domain.to_string(),
            category: "test".to_string(),
            content: "Test recommendation".to_string(),
            technologies: technologies.iter().map(|t| t.to_string()).collect(),
            rationale: "Test rationale".to_string(),
        }
    }

    #[test]
    fn test_detector_creation() {
        let detector = ConflictDetector::new();
        assert_eq!(std::mem::size_of_val(&detector), 0); // Zero-sized type
    }

    #[test]
    fn test_detect_no_conflicts() {
        let detector = ConflictDetector::new();
        let recommendations = vec![
            create_test_recommendation("web", vec!["React"]),
            create_test_recommendation("backend", vec!["Node.js"]),
        ];

        let conflicts = detector.detect_conflicts(recommendations).unwrap();
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_detect_incompatible_technologies() {
        let detector = ConflictDetector::new();
        let recommendations = vec![
            create_test_recommendation("web", vec!["React"]),
            create_test_recommendation("web", vec!["Angular"]),
        ];

        let conflicts = detector.detect_conflicts(recommendations).unwrap();
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].conflict_type, ConflictType::Incompatible);
    }

    #[test]
    fn test_detect_contradictory_recommendations() {
        let detector = ConflictDetector::new();
        let mut rec_a = create_test_recommendation("web", vec!["React"]);
        rec_a.content = "Must use React".to_string();

        let mut rec_b = create_test_recommendation("backend", vec!["Node.js"]);
        rec_b.content = "Must not use React".to_string();

        let conflicts = detector.detect_conflicts(vec![rec_a, rec_b]).unwrap();
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].conflict_type, ConflictType::Contradictory);
    }

    #[test]
    fn test_are_technologies_incompatible() {
        let detector = ConflictDetector::new();

        assert!(detector.are_technologies_incompatible(
            &vec!["React".to_string()],
            &vec!["Angular".to_string()]
        ));

        assert!(detector.are_technologies_incompatible(
            &vec!["Webpack".to_string()],
            &vec!["Vite".to_string()]
        ));

        assert!(!detector.are_technologies_incompatible(
            &vec!["React".to_string()],
            &vec!["Node.js".to_string()]
        ));
    }

    #[test]
    fn test_are_recommendations_contradictory() {
        let detector = ConflictDetector::new();

        let mut rec_a = create_test_recommendation("web", vec!["React"]);
        rec_a.content = "must use React".to_string();

        let mut rec_b = create_test_recommendation("backend", vec!["Node.js"]);
        rec_b.content = "must not use React".to_string();

        assert!(detector.are_recommendations_contradictory(&rec_a, &rec_b));
    }

    #[test]
    fn test_generate_report_no_conflicts() {
        let detector = ConflictDetector::new();
        let report = detector.generate_report(vec![]).unwrap();

        assert!(report.conflicting_recommendations.is_empty());
        assert!(report.analysis.contains("No conflicts"));
    }

    #[test]
    fn test_generate_report_with_conflicts() {
        let detector = ConflictDetector::new();
        let conflict = Conflict {
            recommendation_a: create_test_recommendation("web", vec!["React"]),
            recommendation_b: create_test_recommendation("web", vec!["Angular"]),
            conflict_type: ConflictType::Incompatible,
        };

        let report = detector.generate_report(vec![conflict]).unwrap();

        assert_eq!(report.conflicting_recommendations.len(), 1);
        assert!(report.analysis.contains("1 conflict"));
    }

    #[test]
    fn test_default_detector() {
        let detector = ConflictDetector::default();
        let recommendations = vec![create_test_recommendation("web", vec!["React"])];

        assert!(detector.detect_conflicts(recommendations).is_ok());
    }

    #[test]
    fn test_multiple_conflicts() {
        let detector = ConflictDetector::new();
        let recommendations = vec![
            create_test_recommendation("web", vec!["React"]),
            create_test_recommendation("web", vec!["Angular"]),
            create_test_recommendation("web", vec!["Vue"]),
        ];

        let conflicts = detector.detect_conflicts(recommendations).unwrap();
        // React vs Angular, React vs Vue, Angular vs Vue = 3 conflicts
        assert!(conflicts.len() >= 1);
    }

    #[test]
    fn test_conflict_type_equality() {
        assert_eq!(ConflictType::Incompatible, ConflictType::Incompatible);
        assert_ne!(ConflictType::Incompatible, ConflictType::Contradictory);
    }

    #[test]
    fn test_conflict_serialization() {
        let conflict = Conflict {
            recommendation_a: create_test_recommendation("web", vec!["React"]),
            recommendation_b: create_test_recommendation("web", vec!["Angular"]),
            conflict_type: ConflictType::Incompatible,
        };

        let json = serde_json::to_string(&conflict).expect("serialization failed");
        let deserialized: Conflict = serde_json::from_str(&json).expect("deserialization failed");

        assert_eq!(deserialized.conflict_type, conflict.conflict_type);
    }

    #[test]
    fn test_format_conflict_incompatible() {
        let detector = ConflictDetector::new();
        let conflict = Conflict {
            recommendation_a: create_test_recommendation("web", vec!["React"]),
            recommendation_b: create_test_recommendation("web", vec!["Angular"]),
            conflict_type: ConflictType::Incompatible,
        };

        let formatted = detector.format_conflict(&conflict);
        assert!(formatted.contains("Incompatible"));
        assert!(formatted.contains("web"));
        assert!(formatted.contains("Rationale"));
    }

    #[test]
    fn test_format_conflict_contradictory() {
        let detector = ConflictDetector::new();
        let conflict = Conflict {
            recommendation_a: create_test_recommendation("web", vec!["React"]),
            recommendation_b: create_test_recommendation("backend", vec!["Node.js"]),
            conflict_type: ConflictType::Contradictory,
        };

        let formatted = detector.format_conflict(&conflict);
        assert!(formatted.contains("Contradictory"));
        assert!(formatted.contains("Domain A"));
        assert!(formatted.contains("Domain B"));
    }

    #[test]
    fn test_format_conflict_requires_sequencing() {
        let detector = ConflictDetector::new();
        let conflict = Conflict {
            recommendation_a: create_test_recommendation("backend", vec!["PostgreSQL"]),
            recommendation_b: create_test_recommendation("devops", vec!["Docker"]),
            conflict_type: ConflictType::RequiresSequencing,
        };

        let formatted = detector.format_conflict(&conflict);
        assert!(formatted.contains("Requires Sequencing"));
    }

    #[test]
    fn test_format_report_no_conflicts() {
        let detector = ConflictDetector::new();
        let report = ConflictReport {
            conflicting_recommendations: vec![],
            analysis: "No conflicts detected".to_string(),
            suggested_resolution: "Proceed with all recommendations".to_string(),
        };

        let formatted = detector.format_report(&report);
        assert!(formatted.contains("CONFLICT REPORT"));
        assert!(formatted.contains("No conflicts detected"));
        assert!(formatted.contains("Proceed with all recommendations"));
    }

    #[test]
    fn test_format_report_with_conflicts() {
        let detector = ConflictDetector::new();
        let conflict = Conflict {
            recommendation_a: create_test_recommendation("web", vec!["React"]),
            recommendation_b: create_test_recommendation("web", vec!["Angular"]),
            conflict_type: ConflictType::Incompatible,
        };

        let report = ConflictReport {
            conflicting_recommendations: vec![conflict],
            analysis: "Detected 1 conflict".to_string(),
            suggested_resolution: "Choose one framework".to_string(),
        };

        let formatted = detector.format_report(&report);
        assert!(formatted.contains("CONFLICT REPORT"));
        assert!(formatted.contains("Conflicts Found: 1"));
        assert!(formatted.contains("Conflict 1"));
        assert!(formatted.contains("Choose one framework"));
    }

    #[test]
    fn test_suggest_resolutions_incompatible() {
        let detector = ConflictDetector::new();
        let conflict = Conflict {
            recommendation_a: create_test_recommendation("web", vec!["React"]),
            recommendation_b: create_test_recommendation("web", vec!["Angular"]),
            conflict_type: ConflictType::Incompatible,
        };

        let resolutions = detector.suggest_resolutions(&conflict);
        assert!(!resolutions.is_empty());
        assert!(resolutions[0].contains("Choose one technology"));
    }

    #[test]
    fn test_suggest_resolutions_contradictory() {
        let detector = ConflictDetector::new();
        let conflict = Conflict {
            recommendation_a: create_test_recommendation("web", vec!["React"]),
            recommendation_b: create_test_recommendation("backend", vec!["Node.js"]),
            conflict_type: ConflictType::Contradictory,
        };

        let resolutions = detector.suggest_resolutions(&conflict);
        assert!(!resolutions.is_empty());
        assert!(resolutions[0].contains("Review the rationale"));
    }

    #[test]
    fn test_suggest_resolutions_requires_sequencing() {
        let detector = ConflictDetector::new();
        let conflict = Conflict {
            recommendation_a: create_test_recommendation("backend", vec!["PostgreSQL"]),
            recommendation_b: create_test_recommendation("devops", vec!["Docker"]),
            conflict_type: ConflictType::RequiresSequencing,
        };

        let resolutions = detector.suggest_resolutions(&conflict);
        assert!(!resolutions.is_empty());
        assert!(resolutions[0].contains("Determine the correct order"));
    }

    #[test]
    fn test_analyze_conflict() {
        let detector = ConflictDetector::new();
        let conflict = Conflict {
            recommendation_a: create_test_recommendation("web", vec!["React"]),
            recommendation_b: create_test_recommendation("web", vec!["Angular"]),
            conflict_type: ConflictType::Incompatible,
        };

        let analysis = detector.analyze_conflict(&conflict);
        assert!(analysis.contains("CONFLICT ANALYSIS"));
        assert!(analysis.contains("Incompatible"));
        assert!(analysis.contains("Recommendation A"));
        assert!(analysis.contains("Recommendation B"));
        assert!(analysis.contains("Suggested Resolutions"));
    }

    #[test]
    fn test_analyze_conflict_includes_rationale() {
        let detector = ConflictDetector::new();
        let conflict = Conflict {
            recommendation_a: create_test_recommendation("web", vec!["React"]),
            recommendation_b: create_test_recommendation("web", vec!["Angular"]),
            conflict_type: ConflictType::Incompatible,
        };

        let analysis = detector.analyze_conflict(&conflict);
        assert!(analysis.contains("Rationale"));
        assert!(analysis.contains("Technologies"));
    }
}
