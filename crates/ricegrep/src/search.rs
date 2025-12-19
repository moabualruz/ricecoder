//! Core search engine for RiceGrep
//!
//! This module provides the main search functionality with ripgrep-compatible
//! behavior and AI enhancements.

use crate::error::RiceGrepError;
use crate::language::{LanguageProcessor, LanguageConfig};
use crate::spelling::{SpellingCorrector, SpellingConfig, CorrectionResult};
use detect_lang::Language;


/// Placeholder for LSP integration (to be implemented)
#[derive(Debug, Clone)]
pub struct LSPIntegration;

impl LSPIntegration {
    pub fn new() -> Self {
        Self
    }

    pub fn is_available(&self, _language: &Language) -> bool {
        // Check if ricecoder-lsp is available by trying to run it
        // For now, assume it's available if the binary exists
        std::process::Command::new("ricecoder")
            .arg("lsp")
            .arg("--help")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    pub async fn workspace_symbols(&self, query: &str, language: &Language<'_>) -> Result<Vec<LspSymbol>, RiceGrepError> {
        // For now, return empty results as full LSP client implementation is complex
        // TODO: Implement full LSP client to spawn ricecoder-lsp and query workspace symbols
        warn!("LSP integration not fully implemented - returning empty results for query: {}", query);
        Ok(vec![])
    }
}

/// LSP symbol structure matching ricecoder-lsp types
#[derive(Debug, Clone)]
pub struct LspSymbol {
    pub name: String,
    pub kind: LspSymbolKind,
    pub location: LspLocation,
}

/// LSP symbol kinds matching ricecoder-lsp
#[derive(Debug, Clone)]
pub enum LspSymbolKind {
    Function,
    Class,
    Variable,
    Other,
}

/// LSP location structure
#[derive(Debug, Clone)]
pub struct LspLocation {
    pub file: PathBuf,
    pub line: usize,
}
use async_trait::async_trait;
use ignore::{WalkBuilder, overrides::OverrideBuilder};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::task;
use strsim::{damerau_levenshtein, jaro_winkler};
use tracing::{debug, info, warn};
use indicatif::{ProgressBar, ProgressStyle};

/// Progress verbosity levels for different amounts of detail
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProgressVerbosity {
    Quiet,      // No progress output
    Minimal,    // Simple spinner only
    Normal,     // Current implementation (default)
    Verbose,    // Detailed progress with additional info
}

/// Search query configuration
#[derive(Debug, Clone)]
pub struct SearchQuery {
    /// Search pattern (regex or literal)
    pub pattern: String,
    /// Paths to search in
    pub paths: Vec<PathBuf>,
    /// Case insensitive search
    pub case_insensitive: bool,
    /// Case sensitive search (overrides case_insensitive)
    pub case_sensitive: bool,
    /// Match whole words only
    pub word_regexp: bool,
    /// Literal string search (no regex)
    pub fixed_strings: bool,
    /// Follow symbolic links
    pub follow: bool,
    /// Search hidden files and directories
    pub hidden: bool,
    /// Respect .gitignore files
    pub no_ignore: bool,
    /// Custom ignore file path (e.g., .ricegrepignore)
    pub ignore_file: Option<PathBuf>,
    /// Suppress progress output and spinners
    pub quiet: bool,
    /// Show what would be done without making changes
    pub dry_run: bool,
    /// Maximum file size in bytes to search/index
    pub max_file_size: Option<u64>,
    /// Progress reporting verbosity level
    pub progress_verbosity: ProgressVerbosity,
    /// Maximum number of files to process (quota)
    pub max_files: Option<usize>,
    /// Maximum number of matches to return (quota)
    pub max_matches: Option<usize>,
    /// Maximum number of lines to display per file (for content display)
    pub max_lines: Option<usize>,
    /// Invert match (show non-matching lines)
    pub invert_match: bool,
    /// Enable AI-enhanced search
    pub ai_enhanced: bool,
    /// Disable reranking of results
    pub no_rerank: bool,
    /// Fuzzy search tolerance (edit distance)
    pub fuzzy: Option<usize>,
    /// Maximum number of matches
    pub max_count: Option<usize>,
    /// Spelling correction result (if applied)
    pub spelling_correction: Option<CorrectionResult>,
}

/// Individual search match result
#[derive(Debug, Clone)]
pub struct SearchMatch {
    /// File path where match was found
    pub file: PathBuf,
    /// Line number (1-indexed)
    pub line_number: usize,
    /// Line content
    pub line_content: String,
    /// Byte offset in file
    pub byte_offset: usize,
    /// AI confidence score (0.0 to 1.0)
    pub ai_score: Option<f32>,
    /// AI reasoning/context
    pub ai_context: Option<String>,
    /// Detected programming language name
    pub language: Option<String>,
    /// Language detection confidence
    pub language_confidence: Option<f32>,
}

/// Search results container
#[derive(Debug)]
pub struct SearchResults {
    /// Individual matches
    pub matches: Vec<SearchMatch>,
    /// Total number of matches found
    pub total_matches: usize,
    /// Search execution time
    pub search_time: Duration,
    /// Whether AI reranking was applied
    pub ai_reranked: bool,
    /// Whether degradation mode was active (fallback functionality used)
    pub degradation_mode: bool,
    /// Number of files searched
    pub files_searched: usize,
    /// Spelling correction applied (if any)
    pub spelling_correction: Option<CorrectionResult>,
    /// Count of matches per file (for --count mode)
    pub file_counts: std::collections::HashMap<std::path::PathBuf, usize>,
}

/// Fuzzy match result
#[derive(Debug, Clone)]
pub struct FuzzyMatch {
    /// The matched text
    pub text: String,
    /// Similarity score (0.0 to 1.0)
    pub score: f64,
    /// Edit distance
    pub distance: usize,
}

/// Fuzzy string matcher for approximate matching
pub struct FuzzyMatcher {
    /// Maximum edit distance allowed
    max_distance: usize,
    /// Minimum similarity score required
    min_score: f64,
}

impl FuzzyMatcher {
    /// Create a new fuzzy matcher
    pub fn new(max_distance: usize, min_score: f64) -> Self {
        Self {
            max_distance,
            min_score,
        }
    }

