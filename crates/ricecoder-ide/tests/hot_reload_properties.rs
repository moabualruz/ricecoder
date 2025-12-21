//! Property-based tests for hot-reload functionality
//!
//! **Feature: Hot-Reload Configuration**
//! **Property 3: Hot-Reload Configuration**
//! **Validates: Requirements 2.7, 2.9, 2.10**
//!
//! Tests that configuration reloads without restart and provider switching
//! occurs automatically on availability changes.

use proptest::prelude::*;
use ricecoder_ide::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Generate random configuration paths
fn arb_config_path() -> impl Strategy<Value = String> {
    r"/tmp/config_[a-z0-9]{8}\.yaml"
}

/// Generate random LSP server configurations
fn arb_lsp_server_config() -> impl Strategy<Value = types::LspServerConfig> {
    (
        r"[a-z]{3,10}",
        r"[a-z\-]{3,15}",
        prop::collection::vec(r"[a-z\-]{1,10}", 0..3),
        1000u64..30000u64,
    )
        .prop_map(
            |(language, command, args, timeout)| types::LspServerConfig {
                language,
                command,
                args,
                timeout_ms: timeout,
            },
        )
}

/// Generate random external LSP configurations
#[allow(dead_code)]
fn arb_external_lsp_config() -> impl Strategy<Value = types::ExternalLspConfig> {
    (
        prop::bool::ANY,
        prop::collection::hash_map(r"[a-z]{3,10}", arb_lsp_server_config(), 0..5),
        1000u64..10000u64,
    )
        .prop_map(|(enabled, servers, interval)| types::ExternalLspConfig {
            enabled,
            servers,
            health_check_interval_ms: interval,
        })
}

/// Generate random IDE integration configurations
#[allow(dead_code)]
fn arb_ide_config() -> impl Strategy<Value = types::IdeIntegrationConfig> {
    (arb_external_lsp_config(), prop::bool::ANY).prop_map(|(external_lsp, builtin_enabled)| {
        types::IdeIntegrationConfig {
            vscode: None,
            terminal: None,
            providers: types::ProviderChainConfig {
                external_lsp,
                configured_rules: None,
                builtin_providers: types::BuiltinProvidersConfig {
                    enabled: builtin_enabled,
                    languages: vec!["rust".to_string(), "typescript".to_string()],
                },
            },
        }
    })
}

