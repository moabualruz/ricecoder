//! AI processing for RiceGrep
//!
//! This module provides heuristic-based AI processing for semantic search
//! and natural language understanding capabilities. No external APIs required.

use crate::error::RiceGrepError;
use crate::ai::{QueryUnderstanding, QueryIntent};
use std::collections::HashMap;

/// AI processor using pure heuristic-based processing (no external APIs)
pub struct EmbeddedAIProcessor {
    // Pure heuristic processing - no external dependencies
}

impl EmbeddedAIProcessor {
    /// Create a new AI processor
    pub fn new() -> Self {
        // Pure heuristic processing - no external dependencies
        Self {}
    }







    /// Process query using heuristic-based natural language understanding
    pub async fn process_query(&self, query: &str) -> Result<QueryUnderstanding, RiceGrepError> {
        // Use improved heuristics (OpenAI integration disabled)
        self.process_query_with_heuristics(query)
    }

    // OpenAI LLM integration disabled for now
    // TODO: Re-enable when async-openai API compatibility is resolved

    /// Fallback heuristic processing
    fn process_query_with_heuristics(&self, query: &str) -> Result<QueryUnderstanding, RiceGrepError> {
        let intent = self.classify_intent(query);
        let search_terms = self.extract_search_terms(query);
        let language_context = self.detect_language_context(query);

        Ok(QueryUnderstanding {
            original_query: query.to_string(),
            intent,
            search_terms,
            language_context,
            context: HashMap::new(),
            confidence: 0.6, // Lower confidence for heuristics
        })
    }

    /// Parse intent string to enum
    fn parse_intent(intent_str: &str) -> QueryIntent {
        match intent_str.to_lowercase().as_str() {
            "search" => QueryIntent::Search,
            "find" => QueryIntent::Find,
            "locate" => QueryIntent::Locate,
            "definition" => QueryIntent::Definition,
            "references" => QueryIntent::References,
            "similar" => QueryIntent::Similar,
            "explain" => QueryIntent::Explain,
            _ => QueryIntent::Search,
        }
    }



    /// Rerank search results using heuristic-based semantic understanding
    pub async fn rerank_results(&self, results: &mut crate::search::SearchResults, query: &str) -> Result<(), RiceGrepError> {
        // Use heuristic reranking (OpenAI integration disabled)
        self.rerank_with_semantic_heuristics(results, query).await
    }

    // OpenAI LLM reranking disabled for now
    // TODO: Re-enable when async-openai API compatibility is resolved

    /// Improved semantic reranking using advanced heuristics
    async fn rerank_with_semantic_heuristics(&self, results: &mut crate::search::SearchResults, query: &str) -> Result<(), RiceGrepError> {
        let understanding = self.process_query_with_heuristics(query)?;

        // Enhanced semantic reranking
        for match_result in &mut results.matches {
            let mut score = 0.0f32;

            // Semantic term matching with context awareness
            for term in &understanding.search_terms {
                let term_lower = term.to_lowercase();
                let content_lower = match_result.line_content.to_lowercase();

                if content_lower.contains(&term_lower) {
                    score += 1.0;

                    // Bonus for exact matches
                    if match_result.line_content.contains(term) {
                        score += 0.5;
                    }

                    // Semantic context bonuses
                    score += self.calculate_semantic_bonus(&understanding, &match_result.line_content, term);
                }
            }

            // Language and context awareness
            score += self.calculate_context_bonus(&understanding, match_result);

            // Position and relevance bonuses
            score += self.calculate_position_bonus(match_result);

            match_result.ai_score = Some(score);
        }

        // Sort by semantic score
        results.matches.sort_by(|a, b| {
            let a_score = a.ai_score.unwrap_or(0.0);
            let b_score = b.ai_score.unwrap_or(0.0);
            b_score.partial_cmp(&a_score).unwrap_or(std::cmp::Ordering::Equal)
        });

        results.ai_reranked = true;
        Ok(())
    }

