use std::{path::PathBuf, sync::Arc, time::Instant};

use thiserror::Error;
use uuid::Uuid;

use crate::{
    api::models::{SearchFilters as ApiFilters, SearchRequest, SearchResponse, SearchResult},
    chunking::{ChunkMetadata, LanguageKind},
    metadata::{ChunkMetadataView, MetadataStore},
    nlq::{IntentClassifier, QueryEnricher, QueryFilter, QueryIntent, QueryParser},
    vector::{
        fallback::{FallbackHit, FallbackResult},
        qdrant::SearchFilters as VectorFilters,
        search::HybridQueryEngine,
    },
};

/// Coordinates NLQ parsing + hybrid search so API requests flow through the new pipeline.
pub struct SearchCoordinator {
    parser: QueryParser,
    classifier: IntentClassifier,
    enricher: QueryEnricher,
    hybrid_engine: Arc<HybridQueryEngine>,
    default_limit: usize,
    metadata_store: Arc<MetadataStore>,
}

#[derive(Debug, Error)]
pub enum SearchError {
    #[error("NLQ parsing failure: {0}")]
    Parse(String),
    #[error("NLQ enrichment failure: {0}")]
    Enrich(String),
    #[error("hybrid execution failure: {0}")]
    Hybrid(String),
}

impl SearchCoordinator {
    pub fn new(
        parser: QueryParser,
        classifier: IntentClassifier,
        enricher: QueryEnricher,
        hybrid_engine: Arc<HybridQueryEngine>,
        metadata_store: Arc<MetadataStore>,
        default_limit: usize,
    ) -> Self {
        Self {
            parser,
            classifier,
            enricher,
            hybrid_engine,
            default_limit,
            metadata_store,
        }
    }

    pub async fn execute(&self, request: SearchRequest) -> Result<SearchResponse, SearchError> {
        let parsed = self
            .parser
            .parse(&request.query)
            .map_err(|err| SearchError::Parse(err.to_string()))?;
        self.parser
            .validate(&parsed)
            .map_err(|err| SearchError::Parse(err.to_string()))?;

        let intent = self
            .classifier
            .classify(&parsed)
            .await
            .map_err(|err| SearchError::Hybrid(err.to_string()))?;
        let enriched = self
            .enricher
            .enrich(&parsed, &intent)
            .await
            .map_err(|err| SearchError::Enrich(err.to_string()))?;

        let limit = request.limit.unwrap_or(self.default_limit);
        let filters = Self::merge_filters(request.filters.as_ref(), &enriched.filters);

        let start = Instant::now();
        let fallback = self
            .hybrid_engine
            .search(&Self::join_terms(&enriched.expanded_terms), limit, filters)
            .await
            .map_err(|err| SearchError::Hybrid(err.to_string()))?;
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;

        let hits = fallback.hits;
        let total_found = hits.len();
        let results = hits
            .into_iter()
            .map(|hit| self.map_hit(hit, &enriched.expanded_terms))
            .collect();

        Ok(SearchResponse {
            results,
            total_found,
            query_time_ms: elapsed,
            request_id: Uuid::new_v4().to_string(),
        })
    }

    fn merge_filters(
        api_filters: Option<&ApiFilters>,
        inline: &[QueryFilter],
    ) -> Option<VectorFilters> {
        let mut filters = VectorFilters {
            language: None,
            repository_id: None,
            file_path_pattern: None,
        };
        let mut touched = false;

        if let Some(api) = api_filters {
            if api.language.is_some() {
                filters.language = api.language.clone();
                touched = true;
            }
            if let Some(repo) = api.repository_id {
                filters.repository_id = Some(repo);
                touched = true;
            }
            if api.file_path_pattern.is_some() {
                filters.file_path_pattern = api.file_path_pattern.clone();
                touched = true;
            }
        }

        for filter in inline {
            match filter.field.as_str() {
                "language" | "lang" => {
                    filters.language = Some(filter.value.clone());
                    touched = true;
                }
                "repo" | "repo_id" | "repository" => {
                    if let Ok(id) = filter.value.parse::<u32>() {
                        filters.repository_id = Some(id);
                        touched = true;
                    }
                }
                "file" | "path" => {
                    filters.file_path_pattern = Some(filter.value.clone());
                    touched = true;
                }
                _ => {}
            }
        }

        if touched {
            Some(filters)
        } else {
            None
        }
    }

