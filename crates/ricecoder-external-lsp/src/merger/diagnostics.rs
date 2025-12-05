//! Diagnostics merging

use crate::types::MergeConfig;
use ricecoder_lsp::types::Diagnostic;
use std::collections::HashSet;

/// Merges diagnostics from external LSP and internal providers
pub struct DiagnosticsMerger;

impl DiagnosticsMerger {
    /// Create a new diagnostics merger
    pub fn new() -> Self {
        Self
    }

    /// Merge diagnostics from external LSP and internal provider
    ///
    /// # Arguments
    ///
    /// * `external` - Diagnostics from external LSP server (if available)
    /// * `internal` - Diagnostics from internal provider
    /// * `config` - Merge configuration
    ///
    /// # Returns
    ///
    /// Merged and deduplicated diagnostics
    pub fn merge(
        external: Option<Vec<Diagnostic>>,
        internal: Vec<Diagnostic>,
        config: &MergeConfig,
    ) -> Vec<Diagnostic> {
        let mut result = Vec::new();

        // External diagnostics are authoritative - use them if available
        if let Some(ext) = external {
            result.extend(ext);
        } else if config.include_internal {
            // Only use internal diagnostics if no external available
            result.extend(internal);
        }

        // Deduplicate if configured
        if config.deduplicate {
            result = Self::deduplicate(result);
        }

        result
    }

    /// Deduplicate diagnostics by range and message
    fn deduplicate(diagnostics: Vec<Diagnostic>) -> Vec<Diagnostic> {
        let mut seen = HashSet::new();
        let mut result = Vec::new();

        for diag in diagnostics {
            // Create a key from range and message
            let key = format!(
                "{}:{}:{}:{}:{}",
                diag.range.start.line,
                diag.range.start.character,
                diag.range.end.line,
                diag.range.end.character,
                diag.message
            );

            if !seen.contains(&key) {
                seen.insert(key);
                result.push(diag);
            }
        }

        result
    }
}

impl Default for DiagnosticsMerger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ricecoder_lsp::types::{DiagnosticSeverity, Position, Range};

    fn create_diagnostic(line: u32, message: &str) -> Diagnostic {
        Diagnostic::new(
            Range::new(Position::new(line, 0), Position::new(line, 5)),
            DiagnosticSeverity::Error,
            message.to_string(),
        )
    }

    #[test]
    fn test_merge_external_only() {
        let external = vec![
            create_diagnostic(0, "error 1"),
            create_diagnostic(1, "error 2"),
        ];
        let internal = vec![];
        let config = MergeConfig::default();

        let result = DiagnosticsMerger::merge(Some(external), internal, &config);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].message, "error 1");
        assert_eq!(result[1].message, "error 2");
    }

    #[test]
    fn test_merge_internal_only() {
        let external = None;
        let internal = vec![
            create_diagnostic(0, "warning 1"),
            create_diagnostic(1, "warning 2"),
        ];
        let config = MergeConfig::default();

        let result = DiagnosticsMerger::merge(external, internal, &config);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].message, "warning 1");
        assert_eq!(result[1].message, "warning 2");
    }

    #[test]
    fn test_merge_external_ignores_internal() {
        let external = vec![create_diagnostic(0, "external error")];
        let internal = vec![create_diagnostic(1, "internal warning")];
        let config = MergeConfig::default();

        let result = DiagnosticsMerger::merge(Some(external), internal, &config);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].message, "external error");
    }

    #[test]
    fn test_merge_without_internal() {
        let external = None;
        let internal = vec![create_diagnostic(0, "warning")];
        let config = MergeConfig {
            include_internal: false,
            deduplicate: true,
        };

        let result = DiagnosticsMerger::merge(external, internal, &config);

        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_merge_with_deduplication() {
        let external = vec![
            create_diagnostic(0, "error 1"),
            create_diagnostic(0, "error 1"), // Duplicate
        ];
        let internal = vec![];
        let config = MergeConfig {
            include_internal: true,
            deduplicate: true,
        };

        let result = DiagnosticsMerger::merge(Some(external), internal, &config);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].message, "error 1");
    }

    #[test]
    fn test_merge_without_deduplication() {
        let external = vec![
            create_diagnostic(0, "error 1"),
            create_diagnostic(0, "error 1"), // Duplicate
        ];
        let internal = vec![];
        let config = MergeConfig {
            include_internal: true,
            deduplicate: false,
        };

        let result = DiagnosticsMerger::merge(Some(external), internal, &config);

        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_merge_empty_external() {
        let external = Some(vec![]);
        let internal = vec![create_diagnostic(0, "warning")];
        let config = MergeConfig::default();

        let result = DiagnosticsMerger::merge(external, internal, &config);

        assert_eq!(result.len(), 0);
    }
}
