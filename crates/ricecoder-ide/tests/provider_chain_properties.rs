//! Property-based tests for provider chain
//!
//! **Feature: ricecoder-ide, Property 1: External LSP-First Provider Chain**
//! **Validates: Requirements 2.1, 2.2, 2.3, 2.4, 2.5**

use proptest::prelude::*;
use ricecoder_ide::*;
use std::sync::Arc;

/// Mock provider for testing
#[allow(dead_code)]
struct MockProvider {
    name: String,
    language: String,
    priority: u32,
}

#[async_trait::async_trait]
impl IdeProvider for MockProvider {
    async fn get_completions(
        &self,
        _params: &CompletionParams,
    ) -> Result<Vec<CompletionItem>, IdeError> {
        Ok(vec![CompletionItem {
            label: format!("{}-completion", self.name),
            kind: CompletionItemKind::Function,
            detail: None,
            documentation: None,
            insert_text: format!("{}-completion", self.name),
        }])
    }

    async fn get_diagnostics(
        &self,
        _params: &DiagnosticsParams,
    ) -> Result<Vec<Diagnostic>, IdeError> {
        Ok(vec![])
    }

    async fn get_hover(&self, _params: &HoverParams) -> Result<Option<Hover>, IdeError> {
        Ok(None)
    }

    async fn get_definition(&self, _params: &DefinitionParams) -> Result<Option<Location>, IdeError> {
        Ok(None)
    }

    fn is_available(&self, language: &str) -> bool {
        language == self.language
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Strategy for generating provider availability scenarios
fn provider_availability_strategy() -> impl Strategy<Value = (bool, bool, bool, bool)> {
    (any::<bool>(), any::<bool>(), any::<bool>(), any::<bool>())
}

/// Strategy for generating languages
fn language_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("rust".to_string()),
        Just("typescript".to_string()),
        Just("python".to_string()),
        Just("go".to_string()),
        Just("java".to_string()),
    ]
}

