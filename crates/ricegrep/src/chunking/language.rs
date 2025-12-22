use std::{collections::HashMap, fmt, path::Path};

use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};

lazy_static! {
    static ref EXTENSION_MAP: HashMap<&'static str, LanguageKind> = {
        use LanguageKind::*;
        HashMap::from([
            ("rs", Rust),
            ("py", Python),
            ("js", JavaScript),
            ("mjs", JavaScript),
            ("cjs", JavaScript),
            ("ts", TypeScript),
            ("tsx", Tsx),
            ("jsx", Tsx),
            ("java", Java),
            ("go", Go),
            ("c", C),
            ("h", C),
            ("hpp", Cpp),
            ("hh", Cpp),
            ("cc", Cpp),
            ("cpp", Cpp),
            ("cxx", Cpp),
        ])
    };
    static ref SHEBANG_REGEX: Regex = Regex::new(r"^#!.*\b(?P<cmd>python|node|deno)\b").unwrap();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LanguageKind {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Tsx,
    Java,
    Go,
    C,
    Cpp,
    PlainText,
}

impl LanguageKind {
    pub fn default_supported() -> Vec<LanguageKind> {
        vec![
            LanguageKind::Rust,
            LanguageKind::Python,
            LanguageKind::JavaScript,
            LanguageKind::TypeScript,
            LanguageKind::Tsx,
            LanguageKind::Java,
            LanguageKind::Go,
            LanguageKind::C,
            LanguageKind::Cpp,
        ]
    }
}

impl fmt::Display for LanguageKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Default)]
pub struct LanguageDetector;

impl LanguageDetector {
    pub fn detect(&self, path: &Path, contents: &str) -> Option<LanguageKind> {
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            if let Some(language) = EXTENSION_MAP.get(&ext.to_ascii_lowercase()[..]) {
                return Some(*language);
            }
        }

        if let Some(first_line) = contents.lines().next() {
            if let Some(caps) = SHEBANG_REGEX.captures(first_line) {
                return match &caps["cmd"] {
                    "python" => Some(LanguageKind::Python),
                    "node" | "deno" => Some(LanguageKind::JavaScript),
                    _ => None,
                };
            }
        }

        None
    }
}
