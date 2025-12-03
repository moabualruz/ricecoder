//! Research manager - central coordinator for analysis operations

use crate::error::ResearchError;
use crate::models::{
    ProjectContext, ArchitecturalIntent, ArchitecturalStyle,
    StandardsProfile, NamingConventions, FormattingStyle, CaseStyle, IndentType,
    ImportOrganization, ImportGroup, DocumentationStyle, DocFormat, CodeContext, SearchResult,
};
use crate::project_analyzer::ProjectAnalyzer;
use crate::codebase_scanner::CodebaseScanner;
use crate::semantic_index::SemanticIndex;
use crate::pattern_detector::PatternDetector;
use crate::standards_detector::StandardsDetector;
use crate::architectural_intent::ArchitecturalIntentTracker;
use crate::context_builder::ContextBuilder;
use crate::dependency_analyzer::DependencyAnalyzer;
use crate::cache_manager::CacheManager;
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Central coordinator for research operations
///
/// The ResearchManager orchestrates all analysis components and manages the research lifecycle.
/// It provides the main public API for research queries.
#[derive(Debug, Clone)]
pub struct ResearchManager {
    /// Project analyzer for detecting project type and structure
    project_analyzer: Arc<ProjectAnalyzer>,
    /// Pattern detector for identifying patterns
    pattern_detector: Arc<PatternDetector>,
    /// Standards detector for extracting conventions
    standards_detector: Arc<StandardsDetector>,
    /// Architectural intent tracker for understanding architecture
    architectural_intent_tracker: Arc<ArchitecturalIntentTracker>,
    /// Context builder for selecting relevant files
    context_builder: Arc<ContextBuilder>,
    /// Dependency analyzer for multi-language support
    dependency_analyzer: Arc<DependencyAnalyzer>,
    /// Cache manager for caching analysis results
    cache_manager: Arc<CacheManager>,
}

impl ResearchManager {
    /// Create a new ResearchManager with default configuration
    pub fn new() -> Self {
        ResearchManager {
            project_analyzer: Arc::new(ProjectAnalyzer::new()),
            pattern_detector: Arc::new(PatternDetector::new()),
            standards_detector: Arc::new(StandardsDetector::new()),
            architectural_intent_tracker: Arc::new(ArchitecturalIntentTracker::new()),
            context_builder: Arc::new(ContextBuilder::new(8000)), // Default 8000 token limit
            dependency_analyzer: Arc::new(DependencyAnalyzer::new()),
            cache_manager: Arc::new(CacheManager::new()),
        }
    }

    /// Create a new ResearchManager with custom configuration
    pub fn with_config(
        project_analyzer: ProjectAnalyzer,
        pattern_detector: PatternDetector,
        standards_detector: StandardsDetector,
        architectural_intent_tracker: ArchitecturalIntentTracker,
        context_builder: ContextBuilder,
        dependency_analyzer: DependencyAnalyzer,
        cache_manager: CacheManager,
    ) -> Self {
        ResearchManager {
            project_analyzer: Arc::new(project_analyzer),
            pattern_detector: Arc::new(pattern_detector),
            standards_detector: Arc::new(standards_detector),
            architectural_intent_tracker: Arc::new(architectural_intent_tracker),
            context_builder: Arc::new(context_builder),
            dependency_analyzer: Arc::new(dependency_analyzer),
            cache_manager: Arc::new(cache_manager),
        }
    }

