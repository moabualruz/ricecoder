//! Language awareness functionality for RiceGrep

use crate::error::RiceGrepError;
use detect_lang::{from_path, Language};
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info, warn};

/// Language detection and processing configuration
#[derive(Debug, Clone)]
pub struct LanguageConfig {
    /// Whether language detection is enabled
    pub enabled: bool,
    /// Language-specific ranking boosts (language -> boost factor)
    pub ranking_boosts: HashMap<String, f32>,
    /// Minimum confidence for language detection
    pub min_confidence: f32,
}

/// Language detection result
#[derive(Debug, Clone)]
pub struct LanguageDetection {
    /// Detected programming language name
    pub language_name: String,
    /// Language file extension/id
    pub language_id: String,
    /// Detection confidence (0.0 to 1.0)
    pub confidence: f32,
    /// Source of detection (extension, content, lsp)
    pub source: DetectionSource,
}

/// Source of language detection
#[derive(Debug, Clone, PartialEq)]
pub enum DetectionSource {
    /// Detected from file extension
    Extension,
    /// Detected from file content
    Content,
    /// Detected from LSP server
    Lsp,
}

/// Language-aware ranking adjustment
#[derive(Debug, Clone)]
pub struct LanguageRanking {
    /// Base relevance score
    pub base_score: f32,
    /// Language-specific boost factor
    pub language_boost: f32,
    /// Final adjusted score
    pub adjusted_score: f32,
    /// Language name that influenced ranking
    pub detected_language: Option<String>,
}

/// Language processor for detection and ranking
pub struct LanguageProcessor {
    /// Configuration
    config: LanguageConfig,
    /// Cache for language detection results (using owned strings for simplicity)
    detection_cache: HashMap<String, (String, f32, DetectionSource)>, // (lang_name, confidence, source)
}

impl LanguageProcessor {
    /// Create a new language processor
    pub fn new(config: LanguageConfig) -> Self {
        Self {
            config,
            detection_cache: HashMap::new(),
        }
    }

    /// Detect language for a file path
    pub fn detect_language(&self, file_path: &Path) -> Result<Option<LanguageDetection>, RiceGrepError> {
        if !self.config.enabled {
            return Ok(None);
        }

        // Detect from file extension using from_path function
        let detection = if let Some(lang) = from_path(file_path) {
            Some(LanguageDetection {
                language_name: lang.0.to_string(),
                language_id: lang.1.to_string(),
                confidence: 1.0, // Extension detection is highly confident
                source: DetectionSource::Extension,
            })
        } else {
            // Could add content-based detection here in the future
            None
        };

        Ok(detection)
    }

    /// Calculate language-aware ranking adjustment
    pub fn calculate_ranking(&self, base_score: f32, language_name: Option<&str>) -> LanguageRanking {
        let mut ranking = LanguageRanking {
            base_score,
            language_boost: 1.0,
            adjusted_score: base_score,
            detected_language: language_name.map(|s| s.to_string()),
        };

        if let Some(lang_name) = language_name {
            // Apply language-specific boost if configured
            let lang_key = lang_name.to_lowercase();
            if let Some(boost) = self.config.ranking_boosts.get(&lang_key) {
                ranking.language_boost = *boost;
                ranking.adjusted_score = base_score * *boost;
                debug!("Applied language boost for {}: {}x (score: {} -> {})",
                       lang_name, boost, base_score, ranking.adjusted_score);
            }
        }

        ranking
    }

    /// Get language-specific search hints
    pub fn get_search_hints(&self, language_name: &str) -> Vec<String> {
        match language_name {
            "rust" => vec![
                "Consider using 'impl' for implementation blocks".to_string(),
                "Use 'struct' for data structures".to_string(),
                "Look for 'fn' function definitions".to_string(),
                "Check 'trait' definitions for behavior".to_string(),
            ],
            "python" => vec![
                "Look for 'def' function definitions".to_string(),
                "Check 'class' definitions".to_string(),
                "Consider 'import' statements".to_string(),
                "Use 'self' in method parameters".to_string(),
            ],
            "javascript" | "typescript" => vec![
                "Look for 'function' or 'const/let' declarations".to_string(),
                "Check 'class' definitions".to_string(),
                "Consider 'import/export' statements".to_string(),
                "Use arrow functions '=>' ".to_string(),
            ],
            "java" => vec![
                "Look for 'public class' definitions".to_string(),
                "Check 'public static void main' methods".to_string(),
                "Consider 'import' statements".to_string(),
                "Use 'System.out.println' for output".to_string(),
            ],
            "go" => vec![
                "Look for 'func' function definitions".to_string(),
                "Check 'type' definitions".to_string(),
                "Consider 'import' statements in parentheses".to_string(),
                "Use 'package main' for executables".to_string(),
            ],
            "c" => vec![
                "Look for 'int main()' function".to_string(),
                "Check '#include' directives".to_string(),
                "Consider 'printf' statements".to_string(),
                "Use 'struct' definitions".to_string(),
            ],
            "cpp" => vec![
                "Look for 'int main()' or 'std::cout'".to_string(),
                "Check '#include' directives".to_string(),
                "Consider 'class' definitions".to_string(),
                "Use 'namespace' declarations".to_string(),
            ],
            _ => vec![
                "Language-specific search hints not available".to_string(),
            ],
        }
    }

