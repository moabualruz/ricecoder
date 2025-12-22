use crate::nlq::models::QueryFilter;

/// Extracts inline filters (e.g., `language:rust`, `repo=42`) from tokenized queries.
pub struct FilterExtractor {
    supported_keys: Vec<String>,
}

impl FilterExtractor {
    pub fn new() -> Self {
        Self {
            supported_keys: vec![
                "language".into(),
                "lang".into(),
                "repo".into(),
                "repo_id".into(),
                "repository".into(),
                "file".into(),
                "path".into(),
            ],
        }
    }

    pub fn extract(&self, tokens: &[String]) -> Vec<QueryFilter> {
        tokens
            .iter()
            .filter_map(|token| self.build_filter(token))
            .collect()
    }

    fn build_filter(&self, token: &str) -> Option<QueryFilter> {
        let normalized = token
            .trim()
            .trim_end_matches(|c: char| c == ',' || c == '.');
        if let Some((key, value)) = Self::split_filter(normalized, ':') {
            return self.selected_filter(key, value);
        }
        if let Some((key, value)) = Self::split_filter(normalized, '=') {
            return self.selected_filter(key, value);
        }
        None
    }

    fn selected_filter(&self, key: String, value: String) -> Option<QueryFilter> {
        let key_lower = key.to_lowercase();
        if self.supported_keys.contains(&key_lower) && !value.is_empty() {
            Some(QueryFilter {
                field: key_lower,
                value,
            })
        } else {
            None
        }
    }

    fn split_filter(token: &str, separator: char) -> Option<(String, String)> {
        let parts: Vec<&str> = token.splitn(2, separator).collect();
        if parts.len() != 2 {
            return None;
        }
        let key = parts[0].trim().to_string();
        let value = parts[1]
            .trim()
            .trim_matches(|c| c == '"' || c == '\'')
            .trim()
            .to_string();
        Some((key, value))
    }
}