    /// Analyze a project and gather comprehensive context
    ///
    /// This method performs a multi-stage analysis:
    /// 1. Detect project type
    /// 2. Scan codebase and build semantic index
    /// 3. Detect patterns
    /// 4. Detect standards and conventions
    /// 5. Analyze dependencies
    /// 6. Track architectural intent
    ///
    /// # Arguments
    ///
    /// * `root` - Root path of the project to analyze
    ///
    /// # Returns
    ///
    /// A `ProjectContext` containing all gathered information, or a `ResearchError`
    pub async fn analyze_project(&self, root: &Path) -> Result<ProjectContext, ResearchError> {
        // Verify project exists
        if !root.exists() {
            return Err(ResearchError::ProjectNotFound {
                path: root.to_path_buf(),
                reason: "Directory does not exist or is not accessible".to_string(),
            });
        }

        debug!("Starting project analysis for {:?}", root);

        // Check cache first (with empty file mtimes for initial check)
        use std::collections::HashMap;
        use std::time::SystemTime;
        let empty_mtimes: HashMap<std::path::PathBuf, SystemTime> = HashMap::new();
        if let Ok(Some(cached)) = self.cache_manager.get(root, &empty_mtimes) {
            info!("Using cached analysis for {:?}", root);
            return Ok(cached);
        }

        // 1. Detect project type
        debug!("Step 1: Detecting project type");
        let project_type = self.project_analyzer.detect_type(root)
            .map_err(|e| {
                warn!("Failed to detect project type: {}", e);
                e
            })?;
        debug!("Detected project type: {:?}", project_type);

        // 2. Analyze project structure
        debug!("Step 2: Analyzing project structure");
        let structure = self.project_analyzer.analyze_structure(root)
            .map_err(|e| {
                warn!("Failed to analyze project structure: {}", e);
                e
            })?;
        debug!("Found {} source directories", structure.source_dirs.len());

        // 3. Identify frameworks
        debug!("Step 3: Identifying frameworks");
        let frameworks = self.project_analyzer.identify_frameworks(root)
            .unwrap_or_default();
        debug!("Identified {} frameworks", frameworks.len());

        // 4. Scan codebase and build semantic index
        debug!("Step 4: Scanning codebase");
        let scan_result = CodebaseScanner::scan(root)
            .map_err(|e| {
                warn!("Failed to scan codebase: {}", e);
                e
            })?;
        debug!("Found {} files in codebase", scan_result.files.len());

        // 5. Build semantic index
        debug!("Step 5: Building semantic index");
        let mut semantic_index = SemanticIndex::new();
        
        // Extract symbols from scanned files
        for file_meta in &scan_result.files {
            if let Some(language) = &file_meta.language {
                if let Ok(content) = std::fs::read_to_string(&file_meta.path) {
                    if let Ok(symbols) = crate::symbol_extractor::SymbolExtractor::extract_symbols(
                        &file_meta.path,
                        language,
                        &content,
                    ) {
                        for symbol in symbols {
                            semantic_index.add_symbol(symbol);
                        }
                    }
                }
            }
        }
        debug!("Built semantic index with symbols");

        // 6. Track references
        debug!("Step 6: Tracking cross-file references");
        let mut known_symbols: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        for symbol in semantic_index.all_symbols() {
            known_symbols.insert(symbol.name.clone(), symbol.id.clone());
        }
        
        for file_meta in &scan_result.files {
            if let Some(language) = &file_meta.language {
                if let Ok(content) = std::fs::read_to_string(&file_meta.path) {
                    if let Ok(references) = crate::reference_tracker::ReferenceTracker::track_references(
                        &file_meta.path,
                        language,
                        &content,
                        &known_symbols,
                    ) {
                        for reference in references {
                            semantic_index.add_reference(reference);
                        }
                    }
                }
            }
        }
        debug!("Tracked cross-file references");

        // 7. Detect patterns
        debug!("Step 7: Detecting patterns");
        let patterns = self.pattern_detector.detect(&scan_result)
            .unwrap_or_default();
        debug!("Detected {} patterns", patterns.len());

        // 8. Detect standards and conventions
        debug!("Step 8: Detecting standards and conventions");
        let file_paths: Vec<&Path> = scan_result.files.iter().map(|f| f.path.as_path()).collect();
        let standards = self.standards_detector.detect(&file_paths)
            .unwrap_or_else(|_| StandardsProfile {
                naming_conventions: NamingConventions {
                    function_case: CaseStyle::Mixed,
                    variable_case: CaseStyle::Mixed,
                    class_case: CaseStyle::Mixed,
                    constant_case: CaseStyle::Mixed,
                },
                formatting_style: FormattingStyle {
                    indent_size: 4,
                    indent_type: IndentType::Spaces,
                    line_length: 100,
                },
                import_organization: ImportOrganization {
                    order: vec![ImportGroup::Standard, ImportGroup::External, ImportGroup::Internal],
                    sort_within_group: false,
                },
                documentation_style: DocumentationStyle {
                    format: DocFormat::RustDoc,
                    required_for_public: false,
                },
            });
        debug!("Detected standards and conventions");

        // 9. Analyze dependencies (multi-language)
        debug!("Step 9: Analyzing dependencies");
        let dependencies = self.dependency_analyzer.analyze(root)
            .unwrap_or_default();
        debug!("Found {} dependencies", dependencies.len());

        // 10. Track architectural intent
        debug!("Step 10: Tracking architectural intent");
        let architectural_style = self.architectural_intent_tracker.infer_style(root)
            .unwrap_or(ArchitecturalStyle::Unknown);
        let architectural_intent = ArchitecturalIntent {
            style: architectural_style,
            principles: vec![],
            constraints: vec![],
            decisions: vec![],
        };
        debug!("Inferred architectural style: {:?}", architectural_intent.style);

        // 11. Build final context
        debug!("Step 11: Building final project context");
        let context = ProjectContext {
            project_type,
            languages: scan_result.languages,
            frameworks,
            structure,
            patterns,
            dependencies,
            architectural_intent,
            standards,
        };

        // Cache results
        debug!("Caching analysis results");
        let file_mtimes: HashMap<std::path::PathBuf, SystemTime> = scan_result.files
            .iter()
            .filter_map(|f| {
                std::fs::metadata(&f.path)
                    .ok()
                    .and_then(|m| m.modified().ok())
                    .map(|mtime| (f.path.clone(), mtime))
            })
            .collect();
        
        if let Err(e) = self.cache_manager.set(root, &context, file_mtimes) {
            warn!("Failed to cache analysis results: {}", e);
            // Don't fail the analysis if caching fails
        }

        info!("Project analysis completed successfully for {:?}", root);
        Ok(context)
    }