    fn join_terms(terms: &[String]) -> String {
        terms.join(" ")
    }

    fn map_hit(&self, hit: FallbackHit, terms: &[String]) -> SearchResult {
        SearchResult {
            chunk_id: hit.chunk_id,
            score: hit.final_score,
            content: String::new(),
            metadata: self.chunk_metadata_from_hit(&hit),
            highlights: terms.iter().take(3).cloned().collect::<Vec<String>>(),
        }
    }

    fn chunk_metadata_from_hit(&self, hit: &FallbackHit) -> ChunkMetadata {
        self.metadata_store
            .get(hit.chunk_id)
            .map(|view| self.metadata_from_view(&view))
            .unwrap_or_else(|_| self.metadata_from_hit(hit))
    }

    fn metadata_from_view(&self, view: &ChunkMetadataView) -> ChunkMetadata {
        ChunkMetadata {
            chunk_id: view.chunk_id,
            repository_id: view.repository_id,
            file_path: PathBuf::from(&view.file_path),
            language: Self::language_from_name(&view.language),
            start_line: view.start_line,
            end_line: view.end_line,
            token_count: view.token_count,
            checksum: view.checksum.to_string(),
        }
    }

    fn metadata_from_hit(&self, hit: &FallbackHit) -> ChunkMetadata {
        ChunkMetadata {
            chunk_id: hit.chunk_id,
            repository_id: hit.repository_id,
            file_path: PathBuf::from(hit.file_path.clone()),
            language: Self::language_from_name(&hit.language),
            start_line: 0,
            end_line: 0,
            token_count: 0,
            checksum: String::new(),
        }
    }

