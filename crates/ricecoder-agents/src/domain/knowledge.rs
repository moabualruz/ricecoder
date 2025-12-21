//! Knowledge base for domain-specific expertise

use crate::domain::error::{DomainError, DomainResult};
use crate::domain::models::{AntiPattern, BestPractice, Pattern, TechRecommendation};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Knowledge base for domain expertise
///
/// This struct stores and retrieves domain-specific knowledge,
/// including best practices, technology recommendations, patterns, and anti-patterns.
///
/// # Examples
///
/// ```ignore
/// use ricecoder_agents::domain::KnowledgeBase;
///
/// let kb = KnowledgeBase::new();
/// let recommendations = kb.get_recommendations("web", "framework")?;
/// ```
#[derive(Debug, Clone)]
pub struct KnowledgeBase {
    best_practices: Arc<RwLock<HashMap<String, Vec<BestPractice>>>>,
    tech_recommendations: Arc<RwLock<HashMap<String, Vec<TechRecommendation>>>>,
    patterns: Arc<RwLock<HashMap<String, Vec<Pattern>>>>,
    anti_patterns: Arc<RwLock<HashMap<String, Vec<AntiPattern>>>>,
}

impl KnowledgeBase {
    /// Create a new knowledge base
    pub fn new() -> Self {
        Self {
            best_practices: Arc::new(RwLock::new(HashMap::new())),
            tech_recommendations: Arc::new(RwLock::new(HashMap::new())),
            patterns: Arc::new(RwLock::new(HashMap::new())),
            anti_patterns: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a best practice
    ///
    /// # Arguments
    ///
    /// * `domain` - Domain identifier
    /// * `practice` - Best practice to add
    pub fn add_best_practice(&self, domain: &str, practice: BestPractice) -> DomainResult<()> {
        let mut practices = self
            .best_practices
            .write()
            .map_err(|e| DomainError::internal(format!("Failed to acquire write lock: {}", e)))?;

        practices
            .entry(domain.to_string())
            .or_default()
            .push(practice);

        Ok(())
    }

    /// Get best practices for a domain
    ///
    /// # Arguments
    ///
    /// * `domain` - Domain identifier
    ///
    /// # Returns
    ///
    /// Returns a vector of best practices for the domain
    pub fn get_best_practices(&self, domain: &str) -> DomainResult<Vec<BestPractice>> {
        let practices = self
            .best_practices
            .read()
            .map_err(|e| DomainError::internal(format!("Failed to acquire read lock: {}", e)))?;

        Ok(practices.get(domain).cloned().unwrap_or_default())
    }

    /// Add a technology recommendation
    ///
    /// # Arguments
    ///
    /// * `domain` - Domain identifier
    /// * `recommendation` - Technology recommendation to add
    pub fn add_tech_recommendation(
        &self,
        domain: &str,
        recommendation: TechRecommendation,
    ) -> DomainResult<()> {
        let mut recommendations = self
            .tech_recommendations
            .write()
            .map_err(|e| DomainError::internal(format!("Failed to acquire write lock: {}", e)))?;

        recommendations
            .entry(domain.to_string())
            .or_default()
            .push(recommendation);

        Ok(())
    }

    /// Get technology recommendations for a domain
    ///
    /// # Arguments
    ///
    /// * `domain` - Domain identifier
    ///
    /// # Returns
    ///
    /// Returns a vector of technology recommendations for the domain
    pub fn get_tech_recommendations(&self, domain: &str) -> DomainResult<Vec<TechRecommendation>> {
        let recommendations = self
            .tech_recommendations
            .read()
            .map_err(|e| DomainError::internal(format!("Failed to acquire read lock: {}", e)))?;

        Ok(recommendations.get(domain).cloned().unwrap_or_default())
    }

    /// Get technology recommendation by technology name
    ///
    /// # Arguments
    ///
    /// * `domain` - Domain identifier
    /// * `technology` - Technology name
    ///
    /// # Returns
    ///
    /// Returns the technology recommendation if found
    pub fn get_tech_recommendation(
        &self,
        domain: &str,
        technology: &str,
    ) -> DomainResult<TechRecommendation> {
        let recommendations = self.get_tech_recommendations(domain)?;

        recommendations
            .into_iter()
            .find(|r| r.technology == technology)
            .ok_or_else(|| DomainError::knowledge_not_found(technology))
    }

    /// Add a pattern
    ///
    /// # Arguments
    ///
    /// * `domain` - Domain identifier
    /// * `pattern` - Pattern to add
    pub fn add_pattern(&self, domain: &str, pattern: Pattern) -> DomainResult<()> {
        let mut patterns = self
            .patterns
            .write()
            .map_err(|e| DomainError::internal(format!("Failed to acquire write lock: {}", e)))?;

        patterns
            .entry(domain.to_string())
            .or_default()
            .push(pattern);

        Ok(())
    }

    /// Get patterns for a domain
    ///
    /// # Arguments
    ///
    /// * `domain` - Domain identifier
    ///
    /// # Returns
    ///
    /// Returns a vector of patterns for the domain
    pub fn get_patterns(&self, domain: &str) -> DomainResult<Vec<Pattern>> {
        let patterns = self
            .patterns
            .read()
            .map_err(|e| DomainError::internal(format!("Failed to acquire read lock: {}", e)))?;

        Ok(patterns.get(domain).cloned().unwrap_or_default())
    }

    /// Add an anti-pattern
    ///
    /// # Arguments
    ///
    /// * `domain` - Domain identifier
    /// * `anti_pattern` - Anti-pattern to add
    pub fn add_anti_pattern(&self, domain: &str, anti_pattern: AntiPattern) -> DomainResult<()> {
        let mut anti_patterns = self
            .anti_patterns
            .write()
            .map_err(|e| DomainError::internal(format!("Failed to acquire write lock: {}", e)))?;

        anti_patterns
            .entry(domain.to_string())
            .or_default()
            .push(anti_pattern);

        Ok(())
    }

    /// Get anti-patterns for a domain
    ///
    /// # Arguments
    ///
    /// * `domain` - Domain identifier
    ///
    /// # Returns
    ///
    /// Returns a vector of anti-patterns for the domain
    pub fn get_anti_patterns(&self, domain: &str) -> DomainResult<Vec<AntiPattern>> {
        let anti_patterns = self
            .anti_patterns
            .read()
            .map_err(|e| DomainError::internal(format!("Failed to acquire read lock: {}", e)))?;

        Ok(anti_patterns.get(domain).cloned().unwrap_or_default())
    }

    /// Clear all knowledge for a domain
    ///
    /// # Arguments
    ///
    /// * `domain` - Domain identifier
    pub fn clear_domain(&self, domain: &str) -> DomainResult<()> {
        let mut practices = self
            .best_practices
            .write()
            .map_err(|e| DomainError::internal(format!("Failed to acquire write lock: {}", e)))?;
        practices.remove(domain);

        let mut recommendations = self
            .tech_recommendations
            .write()
            .map_err(|e| DomainError::internal(format!("Failed to acquire write lock: {}", e)))?;
        recommendations.remove(domain);

        let mut patterns = self
            .patterns
            .write()
            .map_err(|e| DomainError::internal(format!("Failed to acquire write lock: {}", e)))?;
        patterns.remove(domain);

        let mut anti_patterns = self
            .anti_patterns
            .write()
            .map_err(|e| DomainError::internal(format!("Failed to acquire write lock: {}", e)))?;
        anti_patterns.remove(domain);

        Ok(())
    }

    /// Clear all knowledge
    pub fn clear_all(&self) -> DomainResult<()> {
        let mut practices = self
            .best_practices
            .write()
            .map_err(|e| DomainError::internal(format!("Failed to acquire write lock: {}", e)))?;
        practices.clear();

        let mut recommendations = self
            .tech_recommendations
            .write()
            .map_err(|e| DomainError::internal(format!("Failed to acquire write lock: {}", e)))?;
        recommendations.clear();

        let mut patterns = self
            .patterns
            .write()
            .map_err(|e| DomainError::internal(format!("Failed to acquire write lock: {}", e)))?;
        patterns.clear();

        let mut anti_patterns = self
            .anti_patterns
            .write()
            .map_err(|e| DomainError::internal(format!("Failed to acquire write lock: {}", e)))?;
        anti_patterns.clear();

        Ok(())
    }
}

impl Default for KnowledgeBase {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_practice(domain: &str) -> BestPractice {
        BestPractice {
            title: "Test Practice".to_string(),
            description: "A test practice".to_string(),
            domain: domain.to_string(),
            technologies: vec!["Tech1".to_string()],
            implementation: "Implementation".to_string(),
        }
    }

    fn create_test_recommendation(domain: &str) -> TechRecommendation {
        TechRecommendation {
            technology: "React".to_string(),
            domain: domain.to_string(),
            use_cases: vec!["SPAs".to_string()],
            pros: vec!["Ecosystem".to_string()],
            cons: vec!["Learning curve".to_string()],
            alternatives: vec!["Vue".to_string()],
        }
    }

    fn create_test_pattern(domain: &str) -> Pattern {
        Pattern {
            name: "Test Pattern".to_string(),
            description: "A test pattern".to_string(),
            domain: domain.to_string(),
            technologies: vec!["Tech1".to_string()],
            use_cases: vec!["Use case 1".to_string()],
        }
    }

    fn create_test_anti_pattern(domain: &str) -> AntiPattern {
        AntiPattern {
            name: "Test Anti-pattern".to_string(),
            description: "A test anti-pattern".to_string(),
            domain: domain.to_string(),
            why_avoid: "Reason".to_string(),
            better_alternative: "Alternative".to_string(),
        }
    }

    #[test]
    fn test_knowledge_base_creation() {
        let kb = KnowledgeBase::new();
        assert!(kb.get_best_practices("web").unwrap().is_empty());
    }

    #[test]
    fn test_add_best_practice() {
        let kb = KnowledgeBase::new();
        let practice = create_test_practice("web");

        kb.add_best_practice("web", practice).unwrap();
        let practices = kb.get_best_practices("web").unwrap();

        assert_eq!(practices.len(), 1);
        assert_eq!(practices[0].title, "Test Practice");
    }

    #[test]
    fn test_add_tech_recommendation() {
        let kb = KnowledgeBase::new();
        let recommendation = create_test_recommendation("web");

        kb.add_tech_recommendation("web", recommendation).unwrap();
        let recommendations = kb.get_tech_recommendations("web").unwrap();

        assert_eq!(recommendations.len(), 1);
        assert_eq!(recommendations[0].technology, "React");
    }

    #[test]
    fn test_get_tech_recommendation() {
        let kb = KnowledgeBase::new();
        let recommendation = create_test_recommendation("web");

        kb.add_tech_recommendation("web", recommendation).unwrap();
        let retrieved = kb.get_tech_recommendation("web", "React").unwrap();

        assert_eq!(retrieved.technology, "React");
    }

    #[test]
    fn test_get_nonexistent_tech_recommendation() {
        let kb = KnowledgeBase::new();
        let result = kb.get_tech_recommendation("web", "NonExistent");

        assert!(result.is_err());
    }

    #[test]
    fn test_add_pattern() {
        let kb = KnowledgeBase::new();
        let pattern = create_test_pattern("web");

        kb.add_pattern("web", pattern).unwrap();
        let patterns = kb.get_patterns("web").unwrap();

        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].name, "Test Pattern");
    }

