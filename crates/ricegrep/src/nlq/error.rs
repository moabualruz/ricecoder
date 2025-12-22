use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProcessingError {
    #[error("parsing failed: {0}")]
    Parse(String),
    #[error("classification failed: {0}")]
    Classification(String),
    #[error("enrichment failed: {0}")]
    Enrichment(String),
    #[error("extraction failed: {0}")]
    Extraction(String),
    #[error("unsupported query")]
    Unsupported,
}

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("query too long ({0} > {1})")]
    TooLong(usize, usize),
    #[error("invalid characters in query")]
    InvalidCharacters,
    #[error("security constraint violated")]
    SecurityViolation,
}