    /// Search the codebase for symbols and patterns
    ///
    /// Performs semantic search across the indexed codebase to find symbols,
    /// references, and patterns matching the query.
    ///
    /// # Arguments
    ///
    /// * `query` - Search query string
    /// * `semantic_index` - The semantic index to search
    ///
    /// # Returns
    ///
    /// A vector of search results, or a `ResearchError`
    pub async fn search_codebase(
        &self,
        query: &str,
        semantic_index: &SemanticIndex,
    ) -> Result<Vec<SearchResult>, ResearchError> {
        if query.is_empty() {
            return Err(ResearchError::SearchFailed {
                query: query.to_string(),
                reason: "Query string cannot be empty".to_string(),
            });
        }

        debug!("Searching codebase for: {}", query);

        // Use semantic index to search by name
        let results = semantic_index.search_by_name(query);
        
        debug!("Found {} search results", results.len());
        Ok(results)
    }

    /// Get context for a specific query
    ///
    /// Automatically selects and prioritizes relevant files based on the query.
    /// This method combines semantic search with relevance scoring to provide
    /// the most relevant context for AI providers.
    ///
    /// # Arguments
    ///
    /// * `query` - Query string describing what context is needed
    /// * `all_files` - All available files to select from
    ///
    /// # Returns
    ///
    /// A `CodeContext` with relevant files and symbols, or a `ResearchError`
    pub async fn get_context_for_query(
        &self,
        query: &str,
        all_files: Vec<crate::models::FileContext>,
    ) -> Result<CodeContext, ResearchError> {
        if query.is_empty() {
            return Err(ResearchError::AnalysisFailed {
                reason: "Query string cannot be empty".to_string(),
                context: "Context building requires a non-empty query to select relevant files".to_string(),
            });
        }

        debug!("Building context for query: {}", query);

        // Select relevant files based on query
        let relevant_files = self.context_builder.select_relevant_files(query, all_files)
            .map_err(|e| {
                warn!("Failed to select relevant files: {}", e);
                e
            })?;

        debug!("Selected {} relevant files", relevant_files.len());

        // Build context from selected files
        let context = self.context_builder.build_context(relevant_files)
            .map_err(|e| {
                warn!("Failed to build context: {}", e);
                e
            })?;

        debug!("Built context with {} tokens", context.total_tokens);
        Ok(context)
    }

    /// Get cache statistics
    ///
    /// # Returns
    ///
    /// Cache statistics including hit rate, miss rate, and size, or a `ResearchError`
    pub fn get_cache_statistics(&self) -> Result<crate::cache_manager::CacheStatistics, ResearchError> {
        self.cache_manager.statistics()
    }

    /// Clear the cache
    ///
    /// # Returns
    ///
    /// A `ResearchError` if clearing fails
    pub fn clear_cache(&self) -> Result<(), ResearchError> {
        self.cache_manager.clear()
    }
}

impl Default for ResearchManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for ResearchManager with custom configuration
#[derive(Debug)]
pub struct ResearchManagerBuilder {
    project_analyzer: Option<ProjectAnalyzer>,
    pattern_detector: Option<PatternDetector>,
    standards_detector: Option<StandardsDetector>,
    architectural_intent_tracker: Option<ArchitecturalIntentTracker>,
    context_builder: Option<ContextBuilder>,
    dependency_analyzer: Option<DependencyAnalyzer>,
    cache_manager: Option<CacheManager>,
}

