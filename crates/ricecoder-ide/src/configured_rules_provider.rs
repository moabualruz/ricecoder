//! Configured rules provider implementation
//!
//! This module implements the IdeProvider trait for custom IDE rules loaded
//! from YAML/JSON configuration files.

use crate::error::{IdeError, IdeResult};
use crate::provider::IdeProvider;
use crate::types::*;
use async_trait::async_trait;
use tracing::debug;

/// Rule for IDE completion
#[derive(Debug, Clone)]
pub struct CompletionRule {
    /// Pattern to match
    pub pattern: String,
    /// Completion items to suggest
    pub suggestions: Vec<CompletionItem>,
}

/// Rule for IDE diagnostics
#[derive(Debug, Clone)]
pub struct DiagnosticsRule {
    /// Pattern to match
    pub pattern: String,
    /// Diagnostics to report
    pub diagnostics: Vec<Diagnostic>,
}

/// Rule for IDE hover
#[derive(Debug, Clone)]
pub struct HoverRule {
    /// Pattern to match
    pub pattern: String,
    /// Hover information
    pub hover: Hover,
}

/// Rule for IDE definition
#[derive(Debug, Clone)]
pub struct DefinitionRule {
    /// Pattern to match
    pub pattern: String,
    /// Definition location
    pub location: Location,
}

/// Configured rules provider
pub struct ConfiguredRulesProvider {
    /// Language this provider supports
    language: String,
    /// Completion rules
    completion_rules: Vec<CompletionRule>,
    /// Diagnostics rules
    diagnostics_rules: Vec<DiagnosticsRule>,
    /// Hover rules
    hover_rules: Vec<HoverRule>,
    /// Definition rules
    definition_rules: Vec<DefinitionRule>,
}

impl ConfiguredRulesProvider {
    /// Create a new configured rules provider
    pub fn new(language: String) -> Self {
        ConfiguredRulesProvider {
            language,
            completion_rules: Vec::new(),
            diagnostics_rules: Vec::new(),
            hover_rules: Vec::new(),
            definition_rules: Vec::new(),
        }
    }

    /// Add a completion rule
    pub fn add_completion_rule(&mut self, rule: CompletionRule) {
        self.completion_rules.push(rule);
    }

    /// Add a diagnostics rule
    pub fn add_diagnostics_rule(&mut self, rule: DiagnosticsRule) {
        self.diagnostics_rules.push(rule);
    }

    /// Add a hover rule
    pub fn add_hover_rule(&mut self, rule: HoverRule) {
        self.hover_rules.push(rule);
    }

    /// Add a definition rule
    pub fn add_definition_rule(&mut self, rule: DefinitionRule) {
        self.definition_rules.push(rule);
    }

    /// Load rules from YAML configuration
    pub async fn load_from_yaml(language: String, yaml_content: &str) -> IdeResult<Self> {
        debug!("Loading configured rules from YAML for language: {}", language);

        let mut provider = ConfiguredRulesProvider::new(language);

        // Parse YAML
        let config: serde_yaml::Value = serde_yaml::from_str(yaml_content).map_err(|e| {
            IdeError::config_error(format!("Failed to parse YAML rules: {}", e))
        })?;

        // Load completion rules
        if let Some(completions) = config.get("completions").and_then(|v| v.as_sequence()) {
            for completion in completions {
                if let Ok(rule) = Self::parse_completion_rule(completion) {
                    provider.add_completion_rule(rule);
                }
            }
        }

        // Load diagnostics rules
        if let Some(diagnostics) = config.get("diagnostics").and_then(|v| v.as_sequence()) {
            for diagnostic in diagnostics {
                if let Ok(rule) = Self::parse_diagnostics_rule(diagnostic) {
                    provider.add_diagnostics_rule(rule);
                }
            }
        }

        // Load hover rules
        if let Some(hovers) = config.get("hovers").and_then(|v| v.as_sequence()) {
            for hover in hovers {
                if let Ok(rule) = Self::parse_hover_rule(hover) {
                    provider.add_hover_rule(rule);
                }
            }
        }

        // Load definition rules
        if let Some(definitions) = config.get("definitions").and_then(|v| v.as_sequence()) {
            for definition in definitions {
                if let Ok(rule) = Self::parse_definition_rule(definition) {
                    provider.add_definition_rule(rule);
                }
            }
        }

        Ok(provider)
    }

