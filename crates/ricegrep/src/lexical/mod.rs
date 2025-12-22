//! BM25 lexical index implementation built on Tantivy.

use std::path::Path;

pub use self::{
    errors::{LexicalError, LexicalResult},
    indexer::{Bm25IndexBuilder, Bm25IndexHandle, Bm25IndexWriter},
    pipeline::{LexicalIngestPipeline, LexicalIngestStats},
    schema::LexicalSchema,
    searcher::{LexicalConfig, LexicalHit, LexicalSearcher, SearchFilters},
    sharding::{LexicalShardManager, LexicalShardSet, ShardKey, ShardedLexicalSearcher},
};

mod errors;
mod indexer;
mod pipeline;
mod schema;
mod searcher;
mod sharding;
