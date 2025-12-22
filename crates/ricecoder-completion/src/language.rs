use std::path::Path;

/// Language detection and identification utilities
///
/// This module provides language detection capabilities for code completion,
/// including file extension detection, content-based detection, and language
/// identification for supported programming languages.
use serde::{Deserialize, Serialize};

/// Supported programming languages for code completion
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    /// Rust programming language
    Rust,
    /// TypeScript/JavaScript programming language
    TypeScript,
    /// Python programming language
    Python,
    /// Go programming language
    Go,
    /// Java programming language
    Java,
    /// Kotlin programming language
    Kotlin,
    /// Dart programming language
    Dart,
    /// Unknown or unsupported language
    Unknown,
}

impl Language {
    /// Detect language from file extension
    ///
    /// # Arguments
    ///
    /// * `ext` - File extension (without the dot)
    ///
    /// # Returns
    ///
    /// The detected language, or `Language::Unknown` if not recognized
    ///
    /// # Example
    ///
    /// ```ignore
    /// assert_eq!(Language::from_extension("rs"), Language::Rust);
    /// assert_eq!(Language::from_extension("ts"), Language::TypeScript);
    /// assert_eq!(Language::from_extension("py"), Language::Python);
    /// ```
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "rs" => Language::Rust,
            "ts" | "tsx" | "js" | "jsx" => Language::TypeScript,
            "py" => Language::Python,
            "go" => Language::Go,
            "java" => Language::Java,
            "kt" | "kts" => Language::Kotlin,
            "dart" => Language::Dart,
            _ => Language::Unknown,
        }
    }

    /// Get file extensions for this language
    ///
    /// # Returns
    ///
    /// A slice of file extensions (without dots) for this language
    ///
    /// # Example
    ///
    /// ```ignore
    /// assert_eq!(Language::Rust.extensions(), &["rs"]);
    /// assert_eq!(Language::TypeScript.extensions(), &["ts", "tsx", "js", "jsx"]);
    /// ```
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            Language::Rust => &["rs"],
            Language::TypeScript => &["ts", "tsx", "js", "jsx"],
            Language::Python => &["py"],
            Language::Go => &["go"],
            Language::Java => &["java"],
            Language::Kotlin => &["kt", "kts"],
            Language::Dart => &["dart"],
            Language::Unknown => &[],
        }
    }

    /// Convert language to string identifier
    ///
    /// # Returns
    ///
    /// A string identifier for this language
    ///
    /// # Example
    ///
    /// ```ignore
    /// assert_eq!(Language::Rust.as_str(), "rust");
    /// assert_eq!(Language::TypeScript.as_str(), "typescript");
    /// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            Language::Rust => "rust",
            Language::TypeScript => "typescript",
            Language::Python => "python",
            Language::Go => "go",
            Language::Java => "java",
            Language::Kotlin => "kotlin",
            Language::Dart => "dart",
            Language::Unknown => "unknown",
        }
    }
}

/// Language detection utilities
///
/// Provides methods for detecting programming languages from file paths,
/// file content, or both.
pub struct LanguageDetector;

impl LanguageDetector {
    /// Detect language from file extension
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file
    ///
    /// # Returns
    ///
    /// The detected language, or `Language::Unknown` if extension is not recognized
    ///
    /// # Example
    ///
    /// ```ignore
    /// let lang = LanguageDetector::from_extension(Path::new("main.rs"));
    /// assert_eq!(lang, Language::Rust);
    /// ```
    pub fn from_extension(path: &Path) -> Language {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(Language::from_extension)
            .unwrap_or(Language::Unknown)
    }

