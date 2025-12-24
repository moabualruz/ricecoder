use thiserror::Error;

pub type ChunkingResult<T> = Result<T, ChunkingError>;

#[derive(Debug, Error)]
pub enum ChunkingError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Tree-Sitter parsing failed: {0}")]
    Parser(String),

    #[error("Unsupported language for file {path}")]
    UnsupportedLanguage { path: String },

    #[error("File exceeds allowed size: {path}")]
    FileTooLarge { path: String },

    #[error("Glob pattern error: {0}")]
    Glob(#[from] glob::PatternError),

    #[error("Ignore walk error: {0}")]
    Ignore(#[from] ignore::Error),

    #[error("Async runtime error: {0}")]
    Runtime(#[from] anyhow::Error),
}
