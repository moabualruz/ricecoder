use crate::hybrid::models::{CandidateChunk, RetrievalSource};

#[derive(Debug, Clone)]
pub struct CandidateSelector {
    pub max_candidates: usize,
    pub deduplication: bool,
}

impl CandidateSelector {
    pub fn new(max_candidates: usize, deduplication: bool) -> Self {
        Self {
            max_candidates,
            deduplication,
        }
    }

    pub fn select_candidates(
        &self,
        bm25_results: Vec<CandidateChunk>,
        ann_results: Vec<CandidateChunk>,
    ) -> Vec<CandidateChunk> {
        let mut combined = Vec::with_capacity(
            bm25_results.len() + ann_results.len(),
        );
        combined.extend(bm25_results);
        combined.extend(ann_results);
        if self.deduplication {
            combined.sort_by(|a, b| a.chunk_id.cmp(&b.chunk_id));
            combined.dedup_by_key(|entry| entry.chunk_id);
        }
        combined.truncate(self.max_candidates);
        combined
    }

    pub fn apply_diversity(&self, candidates: Vec<CandidateChunk>) -> Vec<CandidateChunk> {
        candidates
    }
}