    /// Detect language from file content (shebang or imports)
    ///
    /// # Arguments
    ///
    /// * `content` - The file content to analyze
    ///
    /// # Returns
    ///
    /// The detected language based on content patterns, or `Language::Unknown`
    ///
    /// # Example
    ///
    /// ```ignore
    /// let lang = LanguageDetector::from_content("#!/usr/bin/env python\nprint('hello')");
    /// assert_eq!(lang, Language::Python);
    /// ```
    pub fn from_content(content: &str) -> Language {
        // Check for shebang
        if let Some(first_line) = content.lines().next() {
            if first_line.starts_with("#!") {
                if first_line.contains("python") {
                    return Language::Python;
                } else if first_line.contains("node") || first_line.contains("ts-node") {
                    return Language::TypeScript;
                } else if first_line.contains("ruby") {
                    return Language::Unknown; // Ruby not supported yet
                } else if first_line.contains("bash") || first_line.contains("sh") {
                    return Language::Unknown; // Shell not supported yet
                }
            }
        }

        // Check for language-specific patterns (order matters - most specific first)

        // Check Go (package + func is very specific to Go)
        if content.contains("package ") && content.contains("func ") {
            return Language::Go;
        }

        // Check Java (public class/interface is very specific to Java)
        if content.contains("public class ") || content.contains("public interface ") {
            return Language::Java;
        }

        // Check Dart (void main() is very specific to Dart)
        if content.contains("void main()") {
            return Language::Dart;
        }

        // Check Python (def is Python-specific)
        if content.contains("def ") {
            return Language::Python;
        }

        // Check Rust (use is very specific to Rust)
        if content.contains("use ") {
            return Language::Rust;
        }

        // Check TypeScript/JavaScript (export is more specific than import)
        if content.contains("export ") {
            return Language::TypeScript;
        }

        // Check for import statements (generic, but TypeScript/JS specific in context)
        if content.contains("import ") {
            return Language::TypeScript;
        }

        // Check Kotlin (fun is Kotlin-specific when combined with class/object/interface)
        if content.contains("fun ")
            && (content.contains("class ")
                || content.contains("object ")
                || content.contains("interface "))
        {
            return Language::Kotlin;
        }

        // Fallback: if we see fn, assume Rust (fn is Rust-specific)
        if content.contains("fn ") {
            return Language::Rust;
        }

        // Fallback: if we see fun, assume Kotlin (fun is Kotlin-specific)
        if content.contains("fun ") {
            return Language::Kotlin;
        }

        Language::Unknown
    }

