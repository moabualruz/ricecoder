use std::sync::Arc;

use async_graphql::{
    Context, EmptyMutation, EmptySubscription, InputObject, Object, Schema, SimpleObject,
};

use crate::{
    api::{
        auth::AuthCredentials,
        execution::SearchExecutor,
        models::{RankingConfig, SearchFilters, SearchRequest, SearchResponse, SearchResult},
    },
    chunking::ChunkMetadata,
};

pub type GraphQLSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

#[derive(Clone)]
pub struct GraphQLDependencies {
    executor: Arc<SearchExecutor>,
}

impl GraphQLDependencies {
    pub fn new(executor: Arc<SearchExecutor>) -> Self {
        Self { executor }
    }
}

pub fn build_graphql_schema(executor: Arc<SearchExecutor>) -> GraphQLSchema {
    Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .data(GraphQLDependencies::new(executor))
        .finish()
}

#[derive(Default)]
pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn search(
        &self,
        ctx: &Context<'_>,
        input: GraphQLSearchRequest,
    ) -> async_graphql::Result<GraphQLSearchResponse> {
        let deps = ctx.data::<GraphQLDependencies>()?;
        let credentials = ctx.data::<AuthCredentials>()?.clone();
        let request = input.into_search_request();
        let response = deps
            .executor
            .execute(request, credentials, "graphql.search")
            .await
            .map_err(|err| async_graphql::Error::new(err.to_string()))?;
        Ok(response.into())
    }
}

#[derive(InputObject)]
pub struct GraphQLSearchRequest {
    pub query: String,
    pub limit: Option<i32>,
    pub filters: Option<GraphQLSearchFilters>,
    pub ranking: Option<GraphQLRankingConfig>,
    pub timeout_ms: Option<u64>,
}

impl GraphQLSearchRequest {
    pub fn into_search_request(self) -> SearchRequest {
        SearchRequest {
            query: self.query,
            limit: self.limit.map(|value| value as usize),
            filters: self.filters.map(|filters| filters.into()),
            ranking: self.ranking.map(|ranking| ranking.into()),
            timeout_ms: self.timeout_ms,
        }
    }
}

#[derive(InputObject)]
pub struct GraphQLSearchFilters {
    pub repository_id: Option<u32>,
    pub language: Option<String>,
    pub file_path_pattern: Option<String>,
}

impl From<GraphQLSearchFilters> for SearchFilters {
    fn from(filters: GraphQLSearchFilters) -> Self {
        SearchFilters {
            repository_id: filters.repository_id,
            language: filters.language,
            file_path_pattern: filters.file_path_pattern,
        }
    }
}

#[derive(InputObject)]
pub struct GraphQLRankingConfig {
    pub lexical_weight: f32,
    pub semantic_weight: f32,
    pub rrf_k: usize,
}

impl From<GraphQLRankingConfig> for RankingConfig {
    fn from(config: GraphQLRankingConfig) -> Self {
        RankingConfig {
            lexical_weight: config.lexical_weight,
            semantic_weight: config.semantic_weight,
            rrf_k: config.rrf_k,
        }
    }
}

#[derive(SimpleObject)]
pub struct GraphQLSearchResponse {
    pub results: Vec<GraphQLSearchResult>,
    pub total_found: usize,
    pub query_time_ms: f64,
    pub request_id: String,
}

impl From<SearchResponse> for GraphQLSearchResponse {
    fn from(response: SearchResponse) -> Self {
        Self {
            results: response
                .results
                .into_iter()
                .map(GraphQLSearchResult::from)
                .collect(),
            total_found: response.total_found,
            query_time_ms: response.query_time_ms,
            request_id: response.request_id,
        }
    }
}

#[derive(SimpleObject)]
pub struct GraphQLSearchResult {
    pub chunk_id: u64,
    pub score: f32,
    pub content: String,
    pub metadata: GraphQLChunkMetadata,
    pub highlights: Vec<String>,
}

impl From<SearchResult> for GraphQLSearchResult {
    fn from(result: SearchResult) -> Self {
        Self {
            chunk_id: result.chunk_id,
            score: result.score,
            content: result.content,
            metadata: result.metadata.into(),
            highlights: result.highlights,
        }
    }
}

#[derive(SimpleObject)]
pub struct GraphQLChunkMetadata {
    pub chunk_id: u64,
    pub repository_id: Option<u32>,
    pub file_path: String,
    pub language: String,
    pub start_line: u32,
    pub end_line: u32,
    pub token_count: u32,
    pub checksum: String,
}

impl From<ChunkMetadata> for GraphQLChunkMetadata {
    fn from(metadata: ChunkMetadata) -> Self {
        Self {
            chunk_id: metadata.chunk_id,
            repository_id: metadata.repository_id,
            file_path: metadata.file_path.to_string_lossy().into_owned(),
            language: metadata.language.to_string(),
            start_line: metadata.start_line,
            end_line: metadata.end_line,
            token_count: metadata.token_count,
            checksum: metadata.checksum,
        }
    }
}