    #[test]
    fn test_add_anti_pattern() {
        let kb = KnowledgeBase::new();
        let anti_pattern = create_test_anti_pattern("web");

        kb.add_anti_pattern("web", anti_pattern).unwrap();
        let anti_patterns = kb.get_anti_patterns("web").unwrap();

        assert_eq!(anti_patterns.len(), 1);
        assert_eq!(anti_patterns[0].name, "Test Anti-pattern");
    }

    #[test]
    fn test_clear_domain() {
        let kb = KnowledgeBase::new();

        kb.add_best_practice("web", create_test_practice("web"))
            .unwrap();
        kb.add_tech_recommendation("web", create_test_recommendation("web"))
            .unwrap();

        kb.clear_domain("web").unwrap();

        assert!(kb.get_best_practices("web").unwrap().is_empty());
        assert!(kb.get_tech_recommendations("web").unwrap().is_empty());
    }

    #[test]
    fn test_clear_all() {
        let kb = KnowledgeBase::new();

        kb.add_best_practice("web", create_test_practice("web"))
            .unwrap();
        kb.add_best_practice("backend", create_test_practice("backend"))
            .unwrap();

        kb.clear_all().unwrap();

        assert!(kb.get_best_practices("web").unwrap().is_empty());
        assert!(kb.get_best_practices("backend").unwrap().is_empty());
    }

