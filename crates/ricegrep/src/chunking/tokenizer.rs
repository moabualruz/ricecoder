use lazy_static::lazy_static;
use regex::Regex;
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;

lazy_static! {
    static ref NON_WORD_REGEX: Regex = Regex::new(r"[^\p{L}\p{N}_]+").unwrap();
    static ref CAMEL_REGEX: Regex = Regex::new(r"([a-z0-9])([A-Z])").unwrap();
}

pub struct IdentifierSplitResult {
    pub identifiers: Vec<String>,
    pub identifier_tokens: Vec<String>,
}

pub fn extract_identifiers(source: &str) -> IdentifierSplitResult {
    let mut identifiers = Vec::new();
    let mut identifier_tokens = Vec::new();

    for word in NON_WORD_REGEX.replace_all(source, " ").split_whitespace() {
        let normalized = CAMEL_REGEX
            .replace_all(word, "$1 $2")
            .to_string()
            .to_lowercase()
            .nfc()
            .collect::<String>();
        identifiers.push(word.to_string());
        identifier_tokens.extend(
            normalized
                .split_whitespace()
                .map(|t| t.to_string())
                .filter(|t| !t.is_empty()),
        );
    }

    IdentifierSplitResult {
        identifiers,
        identifier_tokens,
    }
}

pub fn count_tokens(source: &str) -> usize {
    source
        .unicode_words()
        .filter(|w| !w.trim().is_empty())
        .count()
}