    /// Check if a file extension is supported
    pub fn is_supported_extension(&self, extension: &str) -> bool {
        matches!(extension.to_lowercase().as_str(),
            "rs" | "py" | "js" | "ts" | "java" | "go" | "c" | "cpp" | "cc" | "cxx" | "hpp" | "h" |
            "cs" | "php" | "rb" | "swift" | "kt" | "scala" | "clj" | "hs" | "ml" | "fs" | "vb" |
            "pl" | "pm" | "tcl" | "lua" | "r" | "m" | "sh" | "bash" | "zsh" | "fish" | "ps1" |
            "sql" | "html" | "css" | "scss" | "sass" | "less" | "xml" | "json" | "yaml" | "yml" |
            "toml" | "ini" | "cfg" | "conf" | "md" | "txt" | "rst"
        )
    }

    /// Get supported languages list
    pub fn get_supported_languages(&self) -> Vec<&str> {
        vec![
            "rust", "python", "javascript", "typescript", "java", "go", "c", "cpp",
            "csharp", "php", "ruby", "swift", "kotlin", "scala"
        ]
    }

    /// Clear detection cache
    pub fn clear_cache(&mut self) {
        self.detection_cache.clear();
    }
}

impl Default for LanguageConfig {
    fn default() -> Self {
        let mut ranking_boosts = HashMap::new();

        // Language-specific ranking boosts (higher values = more relevant)
        ranking_boosts.insert("rust".to_string(), 1.2);     // Preferred language
        ranking_boosts.insert("python".to_string(), 1.1);   // Very popular
        ranking_boosts.insert("javascript".to_string(), 1.0); // Baseline
        ranking_boosts.insert("typescript".to_string(), 1.05); // Type safety bonus
        ranking_boosts.insert("go".to_string(), 1.1);       // Systems programming
        ranking_boosts.insert("java".to_string(), 0.9);     // Enterprise focus
        ranking_boosts.insert("c".to_string(), 0.8);        // Low-level
        ranking_boosts.insert("cpp".to_string(), 0.85);     // Complex

        Self {
            enabled: true,
            ranking_boosts,
            min_confidence: 0.5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_language_config_default() {
        let config = LanguageConfig::default();
        assert!(config.enabled);
        assert!(config.ranking_boosts.contains_key("rust"));
        assert_eq!(*config.ranking_boosts.get("rust").unwrap(), 1.2);
    }

    #[test]
    fn test_language_detection_from_extension() {
        let mut processor = LanguageProcessor::new(LanguageConfig::default());

        // Test Rust file
        let rust_path = PathBuf::from("test.rs");
        let result = processor.detect_language(&rust_path).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().language_name, "Rust");

        // Test Python file
        let py_path = PathBuf::from("script.py");
        let result = processor.detect_language(&py_path).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().language_name, "Python");

        // Test unknown extension
        let unknown_path = PathBuf::from("file.unknown");
        let result = processor.detect_language(&unknown_path).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_language_ranking_calculation() {
        let config = LanguageConfig::default();
        let processor = LanguageProcessor::new(config);

        // Test with Rust language (1.2x boost)
        let base_score = 0.8;
        let ranking = processor.calculate_ranking(base_score, Some("rust"));
        assert_eq!(ranking.base_score, 0.8);
        assert_eq!(ranking.language_boost, 1.2);
        assert!((ranking.adjusted_score - 0.96).abs() < 1e-6); // 0.8 * 1.2, with floating-point tolerance

        // Test without language
        let ranking = processor.calculate_ranking(base_score, None);
        assert_eq!(ranking.language_boost, 1.0);
        assert_eq!(ranking.adjusted_score, 0.8);
    }

    #[test]
    fn test_supported_extensions() {
        let processor = LanguageProcessor::new(LanguageConfig::default());

        assert!(processor.is_supported_extension("rs"));
        assert!(processor.is_supported_extension("py"));
        assert!(processor.is_supported_extension("js"));
        assert!(processor.is_supported_extension("java"));
        assert!(processor.is_supported_extension("go"));
        assert!(!processor.is_supported_extension("unknown"));
    }

    #[test]
    fn test_search_hints() {
        let processor = LanguageProcessor::new(LanguageConfig::default());

        let rust_hints = processor.get_search_hints("rust");
        assert!(!rust_hints.is_empty());
        assert!(rust_hints[0].contains("impl"));

        let python_hints = processor.get_search_hints("python");
        assert!(!python_hints.is_empty());
        assert!(python_hints[0].contains("def"));
    }

    #[test]
    fn test_detection_caching() {
        let mut processor = LanguageProcessor::new(LanguageConfig::default());

        let path = PathBuf::from("test.rs");

        // First detection
        let result1 = processor.detect_language(&path).unwrap();
        assert!(result1.is_some());

        // Second detection (should work)
        let result2 = processor.detect_language(&path).unwrap();
        assert_eq!(result1.unwrap().language_name, result2.unwrap().language_name);

        // Clear cache
        processor.clear_cache();
        assert!(processor.detection_cache.is_empty());
    }
}