impl ResearchManagerBuilder {
    /// Create a new builder with default components
    pub fn new() -> Self {
        ResearchManagerBuilder {
            project_analyzer: None,
            pattern_detector: None,
            standards_detector: None,
            architectural_intent_tracker: None,
            context_builder: None,
            dependency_analyzer: None,
            cache_manager: None,
        }
    }

    /// Set the project analyzer
    pub fn with_project_analyzer(mut self, analyzer: ProjectAnalyzer) -> Self {
        self.project_analyzer = Some(analyzer);
        self
    }

    /// Set the pattern detector
    pub fn with_pattern_detector(mut self, detector: PatternDetector) -> Self {
        self.pattern_detector = Some(detector);
        self
    }

    /// Set the standards detector
    pub fn with_standards_detector(mut self, detector: StandardsDetector) -> Self {
        self.standards_detector = Some(detector);
        self
    }

    /// Set the architectural intent tracker
    pub fn with_architectural_intent_tracker(mut self, tracker: ArchitecturalIntentTracker) -> Self {
        self.architectural_intent_tracker = Some(tracker);
        self
    }

    /// Set the context builder
    pub fn with_context_builder(mut self, builder: ContextBuilder) -> Self {
        self.context_builder = Some(builder);
        self
    }

    /// Set the dependency analyzer
    pub fn with_dependency_analyzer(mut self, analyzer: DependencyAnalyzer) -> Self {
        self.dependency_analyzer = Some(analyzer);
        self
    }

    /// Set the cache manager
    pub fn with_cache_manager(mut self, manager: CacheManager) -> Self {
        self.cache_manager = Some(manager);
        self
    }

    /// Build the ResearchManager
    pub fn build(self) -> ResearchManager {
        ResearchManager::with_config(
            self.project_analyzer.unwrap_or_default(),
            self.pattern_detector.unwrap_or_default(),
            self.standards_detector.unwrap_or_default(),
            self.architectural_intent_tracker.unwrap_or_default(),
            self.context_builder.unwrap_or_else(|| ContextBuilder::new(8000)),
            self.dependency_analyzer.unwrap_or_default(),
            self.cache_manager.unwrap_or_default(),
        )
    }
}

impl Default for ResearchManagerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_research_manager_creation() {
        let manager = ResearchManager::new();
        // Manager should be created successfully
        assert!(Arc::strong_count(&manager.project_analyzer) > 0);
    }

    #[test]
    fn test_research_manager_default() {
        let manager = ResearchManager::default();
        assert!(Arc::strong_count(&manager.project_analyzer) > 0);
    }

    #[test]
    fn test_research_manager_builder() {
        let manager = ResearchManagerBuilder::new()
            .with_project_analyzer(ProjectAnalyzer::new())
            .with_pattern_detector(PatternDetector::new())
            .build();
        assert!(Arc::strong_count(&manager.project_analyzer) > 0);
    }

    #[test]
    fn test_research_manager_builder_default() {
        let manager = ResearchManagerBuilder::default().build();
        assert!(Arc::strong_count(&manager.project_analyzer) > 0);
    }

    #[tokio::test]
    async fn test_analyze_project_nonexistent_path() {
        let manager = ResearchManager::new();
        let result = manager.analyze_project(Path::new("/nonexistent/path")).await;
        assert!(result.is_err());
        match result {
            Err(ResearchError::ProjectNotFound { path: _, reason: _ }) => (),
            _ => panic!("Expected ProjectNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_search_codebase_empty_query() {
        let manager = ResearchManager::new();
        let semantic_index = SemanticIndex::new();
        let result = manager.search_codebase("", &semantic_index).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_context_for_query_empty_query() {
        let manager = ResearchManager::new();
        let result = manager.get_context_for_query("", vec![]).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_cache_statistics() {
        let manager = ResearchManager::new();
        let stats = manager.get_cache_statistics();
        assert!(stats.is_ok());
        if let Ok(s) = stats {
            assert_eq!(s.hits, 0);
            assert_eq!(s.misses, 0);
        }
    }

    #[test]
    fn test_clear_cache() {
        let manager = ResearchManager::new();
        let result = manager.clear_cache();
        assert!(result.is_ok());
    }
}
