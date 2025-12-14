//! Research manager - central coordinator for analysis operations

use crate::error::ResearchError;
use crate::models::ProjectContext;
use crate::project_analyzer::ProjectAnalyzer;
use crate::reference_tracker::ReferenceTracker;
use crate::relevance_scorer::RelevanceScorer;
use crate::search_engine::SearchEngine;
use crate::semantic_index::SemanticIndex;
use crate::standards_detector::StandardsDetector;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Central coordinator for core research operations
///
/// The ResearchManager orchestrates core analysis components and manages the research lifecycle.
/// It provides the main public API for research queries with MCP integration.
#[derive(Debug, Clone)]
pub struct ResearchManager {
    /// Project analyzer for detecting project type and structure
    project_analyzer: Arc<ProjectAnalyzer>,
    /// Standards detector for extracting conventions
    standards_detector: Arc<StandardsDetector>,
    /// Semantic index for code understanding
    semantic_index: Arc<SemanticIndex>,
    /// Search engine for code and documentation search
    search_engine: Arc<SearchEngine>,
    /// Reference tracker for symbol relationships
    reference_tracker: Arc<ReferenceTracker>,
    /// Relevance scorer for ranking results
    relevance_scorer: Arc<RelevanceScorer>,
}

impl ResearchManager {
    /// Create a new ResearchManager with default configuration
    pub fn new() -> Self {
        let semantic_index = Arc::new(SemanticIndex::new());
        ResearchManager {
            project_analyzer: Arc::new(ProjectAnalyzer::new()),
            standards_detector: Arc::new(StandardsDetector::new()),
            semantic_index: Arc::clone(&semantic_index),
            search_engine: Arc::new(SearchEngine::new((*semantic_index).clone())),
            reference_tracker: Arc::new(ReferenceTracker),
            relevance_scorer: Arc::new(RelevanceScorer::new()),
        }
    }

    /// Create a new ResearchManager with custom components
    pub fn with_components(
        project_analyzer: ProjectAnalyzer,
        standards_detector: StandardsDetector,
        semantic_index: SemanticIndex,
        search_engine: SearchEngine,
        reference_tracker: ReferenceTracker,
        relevance_scorer: RelevanceScorer,
    ) -> Self {
        ResearchManager {
            project_analyzer: Arc::new(project_analyzer),
            standards_detector: Arc::new(standards_detector),
            semantic_index: Arc::new(semantic_index),
            search_engine: Arc::new(search_engine),
            reference_tracker: Arc::new(reference_tracker),
            relevance_scorer: Arc::new(relevance_scorer),
        }
    }

    /// Analyze a project and gather core research context
    ///
    /// This method performs core analysis:
    /// 1. Detect project type and structure
    /// 2. Build semantic index for code understanding
    /// 3. Detect standards and conventions
    /// 4. Initialize search and reference tracking
    ///
    /// # Arguments
    ///
    /// * `root` - Root path of the project to analyze
    ///
    /// # Returns
    ///
    /// A `ProjectContext` containing core analysis information, or a `ResearchError`
    pub async fn analyze_project(&self, root: &Path) -> Result<ProjectContext, ResearchError> {
        // Verify project exists
        if !root.exists() {
            return Err(ResearchError::ProjectNotFound {
                path: root.to_path_buf(),
                reason: "Directory does not exist or is not accessible".to_string(),
            });
        }

        debug!("Starting core project analysis for {:?}", root);

        // 1. Detect project type
        debug!("Step 1: Detecting project type");
        let project_type = self.project_analyzer.detect_type(root).map_err(|e| {
            warn!("Failed to detect project type: {}", e);
            e
        })?;
        debug!("Detected project type: {:?}", project_type);

        // 2. Analyze project structure
        debug!("Step 2: Analyzing project structure");
        let structure = self.project_analyzer.analyze_structure(root).map_err(|e| {
            warn!("Failed to analyze project structure: {}", e);
            e
        })?;
        debug!("Found {} source directories", structure.source_dirs.len());

        // 3. Detect standards and conventions
        debug!("Step 3: Detecting standards and conventions");
        let standards = self.standards_detector.detect(&[root]).unwrap_or_default();
        debug!("Detected standards profile");

        // 4. Build semantic index
        debug!("Step 4: Building semantic index");
        // Initialize semantic index (would be populated during indexing)
        let _semantic_index = &self.semantic_index;

        // 5. Build final context with core information
        debug!("Step 5: Building project context");
        let context = ProjectContext {
            project_type,
            languages: vec![], // Would be populated by project analyzer
            frameworks: vec![], // Would be populated by project analyzer
            structure,
            patterns: vec![], // Patterns moved to separate crate
            dependencies: vec![], // Dependencies moved to separate crate
            architectural_intent: crate::models::ArchitecturalIntent {
                style: crate::models::ArchitecturalStyle::Unknown,
                principles: vec![],
                constraints: vec![],
                decisions: vec![],
            }, // Architectural analysis moved to separate crate
            standards,
        };

        info!("Core project analysis completed successfully for {:?}", root);
        Ok(context)
    }

    /// Search the codebase using the search engine
    pub fn search(&self, query: &str, options: &crate::search_engine::SearchOptions) -> Vec<crate::models::SearchResult> {
        self.search_engine.search_by_name(query, options)
    }

    /// Get relevance score for search results
    pub fn score_relevance(&self, results: &mut Vec<crate::models::SearchResult>, query: &str) {
        // Convert SearchResults to symbols for scoring
        let symbols: Vec<_> = results.iter().map(|r| r.symbol.clone()).collect();
        let scores = self.relevance_scorer.score_symbols(&symbols, query);

        // Update results with scores
        for (i, (_, score)) in scores.iter().enumerate() {
            if let Some(result) = results.get_mut(i) {
                result.relevance = *score;
            }
        }
    }

    /// Track references in code
    pub fn track_references(&self, file_path: &Path, content: &str) -> Result<crate::reference_tracker::ReferenceTrackingResult, ResearchError> {
        // This would use the semantic index to track references
        // For now, return a basic result
        Ok(crate::reference_tracker::ReferenceTrackingResult {
            references_by_symbol: HashMap::new(),
            references_by_file: HashMap::from([(file_path.to_path_buf(), vec![])]),
        })
    }

    /// Get semantic index for advanced queries
    pub fn semantic_index(&self) -> &SemanticIndex {
        &self.semantic_index
    }

    /// Get reference tracker
    pub fn reference_tracker(&self) -> &ReferenceTracker {
        &self.reference_tracker
    }
}