    /// Detect language from both extension and content
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file
    /// * `content` - The file content to analyze
    ///
    /// # Returns
    ///
    /// The detected language, preferring extension detection over content detection
    ///
    /// # Example
    ///
    /// ```ignore
    /// let lang = LanguageDetector::detect(
    ///     Path::new("main.rs"),
    ///     "fn main() {}"
    /// );
    /// assert_eq!(lang, Language::Rust);
    /// ```
    pub fn detect(path: &Path, content: &str) -> Language {
        let from_ext = Self::from_extension(path);
        if from_ext != Language::Unknown {
            return from_ext;
        }
        Self::from_content(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_from_extension() {
        assert_eq!(Language::from_extension("rs"), Language::Rust);
        assert_eq!(Language::from_extension("ts"), Language::TypeScript);
        assert_eq!(Language::from_extension("tsx"), Language::TypeScript);
        assert_eq!(Language::from_extension("js"), Language::TypeScript);
        assert_eq!(Language::from_extension("jsx"), Language::TypeScript);
        assert_eq!(Language::from_extension("py"), Language::Python);
        assert_eq!(Language::from_extension("go"), Language::Go);
        assert_eq!(Language::from_extension("java"), Language::Java);
        assert_eq!(Language::from_extension("kt"), Language::Kotlin);
        assert_eq!(Language::from_extension("kts"), Language::Kotlin);
        assert_eq!(Language::from_extension("dart"), Language::Dart);
        assert_eq!(Language::from_extension("unknown"), Language::Unknown);
    }

    #[test]
    fn test_language_extensions() {
        assert_eq!(Language::Rust.extensions(), &["rs"]);
        assert_eq!(
            Language::TypeScript.extensions(),
            &["ts", "tsx", "js", "jsx"]
        );
        assert_eq!(Language::Python.extensions(), &["py"]);
        assert_eq!(Language::Go.extensions(), &["go"]);
        assert_eq!(Language::Java.extensions(), &["java"]);
        assert_eq!(Language::Kotlin.extensions(), &["kt", "kts"]);
        assert_eq!(Language::Dart.extensions(), &["dart"]);
        assert_eq!(Language::Unknown.extensions(), &[] as &[&str]);
    }

    #[test]
    fn test_language_as_str() {
        assert_eq!(Language::Rust.as_str(), "rust");
        assert_eq!(Language::TypeScript.as_str(), "typescript");
        assert_eq!(Language::Python.as_str(), "python");
        assert_eq!(Language::Go.as_str(), "go");
        assert_eq!(Language::Java.as_str(), "java");
        assert_eq!(Language::Kotlin.as_str(), "kotlin");
        assert_eq!(Language::Dart.as_str(), "dart");
        assert_eq!(Language::Unknown.as_str(), "unknown");
    }

    #[test]
    fn test_language_detector_from_extension() {
        assert_eq!(
            LanguageDetector::from_extension(Path::new("test.rs")),
            Language::Rust
        );
        assert_eq!(
            LanguageDetector::from_extension(Path::new("test.ts")),
            Language::TypeScript
        );
        assert_eq!(
            LanguageDetector::from_extension(Path::new("test.py")),
            Language::Python
        );
        assert_eq!(
            LanguageDetector::from_extension(Path::new("test.go")),
            Language::Go
        );
        assert_eq!(
            LanguageDetector::from_extension(Path::new("test.java")),
            Language::Java
        );
        assert_eq!(
            LanguageDetector::from_extension(Path::new("test.kt")),
            Language::Kotlin
        );
        assert_eq!(
            LanguageDetector::from_extension(Path::new("test.dart")),
            Language::Dart
        );
        assert_eq!(
            LanguageDetector::from_extension(Path::new("test.unknown")),
            Language::Unknown
        );
    }

    #[test]
    fn test_language_detector_from_content_shebang() {
        let python_shebang = "#!/usr/bin/env python\nprint('hello')";
        assert_eq!(
            LanguageDetector::from_content(python_shebang),
            Language::Python
        );

        let node_shebang = "#!/usr/bin/env node\nconsole.log('hello')";
        assert_eq!(
            LanguageDetector::from_content(node_shebang),
            Language::TypeScript
        );
    }

    #[test]
    fn test_language_detector_from_content_patterns() {
        let rust_code = "use std::io;\nfn main() {}";
        assert_eq!(LanguageDetector::from_content(rust_code), Language::Rust);

        let go_code = "package main\nfunc main() {}";
        assert_eq!(LanguageDetector::from_content(go_code), Language::Go);

        let java_code = "public class Main {}";
        assert_eq!(LanguageDetector::from_content(java_code), Language::Java);

        let kotlin_code = "fun main() {}";
        assert_eq!(
            LanguageDetector::from_content(kotlin_code),
            Language::Kotlin
        );

        let dart_code = "void main() {}";
        assert_eq!(LanguageDetector::from_content(dart_code), Language::Dart);

        let ts_code = "import { foo } from 'bar';\nexport const x = 1;";
        assert_eq!(
            LanguageDetector::from_content(ts_code),
            Language::TypeScript
        );

        let py_code = "import os\ndef hello():\n    pass";
        assert_eq!(LanguageDetector::from_content(py_code), Language::Python);
    }

    #[test]
    fn test_language_detector_combined() {
        let path = Path::new("test.rs");
        let content = "fn main() {}";
        assert_eq!(LanguageDetector::detect(path, content), Language::Rust);

        // Test fallback to content detection
        let path = Path::new("test.unknown");
        let content = "fn main() {}";
        assert_eq!(LanguageDetector::detect(path, content), Language::Rust);
    }
}
