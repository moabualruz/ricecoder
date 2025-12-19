//! AI processing for natural language queries and result reranking
//!
//! This module provides AI-powered enhancements to ricegrep including:
//! - Natural language query understanding
//! - Intelligent result reranking
//! - Programming domain knowledge
//! - Query intent classification

use crate::error::RiceGrepError;
use crate::search::{SearchQuery, SearchResults, SearchMatch};
use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Query understanding result from AI processing
#[derive(Debug, Clone)]
pub struct QueryUnderstanding {
    /// Original natural language query
    pub original_query: String,
    /// Classified intent (search, find, locate, etc.)
    pub intent: QueryIntent,
    /// Extracted search terms
    pub search_terms: Vec<String>,
    /// Programming language context (if detected)
    pub language_context: Option<String>,
    /// Additional context or constraints
    pub context: HashMap<String, String>,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
}

/// Query intent classification
#[derive(Debug, Clone, PartialEq)]
pub enum QueryIntent {
    /// General search
    Search,
    /// Find specific function/class
    Find,
    /// Locate usage of something
    Locate,
    /// Find definitions
    Definition,
    /// Find references/usage
    References,
    /// Find similar code
    Similar,
    /// Explain or understand code
    Explain,
}

/// AI processor for query understanding and result enhancement
#[async_trait]
pub trait AIProcessor: Send + Sync {
    /// Process natural language query into structured understanding
    async fn process_query(&self, query: &str) -> Result<QueryUnderstanding, RiceGrepError>;

    /// Rerank search results based on AI understanding
    async fn rerank_results(&mut self, results: &mut SearchResults, query: &str) -> Result<(), RiceGrepError>;

    /// Generate an AI-powered answer based on search results
    async fn generate_answer(&self, query: &str, results: &SearchResults) -> Result<String, RiceGrepError>;

    /// Get confidence threshold for AI features
    fn confidence_threshold(&self) -> f32;

    /// Check if AI processing is available
    fn is_available(&self) -> bool;
}

/// Basic AI processor implementation
pub struct RiceGrepAIProcessor {
    /// Confidence threshold
    confidence_threshold: f32,
    /// Programming domain knowledge
    domain_knowledge: DomainKnowledge,
    /// Embedded AI processor for ML-based processing
    embedded_processor: Option<crate::embedded_ai::EmbeddedAIProcessor>,
}



impl RiceGrepAIProcessor {
    /// Create a new AI processor
    pub fn new() -> Self {
        Self {
            confidence_threshold: 0.7,
            domain_knowledge: DomainKnowledge::new(),
            embedded_processor: crate::embedded_ai::EmbeddedAIProcessor::new().ok(),
        }
    }



    /// Set confidence threshold
    pub fn with_confidence_threshold(mut self, threshold: f32) -> Self {
        self.confidence_threshold = threshold;
        self
    }

    /// Create query understanding from natural language
    fn understand_query(&self, query: &str) -> Result<QueryUnderstanding, RiceGrepError> {
        // Use embedded AI processor if available
        if let Some(ref processor) = self.embedded_processor {
            return processor.process_query(query);
        }

        // Fallback to heuristic-based understanding
        let intent = self.classify_intent(query);
        let search_terms = self.extract_search_terms(query);
        let language_context = self.detect_language_context(query);

        Ok(QueryUnderstanding {
            original_query: query.to_string(),
            intent,
            search_terms,
            language_context,
            context: HashMap::new(),
            confidence: 0.7, // Lower confidence for heuristic approach
        })
    }

    /// Classify query intent
    fn classify_intent(&self, query: &str) -> QueryIntent {
        let query_lower = query.to_lowercase();

        if query_lower.contains("find") || query_lower.contains("locate") {
            if query_lower.contains("definition") || query_lower.contains("def") {
                QueryIntent::Definition
            } else if query_lower.contains("reference") || query_lower.contains("usage") {
                QueryIntent::References
            } else {
                QueryIntent::Find
            }
        } else if query_lower.contains("similar") || query_lower.contains("like") {
            QueryIntent::Similar
        } else if query_lower.contains("explain") || query_lower.contains("what") {
            QueryIntent::Explain
        } else {
            QueryIntent::Search
        }
    }