proptest! {
    /// **Feature: Hot-Reload Configuration, Property 3: Hot-Reload Configuration**
    /// **Validates: Requirements 2.7, 2.9, 2.10**
    ///
    /// Test that configuration hot-reload manager can be created and callbacks registered
    /// without errors. This validates that the hot-reload infrastructure is sound.
    #[test]
    fn prop_hot_reload_manager_creation_succeeds(config_path in arb_config_path()) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let manager = HotReloadManager::new(&config_path);

            // Register callbacks should succeed
            let config_callback = Box::new(|| {});
            assert!(manager.on_config_change(config_callback).await.is_ok());

            let provider_callback = Box::new(|_: &str, _: bool| {});
            assert!(manager.on_provider_availability_change(provider_callback).await.is_ok());
        });
    }

    /// **Feature: Hot-Reload Configuration, Property 3: Hot-Reload Configuration**
    /// **Validates: Requirements 2.7, 2.9, 2.10**
    ///
    /// Test that configuration change notifications are delivered to all registered callbacks.
    /// This validates that the callback mechanism works correctly.
    #[test]
    fn prop_config_change_callbacks_are_invoked(config_path in arb_config_path()) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let manager = HotReloadManager::new(&config_path);

            // Register multiple callbacks
            let call_count1 = Arc::new(AtomicUsize::new(0));
            let call_count2 = Arc::new(AtomicUsize::new(0));

            let count1 = call_count1.clone();
            manager.on_config_change(Box::new(move || {
                count1.fetch_add(1, Ordering::SeqCst);
            })).await.unwrap();

            let count2 = call_count2.clone();
            manager.on_config_change(Box::new(move || {
                count2.fetch_add(1, Ordering::SeqCst);
            })).await.unwrap();

            // Notify changes
            manager.notify_config_changed().await.unwrap();

            // Both callbacks should have been invoked
            assert_eq!(call_count1.load(Ordering::SeqCst), 1);
            assert_eq!(call_count2.load(Ordering::SeqCst), 1);
        });
    }

    /// **Feature: Hot-Reload Configuration, Property 3: Hot-Reload Configuration**
    /// **Validates: Requirements 2.7, 2.9, 2.10**
    ///
    /// Test that provider availability change notifications are delivered correctly.
    /// This validates that provider availability changes are properly communicated.
    #[test]
    fn prop_provider_availability_callbacks_are_invoked(
        config_path in arb_config_path(),
        language in r"[a-z]{3,10}",
        available in prop::bool::ANY
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let manager = HotReloadManager::new(&config_path);

            let call_count = Arc::new(AtomicUsize::new(0));
            let received_language = Arc::new(std::sync::Mutex::new(String::new()));
            let received_available = Arc::new(std::sync::Mutex::new(false));

            let count = call_count.clone();
            let lang = received_language.clone();
            let avail = received_available.clone();

            manager.on_provider_availability_change(Box::new(move |l: &str, a: bool| {
                count.fetch_add(1, Ordering::SeqCst);
                *lang.lock().unwrap() = l.to_string();
                *avail.lock().unwrap() = a;
            })).await.unwrap();

            // Notify availability change
            manager.notify_provider_availability_changed(&language, available).await.unwrap();

            // Callback should have been invoked with correct parameters
            assert_eq!(call_count.load(Ordering::SeqCst), 1);
            assert_eq!(*received_language.lock().unwrap(), language);
            assert_eq!(*received_available.lock().unwrap(), available);
        });
    }

    /// **Feature: Hot-Reload Configuration, Property 3: Hot-Reload Configuration**
    /// **Validates: Requirements 2.7, 2.9, 2.10**
    ///
    /// Test that LSP monitor can be created with various server configurations
    /// and health checks can be initiated without errors.
    #[test]
    fn prop_lsp_monitor_creation_succeeds(servers in prop::collection::hash_map(
        r"[a-z]{3,10}",
        arb_lsp_server_config(),
        0..5
    )) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let monitor = LspMonitor::new(servers.clone());

            // Should be able to register callbacks
            let callback = Arc::new(|_: &str, _: bool| {});
            assert!(monitor.on_availability_changed(callback).await.is_ok());

            // Available languages should match configured servers
            let languages = monitor.available_languages();
            assert_eq!(languages.len(), servers.len());
        });
    }

    /// **Feature: Hot-Reload Configuration, Property 3: Hot-Reload Configuration**
    /// **Validates: Requirements 2.7, 2.9, 2.10**
    ///
    /// Test that configuration hot-reload coordinator can be created and
    /// configuration can be loaded and retrieved without errors.
    #[test]
    fn prop_config_coordinator_creation_succeeds(config_path in arb_config_path()) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let coordinator = ConfigHotReloadCoordinator::new(&config_path);

            // Should be able to get default configuration
            let config = coordinator.get_config().await;
            assert!(config.providers.external_lsp.enabled || config.providers.builtin_providers.enabled);
        });
    }

    /// **Feature: Hot-Reload Configuration, Property 3: Hot-Reload Configuration**
    /// **Validates: Requirements 2.7, 2.9, 2.10**
    ///
    /// Test that multiple callbacks can be registered and all are invoked
    /// when configuration changes occur.
    #[test]
    fn prop_multiple_callbacks_all_invoked(
        config_path in arb_config_path(),
        num_callbacks in 1usize..10
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let manager = HotReloadManager::new(&config_path);

            let call_counts: Vec<Arc<AtomicUsize>> = (0..num_callbacks)
                .map(|_| Arc::new(AtomicUsize::new(0)))
                .collect();

            // Register all callbacks
            for count in &call_counts {
                let c = count.clone();
                manager.on_config_change(Box::new(move || {
                    c.fetch_add(1, Ordering::SeqCst);
                })).await.unwrap();
            }

            // Notify change
            manager.notify_config_changed().await.unwrap();

            // All callbacks should have been invoked exactly once
            for count in &call_counts {
                assert_eq!(count.load(Ordering::SeqCst), 1);
            }
        });
    }

    /// **Feature: Hot-Reload Configuration, Property 3: Hot-Reload Configuration**
    /// **Validates: Requirements 2.7, 2.9, 2.10**
    ///
    /// Test that provider availability changes are correctly tracked and
    /// callbacks receive the correct language and availability status.
    #[test]
    fn prop_provider_availability_tracking(
        config_path in arb_config_path(),
        languages in prop::collection::vec(r"[a-z]{3,10}", 1..5),
        availabilities in prop::collection::vec(prop::bool::ANY, 1..5)
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let manager = HotReloadManager::new(&config_path);

            let received_changes = Arc::new(std::sync::Mutex::new(Vec::new()));
            let changes = received_changes.clone();

            manager.on_provider_availability_change(Box::new(move |lang: &str, avail: bool| {
                changes.lock().unwrap().push((lang.to_string(), avail));
            })).await.unwrap();

            // Notify multiple changes
            for (i, lang) in languages.iter().enumerate() {
                let avail = availabilities.get(i).copied().unwrap_or(false);
                manager.notify_provider_availability_changed(lang, avail).await.unwrap();
            }

            // All changes should have been recorded
            let recorded = received_changes.lock().unwrap();
            assert_eq!(recorded.len(), languages.len());

            for (i, (recorded_lang, recorded_avail)) in recorded.iter().enumerate() {
                assert_eq!(recorded_lang, &languages[i]);
                assert_eq!(*recorded_avail, availabilities.get(i).copied().unwrap_or(false));
            }
        });
    }

    /// **Feature: Hot-Reload Configuration, Property 3: Hot-Reload Configuration**
    /// **Validates: Requirements 2.7, 2.9, 2.10**
    ///
    /// Test that configuration coordinator can be cloned and maintains
    /// consistent state across clones.
    #[test]
    fn prop_config_coordinator_clone_consistency(config_path in arb_config_path()) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let coordinator = ConfigHotReloadCoordinator::new(&config_path);
            let cloned = coordinator.clone();

            // Both should have the same configuration
            let config1 = coordinator.get_config().await;
            let config2 = cloned.get_config().await;

            assert_eq!(config1.providers.external_lsp.enabled, config2.providers.external_lsp.enabled);
            assert_eq!(config1.providers.builtin_providers.enabled, config2.providers.builtin_providers.enabled);
        });
    }
}
