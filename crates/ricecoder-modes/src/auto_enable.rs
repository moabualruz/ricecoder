use crate::models::ComplexityLevel;

/// Detects task complexity and determines if Think More should be auto-enabled
#[derive(Debug, Clone)]
pub struct ComplexityDetector {
    /// Threshold for auto-enabling Think More
    threshold: ComplexityLevel,
}

impl ComplexityDetector {
    /// Create a new complexity detector with a specific threshold
    pub fn new(threshold: ComplexityLevel) -> Self {
        Self { threshold }
    }

    /// Create a detector with default threshold (Complex)
    pub fn default_threshold() -> Self {
        Self {
            threshold: ComplexityLevel::Complex,
        }
    }

    /// Detect complexity from task description
    pub fn detect_complexity(&self, task_description: &str) -> ComplexityLevel {
        let complexity_score = self.calculate_complexity_score(task_description);
        self.score_to_complexity(complexity_score)
    }

    /// Calculate a complexity score based on various factors
    pub fn calculate_complexity_score(&self, task_description: &str) -> f32 {
        let mut score = 0.0;

        // Factor 1: Length of description (longer = more complex)
        let word_count = task_description.split_whitespace().count() as f32;
        score += (word_count / 100.0).min(2.0);

        // Factor 2: Presence of complexity keywords
        let complexity_keywords = [
            "complex", "difficult", "challenging", "intricate", "sophisticated",
            "algorithm", "optimization", "performance", "architecture", "design",
            "refactor", "restructure", "integrate", "coordinate", "orchestrate",
            "analyze", "debug", "troubleshoot", "investigate", "research",
            "multiple", "various", "several", "many", "numerous",
            "concurrent", "parallel", "async", "distributed", "scalable",
        ];

        for keyword in &complexity_keywords {
            if task_description.to_lowercase().contains(keyword) {
                score += 1.0;
            }
        }

        // Factor 3: Presence of uncertainty indicators
        let uncertainty_keywords = [
            "unclear", "ambiguous", "uncertain", "unknown", "not sure",
            "might", "could", "possibly", "perhaps", "maybe",
            "question", "problem", "issue", "bug", "error",
        ];

        for keyword in &uncertainty_keywords {
            if task_description.to_lowercase().contains(keyword) {
                score += 0.5;
            }
        }

        // Factor 4: Presence of multiple requirements
        let requirement_indicators = ["and", "also", "additionally", "furthermore", "moreover"];
        let requirement_count = requirement_indicators
            .iter()
            .filter(|indicator| task_description.to_lowercase().contains(*indicator))
            .count() as f32;
        score += (requirement_count / 5.0).min(1.0);

        // Factor 5: Presence of technical depth indicators
        let technical_keywords = [
            "algorithm", "data structure", "memory", "performance", "optimization",
            "concurrency", "threading", "async", "distributed", "microservice",
            "database", "query", "index", "cache", "transaction",
        ];

        for keyword in &technical_keywords {
            if task_description.to_lowercase().contains(keyword) {
                score += 1.5;
            }
        }

        score
    }

    /// Convert complexity score to complexity level
    fn score_to_complexity(&self, score: f32) -> ComplexityLevel {
        match score {
            s if s < 2.0 => ComplexityLevel::Simple,
            s if s < 5.0 => ComplexityLevel::Moderate,
            _ => ComplexityLevel::Complex,
        }
    }

    /// Check if Think More should be auto-enabled for a given complexity
    pub fn should_auto_enable(&self, complexity: ComplexityLevel) -> bool {
        matches!(
            (complexity, self.threshold),
            (ComplexityLevel::Complex, _)
                | (ComplexityLevel::Moderate, ComplexityLevel::Moderate)
                | (ComplexityLevel::Moderate, ComplexityLevel::Simple)
        )
    }

    /// Set the complexity threshold
    pub fn set_threshold(&mut self, threshold: ComplexityLevel) {
        self.threshold = threshold;
    }

    /// Get the current threshold
    pub fn get_threshold(&self) -> ComplexityLevel {
        self.threshold
    }

    /// Analyze task and return detailed complexity analysis
    pub fn analyze_task(&self, task_description: &str) -> ComplexityAnalysis {
        let complexity = self.detect_complexity(task_description);
        let score = self.calculate_complexity_score(task_description);
        let should_enable = self.should_auto_enable(complexity);

        ComplexityAnalysis {
            complexity,
            score,
            should_enable_think_more: should_enable,
            reasoning: self.generate_reasoning(task_description, complexity, score),
        }
    }

    /// Generate reasoning for the complexity assessment
    fn generate_reasoning(&self, task_description: &str, complexity: ComplexityLevel, score: f32) -> String {
        let mut reasoning = String::new();

        reasoning.push_str(&format!("Complexity Level: {:?}\n", complexity));
        reasoning.push_str(&format!("Complexity Score: {:.2}\n", score));

        // Identify contributing factors
        let mut factors = Vec::new();

        let word_count = task_description.split_whitespace().count();
        if word_count > 100 {
            factors.push(format!("Long description ({} words)", word_count));
        }

        let complexity_keywords = [
            "complex", "difficult", "challenging", "intricate", "sophisticated",
            "algorithm", "optimization", "performance", "architecture", "design",
        ];
        let found_keywords: Vec<&str> = complexity_keywords
            .iter()
            .filter(|kw| task_description.to_lowercase().contains(*kw))
            .copied()
            .collect();
        if !found_keywords.is_empty() {
            factors.push(format!("Complex keywords found: {}", found_keywords.join(", ")));
        }

        let technical_keywords = [
            "algorithm", "data structure", "memory", "performance", "optimization",
            "concurrency", "threading", "async", "distributed",
        ];
        let found_technical: Vec<&str> = technical_keywords
            .iter()
            .filter(|kw| task_description.to_lowercase().contains(*kw))
            .copied()
            .collect();
        if !found_technical.is_empty() {
            factors.push(format!("Technical depth: {}", found_technical.join(", ")));
        }

        if factors.is_empty() {
            reasoning.push_str("Factors: Simple task with straightforward requirements\n");
        } else {
            reasoning.push_str("Factors:\n");
            for factor in factors {
                reasoning.push_str(&format!("  - {}\n", factor));
            }
        }

        reasoning
    }
}