    /// Load rules from JSON configuration
    pub async fn load_from_json(language: String, json_content: &str) -> IdeResult<Self> {
        debug!("Loading configured rules from JSON for language: {}", language);

        let mut provider = ConfiguredRulesProvider::new(language);

        // Parse JSON
        let config: serde_json::Value = serde_json::from_str(json_content).map_err(|e| {
            IdeError::config_error(format!("Failed to parse JSON rules: {}", e))
        })?;

        // Convert to YAML value for uniform processing
        let yaml_config: serde_yaml::Value = serde_yaml::from_str(&serde_json::to_string(&config).unwrap()).map_err(|e| {
            IdeError::config_error(format!("Failed to convert JSON to YAML: {}", e))
        })?;

        // Load completion rules
        if let Some(completions) = yaml_config.get("completions").and_then(|v| v.as_sequence()) {
            for completion in completions {
                if let Ok(rule) = Self::parse_completion_rule(completion) {
                    provider.add_completion_rule(rule);
                }
            }
        }

        // Load diagnostics rules
        if let Some(diagnostics) = yaml_config.get("diagnostics").and_then(|v| v.as_sequence()) {
            for diagnostic in diagnostics {
                if let Ok(rule) = Self::parse_diagnostics_rule(diagnostic) {
                    provider.add_diagnostics_rule(rule);
                }
            }
        }

        // Load hover rules
        if let Some(hovers) = yaml_config.get("hovers").and_then(|v| v.as_sequence()) {
            for hover in hovers {
                if let Ok(rule) = Self::parse_hover_rule(hover) {
                    provider.add_hover_rule(rule);
                }
            }
        }

        // Load definition rules
        if let Some(definitions) = yaml_config.get("definitions").and_then(|v| v.as_sequence()) {
            for definition in definitions {
                if let Ok(rule) = Self::parse_definition_rule(definition) {
                    provider.add_definition_rule(rule);
                }
            }
        }

        Ok(provider)
    }

    /// Parse a completion rule from YAML/JSON
    fn parse_completion_rule(value: &serde_yaml::Value) -> IdeResult<CompletionRule> {
        let pattern = value
            .get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| IdeError::config_error("Completion rule missing 'pattern' field"))?
            .to_string();

