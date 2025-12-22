//! Value objects representing immutable domain concepts

use std::fmt;

use serde::{Deserialize, Serialize};

/// Project identifier - a UUID-based identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectId(uuid::Uuid);

impl ProjectId {
    /// Generate a new random project ID
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }

    /// Create from string representation
    pub fn from_string(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(uuid::Uuid::parse_str(s)?))
    }

    /// Convert to string
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl fmt::Display for ProjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for ProjectId {
    fn default() -> Self {
        Self::new()
    }
}

/// Session identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(uuid::Uuid);

impl SessionId {
    /// Generate a new random session ID
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }

    /// Create from string representation
    pub fn from_string(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(uuid::Uuid::parse_str(s)?))
    }

    /// Convert to string
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

/// File identifier within a project
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FileId(String);

impl FileId {
    /// Create a new file ID from path
    pub fn from_path(path: &str) -> Self {
        Self(path.to_string())
    }

    /// Get the path representation
    pub fn as_path(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for FileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Programming language enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProgrammingLanguage {
    Rust,
    Python,
    TypeScript,
    JavaScript,
    Go,
    Java,
    CSharp,
    C,
    Cpp,
    Swift,
    Kotlin,
    Scala,
    Ruby,
    PHP,
    Other,
}

impl ProgrammingLanguage {
    /// Get file extensions for this language
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            ProgrammingLanguage::Rust => &["rs"],
            ProgrammingLanguage::Python => &["py", "pyw"],
            ProgrammingLanguage::TypeScript => &["ts"],
            ProgrammingLanguage::JavaScript => &["js", "mjs"],
            ProgrammingLanguage::Go => &["go"],
            ProgrammingLanguage::Java => &["java"],
            ProgrammingLanguage::CSharp => &["cs"],
            ProgrammingLanguage::C => &["c", "h"],
            ProgrammingLanguage::Cpp => &["cpp", "cc", "cxx", "hpp", "hxx"],
            ProgrammingLanguage::Swift => &["swift"],
            ProgrammingLanguage::Kotlin => &["kt"],
            ProgrammingLanguage::Scala => &["scala"],
            ProgrammingLanguage::Ruby => &["rb"],
            ProgrammingLanguage::PHP => &["php"],
            ProgrammingLanguage::Other => &[],
        }
    }

    /// Detect language from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        let ext = ext.trim_start_matches('.');
        for lang in [
            ProgrammingLanguage::Rust,
            ProgrammingLanguage::Python,
            ProgrammingLanguage::TypeScript,
            ProgrammingLanguage::JavaScript,
            ProgrammingLanguage::Go,
            ProgrammingLanguage::Java,
            ProgrammingLanguage::CSharp,
            ProgrammingLanguage::C,
            ProgrammingLanguage::Cpp,
            ProgrammingLanguage::Swift,
            ProgrammingLanguage::Kotlin,
            ProgrammingLanguage::Scala,
            ProgrammingLanguage::Ruby,
            ProgrammingLanguage::PHP,
        ] {
            if lang.extensions().contains(&ext) {
                return Some(lang);
            }
        }
        None
    }
}

impl fmt::Display for ProgrammingLanguage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProgrammingLanguage::Rust => write!(f, "Rust"),
            ProgrammingLanguage::Python => write!(f, "Python"),
            ProgrammingLanguage::TypeScript => write!(f, "TypeScript"),
            ProgrammingLanguage::JavaScript => write!(f, "JavaScript"),
            ProgrammingLanguage::Go => write!(f, "Go"),
            ProgrammingLanguage::Java => write!(f, "Java"),
            ProgrammingLanguage::CSharp => write!(f, "C#"),
            ProgrammingLanguage::C => write!(f, "C"),
            ProgrammingLanguage::Cpp => write!(f, "C++"),
            ProgrammingLanguage::Swift => write!(f, "Swift"),
            ProgrammingLanguage::Kotlin => write!(f, "Kotlin"),
            ProgrammingLanguage::Scala => write!(f, "Scala"),
            ProgrammingLanguage::Ruby => write!(f, "Ruby"),
            ProgrammingLanguage::PHP => write!(f, "PHP"),
            ProgrammingLanguage::Other => write!(f, "Other"),
        }
    }
}

/// Semantic version for analysis results
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl SemanticVersion {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return None;
        }

        let major = parts[0].parse().ok()?;
        let minor = parts[1].parse().ok()?;
        let patch = parts[2].parse().ok()?;

        Some(Self {
            major,
            minor,
            patch,
        })
    }
}

impl fmt::Display for SemanticVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl Default for SemanticVersion {
    fn default() -> Self {
        Self::new(0, 1, 0)
    }
}

/// URL wrapper for validated URLs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidUrl(url::Url);

impl ValidUrl {
    pub fn parse(s: &str) -> Result<Self, url::ParseError> {
        Ok(Self(url::Url::parse(s)?))
    }

    pub fn as_url(&self) -> &url::Url {
        &self.0
    }
}

impl fmt::Display for ValidUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// MIME type for file content
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MimeType(String);

impl MimeType {
    pub fn from_path(path: &str) -> Self {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        Self(mime.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MimeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