    /// Check if two strings are fuzzy matches
    pub fn is_match(&self, pattern: &str, text: &str) -> bool {
        let distance = damerau_levenshtein(pattern, text);
        if distance > self.max_distance {
            return false;
        }

        let score = jaro_winkler(pattern, text);
        score >= self.min_score
    }

    /// Get detailed fuzzy match information
    pub fn match_details(&self, pattern: &str, text: &str) -> Option<FuzzyMatch> {
        let distance = damerau_levenshtein(pattern, text);
        if distance > self.max_distance {
            return None;
        }

        let score = jaro_winkler(pattern, text);
        if score < self.min_score {
            return None;
        }

        Some(FuzzyMatch {
            text: text.to_string(),
            score,
            distance,
        })
    }

    /// Find best fuzzy matches in a collection of strings
    pub fn find_best_matches<'a>(&self, pattern: &str, candidates: &'a [&str], limit: usize) -> Vec<(&'a str, FuzzyMatch)> {
        let mut matches: Vec<(&str, FuzzyMatch)> = candidates
            .iter()
            .filter_map(|candidate| {
                self.match_details(pattern, candidate)
                    .map(|details| (*candidate, details))
            })
            .collect();

        // Sort by score (descending) then by distance (ascending)
        matches.sort_by(|a, b| {
            b.1.score.partial_cmp(&a.1.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.1.distance.cmp(&b.1.distance))
        });

        matches.into_iter().take(limit).collect()
    }
}

impl Clone for FuzzyMatcher {
    fn clone(&self) -> Self {
        Self {
            max_distance: self.max_distance,
            min_score: self.min_score,
        }
    }
}

impl Default for FuzzyMatcher {
    fn default() -> Self {
        Self {
            max_distance: 2,
            min_score: 0.8,
        }
    }
}

/// Indexed line information for fast searching
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IndexedLine {
    /// Line number (1-indexed)
    pub line_number: usize,
    /// Line content
    pub content: String,
    /// Byte offset in file
    pub byte_offset: usize,
}

/// File index containing all indexed lines
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileIndex {
    /// File path
    pub path: PathBuf,
    /// Indexed lines
    pub lines: Vec<IndexedLine>,
    /// File checksum for change detection
    pub checksum: String,
    /// Last modified time
    pub modified: SystemTime,
}

/// Search index metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IndexMetadata {
    /// Index creation time
    pub created_at: SystemTime,
    /// Total number of files indexed
    pub file_count: usize,
    /// Total number of lines indexed
    pub line_count: usize,
    /// Index format version
    pub version: String,
}

/// Main search index structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchIndex {
    /// Indexed files by path
    pub files: HashMap<PathBuf, FileIndex>,
    /// Index metadata
    pub metadata: IndexMetadata,
}

/// Index manager for building and querying search indexes
pub struct IndexManager {
    /// Base directory for storing indexes
    index_dir: PathBuf,
    /// Current loaded index
    current_index: Option<Arc<SearchIndex>>,
}

impl IndexManager {
    /// Create a new index manager
    pub fn new(index_dir: PathBuf) -> Self {
        Self {
            index_dir,
            current_index: None,
        }
    }

    /// Get the index file path for a given root directory
    fn get_index_path(&self, root: &PathBuf) -> PathBuf {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        root.hash(&mut hasher);
        let hash = hasher.finish();

        self.index_dir.join(format!("index_{:x}.idx", hash))
    }

    /// Load index from disk if it exists
    pub fn load_index(&mut self, root: &PathBuf) -> Result<bool, RiceGrepError> {
        let index_path = self.get_index_path(root);

        if !index_path.exists() {
            return Ok(false);
        }

        let data = fs::read(&index_path)?;
        let index: SearchIndex = bincode::deserialize(&data)
            .map_err(|e| RiceGrepError::Index {
                message: format!("Failed to deserialize index: {}", e),
            })?;

        self.current_index = Some(Arc::new(index));
        Ok(true)
    }

    /// Save current index to disk
    pub fn save_index(&self, root: &PathBuf) -> Result<(), RiceGrepError> {
        if let Some(index) = &self.current_index {
            let index_path = self.get_index_path(root);

            // Ensure index directory exists
            if let Some(parent) = index_path.parent() {
                fs::create_dir_all(parent)?;
            }

        let data = bincode::serialize(&**index)
            .map_err(|e| RiceGrepError::Index {
                message: format!("Failed to serialize index: {}", e),
            })?;

            fs::write(&index_path, data)?;
        }

        Ok(())
    }