    /// Extract search terms from natural language query
    fn extract_search_terms(&self, query: &str) -> Vec<String> {
        // Simple extraction - split on common words and keep programming terms
        let words: Vec<&str> = query.split_whitespace().collect();
        let mut terms = Vec::new();

        for word in &words {
            let word_lower = word.to_lowercase();
            // Skip common English words, but keep programming keywords
            if !self.domain_knowledge.is_stop_word(&word_lower) || self.domain_knowledge.is_keyword(&word_lower, None) {
                terms.push(word.to_string());
            }
        }

        // If no terms extracted, try to find any programming-related words
        if terms.is_empty() {
            for word in &words {
                let word_lower = word.to_lowercase();
                if word_lower.contains("func") || word_lower.contains("class") ||
                   word_lower.contains("var") || word_lower.contains("method") ||
                   word_lower.contains("def") || word_lower.contains("fn") {
                    terms.push(word.to_string());
                }
            }
        }

        // Fallback: if still no terms, use all non-stop words
        if terms.is_empty() {
            for word in &words {
                if !self.domain_knowledge.is_stop_word(&word.to_lowercase()) {
                    terms.push(word.to_string());
                }
            }
        }

        terms
    }

    /// Detect programming language context
    fn detect_language_context(&self, query: &str) -> Option<String> {
        // Simple language detection based on keywords
        let query_lower = query.to_lowercase();

        if query_lower.contains("rust") || query_lower.contains("fn ") || query_lower.contains("impl") {
            Some("rust".to_string())
        } else if query_lower.contains("python") || query_lower.contains("def ") || query_lower.contains("import ") {
            Some("python".to_string())
        } else if query_lower.contains("javascript") || query_lower.contains("typescript") || query_lower.contains("function") {
            Some("javascript".to_string())
        } else if query_lower.contains("java") || query_lower.contains("class ") {
            Some("java".to_string())
        } else {
            None
        }
    }

    /// Rerank results using embedded AI or fallback to basic method
    async fn rerank_results(&self, results: &mut SearchResults, query: &str) -> Result<(), RiceGrepError> {
        // Use embedded AI processor if available
        if let Some(ref processor) = self.embedded_processor {
            return processor.rerank_results(results, query);
        }

        // Fallback to basic reranking
        self.rerank_results_basic(results, query).await;
        Ok(())
    }

    /// Basic reranking based on term frequency and position (fallback)
    async fn rerank_results_basic(&self, results: &mut SearchResults, query: &str) {
        // Extract search terms from query
        let search_terms = self.extract_search_terms(query);

        // Simple reranking based on term frequency and position
        for match_result in &mut results.matches {
            let mut score = 0.0f32;

            // Boost score for matches containing search terms
            for term in &search_terms {
                if match_result.line_content.to_lowercase().contains(&term.to_lowercase()) {
                    score += 1.0;

                    // Extra boost for exact matches
                    if match_result.line_content.contains(term) {
                        score += 0.5;
                    }
                }
            }

            // Boost for matches near the beginning of files
            if match_result.line_number <= 10 {
                score += 0.2;
            }

            match_result.ai_score = Some(score);
        }

        // Sort by AI score (descending)
        results.matches.sort_by(|a, b| {
            let a_score = a.ai_score.unwrap_or(0.0);
            let b_score = b.ai_score.unwrap_or(0.0);
            b_score.partial_cmp(&a_score).unwrap_or(std::cmp::Ordering::Equal)
        });

        results.ai_reranked = true;
    }
}

/// Programming domain knowledge for query processing
pub struct DomainKnowledge {
    /// Stop words to filter out
    stop_words: HashSet<String>,
    /// Programming keywords by language
    language_keywords: HashMap<String, HashSet<String>>,
}