impl Default for ComplexityDetector {
    fn default() -> Self {
        Self::default_threshold()
    }
}

/// Detailed analysis of task complexity
#[derive(Debug, Clone)]
pub struct ComplexityAnalysis {
    /// Detected complexity level
    pub complexity: ComplexityLevel,
    /// Complexity score (0.0 - infinity)
    pub score: f32,
    /// Whether Think More should be auto-enabled
    pub should_enable_think_more: bool,
    /// Reasoning for the assessment
    pub reasoning: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_creation() {
        let detector = ComplexityDetector::new(ComplexityLevel::Complex);
        assert_eq!(detector.get_threshold(), ComplexityLevel::Complex);
    }

    #[test]
    fn test_default_threshold() {
        let detector = ComplexityDetector::default_threshold();
        assert_eq!(detector.get_threshold(), ComplexityLevel::Complex);
    }

    #[test]
    fn test_detect_simple_task() {
        let detector = ComplexityDetector::default();
        let complexity = detector.detect_complexity("Write a simple function");
        assert_eq!(complexity, ComplexityLevel::Simple);
    }

    #[test]
    fn test_detect_moderate_task() {
        let detector = ComplexityDetector::default();
        let complexity = detector.detect_complexity(
            "Implement a sorting algorithm with optimization and performance considerations"
        );
        assert!(matches!(complexity, ComplexityLevel::Moderate | ComplexityLevel::Complex));
    }

    #[test]
    fn test_detect_complex_task() {
        let detector = ComplexityDetector::default();
        let complexity = detector.detect_complexity(
            "Design and implement a distributed concurrent system with complex algorithm optimization, \
             performance tuning, and architectural considerations for scalability"
        );
        assert_eq!(complexity, ComplexityLevel::Complex);
    }

    #[test]
    fn test_should_auto_enable_complex() {
        let detector = ComplexityDetector::new(ComplexityLevel::Complex);
        assert!(detector.should_auto_enable(ComplexityLevel::Complex));
    }

    #[test]
    fn test_should_not_auto_enable_simple() {
        let detector = ComplexityDetector::new(ComplexityLevel::Complex);
        assert!(!detector.should_auto_enable(ComplexityLevel::Simple));
    }

    #[test]
    fn test_should_auto_enable_moderate_with_moderate_threshold() {
        let detector = ComplexityDetector::new(ComplexityLevel::Moderate);
        assert!(detector.should_auto_enable(ComplexityLevel::Moderate));
    }

    #[test]
    fn test_set_threshold() {
        let mut detector = ComplexityDetector::new(ComplexityLevel::Complex);
        detector.set_threshold(ComplexityLevel::Simple);
        assert_eq!(detector.get_threshold(), ComplexityLevel::Simple);
    }

    #[test]
    fn test_analyze_task() {
        let detector = ComplexityDetector::default();
        let analysis = detector.analyze_task("Write a complex distributed system");
        assert!(!analysis.reasoning.is_empty());
        assert!(analysis.score > 0.0);
    }

    #[test]
    fn test_complexity_keywords_detection() {
        let detector = ComplexityDetector::default();
        let score1 = detector.calculate_complexity_score("Simple task");
        let score2 = detector.calculate_complexity_score("Complex algorithm optimization");
        assert!(score2 > score1);
    }

    #[test]
    fn test_technical_keywords_detection() {
        let detector = ComplexityDetector::default();
        let score1 = detector.calculate_complexity_score("Write code");
        let score2 = detector.calculate_complexity_score("Implement concurrent distributed system");
        assert!(score2 > score1);
    }

    #[test]
    fn test_length_factor() {
        let detector = ComplexityDetector::default();
        let short = "Do something";
        let long = "Do something very complex and intricate with many considerations and requirements and factors";
        let score1 = detector.calculate_complexity_score(short);
        let score2 = detector.calculate_complexity_score(long);
        assert!(score2 > score1);
    }

    #[test]
    fn test_default_implementation() {
        let detector = ComplexityDetector::default();
        assert_eq!(detector.get_threshold(), ComplexityLevel::Complex);
    }

    #[test]
    fn test_score_to_complexity_simple() {
        let detector = ComplexityDetector::default();
        let complexity = detector.score_to_complexity(1.0);
        assert_eq!(complexity, ComplexityLevel::Simple);
    }

    #[test]
    fn test_score_to_complexity_moderate() {
        let detector = ComplexityDetector::default();
        let complexity = detector.score_to_complexity(3.0);
        assert_eq!(complexity, ComplexityLevel::Moderate);
    }

    #[test]
    fn test_score_to_complexity_complex() {
        let detector = ComplexityDetector::default();
        let complexity = detector.score_to_complexity(10.0);
        assert_eq!(complexity, ComplexityLevel::Complex);
    }

    #[test]
    fn test_analysis_includes_reasoning() {
        let detector = ComplexityDetector::default();
        let analysis = detector.analyze_task("Implement complex algorithm");
        assert!(analysis.reasoning.contains("Complexity Level"));
        assert!(analysis.reasoning.contains("Complexity Score"));
    }
}