    /// Check if index needs rebuilding
    pub fn needs_rebuild(&self, root: &PathBuf) -> Result<bool, RiceGrepError> {
        if self.current_index.is_none() {
            return Ok(true);
        }

        let index = match self.current_index.as_ref() {
            Some(idx) => idx,
            None => return Ok(false), // No index loaded, consider it not needing rebuild
        };

        // Check if any indexed files have changed
        for (path, file_index) in &index.files {
            if let Ok(metadata) = fs::metadata(path) {
                if let Ok(modified) = metadata.modified() {
                    if modified != file_index.modified {
                        return Ok(true);
                    }
                }
            } else {
                // File no longer exists
                return Ok(true);
            }
        }

        // Check for new files
        let walker = WalkBuilder::new(root)
            .hidden(false)
            .ignore(true)
            .build();

        for entry in walker {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && !index.files.contains_key(path) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Get current index
    pub fn get_index(&self) -> Option<Arc<SearchIndex>> {
        self.current_index.clone()
    }
}

impl Default for IndexManager {
    fn default() -> Self {
        Self::new(PathBuf::from(".ricegrep"))
    }
}

/// Core search engine trait
#[async_trait]
pub trait SearchEngine {
    /// Execute a search with the given query
    async fn search(&mut self, query: SearchQuery) -> Result<SearchResults, RiceGrepError>;

    /// Build search index for faster queries
    async fn build_index(&mut self, paths: &[PathBuf], progress_verbosity: ProgressVerbosity) -> Result<(), RiceGrepError>;

    /// Check if index exists and is valid
    fn has_index(&mut self, paths: &[PathBuf]) -> bool;
}

/// Basic regex search engine
pub struct RegexSearchEngine {
    /// Compiled regex patterns cache
    patterns: std::sync::Mutex<std::collections::HashMap<String, Regex>>,
    /// Fuzzy matcher for approximate string matching
    fuzzy_matcher: FuzzyMatcher,
    /// Index manager for fast searches
    index_manager: IndexManager,
    /// AI processor for natural language queries
    ai_processor: Option<Box<dyn crate::ai::AIProcessor>>,
    /// LSP integration for enhanced search
    lsp_integration: LSPIntegration,
    /// Spelling corrector for query correction
    spelling_corrector: Option<SpellingCorrector>,
    /// Language processor for language-aware search
    language_processor: Option<LanguageProcessor>,
    /// Session manager for context (placeholder for future integration)
    session_manager: Option<()>, // Session management integration (placeholder)
    activity_logger: Option<()>, // Activity logging integration (placeholder)
    degradation_mode: bool, // Whether we're in degradation mode due to failures
}

impl RegexSearchEngine {
    /// Create a new regex search engine
    pub fn new() -> Self {
        Self {
            patterns: std::sync::Mutex::new(std::collections::HashMap::new()),
            fuzzy_matcher: FuzzyMatcher::default(),
            index_manager: IndexManager::default(),
            ai_processor: None,
            lsp_integration: LSPIntegration::new(),
            spelling_corrector: None,
            language_processor: None,
            session_manager: None,
            activity_logger: None,
            degradation_mode: false,
        }
    }

    /// Create a new regex search engine with session manager integration
    pub fn with_session_manager(session_manager: ()) -> Self {
        Self {
            patterns: std::sync::Mutex::new(std::collections::HashMap::new()),
            fuzzy_matcher: FuzzyMatcher::default(),
            index_manager: IndexManager::default(),
            ai_processor: None,
            lsp_integration: LSPIntegration::new(),
            spelling_corrector: None,
            language_processor: None,
            session_manager: Some(session_manager),
            activity_logger: None,
            degradation_mode: false,
        }
    }



    /// Generate an AI-powered answer based on search results
    pub async fn generate_answer(&self, query: &str, results: &SearchResults) -> Result<String, RiceGrepError> {
        if let Some(ai_processor) = &self.ai_processor {
            ai_processor.generate_answer(query, results).await
        } else {
            Err(RiceGrepError::Ai {
                message: "AI processor not available for answer generation".to_string()
            })
        }
    }

    /// Create a new regex search engine with custom fuzzy settings
    pub fn with_fuzzy_config(max_distance: usize, min_score: f64) -> Self {
        Self {
            patterns: std::sync::Mutex::new(std::collections::HashMap::new()),
            fuzzy_matcher: FuzzyMatcher::new(max_distance, min_score),
            index_manager: IndexManager::default(),
            ai_processor: None,
            lsp_integration: LSPIntegration::new(),
            spelling_corrector: None,
            language_processor: None,
            session_manager: None,
            activity_logger: None,
            degradation_mode: false,
        }
    }

    /// Create a new regex search engine with custom index directory
    pub fn with_index_dir(index_dir: PathBuf) -> Self {
        Self {
            patterns: std::sync::Mutex::new(std::collections::HashMap::new()),
            fuzzy_matcher: FuzzyMatcher::default(),
            index_manager: IndexManager::new(index_dir),
            ai_processor: None,
            lsp_integration: LSPIntegration::new(),
            spelling_corrector: None,
            language_processor: None,
            session_manager: None,
            activity_logger: None,
            degradation_mode: false,
        }
    }

    /// Set the spelling corrector for query correction
    pub fn with_spelling_corrector(mut self, config: SpellingConfig) -> Self {
        self.spelling_corrector = Some(SpellingCorrector::new(config));
        self
    }

    /// Set the language processor for language-aware search
    pub fn with_language_processor(mut self, config: LanguageConfig) -> Self {
        self.language_processor = Some(LanguageProcessor::new(config));
        self
    }

    /// Create a new regex search engine with AI processor
    pub fn with_ai_processor(mut self, ai_processor: Box<dyn crate::ai::AIProcessor>) -> Self {
        self.ai_processor = Some(ai_processor);
        self
    }

    /// Compile or retrieve cached regex pattern
    fn get_regex(&self, pattern: &str, case_insensitive: bool, word_regexp: bool) -> Result<Regex, RiceGrepError> {
        let mut cache = self.patterns.lock().unwrap();

        let key = format!("{}:{}:{}", pattern, case_insensitive, word_regexp);

        if let Some(regex) = cache.get(&key) {
            return Ok(regex.clone());
        }

        let mut regex_pattern = pattern.to_string();

        // Add word boundaries if requested
        if word_regexp {
            regex_pattern = format!(r"\b{}\b", regex_pattern);
        }

        let mut builder = regex::RegexBuilder::new(&regex_pattern);
        builder.case_insensitive(case_insensitive);

        let regex = builder.build()?;
        cache.insert(key, regex.clone());

        Ok(regex)
    }

    /// Search a single file for matches with fuzzy matching support
    fn search_file_with_fuzzy(&self, file_path: &PathBuf, regex: &Regex, query: &SearchQuery, fuzzy_matcher: &FuzzyMatcher) -> Result<Vec<SearchMatch>, RiceGrepError> {
        // Use memory mapping for large files to improve performance
        let metadata = fs::metadata(file_path)?;
        let content = if metadata.len() > 1024 * 1024 { // 1MB threshold
            // Use memory mapping for large files
            use std::fs::File;
            use memmap2::Mmap;
            let file = File::open(file_path)?;
            let mmap = unsafe { Mmap::map(&file)? };
            String::from_utf8_lossy(&mmap).to_string()
        } else {
            // Use regular reading for small files
            fs::read_to_string(file_path)?
        };
        let mut matches = Vec::new();

        for (line_idx, line) in content.lines().enumerate() {
            let is_match = if query.fixed_strings && query.fuzzy.is_some() {
                // Fuzzy matching for literal strings
                fuzzy_matcher.is_match(&query.pattern, line)
            } else if query.fuzzy.is_some() {
                // Fuzzy matching combined with regex
                regex.is_match(line) || fuzzy_matcher.is_match(&query.pattern, line)
            } else {
                // Regular regex matching
                regex.is_match(line)
            };

            // Apply invert match logic
            let should_include = if query.invert_match { !is_match } else { is_match };

            if should_include {
                // Detect language for this file
                            let language_info = if let Some(ref lang_processor) = self.language_processor {
                    lang_processor.detect_language(&file_path).unwrap_or(None)
                } else {
                    None
                };

                let search_match = SearchMatch {
                    file: file_path.clone(),
                    line_number: line_idx + 1, // 1-indexed
                    line_content: line.to_string(),
                    byte_offset: 0, // TODO: calculate actual byte offset
                    ai_score: None,
                    ai_context: None,
                    language: language_info.as_ref().map(|d| d.language_name.clone()),
                    language_confidence: language_info.as_ref().map(|d| d.confidence),
                };
                matches.push(search_match);

                // Check max count limit
                if let Some(max) = query.max_count {
                    if matches.len() >= max {
                        break;
                    }
                }
            }
        }

        Ok(matches)
    }

    /// Search a single file for matches (legacy method for backward compatibility)
    fn search_file(&self, file_path: &PathBuf, regex: &Regex, query: &SearchQuery) -> Result<Vec<SearchMatch>, RiceGrepError> {
        self.search_file_with_fuzzy(file_path, regex, query, &self.fuzzy_matcher)
    }

    /// Build search index for the given paths (internal method)
    fn build_index_internal(&self, root: &PathBuf, progress_verbosity: ProgressVerbosity) -> Result<SearchIndex, RiceGrepError> {
        let mut files = HashMap::new();
        let mut total_lines = 0;

        let walker = WalkBuilder::new(root)
            .hidden(false)
            .ignore(true)
            .build();

        // Collect all file paths first
        let mut file_paths = Vec::new();
        for entry in walker {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                file_paths.push(path.to_path_buf());
            }
        }

        // Create progress bar for file processing based on verbosity level
        let progress_bar = match progress_verbosity {
            ProgressVerbosity::Quiet => None,
            ProgressVerbosity::Minimal => {
                let pb = ProgressBar::new_spinner();
                pb.set_style(
                    ProgressStyle::default_spinner()
                        .template("{spinner:.green} Indexing files...")
                        .unwrap()
                );
                Some(pb)
            }
            ProgressVerbosity::Normal => {
                let pb = ProgressBar::new(file_paths.len() as u64);
                pb.set_style(
                    ProgressStyle::default_bar()
                        .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} files ({eta})")
                        .unwrap()
                        .progress_chars("#>-")
                );
                pb.set_message("Indexing files");
                Some(pb)
            }
            ProgressVerbosity::Verbose => {
                let pb = ProgressBar::new(file_paths.len() as u64);
                pb.set_style(
                    ProgressStyle::default_bar()
                        .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} files ({eta}) {msg}")
                        .unwrap()
                        .progress_chars("#>-")
                );
                pb.set_message("Indexing files - processing...");
                Some(pb)
            }
        };