    /// Calculate semantic bonus based on intent and content
    fn calculate_semantic_bonus(&self, understanding: &QueryUnderstanding, content: &str, term: &str) -> f32 {
        let content_lower = content.to_lowercase();
        let mut bonus = 0.0;

        match understanding.intent {
            QueryIntent::Definition => {
                if content_lower.contains("def ") || content_lower.contains("fn ") ||
                   content_lower.contains("class ") || content_lower.contains("struct ") ||
                   content_lower.contains("interface ") {
                    bonus += 0.8;
                }
                // Bonus for terms near definition keywords
                if self.is_near_definition(content, term) {
                    bonus += 0.5;
                }
            }
            QueryIntent::References => {
                if content_lower.contains("use ") || content_lower.contains("import ") ||
                   content_lower.contains("from ") || content_lower.contains("require ") {
                    bonus += 0.6;
                }
            }
            QueryIntent::Find => {
                // General finding - context-aware bonuses
                if content_lower.contains("todo") || content_lower.contains("fixme") ||
                   content_lower.contains("hack") {
                    bonus += 0.3; // Important markers
                }
            }
            QueryIntent::Similar => {
                // For similarity, boost based on term frequency in context
                bonus += 0.2;
            }
            QueryIntent::Explain => {
                if content_lower.contains("comment") || content_lower.contains("//") ||
                   content_lower.contains("/*") || content_lower.contains("# ") {
                    bonus += 0.4;
                }
            }
            _ => {}
        }

        bonus
    }

    /// Calculate context and language bonuses
    fn calculate_context_bonus(&self, understanding: &QueryUnderstanding, match_result: &crate::search::SearchMatch) -> f32 {
        let mut bonus = 0.0;

        // Language context bonus
        if let Some(ref query_lang) = understanding.language_context {
            if let Some(ref detected_lang) = match_result.language {
                if detected_lang.to_lowercase().contains(&query_lang.to_lowercase()) {
                    bonus += 0.4;
                }
            }
        }

        // Programming paradigm bonuses
        let content_lower = match_result.line_content.to_lowercase();
        if understanding.intent == QueryIntent::Definition {
            if content_lower.contains("async") || content_lower.contains("await") {
                bonus += 0.2; // Async programming
            }
            if content_lower.contains("trait") || content_lower.contains("impl") {
                bonus += 0.3; // OOP/traits
            }
        }

        bonus
    }

    /// Calculate position and structural bonuses
    fn calculate_position_bonus(&self, match_result: &crate::search::SearchMatch) -> f32 {
        let mut bonus = 0.0;

        // Position bonuses
        if match_result.line_number <= 10 {
            bonus += 0.2; // Early in file
        } else if match_result.line_number <= 50 {
            bonus += 0.1; // Still early
        }

        // Structural bonuses
        let content_lower = match_result.line_content.to_lowercase();
        if content_lower.trim_start().starts_with("//") ||
           content_lower.trim_start().starts_with("#") ||
           content_lower.trim_start().starts_with("/*") {
            bonus += 0.1; // Comments are often important
        }

        bonus
    }

    /// Check if a term is near definition keywords
    fn is_near_definition(&self, content: &str, term: &str) -> bool {
        let content_lower = content.to_lowercase();
        let term_lower = term.to_lowercase();

        // Look for term within 20 characters of definition keywords
        for keyword in &["def ", "fn ", "class ", "struct ", "interface ", "type "] {
            if let Some(pos) = content_lower.find(keyword) {
                let start = pos.saturating_sub(20);
                let end = (pos + keyword.len() + 20).min(content.len());
                let context = &content_lower[start..end];
                if context.contains(&term_lower) {
                    return true;
                }
            }
        }

        false
    }