proptest! {
    /// Property 1: External LSP-First Provider Chain
    /// For any IDE feature request (completion, diagnostics, hover, definition),
    /// the system SHALL attempt to use external LSP servers first, then fall back
    /// through the provider chain in order: LSP → Configured Rules → Built-in Providers → Generic Features.
    #[test]
    fn prop_lsp_first_provider_chain(
        (has_lsp, has_rules, has_builtin, _has_generic) in provider_availability_strategy(),
        language in language_strategy()
    ) {
        // Create a provider registry
        let generic = Arc::new(MockProvider {
            name: "generic".to_string(),
            language: language.clone(),
            priority: 4,
        });
        let mut registry = ProviderRegistry::new(generic);

        // Register providers based on availability
        if has_lsp {
            let lsp = Arc::new(MockProvider {
                name: "lsp".to_string(),
                language: language.clone(),
                priority: 1,
            });
            registry.register_lsp_provider(language.clone(), lsp);
        }

        if has_rules {
            let rules = Arc::new(MockProvider {
                name: "rules".to_string(),
                language: language.clone(),
                priority: 2,
            });
            registry.register_configured_provider(language.clone(), rules);
        }

        if has_builtin {
            let builtin = Arc::new(MockProvider {
                name: "builtin".to_string(),
                language: language.clone(),
                priority: 3,
            });
            registry.register_builtin_provider(language.clone(), builtin);
        }

        // Get the provider for the language
        let provider = registry.get_provider(&language);

        // Verify the correct provider is selected based on priority
        if has_lsp {
            prop_assert_eq!(provider.name(), "lsp", "LSP provider should be selected first");
        } else if has_rules {
            prop_assert_eq!(provider.name(), "rules", "Rules provider should be selected second");
        } else if has_builtin {
            prop_assert_eq!(provider.name(), "builtin", "Built-in provider should be selected third");
        } else {
            prop_assert_eq!(provider.name(), "generic", "Generic provider should be selected last");
        }
    }

    /// Property 1 (continued): Verify provider chain for multiple languages
    /// The provider chain should work correctly for any language combination.
    #[test]
    fn prop_provider_chain_multiple_languages(
        (has_lsp_rust, has_rules_rust, has_builtin_rust, _) in provider_availability_strategy(),
        (has_lsp_ts, has_rules_ts, has_builtin_ts, _) in provider_availability_strategy(),
    ) {
        let generic = Arc::new(MockProvider {
            name: "generic".to_string(),
            language: "generic".to_string(),
            priority: 4,
        });
        let mut registry = ProviderRegistry::new(generic);

        // Register Rust providers
        if has_lsp_rust {
            registry.register_lsp_provider(
                "rust".to_string(),
                Arc::new(MockProvider {
                    name: "rust-lsp".to_string(),
                    language: "rust".to_string(),
                    priority: 1,
                }),
            );
        }
        if has_rules_rust {
            registry.register_configured_provider(
                "rust".to_string(),
                Arc::new(MockProvider {
                    name: "rust-rules".to_string(),
                    language: "rust".to_string(),
                    priority: 2,
                }),
            );
        }
        if has_builtin_rust {
            registry.register_builtin_provider(
                "rust".to_string(),
                Arc::new(MockProvider {
                    name: "rust-builtin".to_string(),
                    language: "rust".to_string(),
                    priority: 3,
                }),
            );
        }

        // Register TypeScript providers
        if has_lsp_ts {
            registry.register_lsp_provider(
                "typescript".to_string(),
                Arc::new(MockProvider {
                    name: "ts-lsp".to_string(),
                    language: "typescript".to_string(),
                    priority: 1,
                }),
            );
        }
        if has_rules_ts {
            registry.register_configured_provider(
                "typescript".to_string(),
                Arc::new(MockProvider {
                    name: "ts-rules".to_string(),
                    language: "typescript".to_string(),
                    priority: 2,
                }),
            );
        }
        if has_builtin_ts {
            registry.register_builtin_provider(
                "typescript".to_string(),
                Arc::new(MockProvider {
                    name: "ts-builtin".to_string(),
                    language: "typescript".to_string(),
                    priority: 3,
                }),
            );
        }

        // Verify Rust provider selection
        let rust_provider = registry.get_provider("rust");
        if has_lsp_rust {
            prop_assert_eq!(rust_provider.name(), "rust-lsp");
        } else if has_rules_rust {
            prop_assert_eq!(rust_provider.name(), "rust-rules");
        } else if has_builtin_rust {
            prop_assert_eq!(rust_provider.name(), "rust-builtin");
        } else {
            prop_assert_eq!(rust_provider.name(), "generic");
        }

        // Verify TypeScript provider selection
        let ts_provider = registry.get_provider("typescript");
        if has_lsp_ts {
            prop_assert_eq!(ts_provider.name(), "ts-lsp");
        } else if has_rules_ts {
            prop_assert_eq!(ts_provider.name(), "ts-rules");
        } else if has_builtin_ts {
            prop_assert_eq!(ts_provider.name(), "ts-builtin");
        } else {
            prop_assert_eq!(ts_provider.name(), "generic");
        }
    }

    /// Property 1 (continued): Verify provider availability queries
    /// The system should correctly report provider availability for any language.
    #[test]
    fn prop_provider_availability_queries(
        (has_lsp, has_rules, has_builtin, _) in provider_availability_strategy(),
        language in language_strategy()
    ) {
        let generic = Arc::new(MockProvider {
            name: "generic".to_string(),
            language: language.clone(),
            priority: 4,
        });
        let mut registry = ProviderRegistry::new(generic);

        if has_lsp {
            registry.register_lsp_provider(
                language.clone(),
                Arc::new(MockProvider {
                    name: "lsp".to_string(),
                    language: language.clone(),
                    priority: 1,
                }),
            );
        }

        if has_rules {
            registry.register_configured_provider(
                language.clone(),
                Arc::new(MockProvider {
                    name: "rules".to_string(),
                    language: language.clone(),
                    priority: 2,
                }),
            );
        }

        if has_builtin {
            registry.register_builtin_provider(
                language.clone(),
                Arc::new(MockProvider {
                    name: "builtin".to_string(),
                    language: language.clone(),
                    priority: 3,
                }),
            );
        }

        // Check availability
        let is_available = registry.is_provider_available(&language);
        let expected_available = has_lsp || has_rules || has_builtin;
        prop_assert_eq!(is_available, expected_available, "Provider availability should match registration");
    }

    /// Property 1 (continued): Verify provider unregistration
    /// When a provider is unregistered, the system should fall back to the next provider.
    #[test]
    fn prop_provider_unregistration_fallback(
        language in language_strategy()
    ) {
        let generic = Arc::new(MockProvider {
            name: "generic".to_string(),
            language: language.clone(),
            priority: 4,
        });
        let mut registry = ProviderRegistry::new(generic);

        // Register all providers
        registry.register_lsp_provider(
            language.clone(),
            Arc::new(MockProvider {
                name: "lsp".to_string(),
                language: language.clone(),
                priority: 1,
            }),
        );
        registry.register_configured_provider(
            language.clone(),
            Arc::new(MockProvider {
                name: "rules".to_string(),
                language: language.clone(),
                priority: 2,
            }),
        );
        registry.register_builtin_provider(
            language.clone(),
            Arc::new(MockProvider {
                name: "builtin".to_string(),
                language: language.clone(),
                priority: 3,
            }),
        );

        // Verify LSP is selected
        let provider = registry.get_provider(&language);
        prop_assert_eq!(provider.name(), "lsp");

        // Unregister LSP
        registry.unregister_lsp_provider(&language);

        // Verify rules is now selected
        let provider = registry.get_provider(&language);
        prop_assert_eq!(provider.name(), "rules");

        // Unregister rules
        registry.unregister_configured_provider(&language);

        // Verify builtin is now selected
        let provider = registry.get_provider(&language);
        prop_assert_eq!(provider.name(), "builtin");

        // Unregister builtin
        registry.unregister_builtin_provider(&language);

        // Verify generic is now selected
        let provider = registry.get_provider(&language);
        prop_assert_eq!(provider.name(), "generic");
    }
}
