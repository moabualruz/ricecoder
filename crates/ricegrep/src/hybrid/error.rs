use thiserror::Error;

#[derive(Debug, Error)]
pub enum FusionError {
    #[error("no candidates available")]
    NoCandidates,
    #[error("fusion method unsupported: {0}")]
    UnsupportedMethod(String),
    #[error("quality validation failed: {0}")]
    QualityValidation(String),
    #[error("normalization failed: {0}")]
    Normalization(String),
    #[error("fusion timeout")]
    Timeout,
}

#[derive(Debug, Error)]
pub enum RankingError {
    #[error("invalid candidate data: {0}")]
    InvalidCandidate(String),
    #[error("ranking algorithm error: {0}")]
    Algorithm(String),
    #[error("resource limit exceeded")]
    ResourceLimit,
}
