//! Completion merging

use crate::types::MergeConfig;
use ricecoder_completion::types::CompletionItem;
use std::collections::HashSet;

/// Merges completions from external LSP and internal providers
pub struct CompletionMerger;

impl CompletionMerger {
    /// Create a new completion merger
    pub fn new() -> Self {
        Self
    }

    /// Merge completions from external LSP and internal provider
    ///
    /// # Arguments
    ///
    /// * `external` - Completions from external LSP server (if available)
    /// * `internal` - Completions from internal provider
    /// * `config` - Merge configuration
    ///
    /// # Returns
    ///
    /// Merged and deduplicated completion items
    pub fn merge(
        external: Option<Vec<CompletionItem>>,
        internal: Vec<CompletionItem>,
        config: &MergeConfig,
    ) -> Vec<CompletionItem> {
        let mut result = Vec::new();
        let mut seen_labels = HashSet::new();

        // Add external completions first (higher priority)
        if let Some(ext) = external {
            for item in ext {
                if config.deduplicate {
                    if !seen_labels.contains(&item.label) {
                        seen_labels.insert(item.label.clone());
                        result.push(item);
                    }
                } else {
                    result.push(item);
                }
            }
        }

        // Add internal completions if configured
        if config.include_internal {
            for item in internal {
                if config.deduplicate {
                    if !seen_labels.contains(&item.label) {
                        seen_labels.insert(item.label.clone());
                        result.push(item);
                    }
                } else {
                    result.push(item);
                }
            }
        }

        // Sort by score (descending)
        result.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        result
    }
}

impl Default for CompletionMerger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_completion::types::CompletionItemKind;

    fn create_completion(label: &str, score: f32) -> CompletionItem {
        CompletionItem::new(
            label.to_string(),
            CompletionItemKind::Variable,
            label.to_string(),
        )
        .with_score(score)
    }

    #[test]
    fn test_merge_external_only() {
        let external = vec![
            create_completion("foo", 0.9),
            create_completion("bar", 0.8),
        ];
        let internal = vec![];
        let config = MergeConfig::default();

        let result = CompletionMerger::merge(Some(external), internal, &config);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].label, "foo");
        assert_eq!(result[1].label, "bar");
    }

    #[test]
    fn test_merge_internal_only() {
        let external = None;
        let internal = vec![
            create_completion("baz", 0.7),
            create_completion("qux", 0.6),
        ];
        let config = MergeConfig::default();

        let result = CompletionMerger::merge(external, internal, &config);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].label, "baz");
        assert_eq!(result[1].label, "qux");
    }

    #[test]
    fn test_merge_both_with_deduplication() {
        let external = vec![
            create_completion("foo", 0.9),
            create_completion("bar", 0.8),
        ];
        let internal = vec![
            create_completion("foo", 0.5), // Duplicate, should be skipped
            create_completion("baz", 0.7),
        ];
        let config = MergeConfig {
            include_internal: true,
            deduplicate: true,
        };

        let result = CompletionMerger::merge(Some(external), internal, &config);

        assert_eq!(result.len(), 3);
        // Results should be sorted by score (descending)
        assert_eq!(result[0].label, "foo"); // 0.9
        assert_eq!(result[1].label, "bar"); // 0.8
        assert_eq!(result[2].label, "baz"); // 0.7
    }

    #[test]
    fn test_merge_both_without_deduplication() {
        let external = vec![create_completion("foo", 0.9)];
        let internal = vec![create_completion("foo", 0.5)];
        let config = MergeConfig {
            include_internal: true,
            deduplicate: false,
        };

        let result = CompletionMerger::merge(Some(external), internal, &config);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].label, "foo");
        assert_eq!(result[1].label, "foo");
    }

    #[test]
    fn test_merge_without_internal() {
        let external = vec![create_completion("foo", 0.9)];
        let internal = vec![create_completion("bar", 0.8)];
        let config = MergeConfig {
            include_internal: false,
            deduplicate: true,
        };

        let result = CompletionMerger::merge(Some(external), internal, &config);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].label, "foo");
    }

    #[test]
    fn test_merge_sorting_by_score() {
        let external = vec![
            create_completion("low", 0.3),
            create_completion("high", 0.9),
            create_completion("mid", 0.6),
        ];
        let internal = vec![];
        let config = MergeConfig::default();

        let result = CompletionMerger::merge(Some(external), internal, &config);

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].label, "high");
        assert_eq!(result[1].label, "mid");
        assert_eq!(result[2].label, "low");
    }

    #[test]
    fn test_merge_empty_external() {
        let external = Some(vec![]);
        let internal = vec![create_completion("foo", 0.8)];
        let config = MergeConfig::default();

        let result = CompletionMerger::merge(external, internal, &config);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].label, "foo");
    }
}