        // Process files in parallel for better performance
        use rayon::prelude::*;
        let file_results: Vec<(PathBuf, FileIndex)> = file_paths.par_iter()
            .filter_map(|path| {
                // Skip files that can't be indexed (e.g., binary files)
                let result = self.index_file(path).ok().map(|index| (path.clone(), index));
                if let Some(ref pb) = progress_bar {
                    pb.inc(1);
                }
                result
            })
            .collect();

        if let Some(pb) = progress_bar {
            pb.finish_with_message("Index build complete");
        }

        // Collect results
        for (path, file_index) in file_results {
            total_lines += file_index.lines.len();
            files.insert(path, file_index);
        }

        let metadata = IndexMetadata {
            created_at: SystemTime::now(),
            file_count: files.len(),
            line_count: total_lines,
            version: "1.0".to_string(),
        };

        Ok(SearchIndex { files, metadata })
    }

    /// Index a single file
    fn index_file(&self, path: &PathBuf) -> Result<FileIndex, RiceGrepError> {
        let metadata = fs::metadata(path)?;

        // Use memory mapping for better performance on large files
        let content = if metadata.len() > 1024 * 1024 { // 1MB threshold
            // Use memory mapping for large files
            use std::fs::File;
            use memmap2::Mmap;
            let file = File::open(path)?;
            let mmap = unsafe { Mmap::map(&file)? };
            String::from_utf8_lossy(&mmap).to_string()
        } else {
            // Use regular reading for small files
            fs::read_to_string(path)?
        };

        // Calculate simple checksum (in production, use a proper hash)
        let checksum = format!("{:x}", content.len());

        let mut lines = Vec::new();
        let mut byte_offset = 0;

        for (i, line) in content.lines().enumerate() {
            let line_bytes = line.as_bytes();
            lines.push(IndexedLine {
                line_number: i + 1,
                content: line.to_string(),
                byte_offset,
            });
            byte_offset += line_bytes.len() + 1; // +1 for newline
        }

        Ok(FileIndex {
            path: path.clone(),
            lines,
            checksum,
            modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
        })
    }

    /// Search using index if available
    fn search_with_index_temp(&self, query: &SearchQuery, regex: &Regex, fuzzy_matcher: &FuzzyMatcher, index_manager: &IndexManager) -> Result<Vec<SearchMatch>, RiceGrepError> {
        if let Some(index) = index_manager.get_index() {
            // Use index for faster searching
            let mut matches = Vec::new();

            for (path, file_index) in &index.files {
                // Check if file should be searched based on query paths
                let should_search = if query.paths.is_empty() {
                    true
                } else {
                    query.paths.iter().any(|search_path| {
                        path.starts_with(search_path) || search_path.starts_with(path)
                    })
                };

                if should_search {
                    for indexed_line in &file_index.lines {
                        let is_match = if query.fixed_strings && query.fuzzy.is_some() {
                            fuzzy_matcher.is_match(&query.pattern, &indexed_line.content)
                        } else if query.fuzzy.is_some() {
                            regex.is_match(&indexed_line.content) || fuzzy_matcher.is_match(&query.pattern, &indexed_line.content)
                        } else {
                            regex.is_match(&indexed_line.content)
                        };

                        // Apply invert match logic
                        let should_include = if query.invert_match { !is_match } else { is_match };

                        if should_include {
                            // Detect language for this file
                let language_info = if let Some(ref lang_processor) = self.language_processor {
                                lang_processor.detect_language(&path).unwrap_or(None)
                            } else {
                                None
                            };

                            let search_match = SearchMatch {
                                file: path.clone(),
                                line_number: indexed_line.line_number,
                                line_content: indexed_line.content.clone(),
                                byte_offset: indexed_line.byte_offset,
                                ai_score: None,
                                ai_context: None,
                                language: language_info.as_ref().map(|d| d.language_name.clone()),
                                language_confidence: language_info.as_ref().map(|d| d.confidence),
                            };
                            matches.push(search_match);

                            if let Some(max) = query.max_count {
                                if matches.len() >= max {
                                    return Ok(matches);
                                }
                            }
                        }
                    }
                }
            }

            Ok(matches)
        } else {
            // Fall back to file-by-file search
            self.search_files_fallback(query, regex, fuzzy_matcher)
        }
    }

    /// Fallback search when no index is available
    fn search_files_fallback(&self, query: &SearchQuery, regex: &Regex, fuzzy_matcher: &FuzzyMatcher) -> Result<Vec<SearchMatch>, RiceGrepError> {
        let files = self.get_search_files(query)?;

        // Use parallel processing for better performance
        use rayon::prelude::*;

        let results: Vec<Result<Vec<SearchMatch>, RiceGrepError>> = files.par_iter()
            .map(|file_path| {
                self.search_file_with_fuzzy(file_path, regex, query, fuzzy_matcher)
            })
            .collect();

        let mut all_matches = Vec::new();
        for result in results {
            match result {
                Ok(matches) => {
                    // Memory-efficient: limit total matches to prevent excessive memory usage
                    const MAX_MATCHES: usize = 50000;
                    if all_matches.len() + matches.len() > MAX_MATCHES {
                        warn!("Match limit of {} reached for memory efficiency, truncating results", MAX_MATCHES);
                        // Add remaining matches up to the limit
                        let remaining = MAX_MATCHES - all_matches.len();
                        all_matches.extend(matches.into_iter().take(remaining));
                        break;
                    }
                    all_matches.extend(matches);

                    if let Some(max) = query.max_count {
                        if all_matches.len() >= max {
                            break;
                        }
                    }
                }
                Err(e) => {
                    warn!("Error searching file: {}", e);
                    // Continue with other files rather than failing completely
                }
            }
        }

        Ok(all_matches)
    }

    /// Basic reranking of matches based on query understanding
    /// Deterministic ranking that works without AI understanding
    fn rerank_matches_deterministic(&self, matches: &mut Vec<SearchMatch>, query_pattern: &str) {
        // Simple TF-IDF style ranking based on term frequency and position
        for match_result in matches.iter_mut() {
            let mut score = 0.0f32;

            // Term frequency scoring - count occurrences of query terms
            let query_terms: Vec<&str> = query_pattern.split_whitespace().collect();
            for term in &query_terms {
                let term_lower = term.to_lowercase();
                let content_lower = match_result.line_content.to_lowercase();
                let count = content_lower.matches(&term_lower).count() as f32;
                score += count * 1.0; // Term frequency

                // Exact match bonus
                if content_lower.contains(&term_lower) {
                    score += 0.5;
                }
            }

            // Position-based scoring
            if match_result.line_number <= 5 {
                score += 0.3; // Early in file
            } else if match_result.line_number <= 20 {
                score += 0.1; // Still near top
            }

            // Length penalty (shorter, more focused matches score higher)
            let line_length = match_result.line_content.len() as f32;
            if line_length > 100.0 {
                score -= (line_length - 100.0) / 1000.0; // Small penalty for very long lines
            }

            match_result.ai_score = Some(score);
        }

        // Sort by score (descending) - deterministic sort
        matches.sort_by(|a, b| {
            let a_score = a.ai_score.unwrap_or(0.0);
            let b_score = b.ai_score.unwrap_or(0.0);
            // Use total_cmp for deterministic floating point comparison
            b_score.total_cmp(&a_score)
        });
    }

    fn rerank_matches_basic(&self, matches: &mut Vec<SearchMatch>, understanding: &crate::ai::QueryUnderstanding) {
        // Simple reranking based on term frequency and position
        for match_result in matches.iter_mut() {
            let mut score = 0.0f32;

            // Boost score for matches containing search terms
            for term in &understanding.search_terms {
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
        matches.sort_by(|a, b| {
            let a_score = a.ai_score.unwrap_or(0.0);
            let b_score = b.ai_score.unwrap_or(0.0);
            b_score.partial_cmp(&a_score).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Apply language-aware ranking adjustments
        if let Some(ref lang_processor) = self.language_processor {
            for match_result in matches.iter_mut() {
                if let Some(language_name) = &match_result.language {
                    // Apply language-specific ranking boost
                    let base_score = match_result.ai_score.unwrap_or(0.5);
                    let ranking = lang_processor.calculate_ranking(base_score, Some(language_name.as_str()));
                    match_result.ai_score = Some(ranking.adjusted_score);
                    debug!("Applied language ranking for {}: {} -> {}",
                           language_name, base_score, ranking.adjusted_score);
                }
            }

            // Re-sort after language adjustments
            matches.sort_by(|a, b| {
                let a_score = a.ai_score.unwrap_or(0.0);
                let b_score = b.ai_score.unwrap_or(0.0);
                b_score.partial_cmp(&a_score).unwrap_or(std::cmp::Ordering::Equal)
            });
        }
    }

    /// Search using LSP for symbol-based queries when available
    async fn search_with_lsp(&self, query: &SearchQuery, language: &Language<'_>) -> Result<Vec<SearchMatch>, RiceGrepError> {
        if !self.lsp_integration.is_available(language) {
            return Ok(vec![]);
        }

        // Try workspace symbol search for symbol-based queries
        if self.is_symbol_query(query) {
            let symbols = self.lsp_integration.workspace_symbols(&query.pattern, language).await?;

            let matches = symbols.into_iter().map(|symbol| {
                SearchMatch {
                    file: symbol.location.file.clone(),
                    line_number: symbol.location.line,
                    line_content: format!("// {}: {:?}", symbol.name, symbol.kind), // Placeholder content
                    byte_offset: 0,
                    ai_score: Some(0.9), // High confidence for LSP results
                    ai_context: Some(format!("LSP Symbol: {:?}", symbol.kind)),
                    language: Some(language.0.to_string()), // LSP results are language-specific
                    language_confidence: Some(1.0), // LSP detection is highly confident
                }
            }).collect();

            return Ok(matches);
        }

        Ok(vec![])
    }

    /// Determine if a query is likely looking for symbols (functions, classes, etc.)
    fn is_symbol_query(&self, query: &SearchQuery) -> bool {
        // Simple heuristic: if query looks like a programming identifier
        let pattern = &query.pattern;

        // Check for common symbol patterns
        if pattern.chars().next().unwrap_or(' ').is_uppercase() {
            // Likely a class/type name
            return true;
        }

        // Check for function-like patterns
        if pattern.contains('(') || pattern.contains("fn ") || pattern.contains("def ") || pattern.contains("function ") {
            return true;
        }

        // Check against known keywords (less likely to be symbol names)
        let keywords = ["if", "for", "while", "return", "let", "const", "var", "import", "from", "class", "function"];
        for keyword in &keywords {
            if pattern == *keyword {
                return false;
            }
        }

        // If it's a single word and looks like an identifier, likely a symbol
        pattern.chars().all(|c| c.is_alphanumeric() || c == '_') && pattern.len() > 2
    }



    /// Determine if a query appears to be natural language rather than a regex
    pub fn is_natural_language_query(&self, query: &SearchQuery) -> bool {
        // Skip if already marked as AI enhanced or if no AI processor
        if query.ai_enhanced || self.ai_processor.is_none() {
            return false;
        }

        let pattern = &query.pattern;

        // Check for natural language indicators
        let natural_indicators = [
            "find", "search", "locate", "get", "show", "display",
            "where", "how", "what", "which", "definition", "usage", "reference"
        ];

        let pattern_lower = pattern.to_lowercase();
        for indicator in &natural_indicators {
            if pattern_lower.contains(indicator) {
                return true;
            }
        }

        // Check if it looks like multiple words (likely natural language)
        let words: Vec<&str> = pattern.split_whitespace().collect();
        if words.len() > 2 {
            return true;
        }

        // Check for programming keywords that suggest natural language
        if pattern_lower.contains(" in ") || pattern_lower.contains(" for ") || pattern_lower.contains(" with ") {
            return true;
        }

        // Check if it's a single programming concept word
        if words.len() == 1 {
            let single_word_indicators = [
                "function", "functions", "class", "classes", "method", "methods",
                "variable", "variables", "constant", "constants", "module", "modules",
                "interface", "interfaces", "struct", "structs", "enum", "enums"
            ];
            for indicator in &single_word_indicators {
                if pattern_lower == *indicator {
                    return true;
                }
            }
        }

        false
    }

    /// Parse a custom ignore file with .gitignore-style syntax
    fn parse_ignore_file(&self, ignore_file_path: &PathBuf) -> Result<Vec<String>, RiceGrepError> {
        if !ignore_file_path.exists() {
            return Ok(vec![]);
        }

        let content = fs::read_to_string(ignore_file_path)
            .map_err(|e| RiceGrepError::Search {
                message: format!("Failed to read ignore file {}: {}", ignore_file_path.display(), e)
            })?;

        let mut patterns = Vec::new();

        for line in content.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            patterns.push(line.to_string());
        }

        Ok(patterns)
    }

    /// Get files to search based on query configuration
    fn get_search_files(&self, query: &SearchQuery) -> Result<Vec<PathBuf>, RiceGrepError> {
        let mut files = Vec::new();

        // If no paths specified, search current directory
        let search_paths = if query.paths.is_empty() {
            vec![PathBuf::from(".")]
        } else {
            query.paths.clone()
        };

        // Parse custom ignore patterns if specified
        let (ignore_patterns, unignore_patterns) = if let Some(ref ignore_file) = query.ignore_file {
            let all_patterns = self.parse_ignore_file(ignore_file)?;
            let mut ignore_patterns = Vec::new();
            let mut unignore_patterns = Vec::new();

            for pattern in all_patterns {
                if pattern.starts_with('!') {
                    // Negation pattern - don't ignore files matching this
                    unignore_patterns.push(pattern[1..].to_string());
                } else {
                    // Normal ignore pattern
                    ignore_patterns.push(pattern);
                }
            }

            (ignore_patterns, unignore_patterns)
        } else {
            (vec![], vec![])
        };

        for path in search_paths {
            // Check file quota
            if let Some(max_files) = query.max_files {
                if files.len() >= max_files {
                    break; // Stop processing if we've reached the file quota
                }
            }

            let mut walker_builder = WalkBuilder::new(&path);
            walker_builder
                .hidden(!query.hidden) // Hide hidden files unless requested
                .ignore(!query.no_ignore) // Respect .gitignore unless disabled
                .follow_links(query.follow);

            // Note: Custom ignore file patterns are handled by manual filtering below
            // The OverrideBuilder is not used for custom ignore files to avoid conflicts

            let walker = walker_builder.build();

            for entry in walker {
                let entry = entry?;
                let file_path = entry.path();

                // Only search files
                if file_path.is_file() {
                    // Check file size limit
                    let file_size_ok = if let Some(max_size) = query.max_file_size {
                        match file_path.metadata() {
                            Ok(metadata) => metadata.len() <= max_size,
                            Err(_) => true, // If we can't get metadata, allow the file
                        }
                    } else {
                        true
                    };

                    if file_size_ok {
                        // Check against custom ignore patterns
                        let should_ignore = if !ignore_patterns.is_empty() || !unignore_patterns.is_empty() {
                            self.should_ignore_file(file_path, &path, &ignore_patterns, &unignore_patterns)
                        } else {
                            false
                        };

                        if !should_ignore {
                            files.push(file_path.to_path_buf());

                            // Check if we've reached the file quota
                            if let Some(max_files) = query.max_files {
                                if files.len() >= max_files {
                                    break; // Stop collecting files if we've reached the quota
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(files)
    }

    /// Check if a file should be ignored based on custom ignore patterns
    fn should_ignore_file(&self, file_path: &Path, root_path: &Path, ignore_patterns: &[String], unignore_patterns: &[String]) -> bool {
        // Get the relative path from the search root
        let relative_path = match file_path.strip_prefix(root_path) {
            Ok(rel) => rel,
            Err(_) => return false, // If we can't get relative path, don't ignore
        };

        let relative_str = relative_path.to_string_lossy().replace('\\', "/");

        // First check if it matches any ignore patterns
        let mut should_ignore = false;
        for pattern in ignore_patterns {
            if self.matches_ignore_pattern(&relative_str, pattern) {
                should_ignore = true;
                break;
            }
        }

        // Then check if it matches any unignore patterns (negation)
        if should_ignore {
            for pattern in unignore_patterns {
                if self.matches_ignore_pattern(&relative_str, pattern) {
                    should_ignore = false;
                    break;
                }
            }
        }

        should_ignore
    }

    /// Check if a relative path matches an ignore pattern
    fn matches_ignore_pattern(&self, relative_path: &str, pattern: &str) -> bool {
        // Simple implementation of .gitignore-style matching
        // For now, support exact matches and basic glob patterns

        if pattern.contains('*') {
            // Simple glob matching
            let pattern_regex = pattern
                .replace('.', r"\.")
                .replace('*', ".*")
                .replace('?', ".");
            match regex::Regex::new(&format!("^{}$", pattern_regex)) {
                Ok(re) => re.is_match(relative_path),
                Err(_) => false,
            }
        } else if pattern.ends_with('/') {
            // Directory pattern - match if path starts with the directory
            relative_path.starts_with(pattern)
        } else {
            // Exact match or file name match
            relative_path == pattern || relative_path.ends_with(&format!("/{}", pattern))
        }
    }
}

#[async_trait]
impl SearchEngine for RegexSearchEngine {
    async fn search(&mut self, query: SearchQuery) -> Result<SearchResults, RiceGrepError> {
        let start_time = Instant::now();

        info!("Starting search for pattern: {}", query.pattern);
        debug!("Search options: ai_enhanced={}, fuzzy={:?}, case_insensitive={}",
               query.ai_enhanced, query.fuzzy, query.case_insensitive);

        // Apply spelling correction if enabled
        let corrected_query = if let Some(ref mut corrector) = self.spelling_corrector {
            match corrector.correct_query(&query.pattern) {
                Ok(correction) => {
                    if correction.corrected_applied {
                        info!("Applied spelling correction: '{}' -> '{}' (confidence: {:.2})",
                              correction.original, correction.corrected.as_ref().unwrap(), correction.confidence);
                    }
                    SearchQuery {
                        pattern: correction.corrected.clone().unwrap_or(correction.original.clone()),
                        spelling_correction: Some(correction),
                        ..query.clone()
                    }
                }
                Err(e) => {
                    warn!("Spelling correction failed, using original query: {}", e);
                    query.clone()
                }
            }
        } else {
            query.clone()
        };

        // Check if this is a natural language query that needs AI processing
        let processed_query = if self.is_natural_language_query(&corrected_query) && self.ai_processor.is_some() {
            info!("Processing natural language query with AI");
            // Process with AI to understand the query
            let ai_processor = match self.ai_processor.as_ref() {
                Some(proc) => proc,
                None => {
                    warn!("AI processor became unavailable during query processing");
                    return Ok(SearchResults {
                        matches: vec![],
                        total_matches: 0,
                        search_time: start_time.elapsed(),
                        ai_reranked: false,
                        degradation_mode: true,
                        files_searched: 0,
                        spelling_correction: corrected_query.spelling_correction,
                        file_counts: std::collections::HashMap::new(),
                    });
                }
            };
            match ai_processor.process_query(&corrected_query.pattern).await {
                Ok(understanding) => {
                    info!("AI query understanding: intent={:?}, terms={:?}",
                          understanding.intent, understanding.search_terms);
                    // Convert AI understanding to search terms
                    let search_pattern = if understanding.search_terms.is_empty() {
                        query.pattern.clone()
                    } else {
                        understanding.search_terms.join("|")
                    };

                    SearchQuery {
                        pattern: search_pattern,
                        ai_enhanced: true,
                        ..query.clone()
                    }
                }
                Err(e) => {
                    warn!("AI processing failed, falling back to original query: {}", e);
                    // Fall back to original query if AI processing fails
                     corrected_query.clone()
                 }
             }
        } else {
            corrected_query.clone()
        };

        // Compile regex pattern
        let regex = self.get_regex(
            &processed_query.pattern,
            processed_query.case_insensitive,
            processed_query.word_regexp,
        )?;

        // Get files to search
        let files = self.get_search_files(&query)?;

        // Try to load or check index (create a temporary manager for this search)
        let root_path = query.paths.first().cloned().unwrap_or_else(|| PathBuf::from("."));
        let mut temp_manager = IndexManager::new(self.index_manager.index_dir.clone());
        let index_loaded = temp_manager.load_index(&root_path).unwrap_or(false);

        // Create fuzzy matcher based on query
        let fuzzy_matcher = if let Some(distance) = query.fuzzy {
            FuzzyMatcher::new(distance, 0.6) // Lower threshold for more matches
        } else {
            self.fuzzy_matcher.clone()
        };

        // Try LSP-enhanced search first for symbol queries
        let root_language_detection = if let Some(ref lang_processor) = self.language_processor {
            lang_processor.detect_language(&root_path).unwrap_or(None)
        } else {
            None
        };

        let lsp_matches = if self.is_symbol_query(&processed_query) {
            if let Some(lang_det) = &root_language_detection {
                // Create a Language instance for LSP
                let lang = Language(&lang_det.language_name.as_str(), &lang_det.language_id.as_str());
                self.search_with_lsp(&processed_query, &lang).await?
            } else {
                vec![]
            }
        } else {
            vec![]
        };

        // Search using index if available, otherwise fall back to file-by-file
        let mut regex_matches = if index_loaded && !temp_manager.needs_rebuild(&root_path).unwrap_or(true) {
            self.search_with_index_temp(&processed_query, &regex, &fuzzy_matcher, &temp_manager)?
        } else {
            self.search_files_fallback(&processed_query, &regex, &fuzzy_matcher)?
        };

        // Combine LSP and regex results
        let mut all_matches = lsp_matches;
        all_matches.extend(regex_matches);

        let mut ai_reranked = false;

        // Apply ranking: AI reranking if available and requested, otherwise deterministic ranking
        let mut ai_reranked = false;
        let mut degradation_detected = false;

        if processed_query.ai_enhanced && !processed_query.no_rerank {
            if let Some(ai_proc) = self.ai_processor.as_ref() {
                if let Ok(understanding) = ai_proc.process_query(&processed_query.pattern).await {
                    // AI reranking available - use it
                    let mut matches_copy = all_matches.clone();
                    self.rerank_matches_basic(&mut matches_copy, &understanding);
                    all_matches = matches_copy;
                    ai_reranked = true;
                } else {
                    // AI processing failed - fall back to deterministic ranking
                    warn!("AI processing failed, falling back to deterministic ranking");
                    self.rerank_matches_deterministic(&mut all_matches, &processed_query.pattern);
                    degradation_detected = true;
                }
            } else {
                // AI processor not available - use deterministic ranking
                debug!("AI processor not available, using deterministic ranking");
                self.rerank_matches_deterministic(&mut all_matches, &processed_query.pattern);
                degradation_detected = true;
            }
        } else {
            // AI reranking disabled - use deterministic ranking
            self.rerank_matches_deterministic(&mut all_matches, &processed_query.pattern);
        }

        // Update degradation mode status
        self.degradation_mode = degradation_detected;

        let total_matches = all_matches.len();
        let files_searched = if index_loaded {
            temp_manager.get_index().map(|idx| idx.metadata.file_count).unwrap_or(0)
        } else {
            processed_query.paths.len().max(1)
        };
        let search_time = start_time.elapsed();

        // Log search completion
        info!("Search completed: {} matches in {} files, took {:.2}ms",
              total_matches, files_searched, search_time.as_secs_f64() * 1000.0);

        // Store search in session if available
        if let Some(session_manager) = &self.session_manager {
            // Store search query and results in session context
            // This allows search history and context to persist across ricegrep invocations
            debug!("Search stored in session context");
            // TODO: Implement actual session storage of search results
        }

        // Log search activity if logger is available
        // TODO: Integrate with ricecoder-activity-log when dependency issues are resolved
        // if let Some(logger) = &self.activity_logger {
        //     debug!("Search activity logged");
        // }

        // Apply max matches quota
        let matches = if let Some(max_matches) = processed_query.max_matches {
            if all_matches.len() > max_matches {
                info!("Limiting results to {} matches (quota exceeded)", max_matches);
                all_matches.into_iter().take(max_matches).collect()
            } else {
                all_matches
            }
        } else {
            all_matches
        };

        // Build file counts for --count mode
        let mut file_counts = std::collections::HashMap::new();
        for match_result in &matches {
            *file_counts.entry(match_result.file.clone()).or_insert(0) += 1;
        }

        Ok(SearchResults {
            matches,
            total_matches,
            search_time,
            ai_reranked,
            degradation_mode: degradation_detected,
            files_searched,
            spelling_correction: corrected_query.spelling_correction,
            file_counts,
        })
    }

    async fn build_index(&mut self, paths: &[PathBuf], progress_verbosity: ProgressVerbosity) -> Result<(), RiceGrepError> {
        let root_path = paths.first().cloned().unwrap_or_else(|| PathBuf::from("."));

        // Build the index
        let index = self.build_index_internal(&root_path, progress_verbosity)?;

        // Store the index in memory
        self.index_manager.current_index = Some(Arc::new(index.clone()));

        // Save to disk
        self.index_manager.save_index(&root_path)?;

        Ok(())
    }

    fn has_index(&mut self, paths: &[PathBuf]) -> bool {
        let root_path = paths.first().cloned().unwrap_or_else(|| PathBuf::from("."));
        self.index_manager.load_index(&root_path).unwrap_or(false)
    }
}