    #[test]
    fn test_multiple_domains() {
        let kb = KnowledgeBase::new();

        kb.add_best_practice("web", create_test_practice("web"))
            .unwrap();
        kb.add_best_practice("backend", create_test_practice("backend"))
            .unwrap();

        let web_practices = kb.get_best_practices("web").unwrap();
        let backend_practices = kb.get_best_practices("backend").unwrap();

        assert_eq!(web_practices.len(), 1);
        assert_eq!(backend_practices.len(), 1);
    }

    #[test]
    fn test_default_knowledge_base() {
        let kb = KnowledgeBase::default();
        assert!(kb.get_best_practices("web").unwrap().is_empty());
    }

    #[test]
    fn test_multiple_practices_per_domain() {
        let kb = KnowledgeBase::new();

        kb.add_best_practice("web", create_test_practice("web"))
            .unwrap();
        let mut practice2 = create_test_practice("web");
        practice2.title = "Practice 2".to_string();
        kb.add_best_practice("web", practice2).unwrap();

        let practices = kb.get_best_practices("web").unwrap();
        assert_eq!(practices.len(), 2);
    }

    #[test]
    fn test_multiple_recommendations_per_domain() {
        let kb = KnowledgeBase::new();

        kb.add_tech_recommendation("web", create_test_recommendation("web"))
            .unwrap();
        let mut rec2 = create_test_recommendation("web");
        rec2.technology = "Vue".to_string();
        kb.add_tech_recommendation("web", rec2).unwrap();

        let recommendations = kb.get_tech_recommendations("web").unwrap();
        assert_eq!(recommendations.len(), 2);
    }
}
