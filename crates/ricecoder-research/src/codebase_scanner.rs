//! Codebase scanning and file discovery

use crate::error::ResearchError;
use crate::models::{Framework, Language};
use ignore::WalkBuilder;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// File metadata extracted during scanning
#[derive(Debug, Clone)]
pub struct FileMetadata {
    /// Path to the file
    pub path: PathBuf,
    /// File language
    pub language: Option<Language>,
    /// File size in bytes
    pub size: u64,
    /// Whether the file is a test file
    pub is_test: bool,
}

/// Result of codebase scanning
#[derive(Debug, Clone)]
pub struct ScanResult {
    /// All files found in the codebase
    pub files: Vec<FileMetadata>,
    /// Languages detected in the codebase
    pub languages: Vec<Language>,
    /// Frameworks detected in the codebase
    pub frameworks: Vec<Framework>,
    /// Source directories
    pub source_dirs: Vec<PathBuf>,
    /// Test directories
    pub test_dirs: Vec<PathBuf>,
}

/// Scans a codebase to discover files and extract metadata
pub struct CodebaseScanner;

impl CodebaseScanner {
    /// Scan a project directory and extract file metadata
    ///
    /// # Arguments
    /// * `root` - Root directory of the project
    ///
    /// # Returns
    /// A `ScanResult` containing all discovered files and metadata
    pub fn scan(root: &Path) -> Result<ScanResult, ResearchError> {
        if !root.exists() {
            return Err(ResearchError::ProjectNotFound {
                path: root.to_path_buf(),
                reason: "Cannot scan codebase: root directory does not exist".to_string(),
            });
        }

        let mut files = Vec::new();
        let mut languages = HashSet::new();
        let mut source_dirs = HashSet::new();
        let mut test_dirs = HashSet::new();

        // Use ignore crate to respect .gitignore
        let walker = WalkBuilder::new(root).hidden(true).git_ignore(true).build();

        for entry in walker {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let path = entry.path();

            // Skip directories
            if path.is_dir() {
                continue;
            }

            // Extract language from file extension
            let language = Self::detect_language(path);
            let is_test = Self::is_test_file(path);

            // Track source and test directories
            if let Some(parent) = path.parent() {
                if is_test {
                    test_dirs.insert(parent.to_path_buf());
                } else if language.is_some() {
                    source_dirs.insert(parent.to_path_buf());
                }
            }

            if let Ok(metadata) = std::fs::metadata(path) {
                files.push(FileMetadata {
                    path: path.to_path_buf(),
                    language: language.clone(),
                    size: metadata.len(),
                    is_test,
                });

                if let Some(lang) = language {
                    languages.insert(lang);
                }
            }
        }

        // Convert HashSets to Vecs
        let mut languages_vec: Vec<Language> = languages.into_iter().collect();
        languages_vec.sort_by(|a, b| format!("{:?}", a).cmp(&format!("{:?}", b)));

        let mut source_dirs_vec: Vec<PathBuf> = source_dirs.into_iter().collect();
        source_dirs_vec.sort();

        let mut test_dirs_vec: Vec<PathBuf> = test_dirs.into_iter().collect();
        test_dirs_vec.sort();

        Ok(ScanResult {
            files,
            languages: languages_vec,
            frameworks: Vec::new(), // Will be populated by other components
            source_dirs: source_dirs_vec,
            test_dirs: test_dirs_vec,
        })
    }

    /// Detect the language of a file based on its extension
    fn detect_language(path: &Path) -> Option<Language> {
        let extension = path.extension()?.to_str()?;

        match extension {
            "rs" => Some(Language::Rust),
            "ts" | "tsx" | "js" | "jsx" => Some(Language::TypeScript),
            "py" => Some(Language::Python),
            "go" => Some(Language::Go),
            "java" => Some(Language::Java),
            "kt" | "kts" => Some(Language::Kotlin),
            "cs" => Some(Language::CSharp),
            "php" => Some(Language::Php),
            "rb" => Some(Language::Ruby),
            "swift" => Some(Language::Swift),
            "dart" => Some(Language::Dart),
            _ => None,
        }
    }

    /// Check if a file is a test file
    fn is_test_file(path: &Path) -> bool {
        // Check for common test directory patterns
        for component in path.components() {
            if let std::path::Component::Normal(name) = component {
                let name_str = name.to_string_lossy();
                if name_str == "tests" || name_str == "test" || name_str == "__tests__" {
                    return true;
                }
            }
        }

        // Check for test file naming patterns
        let file_name = path.file_name().unwrap_or_default().to_string_lossy();
        file_name.ends_with("_test.rs")
            || file_name.ends_with(".test.ts")
            || file_name.ends_with(".test.js")
            || file_name.ends_with("_test.py")
            || file_name.ends_with("_test.go")
            || file_name.ends_with("Test.java")
            || file_name.ends_with("Test.kt")
            || file_name.ends_with("Tests.cs")
            || file_name.ends_with("_test.rb")
            || file_name.ends_with("Tests.swift")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_detect_language_rust() {
        let path = PathBuf::from("main.rs");
        assert_eq!(
            CodebaseScanner::detect_language(&path),
            Some(Language::Rust)
        );
    }

    #[test]
    fn test_detect_language_typescript() {
        let path = PathBuf::from("main.ts");
        assert_eq!(
            CodebaseScanner::detect_language(&path),
            Some(Language::TypeScript)
        );
    }

    #[test]
    fn test_detect_language_python() {
        let path = PathBuf::from("main.py");
        assert_eq!(
            CodebaseScanner::detect_language(&path),
            Some(Language::Python)
        );
    }

    #[test]
    fn test_detect_language_unknown() {
        let path = PathBuf::from("README.md");
        assert_eq!(CodebaseScanner::detect_language(&path), None);
    }

    #[test]
    fn test_is_test_file_rust() {
        let path = PathBuf::from("src/lib_test.rs");
        assert!(CodebaseScanner::is_test_file(&path));
    }

    #[test]
    fn test_is_test_file_typescript() {
        let path = PathBuf::from("src/main.test.ts");
        assert!(CodebaseScanner::is_test_file(&path));
    }

    #[test]
    fn test_is_test_file_directory() {
        let path = PathBuf::from("tests/integration.rs");
        assert!(CodebaseScanner::is_test_file(&path));
    }

    #[test]
    fn test_is_test_file_not_test() {
        let path = PathBuf::from("src/main.rs");
        assert!(!CodebaseScanner::is_test_file(&path));
    }

    #[test]
    fn test_scan_simple_project() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        // Create some test files
        fs::create_dir_all(root.join("src"))?;
        fs::create_dir_all(root.join("tests"))?;
        fs::write(root.join("src/main.rs"), "fn main() {}")?;
        fs::write(root.join("src/lib.rs"), "pub fn lib() {}")?;
        fs::write(
            root.join("tests/integration_test.rs"),
            "#[test]\nfn test() {}",
        )?;

        let result = CodebaseScanner::scan(root)?;

        assert_eq!(result.files.len(), 3);
        assert!(result.languages.contains(&Language::Rust));
        assert!(!result.source_dirs.is_empty());
        assert!(!result.test_dirs.is_empty());

        Ok(())
    }

    #[test]
    fn test_scan_nonexistent_directory() {
        let result = CodebaseScanner::scan(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }
}