impl DomainKnowledge {
    /// Create new domain knowledge
    pub fn new() -> Self {
        let mut stop_words = HashSet::new();
        for word in ["the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by", "is", "are", "was", "were", "be", "been", "being", "have", "has", "had", "do", "does", "did", "will", "would", "could", "should", "may", "might", "must", "can", "find", "search", "locate", "get", "show", "display", "print"] {
            stop_words.insert(word.to_string());
        }

        let mut language_keywords = HashMap::new();

        // Rust keywords
        let mut rust_keywords = HashSet::new();
        for kw in ["fn", "struct", "enum", "impl", "trait", "let", "mut", "const", "static", "pub", "crate", "mod", "use", "as", "type", "where", "async", "await", "move", "clone"] {
            rust_keywords.insert(kw.to_string());
        }
        language_keywords.insert("rust".to_string(), rust_keywords);

        // Python keywords
        let mut python_keywords = HashSet::new();
        for kw in ["def", "class", "import", "from", "if", "elif", "else", "for", "while", "try", "except", "with", "as", "lambda", "self", "return", "yield", "async", "await"] {
            python_keywords.insert(kw.to_string());
        }
        language_keywords.insert("python".to_string(), python_keywords);

        Self {
            stop_words,
            language_keywords,
        }
    }

    /// Check if word is a stop word
    pub fn is_stop_word(&self, word: &str) -> bool {
        self.stop_words.contains(word)
    }

    /// Check if word is a programming keyword
    pub fn is_keyword(&self, word: &str, language: Option<&str>) -> bool {
        if let Some(lang) = language {
            if let Some(keywords) = self.language_keywords.get(lang) {
                return keywords.contains(word);
            }
        }

        // Check all languages
        for keywords in self.language_keywords.values() {
            if keywords.contains(word) {
                return true;
            }
        }

        false
    }
}

impl Default for DomainKnowledge {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AIProcessor for RiceGrepAIProcessor {
    async fn process_query(&self, query: &str) -> Result<QueryUnderstanding, RiceGrepError> {
        // Use embedded AI processor or fallback to heuristics
        self.understand_query(query)
    }

    async fn rerank_results(&mut self, results: &mut SearchResults, query: &str) -> Result<(), RiceGrepError> {
        // Use embedded AI processor or fallback to basic reranking
        self.rerank_results(results, query).await
    }

    async fn generate_answer(&self, query: &str, results: &SearchResults) -> Result<String, RiceGrepError> {
        // Basic answer generation
        if results.matches.is_empty() {
            return Ok(format!("No matches found for query: {}", query));
        }

        let mut answer = format!("Based on your query '{}', I found {} matches across {} files:\n\n",
                                query, results.total_matches, results.files_searched);

        // Group matches by file and show top matches
        let mut file_matches: HashMap<&std::path::Path, Vec<&SearchMatch>> = HashMap::new();
        for match_result in &results.matches {
            file_matches.entry(&match_result.file).or_insert_with(Vec::new).push(match_result);
        }

        // Show summary for top 3 files
        for (file_path, matches) in file_matches.iter().take(3) {
            answer.push_str(&format!("📁 {} ({} matches):\n", file_path.display(), matches.len()));
            for match_result in matches.iter().take(2) {
                answer.push_str(&format!("  Line {}: {}\n",
                                       match_result.line_number,
                                       match_result.line_content.trim()));
            }
            if matches.len() > 2 {
                answer.push_str(&format!("  ... and {} more matches\n", matches.len() - 2));
            }
            answer.push_str("\n");
        }

        if file_matches.len() > 3 {
            answer.push_str(&format!("... and {} more files with matches\n", file_matches.len() - 3));
        }

        Ok(answer)
    }

    fn confidence_threshold(&self) -> f32 {
        self.confidence_threshold
    }

    fn is_available(&self) -> bool {
        // Always available with heuristic processing
        true
    }
}

impl Default for RiceGrepAIProcessor {
    fn default() -> Self {
        Self::new()
    }
}