    /// Generate answer using embedded model
    pub async fn generate_answer(&self, query: &str, results: &crate::search::SearchResults) -> Result<String, RiceGrepError> {
        if results.matches.is_empty() {
            return Ok(format!("No matches found for query: {}", query));
        }

        let mut answer = format!("Based on your query '{}', I found {} matches:\n\n", query, results.total_matches);

        // Group by file and show top matches
        let mut file_matches: HashMap<&std::path::Path, Vec<&crate::search::SearchMatch>> = HashMap::new();
        for match_result in &results.matches {
            file_matches.entry(&match_result.file).or_insert_with(Vec::new).push(match_result);
        }

        // Show top 3 files with matches
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

    /// Classify query intent (enhanced version)
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

    /// Extract search terms with better understanding
    fn extract_search_terms(&self, query: &str) -> Vec<String> {
        let words: Vec<&str> = query.split_whitespace().collect();
        let mut terms = Vec::new();

        for word in &words {
            let word_lower = word.to_lowercase();

            // Skip stop words but keep programming terms and important keywords
            if !self.is_stop_word(&word_lower) ||
               self.is_programming_keyword(&word_lower) ||
               self.is_intent_keyword(&word_lower) {
                terms.push(word.to_string());
            }
        }

        // If no terms extracted, try to find programming-related patterns
        if terms.is_empty() {
            for word in &words {
                let word_lower = word.to_lowercase();
                if word_lower.contains("func") || word_lower.contains("class") ||
                   word_lower.contains("method") || word_lower.contains("variable") ||
                   word_lower.contains("const") || word_lower.contains("def") ||
                   word_lower.contains("fn") || word_lower.contains("struct") {
                    terms.push(word.to_string());
                }
            }
        }

        terms
    }

    /// Detect language context with better accuracy
    fn detect_language_context(&self, query: &str) -> Option<String> {
        let query_lower = query.to_lowercase();

        if query_lower.contains("rust") || query_lower.contains("fn ") ||
           query_lower.contains("impl") || query_lower.contains("trait") ||
           query_lower.contains("struct") || query_lower.contains("enum") {
            Some("rust".to_string())
        } else if query_lower.contains("python") || query_lower.contains("def ") ||
                  query_lower.contains("import ") || query_lower.contains("class ") ||
                  query_lower.contains("self.") {
            Some("python".to_string())
        } else if query_lower.contains("javascript") || query_lower.contains("typescript") ||
                  query_lower.contains("function") || query_lower.contains("const ") ||
                  query_lower.contains("let ") {
            Some("javascript".to_string())
        } else if query_lower.contains("java") || query_lower.contains("public class") ||
                  query_lower.contains("private ") || query_lower.contains("protected ") {
            Some("java".to_string())
        } else if query_lower.contains("go ") || query_lower.contains("golang") ||
                  query_lower.contains("func ") || query_lower.contains("package ") {
            Some("go".to_string())
        } else {
            None
        }
    }

    /// Check if word is a stop word
    fn is_stop_word(&self, word: &str) -> bool {
        matches!(word, "the" | "a" | "an" | "and" | "or" | "but" | "in" | "on" |
                       "at" | "to" | "for" | "of" | "with" | "by" | "is" | "are" |
                       "was" | "were" | "be" | "been" | "being" | "have" | "has" |
                       "had" | "do" | "does" | "did" | "will" | "would" | "could" |
                       "should" | "may" | "might" | "must" | "can" | "find" | "search" |
                       "locate" | "get" | "show" | "display" | "print" | "help" | "exit" |
                       "quit" | "run" | "build" | "test" | "check" | "fix" | "add" | "remove" |
                       "update" | "change" | "set" | "call" | "return")
    }

    /// Check if word is a programming keyword
    fn is_programming_keyword(&self, word: &str) -> bool {
        matches!(word, "function" | "variable" | "class" | "struct" | "enum" | "const" |
                       "let" | "mut" | "fn" | "return" | "for" | "while" | "match" | "async" |
                       "await" | "string" | "vector" | "error" | "result" | "option" | "clone" |
                       "copy" | "move" | "borrow" | "lifetime" | "generic" | "macro" | "derive" |
                       "debug" | "display" | "default" | "partialeq" | "eq" | "hash" | "serde" |
                       "json" | "file" | "path" | "read" | "write" | "open" | "close" | "create" |
                       "delete" | "copy" | "move" | "list" | "show")
    }

    /// Check if word is an intent keyword
    fn is_intent_keyword(&self, word: &str) -> bool {
        matches!(word, "definition" | "def" | "reference" | "usage" | "similar" | "like" |
                        "explain" | "what" | "how" | "where" | "which" | "when" | "why")
    }
}