        let suggestions = value
            .get("suggestions")
            .and_then(|v| v.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter_map(|item| {
                        let label = item.get("label")?.as_str()?.to_string();
                        let kind = item
                            .get("kind")
                            .and_then(|k| k.as_str())
                            .and_then(|k| Self::parse_completion_kind(k))
                            .unwrap_or(CompletionItemKind::Text);
                        let detail = item.get("detail").and_then(|d| d.as_str()).map(|s| s.to_string());
                        let documentation = item
                            .get("documentation")
                            .and_then(|d| d.as_str())
                            .map(|s| s.to_string());
                        let insert_text = item
                            .get("insertText")
                            .and_then(|t| t.as_str())
                            .unwrap_or(&label)
                            .to_string();

                        Some(CompletionItem {
                            label,
                            kind,
                            detail,
                            documentation,
                            insert_text,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(CompletionRule { pattern, suggestions })
    }

    /// Parse a diagnostics rule from YAML/JSON
    fn parse_diagnostics_rule(value: &serde_yaml::Value) -> IdeResult<DiagnosticsRule> {
        let pattern = value
            .get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| IdeError::config_error("Diagnostics rule missing 'pattern' field"))?
            .to_string();

        let diagnostics = value
            .get("diagnostics")
            .and_then(|v| v.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter_map(|item| {
                        let message = item.get("message")?.as_str()?.to_string();
                        let severity = item
                            .get("severity")
                            .and_then(|s| s.as_str())
                            .and_then(|s| Self::parse_severity(s))
                            .unwrap_or(DiagnosticSeverity::Information);
                        let source = item
                            .get("source")
                            .and_then(|s| s.as_str())
                            .unwrap_or("configured-rules")
                            .to_string();
                        let range = Range {
                            start: Position {
                                line: 0,
                                character: 0,
                            },
                            end: Position {
                                line: 0,
                                character: 0,
                            },
                        };

                        Some(Diagnostic {
                            range,
                            severity,
                            message,
                            source,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(DiagnosticsRule { pattern, diagnostics })
    }

    /// Parse a hover rule from YAML/JSON
    fn parse_hover_rule(value: &serde_yaml::Value) -> IdeResult<HoverRule> {
        let pattern = value
            .get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| IdeError::config_error("Hover rule missing 'pattern' field"))?
            .to_string();

        let contents = value
            .get("contents")
            .and_then(|v| v.as_str())
            .ok_or_else(|| IdeError::config_error("Hover rule missing 'contents' field"))?
            .to_string();

        let hover = Hover {
            contents,
            range: None,
        };

        Ok(HoverRule { pattern, hover })
    }

    /// Parse a definition rule from YAML/JSON
    fn parse_definition_rule(value: &serde_yaml::Value) -> IdeResult<DefinitionRule> {
        let pattern = value
            .get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| IdeError::config_error("Definition rule missing 'pattern' field"))?
            .to_string();

        let file_path = value
            .get("file")
            .and_then(|v| v.as_str())
            .ok_or_else(|| IdeError::config_error("Definition rule missing 'file' field"))?
            .to_string();

        let line = value
            .get("line")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        let character = value
            .get("character")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        let location = Location {
            file_path,
            range: Range {
                start: Position { line, character },
                end: Position { line, character },
            },
        };

        Ok(DefinitionRule { pattern, location })
    }

    /// Parse completion kind from string
    fn parse_completion_kind(kind_str: &str) -> Option<CompletionItemKind> {
        match kind_str.to_lowercase().as_str() {
            "text" => Some(CompletionItemKind::Text),
            "method" => Some(CompletionItemKind::Method),
            "function" => Some(CompletionItemKind::Function),
            "constructor" => Some(CompletionItemKind::Constructor),
            "field" => Some(CompletionItemKind::Field),
            "variable" => Some(CompletionItemKind::Variable),
            "class" => Some(CompletionItemKind::Class),
            "interface" => Some(CompletionItemKind::Interface),
            "module" => Some(CompletionItemKind::Module),
            "property" => Some(CompletionItemKind::Property),
            "unit" => Some(CompletionItemKind::Unit),
            "value" => Some(CompletionItemKind::Value),
            "enum" => Some(CompletionItemKind::Enum),
            "keyword" => Some(CompletionItemKind::Keyword),
            "snippet" => Some(CompletionItemKind::Snippet),
            "color" => Some(CompletionItemKind::Color),
            "file" => Some(CompletionItemKind::File),
            "reference" => Some(CompletionItemKind::Reference),
            "folder" => Some(CompletionItemKind::Folder),
            "enummember" => Some(CompletionItemKind::EnumMember),
            "constant" => Some(CompletionItemKind::Constant),
            "struct" => Some(CompletionItemKind::Struct),
            "event" => Some(CompletionItemKind::Event),
            "operator" => Some(CompletionItemKind::Operator),
            "typeparameter" => Some(CompletionItemKind::TypeParameter),
            _ => None,
        }
    }

    /// Parse severity from string
    fn parse_severity(severity_str: &str) -> Option<DiagnosticSeverity> {
        match severity_str.to_lowercase().as_str() {
            "error" => Some(DiagnosticSeverity::Error),
            "warning" => Some(DiagnosticSeverity::Warning),
            "information" | "info" => Some(DiagnosticSeverity::Information),
            "hint" => Some(DiagnosticSeverity::Hint),
            _ => None,
        }
    }

    /// Check if context matches pattern (simple substring matching)
    fn matches_pattern(&self, pattern: &str, context: &str) -> bool {
        context.contains(pattern)
    }
}

#[async_trait]
impl IdeProvider for ConfiguredRulesProvider {
    async fn get_completions(&self, params: &CompletionParams) -> IdeResult<Vec<CompletionItem>> {
        debug!(
            "Getting completions from configured rules for language: {}",
            self.language
        );

        let mut completions = Vec::new();

        for rule in &self.completion_rules {
            if self.matches_pattern(&rule.pattern, &params.context) {
                completions.extend(rule.suggestions.clone());
            }
        }

        Ok(completions)
    }

    async fn get_diagnostics(&self, params: &DiagnosticsParams) -> IdeResult<Vec<Diagnostic>> {
        debug!(
            "Getting diagnostics from configured rules for language: {}",
            self.language
        );

        let mut diagnostics = Vec::new();

        for rule in &self.diagnostics_rules {
            if self.matches_pattern(&rule.pattern, &params.source) {
                diagnostics.extend(rule.diagnostics.clone());
            }
        }

        Ok(diagnostics)
    }

    async fn get_hover(&self, _params: &HoverParams) -> IdeResult<Option<Hover>> {
        debug!(
            "Getting hover from configured rules for language: {}",
            self.language
        );

        // For now, return None
        // In a full implementation, this would match patterns and return hover info
        Ok(None)
    }

    async fn get_definition(&self, _params: &DefinitionParams) -> IdeResult<Option<Location>> {
        debug!(
            "Getting definition from configured rules for language: {}",
            self.language
        );

        // For now, return None
        // In a full implementation, this would match patterns and return definition location
        Ok(None)
    }

    fn is_available(&self, language: &str) -> bool {
        language == self.language
    }

    fn name(&self) -> &str {
        "configured-rules"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_completion_kind() {
        assert_eq!(
            ConfiguredRulesProvider::parse_completion_kind("function"),
            Some(CompletionItemKind::Function)
        );
        assert_eq!(
            ConfiguredRulesProvider::parse_completion_kind("class"),
            Some(CompletionItemKind::Class)
        );
        assert_eq!(
            ConfiguredRulesProvider::parse_completion_kind("unknown"),
            None
        );
    }

    #[test]
    fn test_parse_severity() {
        assert_eq!(
            ConfiguredRulesProvider::parse_severity("error"),
            Some(DiagnosticSeverity::Error)
        );
        assert_eq!(
            ConfiguredRulesProvider::parse_severity("warning"),
            Some(DiagnosticSeverity::Warning)
        );
        assert_eq!(
            ConfiguredRulesProvider::parse_severity("unknown"),
            None
        );
    }

    #[test]
    fn test_new_provider() {
        let provider = ConfiguredRulesProvider::new("rust".to_string());
        assert_eq!(provider.language, "rust");
        assert!(provider.completion_rules.is_empty());
    }

    #[test]
    fn test_add_completion_rule() {
        let mut provider = ConfiguredRulesProvider::new("rust".to_string());
        let rule = CompletionRule {
            pattern: "fn ".to_string(),
            suggestions: vec![CompletionItem {
                label: "test".to_string(),
                kind: CompletionItemKind::Function,
                detail: None,
                documentation: None,
                insert_text: "test()".to_string(),
            }],
        };

        provider.add_completion_rule(rule);
        assert_eq!(provider.completion_rules.len(), 1);
    }

    #[test]
    fn test_matches_pattern() {
        let provider = ConfiguredRulesProvider::new("rust".to_string());
        assert!(provider.matches_pattern("fn ", "fn test() {"));
        assert!(!provider.matches_pattern("fn ", "let x = 5;"));
    }

    #[tokio::test]
    async fn test_get_completions_with_matching_rule() {
        let mut provider = ConfiguredRulesProvider::new("rust".to_string());
        let rule = CompletionRule {
            pattern: "fn ".to_string(),
            suggestions: vec![CompletionItem {
                label: "test".to_string(),
                kind: CompletionItemKind::Function,
                detail: None,
                documentation: None,
                insert_text: "test()".to_string(),
            }],
        };

        provider.add_completion_rule(rule);

        let params = CompletionParams {
            language: "rust".to_string(),
            file_path: "src/main.rs".to_string(),
            position: Position {
                line: 10,
                character: 5,
            },
            context: "fn test".to_string(),
        };

        let result = provider.get_completions(&params).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_get_diagnostics_with_matching_rule() {
        let mut provider = ConfiguredRulesProvider::new("rust".to_string());
        let rule = DiagnosticsRule {
            pattern: "unused".to_string(),
            diagnostics: vec![Diagnostic {
                range: Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: 0,
                        character: 0,
                    },
                },
                severity: DiagnosticSeverity::Warning,
                message: "unused variable".to_string(),
                source: "configured-rules".to_string(),
            }],
        };

        provider.add_diagnostics_rule(rule);

        let params = DiagnosticsParams {
            language: "rust".to_string(),
            file_path: "src/main.rs".to_string(),
            source: "let unused = 5;".to_string(),
        };

        let result = provider.get_diagnostics(&params).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_is_available() {
        let provider = ConfiguredRulesProvider::new("rust".to_string());
        assert!(provider.is_available("rust"));
        assert!(!provider.is_available("typescript"));
    }
}