    fn language_from_name(value: &str) -> LanguageKind {
        match value.to_lowercase().as_str() {
            "rust" => LanguageKind::Rust,
            "python" => LanguageKind::Python,
            "javascript" => LanguageKind::JavaScript,
            "typescript" => LanguageKind::TypeScript,
            "tsx" => LanguageKind::Tsx,
            "java" => LanguageKind::Java,
            "go" => LanguageKind::Go,
            "c" => LanguageKind::C,
            "cpp" => LanguageKind::Cpp,
            _ => LanguageKind::PlainText,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Arc;

    use anyhow::Result;
    use async_trait::async_trait;
    use tempfile::NamedTempFile;

    use super::*;
    use crate::{
        chunking::{Chunk, ChunkMetadata, LanguageKind},
        metadata::{MetadataStore, MetadataWriter},
        nlq::{IntentClassifier, QueryEnricher, QueryParser},
        vector::{
            fallback::{FallbackArtifacts, FallbackEngine, FallbackHit, FallbackWeights, PmiGraph},
            qdrant::{SearchFilters as VectorFilters, VectorHit, VectorSearchBackend},
            search::{EmbeddingProvider, HybridQueryEngine},
        },
    };

    struct DummyEmbedding {
        dimension: usize,
    }

    impl EmbeddingProvider for DummyEmbedding {
        fn dimension(&self) -> usize {
            self.dimension
        }

        fn embed_text(&self, _text: &str) -> Result<Vec<f32>> {
            Ok(vec![0.0; self.dimension])
        }
    }

    struct DummyVectorBackend {
        dimension: usize,
    }

    #[async_trait]
    impl VectorSearchBackend for DummyVectorBackend {
        fn dimension(&self) -> usize {
            self.dimension
        }

        async fn search_vectors(
            &self,
            _vector: Vec<f32>,
            _limit: usize,
            _filters: Option<VectorFilters>,
        ) -> Result<Vec<VectorHit>> {
            Ok(Vec::new())
        }
    }

    fn stub_chunk(id: u64) -> Chunk {
        let path = PathBuf::from(format!("chunk_{id}.rs"));
        let metadata = ChunkMetadata {
            chunk_id: id,
            repository_id: Some(1),
            file_path: path.clone(),
            language: LanguageKind::Rust,
            start_line: 42,
            end_line: 43,
            token_count: 5,
            checksum: format!("cksum-{id}"),
        };
        Chunk {
            id,
            language: metadata.language,
            file_path: path,
            start_line: metadata.start_line,
            end_line: metadata.end_line,
            text: "fn stub() {}".to_string(),
            identifiers: vec!["stub".into()],
            identifier_tokens: vec!["stub".into()],
            metadata,
        }
    }

    fn create_hybrid_engine() -> Arc<HybridQueryEngine> {
        let embeddings = Arc::new(DummyEmbedding { dimension: 8 });
        let vector_backend = Arc::new(DummyVectorBackend { dimension: 8 });
        let artifacts = Arc::new(FallbackArtifacts::new(Arc::new(PmiGraph::default())));
        let fallback_engine = Arc::new(FallbackEngine::new(
            artifacts.clone(),
            FallbackWeights::default(),
        ));
        Arc::new(HybridQueryEngine::new(
            embeddings,
            vector_backend,
            fallback_engine,
            artifacts,
        ))
    }

    #[test]
    fn map_hit_prefers_metadata_store() {
        let mut writer = MetadataWriter::new();
        writer.add_chunk(&stub_chunk(1));
        let file = NamedTempFile::new().expect("create temp metadata file");
        writer.finalize(file.path()).expect("persist metadata");
        let metadata_store =
            Arc::new(MetadataStore::load(file.path()).expect("load metadata store"));
        let expected_checksum = metadata_store
            .get(1)
            .expect("read chunk view")
            .checksum
            .to_string();

        let coordinator = super::SearchCoordinator {
            parser: QueryParser::new(1024),
            classifier: IntentClassifier::with_defaults(),
            enricher: QueryEnricher::new(),
            hybrid_engine: create_hybrid_engine(),
            default_limit: 10,
            metadata_store: metadata_store.clone(),
        };

        let hit = FallbackHit {
            chunk_id: 1,
            file_path: "chunk_1.rs".to_string(),
            language: "Rust".to_string(),
            repository_id: Some(1),
            bm25_score: 1.0,
            identifier_score: 0.0,
            pmi_score: 0.0,
            ngram_score: 0.0,
            final_score: 1.0,
        };

        let result = coordinator.map_hit(hit, &["stub".into()]);
        assert_eq!(result.metadata.chunk_id, 1);
        assert_eq!(result.metadata.start_line, 42);
        assert_eq!(result.metadata.end_line, 43);
        assert_eq!(result.metadata.file_path, PathBuf::from("chunk_1.rs"));
        assert_eq!(result.metadata.checksum, expected_checksum);
    }

    #[test]
    fn map_hit_falls_back_when_missing_metadata() {
        let empty_file = NamedTempFile::new().expect("temp");
        MetadataWriter::new()
            .finalize(empty_file.path())
            .expect("write empty metadata");
        let metadata_store = Arc::new(MetadataStore::load(empty_file.path()).expect("empty store"));

        let coordinator = super::SearchCoordinator {
            parser: QueryParser::new(1024),
            classifier: IntentClassifier::with_defaults(),
            enricher: QueryEnricher::new(),
            hybrid_engine: create_hybrid_engine(),
            default_limit: 10,
            metadata_store,
        };

        let hit = FallbackHit {
            chunk_id: 7,
            file_path: "missing.rs".to_string(),
            language: "Python".to_string(),
            repository_id: None,
            bm25_score: 1.0,
            identifier_score: 0.0,
            pmi_score: 0.0,
            ngram_score: 0.0,
            final_score: 1.0,
        };

        let result = coordinator.map_hit(hit, &["missing".into()]);
        assert_eq!(result.metadata.chunk_id, 7);
        assert_eq!(result.metadata.file_path, PathBuf::from("missing.rs"));
        assert_eq!(result.metadata.language, LanguageKind::Python);
        assert_eq!(result.metadata.checksum, String::new());
    }
}
