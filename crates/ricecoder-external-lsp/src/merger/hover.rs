//! Hover merging

use crate::types::MergeConfig;

/// Merges hover information from external LSP and internal providers
pub struct HoverMerger;

impl HoverMerger {
    /// Create a new hover merger
    pub fn new() -> Self {
        Self
    }

    /// Merge hover information from external LSP and internal provider
    ///
    /// # Arguments
    ///
    /// * `external` - Hover information from external LSP server (if available)
    /// * `internal` - Hover information from internal provider
    /// * `config` - Merge configuration
    ///
    /// # Returns
    ///
    /// Merged hover information (external takes precedence)
    pub fn merge(
        external: Option<String>,
        internal: Option<String>,
        config: &MergeConfig,
    ) -> Option<String> {
        // External hover takes precedence
        if let Some(ext) = external {
            return Some(ext);
        }

        // Fall back to internal if configured
        if config.include_internal {
            return internal;
        }

        None
    }

    /// Combine hover information from multiple sources
    ///
    /// # Arguments
    ///
    /// * `external` - Hover information from external LSP server (if available)
    /// * `internal` - Hover information from internal provider
    ///
    /// # Returns
    ///
    /// Combined hover information with both sources
    pub fn combine(external: Option<String>, internal: Option<String>) -> Option<String> {
        match (external, internal) {
            (Some(ext), Some(int)) => {
                // Combine both sources
                Some(format!("{}\n\n---\n\n{}", ext, int))
            }
            (Some(ext), None) => Some(ext),
            (None, Some(int)) => Some(int),
            (None, None) => None,
        }
    }
}

impl Default for HoverMerger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_external_only() {
        let external = Some("External hover".to_string());
        let internal = Some("Internal hover".to_string());
        let config = MergeConfig::default();

        let result = HoverMerger::merge(external, internal, &config);

        assert_eq!(result, Some("External hover".to_string()));
    }

    #[test]
    fn test_merge_internal_only() {
        let external = None;
        let internal = Some("Internal hover".to_string());
        let config = MergeConfig::default();

        let result = HoverMerger::merge(external, internal, &config);

        assert_eq!(result, Some("Internal hover".to_string()));
    }

    #[test]
    fn test_merge_without_internal() {
        let external = None;
        let internal = Some("Internal hover".to_string());
        let config = MergeConfig {
            include_internal: false,
            deduplicate: true,
        };

        let result = HoverMerger::merge(external, internal, &config);

        assert_eq!(result, None);
    }

    #[test]
    fn test_merge_both_none() {
        let external = None;
        let internal = None;
        let config = MergeConfig::default();

        let result = HoverMerger::merge(external, internal, &config);

        assert_eq!(result, None);
    }

    #[test]
    fn test_combine_both() {
        let external = Some("External hover".to_string());
        let internal = Some("Internal hover".to_string());

        let result = HoverMerger::combine(external, internal);

        assert!(result.is_some());
        let combined = result.unwrap();
        assert!(combined.contains("External hover"));
        assert!(combined.contains("Internal hover"));
        assert!(combined.contains("---"));
    }

    #[test]
    fn test_combine_external_only() {
        let external = Some("External hover".to_string());
        let internal = None;

        let result = HoverMerger::combine(external, internal);

        assert_eq!(result, Some("External hover".to_string()));
    }

    #[test]
    fn test_combine_internal_only() {
        let external = None;
        let internal = Some("Internal hover".to_string());

        let result = HoverMerger::combine(external, internal);

        assert_eq!(result, Some("Internal hover".to_string()));
    }

    #[test]
    fn test_combine_both_none() {
        let external = None;
        let internal = None;

        let result = HoverMerger::combine(external, internal);

        assert_eq!(result, None);
    }
}
